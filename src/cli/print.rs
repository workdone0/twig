//! `twig --print` / `-p` — pretty-print a JSON or YAML file and exit.
//!
//! For JSON: `serde_json::to_string_pretty` plus a small ANSI syntax
//! highlighter when stdout is a TTY.
//! For YAML: `serde_yml::to_string` preserves key order.
//!
//! In both cases a metadata panel (file, size, type, item count) is
//! printed to stderr so the JSON on stdout remains valid for piping.

use std::io::{IsTerminal, Read, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use owo_colors::{OwoColorize, Stream::Stdout};

use crate::cli::Cli;

pub fn run(cli: &Cli) -> Result<()> {
    let file = &cli.file;
    let is_yaml = is_yaml(file);

    let mut content = String::new();
    let mut fh =
        std::fs::File::open(file).with_context(|| format!("opening {}", file.display()))?;
    fh.read_to_string(&mut content)
        .with_context(|| format!("reading {}", file.display()))?;

    if is_yaml {
        let parsed: serde_yml::Value =
            serde_yml::from_str(&content).map_err(|e| anyhow!("failed to parse YAML: {e}"))?;
        let serialized =
            serde_yml::to_string(&parsed).map_err(|e| anyhow!("failed to serialize YAML: {e}"))?;
        let meta = build_yaml_metadata(file, &content, &parsed);
        write_metadata(&meta)?;
        if let Some(out) = &cli.output {
            std::fs::File::create(out)
                .with_context(|| format!("creating {}", out.display()))?
                .write_all(serialized.as_bytes())
                .with_context(|| format!("writing {}", out.display()))?;
            let ok = "ok".if_supports_color(Stdout, |t| t.green());
            eprintln!("{ok} YAML written to {}", out.display());
        } else {
            print!("{serialized}");
        }
        return Ok(());
    }

    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow!("failed to parse JSON: {e}\n(use --fix to attempt repair)"))?;
    let serialized = serde_json::to_string_pretty(&parsed)?;
    let indented = indent(&serialized, cli.indent);
    let meta = build_json_metadata(file, &content, &parsed);
    write_metadata(&meta)?;

    if let Some(out) = &cli.output {
        std::fs::File::create(out)
            .with_context(|| format!("creating {}", out.display()))?
            .write_all(serialized.as_bytes())
            .with_context(|| format!("writing {}", out.display()))?;
        let label = if cli.fix { "Fixed" } else { "Formatted" };
        let ok = "OK".if_supports_color(Stdout, |t| t.green());
        let colored_label = label.if_supports_color(Stdout, |t| t.green());
        eprintln!("{colored_label} {ok} JSON written to {}", out.display());
    } else if std::io::stdout().is_terminal() {
        print_colored_json(&indented);
    } else {
        println!("{indented}");
    }
    Ok(())
}

fn is_yaml(p: &Path) -> bool {
    matches!(p.extension().and_then(|s| s.to_str()), Some("yaml" | "yml"))
}

