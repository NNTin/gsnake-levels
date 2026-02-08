use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod analysis;
mod generate;
mod levels;
mod migration;
mod name_generator;
mod playback;
mod playback_generator;
mod render;
mod sync_metadata;
mod toml_generator;
mod validate_levels_toml;
mod verify;
mod verify_all;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Verify that a level is solvable using its playback file
    Verify {
        /// Path to the level JSON file
        level: PathBuf,

        /// Optional explicit playback file path
        #[arg(long)]
        playback: Option<PathBuf>,
    },

    /// Replay a level solution visually in the terminal
    Replay {
        /// Path to the level JSON file
        level: PathBuf,

        /// Path to the playback JSON file
        playback: PathBuf,
    },

    /// Verify all levels in all difficulty folders
    VerifyAll,

    /// Aggregate levels into a single levels.json on stdout
    GenerateLevelsJson {
        /// Optional difficulty filter, e.g. "easy,medium"
        #[arg(long)]
        filter: Option<String>,

        /// Dry run: do not output JSON
        #[arg(long)]
        dry_run: bool,

        /// Disable automatic metadata sync before aggregation
        #[arg(long)]
        no_sync: bool,
    },

    /// Render asciinema and SVG documentation
    Render {
        /// Path to the level JSON file
        level: PathBuf,

        /// Path to the playback JSON file
        playback: PathBuf,
    },

    /// Sync level metadata (names, levels.toml, playbacks)
    SyncMetadata {
        /// Optional difficulty filter (easy, medium, or hard)
        #[arg(long)]
        difficulty: Option<String>,
    },

    /// Validate levels.toml files for all difficulties
    ValidateLevelsToml,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Verify { level, playback } => {
            let playback_path = verify::resolve_playback_path(&level, playback)
                .with_context(|| "Failed to resolve playback path")?;
            let result = verify::verify_level(&level, &playback_path);
            let solved = result.is_ok();
            levels::update_solved_status(&level, solved)
                .with_context(|| "Failed to update levels.toml metadata")?;
            result
        },
        Command::Replay { level, playback } => render::run_replay(&level, &playback),
        Command::VerifyAll => verify_all::run_verify_all(),
        Command::GenerateLevelsJson {
            filter,
            dry_run,
            no_sync,
        } => {
            let sync = !no_sync;
            generate::run_generate_levels_json(filter.as_deref(), dry_run, sync)
        },
        Command::Render { level, playback } => render::run_render(&level, &playback),
        Command::SyncMetadata { difficulty } => {
            let summary = sync_metadata::sync_metadata(difficulty.as_deref())?;
            println!("\nSync completed successfully:");
            println!("  - Generated {} names", summary.names_generated);
            println!(
                "  - Updated {} levels.toml files",
                summary.toml_files_updated
            );
            println!("  - Created {} playbacks", summary.playbacks_created);
            Ok(())
        },
        Command::ValidateLevelsToml => validate_levels_toml::run_validate_levels_toml(),
    }
}
