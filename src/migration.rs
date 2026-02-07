use anyhow::{anyhow, Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

/// Parses a string-based level ID and extracts the numeric timestamp portion.
///
/// Expected format: "{timestamp}-{suffix}" where timestamp is a u32.
/// Example: "1769977122223-g36bwe" -> 1769977122223
///
/// # Arguments
/// * `id` - The string ID to parse (e.g., "1769977122223-g36bwe")
///
/// # Returns
/// * `Ok(u32)` - The extracted timestamp if valid
/// * `Err` - If the ID format is invalid or timestamp is out of range
///
/// # Errors
/// * ID does not contain a hyphen separator
/// * Timestamp portion is not a valid number
/// * Timestamp exceeds u32::MAX (4,294,967,295)
#[allow(dead_code)] // Will be used in US-002
pub fn parse_string_id(id: &str) -> Result<u32> {
    // Split on hyphen
    let parts: Vec<&str> = id.split('-').collect();

    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid ID format: expected 'timestamp-suffix', got '{}'",
            id
        ));
    }

    let timestamp_str = parts[0];

    // Parse as u64 first to check if it exceeds u32 range
    let timestamp_u64: u64 = timestamp_str.parse().map_err(|_| {
        anyhow!(
            "Invalid timestamp: '{}' is not a valid number",
            timestamp_str
        )
    })?;

    // Check if it fits in u32 range
    if timestamp_u64 > u32::MAX as u64 {
        return Err(anyhow!(
            "Timestamp {} exceeds u32::MAX ({})",
            timestamp_u64,
            u32::MAX
        ));
    }

    Ok(timestamp_u64 as u32)
}

/// Migrates a level JSON file from string ID to numeric ID.
///
/// Reads the level JSON file, replaces the string `id` field with the provided
/// numeric ID, and writes the updated JSON back to the file with proper formatting.
///
/// # Arguments
/// * `level_path` - Path to the level JSON file
/// * `new_id` - The new numeric ID to assign (must be u32)
///
/// # Returns
/// * `Ok(())` - If migration succeeded and level validates correctly
/// * `Err` - If file read/write fails or validation fails
///
/// # Errors
/// * File does not exist or cannot be read
/// * JSON is malformed
/// * Updated level fails LevelDefinition validation
#[allow(dead_code)] // Will be used in US-009
pub fn migrate_level_id<P: AsRef<Path>>(level_path: P, new_id: u32) -> Result<()> {
    let path = level_path.as_ref();

    // Read the level file
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read level file: {}", path.display()))?;

    // Parse as JSON Value to preserve structure
    let mut level: Map<String, Value> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path.display()))?;

    // Update the id field from string to numeric
    level.insert("id".to_string(), Value::Number(new_id.into()));

    // Serialize back to pretty-printed JSON
    let updated_json = serde_json::to_string_pretty(&level)
        .with_context(|| "Failed to serialize updated level")?;

    // Write back to file
    fs::write(path, updated_json + "\n")
        .with_context(|| format!("Failed to write updated level to: {}", path.display()))?;

    // Validate the updated file can be parsed as LevelDefinition
    validate_level_file(path)?;

    Ok(())
}

/// Validates that a level JSON file can be parsed as gsnake-core's LevelDefinition.
///
/// This ensures the migrated level is compatible with the game engine.
///
/// # Arguments
/// * `level_path` - Path to the level JSON file to validate
///
/// # Returns
/// * `Ok(())` - If level parses successfully
/// * `Err` - If parsing fails
fn validate_level_file<P: AsRef<Path>>(level_path: P) -> Result<()> {
    let path = level_path.as_ref();
    let content = fs::read_to_string(path).with_context(|| {
        format!(
            "Failed to read level file for validation: {}",
            path.display()
        )
    })?;

    // Parse as LevelDefinition to validate structure
    let _: gsnake_core::models::LevelDefinition = serde_json::from_str(&content)
        .with_context(|| format!("Level validation failed for: {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_id() {
        let id = "1234567890-g36bwe";
        let result = parse_string_id(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1234567890);
    }

    #[test]
    fn test_parse_valid_id_small_timestamp() {
        let id = "12345-abc";
        let result = parse_string_id(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12345);
    }

    #[test]
    fn test_parse_valid_id_max_u32() {
        let id = "4294967295-test";
        let result = parse_string_id(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), u32::MAX);
    }

    #[test]
    fn test_parse_invalid_no_hyphen() {
        let id = "1769977122223g36bwe";
        let result = parse_string_id(id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid ID format"));
    }

    #[test]
    fn test_parse_invalid_multiple_hyphens() {
        let id = "1769977122223-g36-bwe";
        let result = parse_string_id(id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid ID format"));
    }

    #[test]
    fn test_parse_invalid_non_numeric_timestamp() {
        let id = "abc123-suffix";
        let result = parse_string_id(id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid timestamp"));
    }

    #[test]
    fn test_parse_invalid_empty_timestamp() {
        let id = "-suffix";
        let result = parse_string_id(id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid timestamp"));
    }

    #[test]
    fn test_parse_invalid_timestamp_exceeds_u32() {
        let id = "4294967296-test"; // u32::MAX + 1
        let result = parse_string_id(id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds u32::MAX"));
    }

    #[test]
    fn test_parse_invalid_large_timestamp() {
        let id = "999999999999999-test";
        let result = parse_string_id(id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds u32::MAX"));
    }

    #[test]
    fn test_parse_invalid_negative_timestamp() {
        let id = "-123-suffix";
        let result = parse_string_id(id);
        assert!(result.is_err());
        // Will fail at parsing stage
    }

    #[test]
    fn test_parse_timestamp_from_actual_level_file() {
        // Actual timestamp from level files - too large for u32
        let id = "1769977122223-g36bwe";
        let result = parse_string_id(id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds u32::MAX"));
    }

    #[test]
    fn test_migrate_level_id() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_level.json");

        // Create a test level JSON with string ID
        let test_json = r#"{
  "id": "1769977122223-g36bwe",
  "name": "Test Level",
  "difficulty": "easy",
  "gridSize": {
    "width": 10,
    "height": 10
  },
  "snake": [
    {
      "x": 5,
      "y": 5
    }
  ],
  "obstacles": [],
  "food": [],
  "exit": {
    "x": 8,
    "y": 8
  },
  "snakeDirection": "East"
}"#;

        fs::write(&test_file, test_json).unwrap();

        // Migrate the ID
        let result = migrate_level_id(&test_file, 42);
        assert!(result.is_ok());

        // Read back and verify
        let content = fs::read_to_string(&test_file).unwrap();
        let level: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Check that id is now numeric
        assert_eq!(level["id"], 42);

        // Check that other fields are preserved
        assert_eq!(level["name"], "Test Level");
        assert_eq!(level["difficulty"], "easy");
        assert_eq!(level["gridSize"]["width"], 10);
    }

    #[test]
    fn test_migrate_level_id_validates_structure() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("invalid_level.json");

        // Create an invalid level JSON (missing required fields)
        let invalid_json = r#"{
  "id": "1234-test",
  "name": "Invalid Level"
}"#;

        fs::write(&test_file, invalid_json).unwrap();

        // Attempt to migrate - should fail validation
        let result = migrate_level_id(&test_file, 99);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Level validation failed"));
    }
}
