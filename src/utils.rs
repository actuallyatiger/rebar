//! Core commands for the Rebar VCS

use commands::HashError;

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
    if hex.len() != 64 {
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
