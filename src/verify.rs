use crate::playback::load_playback_directions;
use anyhow::{bail, Context, Result};
use gsnake_core::{engine::GameEngine, GameStatus, LevelDefinition};
use std::{fs, path::{Component, Path, PathBuf}};

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
            }
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

    let mut engine = GameEngine::new(level);
    let mut frame = engine.generate_frame();

    for direction in directions {
        if frame.state.status != GameStatus::Playing {
            break;
        }

        engine.process_move(direction);
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
    let level: LevelDefinition = serde_json::from_str(&contents)
        .with_context(|| "Failed to parse level JSON")?;
    Ok(level)
}
