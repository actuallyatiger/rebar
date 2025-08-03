//! Rebar - A new version control system written in Rust

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
}

fn main() {
    let args = Args::parse();

    match args.command {
        // Create the main .rebar directory
        Command::Init => commands::init(),
    }
}
