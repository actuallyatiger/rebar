//! Common types for Rebar

use std::str::FromStr;

use super::errors::{ObjectError, RebarError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    // TODO: Implement other object types
    // Tree,
    // Commit,
}

impl FromStr for ObjectType {
    type Err = RebarError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            // "tree" => Ok(ObjectType::Tree),
            // "commit" => Ok(ObjectType::Commit),
            _ => Err(RebarError::Object(ObjectError::InvalidType {
                found: s.to_string(),
            })),
        }
    }
}
