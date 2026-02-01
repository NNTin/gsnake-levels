use crate::{levels, verify};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

pub fn run_verify_all() -> Result<()> {
    let levels_root = levels::find_levels_root()?;
    let mut any_failed = false;

    for difficulty in levels::DEFAULT_DIFFICULTIES {
        let levels_toml_path = levels_root.join(difficulty).join("levels.toml");
        if !levels_toml_path.exists() {
            continue;
        }

        let mut levels_toml = levels::read_levels_toml(&levels_toml_path)?;
        let mut updated = false;

        for entry in &mut levels_toml.level {
            let file = match entry.file.as_deref() {
                Some(file) => file,
                None => continue,
            };
            let level_path = levels_root.join(difficulty).join(file);
            if !level_path.exists() {
                bail!("Level file not found: {}", level_path.display());
            }

            let playback_path = infer_playback_path(&levels_root, &level_path)?;
            if !playback_path.exists() {
                continue;
            }

            match verify::verify_level(&level_path, &playback_path) {
                Ok(()) => {
                    entry.solved = Some(true);
                },
                Err(error) => {
                    entry.solved = Some(false);
                    any_failed = true;
                    eprintln!("Verification failed for {}: {error}", level_path.display());
                },
            }
            updated = true;
        }

        if updated {
            levels::write_levels_toml(&levels_toml_path, &levels_toml)
                .with_context(|| format!("Failed to write {}", levels_toml_path.display()))?;
        }
    }

    if any_failed {
        bail!("One or more levels failed verification")
    } else {
        Ok(())
    }
}

fn infer_playback_path(levels_root: &PathBuf, level_path: &Path) -> Result<PathBuf> {
    let relative = level_path.strip_prefix(levels_root).with_context(|| {
        format!(
            "Level path {} is not under levels root {}",
            level_path.display(),
            levels_root.display()
        )
    })?;
    let mut playback = levels_root
        .parent()
        .unwrap_or(levels_root)
        .join("playbacks");
    for component in relative.components() {
        playback.push(component);
    }
    Ok(playback)
}
