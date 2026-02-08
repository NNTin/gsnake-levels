use crate::levels::{LevelMeta, LevelsToml};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Minimal level structure to read the name field
#[derive(Deserialize)]
struct LevelNameOnly {
    name: String,
}

/// Scans a difficulty directory for JSON files and generates levels.toml
#[allow(dead_code)]
pub fn generate_levels_toml(difficulty_dir: &Path, difficulty: &str) -> Result<()> {
    // Verify directory exists
    if !difficulty_dir.exists() || !difficulty_dir.is_dir() {
        bail!(
            "Directory does not exist or is not a directory: {}",
            difficulty_dir.display()
        );
    }

    // Scan for JSON files
    let entries = fs::read_dir(difficulty_dir)
        .with_context(|| format!("Failed to read directory: {}", difficulty_dir.display()))?;

    let mut level_metas = Vec::new();

    for entry in entries {
        let entry = entry.with_context(|| {
            format!(
                "Failed to read directory entry in {}",
                difficulty_dir.display()
            )
        })?;
        let path = entry.path();

        // Only process .json files
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Get the filename
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", path.display()))?
            .to_string();

        // Get the id (filename without extension)
        let id = path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file stem: {}", path.display()))?
            .to_string();

        // Verify JSON file exists and is readable
        if !path.exists() {
            bail!("JSON file does not exist: {}", path.display());
        }

        // Read the level's name field for the description
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read level file: {}", path.display()))?;

        let level_data: LevelNameOnly = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse level JSON: {}", path.display()))?;

        // Create the metadata entry
        let meta = LevelMeta {
            id: Some(id),
            file: Some(filename),
            author: Some("gsnake".to_string()),
            solved: Some(true),
            difficulty: Some(difficulty.to_string()),
            tags: Some(vec![]),
            description: Some(level_data.name),
        };

        level_metas.push(meta);
    }

    // Sort by id for consistent ordering
    level_metas.sort_by(|a, b| {
        let id_a = a.id.as_deref().unwrap_or("");
        let id_b = b.id.as_deref().unwrap_or("");
        id_a.cmp(id_b)
    });

    // Create the TOML structure
    let levels_toml = LevelsToml { level: level_metas };

    // Write to levels.toml in the difficulty directory
    let toml_path = difficulty_dir.join("levels.toml");
    let output = toml::to_string_pretty(&levels_toml).with_context(|| {
        format!(
            "Failed to serialize levels.toml for {}",
            difficulty_dir.display()
        )
    })?;

    fs::write(&toml_path, output)
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;

    Ok(())
}

/// Generates levels.toml for all difficulty directories
#[allow(dead_code)]
pub fn generate_all_levels_toml(levels_root: &Path) -> Result<Vec<String>> {
    let difficulties = ["easy", "medium", "hard"];
    let mut results = Vec::new();

    for difficulty in &difficulties {
        let difficulty_dir = levels_root.join(difficulty);

        if !difficulty_dir.exists() {
            continue; // Skip if directory doesn't exist
        }

        generate_levels_toml(&difficulty_dir, difficulty).with_context(|| {
            format!(
                "Failed to generate levels.toml for difficulty: {}",
                difficulty
            )
        })?;

        results.push(difficulty.to_string());
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_level_json(dir: &Path, filename: &str, name: &str) -> Result<PathBuf> {
        let level_json = serde_json::json!({
            "id": "test-id",
            "name": name,
            "difficulty": "easy",
            "gridSize": { "width": 10, "height": 10 },
            "snake": [{ "x": 0, "y": 0 }],
            "obstacles": [],
            "food": [],
            "exit": { "x": 5, "y": 5 },
            "snakeDirection": "East",
            "floatingFood": [],
            "fallingFood": [],
            "stones": [],
            "spikes": [],
            "totalFood": 0
        });

        let path = dir.join(filename);
        fs::write(&path, serde_json::to_string_pretty(&level_json)?)?;
        Ok(path)
    }

    #[test]
    fn test_generate_levels_toml_single_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let easy_dir = temp_dir.path().join("easy");
        fs::create_dir(&easy_dir)?;

        // Create test level files
        create_test_level_json(&easy_dir, "level_001.json", "Test Level One")?;
        create_test_level_json(&easy_dir, "level_002.json", "Test Level Two")?;

        // Generate levels.toml
        generate_levels_toml(&easy_dir, "easy")?;

        // Verify levels.toml was created
        let toml_path = easy_dir.join("levels.toml");
        assert!(toml_path.exists());

        // Read and parse the TOML
        let contents = fs::read_to_string(&toml_path)?;
        let levels_toml: LevelsToml = toml::from_str(&contents)?;

        // Verify entries
        assert_eq!(levels_toml.level.len(), 2);

        // Check first entry
        let level1 = &levels_toml.level[0];
        assert_eq!(level1.id.as_deref(), Some("level_001"));
        assert_eq!(level1.file.as_deref(), Some("level_001.json"));
        assert_eq!(level1.author.as_deref(), Some("gsnake"));
        assert_eq!(level1.solved, Some(true));
        assert_eq!(level1.difficulty.as_deref(), Some("easy"));
        assert_eq!(level1.description.as_deref(), Some("Test Level One"));

        Ok(())
    }

    #[test]
    fn test_generate_levels_toml_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent");

        let result = generate_levels_toml(&nonexistent, "easy");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_all_levels_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create difficulty directories
        let easy_dir = temp_dir.path().join("easy");
        let medium_dir = temp_dir.path().join("medium");
        let hard_dir = temp_dir.path().join("hard");

        fs::create_dir(&easy_dir)?;
        fs::create_dir(&medium_dir)?;
        fs::create_dir(&hard_dir)?;

        // Create test levels
        create_test_level_json(&easy_dir, "level_001.json", "Easy Level")?;
        create_test_level_json(&medium_dir, "level_002.json", "Medium Level")?;
        create_test_level_json(&hard_dir, "level_003.json", "Hard Level")?;

        // Generate all
        let results = generate_all_levels_toml(temp_dir.path())?;

        assert_eq!(results.len(), 3);
        assert!(results.contains(&"easy".to_string()));
        assert!(results.contains(&"medium".to_string()));
        assert!(results.contains(&"hard".to_string()));

        // Verify all TOML files exist
        assert!(easy_dir.join("levels.toml").exists());
        assert!(medium_dir.join("levels.toml").exists());
        assert!(hard_dir.join("levels.toml").exists());

        Ok(())
    }

    #[test]
    fn test_generate_levels_toml_sorts_by_id() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let easy_dir = temp_dir.path().join("easy");
        fs::create_dir(&easy_dir)?;

        // Create files in non-alphabetical order
        create_test_level_json(&easy_dir, "level_003.json", "Level C")?;
        create_test_level_json(&easy_dir, "level_001.json", "Level A")?;
        create_test_level_json(&easy_dir, "level_002.json", "Level B")?;

        generate_levels_toml(&easy_dir, "easy")?;

        let toml_path = easy_dir.join("levels.toml");
        let contents = fs::read_to_string(&toml_path)?;
        let levels_toml: LevelsToml = toml::from_str(&contents)?;

        // Verify sorted order
        assert_eq!(levels_toml.level[0].id.as_deref(), Some("level_001"));
        assert_eq!(levels_toml.level[1].id.as_deref(), Some("level_002"));
        assert_eq!(levels_toml.level[2].id.as_deref(), Some("level_003"));

        Ok(())
    }
}
