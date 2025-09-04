//! Error handling for the Rebar VCS

use std::fmt;

use crate::globals::HASH_SIZE;

#[derive(Debug)]
pub enum RebarError {
    Io(IoError),
    Hash(HashError),
    Object(ObjectError),
    Input(InputError),
}

impl fmt::Display for RebarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RebarError::Io(err) => write!(f, "IO error: {err}"),
            RebarError::Hash(err) => write!(f, "Hash error: {err}"),
            RebarError::Object(err) => write!(f, "Object error: {err}"),
            RebarError::Input(err) => write!(f, "Input error: {err}"),
        }
    }
}

impl std::error::Error for RebarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RebarError::Io(err) => Some(err),
            RebarError::Hash(err) => Some(err),
            RebarError::Object(err) => Some(err),
            RebarError::Input(err) => Some(err),
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
    EmptyPath,
    PathNotExists {
        path: String,
    },
    NotAFile {
        path: String,
    },
    NoRepository {
        path: String,
    },
    Other(std::io::Error),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::Permission { path, .. } => write!(f, "Permission denied: {path}"),
            IoError::AlreadyExists { path } => {
                write!(f, "File or directory already exists: {path}")
            }
            IoError::NotFound { path } => write!(f, "File or directory not found: {path}"),
            IoError::EmptyPath => write!(f, "Path cannot be empty"),
            IoError::PathNotExists { path } => write!(f, "Path does not exist: {path}"),
            IoError::NotAFile { path } => write!(f, "Path is not a file: {path}"),
            IoError::NoRepository { path } => {
                write!(f, "Path '{path}' is not inside a Rebar repository")
            }
            IoError::Other(err) => write!(f, "IO error: {err}"),
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
pub enum InputError {
    ArgumentConflict { message: String },
    MissingArgument { argument: String },
    InvalidArgument { argument: String, reason: String },
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::ArgumentConflict { message } => write!(f, "Argument conflict: {message}"),
            InputError::MissingArgument { argument } => {
                write!(f, "Missing required argument: {argument}")
            }
            InputError::InvalidArgument { argument, reason } => {
                write!(f, "Invalid argument '{argument}': {reason}")
            }
        }
    }
}

