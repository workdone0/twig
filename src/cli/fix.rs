//! Repair JSON and format the resulting value using the common output path.
use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;
use std::path::Path;

pub fn run(cli: &Cli) -> Result<()> {
    super::print::run(cli)
}

pub fn run_from_path(path: &Path) -> Result<()> {
    let cli = Cli::try_parse_from([
        std::ffi::OsStr::new("twig"),
        path.as_os_str(),
        std::ffi::OsStr::new("--fix"),
        std::ffi::OsStr::new("-o"),
        path.as_os_str(),
    ])?;
    run(&cli)
}
