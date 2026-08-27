//! Persistent JSON-backed user configuration.
//!
//! Mirrors `core/config.py` from the Python version: single file
//! (`config.json`) living in the per-user config directory, holding a
//! small `theme` string for now. Unknown keys on disk are kept (merged
//! into defaults) so future additions don't need a migration step.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::paths::config_dir;
use crate::tui::theme::DEFAULT_THEME_NAME;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config(BTreeMap<String, serde_json::Value>);

impl Default for Config {
    fn default() -> Self {
        let mut map = BTreeMap::new();
        map.insert("theme".into(), serde_json::Value::from(DEFAULT_THEME_NAME));
        Self(map)
    }
}

impl Config {
    /// Load the config from disk, falling back to defaults if the file
    /// is missing or unreadable. I/O and parse errors are surfaced via
    /// stderr so the TUI can still start.
    pub fn load() -> Self {
        Self::load_from(&default_path())
    }

    /// Variant used by tests and by callers that want to keep their
    /// config under a non-default path (e.g. inside a tempdir).
    pub fn load_from(path: &Path) -> Self {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };
        match serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&raw) {
            Ok(map) => {
                let mut cfg = Self::default();
                cfg.0.extend(map);
                cfg
            }
            Err(e) => {
                eprintln!("twig: warning: failed to parse config.json: {e}");
                Self::default()
            }
        }
    }

    /// Persist the current config to its default location on disk.
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&default_path())
    }

    /// Variant for tests / embedded use.
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.0)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    /// Update a value and persist to the default location.
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) -> std::io::Result<()> {
        self.0.insert(key.into(), value);
        self.save()
    }
}

/// Returns the absolute path to the default `config.json`.
pub fn default_path() -> PathBuf {
    config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_catppuccin_theme() {
        let cfg = Config::default();
        assert_eq!(cfg.get_string("theme"), Some(DEFAULT_THEME_NAME));
    }

    #[test]
    fn set_then_save_persists_to_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.json");

        let mut cfg = Config::load_from(&path);
        cfg.0
            .insert("theme".into(), serde_json::Value::from("dracula"));
        cfg.save_to(&path).unwrap();

        let again = Config::load_from(&path);
        assert_eq!(again.get_string("theme"), Some("dracula"));

        let on_disk = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&on_disk).unwrap();
        assert_eq!(parsed.get("theme").unwrap(), "dracula");
    }

    #[test]
    fn loading_existing_file_merges_with_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(&path, r#"{ "theme": "solarized-dark", "future_key": 42 }"#).unwrap();

        let cfg = Config::load_from(&path);
        assert_eq!(cfg.get_string("theme"), Some("solarized-dark"));
        assert_eq!(cfg.get("future_key").unwrap(), &serde_json::json!(42));
    }

    #[test]
    fn corrupted_config_falls_back_to_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(&path, "this is not json").unwrap();

        let cfg = Config::load_from(&path);
        assert_eq!(cfg.get_string("theme"), Some(DEFAULT_THEME_NAME));
    }

    #[test]
    fn missing_file_returns_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("does-not-exist.json");
        let cfg = Config::load_from(&path);
        assert_eq!(cfg.get_string("theme"), Some(DEFAULT_THEME_NAME));
    }
}
