use gsnake_core::models::{LevelDefinition, Position};
use std::collections::HashSet;

/// Represents special mechanics present in a level
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct LevelMechanics {
    pub has_floating_food: bool,
    pub has_falling_food: bool,
    pub has_stones: bool,
    pub has_spikes: bool,
}

/// Represents detected obstacle patterns in a level
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ObstaclePattern {
    VerticalWall,
    HorizontalWall,
    Scattered,
    None,
}

/// Represents complexity metrics for a level
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ComplexityMetrics {
    pub obstacle_density: f32,
    pub food_count: usize,
    pub grid_area: i32,
}

/// Complete analysis result for a level
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LevelAnalysis {
    pub mechanics: LevelMechanics,
    pub pattern: ObstaclePattern,
    pub complexity: ComplexityMetrics,
}

/// Analyzes a level definition and returns structured analysis
#[allow(dead_code)]
pub fn analyze_level(level: &LevelDefinition) -> LevelAnalysis {
    let mechanics = detect_mechanics(level);
    let pattern = detect_obstacle_pattern(&level.obstacles);
    let complexity = calculate_complexity(level);

    LevelAnalysis {
        mechanics,
        pattern,
        complexity,
    }
}

/// Detects which special mechanics are present in the level
fn detect_mechanics(level: &LevelDefinition) -> LevelMechanics {
    LevelMechanics {
        has_floating_food: !level.floating_food.is_empty(),
        has_falling_food: !level.falling_food.is_empty(),
        has_stones: !level.stones.is_empty(),
        has_spikes: !level.spikes.is_empty(),
    }
}

/// Detects the primary obstacle pattern in the level
fn detect_obstacle_pattern(obstacles: &[Position]) -> ObstaclePattern {
    if obstacles.is_empty() {
        return ObstaclePattern::None;
    }

    // Count vertical and horizontal alignments
    let mut x_coords: HashSet<i32> = HashSet::new();
    let mut y_coords: HashSet<i32> = HashSet::new();

    for pos in obstacles {
        x_coords.insert(pos.x);
        y_coords.insert(pos.y);
    }

    // Check for vertical wall pattern (many obstacles sharing same x coordinate)
    let max_vertical_count = x_coords
        .iter()
        .map(|&x| obstacles.iter().filter(|p| p.x == x).count())
        .max()
        .unwrap_or(0);

    // Check for horizontal wall pattern (many obstacles sharing same y coordinate)
    let max_horizontal_count = y_coords
        .iter()
        .map(|&y| obstacles.iter().filter(|p| p.y == y).count())
        .max()
        .unwrap_or(0);

    // If 40% or more obstacles are aligned vertically
    if max_vertical_count >= (obstacles.len() * 4) / 10 {
        return ObstaclePattern::VerticalWall;
    }

    // If 40% or more obstacles are aligned horizontally
    if max_horizontal_count >= (obstacles.len() * 4) / 10 {
        return ObstaclePattern::HorizontalWall;
    }

    // Otherwise, obstacles are scattered
    ObstaclePattern::Scattered
}

