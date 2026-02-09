use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;

use crate::levels::DEFAULT_DIFFICULTIES;
use crate::name_generator::generate_names_for_directory;
use crate::playback_generator::{
    generate_all_playbacks, generate_playbacks_for_difficulty, update_solved_status_from_results,
};
use crate::toml_generator::{generate_all_levels_toml, generate_levels_toml};

#[derive(Debug)]
pub struct SyncSummary {
    pub names_generated: usize,
    pub toml_files_updated: usize,
    pub playbacks_created: usize,
}

/// Sync metadata for all difficulties or a specific one
pub fn sync_metadata(difficulty: Option<&str>) -> Result<SyncSummary> {
    let levels_root = crate::levels::find_levels_root()?;
    let playbacks_root = levels_root
        .parent()
        .map(|parent| parent.join("playbacks"))
        .unwrap_or_else(|| Path::new("playbacks").to_path_buf());
    sync_metadata_with_roots(&levels_root, &playbacks_root, difficulty)
}

fn resolve_difficulties(difficulty: Option<&str>) -> Result<Vec<&'static str>> {
    if let Some(raw) = difficulty {
        let normalized = raw.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            anyhow::bail!("Difficulty filter cannot be empty");
        }

        if let Some(selected) = DEFAULT_DIFFICULTIES
            .iter()
            .copied()
            .find(|item| *item == normalized)
        {
            return Ok(vec![selected]);
        }

        anyhow::bail!(
            "Unknown difficulty '{}'. Expected one of: easy, medium, hard",
            raw
        );
    }

    Ok(DEFAULT_DIFFICULTIES.to_vec())
}

