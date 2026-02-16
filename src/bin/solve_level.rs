use anyhow::{Context, Result};
use clap::Parser;
use gsnake_levels::solver::solve_level_to_playback;
use std::path::PathBuf;

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

fn main() -> Result<()> {
    let args = Args::parse();
    let move_count = solve_level_to_playback(&args.level_path, &args.output_path, args.max_depth)
        .with_context(|| "Failed to generate playback")?;

    println!(
        "Solved {} in {} moves",
        args.level_path.display(),
        move_count
    );
    Ok(())
}
