use anyhow::{bail, Context, Result};
use gsnake_core::Direction;
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
struct PlaybackFileStep {
    key: String,
    #[allow(dead_code)]
    delay_ms: u64,
}

pub fn load_playback_directions(path: &Path) -> Result<Vec<Direction>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read playback file: {}", path.display()))?;
    let raw_steps: Vec<PlaybackFileStep> =
        serde_json::from_str(&contents).with_context(|| "Failed to parse playback JSON")?;

    if raw_steps.is_empty() {
        bail!("Playback input file is empty");
    }

    let mut directions = Vec::with_capacity(raw_steps.len());
    for (index, step) in raw_steps.into_iter().enumerate() {
        let direction = parse_key(&step.key).with_context(|| {
            format!(
                "Failed to parse playback step {} in {}",
                index + 1,
                path.display()
            )
        })?;
        directions.push(direction);
    }

    Ok(directions)
}

fn parse_key(key: &str) -> Result<Direction> {
    if key.len() == 1 {
        let ch = key
            .chars()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Playback key cannot be empty"))?;
        if matches!(ch, 'R' | 'D' | 'L' | 'U') {
            return parse_string_char(ch);
        }
    }

    let normalized = key.trim().to_lowercase();
    match normalized.as_str() {
        "right" | "east" => Ok(Direction::East),
        "down" | "south" => Ok(Direction::South),
        "left" | "west" => Ok(Direction::West),
        "up" | "north" => Ok(Direction::North),
        _ => bail!("Invalid key '{key}'. Use Right/Left/Up/Down (or R/L/U/D)."),
    }
}

fn parse_string_char(ch: char) -> Result<Direction> {
    match ch {
        'R' => Ok(Direction::East),
        'D' => Ok(Direction::South),
        'L' => Ok(Direction::West),
        'U' => Ok(Direction::North),
        _ => bail!("Invalid input character '{ch}'. Use R, D, L, U for moves."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_playback_directions_valid_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"key": "Right", "delay_ms": 200}},
                {{"key": "Down", "delay_ms": 200}},
                {{"key": "Left", "delay_ms": 200}},
                {{"key": "Up", "delay_ms": 200}}
            ]"#
        )
        .unwrap();

        let result = load_playback_directions(file.path());
        assert!(result.is_ok());

        let directions = result.unwrap();
        assert_eq!(directions.len(), 4);
        assert_eq!(directions[0], Direction::East);
        assert_eq!(directions[1], Direction::South);
        assert_eq!(directions[2], Direction::West);
        assert_eq!(directions[3], Direction::North);
    }

    #[test]
    fn test_load_playback_directions_from_real_fixture() {
        let playback_path = Path::new("playbacks/easy/level_001.json");
        if playback_path.exists() {
            let result = load_playback_directions(playback_path);
            assert!(result.is_ok());

            let directions = result.unwrap();
            assert!(!directions.is_empty());
            assert!(directions.iter().all(|d| matches!(
                d,
                Direction::East | Direction::South | Direction::West | Direction::North
            )));
        }
    }

    #[test]
    fn test_load_playback_directions_missing_file() {
        let result = load_playback_directions(Path::new("nonexistent/playback.json"));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Failed to read playback file"));
    }

    #[test]
    fn test_load_playback_directions_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{{invalid json}}").unwrap();

        let result = load_playback_directions(file.path());
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Failed to parse playback JSON"));
    }

    #[test]
    fn test_load_playback_directions_wrong_schema() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"invalid_field": "value"}}
            ]"#
        )
        .unwrap();

        let result = load_playback_directions(file.path());
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Failed to parse playback JSON"));
    }

    #[test]
    fn test_load_playback_directions_empty_array() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[]").unwrap();

        let result = load_playback_directions(file.path());
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Playback input file is empty"));
    }

    #[test]
    fn test_load_playback_directions_short_keys() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"key": "R", "delay_ms": 100}},
                {{"key": "D", "delay_ms": 100}},
                {{"key": "L", "delay_ms": 100}},
                {{"key": "U", "delay_ms": 100}}
            ]"#
        )
        .unwrap();

        let result = load_playback_directions(file.path());
        assert!(result.is_ok());

        let directions = result.unwrap();
        assert_eq!(directions.len(), 4);
        assert_eq!(directions[0], Direction::East);
        assert_eq!(directions[1], Direction::South);
        assert_eq!(directions[2], Direction::West);
        assert_eq!(directions[3], Direction::North);
    }

    #[test]
    fn test_load_playback_directions_long_keys() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"key": "east", "delay_ms": 100}},
                {{"key": "south", "delay_ms": 100}},
                {{"key": "west", "delay_ms": 100}},
                {{"key": "north", "delay_ms": 100}}
            ]"#
        )
        .unwrap();

        let result = load_playback_directions(file.path());
        assert!(result.is_ok());

        let directions = result.unwrap();
        assert_eq!(directions.len(), 4);
        assert_eq!(directions[0], Direction::East);
        assert_eq!(directions[1], Direction::South);
        assert_eq!(directions[2], Direction::West);
        assert_eq!(directions[3], Direction::North);
    }

    #[test]
    fn test_load_playback_directions_invalid_key() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"key": "invalid", "delay_ms": 100}}
            ]"#
        )
        .unwrap();

        let result = load_playback_directions(file.path());
        assert!(result.is_err());

        let error = result.unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("Invalid key"));
    }

    #[test]
    fn test_load_playback_directions_invalid_key_reports_step_context() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"key": "X", "delay_ms": 100}}
            ]"#
        )
        .unwrap();

        let result = load_playback_directions(file.path());
        assert!(result.is_err());

        let error = result.unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("Failed to parse playback step 1"));
        assert!(message.contains("Invalid key 'X'"));
    }
}
