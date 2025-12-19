//! Rebar - A new version control system written in Rust

mod utils;
mod commands;

use crate::utils::errors::{InputError, RebarError};

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
    HashObject {
        /// The path to the file to hash, if not reading from stdin
        path: Option<String>,
        /// Whether to read the file contents from stdin
        #[arg(long)]
        stdin: bool,
        /// Should the object be written to the current repository
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
        Command::Init => crate::commands::init().map_err(RebarError::from),
        Command::CatFile { hash } => {
            if let Err(e) = crate::utils::validate_hex(&hash) {
                handle_error(RebarError::Input(InputError::InvalidArgument {
                    argument: "hash".to_string(),
                    reason: e.to_string(),
                }));
            }
            crate::commands::cat_file(&hash)
        }
        Command::HashObject { path, stdin, write } => {
            if stdin && path.is_some() {
                handle_error(RebarError::Input(
                    InputError::ArgumentConflict {
                        message: "Cannot specify both a path and to read from stdin".to_string(),
                    },
                ))
            } else if !stdin && path.is_none() {
                handle_error(RebarError::Input(InputError::MissingArgument {
                    argument: "path (or --stdin)".to_string(),
                }))
            }

            if let Some(ref p) = path
                && let Err(e) = crate::utils::validate_path(p)
            {
                handle_error(RebarError::from(e))
            }
            crate::commands::hash_object(path.as_deref(), stdin, write)
        }
    };

    if let Err(e) = result {
        handle_error(e);
    }
}
