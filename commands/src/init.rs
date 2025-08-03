/// Initialize a new .rebar repository
use std::fs;

pub fn init() {
    // Create the main .rebar directory
    fs::create_dir(".rebar").unwrap();

    // Create subdirectories
    fs::create_dir(".rebar/objects").unwrap();
    fs::create_dir(".rebar/pointers").unwrap();

    // Write to .rebar/HEAD
    fs::write(".rebar/HEAD", "ref: refs/heads/main\n").unwrap();
}
