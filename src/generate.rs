use crate::levels;
use crate::sync_metadata;
use anyhow::{bail, Context, Result};
use gsnake_core::LevelDefinition;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn run_generate_levels_json(filter: Option<&str>, dry_run: bool, sync: bool) -> Result<()> {
    let levels_root = levels::find_levels_root()?;
    let playbacks_root = levels_root
        .parent()
        .map(|parent| parent.join("playbacks"))
        .unwrap_or_else(|| PathBuf::from("playbacks"));
    let difficulties = parse_filter(filter)?;

    // Run metadata sync if enabled (default behavior)
    if sync {
        eprintln!("Running metadata sync...");
        let difficulty_filter = if difficulties.len() == levels::DEFAULT_DIFFICULTIES.len() {
            None
        } else {
            Some(difficulties.join(","))
        };
        let summary = sync_metadata::sync_metadata_with_roots(
            &levels_root,
            &playbacks_root,
            difficulty_filter.as_deref(),
        )
        .with_context(|| "Metadata sync failed, aborting generate-levels-json")?;

        eprintln!("Sync completed:");
        eprintln!("  - Generated {} names", summary.names_generated);
        eprintln!(
            "  - Updated {} levels.toml files",
            summary.toml_files_updated
        );
        eprintln!("  - Created {} playbacks", summary.playbacks_created);
        eprintln!();
    }

    let mut aggregated: Vec<LevelDefinition> = Vec::new();

    for difficulty in difficulties {
        let levels_toml_path = levels_root.join(difficulty).join("levels.toml");
        if !levels_toml_path.exists() {
            continue;
        }

        let levels_toml = levels::read_levels_toml(&levels_toml_path)?;
        for entry in levels_toml.level {
            let file = match entry.file.as_deref() {
                Some(file) => file,
                None => continue,
            };
            let level_path = levels_root.join(difficulty).join(file);
            if !level_path.exists() {
                bail!("Level file not found: {}", level_path.display());
            }

            let mut level = load_level(&level_path)?;
            let difficulty_value = entry
                .difficulty
                .as_deref()
                .unwrap_or(difficulty)
                .to_string();
            level.difficulty = Some(difficulty_value);
            aggregated.push(level);
        }
    }

    if dry_run {
        return Ok(());
    }

    let output = serde_json::to_string_pretty(&aggregated)
        .with_context(|| "Failed to serialize aggregated levels JSON")?;
    println!("{output}");
    Ok(())
}

fn parse_filter(filter: Option<&str>) -> Result<Vec<&'static str>> {
    if let Some(raw) = filter {
        let mut selected = Vec::new();
        let requested: HashSet<String> = raw
            .split(',')
            .map(|item| item.trim().to_lowercase())
            .filter(|item| !item.is_empty())
            .collect();

        for difficulty in levels::DEFAULT_DIFFICULTIES {
            if requested.contains(difficulty) {
                selected.push(difficulty);
            }
        }

        if selected.is_empty() {
            bail!("Filter did not match any known difficulty (easy, medium, hard)");
        }
        return Ok(selected);
    }

    Ok(levels::DEFAULT_DIFFICULTIES.to_vec())
}

fn load_level(level_path: &PathBuf) -> Result<LevelDefinition> {
    let contents = std::fs::read_to_string(level_path)
        .with_context(|| format!("Failed to read level file: {}", level_path.display()))?;
    let level: LevelDefinition =
        serde_json::from_str(&contents).with_context(|| "Failed to parse level JSON")?;
    Ok(level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::{LevelMeta, LevelsToml};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_level_json(levels_dir: &Path, filename: &str, name: &str) -> Result<()> {
        fs::create_dir_all(levels_dir)?;
        let level_json = serde_json::json!({
            "id": 1,
            "name": name,
            "difficulty": "easy",
            "gridSize": { "width": 10, "height": 10 },
            "snake": [{ "x": 0, "y": 0 }],
            "obstacles": [],
            "food": [{ "x": 1, "y": 0 }],
            "exit": { "x": 5, "y": 5 },
            "snakeDirection": "East",
            "floatingFood": [],
            "fallingFood": [],
            "stones": [],
            "spikes": [],
            "totalFood": 1
        });
        fs::write(
            levels_dir.join(filename),
            serde_json::to_string_pretty(&level_json)?,
        )?;
        Ok(())
    }

    fn write_levels_toml(levels_dir: &Path, difficulty: &str, file: &str) -> Result<()> {
        let levels_toml = LevelsToml {
            level: vec![LevelMeta {
                id: Some(file.trim_end_matches(".json").to_string()),
                file: Some(file.to_string()),
                author: Some("gsnake".to_string()),
                solved: Some(true),
                difficulty: Some(difficulty.to_string()),
                tags: Some(vec![]),
                description: Some("Test level".to_string()),
            }],
        };
        let output = toml::to_string_pretty(&levels_toml)?;
        fs::write(levels_dir.join("levels.toml"), output)?;
        Ok(())
    }

    #[test]
    fn test_parse_filter_defaults_to_all_difficulties() -> Result<()> {
        assert_eq!(parse_filter(None)?, levels::DEFAULT_DIFFICULTIES.to_vec());
        Ok(())
    }

    #[test]
    fn test_parse_filter_is_case_insensitive_and_ordered() -> Result<()> {
        let filtered = parse_filter(Some(" hard , EASY "))?;
        assert_eq!(filtered, vec!["easy", "hard"]);
        Ok(())
    }

    #[test]
    fn test_parse_filter_unknown_value_fails() {
        let result = parse_filter(Some("legendary"));
        assert!(result.is_err());
        let error = result
            .expect_err("Expected invalid filter error")
            .to_string();
        assert!(error.contains("Filter did not match any known difficulty"));
    }

    #[test]
    fn test_run_generate_levels_json_success_from_package_directory() -> Result<()> {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new()?;
        let easy_dir = temp_dir.path().join("levels/easy");
        create_test_level_json(&easy_dir, "level_001.json", "Package Level")?;
        write_levels_toml(&easy_dir, "easy", "level_001.json")?;
        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());

        run_generate_levels_json(Some("easy"), true, false)
    }

    #[test]
    fn test_run_generate_levels_json_success_from_repo_root() -> Result<()> {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new()?;
        let easy_dir = temp_dir.path().join("gsnake-levels/levels/easy");
        create_test_level_json(&easy_dir, "level_001.json", "Nested Level")?;
        write_levels_toml(&easy_dir, "easy", "level_001.json")?;
        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());

        run_generate_levels_json(Some("easy"), true, false)
    }

    #[test]
    fn test_run_generate_levels_json_missing_level_file_fails() -> Result<()> {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new()?;
        let easy_dir = temp_dir.path().join("levels/easy");
        fs::create_dir_all(&easy_dir)?;
        write_levels_toml(&easy_dir, "easy", "missing_level.json")?;
        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());

        let result = run_generate_levels_json(Some("easy"), true, false);
        assert!(result.is_err());
        let error = result
            .expect_err("Expected missing level error")
            .to_string();
        assert!(error.contains("Level file not found"));
        Ok(())
    }

    #[test]
    fn test_run_generate_levels_json_with_sync_enabled() -> Result<()> {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new()?;
        fs::create_dir_all(temp_dir.path().join("levels/easy"))?;
        fs::create_dir_all(temp_dir.path().join("levels/medium"))?;
        fs::create_dir_all(temp_dir.path().join("levels/hard"))?;
        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());

        run_generate_levels_json(None, true, true)
    }
}
