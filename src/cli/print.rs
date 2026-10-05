//! Formatting is deliberately plain text so stdout and file output agree.
use crate::cli::{is_yaml, write_output, Cli};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Read;

pub fn run(cli: &Cli) -> Result<()> {
    let file = cli.file.as_deref().context("a file is required")?;
    let mut content = String::new();
    if file == std::path::Path::new("-") {
        std::io::stdin().read_to_string(&mut content)?;
    } else {
        content =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    }
    let text = if is_yaml(file) {
        anyhow::ensure!(!cli.fix, "--fix is only supported for JSON files");
        let mut documents = Vec::new();
        for doc in serde_norway::Deserializer::from_str(&content) {
            let value = serde_norway::Value::deserialize(doc).context("failed to parse YAML")?;
            documents.push(serde_norway::to_string(&value)?);
        }
        documents.join("---\n").trim_end().to_string()
    } else {
        if cli.fix {
            content = crate::core::cleaner::repair_json(&content)?;
        }
        let value: serde_json::Value = serde_json::from_str(&content)
            .context("failed to parse JSON (use --fix to attempt repair)")?;
        let spaces = vec![b' '; cli.indent as usize];
        let formatter = serde_json::ser::PrettyFormatter::with_indent(&spaces);
        let mut bytes = Vec::new();
        value.serialize(&mut serde_json::Serializer::with_formatter(
            &mut bytes, formatter,
        ))?;
        String::from_utf8(bytes)?
    };
    write_output(cli.output.as_deref(), &text)
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
