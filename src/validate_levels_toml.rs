use anyhow::Result;
use gsnake_core::models::LevelDefinition;
use std::{fs, path::Path, process};

use crate::levels::{find_levels_root, LevelsToml, DEFAULT_DIFFICULTIES};

/// Exit codes for validation failures
const EXIT_CODE_VALIDATION_ERROR: i32 = 1;
const EXIT_CODE_IO_ERROR: i32 = 2;
const EXIT_CODE_PARSE_ERROR: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationIssueKind {
    Io,
    Parse,
    Validation,
}

impl ValidationIssueKind {
    fn label(self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::Parse => "parse",
            Self::Validation => "validation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationIssue {
    kind: ValidationIssueKind,
    message: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    fn push(&mut self, kind: ValidationIssueKind, message: impl Into<String>) {
        self.issues.push(ValidationIssue {
            kind,
            message: message.into(),
        });
    }

    fn extend(&mut self, mut other: ValidationReport) {
        self.issues.append(&mut other.issues);
    }

    fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    fn exit_code(&self) -> i32 {
        if self
            .issues
            .iter()
            .any(|issue| issue.kind == ValidationIssueKind::Parse)
        {
            EXIT_CODE_PARSE_ERROR
        } else if self
            .issues
            .iter()
            .any(|issue| issue.kind == ValidationIssueKind::Io)
        {
            EXIT_CODE_IO_ERROR
        } else {
            EXIT_CODE_VALIDATION_ERROR
        }
    }

    fn format_for_stderr(&self) -> String {
        let mut output = format!("Validation failed with {} issue(s):", self.issues.len());
        for (index, issue) in self.issues.iter().enumerate() {
            output.push_str(&format!(
                "\n  {}. [{}] {}",
                index + 1,
                issue.kind.label(),
                issue.message
            ));
        }

        output
    }
}

pub fn run_validate_levels_toml() -> Result<()> {
    let report = validate_all_levels_toml()?;

    if report.is_empty() {
        println!("✓ All levels.toml files are valid");
        return Ok(());
    }

    eprintln!("{}", report.format_for_stderr());
    process::exit(report.exit_code());
}

fn validate_all_levels_toml() -> Result<ValidationReport> {
    let levels_root = find_levels_root()?;
    Ok(validate_all_levels_toml_with_root(&levels_root))
}

fn validate_all_levels_toml_with_root(levels_root: &Path) -> ValidationReport {
    let mut report = ValidationReport::default();

    for difficulty in DEFAULT_DIFFICULTIES {
        let difficulty_dir = levels_root.join(difficulty);
        report.extend(validate_difficulty_levels_toml(&difficulty_dir, difficulty));
    }

    report
}

fn validate_difficulty_levels_toml(difficulty_dir: &Path, difficulty: &str) -> ValidationReport {
    let mut report = ValidationReport::default();
    let levels_toml_path = difficulty_dir.join("levels.toml");

    // Check that levels.toml exists
    if !levels_toml_path.exists() {
        report.push(
            ValidationIssueKind::Io,
            format!(
                "levels.toml not found for difficulty '{}': {}",
                difficulty,
                levels_toml_path.display()
            ),
        );
        return report;
    }

    // Parse levels.toml
    let levels_toml = match parse_levels_toml(&levels_toml_path, difficulty) {
        Ok(levels_toml) => levels_toml,
        Err(issue) => {
            report.issues.push(issue);
            return report;
        },
    };

    // Validate each level entry
    for (index, level_entry) in levels_toml.level.iter().enumerate() {
        let Some(file_name) = level_entry.file.as_ref() else {
            report.push(
                ValidationIssueKind::Validation,
                format!(
                    "Missing 'file' field for difficulty '{}' at entry index {} in {}",
                    difficulty,
                    index,
                    levels_toml_path.display()
                ),
            );
            continue;
        };

        let level_json_path = difficulty_dir.join(file_name);

        // Check that JSON file exists
        if !level_json_path.exists() {
            report.push(
                ValidationIssueKind::Io,
                format!(
                    "Referenced level JSON file does not exist: {} (from {})",
                    level_json_path.display(),
                    levels_toml_path.display()
                ),
            );
            continue;
        }

        // Parse JSON file as LevelDefinition
        if let Some(issue) = validate_level_json(&level_json_path) {
            report.issues.push(issue);
        }
    }

    report
}

fn parse_levels_toml(
    path: &Path,
    difficulty: &str,
) -> std::result::Result<LevelsToml, ValidationIssue> {
    let contents = fs::read_to_string(path).map_err(|error| ValidationIssue {
        kind: ValidationIssueKind::Io,
        message: format!(
            "Failed to read levels.toml for difficulty '{}': {} ({error})",
            difficulty,
            path.display()
        ),
    })?;

    toml::from_str::<LevelsToml>(&contents).map_err(|error| ValidationIssue {
        kind: ValidationIssueKind::Parse,
        message: format!(
            "Failed to parse levels.toml for difficulty '{}': {} ({error})",
            difficulty,
            path.display()
        ),
    })
}

fn validate_level_json(path: &Path) -> Option<ValidationIssue> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            return Some(ValidationIssue {
                kind: ValidationIssueKind::Io,
                message: format!(
                    "Failed to read level JSON file: {} ({error})",
                    path.display()
                ),
            });
        },
    };

    match serde_json::from_str::<LevelDefinition>(&content) {
        Ok(_) => None,
        Err(error) => Some(ValidationIssue {
            kind: ValidationIssueKind::Parse,
            message: format!(
                "Failed to parse level JSON as LevelDefinition: {} ({error})",
                path.display()
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::{LevelMeta, LevelsToml};
    use std::fs;
    use tempfile::TempDir;

    fn create_level_meta(file: Option<&str>) -> LevelMeta {
        LevelMeta {
            id: Some("test".to_string()),
            file: file.map(|value| value.to_string()),
            author: Some("test".to_string()),
            solved: Some(true),
            difficulty: Some("easy".to_string()),
            tags: Some(vec![]),
            description: Some("Test".to_string()),
        }
    }

    #[test]
    fn test_validate_missing_levels_toml() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let report = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].kind, ValidationIssueKind::Io);
        assert!(report.issues[0].message.contains("levels.toml not found"));
    }

    #[test]
    fn test_validate_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let levels_toml_path = difficulty_dir.join("levels.toml");
        fs::write(&levels_toml_path, "invalid toml content [[[").unwrap();

        let report = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].kind, ValidationIssueKind::Parse);
        assert!(report.issues[0]
            .message
            .contains("Failed to parse levels.toml"));
    }

    #[test]
    fn test_validate_missing_json_file() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let levels_toml = LevelsToml {
            level: vec![create_level_meta(Some("missing.json"))],
        };

        let levels_toml_path = difficulty_dir.join("levels.toml");
        crate::levels::write_levels_toml(&levels_toml_path, &levels_toml).unwrap();

        let report = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].kind, ValidationIssueKind::Io);
        assert!(report.issues[0].message.contains("does not exist"));
    }

    #[test]
    fn test_validate_invalid_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let level_json_path = difficulty_dir.join("test.json");
        fs::write(&level_json_path, "{invalid json}").unwrap();

        let levels_toml = LevelsToml {
            level: vec![create_level_meta(Some("test.json"))],
        };

        let levels_toml_path = difficulty_dir.join("levels.toml");
        crate::levels::write_levels_toml(&levels_toml_path, &levels_toml).unwrap();

        let report = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].kind, ValidationIssueKind::Parse);
        assert!(report.issues[0]
            .message
            .contains("Failed to parse level JSON"));
    }

    #[test]
    fn test_validate_difficulty_aggregates_multiple_issues() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        let invalid_json_path = difficulty_dir.join("invalid.json");
        fs::write(&invalid_json_path, "{invalid json}").unwrap();

        let levels_toml = LevelsToml {
            level: vec![
                create_level_meta(None),
                create_level_meta(Some("missing.json")),
                create_level_meta(Some("invalid.json")),
            ],
        };

        let levels_toml_path = difficulty_dir.join("levels.toml");
        crate::levels::write_levels_toml(&levels_toml_path, &levels_toml).unwrap();

        let report = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert_eq!(report.issues.len(), 3);
        assert_eq!(report.issues[0].kind, ValidationIssueKind::Validation);
        assert_eq!(report.issues[1].kind, ValidationIssueKind::Io);
        assert_eq!(report.issues[2].kind, ValidationIssueKind::Parse);
    }

    #[test]
    fn test_validate_all_levels_toml_aggregates_across_difficulties() {
        let temp_dir = TempDir::new().unwrap();
        let levels_root = temp_dir.path().join("levels");
        let easy_dir = levels_root.join("easy");
        let medium_dir = levels_root.join("medium");
        let hard_dir = levels_root.join("hard");
        fs::create_dir_all(&easy_dir).unwrap();
        fs::create_dir_all(&medium_dir).unwrap();
        fs::create_dir_all(&hard_dir).unwrap();

        let easy_toml = LevelsToml {
            level: vec![create_level_meta(Some("missing.json"))],
        };
        crate::levels::write_levels_toml(&easy_dir.join("levels.toml"), &easy_toml).unwrap();

        fs::write(medium_dir.join("levels.toml"), "invalid toml [[[").unwrap();

        let hard_level_json = r#"{
            "id": 1,
            "name": "Hard Level",
            "difficulty": "hard",
            "gridSize": {"width": 10, "height": 10},
            "snake": [{"x": 5, "y": 5}, {"x": 4, "y": 5}],
            "snakeDirection": "East",
            "obstacles": [],
            "food": [],
            "exit": {"x": 7, "y": 7},
            "floatingFood": [],
            "fallingFood": [],
            "stones": [],
            "spikes": [],
            "totalFood": 0
        }"#;
        fs::write(hard_dir.join("valid.json"), hard_level_json).unwrap();
        let hard_toml = LevelsToml {
            level: vec![create_level_meta(Some("valid.json"))],
        };
        crate::levels::write_levels_toml(&hard_dir.join("levels.toml"), &hard_toml).unwrap();

        let report = validate_all_levels_toml_with_root(&levels_root);
        assert_eq!(report.issues.len(), 2);
        assert_eq!(report.issues[0].kind, ValidationIssueKind::Io);
        assert_eq!(report.issues[1].kind, ValidationIssueKind::Parse);
    }

    #[test]
    fn test_validation_report_format_is_stable() {
        let mut report = ValidationReport::default();
        report.push(
            ValidationIssueKind::Io,
            "Referenced level JSON file does not exist: /tmp/missing.json (from /tmp/levels.toml)",
        );
        report.push(
            ValidationIssueKind::Parse,
            "Failed to parse level JSON as LevelDefinition: /tmp/invalid.json (expected value at line 1 column 1)",
        );

        let output = report.format_for_stderr();
        assert_eq!(
            output,
            "Validation failed with 2 issue(s):\n  1. [io] Referenced level JSON file does not exist: /tmp/missing.json (from /tmp/levels.toml)\n  2. [parse] Failed to parse level JSON as LevelDefinition: /tmp/invalid.json (expected value at line 1 column 1)"
        );
    }

    #[test]
    fn test_validate_valid_levels() {
        let temp_dir = TempDir::new().unwrap();
        let difficulty_dir = temp_dir.path().join("easy");
        fs::create_dir(&difficulty_dir).unwrap();

        // Create a valid level JSON
        let level_json = r#"{
            "id": 1,
            "name": "Test Level",
            "difficulty": "easy",
            "gridSize": {"width": 10, "height": 10},
            "snake": [{"x": 5, "y": 5}, {"x": 4, "y": 5}],
            "snakeDirection": "East",
            "obstacles": [],
            "food": [],
            "exit": {"x": 7, "y": 7},
            "floatingFood": [],
            "fallingFood": [],
            "stones": [],
            "spikes": [],
            "totalFood": 0
        }"#;

        let level_json_path = difficulty_dir.join("test.json");
        fs::write(&level_json_path, level_json).unwrap();

        let levels_toml = LevelsToml {
            level: vec![create_level_meta(Some("test.json"))],
        };

        let levels_toml_path = difficulty_dir.join("levels.toml");
        crate::levels::write_levels_toml(&levels_toml_path, &levels_toml).unwrap();

        let report = validate_difficulty_levels_toml(&difficulty_dir, "easy");
        assert!(report.issues.is_empty());
    }
}
