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
    let raw_steps: Vec<PlaybackFileStep> = serde_json::from_str(&contents)
        .with_context(|| "Failed to parse playback JSON")?;

    if raw_steps.is_empty() {
        bail!("Playback input file is empty");
    }

    let mut directions = Vec::with_capacity(raw_steps.len());
    for step in raw_steps {
        let direction = parse_key(&step.key)?;
        directions.push(direction);
    }

    Ok(directions)
}

fn parse_key(key: &str) -> Result<Direction> {
    if key.len() == 1 {
        let ch = key.chars().next().unwrap();
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
        _ => bail!(
            "Invalid key '{key}'. Use Right/Left/Up/Down (or R/L/U/D)."
        ),
    }
}

fn parse_string_char(ch: char) -> Result<Direction> {
    match ch {
        'R' => Ok(Direction::East),
        'D' => Ok(Direction::South),
        'L' => Ok(Direction::West),
        'U' => Ok(Direction::North),
        _ => bail!(
            "Invalid input character '{ch}'. Use R, D, L, U for moves."
        ),
    }
}
