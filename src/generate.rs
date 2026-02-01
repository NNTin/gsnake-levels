use crate::levels;
use anyhow::{bail, Context, Result};
use gsnake_core::LevelDefinition;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn run_generate_levels_json(filter: Option<&str>, dry_run: bool) -> Result<()> {
    let levels_root = levels::find_levels_root()?;
    let difficulties = parse_filter(filter)?;
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
