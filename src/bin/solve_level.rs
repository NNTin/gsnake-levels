use anyhow::{bail, Context, Result};
use clap::Parser;
use gsnake_core::{engine::GameEngine, Direction, GameStatus, LevelDefinition};
use serde::Serialize;
use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::PathBuf,
};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum StatusCode {
    Playing,
    GameOver,
    LevelComplete,
    AllComplete,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct StateKey {
    snake: Vec<gsnake_core::Position>,
    snake_dir: i8,
    food: Vec<gsnake_core::Position>,
    floating_food: Vec<gsnake_core::Position>,
    falling_food: Vec<gsnake_core::Position>,
    stones: Vec<gsnake_core::Position>,
    spikes: Vec<gsnake_core::Position>,
    exit_is_solid: bool,
    food_collected: u32,
    status: StatusCode,
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

fn solve_level(level: LevelDefinition, max_depth: usize) -> Result<Vec<Direction>> {
    let engine = GameEngine::new(level);
    let mut queue: VecDeque<(GameEngine, Vec<Direction>)> = VecDeque::new();
    let mut visited: HashSet<StateKey> = HashSet::new();

    queue.push_back((engine, Vec::new()));

    while let Some((engine, path)) = queue.pop_front() {
        if path.len() > max_depth {
            continue;
        }

        let status = engine.game_state().status;
        if status == GameStatus::LevelComplete || status == GameStatus::AllComplete {
            return Ok(path);
        }
        if status == GameStatus::GameOver {
            continue;
        }

        let key = state_key(&engine);
        if !visited.insert(key) {
            continue;
        }

        for direction in [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ] {
            let mut next = engine.clone();
            if !next.process_move(direction) {
                continue;
            }
            let _ = next.generate_frame();
            let mut next_path = path.clone();
            next_path.push(direction);
            queue.push_back((next, next_path));
        }
    }

    bail!("No solution found")
}

fn state_key(engine: &GameEngine) -> StateKey {
    let level_state = engine.level_state();
    let game_state = engine.game_state();

    StateKey {
        snake: level_state.snake.segments.clone(),
        snake_dir: direction_code(level_state.snake.direction),
        food: level_state.food.clone(),
        floating_food: level_state.floating_food.clone(),
        falling_food: level_state.falling_food.clone(),
        stones: level_state.stones.clone(),
        spikes: level_state.spikes.clone(),
        exit_is_solid: level_state.exit_is_solid,
        food_collected: game_state.food_collected,
        status: status_code(game_state.status),
    }
}

fn direction_code(direction: Option<Direction>) -> i8 {
    match direction {
        Some(Direction::North) => 0,
        Some(Direction::South) => 1,
        Some(Direction::East) => 2,
        Some(Direction::West) => 3,
        None => -1,
    }
}

fn status_code(status: GameStatus) -> StatusCode {
    match status {
        GameStatus::Playing => StatusCode::Playing,
        GameStatus::GameOver => StatusCode::GameOver,
        GameStatus::LevelComplete => StatusCode::LevelComplete,
        GameStatus::AllComplete => StatusCode::AllComplete,
    }
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::North => "Up",
        Direction::South => "Down",
        Direction::East => "Right",
        Direction::West => "Left",
    }
}

fn load_level(level_path: &PathBuf) -> Result<LevelDefinition> {
    let contents = fs::read_to_string(level_path)
        .with_context(|| format!("Failed to read level file: {}", level_path.display()))?;
    let level: LevelDefinition =
        serde_json::from_str(&contents).with_context(|| "Failed to parse level JSON")?;
    Ok(level)
}
