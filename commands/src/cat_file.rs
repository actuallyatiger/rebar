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
    let mut parts = header_line.split_whitespace();

    let object_type_str = parts.next().ok_or_else(|| ObjectError::MalformedHeader {
        reason: "Missing object type".to_string(),
    })?;

    // Compressed size of the object
    let size_str = parts.next().ok_or_else(|| ObjectError::MalformedHeader {
        reason: "Missing size".to_string(),
    })?;

    let object_type = ObjectType::from_str(object_type_str)?;
    let size = size_str
        .parse::<usize>()
        .map_err(|_| ObjectError::MalformedHeader {
            reason: format!("Invalid size: {size_str}"),
        })?;

    Ok((object_type, size))
}

pub fn cat_file(hash: &str) -> Result<(), RebarError> {
    cat_file_from_path(hash, ".")
}

fn cat_file_from_path(hash: &str, start_path: &str) -> Result<(), RebarError> {
    // find the repository and file
    let repo_path = utils::find_repository(start_path).map_err(RebarError::from)?;
    let path = format!("{repo_path}/objects/{hash}");
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
                    reason: format!("Decompression failed: {e}"),
                })?;

            print!("{}", String::from_utf8_lossy(&decompressed));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    use utils::errors::{ObjectError, RebarError};
    use utils::types::ObjectType;

    // Helper function to create a test repository
    fn create_test_repository() -> Result<TempDir, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let rebar_dir = temp_dir.path().join(".rebar");
        fs::create_dir(&rebar_dir)?;
        let objects_dir = rebar_dir.join("objects");
        fs::create_dir(&objects_dir)?;
        Ok(temp_dir)
    }

    // Helper function to create a valid blob object file
    fn create_blob_object(
        temp_dir: &TempDir,
        hash: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let compressed_content = zstd::encode_all(content.as_bytes(), 3)?;
        let header = format!("blob {}\n", compressed_content.len());

        let objects_dir = temp_dir.path().join(".rebar").join("objects");
        let object_path = objects_dir.join(hash);

        let mut file = fs::File::create(object_path)?;
        file.write_all(header.as_bytes())?;
        file.write_all(&compressed_content)?;

        Ok(())
    }

    #[test]
    fn test_parse_header_valid() {
        let header = "blob 1024\n";
        let result = parse_header(header).unwrap();
        assert_eq!(result.0, ObjectType::Blob);
        assert_eq!(result.1, 1024);
    }

    #[test]
    fn test_parse_header_missing_type() {
        let header = "\n";
        match parse_header(header) {
            Err(RebarError::Object(ObjectError::MalformedHeader { reason })) => {
                assert_eq!(reason, "Missing object type");
            }
            _ => panic!("Expected MalformedHeader error for missing type"),
        }
    }

    #[test]
    fn test_parse_header_missing_size() {
        let header = "blob\n";
        match parse_header(header) {
            Err(RebarError::Object(ObjectError::MalformedHeader { reason })) => {
                assert_eq!(reason, "Missing size");
            }
            _ => panic!("Expected MalformedHeader error for missing size"),
        }
    }

    #[test]
    fn test_parse_header_invalid_type() {
        let header = "invalid 1024\n";
        match parse_header(header) {
            Err(RebarError::Object(ObjectError::InvalidType { found })) => {
                assert_eq!(found, "invalid");
            }
            _ => panic!("Expected InvalidType error"),
        }
    }

    #[test]
    fn test_parse_header_invalid_size() {
        let header = "blob notanumber\n";
        match parse_header(header) {
            Err(RebarError::Object(ObjectError::MalformedHeader { reason })) => {
                assert!(reason.contains("Invalid size: notanumber"));
            }
            _ => panic!("Expected MalformedHeader error for invalid size"),
        }
    }

    #[test]
    fn test_parse_header_negative_size() {
        let header = "blob -100\n";
        match parse_header(header) {
            Err(RebarError::Object(ObjectError::MalformedHeader { reason })) => {
                assert!(reason.contains("Invalid size: -100"));
            }
            _ => panic!("Expected MalformedHeader error for negative size"),
        }
    }

    #[test]
    fn test_parse_header_extra_whitespace() {
        let header = "  blob   1024  \n";
        let result = parse_header(header).unwrap();
        assert_eq!(result.0, ObjectType::Blob);
        assert_eq!(result.1, 1024);
    }

    #[test]
    fn test_cat_file_valid_blob() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "a".repeat(64);
        let content = "Hello, World!";

        create_blob_object(&temp_dir, &hash, content)?;

        // Use the temp directory path directly instead of changing working directory
        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_cat_file_nonexistent_hash() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "b".repeat(64);

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        match result {
            Err(RebarError::Io(utils::errors::IoError::NotFound { path })) => {
                assert!(path.contains(&hash));
            }
            _ => panic!("Expected NotFound error"),
        }
        Ok(())
    }

    #[test]
    fn test_cat_file_no_repository() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let hash = "c".repeat(64);

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        match result {
            Err(RebarError::Io(utils::errors::IoError::NoRepository { .. })) => {
                // Expected error
            }
            _ => panic!("Expected NoRepository error"),
        }
        Ok(())
    }

    #[test]
    fn test_cat_file_invalid_header() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "d".repeat(64);

        // Create a file with invalid header (invalid object type)
        let objects_dir = temp_dir.path().join(".rebar").join("objects");
        let object_path = objects_dir.join(&hash);
        let mut file = fs::File::create(object_path)?;
        file.write_all(b"invalid 100\n")?;
        file.write_all(b"some content")?;

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        match result {
            Err(RebarError::Object(ObjectError::InvalidType { found })) => {
                assert_eq!(found, "invalid");
            }
            _ => panic!("Expected InvalidType error, got: {:?}", result),
        }
        Ok(())
    }

    #[test]
    fn test_cat_file_malformed_header() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "x".repeat(64);

        // Create a file with truly malformed header (no size)
        let objects_dir = temp_dir.path().join(".rebar").join("objects");
        let object_path = objects_dir.join(&hash);
        let mut file = fs::File::create(object_path)?;
        file.write_all(b"blob\n")?;
        file.write_all(b"some content")?;

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        match result {
            Err(RebarError::Object(ObjectError::MalformedHeader { reason })) => {
                assert_eq!(reason, "Missing size");
            }
            _ => panic!("Expected MalformedHeader error, got: {:?}", result),
        }
        Ok(())
    }

    #[test]
    fn test_cat_file_size_too_large() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "e".repeat(64);

        // Create a file with size larger than FILE_SIZE_LIMIT
        let objects_dir = temp_dir.path().join(".rebar").join("objects");
        let object_path = objects_dir.join(&hash);
        let mut file = fs::File::create(object_path)?;
        let header = format!("blob {}\n", FILE_SIZE_LIMIT + 1);
        file.write_all(header.as_bytes())?;
        file.write_all(b"some content")?;

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        match result {
            Err(RebarError::Object(ObjectError::InvalidLength { expected, actual })) => {
                assert_eq!(expected, FILE_SIZE_LIMIT);
                assert_eq!(actual, Some(FILE_SIZE_LIMIT + 1));
            }
            _ => panic!("Expected InvalidLength error for size too large"),
        }
        Ok(())
    }

    #[test]
    fn test_cat_file_content_length_mismatch_too_short() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "f".repeat(64);

        // Create a file where actual content is shorter than header claims
        let objects_dir = temp_dir.path().join(".rebar").join("objects");
        let object_path = objects_dir.join(&hash);
        let mut file = fs::File::create(object_path)?;
        file.write_all(b"blob 100\n")?;
        file.write_all(b"short")?; // Only 5 bytes, but header says 100

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        match result {
            Err(RebarError::Object(ObjectError::InvalidLength { expected, actual })) => {
                assert_eq!(expected, 100);
                assert_eq!(actual, Some(5));
            }
            _ => panic!("Expected InvalidLength error for content too short"),
        }
        Ok(())
    }

    #[test]
    fn test_cat_file_content_length_mismatch_too_long() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "g".repeat(64);

        // Create a file where actual content is longer than header claims
        let objects_dir = temp_dir.path().join(".rebar").join("objects");
        let object_path = objects_dir.join(&hash);
        let mut file = fs::File::create(object_path)?;
        file.write_all(b"blob 5\n")?;
        file.write_all(b"this is much longer than 5 bytes")?;

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        match result {
            Err(RebarError::Object(ObjectError::InvalidLength { expected, actual })) => {
                assert_eq!(expected, 5);
                assert_eq!(actual, None); // Indicates content is longer
            }
            _ => panic!("Expected InvalidLength error for content too long"),
        }
        Ok(())
    }

    #[test]
    fn test_cat_file_corrupted_compression() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "h".repeat(64);

        // Create a file with invalid compressed content
        let objects_dir = temp_dir.path().join(".rebar").join("objects");
        let object_path = objects_dir.join(&hash);
        let mut file = fs::File::create(object_path)?;
        file.write_all(b"blob 10\n")?;
        file.write_all(b"corrupted!")?; // Not valid zstd data

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        match result {
            Err(RebarError::Object(ObjectError::CorruptedContent { reason })) => {
                assert!(reason.contains("Decompression failed"));
            }
            _ => panic!("Expected CorruptedContent error"),
        }
        Ok(())
    }

    #[test]
    fn test_cat_file_empty_content() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "i".repeat(64);

        // Create a blob with empty content
        create_blob_object(&temp_dir, &hash, "")?;

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_cat_file_large_valid_content() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "j".repeat(64);

        // Create content that's large but within limits
        let content = "A".repeat(1000);
        create_blob_object(&temp_dir, &hash, &content)?;

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_cat_file_unicode_content() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repository()?;
        let hash = "k".repeat(64);

        // Create content with unicode characters
        let content = "Hello, 世界! 🦀 Rust is awesome!";
        create_blob_object(&temp_dir, &hash, content)?;

        let result = cat_file_from_path(&hash, temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_parse_header_zero_size() {
        let header = "blob 0\n";
        let result = parse_header(header).unwrap();
        assert_eq!(result.0, ObjectType::Blob);
        assert_eq!(result.1, 0);
    }

    #[test]
    fn test_parse_header_max_size() {
        let header = format!("blob {}\n", usize::MAX);
        let result = parse_header(&header).unwrap();
        assert_eq!(result.0, ObjectType::Blob);
        assert_eq!(result.1, usize::MAX);
    }
}
