use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
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

    // Ensure playback directory exists
    if let Some(parent) = playback_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Run solve_level binary
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("solve_level")
        .arg("--")
        .arg(level_path)
        .arg(playback_path)
        .arg("--max-depth")
        .arg(max_depth.to_string())
        .output()
        .context("Failed to execute solve_level binary")?;

    let solved = output.status.success();

    let error = if !solved {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Some(stderr.trim().to_string())
    } else {
        None
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

    // Scan for JSON files
    let entries = fs::read_dir(levels_dir)
        .with_context(|| format!("Failed to read directory: {}", levels_dir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
}