fn indent(text: &str, n: usize) -> String {
    if n == 2 {
        return text.to_string();
    }
    text.lines()
        .map(|l| {
            let leading = l.chars().take_while(|c| *c == ' ').count();
            if leading == 0 {
                l.to_string()
            } else {
                let stripped = &l[leading..];
                format!("{}{stripped}", " ".repeat((leading / 2) * n))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

struct Metadata {
    file: String,
    size: usize,
    type_label: String,
    count: usize,
}

fn build_json_metadata(file: &Path, content: &str, parsed: &serde_json::Value) -> Metadata {
    let (label, count) = match parsed {
        serde_json::Value::Object(map) => ("Object", map.len()),
        serde_json::Value::Array(arr) => ("Array", arr.len()),
        _ => ("Primitive", 1),
    };
    Metadata {
        file: file.display().to_string(),
        size: content.len(),
        type_label: label.to_string(),
        count,
    }
}

fn build_yaml_metadata(file: &Path, content: &str, parsed: &serde_yml::Value) -> Metadata {
    let (label, count) = match parsed {
        serde_yml::Value::Mapping(m) => ("Object", m.len()),
        serde_yml::Value::Sequence(s) => ("Array", s.len()),
        _ => ("Primitive", 1),
    };
    Metadata {
        file: file.display().to_string(),
        size: content.len(),
        type_label: label.to_string(),
        count,
    }
}

fn write_metadata(m: &Metadata) -> Result<()> {
    // Use single-style colorization only; chained styles like
    // `.bold().cyan()` borrow from a temporary inside the closure
    // and don't survive `.to_string()` cleanly.
    let header = "── Twig ──"
        .if_supports_color(Stdout, |t| t.bright_cyan())
        .to_string();
    let file_label = "File:".if_supports_color(Stdout, |t| t.bold()).to_string();
    let size_label = "Size:".if_supports_color(Stdout, |t| t.bold()).to_string();
    let type_label = "Type:".if_supports_color(Stdout, |t| t.bold()).to_string();

    let mut out = std::io::stderr().lock();
    writeln!(out, "{header}")?;
    writeln!(out, "  {file_label} {}", m.file)?;
    writeln!(out, "  {size_label} {:.1} KB", m.size as f64 / 1024.0)?;
    writeln!(out, "  {type_label} {} ({} items)", m.type_label, m.count)?;
    writeln!(out)?;
    Ok(())
}

/// Minimal ANSI colorizer for JSON. Highlights keys, strings, numbers,
/// booleans, nulls, and structural punctuation.
fn print_colored_json(text: &str) {
    let mut out = String::with_capacity(text.len() * 2);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                let mut s = String::from(c);
                while let Some(&next) = chars.peek() {
                    s.push(chars.next().unwrap());
                    if next == '"' && !s.ends_with("\\\"") {
                        break;
                    }
                }
                // Probe forward: if next non-whitespace is `:` / `,` / `}` / `]` / newline -> key.
                let mut probe = chars.clone();
                while let Some(&p) = probe.peek() {
                    if p == ' ' || p == '\t' {
                        probe.next();
                    } else {
                        break;
                    }
                }
                let key_like = matches!(probe.peek(), Some(':' | ',' | '\n' | '}' | ']'));
                let colored = if key_like {
                    s.if_supports_color(Stdout, |t| t.blue()).to_string()
                } else {
                    s.if_supports_color(Stdout, |t| t.green()).to_string()
                };
                out.push_str(&colored);
            }
            '-' | '0'..='9' => {
                let mut s = String::from(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit()
                        || next == '.'
                        || next == 'e'
                        || next == 'E'
                        || next == '+'
                        || next == '-'
                    {
                        s.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                let colored = s.if_supports_color(Stdout, |t| t.yellow()).to_string();
                out.push_str(&colored);
            }
            't' | 'f' => {
                let mut rest = String::from(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphabetic() {
                        rest.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if rest == "true" || rest == "false" {
                    let colored = rest.if_supports_color(Stdout, |t| t.magenta()).to_string();
                    out.push_str(&colored);
                } else {
                    out.push_str(&rest);
                }
            }
            'n' => {
                let mut rest = String::from(c);
                for _ in 0..3 {
                    if let Some(&next) = chars.peek() {
                        if next.is_ascii_alphabetic() {
                            rest.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                }
                if rest == "null" {
                    let colored = rest.if_supports_color(Stdout, |t| t.dimmed()).to_string();
                    out.push_str(&colored);
                } else {
                    out.push_str(&rest);
                }
            }
            '{' | '}' | '[' | ']' | ',' | ':' => {
                let colored = c
                    .to_string()
                    .if_supports_color(Stdout, |t| t.dimmed())
                    .to_string();
                out.push_str(&colored);
            }
            other => out.push(other),
        }
    }
    println!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tempfile::tempdir;

    fn cli_for(args: &[&str]) -> Result<Cli> {
        Ok(Cli::try_parse_from(args)?)
    }

    #[test]
    fn prints_valid_json_to_stdout() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("input.json");
        std::fs::write(&path, r#"{"a":1,"b":[1,2]}"#).unwrap();
        let cli = cli_for(&["twig", path.to_str().unwrap(), "-p"]).unwrap();
        run(&cli).unwrap();
    }

    #[test]
    fn fix_writes_repaired_json() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("bad.json");
        let out = dir.path().join("good.json");
        std::fs::write(&src, "{'a': 1,}").unwrap();
        let cli = cli_for(&[
            "twig",
            src.to_str().unwrap(),
            "--fix",
            "-o",
            out.to_str().unwrap(),
        ])
        .unwrap();
        crate::cli::fix::run(&cli).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed, serde_json::json!({"a": 1}));
    }

    #[test]
    fn fix_rejects_yaml() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("k.yaml");
        std::fs::write(&src, "foo: bar").unwrap();
        let cli = cli_for(&["twig", src.to_str().unwrap(), "--fix"]).unwrap();
        assert!(crate::cli::fix::run(&cli).is_err());
    }
}