impl std::error::Error for InputError {}

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
                    "Incorrect hash length: expected {HASH_SIZE}, got {length} chars"
                )
            }
            HashError::InvalidCharacter {
                position,
                character,
            } => {
                write!(f, "Invalid character '{character}' at position {position}")
            }
            HashError::Conversion(msg) => write!(f, "Hash conversion error: {msg}"),
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
    /// Failed to compress an object
    CompressionError { reason: String },
}

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectError::InvalidType { found } => {
                write!(
                    f,
                    "Invalid object type '{found}' (expected blob, tree, or commit)"
                )
            }
            ObjectError::InvalidLength { expected, actual } => match actual {
                None => {
                    write!(
                        f,
                        "Object length mismatch: header indicates {expected} bytes, but content length is larger"
                    )
                }
                Some(actual) => {
                    write!(
                        f,
                        "Object length mismatch: header indicates {expected} bytes, but content is {actual} bytes"
                    )
                }
            },
            ObjectError::MalformedHeader { reason } => {
                write!(f, "Malformed object header: {reason}")
            }
            ObjectError::CorruptedContent { reason } => {
                write!(f, "Corrupted object content: {reason}")
            }
            ObjectError::InvalidFormat {
                object_type,
                reason,
            } => {
                write!(f, "Invalid {object_type} object format: {reason}")
            }
            ObjectError::MissingField { field, object_type } => {
                write!(
                    f,
                    "Missing required field '{field}' in {object_type} object"
                )
            }
            ObjectError::CompressionError { reason } => {
                write!(f, "Failed to compress object: {reason}")
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

impl From<InputError> for RebarError {
    fn from(err: InputError) -> Self {
        RebarError::Input(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io;

    #[test]
    fn test_hash_error_display() {
        let err = HashError::InvalidLength { length: 32 };
        assert_eq!(
            format!("{err}"),
            format!("Incorrect hash length: expected {HASH_SIZE}, got 32 chars")
        );

        let err = HashError::InvalidCharacter {
            position: 5,
            character: 'z',
        };
        assert_eq!(format!("{err}"), "Invalid character 'z' at position 5");

        let err = HashError::Conversion("test error".to_string());
        assert_eq!(format!("{err}"), "Hash conversion error: test error");
    }

    #[test]
    fn test_io_error_display() {
        let err = IoError::EmptyPath;
        assert_eq!(format!("{err}"), "Path cannot be empty");

        let err = IoError::PathNotExists {
            path: "/test/path".to_string(),
        };
        assert_eq!(format!("{err}"), "Path does not exist: /test/path");

        let err = IoError::NotAFile {
            path: "/test/dir".to_string(),
        };
        assert_eq!(format!("{err}"), "Path is not a file: /test/dir");

        let err = IoError::NoRepository {
            path: "/test/path".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Path '/test/path' is not inside a Rebar repository"
        );

        let err = IoError::AlreadyExists {
            path: "/test/file".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "File or directory already exists: /test/file"
        );

        let err = IoError::NotFound {
            path: "/test/missing".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "File or directory not found: /test/missing"
        );
    }

    #[test]
    fn test_input_error_display() {
        let err = InputError::ArgumentConflict {
            message: "conflicting args".to_string(),
        };
        assert_eq!(format!("{err}"), "Argument conflict: conflicting args");

        let err = InputError::MissingArgument {
            argument: "file".to_string(),
        };
        assert_eq!(format!("{err}"), "Missing required argument: file");

        let err = InputError::InvalidArgument {
            argument: "count".to_string(),
            reason: "must be positive".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Invalid argument 'count': must be positive"
        );
    }

    #[test]
    fn test_object_error_display() {
        let err = ObjectError::InvalidType {
            found: "unknown".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Invalid object type 'unknown' (expected blob, tree, or commit)"
        );

        let err = ObjectError::InvalidLength {
            expected: 100,
            actual: Some(50),
        };
        assert_eq!(
            format!("{err}"),
            "Object length mismatch: header indicates 100 bytes, but content is 50 bytes"
        );

        let err = ObjectError::InvalidLength {
            expected: 100,
            actual: None,
        };
        assert_eq!(
            format!("{err}"),
            "Object length mismatch: header indicates 100 bytes, but content length is larger"
        );

        let err = ObjectError::MalformedHeader {
            reason: "missing type".to_string(),
        };
        assert_eq!(format!("{err}"), "Malformed object header: missing type");

        let err = ObjectError::CorruptedContent {
            reason: "bad checksum".to_string(),
        };
        assert_eq!(format!("{err}"), "Corrupted object content: bad checksum");

        let err = ObjectError::CompressionError {
            reason: "zstd failed".to_string(),
        };
        assert_eq!(format!("{err}"), "Failed to compress object: zstd failed");
    }

    #[test]
    fn test_rebar_error_display() {
        let hash_err = HashError::InvalidLength { length: 32 };
        let rebar_err = RebarError::Hash(hash_err);
        assert!(format!("{rebar_err}").starts_with("Hash error:"));

        let io_err = IoError::EmptyPath;
        let rebar_err = RebarError::Io(io_err);
        assert!(format!("{rebar_err}").starts_with("IO error:"));

        let obj_err = ObjectError::InvalidType {
            found: "unknown".to_string(),
        };
        let rebar_err = RebarError::Object(obj_err);
        assert!(format!("{rebar_err}").starts_with("Object error:"));

        let input_err = InputError::MissingArgument {
            argument: "test".to_string(),
        };
        let rebar_err = RebarError::Input(input_err);
        assert!(format!("{rebar_err}").starts_with("Input error:"));
    }

    #[test]
    fn test_error_conversions() {
        // Test std::io::Error to IoError conversion
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let io_err: IoError = io_error.into();
        match io_err {
            IoError::NotFound { path } => assert_eq!(path, "unknown"),
            _ => panic!("Expected NotFound variant"),
        }

        // Test permission denied conversion
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let io_err: IoError = io_error.into();
        match io_err {
            IoError::Permission { path, .. } => assert_eq!(path, "unknown"),
            _ => panic!("Expected Permission variant"),
        }

        // Test already exists conversion
        let io_error = io::Error::new(io::ErrorKind::AlreadyExists, "already exists");
        let io_err: IoError = io_error.into();
        match io_err {
            IoError::AlreadyExists { path } => assert_eq!(path, "unknown"),
            _ => panic!("Expected AlreadyExists variant"),
        }

        // Test other error types conversion
        let io_error = io::Error::new(io::ErrorKind::InvalidData, "invalid data");
        let io_err: IoError = io_error.into();
        match io_err {
            IoError::Other(_) => (),
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_rebar_error_conversions() {
        let hash_err = HashError::InvalidLength { length: 32 };
        let rebar_err: RebarError = hash_err.into();
        match rebar_err {
            RebarError::Hash(_) => (),
            _ => panic!("Expected Hash variant"),
        }

        let io_err = IoError::EmptyPath;
        let rebar_err: RebarError = io_err.into();
        match rebar_err {
            RebarError::Io(_) => (),
            _ => panic!("Expected Io variant"),
        }

        let obj_err = ObjectError::InvalidType {
            found: "test".to_string(),
        };
        let rebar_err: RebarError = obj_err.into();
        match rebar_err {
            RebarError::Object(_) => (),
            _ => panic!("Expected Object variant"),
        }

        let input_err = InputError::MissingArgument {
            argument: "test".to_string(),
        };
        let rebar_err: RebarError = input_err.into();
        match rebar_err {
            RebarError::Input(_) => (),
            _ => panic!("Expected Input variant"),
        }
    }

    #[test]
    fn test_error_source() {
        // Test that permission error returns source
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let io_err = IoError::Permission {
            path: "test".to_string(),
            source: io_error,
        };
        assert!(io_err.source().is_some());

        // Test that other errors don't return source
        let io_err = IoError::EmptyPath;
        assert!(io_err.source().is_none());

        // Test RebarError source propagation
        let hash_err = HashError::InvalidLength { length: 32 };
        let rebar_err = RebarError::Hash(hash_err);
        assert!(rebar_err.source().is_some());
    }
}
