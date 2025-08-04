//! Rebar - A new version control system written in Rust

mod utils;

use clap::{Parser, Subcommand};

/// A new version control system written in Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize a new .rebar repository
    Init,
    /// Print the contents of a .rebar object
    CatFile { hash: String },
}

fn main() {
    let args = Args::parse();

    // TODO: Implement other commands
    match args.command {
        // Create the main .rebar directory
        Command::Init => commands::init(),
        Command::CatFile { hash } => {
            utils::validate_hex(&hash).unwrap();
            commands::cat_file(&hash).unwrap()
        }
    }
}
