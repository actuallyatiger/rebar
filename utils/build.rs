//! Build script for the Rebar VCS

use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    hash_size: u8,
    file_size_limit: usize,
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
    println!("cargo:rustc-env=HASH_SIZE={}", config.hash_size);
    println!("cargo:rustc-env=FILE_SIZE_LIMIT={}", config.file_size_limit);
}
