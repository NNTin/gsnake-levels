use anyhow::{bail, Context, Result};
use gsnake_core::models::LevelDefinition;
use std::{
    fs,
    path::{Path, PathBuf},
    process,
};

use crate::levels::{read_levels_toml, DEFAULT_DIFFICULTIES};

/// Exit codes for validation failures
const EXIT_CODE_VALIDATION_ERROR: i32 = 1;
const EXIT_CODE_IO_ERROR: i32 = 2;
const EXIT_CODE_PARSE_ERROR: i32 = 3;

pub fn run_validate_levels_toml() -> Result<()> {
    let result = validate_all_levels_toml();

    match result {
        Ok(()) => {
            println!("✓ All levels.toml files are valid");
            Ok(())
        },
        Err(e) => {
            // Determine exit code based on error type
            let exit_code = if e.to_string().contains("No such file")
                || e.to_string().contains("Failed to read")
            {
                EXIT_CODE_IO_ERROR
            } else if e.to_string().contains("Failed to parse")
                || e.to_string().contains("invalid type")
            {
                EXIT_CODE_PARSE_ERROR
            } else {
                EXIT_CODE_VALIDATION_ERROR
            };

            eprintln!("Validation failed: {}", e);
            process::exit(exit_code);
        },
    }
}

fn validate_all_levels_toml() -> Result<()> {
    let levels_root =
        crate::levels::find_levels_root().context("Failed to find levels directory")?;

    for difficulty in DEFAULT_DIFFICULTIES {
        let difficulty_dir = levels_root.join(difficulty);
        validate_difficulty_levels_toml(&difficulty_dir, difficulty)?;
    }

    Ok(())
}

fn validate_difficulty_levels_toml(difficulty_dir: &Path, difficulty: &str) -> Result<()> {
    let levels_toml_path = difficulty_dir.join("levels.toml");

    // Check that levels.toml exists
    if !levels_toml_path.exists() {
        bail!(
            "levels.toml not found for difficulty '{}': {}",
            difficulty,
            levels_toml_path.display()
        );
    }

    // Parse levels.toml
    let levels_toml = read_levels_toml(&levels_toml_path).with_context(|| {
        format!(
            "Failed to parse levels.toml for difficulty '{}': {}",
            difficulty,
            levels_toml_path.display()
        )
    })?;

    // Validate each level entry
    for level_entry in &levels_toml.level {
        let file_name = level_entry.file.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "Missing 'file' field in levels.toml for difficulty '{}': {}",
                difficulty,
                levels_toml_path.display()
            )
        })?;

        let level_json_path = difficulty_dir.join(file_name);

        // Check that JSON file exists
        if !level_json_path.exists() {
            bail!(
                "Referenced level JSON file does not exist: {} (from {})",
                level_json_path.display(),
                levels_toml_path.display()
            );
        }

        // Parse JSON file as LevelDefinition
        validate_level_json(&level_json_path)?;
    }

    Ok(())
}

fn validate_level_json(path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read level JSON file: {}", path.display()))?;

    let _: LevelDefinition = serde_json::from_str(&content).with_context(|| {
        format!(
            "Failed to parse level JSON as LevelDefinition: {}",
            path.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::LevelsToml;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_missing_levels_toml() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let result = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("levels.toml not found"));
    }

    #[test]
    fn test_validate_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let levels_toml_path = difficulty_dir.join("levels.toml");
        fs::write(&levels_toml_path, "invalid toml content [[[").unwrap();

        let result = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_validate_missing_json_file() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let levels_toml = LevelsToml {
            level: vec![crate::levels::LevelMeta {
                id: Some("test".to_string()),
                file: Some("missing.json".to_string()),
                author: Some("test".to_string()),
                solved: Some(true),
                difficulty: Some("easy".to_string()),
                tags: Some(vec![]),
                description: Some("Test".to_string()),
            }],
        };

        let levels_toml_path = difficulty_dir.join("levels.toml");
        crate::levels::write_levels_toml(&levels_toml_path, &levels_toml).unwrap();

        let result = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_invalid_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let level_json_path = difficulty_dir.join("test.json");
        fs::write(&level_json_path, "{invalid json}").unwrap();

        let levels_toml = LevelsToml {
            level: vec![crate::levels::LevelMeta {
                id: Some("test".to_string()),
                file: Some("test.json".to_string()),
                author: Some("test".to_string()),
                solved: Some(true),
                difficulty: Some("easy".to_string()),
                tags: Some(vec![]),
                description: Some("Test".to_string()),
            }],
        };

        let levels_toml_path = difficulty_dir.join("levels.toml");
        crate::levels::write_levels_toml(&levels_toml_path, &levels_toml).unwrap();

        let result = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_validate_valid_levels() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        // Create a valid level JSON
        let level_json = r#"{
            "id": 1,
            "name": "Test Level",
            "difficulty": "easy",
            "gridSize": {"width": 10, "height": 10},
            "snake": [{"x": 5, "y": 5}, {"x": 4, "y": 5}],
            "snakeDirection": "East",
            "obstacles": [],
            "food": [],
            "exit": {"x": 7, "y": 7},
            "floatingFood": [],
            "fallingFood": [],
            "stones": [],
            "spikes": [],
            "totalFood": 0
        }"#;

        let level_json_path = difficulty_dir.join("test.json");
        fs::write(&level_json_path, level_json).unwrap();

        let levels_toml = LevelsToml {
            level: vec![crate::levels::LevelMeta {
                id: Some("test".to_string()),
                file: Some("test.json".to_string()),
                author: Some("test".to_string()),
                solved: Some(true),
                difficulty: Some("easy".to_string()),
                tags: Some(vec![]),
                description: Some("Test".to_string()),
            }],
        };

        let levels_toml_path = difficulty_dir.join("levels.toml");
        crate::levels::write_levels_toml(&levels_toml_path, &levels_toml).unwrap();

        let result = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        if let Err(e) = &result {
            eprintln!("Error: {:#}", e);
        }
        assert!(result.is_ok());
    }
}
