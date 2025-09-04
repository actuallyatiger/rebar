//! Common types for Rebar

use std::str::FromStr;

use crate::errors::{ObjectError, RebarError};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{ObjectError, RebarError};

    #[test]
    fn test_object_type_from_str_valid() {
        let obj_type = ObjectType::from_str("blob").unwrap();
        assert_eq!(obj_type, ObjectType::Blob);
    }

    #[test]
    fn test_object_type_from_str_invalid() {
        let result = ObjectType::from_str("invalid");
        assert!(result.is_err());

        match result {
            Err(RebarError::Object(ObjectError::InvalidType { found })) => {
                assert_eq!(found, "invalid");
            }
            _ => panic!("Expected InvalidType error"),
        }
    }

    #[test]
    fn test_object_type_from_str_case_sensitive() {
        // Test that "BLOB" is invalid (case sensitive)
        let result = ObjectType::from_str("BLOB");
        assert!(result.is_err());

        // Test that "Blob" is invalid (case sensitive)
        let result = ObjectType::from_str("Blob");
        assert!(result.is_err());
    }

    #[test]
    fn test_object_type_from_str_empty() {
        let result = ObjectType::from_str("");
        assert!(result.is_err());

        match result {
            Err(RebarError::Object(ObjectError::InvalidType { found })) => {
                assert_eq!(found, "");
            }
            _ => panic!("Expected InvalidType error"),
        }
    }

    #[test]
    fn test_object_type_debug() {
        let obj_type = ObjectType::Blob;
        assert_eq!(format!("{:?}", obj_type), "Blob");
    }

    #[test]
    fn test_object_type_equality() {
        let obj1 = ObjectType::Blob;
        let obj2 = ObjectType::Blob;
        assert_eq!(obj1, obj2);

        let obj3 = ObjectType::from_str("blob").unwrap();
        assert_eq!(obj1, obj3);
    }

    #[test]
    fn test_object_type_clone() {
        let obj1 = ObjectType::Blob;
        let obj2 = obj1.clone();
        assert_eq!(obj1, obj2);
    }
}
