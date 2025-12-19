//! Hash object Command

use sha2::{Digest, Sha256};
use std::io::{Read, Write};

use crate::utils::globals::COMPRESSION_LEVEL;

use crate::utils::errors::{IoError, ObjectError, RebarError};

fn read_stdin() -> Result<String, RebarError> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| IoError::Other(e))?;
    Ok(buf)
}

fn read_file(path: &str) -> Result<String, RebarError> {
    std::fs::read_to_string(path).map_err(|e| {
        match e.kind() {
            std::io::ErrorKind::NotFound => IoError::NotFound {
                path: path.to_string(),
            },
            std::io::ErrorKind::PermissionDenied => IoError::Permission {
                path: path.to_string(),
                source: e,
            },
            _ => IoError::Other(e),
        }
        .into()
    })
}

pub fn hash_object(path: Option<&str>, stdin: bool, write: bool) -> Result<(), RebarError> {
    // get contents either from stdin or provided file
    let contents = if stdin {
        read_stdin()?
    } else {
        read_file(path.unwrap())?
    };

    // now we have the contents
    let encoded = match zstd::stream::encode_all(contents.as_bytes(), COMPRESSION_LEVEL as i32) {
        Ok(data) => data,
        Err(e) => {
            return Err(ObjectError::CompressionError {
                reason: e.to_string(),
            }
            .into());
        }
    };

    // add header (type + length)
    let header = format!("{} {}\n", "blob", encoded.len())
        .as_bytes()
        .to_vec();

    // hash the contents of the header and encoded
    let mut hasher = Sha256::new();
    hasher.update(&header);
    hasher.update(&encoded);
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);

    if write {
        // find the repository and then the path to the object
        let repo_path = crate::utils::find_repository(".").map_err(RebarError::from)?;
        let object_path = format!("{}/objects/{}", repo_path, hash_hex);

        // check that the object doesn't already exist
        if std::path::Path::new(&object_path).exists() {
            return Err(IoError::AlreadyExists { path: object_path }.into());
        }

        // create the file and write the contents to it
        let mut file = std::fs::File::create(object_path).map_err(RebarError::from)?;
        file.write_all(&header).map_err(RebarError::from)?;
        file.write_all(&encoded).map_err(RebarError::from)?;
    } else {
        // stdout
        println!("{}", hash_hex)
    }

    Ok(())
}
