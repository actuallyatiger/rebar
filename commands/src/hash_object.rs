//! Hash object Command

use sha2::{Digest, Sha256};
use std::io::{Read, Write};

use utils::errors::{IoError, ObjectError, RebarError};

fn read_stdin() -> String {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .expect("Failed to read from stdin");
    buf
}

fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).expect("Failed to read file")
}

pub fn hash_object(path: Option<&str>, stdin: bool, write: bool) -> Result<(), RebarError> {
    /* Steps:
    1. if stdin, read, else get the file
    2. use zstd to compress the body
    3. use sha256 to hash the contents
    4. if write, write the object to the current repository,
    else output to terminal */

    let contents = if stdin {
        read_stdin()
    } else {
        read_file(path.unwrap())
    };

    // now we have the contents
    let encoded = match zstd::stream::encode_all(contents.as_bytes(), 3) {
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

    if write {
        // hash the contents of the header and encoded
        let mut hasher = Sha256::new();
        hasher.update(&header);
        hasher.update(&encoded);
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);

        // find the repository and then the path to the object
        let repo_path = utils::find_repository(".").map_err(RebarError::from)?;
        println!("DEBUG: Found repository: {}", repo_path);
        let object_path = format!("{}/objects/{}", repo_path, hash_hex);
        println!("DEBUG: Object path: {}", object_path);

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
        let header_str = String::from_utf8_lossy(&header);
        let encoded_str = String::from_utf8_lossy(&encoded);
        println!("{}{}", header_str, encoded_str);
    }

    Ok(())
}
