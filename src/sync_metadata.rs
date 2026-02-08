use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;

use crate::levels::DEFAULT_DIFFICULTIES;
use crate::name_generator::generate_names_for_directory;
use crate::playback_generator::{
    generate_all_playbacks, generate_playbacks_for_difficulty, update_solved_status_from_results,
};
use crate::toml_generator::{generate_all_levels_toml, generate_levels_toml};

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

/// Sync metadata using explicit levels/playbacks roots.
pub fn sync_metadata_with_roots(
    levels_root: &Path,
    playbacks_root: &Path,
    difficulty: Option<&str>,
) -> Result<SyncSummary> {
    if !levels_root.exists() {
        anyhow::bail!("Levels directory not found: {}", levels_root.display());
    }

    let difficulties = if let Some(diff) = difficulty {
        vec![diff]
    } else {
        DEFAULT_DIFFICULTIES.to_vec()
    };

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

    let playback_results = if let Some(diff) = difficulty {
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
