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
    /// Compute object ID and optionally create a blob from a file
    HashObject {
        /// Path to the file to hash (if not using --stdin)
        path: Option<String>,
        /// Read input from stdin instead of a file
        #[arg(long)]
        stdin: bool,
        /// Actually write the object into the object database
        #[arg(short, long)]
        write: bool,
    },
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
        Command::HashObject { path, stdin, write } => {
            if stdin && path.is_some() {
                eprintln!("Error: Cannot specify both --stdin and a file path");
                std::process::exit(1);
            }
            if !stdin && path.is_none() {
                eprintln!("Error: Must specify either --stdin or a file path");
                std::process::exit(1);
            }
            
            if let Some(ref path) = path {
                if let Err(e) = utils::validate_path(path) {
                    handle_error(RebarError::from(e));
                }
            }
            
            commands::hash_object(path.as_deref(), stdin, write)
        }
    };

    if let Err(e) = result {
        handle_error(e);
    }
}
