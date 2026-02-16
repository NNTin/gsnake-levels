use crate::{levels, solver::solve_level_to_playback};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Result of playback generation for a single level
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlaybackResult {
    pub level_id: String,
    pub level_path: PathBuf,
    pub playback_path: PathBuf,
    pub solved: bool,
    pub error: Option<String>,
}

/// Generate playback for a single level file
#[allow(dead_code)]
pub fn generate_playback_for_level(
    level_path: &Path,
    playback_path: &Path,
    max_depth: usize,
) -> Result<PlaybackResult> {
    let level_id = level_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid level filename"))?
        .to_string();

    let playback_result = solve_level_to_playback(level_path, playback_path, max_depth);
    let (solved, error) = match playback_result {
        Ok(_) => (true, None),
        Err(err) => (false, Some(format!("{err:#}"))),
    };

    Ok(PlaybackResult {
        level_id,
        level_path: level_path.to_path_buf(),
        playback_path: playback_path.to_path_buf(),
        solved,
        error,
    })
}

/// Generate playbacks for all levels in a difficulty directory
#[allow(dead_code)]
pub fn generate_playbacks_for_difficulty(
    levels_dir: &Path,
    playbacks_dir: &Path,
    max_depth: usize,
) -> Result<Vec<PlaybackResult>> {
    let mut results = Vec::new();
    let mut level_paths = Vec::new();

    // Scan for JSON files
    let entries = fs::read_dir(levels_dir)
        .with_context(|| format!("Failed to read directory: {}", levels_dir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            level_paths.push(path);
        }
    }

    level_paths.sort();

    for path in level_paths {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

        let playback_path = playbacks_dir.join(filename);

        match generate_playback_for_level(&path, &playback_path, max_depth) {
            Ok(result) => {
                if !result.solved {
                    eprintln!(
                        "Warning: Failed to solve level {} - {}",
                        result.level_id,
                        result.error.as_deref().unwrap_or("unknown error")
                    );
                }
                results.push(result);
            },
            Err(e) => {
                eprintln!("Error processing level {}: {}", filename, e);
            },
        }
    }

    Ok(results)
}

/// Generate playbacks for all difficulty levels (easy, medium, hard)
#[allow(dead_code)]
pub fn generate_all_playbacks(
    levels_root: &Path,
    playbacks_root: &Path,
    max_depth: usize,
) -> Result<Vec<PlaybackResult>> {
    let mut all_results = Vec::new();

    for difficulty in ["easy", "medium", "hard"] {
        let levels_dir = levels_root.join(difficulty);
        let playbacks_dir = playbacks_root.join(difficulty);

        if levels_dir.exists() {
            let results = generate_playbacks_for_difficulty(&levels_dir, &playbacks_dir, max_depth)
                .with_context(|| format!("Failed to generate playbacks for {}", difficulty))?;
            all_results.extend(results);
        }
    }

    Ok(all_results)
}

/// Get lists of solved and unsolved level IDs
#[allow(dead_code)]
pub fn get_solved_unsolved_lists(results: &[PlaybackResult]) -> (Vec<String>, Vec<String>) {
    let mut solved = Vec::new();
    let mut unsolved = Vec::new();

    for result in results {
        if result.solved {
            solved.push(result.level_id.clone());
        } else {
            unsolved.push(result.level_id.clone());
        }
    }

    (solved, unsolved)
}

