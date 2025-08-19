//! Build script for the Rebar VCS

use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    // hash_size: u8,
    #[serde(default)]
    file_size_limit: FileSizeLimit,

    #[serde(default)]
    compression_level: CompressionLevel,
}

#[derive(Deserialize, Debug)]
struct FileSizeLimit(usize);
impl Default for FileSizeLimit {
    fn default() -> Self {
        FileSizeLimit(10485760) // 10 MB
    }
}

#[derive(Deserialize, Debug)]
struct CompressionLevel(u8);
impl Default for CompressionLevel {
    fn default() -> Self {
        CompressionLevel(3) // Default compression level
    }
}

fn main() {
    // Re-run this script if `config.json` changes.
    println!("cargo:rerun-if-changed=config.json");

    // Read the config file.
    let config_str = fs::read_to_string("config.json")
        .expect("Failed to read config.json. Does the file exist?");

    // Parse the JSON string into our `Config` struct.
    let config: Config =
        serde_json::from_str(&config_str).expect("Failed to parse config.json. Is it valid JSON?");

    // Pass the values to the Rust compiler.
    // println!("cargo:rustc-env=HASH_SIZE={}", config.hash_size);
    println!(
        "cargo:rustc-env=FILE_SIZE_LIMIT={}",
        config.file_size_limit.0
    );

    // constrain compression level from 0..=22
    if config.compression_level.0 > 22 {
        panic!("Compression level must be between 1 and 22, or 0 to use default.");
    }

    println!(
        "cargo:rustc-env=COMPRESSION_LEVEL={}",
        config.compression_level.0
    );
}
