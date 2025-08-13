//! Rebar - A new version control system written in Rust

use utils::errors::RebarError;

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
    /// Create a new .rebar object
    HashFile { path: String },
}

fn handle_error(error: RebarError) {
    eprintln!("{error}");
    std::process::exit(1);
}

fn main() {
    let args = Args::parse();

    // TODO: Implement other commands
    let result = match args.command {
        Command::Init => commands::init().map_err(RebarError::from),
        Command::CatFile { hash } => {
            if let Err(e) = utils::validate_hex(&hash) {
                handle_error(RebarError::from(e));
            }
            commands::cat_file(&hash)
        }
        Command::HashFile { path } => {
            if let Err(e) = utils::validate_path(&path) {
                handle_error(RebarError::from(e));
            }
            // commands::hash_file(&path)
            todo!("Implement the hash_file method")
        }
    };

    if let Err(e) = result {
        handle_error(e);
    }
}
