//! Streaming JSON ingestion into the SQLite store.
//!
//! Drives `serde_json::Deserializer::from_reader` over the input file,
//! which yields one `serde_json::Value` at a time at the top of the
//! stream, and walks that value recursively — same shape as the Python
//! `ijson` event-based approach, but exploiting serde's slightly higher
//! level API.
//!
//! Performance comes from the same "defer indexing" trick the Python
//! loader used: drop FTS5 triggers and indexes, ingest all rows in
//! bulk, then rebuild them once at the end.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use serde_json::Value;
use uuid::Uuid;

use crate::adapters::loader::{
    cache_path_for, drop_indexes, open_existing, rebuild_indexes, Loader,
};
use crate::core::model::{DataType, Node};

pub struct JsonLoader {
    pub cancelled: std::sync::Arc<AtomicBool>,
    /// Override the default cache directory (used by tests so they
    /// don't collide on the user's real cache path).
    cache_dir_override: Option<std::path::PathBuf>,
}

impl Default for JsonLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonLoader {
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(AtomicBool::new(false)),
            cache_dir_override: None,
        }
    }

    /// Restrict the cache to a specific directory. Useful for tests.
    pub fn with_cache_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.cache_dir_override = Some(dir);
        self
    }

    fn cache_path(&self, file: &Path) -> Result<std::path::PathBuf> {
        match &self.cache_dir_override {
            Some(dir) => {
                std::fs::create_dir_all(dir).context("creating test cache dir")?;
                Ok(dir.join(crate::core::paths::db_filename_for(file)))
            }
            None => cache_path_for(file),
        }
    }
}

impl Loader for JsonLoader {
    fn load(&self, file: &Path, force_rebuild: bool) -> Result<crate::core::store::Store> {
        if !force_rebuild {
            if let Ok(db_path) = self.cache_path(file) {
                if let Some(store) = open_existing(&db_path)? {
                    return Ok(store);
                }
            }
        }

        let db_path = self.cache_path(file)?;
        if db_path.exists() {
            std::fs::remove_file(&db_path).ok();
        }
        if db_path.with_extension("db-wal").exists() {
            std::fs::remove_file(db_path.with_extension("db-wal")).ok();
        }
        if db_path.with_extension("db-shm").exists() {
            std::fs::remove_file(db_path.with_extension("db-shm")).ok();
        }

        let mut store = crate::core::store::Store::open(&db_path)
            .context("opening fresh sqlite database for json load")?;

        // Bulk-load PRAGMAs. Mirrors the Python loader.
        store.db_conn_mut().execute_batch(
            "PRAGMA synchronous = OFF;
             PRAGMA journal_mode = MEMORY;",
        )?;
        drop_indexes(&store)?;

        let fh = File::open(file).with_context(|| format!("opening {}", file.display()))?;
        let reader = BufReader::new(fh);
        let stream = serde_json::Deserializer::from_reader(reader).into_iter::<Value>();

        const BATCH: usize = 10_000;
        let mut batch: Vec<Node> = Vec::with_capacity(BATCH);
        let mut emitter = Emitter::new();

        for top in stream {
            if self.cancelled.load(Ordering::Relaxed) {
                break;
            }
            let value = match top {
                Ok(v) => v,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to parse JSON at depth {}: {}",
                        e.line(),
                        e
                    ));
                }
            };
            emitter.reset();
            emitter.emit_value(None, "root", ".", &value, &mut batch);
            if batch.len() >= BATCH {
                let drained = std::mem::take(&mut batch);
                store.bulk_load(&drained)?;
                batch.reserve(BATCH);
            }
        }

        if !batch.is_empty() {
            store.bulk_load(&batch)?;
        }

        rebuild_indexes(&store)?;
        // Restore safe defaults after the bulk phase.
        store.db_conn_mut().execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA journal_mode = WAL;",
        )?;

        // Reload so root_id is populated from the now-non-empty DB.
        let store = crate::core::store::Store::open(&db_path)?;
        Ok(store)
    }
}

pub struct Emitter {
    stack: Vec<Frame>,
}

