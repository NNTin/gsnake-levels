use gsnake_levels::verify::verify_level;
use std::path::Path;

#[test]
fn test_verify_level_easy_001_succeeds() {
    let level_path = Path::new("levels/easy/level_001.json");
    let playback_path = Path::new("playbacks/easy/level_001.json");

    if level_path.exists() && playback_path.exists() {
        let result = verify_level(level_path, playback_path);
        assert!(
            result.is_ok(),
            "verify_level should succeed for valid level+playback pair: {:?}",
            result.unwrap_err()
        );
    }
}

#[test]
fn test_verify_level_does_not_mutate_files() {
    let level_path = Path::new("levels/easy/level_001.json");
    let playback_path = Path::new("playbacks/easy/level_001.json");

    if level_path.exists() && playback_path.exists() {
        let level_content_before = std::fs::read_to_string(level_path).unwrap();
        let playback_content_before = std::fs::read_to_string(playback_path).unwrap();

        let _ = verify_level(level_path, playback_path);

        let level_content_after = std::fs::read_to_string(level_path).unwrap();
        let playback_content_after = std::fs::read_to_string(playback_path).unwrap();

        assert_eq!(
            level_content_before, level_content_after,
            "Level file should not be mutated"
        );
        assert_eq!(
            playback_content_before, playback_content_after,
            "Playback file should not be mutated"
        );
    }
}

#[test]
fn test_verify_level_with_multiple_levels() {
    let test_cases = vec![
        (
            "levels/easy/level_001.json",
            "playbacks/easy/level_001.json",
        ),
        (
            "levels/easy/level_002.json",
            "playbacks/easy/level_002.json",
        ),
        (
            "levels/easy/level_003.json",
            "playbacks/easy/level_003.json",
        ),
    ];

    for (level_path, playback_path) in test_cases {
        let level_path = Path::new(level_path);
        let playback_path = Path::new(playback_path);

        if level_path.exists() && playback_path.exists() {
            let result = verify_level(level_path, playback_path);
            assert!(
                result.is_ok(),
                "verify_level should succeed for {} + {}: {:?}",
                level_path.display(),
                playback_path.display(),
                result.as_ref().unwrap_err()
            );
        }
    }
}
