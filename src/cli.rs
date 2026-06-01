/// CLI argument parsing with clap derive.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "steady",
    about = "Motor accessibility daemon that filters pointer input to reduce tremor and smooth movement.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the filtering daemon in the foreground.
    Start {
        /// Path to a custom config file (TOML).
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Print the default configuration to stdout.
    /// Pipe to a file to create your own config.
    Defaults,
    /// Show where the config file is expected and whether it exists.
    Status,
    /// Validate a config file without starting the daemon.
    Validate {
        /// Path to the config file to validate. Uses the default path if omitted.
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}
