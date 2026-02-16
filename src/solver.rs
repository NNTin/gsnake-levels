use anyhow::{bail, Context, Result};
use gsnake_core::{engine::GameEngine, Direction, GameStatus, LevelDefinition, Position};
use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::Path,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum StatusCode {
    Playing,
    GameOver,
    LevelComplete,
    AllComplete,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct StateKey {
    snake: Vec<Position>,
    snake_dir: i8,
    food: Vec<Position>,
    floating_food: Vec<Position>,
    falling_food: Vec<Position>,
    stones: Vec<Position>,
    spikes: Vec<Position>,
    exit_is_solid: bool,
    food_collected: u32,
    status: StatusCode,
}

pub fn solve_level(level: LevelDefinition, max_depth: usize) -> Result<Vec<Direction>> {
    let engine = GameEngine::new(level).context("Invalid grid size in level definition")?;
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
            let Ok(processed) = next.process_move(direction) else {
                continue;
            };
            if !processed {
                continue;
            }
            let mut next_path = path.clone();
            next_path.push(direction);
            queue.push_back((next, next_path));
        }
    }

    bail!("No solution found")
}

pub fn load_level(level_path: &Path) -> Result<LevelDefinition> {
    let contents = fs::read_to_string(level_path)
        .with_context(|| format!("Failed to read level file: {}", level_path.display()))?;
    let level: LevelDefinition =
        serde_json::from_str(&contents).with_context(|| "Failed to parse level JSON")?;
    Ok(level)
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
