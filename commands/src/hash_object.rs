//! Create a .rebar object from file or stdin

use std::{
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    path::Path,
};

use utils::errors::{IoError, RebarError};
use sha2::{Digest, Sha256};

/// Hash object and optionally write to object database
pub fn hash_object(file_path: Option<&str>, from_stdin: bool, write_object: bool) -> Result<(), RebarError> {
    // Read content from file or stdin
    let content = if from_stdin {
        read_from_stdin()?
    } else if let Some(path) = file_path {
        read_from_file(path)?
    } else {
        return Err(RebarError::Io(IoError::Other(
            io::Error::new(io::ErrorKind::InvalidInput, "No input source specified")
        )));
    };

    // Create the blob header and content for hashing (Git format)
    let blob_content = format!("blob {}\0", content.len());
    let mut hash_input = blob_content.into_bytes();
    hash_input.extend_from_slice(&content);

    // Calculate SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(&hash_input);
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(hash_bytes);

    if write_object {
        write_object_to_filesystem(&hash_hex, &content)?;
    }

    // Print the hash (like Git does)
    println!("{}", hash_hex);

    Ok(())
}

fn read_from_stdin() -> Result<Vec<u8>, RebarError> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn read_from_file(path: &str) -> Result<Vec<u8>, RebarError> {
    fs::read(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => RebarError::Io(IoError::NotFound { 
            path: path.to_string() 
        }),
        io::ErrorKind::PermissionDenied => RebarError::Io(IoError::Permission {
            path: path.to_string(),
            source: e,
        }),
        _ => RebarError::Io(IoError::Other(e)),
    })
}

fn write_object_to_filesystem(hash: &str, content: &[u8]) -> Result<(), RebarError> {
    // Ensure .rebar/objects directory exists
    let objects_dir = ".rebar/objects";
    if !Path::new(objects_dir).exists() {
        return Err(RebarError::Io(IoError::NotFound {
            path: objects_dir.to_string(),
        }));
    }

    // Compress the content using zstd
    let compressed_content = zstd::encode_all(content, 3).map_err(|e| RebarError::Io(IoError::Other(
        io::Error::new(io::ErrorKind::Other, format!("Compression failed: {}", e))
    )))?;

    // Create object file path
    let object_path = format!("{}/{}", objects_dir, hash);
    
    // Write header + compressed content to file
    let file = File::create(&object_path).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => RebarError::Io(IoError::Permission {
            path: object_path.clone(),
            source: e,
        }),
        _ => RebarError::Io(IoError::Other(e)),
    })?;

    let mut writer = BufWriter::new(file);
    
    // Write header (blob compressed_size) - cat_file expects this to match the compressed content size
    writeln!(writer, "blob {}", compressed_content.len())?;
    
    // Write compressed content
    writer.write_all(&compressed_content)?;
    writer.flush()?;

    Ok(())
}