/// Update levels.toml solved status based on playback generation results
#[allow(dead_code)]
pub fn update_solved_status_from_results(results: &[PlaybackResult]) -> Result<()> {
    for result in results {
        levels::update_solved_status(&result.level_path, result.solved).with_context(|| {
            format!(
                "Failed to update solved status for level: {}",
                result.level_id
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn first_easy_level_fixture() -> PathBuf {
        let mut fixtures: Vec<PathBuf> = fs::read_dir("levels/easy")
            .unwrap()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                (path.extension().and_then(|ext| ext.to_str()) == Some("json")).then_some(path)
            })
            .collect();
        fixtures.sort();
        fixtures.into_iter().next().expect("Expected easy fixture")
    }

    #[test]
    fn test_generate_playback_for_level_writes_compatible_json() {
        let temp_dir = TempDir::new().unwrap();
        let level_path = first_easy_level_fixture();
        let playback_path = temp_dir.path().join("playbacks/level_001.json");

        let result = generate_playback_for_level(&level_path, &playback_path, 50).unwrap();
        assert!(result.solved);
        assert!(result.error.is_none());
        assert!(playback_path.exists());

        let playback_content = fs::read_to_string(&playback_path).unwrap();
        let steps: Vec<Value> = serde_json::from_str(&playback_content).unwrap();
        assert!(!steps.is_empty());
        for step in steps {
            assert!(step.get("key").and_then(Value::as_str).is_some());
            assert_eq!(step.get("delay_ms").and_then(Value::as_u64), Some(200));
        }
    }

    #[test]
    fn test_generate_playback_for_level_returns_unsolved_on_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let level_path = temp_dir.path().join("broken_level.json");
        let playback_path = temp_dir.path().join("playbacks/broken_level.json");
        fs::write(&level_path, "{not-json}").unwrap();

        let result = generate_playback_for_level(&level_path, &playback_path, 50).unwrap();
        assert!(!result.solved);
        let error = result.error.expect("Expected error message");
        assert!(error.contains("Failed to parse level JSON"));
        assert!(!playback_path.exists());
    }

    #[test]
    fn test_get_solved_unsolved_lists() {
        let results = vec![
            PlaybackResult {
                level_id: "level1".to_string(),
                level_path: PathBuf::from("level1.json"),
                playback_path: PathBuf::from("level1-playback.json"),
                solved: true,
                error: None,
            },
            PlaybackResult {
                level_id: "level2".to_string(),
                level_path: PathBuf::from("level2.json"),
                playback_path: PathBuf::from("level2-playback.json"),
                solved: false,
                error: Some("No solution found".to_string()),
            },
            PlaybackResult {
                level_id: "level3".to_string(),
                level_path: PathBuf::from("level3.json"),
                playback_path: PathBuf::from("level3-playback.json"),
                solved: true,
                error: None,
            },
        ];

        let (solved, unsolved) = get_solved_unsolved_lists(&results);

        assert_eq!(solved.len(), 2);
        assert_eq!(unsolved.len(), 1);
        assert!(solved.contains(&"level1".to_string()));
        assert!(solved.contains(&"level3".to_string()));
        assert!(unsolved.contains(&"level2".to_string()));
    }

    #[test]
    fn test_get_solved_unsolved_lists_empty() {
        let results = vec![];
        let (solved, unsolved) = get_solved_unsolved_lists(&results);

        assert_eq!(solved.len(), 0);
        assert_eq!(unsolved.len(), 0);
    }

    #[test]
    fn test_get_solved_unsolved_lists_all_solved() {
        let results = vec![
            PlaybackResult {
                level_id: "level1".to_string(),
                level_path: PathBuf::from("level1.json"),
                playback_path: PathBuf::from("level1-playback.json"),
                solved: true,
                error: None,
            },
            PlaybackResult {
                level_id: "level2".to_string(),
                level_path: PathBuf::from("level2.json"),
                playback_path: PathBuf::from("level2-playback.json"),
                solved: true,
                error: None,
            },
        ];

        let (solved, unsolved) = get_solved_unsolved_lists(&results);

        assert_eq!(solved.len(), 2);
        assert_eq!(unsolved.len(), 0);
    }

    #[test]
    fn test_get_solved_unsolved_lists_all_unsolved() {
        let results = vec![
            PlaybackResult {
                level_id: "level1".to_string(),
                level_path: PathBuf::from("level1.json"),
                playback_path: PathBuf::from("level1-playback.json"),
                solved: false,
                error: Some("No solution".to_string()),
            },
            PlaybackResult {
                level_id: "level2".to_string(),
                level_path: PathBuf::from("level2.json"),
                playback_path: PathBuf::from("level2-playback.json"),
                solved: false,
                error: Some("Too complex".to_string()),
            },
        ];

        let (solved, unsolved) = get_solved_unsolved_lists(&results);

        assert_eq!(solved.len(), 0);
        assert_eq!(unsolved.len(), 2);
    }

    #[test]
    fn test_generate_playbacks_for_difficulty_no_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let levels_dir = temp_dir.path().join("levels");
        let playbacks_dir = temp_dir.path().join("playbacks");

        fs::create_dir_all(&levels_dir).unwrap();
        fs::create_dir_all(&playbacks_dir).unwrap();

        // Create a non-JSON file
        fs::write(levels_dir.join("readme.txt"), "test").unwrap();

        let results = generate_playbacks_for_difficulty(&levels_dir, &playbacks_dir, 500).unwrap();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_generate_all_playbacks_missing_difficulty_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let levels_root = temp_dir.path().join("levels");
        let playbacks_root = temp_dir.path().join("playbacks");

        // Don't create difficulty directories

        let results = generate_all_playbacks(&levels_root, &playbacks_root, 500).unwrap();

        // Should succeed but return empty results
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_update_solved_status_from_results() {
        use crate::levels::{LevelMeta, LevelsToml};

        let temp_dir = TempDir::new().unwrap();
        let levels_dir = temp_dir.path().join("levels");
        fs::create_dir_all(&levels_dir).unwrap();

        // Create test level files
        let level1_path = levels_dir.join("level1.json");
        let level2_path = levels_dir.join("level2.json");
        fs::write(&level1_path, "{}").unwrap();
        fs::write(&level2_path, "{}").unwrap();

        // Create initial levels.toml with both levels marked as solved=true
        let levels_toml = LevelsToml {
            level: vec![
                LevelMeta {
                    id: Some("level1".to_string()),
                    file: Some("level1.json".to_string()),
                    author: Some("gsnake".to_string()),
                    solved: Some(true),
                    difficulty: Some("easy".to_string()),
                    tags: Some(vec![]),
                    description: Some("Level 1".to_string()),
                },
                LevelMeta {
                    id: Some("level2".to_string()),
                    file: Some("level2.json".to_string()),
                    author: Some("gsnake".to_string()),
                    solved: Some(true),
                    difficulty: Some("easy".to_string()),
                    tags: Some(vec![]),
                    description: Some("Level 2".to_string()),
                },
            ],
        };

        let toml_path = levels_dir.join("levels.toml");
        let toml_content = toml::to_string_pretty(&levels_toml).unwrap();
        fs::write(&toml_path, toml_content).unwrap();

        // Create playback results with mixed solved/unsolved
        let results = vec![
            PlaybackResult {
                level_id: "level1".to_string(),
                level_path: level1_path,
                playback_path: PathBuf::from("level1-playback.json"),
                solved: true,
                error: None,
            },
            PlaybackResult {
                level_id: "level2".to_string(),
                level_path: level2_path,
                playback_path: PathBuf::from("level2-playback.json"),
                solved: false,
                error: Some("No solution found".to_string()),
            },
        ];

        // Update solved status from results
        update_solved_status_from_results(&results).unwrap();

        // Read back the levels.toml and verify
        let updated_content = fs::read_to_string(&toml_path).unwrap();
        let updated_toml: LevelsToml = toml::from_str(&updated_content).unwrap();

        // level1 should still be solved=true
        let level1_entry = updated_toml
            .level
            .iter()
            .find(|l| l.file.as_deref() == Some("level1.json"))
            .unwrap();
        assert_eq!(level1_entry.solved, Some(true));

        // level2 should now be solved=false
        let level2_entry = updated_toml
            .level
            .iter()
            .find(|l| l.file.as_deref() == Some("level2.json"))
            .unwrap();
        assert_eq!(level2_entry.solved, Some(false));
    }

    #[test]
    fn test_update_solved_status_from_results_empty() {
        let results = vec![];
        // Should succeed with empty results
        update_solved_status_from_results(&results).unwrap();
    }
}