/// Sync metadata using explicit levels/playbacks roots.
pub fn sync_metadata_with_roots(
    levels_root: &Path,
    playbacks_root: &Path,
    difficulty: Option<&str>,
) -> Result<SyncSummary> {
    if !levels_root.exists() {
        anyhow::bail!("Levels directory not found: {}", levels_root.display());
    }

    let difficulties = resolve_difficulties(difficulty)?;

    let mut total_names = 0;
    let mut used_names = HashSet::new();

    // Step 1: Generate names for all levels
    println!("Generating level names...");
    for diff in &difficulties {
        let diff_path = levels_root.join(diff);
        if !diff_path.exists() {
            println!("  Skipping {}: directory not found", diff);
            continue;
        }

        let results = generate_names_for_directory(&diff_path, &mut used_names)
            .with_context(|| format!("Failed to generate names for {}", diff))?;

        println!("  {}: {} names generated", diff, results.len());
        total_names += results.len();
    }

    // Step 2: Generate levels.toml files
    println!("Generating levels.toml files...");
    let toml_results = if difficulty.is_some() {
        // Single difficulty
        let diff = difficulties[0];
        let diff_path = levels_root.join(diff);
        generate_levels_toml(&diff_path, diff)
            .with_context(|| format!("Failed to generate levels.toml for {}", diff))?;
        vec![format!("levels/{}/levels.toml", diff)]
    } else {
        // All difficulties
        generate_all_levels_toml(levels_root)
            .with_context(|| "Failed to generate levels.toml files")?
    };

    println!("  {} levels.toml files updated", toml_results.len());

    // Step 3: Generate playbacks
    println!("Generating playbacks...");
    let max_depth = 500; // Default from US-006

    let playback_results = if difficulty.is_some() {
        let diff = difficulties[0];
        let levels_dir = levels_root.join(diff);
        let playbacks_dir = playbacks_root.join(diff);
        generate_playbacks_for_difficulty(&levels_dir, &playbacks_dir, max_depth)
            .with_context(|| format!("Failed to generate playbacks for {}", diff))?
    } else {
        generate_all_playbacks(levels_root, playbacks_root, max_depth)
            .with_context(|| "Failed to generate playbacks")?
    };

    let solved_count = playback_results.iter().filter(|r| r.solved).count();
    println!("  {} playbacks created", solved_count);

    // Step 4: Update solved status in levels.toml
    println!("Updating solved status...");
    update_solved_status_from_results(&playback_results)
        .with_context(|| "Failed to update solved status")?;

    Ok(SyncSummary {
        names_generated: total_names,
        toml_files_updated: toml_results.len(),
        playbacks_created: solved_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_difficulty_dirs(levels_root: &Path, difficulties: &[&str]) -> Result<()> {
        for difficulty in difficulties {
            fs::create_dir_all(levels_root.join(difficulty))?;
        }
        Ok(())
    }

    #[test]
    fn test_sync_metadata_with_roots_success_all_difficulties() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let levels_root = temp_dir.path().join("levels");
        let playbacks_root = temp_dir.path().join("playbacks");

        create_difficulty_dirs(&levels_root, &DEFAULT_DIFFICULTIES)?;

        let summary = sync_metadata_with_roots(&levels_root, &playbacks_root, None)?;
        assert_eq!(summary.names_generated, 0);
        assert_eq!(summary.toml_files_updated, 3);
        assert_eq!(summary.playbacks_created, 0);

        assert!(levels_root.join("easy/levels.toml").exists());
        assert!(levels_root.join("medium/levels.toml").exists());
        assert!(levels_root.join("hard/levels.toml").exists());
        Ok(())
    }

    #[test]
    fn test_sync_metadata_with_roots_missing_levels_root_fails() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let levels_root = temp_dir.path().join("missing-levels");
        let playbacks_root = temp_dir.path().join("playbacks");

        let result = sync_metadata_with_roots(&levels_root, &playbacks_root, None);
        assert!(result.is_err());
        let error = result
            .expect_err("Expected missing levels root error")
            .to_string();
        assert!(error.contains("Levels directory not found"));
    }

    #[test]
    fn test_sync_metadata_with_roots_rejects_unknown_difficulty() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let levels_root = temp_dir.path().join("levels");
        let playbacks_root = temp_dir.path().join("playbacks");
        create_difficulty_dirs(&levels_root, &["easy"])?;

        let result = sync_metadata_with_roots(&levels_root, &playbacks_root, Some("legendary"));
        assert!(result.is_err());
        let error = result
            .expect_err("Expected unknown difficulty error")
            .to_string();
        assert!(error.contains("Unknown difficulty"));
        Ok(())
    }

    #[test]
    fn test_sync_metadata_with_roots_normalizes_difficulty_filter() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let levels_root = temp_dir.path().join("levels");
        let playbacks_root = temp_dir.path().join("playbacks");
        create_difficulty_dirs(&levels_root, &["easy"])?;

        let summary = sync_metadata_with_roots(&levels_root, &playbacks_root, Some(" EASY "))?;
        assert_eq!(summary.names_generated, 0);
        assert_eq!(summary.toml_files_updated, 1);
        assert_eq!(summary.playbacks_created, 0);
        assert!(levels_root.join("easy/levels.toml").exists());
        Ok(())
    }

    #[test]
    fn test_sync_metadata_resolves_levels_root_from_package_directory() -> Result<()> {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new()?;
        let levels_root = temp_dir.path().join("levels");
        create_difficulty_dirs(&levels_root, &DEFAULT_DIFFICULTIES)?;
        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());

        let summary = sync_metadata(None)?;
        assert_eq!(summary.toml_files_updated, 3);
        assert!(levels_root.join("easy/levels.toml").exists());
        Ok(())
    }

    #[test]
    fn test_sync_metadata_resolves_levels_root_from_repo_root() -> Result<()> {
        let _lock = crate::test_cwd::cwd_mutex()
            .lock()
            .expect("Failed to lock cwd mutex");

        let temp_dir = TempDir::new()?;
        let package_root = temp_dir.path().join("gsnake-levels");
        let levels_root = package_root.join("levels");
        create_difficulty_dirs(&levels_root, &DEFAULT_DIFFICULTIES)?;
        let _cwd = crate::test_cwd::CwdGuard::set(temp_dir.path());

        let summary = sync_metadata(None)?;
        assert_eq!(summary.toml_files_updated, 3);
        assert!(levels_root.join("easy/levels.toml").exists());
        Ok(())
    }
}
