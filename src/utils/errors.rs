//! Error handling for the Rebar VCS

use super::globals::HASH_SIZE;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RebarError {
    #[error("IO error - {0}")]
    Io(#[from] IoError),
    #[error("Hash error - {0}")]
    Hash(#[from] HashError),
    #[error("Object error - {0}")]
    Object(#[from] ObjectError),
    #[error("Input error - {0}")]
    Input(#[from] InputError),
}

#[derive(Debug, Error)]
pub enum IoError {
    #[error("Permission denied: {path}")]
    Permission {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("File or directory already exists: {path}")]
    AlreadyExists { path: String },
    #[error("File or directory not found: {path}")]
    NotFound { path: String },
    #[error("Path cannot be empty")]
    EmptyPath,
    #[error("Path does not exist: {path}")]
    PathNotExists { path: String },
    #[error("Path is not a file: {path}")]
    NotAFile { path: String },
    #[error("Path '{path}' is not inside a Rebar repository")]
    NoRepository { path: String },
    #[error("IO error: {0}")]
    Other(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Argument conflict: {message}")]
    ArgumentConflict { message: String },
    #[error("Missing required argument: {argument}")]
    MissingArgument { argument: String },
    #[error("Invalid argument '{argument}': {reason}")]
    InvalidArgument { argument: String, reason: String },
}

#[derive(Debug, Error)]
pub enum HashError {
    #[error("Incorrect hash length: expected {}, got {length} chars", HASH_SIZE)]
    InvalidLength { length: usize },
    #[error("Invalid character '{character}' at position {position}")]
    InvalidCharacter { position: usize, character: char },
}

#[derive(Debug, Error)]
pub enum ObjectError {
    /// Invalid object type (not blob, tree, or commit)
    #[error("Invalid object type '{found}' (expected blob, tree, or commit)")]
    InvalidType { found: String },
    /// Object header indicates a different length higher than actual content
    #[error(
        "Object length mismatch: header indicates {expected} bytes, but content is {actual:?} bytes"
    )]
    InvalidLength {
        expected: usize,
        actual: Option<usize>,
    },
    /// Missing or malformed object header
    #[error("Malformed object header: {reason}")]
    MalformedHeader { reason: String },
    /// Object content is corrupted or invalid
    #[error("Corrupted object content: {reason}")]
    CorruptedContent { reason: String },
    /// Failed to compress an object
    #[error("Failed to compress object: {reason}")]
    CompressionError { reason: String },
}
