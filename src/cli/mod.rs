//! CLI argument parsing and non-interactive subcommands.
//!
//! `Cli` is the `clap` parser; `fix::run` and `print::run` implement
//! the two non-TUI modes (`--fix` and `--print` / `-p`).

pub mod fix;
pub mod print;

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "twig",
    version,
    about = "Inspect. Navigate. Understand. A modern, terminal-based data explorer.",
    long_about = None,
)]
pub struct Cli {
    /// The JSON / YAML / HAR file to explore.
    pub file: PathBuf,

    /// Attempt to repair malformed JSON and exit.
    #[arg(long)]
    pub fix: bool,

    /// Pretty-print the file (after --fix if applicable) and exit.
    #[arg(short = 'p', long)]
    pub print: bool,

    /// Output file. For --fix, saves the repaired JSON. For --print,
    /// saves the formatted JSON. If omitted, prints to stdout.
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Number of spaces for indentation (default: 2).
    #[arg(long, default_value_t = 2)]
    pub indent: usize,

    /// Force rebuild of the internal SQLite database cache.
    #[arg(long)]
    pub rebuild_db: bool,
}