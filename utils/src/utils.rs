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
    while let Some(parent) = current.parent() {
        if parent.join(".rebar").exists() {
            return Ok(parent.join(".rebar").to_string_lossy().into_owned());
        }
        current = parent;
    }
    Err(IoError::NoRepository {
        path: path.to_string(),
    })
}