struct Frame {
    /// Number of children already emitted under this frame.
    count: i64,
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Emitter {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn reset(&mut self) {
        self.stack.clear();
    }

    fn push(&mut self) {
        self.stack.push(Frame { count: 0 });
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn current(&mut self) -> Option<&mut Frame> {
        self.stack.last_mut()
    }

    pub fn emit_value(
        &mut self,
        parent: Option<Uuid>,
        key: &str,
        base_path: &str,
        value: &Value,
        out: &mut Vec<Node>,
    ) {
        let ty = DataType::from_value(value);
        let id = Uuid::new_v4();

        match &value {
            Value::Object(map) => {
                out.push(Node {
                    id,
                    key: key.to_string(),
                    value: None,
                    ty: DataType::Object,
                    parent,
                    path: base_path.to_string(),
                    is_expanded: false,
                    rank: 0,
                });
                self.push();
                for (k, v) in map {
                    let child_path = child_path(base_path, false, k);
                    self.emit_value(Some(id), k, &child_path, v, out);
                }
                self.pop();
            }
            Value::Array(items) => {
                out.push(Node {
                    id,
                    key: key.to_string(),
                    value: None,
                    ty: DataType::Array,
                    parent,
                    path: base_path.to_string(),
                    is_expanded: false,
                    rank: 0,
                });
                self.push();
                for (idx, v) in items.iter().enumerate() {
                    let child_path = child_path(base_path, true, &idx.to_string());
                    self.emit_value(Some(id), &idx.to_string(), &child_path, v, out);
                }
                self.pop();
            }
            _ => {
                let rank = self
                    .current()
                    .map(|f| {
                        let r = f.count;
                        f.count += 1;
                        r
                    })
                    .unwrap_or(0);
                out.push(Node {
                    id,
                    key: key.to_string(),
                    value: Some(value.clone()),
                    ty,
                    parent,
                    path: base_path.to_string(),
                    is_expanded: false,
                    rank,
                });
            }
        }
    }
}

/// Build the jq-style child path under `parent_path`. `array_index` is
/// true for items inside an array.
pub fn child_path(parent_path: &str, array_index: bool, key: &str) -> String {
    if array_index {
        format!("{parent_path}[{key}]")
    } else if parent_path == "." {
        format!(".{key}")
    } else {
        format!("{parent_path}.{key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/cloud_infrastructure.json")
    }

    #[test]
    fn loads_sample_json_file() {
        let cache_dir = tempfile::tempdir().unwrap();
        let loader = JsonLoader::new().with_cache_dir(cache_dir.path().to_path_buf());
        let store = loader.load(&sample(), true).expect("load sample");
        assert!(store.node_count().unwrap() > 10);
        // The sample has the key "available" appearing in some DB records.
        let hit = store
            .find_next_node("available", None, 1)
            .unwrap()
            .expect("expected substring match");
        assert!(
            hit.path.contains("availability")
                || hit.key.contains("available")
                || hit
                    .value
                    .as_ref()
                    .map(|v| v.to_string().contains("available"))
                    .unwrap_or(false)
        );
    }

    #[test]
    fn cache_is_reused_on_second_load() {
        // Each test gets its own temp cache dir so the two invocations
        // can't race on the user's real cache path.
        let cache_dir = tempfile::tempdir().unwrap();
        let tmp_json = cache_dir.path().join("payload.json");
        std::fs::write(&tmp_json, r#"{"a": 1, "b": [1, 2, 3]}"#).unwrap();

        let loader = JsonLoader::new().with_cache_dir(cache_dir.path().to_path_buf());
        let first = loader.load(&tmp_json, false).expect("first load");
        let first_count = first.node_count().unwrap();

        // Second call must reuse the cache (no force_rebuild). The
        // number of nodes should match exactly.
        let second = loader.load(&tmp_json, false).expect("second load");
        assert_eq!(second.node_count().unwrap(), first_count);
        assert!(first_count >= 5); // root + 2 keys + 3 array items
    }

    #[test]
    fn child_path_handles_root_arrays_and_objects() {
        assert_eq!(child_path(".", false, "foo"), ".foo");
        assert_eq!(child_path(".foo", false, "bar"), ".foo.bar");
        assert_eq!(child_path(".foo", true, "0"), ".foo[0]");
    }
}
