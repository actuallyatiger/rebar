//! Core commands for the Rebar VCS

use crate::errors::{HashError, IoError};

use crate::globals::HASH_SIZE;

/// Check if a hash contains an invalid character
fn check_invalid_char(hash: &str) -> Option<usize> {
    for (pos, c) in hash.chars().enumerate() {
        if !c.is_ascii_hexdigit() {
            return Some(pos);
        }
    }
    None
}

/// Validate hash is a valid 256-bit hexadecimal value
pub fn validate_hex(hex: &str) -> Result<(), HashError> {
    if hex.len() != HASH_SIZE as usize {
        Err(HashError::InvalidLength { length: hex.len() })
    } else if let Some(pos) = check_invalid_char(hex) {
        Err(HashError::InvalidCharacter {
            position: pos,
            character: hex.chars().nth(pos).unwrap(),
        })
    } else {
        Ok(())
    }
}

/// Validate a file path exists and is a file
pub fn validate_path(path: &str) -> Result<(), IoError> {
    if path.is_empty() {
        Err(IoError::EmptyPath)
    } else if !std::path::Path::new(path).exists() {
        Err(IoError::PathNotExists {
            path: path.to_string(),
        })
    } else if !std::path::Path::new(path).is_file() {
        Err(IoError::NotAFile {
            path: path.to_string(),
        })
    } else {
        Ok(())
    }
}

