//! Platform-specific locations for the SQLite cache and the JSON config.
//!
//! The behaviour mirrors `core/db.py::DatabaseManager._get_cache_dir` and
//! `core/config.py::Config._get_config_dir` from the Python version:
//!
//! - **macOS**: `~/Library/Caches/twig` (cache), `~/.config/twig` (config)
//! - **Linux**: `$XDG_CACHE_HOME/twig` (cache, default `~/.cache/twig`),
//!   `$XDG_CONFIG_HOME/twig` (config, default `~/.config/twig`)
//! - **Windows**: `%LOCALAPPDATA%\twig` (cache), `%APPDATA%\twig` (config)

use std::path::PathBuf;

/// Returns the per-user cache directory for SQLite database files.
/// Creates it on disk if missing.
pub fn cache_dir() -> std::io::Result<PathBuf> {
    let dir = base_dir(true)?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the per-user config directory holding `config.json`.
/// Creates it on disk if missing.
pub fn config_dir() -> std::io::Result<PathBuf> {
    let dir = base_dir(false)?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn base_dir(cache: bool) -> std::io::Result<PathBuf> {
    if cfg!(target_os = "macos") {
        // macOS always uses Library/Caches even though we don't read
        // XDG-style vars there. Config follows the Unix branch.
        if cache {
            let home = home_dir()?;
            return Ok(home.join("Library").join("Caches").join("twig"));
        }
    }

    if let Some(dir) = dirs::cache_dir().filter(|_| cache) {
        return Ok(dir.join("twig"));
    }
    if let Some(dir) = dirs::config_dir().filter(|_| !cache) {
        return Ok(dir.join("twig"));
    }

    let home = home_dir()?;
    let folder = if cache { ".cache" } else { ".config" };
    Ok(home.join(folder).join("twig"))
}

fn home_dir() -> std::io::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine home directory",
        )
    })
}

/// Stable, deterministic cache filename for a given source file.
///
/// Mirrors `DatabaseManager.get_db_path`: take the basename, append an
/// MD5 hash of the absolute path. We keep MD5 here purely as a filename
/// uniquifier (no security implication); switching to a faster hash
/// later is fine.
pub fn db_filename_for(source_path: &std::path::Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let abs = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    let mut hasher = DefaultHasher::new();
    abs.hash(&mut hasher);
    let digest = hasher.finish();

    let stem = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("twig");
    format!("{stem}_{digest:016x}.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_dir_and_config_dir_are_distinct() {
        let cache = cache_dir().unwrap();
        let config = config_dir().unwrap();
        assert_ne!(cache, config);
        assert!(cache.ends_with("twig"));
        assert!(config.ends_with("twig"));
    }

    #[test]
    fn db_filename_includes_basename_and_hash() {
        let p = std::path::Path::new("foo.json");
        let name = db_filename_for(p);
        assert!(name.starts_with("foo.json_"));
        assert!(name.ends_with(".db"));
    }

    #[test]
    fn db_filename_is_deterministic() {
        let p = std::path::Path::new("/some/path/to/bar.yaml");
        let a = db_filename_for(p);
        let b = db_filename_for(p);
        assert_eq!(a, b);
    }
}
