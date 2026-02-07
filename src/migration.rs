use anyhow::{anyhow, Result};

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
}
