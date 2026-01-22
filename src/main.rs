use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod levels;
mod playback;
mod generate;
mod render;
mod verify_all;
mod verify;

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
    },

    /// Render asciinema and SVG documentation
    Render {
        /// Path to the level JSON file
        level: PathBuf,

        /// Path to the playback JSON file
        playback: PathBuf,
    },
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
        }
        Command::Replay { level, playback } => {
            render::run_replay(&level, &playback)
        }
        Command::VerifyAll => verify_all::run_verify_all(),
        Command::GenerateLevelsJson { filter, dry_run } => {
            generate::run_generate_levels_json(filter.as_deref(), dry_run)
        }
        Command::Render { level, playback } => {
            render::run_render(&level, &playback)
        }
    }
}
