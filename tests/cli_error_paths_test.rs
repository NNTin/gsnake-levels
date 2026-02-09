use gsnake_levels::levels::{write_levels_toml, LevelMeta, LevelsToml};
use serde_json::json;
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

fn write_test_level(path: &Path) {
    let level = json!({
        "id": 1,
        "name": "CLI Test Level",
        "difficulty": "easy",
        "gridSize": { "width": 5, "height": 5 },
        "snake": [{ "x": 0, "y": 0 }],
        "snakeDirection": "East",
        "obstacles": [],
        "food": [],
        "exit": { "x": 4, "y": 0 },
        "floatingFood": [],
        "fallingFood": [],
        "stones": [],
        "spikes": [],
        "totalFood": 0
    });
    fs::write(path, serde_json::to_string_pretty(&level).unwrap()).unwrap();
}

fn write_levels_metadata(levels_toml_path: &Path, file: &str, solved: Option<bool>) {
    let levels_toml = LevelsToml {
        level: vec![LevelMeta {
            id: Some("cli-test-level".to_string()),
            file: Some(file.to_string()),
            author: Some("gsnake".to_string()),
            solved,
            difficulty: Some("easy".to_string()),
            tags: Some(vec![]),
            description: Some("CLI error-path test level".to_string()),
        }],
    };
    write_levels_toml(levels_toml_path, &levels_toml).unwrap();
}

fn run_levels_command(current_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gsnake-levels"))
        .args(args)
        .current_dir(current_dir)
        .output()
        .expect("failed to run gsnake-levels binary")
}

#[test]
fn test_verify_command_returns_error_for_uninferrable_playback_path() {
    let temp_dir = TempDir::new().unwrap();
    let level_path = temp_dir.path().join("custom/easy/level.json");
    fs::create_dir_all(level_path.parent().unwrap()).unwrap();
    write_test_level(&level_path);

    let output = run_levels_command(temp_dir.path(), &["verify", "custom/easy/level.json"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("Failed to resolve playback path"));
    assert!(stderr.contains("Unable to infer playback path"));
}

#[test]
fn test_verify_command_returns_error_for_malformed_playback_file() {
    let temp_dir = TempDir::new().unwrap();
    let level_path = temp_dir.path().join("levels/easy/level.json");
    let playback_path = temp_dir.path().join("playbacks/easy/level.json");
    fs::create_dir_all(level_path.parent().unwrap()).unwrap();
    fs::create_dir_all(playback_path.parent().unwrap()).unwrap();
    write_test_level(&level_path);
    fs::write(&playback_path, "{malformed-json}").unwrap();

    let output = run_levels_command(temp_dir.path(), &["verify", "levels/easy/level.json"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("Failed to load playback"));
    assert!(stderr.contains("Failed to parse playback JSON"));
}

#[test]
fn test_verify_all_command_returns_error_for_missing_level_file() {
    let temp_dir = TempDir::new().unwrap();
    let easy_dir = temp_dir.path().join("levels/easy");
    fs::create_dir_all(&easy_dir).unwrap();
    write_levels_metadata(&easy_dir.join("levels.toml"), "missing.json", Some(true));

    let output = run_levels_command(temp_dir.path(), &["verify-all"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("Level file not found"));
}
