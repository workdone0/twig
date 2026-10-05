//! Argument parsing and shared output utilities.
pub mod check;
pub mod fix;
pub mod print;
use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(group(clap::ArgGroup::new("format_mode").args(["fix", "print"]).multiple(true)))]
#[command(
    name = "twig",
    version,
    disable_version_flag = true,
    about = "Inspect. Navigate. Understand. A terminal explorer for JSON and YAML."
)]
pub struct Cli {
    /// JSON, YAML, or HAR file. Use '-' for stdin in non-interactive modes.
    #[arg(
        required_unless_present = "clear_cache",
        conflicts_with = "clear_cache"
    )]
    pub file: Option<PathBuf>,
    /// Repair JSON before formatting. May be combined with --print.
    #[arg(long, conflicts_with = "check")]
    pub fix: bool,
    /// Format JSON or YAML documents and exit.
    #[arg(short = 'p', long, conflicts_with = "check")]
    pub print: bool,
    /// Atomically write formatted output to this file.
    #[arg(short = 'o', long, requires = "format_mode")]
    pub output: Option<PathBuf>,
    /// JSON indentation width, from 0 to 16. YAML uses its standard formatter.
    #[arg(short='i', long, default_value_t=2, value_parser=clap::value_parser!(u8).range(0..=16))]
    pub indent: u8,
    /// Rebuild the cache even when source content matches.
    #[arg(long, conflicts_with_all=["fix", "print", "clear_cache"])]
    pub rebuild_db: bool,
    /// Validate input and report ingestion timing without opening the TUI.
    #[arg(long)]
    pub check: bool,
    /// Use a temporary database removed when Twig closes.
    #[arg(long, conflicts_with = "clear_cache")]
    pub no_cache: bool,
    /// Remove stored SQLite caches and exit (close other Twig processes first).
    #[arg(long, conflicts_with_all=["fix", "print", "check", "output"])]
    pub clear_cache: bool,
    /// Print version.
    #[arg(short='V', long, visible_short_alias='v', action=clap::ArgAction::Version)]
    pub version: Option<bool>,
}

pub fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("yaml") || e.eq_ignore_ascii_case("yml"))
}

pub fn write_output(path: Option<&Path>, text: &str) -> Result<()> {
    if let Some(path) = path {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temp =
            tempfile::NamedTempFile::new_in(parent).context("creating output temporary file")?;
        if let Ok(meta) = std::fs::metadata(path) {
            temp.as_file().set_permissions(meta.permissions())?;
        }
        temp.write_all(text.as_bytes())?;
        temp.write_all(b"\n")?;
        temp.as_file().sync_all()?;
        temp.persist(path)
            .with_context(|| format!("writing {}", path.display()))?;
    } else {
        let mut out = std::io::stdout().lock();
        writeln!(out, "{text}")?;
    }
    Ok(())
}
