//! Loader trait and shared caching helpers.
//!
//! Mirrors `adapters/base_loader.py` from the Python version. A loader
//! takes a path on disk, decides whether the cached SQLite database is
//! still valid, and (if not) re-parses the file into the store.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::core::paths::{cache_dir, db_filename_for};
use crate::core::store::Store;

/// A format-specific loader (JSON, YAML, …).
pub trait Loader: Send + Sync {
    /// Parse `file` and return a populated, on-disk [`Store`]. If a
    /// valid cache exists and `force_rebuild` is false, the cached
    /// store is opened without re-parsing.
    fn load(&self, file: &Path, force_rebuild: bool) -> Result<Store>;
}

/// Compute the per-file cache path. Creates the cache directory on demand.
pub fn cache_path_for(file: &Path) -> Result<PathBuf> {
    let dir = cache_dir().context("creating twig cache directory")?;
    Ok(dir.join(db_filename_for(file)))
}

/// Open the on-disk cache at `db_path`, returning `Ok(None)` if the
/// cache is missing, empty, or unreadable. Used by loaders to decide
/// whether ingestion is required.
pub fn open_existing(db_path: &Path) -> Result<Option<Store>> {
    if !db_path.exists() {
        return Ok(None);
    }
    let store = Store::open(db_path).context("opening cached database")?;
    if store.node_count().unwrap_or(0) == 0 {
        return Ok(None);
    }
    Ok(Some(store))
}

/// Drop the FTS triggers, indexes, and search table so bulk inserts go
/// fast. Mirrors `_drop_indexes` in `sqlite_loader.py`.
pub fn drop_indexes(store: &Store) -> Result<()> {
    store.db_conn().execute_batch(
        "DROP INDEX IF EXISTS idx_parent_rank;
         DROP INDEX IF EXISTS idx_path;
         DROP TRIGGER IF EXISTS nodes_ai;
         DROP TRIGGER IF EXISTS nodes_ad;
         DROP TRIGGER IF EXISTS nodes_au;
         DROP TABLE IF EXISTS nodes_search;",
    )?;
    Ok(())
}

/// Recreate indexes and repopulate the FTS5 mirror after bulk load.
/// Mirrors `_rebuild_indexes` in `sqlite_loader.py`.
pub fn rebuild_indexes(store: &Store) -> Result<()> {
    store.db_conn().execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_parent_rank ON nodes(parent_id, rank);
         CREATE INDEX IF NOT EXISTS idx_path ON nodes(path);
         CREATE VIRTUAL TABLE IF NOT EXISTS nodes_search USING fts5(
             key, value, path, content='nodes', content_rowid='rowid'
         );
         INSERT INTO nodes_search(rowid, key, value, path)
             SELECT rowid, key, value, path FROM nodes;
         CREATE TRIGGER IF NOT EXISTS nodes_ai AFTER INSERT ON nodes BEGIN
             INSERT INTO nodes_search(rowid, key, value, path)
                 VALUES (new.rowid, new.key, new.value, new.path);
         END;
         CREATE TRIGGER IF NOT EXISTS nodes_ad AFTER DELETE ON nodes BEGIN
             INSERT INTO nodes_search(nodes_search, rowid, key, value, path)
                 VALUES('delete', old.rowid, old.key, old.value, old.path);
         END;
         CREATE TRIGGER IF NOT EXISTS nodes_au AFTER UPDATE ON nodes BEGIN
             INSERT INTO nodes_search(nodes_search, rowid, key, value, path)
                 VALUES('delete', old.rowid, old.key, old.value, old.path);
             INSERT INTO nodes_search(rowid, key, value, path)
                 VALUES (new.rowid, new.key, new.value, new.path);
         END;",
    )?;
    Ok(())
}
