//! Error handling for the Rebar VCS

use std::fmt;

#[derive(Debug)]
pub enum RebarError {
    Io(IoError),
    Hash(HashError),
    Object(ObjectError),
}

impl fmt::Display for RebarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RebarError::Io(err) => write!(f, "IO error: {}", err),
            RebarError::Hash(err) => write!(f, "Hash error: {}", err),
            RebarError::Object(err) => write!(f, "Object error: {}", err),
        }
    }
}

impl std::error::Error for RebarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RebarError::Io(err) => Some(err),
            RebarError::Hash(err) => Some(err),
            RebarError::Object(err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub enum IoError {
    Permission {
        path: String,
        source: std::io::Error,
    },
    AlreadyExists {
        path: String,
    },
    NotFound {
        path: String,
    },
    Other(std::io::Error),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::Permission { path, .. } => write!(f, "Permission denied: {}", path),
            IoError::AlreadyExists { path } => {
                write!(f, "File or directory already exists: {}", path)
            }
            IoError::NotFound { path } => write!(f, "File or directory not found: {}", path),
            IoError::Other(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for IoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IoError::Permission { source, .. } => Some(source),
            IoError::Other(err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum HashError {
    InvalidLength { length: usize },
    InvalidCharacter { position: usize, character: char },
    Conversion(String),
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashError::InvalidLength { length } => {
                write!(
                    f,
                    "Incorrect hash length: expected 64, got {} chars",
                    length
                )
            }
            HashError::InvalidCharacter {
                position,
                character,
            } => {
                write!(
                    f,
                    "Invalid character '{}' at position {}",
                    character, position
                )
            }
            HashError::Conversion(msg) => write!(f, "Hash conversion error: {}", msg),
        }
    }
}

impl std::error::Error for HashError {}

#[derive(Debug)]
pub enum ObjectError {
    /// Invalid object type (not blob, tree, or commit)
    InvalidType { found: String },
    /// Object header indicates a different length higher than actual content
    InvalidLength {
        expected: usize,
        actual: Option<usize>,
    },
    /// Missing or malformed object header
    MalformedHeader { reason: String },
    /// Object content is corrupted or invalid
    CorruptedContent { reason: String },
    /// Invalid object format
    InvalidFormat { object_type: String, reason: String },
    /// Missing required field in object
    MissingField { field: String, object_type: String },
}

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectError::InvalidType { found } => {
                write!(
                    f,
                    "Invalid object type '{}' (expected blob, tree, or commit)",
                    found
                )
            }
            ObjectError::InvalidLength { expected, actual } => match actual {
                None => {
                    return write!(
                        f,
                        "Object length mismatch: header indicates {} bytes, but content length is larger",
                        expected
                    );
                }
                Some(actual) => {
                    write!(
                        f,
                        "Object length mismatch: header indicates {} bytes, but content is {} bytes",
                        expected, actual
                    )
                }
            },
            ObjectError::MalformedHeader { reason } => {
                write!(f, "Malformed object header: {}", reason)
            }
            ObjectError::CorruptedContent { reason } => {
                write!(f, "Corrupted object content: {}", reason)
            }
            ObjectError::InvalidFormat {
                object_type,
                reason,
            } => {
                write!(f, "Invalid {} object format: {}", object_type, reason)
            }
            ObjectError::MissingField { field, object_type } => {
                write!(
                    f,
                    "Missing required field '{}' in {} object",
                    field, object_type
                )
            }
        }
    }
}

impl std::error::Error for ObjectError {}

// Conversion traits for easy error propagation
impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => IoError::Permission {
                path: "unknown".to_string(), // Default path when context is not available
                source: err,
            },
            std::io::ErrorKind::AlreadyExists => IoError::AlreadyExists {
                path: "unknown".to_string(),
            },
            std::io::ErrorKind::NotFound => IoError::NotFound {
                path: "unknown".to_string(),
            },
            _ => IoError::Other(err),
        }
    }
}

impl From<std::io::Error> for RebarError {
    fn from(err: std::io::Error) -> Self {
        RebarError::Io(IoError::from(err))
    }
}

impl From<IoError> for RebarError {
    fn from(err: IoError) -> Self {
        RebarError::Io(err)
    }
}

impl From<HashError> for RebarError {
    fn from(err: HashError) -> Self {
        RebarError::Hash(err)
    }
}

impl From<ObjectError> for RebarError {
    fn from(err: ObjectError) -> Self {
        RebarError::Object(err)
    }
}