/// Calculates complexity metrics for the level
fn calculate_complexity(level: &LevelDefinition) -> ComplexityMetrics {
    let grid_area = level.grid_size.width * level.grid_size.height;
    let obstacle_count = level.obstacles.len() as i32;
    let obstacle_density = if grid_area > 0 {
        obstacle_count as f32 / grid_area as f32
    } else {
        0.0
    };

    let food_count = level.food.len() + level.floating_food.len() + level.falling_food.len();

    ComplexityMetrics {
        obstacle_density,
        food_count,
        grid_area,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gsnake_core::models::{Direction, GridSize};

    fn create_test_level(
        obstacles: Vec<Position>,
        floating_food: Vec<Position>,
        falling_food: Vec<Position>,
        stones: Vec<Position>,
        spikes: Vec<Position>,
        grid_size: GridSize,
    ) -> LevelDefinition {
        LevelDefinition {
            id: 1,
            name: "Test Level".to_string(),
            difficulty: Some("easy".to_string()),
            grid_size,
            snake: vec![Position::new(0, 0)],
            obstacles,
            food: vec![],
            exit: Position::new(5, 5),
            snake_direction: Direction::East,
            floating_food,
            falling_food,
            stones,
            spikes,
            exit_is_solid: Some(true),
            total_food: Some(0),
        }
    }

    #[test]
    fn test_detect_mechanics_all_present() {
        let level = create_test_level(
            vec![],
            vec![Position::new(1, 1)],
            vec![Position::new(2, 2)],
            vec![Position::new(3, 3)],
            vec![Position::new(4, 4)],
            GridSize::new(10, 10),
        );

        let mechanics = detect_mechanics(&level);
        assert!(mechanics.has_floating_food);
        assert!(mechanics.has_falling_food);
        assert!(mechanics.has_stones);
        assert!(mechanics.has_spikes);
    }

    #[test]
    fn test_detect_mechanics_none_present() {
        let level = create_test_level(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            GridSize::new(10, 10),
        );

        let mechanics = detect_mechanics(&level);
        assert!(!mechanics.has_floating_food);
        assert!(!mechanics.has_falling_food);
        assert!(!mechanics.has_stones);
        assert!(!mechanics.has_spikes);
    }

    #[test]
    fn test_detect_mechanics_partial() {
        let level = create_test_level(
            vec![],
            vec![Position::new(1, 1)],
            vec![],
            vec![],
            vec![Position::new(4, 4)],
            GridSize::new(10, 10),
        );

        let mechanics = detect_mechanics(&level);
        assert!(mechanics.has_floating_food);
        assert!(!mechanics.has_falling_food);
        assert!(!mechanics.has_stones);
        assert!(mechanics.has_spikes);
    }

    #[test]
    fn test_detect_vertical_wall_pattern() {
        // Create a vertical wall at x=5
        let obstacles = vec![
            Position::new(5, 0),
            Position::new(5, 1),
            Position::new(5, 2),
            Position::new(5, 3),
            Position::new(5, 4),
            Position::new(5, 5),
            Position::new(5, 6),
            Position::new(5, 7),
            Position::new(2, 2), // Add some scattered obstacles
            Position::new(8, 3),
        ];

        let pattern = detect_obstacle_pattern(&obstacles);
        assert_eq!(pattern, ObstaclePattern::VerticalWall);
    }

    #[test]
    fn test_detect_horizontal_wall_pattern() {
        // Create a horizontal wall at y=3
        let obstacles = vec![
            Position::new(0, 3),
            Position::new(1, 3),
            Position::new(2, 3),
            Position::new(3, 3),
            Position::new(4, 3),
            Position::new(5, 3),
            Position::new(6, 3),
            Position::new(7, 3),
            Position::new(2, 1), // Add some scattered obstacles
            Position::new(4, 5),
        ];

        let pattern = detect_obstacle_pattern(&obstacles);
        assert_eq!(pattern, ObstaclePattern::HorizontalWall);
    }

    #[test]
    fn test_detect_scattered_pattern() {
        let obstacles = vec![
            Position::new(1, 1),
            Position::new(3, 2),
            Position::new(5, 4),
            Position::new(2, 6),
            Position::new(8, 3),
            Position::new(4, 8),
        ];

        let pattern = detect_obstacle_pattern(&obstacles);
        assert_eq!(pattern, ObstaclePattern::Scattered);
    }

    #[test]
    fn test_detect_no_obstacles() {
        let obstacles = vec![];
        let pattern = detect_obstacle_pattern(&obstacles);
        assert_eq!(pattern, ObstaclePattern::None);
    }

    #[test]
    fn test_calculate_complexity() {
        let obstacles = vec![
            Position::new(0, 0),
            Position::new(1, 1),
            Position::new(2, 2),
        ];

        let level = create_test_level(
            obstacles,
            vec![Position::new(3, 3)],
            vec![Position::new(4, 4)],
            vec![],
            vec![],
            GridSize::new(10, 10),
        );

        let complexity = calculate_complexity(&level);
        assert_eq!(complexity.grid_area, 100);
        assert_eq!(complexity.food_count, 2);
        assert_eq!(complexity.obstacle_density, 0.03);
    }

    #[test]
    fn test_calculate_complexity_high_density() {
        let mut obstacles = vec![];
        for i in 0..25 {
            obstacles.push(Position::new(i % 5, i / 5));
        }

        let level = create_test_level(
            obstacles,
            vec![],
            vec![],
            vec![],
            vec![],
            GridSize::new(10, 10),
        );

        let complexity = calculate_complexity(&level);
        assert_eq!(complexity.grid_area, 100);
        assert_eq!(complexity.obstacle_density, 0.25);
    }

    #[test]
    fn test_analyze_level_complete() {
        let obstacles = vec![
            Position::new(5, 0),
            Position::new(5, 1),
            Position::new(5, 2),
            Position::new(5, 3),
            Position::new(5, 4),
        ];

        let level = create_test_level(
            obstacles,
            vec![Position::new(1, 1)],
            vec![],
            vec![],
            vec![Position::new(8, 8)],
            GridSize::new(10, 10),
        );

        let analysis = analyze_level(&level);

        assert!(analysis.mechanics.has_floating_food);
        assert!(!analysis.mechanics.has_falling_food);
        assert!(!analysis.mechanics.has_stones);
        assert!(analysis.mechanics.has_spikes);

        assert_eq!(analysis.pattern, ObstaclePattern::VerticalWall);

        assert_eq!(analysis.complexity.grid_area, 100);
        assert_eq!(analysis.complexity.food_count, 1);
        assert_eq!(analysis.complexity.obstacle_density, 0.05);
    }
}
