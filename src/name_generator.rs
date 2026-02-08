use crate::analysis::{analyze_level, LevelAnalysis, ObstaclePattern};
use gsnake_core::models::LevelDefinition;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

/// Generates a creative name for a level based on its analysis
#[allow(dead_code)]
pub fn generate_name(analysis: &LevelAnalysis, used_names: &mut HashSet<String>) -> String {
    let mut name_parts = Vec::new();

    // Priority 1: Special mechanics
    if analysis.mechanics.has_floating_food {
        name_parts.push("Floating");
    }
    if analysis.mechanics.has_falling_food {
        name_parts.push("Falling");
    }
    if analysis.mechanics.has_stones {
        name_parts.push("Stone");
    }
    if analysis.mechanics.has_spikes {
        name_parts.push("Spike");
    }

    // Priority 2: Obstacle patterns
    let pattern_word = match analysis.pattern {
        ObstaclePattern::VerticalWall => Some("Tower"),
        ObstaclePattern::HorizontalWall => Some("Bridge"),
        ObstaclePattern::Scattered => {
            // Only use "Islands" if there are scattered obstacles
            if analysis.complexity.obstacle_density > 0.0 {
                Some("Islands")
            } else {
                None
            }
        },
        ObstaclePattern::None => None,
    };

    if let Some(pattern) = pattern_word {
        name_parts.push(pattern);
    }

    // Priority 3: Complexity indicators
    if analysis.complexity.obstacle_density > 0.15 {
        name_parts.push("Dense");
    } else if analysis.complexity.food_count > 5 {
        name_parts.push("Feast");
    }

    // If we have no parts yet, use a generic name based on complexity
    if name_parts.is_empty() {
        if analysis.complexity.obstacle_density > 0.1 {
            name_parts.push("Maze");
        } else {
            name_parts.push("Simple");
        }
    }

    // Ensure name is 1-4 words (trim if needed)
    if name_parts.len() > 4 {
        name_parts.truncate(4);
    }

    // Create base name
    let mut name = name_parts.join(" ");

    // Ensure uniqueness by appending numbers if needed
    let mut counter = 1;
    let base_name = name.clone();
    while used_names.contains(&name) {
        counter += 1;
        name = format!("{} {}", base_name, counter);
    }

    used_names.insert(name.clone());
    name
}

/// Updates a level JSON file with a generated name
#[allow(dead_code)]
pub fn update_level_name(file_path: &Path) -> io::Result<()> {
    // Read the JSON file
    let contents = fs::read_to_string(file_path)?;
    let mut level: serde_json::Value = serde_json::from_str(&contents)?;

    // Parse as LevelDefinition for analysis
    let level_def: LevelDefinition = serde_json::from_str(&contents)?;

    // Analyze the level
    let analysis = analyze_level(&level_def);

    // Generate name (use a temporary set since we're processing one level)
    let mut used_names = HashSet::new();
    let new_name = generate_name(&analysis, &mut used_names);

    // Update the name field
    if let Some(obj) = level.as_object_mut() {
        obj.insert("name".to_string(), serde_json::Value::String(new_name));
    }

    // Write back to file with pretty formatting
    let updated_json = serde_json::to_string_pretty(&level)?;
    fs::write(file_path, updated_json)?;

    Ok(())
}

