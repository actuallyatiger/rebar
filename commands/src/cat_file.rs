//! Print a .rebar object

use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use utils::errors::{IoError, ObjectError, RebarError};
use utils::types::ObjectType;

use utils::globals::FILE_SIZE_LIMIT;

fn parse_header(header_line: &str) -> Result<(ObjectType, usize), RebarError> {
    let mut parts = header_line.trim().split_whitespace();

    let object_type_str = parts.next().ok_or_else(|| ObjectError::MalformedHeader {
        reason: "Missing object type".to_string(),
    })?;

    let size_str = parts.next().ok_or_else(|| ObjectError::MalformedHeader {
        reason: "Missing size".to_string(),
    })?;

    let object_type = ObjectType::from_str(object_type_str)?;
    let size = size_str
        .parse::<usize>()
        .map_err(|_| ObjectError::MalformedHeader {
            reason: format!("Invalid size: {}", size_str),
        })?;

    Ok((object_type, size))
}

pub fn cat_file(hash: &str) -> Result<(), RebarError> {
    // find the repository and file
    let repo_path = utils::find_repository(".").map_err(RebarError::from)?;
    let path = format!("{}/objects/{}", repo_path, hash);
    let file = File::open(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => RebarError::Io(IoError::NotFound { path: path.clone() }),
        std::io::ErrorKind::PermissionDenied => RebarError::Io(IoError::Permission {
            path: path.clone(),
            source: e,
        }),
        _ => RebarError::Io(IoError::Other(e)),
    })?;

    let mut reader = BufReader::new(file);
    let mut header_line = String::new();
    reader.read_line(&mut header_line)?;

    let (object_type, size) = parse_header(&header_line)?;

    if size > FILE_SIZE_LIMIT {
        return Err(ObjectError::InvalidLength {
            expected: FILE_SIZE_LIMIT,
            actual: Some(size),
        }
        .into());
    }

    let mut content = vec![0; size];
    let bytes_read = reader.read(&mut content)?;

    // Check if we read the expected amount
    if bytes_read != size {
        return Err(ObjectError::InvalidLength {
            expected: size,
            actual: Some(bytes_read),
        }
        .into());
    }

    // Verify no extra content
    let mut extra = [0u8; 1];
    if reader.read(&mut extra)? > 0 {
        return Err(ObjectError::InvalidLength {
            expected: size,
            actual: None, // We know there's at least one extra byte
        }
        .into());
    }

    match object_type {
        ObjectType::Blob => {
            let decompressed =
                zstd::decode_all(&content[..]).map_err(|e| ObjectError::CorruptedContent {
                    reason: format!("Decompression failed: {}", e),
                })?;

            print!("{}", String::from_utf8_lossy(&decompressed));
        }
    }

    Ok(())
}
