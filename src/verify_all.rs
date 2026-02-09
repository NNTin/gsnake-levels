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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::{read_levels_toml, write_levels_toml, LevelMeta, LevelsToml};
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn write_test_level(path: &Path) {
        let level = json!({
            "id": 1,
            "name": "Verify-All Test Level",
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
                id: Some("verify-all-level".to_string()),
                file: Some(file.to_string()),
                author: Some("gsnake".to_string()),
                solved,
                difficulty: Some("easy".to_string()),
                tags: Some(vec![]),
                description: Some("Verify-all test level".to_string()),
            }],
        };
        write_levels_toml(levels_toml_path, &levels_toml).unwrap();
    }

    #[test]
    fn test_infer_playback_path_fails_when_level_outside_root() {
        let temp_dir = TempDir::new().unwrap();
        let levels_root = temp_dir.path().join("levels");
        let external_level = temp_dir.path().join("outside/level.json");

        let error = infer_playback_path(&levels_root, &external_level).unwrap_err();
        assert!(error.to_string().contains("is not under levels root"));
    }

    #[test]
    fn test_run_verify_all_fails_when_level_file_missing() {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new().unwrap();
        let easy_dir = temp_dir.path().join("levels/easy");
        fs::create_dir_all(&easy_dir).unwrap();
        write_levels_metadata(&easy_dir.join("levels.toml"), "missing.json", Some(true));
        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());

        let error = run_verify_all().unwrap_err();
        assert!(error.to_string().contains("Level file not found"));
    }

    #[test]
    fn test_run_verify_all_skips_missing_playback_without_mutating_status() {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new().unwrap();
        let easy_dir = temp_dir.path().join("levels/easy");
        fs::create_dir_all(&easy_dir).unwrap();

        let level_file = "level.json";
        write_test_level(&easy_dir.join(level_file));
        write_levels_metadata(&easy_dir.join("levels.toml"), level_file, Some(true));

        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());
        run_verify_all().expect("verify-all should skip missing playback files");

        let updated = read_levels_toml(&easy_dir.join("levels.toml")).unwrap();
        assert_eq!(updated.level[0].solved, Some(true));
    }

    #[test]
    fn test_run_verify_all_marks_unsolved_when_playback_is_invalid() {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new().unwrap();
        let easy_dir = temp_dir.path().join("levels/easy");
        let playbacks_dir = temp_dir.path().join("playbacks/easy");
        fs::create_dir_all(&easy_dir).unwrap();
        fs::create_dir_all(&playbacks_dir).unwrap();

        let level_file = "level.json";
        write_test_level(&easy_dir.join(level_file));
        write_levels_metadata(&easy_dir.join("levels.toml"), level_file, Some(true));
        fs::write(playbacks_dir.join(level_file), "{malformed-json}").unwrap();

        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());
        let error = run_verify_all().unwrap_err();
        assert!(error
            .to_string()
            .contains("One or more levels failed verification"));

        let updated = read_levels_toml(&easy_dir.join("levels.toml")).unwrap();
        assert_eq!(updated.level[0].solved, Some(false));
    }
}
