use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const DEFAULT_DIFFICULTIES: [&str; 3] = ["easy", "medium", "hard"];

#[derive(Debug, Serialize, Deserialize)]
pub struct LevelsToml {
    #[serde(default)]
    pub level: Vec<LevelMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LevelMeta {
    pub id: Option<String>,
    pub file: Option<String>,
    pub author: Option<String>,
    pub solved: Option<bool>,
    pub difficulty: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
}

pub fn update_solved_status(level_path: &Path, solved: bool) -> Result<()> {
    let levels_toml_path = levels_toml_path_for(level_path);
    if !levels_toml_path.exists() {
        return Ok(());
    }

    let contents = fs::read_to_string(&levels_toml_path)
        .with_context(|| format!("Failed to read {}", levels_toml_path.display()))?;
    let mut levels_toml: LevelsToml = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", levels_toml_path.display()))?;

    let file_name = level_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Level path has no valid filename"))?;

    let mut updated = false;
    for entry in &mut levels_toml.level {
        if entry.file.as_deref() == Some(file_name) {
            entry.solved = Some(solved);
            updated = true;
            break;
        }
    }

    if !updated {
        return Ok(());
    }

    let output = toml::to_string_pretty(&levels_toml)
        .with_context(|| format!("Failed to serialize {}", levels_toml_path.display()))?;
    fs::write(&levels_toml_path, output)
        .with_context(|| format!("Failed to write {}", levels_toml_path.display()))?;

    Ok(())
}

pub fn levels_toml_path_for(level_path: &Path) -> PathBuf {
    level_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("levels.toml")
}

pub fn read_levels_toml(path: &Path) -> Result<LevelsToml> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let levels_toml: LevelsToml =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(levels_toml)
}

pub fn write_levels_toml(path: &Path, levels_toml: &LevelsToml) -> Result<()> {
    let output = toml::to_string_pretty(levels_toml)
        .with_context(|| format!("Failed to serialize {}", path.display()))?;
    fs::write(path, output).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn find_levels_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Failed to read current directory")?;
    let direct = cwd.join("levels");
    if direct.is_dir() {
        return Ok(direct);
    }

    let nested = cwd.join("gsnake-levels").join("levels");
    if nested.is_dir() {
        return Ok(nested);
    }

    bail!(
        "Could not find levels directory. Expected ./levels or ./gsnake-levels/levels from {}",
        cwd.display()
    )
}
