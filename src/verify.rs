use crate::playback::load_playback_directions;
use anyhow::{bail, Context, Result};
use gsnake_core::{engine::GameEngine, GameStatus, LevelDefinition};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub fn resolve_playback_path(level_path: &Path, override_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(path);
    }

    let mut replaced = PathBuf::new();
    let mut replaced_any = false;
    for component in level_path.components() {
        match component {
            Component::Normal(name) if name == "levels" && !replaced_any => {
                replaced.push("playbacks");
                replaced_any = true;
            },
            _ => replaced.push(component.as_os_str()),
        }
    }

    if replaced_any {
        return Ok(replaced);
    }

    bail!(
        "Unable to infer playback path from {}. Provide --playback.",
        level_path.display()
    )
}

pub fn verify_level(level_path: &Path, playback_path: &Path) -> Result<()> {
    let level = load_level(level_path)
        .with_context(|| format!("Failed to load level: {}", level_path.display()))?;
    let directions = load_playback_directions(playback_path)
        .with_context(|| format!("Failed to load playback: {}", playback_path.display()))?;

    let mut engine = GameEngine::new(level)
        .with_context(|| format!("Invalid grid size in level file: {}", level_path.display()))?;
    let mut frame = engine.generate_frame();

    for direction in directions {
        if frame.state.status != GameStatus::Playing {
            break;
        }

        engine
            .process_move(direction)
            .with_context(|| format!("Engine move failed for direction {direction:?}"))?;
        frame = engine.generate_frame();
    }

    match frame.state.status {
        GameStatus::LevelComplete | GameStatus::AllComplete => Ok(()),
        GameStatus::GameOver => bail!("Playback resulted in Game Over"),
        GameStatus::Playing => bail!("Playback did not complete the level"),
    }
}