/// Generates names for all levels in a directory, ensuring uniqueness
#[allow(dead_code)]
pub fn generate_names_for_directory(
    dir_path: &Path,
    used_names: &mut HashSet<String>,
) -> io::Result<Vec<(String, String)>> {
    let mut results = Vec::new();

    // Read all JSON files in the directory
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            // Read and parse the level
            let contents = fs::read_to_string(&path)?;
            let level_def: LevelDefinition = serde_json::from_str(&contents)?;

            // Analyze and generate name
            let analysis = analyze_level(&level_def);
            let new_name = generate_name(&analysis, used_names);

            // Update the JSON file
            let mut level: serde_json::Value = serde_json::from_str(&contents)?;
            if let Some(obj) = level.as_object_mut() {
                obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(new_name.clone()),
                );
            }

            // Write back
            let updated_json = serde_json::to_string_pretty(&level)?;
            fs::write(&path, updated_json)?;

            results.push((path.display().to_string(), new_name));
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{ComplexityMetrics, LevelMechanics};
    use std::collections::HashSet;

    fn create_analysis(
        has_floating: bool,
        has_falling: bool,
        has_stones: bool,
        has_spikes: bool,
        pattern: ObstaclePattern,
        density: f32,
        food_count: usize,
    ) -> LevelAnalysis {
        LevelAnalysis {
            mechanics: LevelMechanics {
                has_floating_food: has_floating,
                has_falling_food: has_falling,
                has_stones,
                has_spikes,
            },
            pattern,
            complexity: ComplexityMetrics {
                obstacle_density: density,
                food_count,
                grid_area: 100,
            },
        }
    }

    #[test]
    fn test_generate_name_with_floating_and_spikes() {
        let analysis =
            create_analysis(true, false, false, true, ObstaclePattern::Scattered, 0.1, 3);
        let mut used = HashSet::new();
        let name = generate_name(&analysis, &mut used);

        assert!(name.contains("Floating"));
        assert!(name.contains("Spike"));
        assert!(used.contains(&name));
    }

    #[test]
    fn test_generate_name_with_pattern() {
        let analysis = create_analysis(
            false,
            false,
            false,
            false,
            ObstaclePattern::VerticalWall,
            0.1,
            2,
        );
        let mut used = HashSet::new();
        let name = generate_name(&analysis, &mut used);

        assert!(name.contains("Tower"));
    }

    #[test]
    fn test_generate_name_uniqueness() {
        let analysis = create_analysis(true, false, false, false, ObstaclePattern::None, 0.05, 2);
        let mut used = HashSet::new();

        let name1 = generate_name(&analysis, &mut used);
        let name2 = generate_name(&analysis, &mut used);

        assert_ne!(name1, name2);
        assert!(used.contains(&name1));
        assert!(used.contains(&name2));
    }

    #[test]
    fn test_generate_name_with_high_density() {
        let analysis = create_analysis(
            false,
            false,
            false,
            false,
            ObstaclePattern::Scattered,
            0.20,
            2,
        );
        let mut used = HashSet::new();
        let name = generate_name(&analysis, &mut used);

        assert!(name.contains("Dense") || name.contains("Islands"));
    }

    #[test]
    fn test_generate_name_with_many_food() {
        let analysis = create_analysis(false, false, false, false, ObstaclePattern::None, 0.05, 8);
        let mut used = HashSet::new();
        let name = generate_name(&analysis, &mut used);

        assert!(name.contains("Feast"));
    }

    #[test]
    fn test_generate_name_simple_level() {
        let analysis = create_analysis(false, false, false, false, ObstaclePattern::None, 0.02, 1);
        let mut used = HashSet::new();
        let name = generate_name(&analysis, &mut used);

        assert!(name.contains("Simple"));
    }

    #[test]
    fn test_generate_name_all_mechanics() {
        let analysis = create_analysis(
            true,
            true,
            true,
            true,
            ObstaclePattern::HorizontalWall,
            0.1,
            3,
        );
        let mut used = HashSet::new();
        let name = generate_name(&analysis, &mut used);

        // Should be limited to 4 words max
        let word_count = name.split_whitespace().count();
        assert!(word_count <= 4);
    }

    #[test]
    fn test_generate_name_horizontal_wall() {
        let analysis = create_analysis(
            false,
            false,
            false,
            false,
            ObstaclePattern::HorizontalWall,
            0.1,
            2,
        );
        let mut used = HashSet::new();
        let name = generate_name(&analysis, &mut used);

        assert!(name.contains("Bridge"));
    }
}