/// Path to the closest .rebar directory
pub fn find_repository(path: &str) -> Result<String, IoError> {
    let mut current = std::path::Path::new(path);

    // Check current directory first, then traverse up the directory tree
    loop {
        if current.join(".rebar").exists() {
            return Ok(current.join(".rebar").to_string_lossy().into_owned());
        }

        // Move to parent directory
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    Err(IoError::NoRepository {
        path: path.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{HashError, IoError};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_hex_valid() {
        // Valid 64-character hex string (SHA256)
        let valid_hash = "a".repeat(64);
        assert!(validate_hex(&valid_hash).is_ok());

        // Mixed case hex
        let mixed_case = "AbCdEf0123456789".repeat(4);
        assert!(validate_hex(&mixed_case).is_ok());
    }

    #[test]
    fn test_validate_hex_invalid_length() {
        // Too short
        let short_hash = "abc123";
        match validate_hex(&short_hash) {
            Err(HashError::InvalidLength { length }) => assert_eq!(length, 6),
            _ => panic!("Expected InvalidLength error"),
        }

        // Too long
        let long_hash = "a".repeat(65);
        match validate_hex(&long_hash) {
            Err(HashError::InvalidLength { length }) => assert_eq!(length, 65),
            _ => panic!("Expected InvalidLength error"),
        }

        // Empty string
        match validate_hex("") {
            Err(HashError::InvalidLength { length }) => assert_eq!(length, 0),
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn test_validate_hex_invalid_characters() {
        // Invalid character 'g'
        let invalid_char = "g".repeat(64);
        match validate_hex(&invalid_char) {
            Err(HashError::InvalidCharacter {
                position,
                character,
            }) => {
                assert_eq!(position, 0);
                assert_eq!(character, 'g');
            }
            _ => panic!("Expected InvalidCharacter error"),
        }

        // Invalid character in middle
        let mixed_invalid = "a".repeat(32) + "z" + &"b".repeat(31);
        match validate_hex(&mixed_invalid) {
            Err(HashError::InvalidCharacter {
                position,
                character,
            }) => {
                assert_eq!(position, 32);
                assert_eq!(character, 'z');
            }
            _ => panic!("Expected InvalidCharacter error"),
        }

        // Special characters
        let special_chars = "a".repeat(63) + "@";
        match validate_hex(&special_chars) {
            Err(HashError::InvalidCharacter {
                position,
                character,
            }) => {
                assert_eq!(position, 63);
                assert_eq!(character, '@');
            }
            _ => panic!("Expected InvalidCharacter error"),
        }
    }

    #[test]
    fn test_validate_path_empty() {
        match validate_path("") {
            Err(IoError::EmptyPath) => (),
            _ => panic!("Expected EmptyPath error"),
        }
    }

    #[test]
    fn test_validate_path_valid_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content")?;

        let path_str = file_path.to_str().unwrap();
        assert!(validate_path(path_str).is_ok());
        Ok(())
    }

    #[test]
    fn test_validate_path_nonexistent() {
        let nonexistent_path = "/path/that/does/not/exist/file.txt";
        match validate_path(nonexistent_path) {
            Err(IoError::PathNotExists { path }) => {
                assert_eq!(path, nonexistent_path);
            }
            _ => panic!("Expected PathNotExists error"),
        }
    }

    #[test]
    fn test_validate_path_directory() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path)?;

        let path_str = dir_path.to_str().unwrap();
        match validate_path(path_str) {
            Err(IoError::NotAFile { path }) => {
                assert_eq!(path, path_str);
            }
            _ => panic!("Expected NotAFile error"),
        }
        Ok(())
    }

    #[test]
    fn test_find_repository_exists() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let rebar_dir = temp_dir.path().join(".rebar");
        fs::create_dir(&rebar_dir)?;

        // Test from the directory containing .rebar
        let result = find_repository(temp_dir.path().to_str().unwrap())?;
        assert_eq!(result, rebar_dir.to_str().unwrap());

        // Test from a subdirectory
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir)?;
        let result = find_repository(sub_dir.to_str().unwrap())?;
        assert_eq!(result, rebar_dir.to_str().unwrap());

        Ok(())
    }

    #[test]
    fn test_find_repository_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir)?;

        let path_str = sub_dir.to_str().unwrap();
        match find_repository(path_str) {
            Err(IoError::NoRepository { path }) => {
                assert_eq!(path, path_str);
            }
            _ => panic!("Expected NoRepository error"),
        }
        Ok(())
    }

    #[test]
    fn test_find_repository_nested() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let rebar_dir = temp_dir.path().join(".rebar");
        fs::create_dir(&rebar_dir)?;

        // Create nested subdirectories
        let nested_path = temp_dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&nested_path)?;

        let result = find_repository(nested_path.to_str().unwrap())?;
        assert_eq!(result, rebar_dir.to_str().unwrap());

        Ok(())
    }

    #[test]
    fn test_check_invalid_char_valid() {
        assert_eq!(check_invalid_char("0123456789abcdefABCDEF"), None);
        assert_eq!(check_invalid_char(""), None);
        assert_eq!(check_invalid_char("f"), None);
    }

    #[test]
    fn test_check_invalid_char_invalid() {
        assert_eq!(check_invalid_char("g"), Some(0));
        assert_eq!(check_invalid_char("abcg"), Some(3));
        assert_eq!(check_invalid_char("@"), Some(0));
        assert_eq!(check_invalid_char("abc@def"), Some(3));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_validate_hex_random_valid_strings(
            s in "[0-9a-fA-F]{64}"
        ) {
            prop_assert!(validate_hex(&s).is_ok());
        }

        #[test]
        fn test_validate_hex_random_invalid_lengths(
            s in "[0-9a-fA-F]{0,63}|[0-9a-fA-F]{65,100}"
        ) {
            prop_assert!(validate_hex(&s).is_err());
        }

        #[test]
        fn test_validate_hex_random_invalid_chars(
            position in 0usize..64,
            invalid_char in "[g-zG-Z@#$%^&*()_+]"
        ) {
            // Generate exactly 64 characters with an invalid char at the specified position
            let mut chars: Vec<char> = (0..64).map(|i| if i % 2 == 0 { 'a' } else { 'b' }).collect();
            chars[position] = invalid_char.chars().next().unwrap();
            let s: String = chars.into_iter().collect();

            prop_assert_eq!(s.len(), 64);
            prop_assert!(validate_hex(&s).is_err());
        }

        #[test]
        fn test_validate_path_empty_always_fails(
            _dummy in any::<u8>() // Just to make it a property test
        ) {
            prop_assert!(validate_path("").is_err());
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_validate_hex_performance() {
        let valid_hash = "a".repeat(64);
        let start = Instant::now();

        // Run validation many times
        for _ in 0..10000 {
            let _ = validate_hex(&valid_hash);
        }

        let duration = start.elapsed();
        // Should complete 10k validations in reasonable time (< 100ms)
        assert!(
            duration.as_millis() < 100,
            "Hash validation too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_check_invalid_char_performance() {
        let valid_string = "0123456789abcdefABCDEF".repeat(3); // 66 chars
        let start = Instant::now();

        for _ in 0..10000 {
            let _ = check_invalid_char(&valid_string);
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 50,
            "Character validation too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_validate_hex_large_strings() {
        // Test with various sizes around the boundary
        let sizes = vec![1, 32, 63, 64, 65, 128, 256];

        for size in sizes {
            let test_string = "a".repeat(size);
            let start = Instant::now();

            let _ = validate_hex(&test_string);

            let duration = start.elapsed();
            assert!(
                duration.as_micros() < 1000,
                "Validation of size {size} too slow: {duration:?}",
            );
        }
    }
}
