use anyhow::{Context, Result};
use clap::Parser;
use gsnake_core::Direction;
use gsnake_levels::solver::{load_level, solve_level};
use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "solve_level")]
#[command(about = "Solve a gSnake level and generate playback solution")]
struct Args {
    /// Path to the level JSON file
    level_path: PathBuf,

    /// Path to save the playback solution JSON
    output_path: PathBuf,

    /// Maximum search depth for solver (default: 500)
    #[arg(short = 'd', long = "max-depth", default_value = "500")]
    max_depth: usize,
}

#[derive(Serialize)]
struct PlaybackStep {
    key: String,
    delay_ms: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let level = load_level(&args.level_path)?;
    let solution = solve_level(level, args.max_depth)
        .with_context(|| format!("No solution found within depth {}", args.max_depth))?;

    let steps: Vec<PlaybackStep> = solution
        .into_iter()
        .map(|dir| PlaybackStep {
            key: direction_name(dir).to_string(),
            delay_ms: 200,
        })
        .collect();

    if let Some(parent) = args.output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(
        &args.output_path,
        serde_json::to_string_pretty(&steps)? + "\n",
    )
    .with_context(|| format!("Failed to write {}", args.output_path.display()))?;

    println!(
        "Solved {} in {} moves",
        args.level_path.display(),
        steps.len()
    );
    Ok(())
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::North => "Up",
        Direction::South => "Down",
        Direction::East => "Right",
        Direction::West => "Left",
    }
}
