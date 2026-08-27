//! `twig --fix` — repair malformed JSON.
//!
//! Reads the input file, runs it through `core::cleaner::repair_json`,
//! and either writes the repaired result to `-o` or prints it to stdout.
//! YAML is rejected because repair is JSON-only by design (matches the
//! Python version's behavior).

use std::io::{Read, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use owo_colors::{OwoColorize, Stream::Stdout};

use crate::cli::Cli;

/// Entry point invoked by `main` for `--fix`.
pub fn run(cli: &Cli) -> Result<()> {
    let file = &cli.file;
    let is_yaml = is_yaml(file);

    if is_yaml {
        return Err(anyhow!("--fix is currently only supported for JSON files."));
    }

    let mut content = String::new();
    let mut fh =
        std::fs::File::open(file).with_context(|| format!("opening {}", file.display()))?;
    fh.read_to_string(&mut content)
        .with_context(|| format!("reading {}", file.display()))?;

    let repaired = crate::core::cleaner::repair_json(&content)?;

    if let Some(out) = &cli.output {
        std::fs::File::create(out)
            .with_context(|| format!("creating {}", out.display()))?
            .write_all(repaired.as_bytes())
            .with_context(|| format!("writing {}", out.display()))?;
        let fixed = "fixed".if_supports_color(Stdout, |t| t.green());
        eprintln!("{fixed} {} -> {}", file.display(), out.display());
    } else {
        println!("{repaired}");
    }
    Ok(())
}

fn is_yaml(p: &Path) -> bool {
    matches!(p.extension().and_then(|s| s.to_str()), Some("yaml" | "yml"))
}
