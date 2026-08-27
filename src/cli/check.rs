//! `twig --check` — exercise the streaming ingestion pipeline
//! without opening the TUI.
//!
//! Loads the file through the same `Loader` trait the TUI uses,
//! measures elapsed time, prints node / file-size stats, and exits
//! 0 on success or non-zero on parse / IO failure. Useful in CI
//! and for ad-hoc benchmarks where a TTY isn't available.

use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result};

use crate::adapters::json_loader::JsonLoader;
use crate::adapters::loader::Loader;
use crate::adapters::yaml_loader::YamlLoader;

pub fn run(file: &Path, force_rebuild: bool) -> Result<()> {
    let size = std::fs::metadata(file)
        .with_context(|| format!("stat {}", file.display()))?
        .len();

    let started = Instant::now();
    let loader: Box<dyn Loader> = match file
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("yaml") | Some("yml") => Box::new(YamlLoader::new()),
        _ => Box::new(JsonLoader::new()),
    };
    let store = loader
        .load(file, force_rebuild)
        .with_context(|| format!("loading {}", file.display()))?;
    let elapsed = started.elapsed();

    let nodes = store.node_count().unwrap_or(0);
    let ms = elapsed.as_secs_f64() * 1000.0;
    let mb_per_s = if elapsed.as_secs_f64() > 0.0 {
        size as f64 / 1_048_576.0 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    println!("file:        {}", file.display());
    println!(
        "size:        {:.2} MB ({size} bytes)",
        size as f64 / 1_048_576.0
    );
    println!("nodes:       {nodes}");
    println!("elapsed:     {ms:.0} ms");
    println!("throughput:  {mb_per_s:.2} MB/s");
    Ok(())
}