fn load_level(level_path: &Path) -> Result<LevelDefinition> {
    let contents = fs::read_to_string(level_path)
        .with_context(|| format!("Failed to read level file: {}", level_path.display()))?;
    let level: LevelDefinition =
        serde_json::from_str(&contents).with_context(|| "Failed to parse level JSON")?;
    Ok(level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn write_test_level(path: &Path, exit_x: i32, spikes: &[(i32, i32)]) {
        let spikes_json: Vec<_> = spikes
            .iter()
            .map(|(x, y)| json!({ "x": x, "y": y }))
            .collect();
        let level = json!({
            "id": 1,
            "name": "Test Level",
            "difficulty": "easy",
            "gridSize": { "width": 5, "height": 5 },
            "snake": [{ "x": 0, "y": 0 }],
            "snakeDirection": "East",
            "obstacles": [],
            "food": [],
            "exit": { "x": exit_x, "y": 0 },
            "floatingFood": [],
            "fallingFood": [],
            "stones": [],
            "spikes": spikes_json,
            "totalFood": 0
        });
        fs::write(path, serde_json::to_string_pretty(&level).unwrap()).unwrap();
    }

    fn write_playback(path: &Path, keys: &[&str]) {
        let steps: Vec<_> = keys
            .iter()
            .map(|key| json!({ "key": key, "delay_ms": 1 }))
            .collect();
        fs::write(path, serde_json::to_string_pretty(&steps).unwrap()).unwrap();
    }

    #[test]
    fn test_resolve_playback_path_valid_easy_level() {
        let level_path = Path::new("levels/easy/level_001.json");
        let result = resolve_playback_path(level_path, None);

        assert!(result.is_ok());
        let playback_path = result.unwrap();
        assert_eq!(
            playback_path,
            PathBuf::from("playbacks/easy/level_001.json")
        );
    }

    #[test]
    fn test_resolve_playback_path_valid_medium_level() {
        let level_path = Path::new("levels/medium/level_005.json");
        let result = resolve_playback_path(level_path, None);

        assert!(result.is_ok());
        let playback_path = result.unwrap();
        assert_eq!(
            playback_path,
            PathBuf::from("playbacks/medium/level_005.json")
        );
    }

    #[test]
    fn test_resolve_playback_path_valid_hard_level() {
        let level_path = Path::new("levels/hard/level_010.json");
        let result = resolve_playback_path(level_path, None);

        assert!(result.is_ok());
        let playback_path = result.unwrap();
        assert_eq!(
            playback_path,
            PathBuf::from("playbacks/hard/level_010.json")
        );
    }

    #[test]
    fn test_resolve_playback_path_with_override() {
        let level_path = Path::new("levels/easy/level_001.json");
        let override_path = PathBuf::from("custom/path/to/playback.json");
        let result = resolve_playback_path(level_path, Some(override_path.clone()));

        assert!(result.is_ok());
        let playback_path = result.unwrap();
        assert_eq!(playback_path, override_path);
    }

    #[test]
    fn test_resolve_playback_path_missing_levels_directory() {
        let level_path = Path::new("invalid/easy/level_001.json");
        let result = resolve_playback_path(level_path, None);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unable to infer playback path"));
    }

    #[test]
    fn test_resolve_playback_path_no_levels_component() {
        let level_path = Path::new("some/other/path/file.json");
        let result = resolve_playback_path(level_path, None);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unable to infer playback path"));
    }

    #[test]
    fn test_resolve_playback_path_absolute_path() {
        let level_path = Path::new("/absolute/path/levels/easy/level_001.json");
        let result = resolve_playback_path(level_path, None);

        assert!(result.is_ok());
        let playback_path = result.unwrap();
        assert_eq!(
            playback_path,
            PathBuf::from("/absolute/path/playbacks/easy/level_001.json")
        );
    }

    #[test]
    fn test_resolve_playback_path_nested_levels() {
        let level_path = Path::new("some/nested/levels/easy/level_001.json");
        let result = resolve_playback_path(level_path, None);

        assert!(result.is_ok());
        let playback_path = result.unwrap();
        assert_eq!(
            playback_path,
            PathBuf::from("some/nested/playbacks/easy/level_001.json")
        );
    }

    #[test]
    fn test_verify_level_missing_level_file() {
        let temp_dir = TempDir::new().unwrap();
        let missing_level_path = temp_dir.path().join("missing_level.json");
        let playback_path = temp_dir.path().join("playback.json");
        write_playback(&playback_path, &["Right"]);

        let error = verify_level(&missing_level_path, &playback_path).unwrap_err();
        let message = format!("{:#}", error);

        assert!(message.contains("Failed to load level"));
        assert!(message.contains("Failed to read level file"));
    }

    #[test]
    fn test_verify_level_missing_playback_file() {
        let temp_dir = TempDir::new().unwrap();
        let level_path = temp_dir.path().join("level.json");
        let missing_playback_path = temp_dir.path().join("missing_playback.json");
        write_test_level(&level_path, 4, &[]);

        let error = verify_level(&level_path, &missing_playback_path).unwrap_err();
        let message = format!("{:#}", error);

        assert!(message.contains("Failed to load playback"));
        assert!(message.contains("Failed to read playback file"));
    }

    #[test]
    fn test_verify_level_malformed_playback_file() {
        let temp_dir = TempDir::new().unwrap();
        let level_path = temp_dir.path().join("level.json");
        let playback_path = temp_dir.path().join("playback.json");
        write_test_level(&level_path, 4, &[]);
        fs::write(&playback_path, "{not-json}").unwrap();

        let error = verify_level(&level_path, &playback_path).unwrap_err();
        let message = format!("{:#}", error);

        assert!(message.contains("Failed to load playback"));
        assert!(message.contains("Failed to parse playback JSON"));
    }

    #[test]
    fn test_verify_level_returns_not_complete_error() {
        let temp_dir = TempDir::new().unwrap();
        let level_path = temp_dir.path().join("level.json");
        let playback_path = temp_dir.path().join("playback.json");
        write_test_level(&level_path, 4, &[]);
        write_playback(&playback_path, &["Right"]);

        let error = verify_level(&level_path, &playback_path).unwrap_err();
        assert!(error
            .to_string()
            .contains("Playback did not complete the level"));
    }

    #[test]
    fn test_verify_level_returns_game_over_error() {
        let temp_dir = TempDir::new().unwrap();
        let level_path = temp_dir.path().join("level.json");
        let playback_path = temp_dir.path().join("playback.json");
        write_test_level(&level_path, 4, &[(1, 0)]);
        write_playback(&playback_path, &["Right"]);

        let error = verify_level(&level_path, &playback_path).unwrap_err();
        assert!(error.to_string().contains("Playback resulted in Game Over"));
    }
}
