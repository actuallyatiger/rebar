use crate::error::IoError;
/// Initialize a new .rebar repository
use std::fs;

fn create_dir_with_context(path: &str) -> Result<(), IoError> {
    fs::create_dir(path).map_err(|err| match err.kind() {
        std::io::ErrorKind::AlreadyExists => IoError::AlreadyExists {
            path: path.to_string(),
        },
        std::io::ErrorKind::PermissionDenied => IoError::Permission {
            path: path.to_string(),
            source: err,
        },
        std::io::ErrorKind::NotFound => IoError::NotFound {
            path: path.to_string(),
        },
        _ => IoError::Other(err),
    })
}

fn write_file_with_context(path: &str, content: &str) -> Result<(), IoError> {
    fs::write(path, content).map_err(|err| match err.kind() {
        std::io::ErrorKind::PermissionDenied => IoError::Permission {
            path: path.to_string(),
            source: err,
        },
        std::io::ErrorKind::NotFound => IoError::NotFound {
            path: path.to_string(),
        },
        _ => IoError::Other(err),
    })
}

pub fn init() -> Result<(), IoError> {
    // Create the main .rebar directory
    create_dir_with_context(".rebar")?;

    // Create subdirectories
    create_dir_with_context(".rebar/objects")?;
    create_dir_with_context(".rebar/pointers")?;

    // Write to .rebar/HEAD
    write_file_with_context(".rebar/HEAD", "ref: refs/heads/main\n")?;

    Ok(())
}
