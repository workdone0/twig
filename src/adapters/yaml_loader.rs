//! Streaming YAML ingestion into the SQLite store.
//!
//! Drives `serde_yml::Deserializer::from_reader`, which yields one
//! YAML document at a time. Each document is converted to a
//! `serde_json::Value` so we can reuse the same `Emitter` machinery the
//! JSON loader uses, then fed through the same defer-indexing dance.
//!
//! Single-document YAML files are wrapped in a virtual array root so
//! `.kind` resolves to `.[0].kind` (the Python loader did the same).

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::adapters::json_loader::Emitter;
use crate::adapters::loader::{
    cache_path_for, drop_indexes, open_existing, rebuild_indexes, Loader,
};
use crate::core::model::Node;
use crate::core::store::Store;

pub struct YamlLoader {
    /// Override the default cache directory (used by tests).
    cache_dir_override: Option<std::path::PathBuf>,
}

impl Default for YamlLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl YamlLoader {
    pub fn new() -> Self {
        Self {
            cache_dir_override: None,
        }
    }

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

impl Loader for YamlLoader {
    fn load(&self, file: &Path, force_rebuild: bool) -> Result<Store> {
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

        let mut store =
            Store::open(&db_path).context("opening fresh sqlite database for yaml load")?;
        store.db_conn_mut().execute_batch(
            "PRAGMA synchronous = OFF;
             PRAGMA journal_mode = MEMORY;",
        )?;
        drop_indexes(&store)?;

        let fh = File::open(file).with_context(|| format!("opening {}", file.display()))?;
        let reader = BufReader::new(fh);
        let iter = serde_yml::Deserializer::from_reader(reader);

        const BATCH: usize = 10_000;
        let mut batch: Vec<Node> = Vec::with_capacity(BATCH);
        let mut emitter = Emitter::new();

        // The Python loader always wraps the YAML stream in a virtual
        // array root so single-document YAML looks like .[0].… under
        // the hood. We do the same: emit a root Array and append each
        // top-level document to it.
        let virtual_root_id = uuid::Uuid::new_v4();
        batch.push(Node {
            id: virtual_root_id,
            key: "root".to_string(),
            value: None,
            ty: crate::core::model::DataType::Array,
            parent: None,
            path: ".".to_string(),
            is_expanded: false,
            rank: 0,
        });

        for (rank, doc) in (0_i64..).zip(iter) {
            let value: Value = match serde::Deserialize::deserialize(doc) {
                Ok(v) => v,
                Err(e) => {
                    let (line, col) = e
                        .location()
                        .map(|l| (l.line(), l.column()))
                        .unwrap_or((0, 0));
                    return Err(anyhow::anyhow!(
                        "YAML parse error at line {line}, column {col}: {e}"
                    ));
                }
            };
            let base = format!(".[{rank}]");
            emitter.reset();
            emitter.emit_value(
                Some(virtual_root_id),
                &rank.to_string(),
                &base,
                &value,
                &mut batch,
            );
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
        store.db_conn_mut().execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA journal_mode = WAL;",
        )?;

        let store = Store::open(&db_path)?;
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/k8s_manifest.yaml")
    }

    #[test]
    fn loads_single_document_yaml() {
        let cache_dir = tempfile::tempdir().unwrap();
        let loader = YamlLoader::new().with_cache_dir(cache_dir.path().to_path_buf());
        let store = loader.load(&sample(), true).expect("load k8s manifest");
        assert!(store.node_count().unwrap() > 10);

        // The YAML loader wraps single-document YAML in a virtual
        // array root, so .[0].kind should resolve to "Deployment".
        let node = store
            .resolve_path(".[0].kind")
            .unwrap()
            .expect("kind should be reachable");
        assert_eq!(
            node.value.as_ref().unwrap(),
            &Value::String("Deployment".into())
        );
    }

    #[test]
    fn virtual_root_falls_back_for_single_doc_lookup() {
        let cache_dir = tempfile::tempdir().unwrap();
        let loader = YamlLoader::new().with_cache_dir(cache_dir.path().to_path_buf());
        let store = loader.load(&sample(), true).unwrap();

        // `.kind` (no `[0]`) should fall back to `.[0].kind`.
        let node = store
            .resolve_path(".kind")
            .unwrap()
            .expect("single-doc fallback");
        assert_eq!(
            node.value.as_ref().unwrap(),
            &Value::String("Deployment".into())
        );
    }

    #[test]
    fn yaml_loader_reuses_cache() {
        let cache_dir = tempfile::tempdir().unwrap();
        let tmp_yaml = cache_dir.path().join("payload.yaml");
        std::fs::write(&tmp_yaml, "foo: bar\nbaz:\n  - 1\n  - 2\n").unwrap();

        let loader = YamlLoader::new().with_cache_dir(cache_dir.path().to_path_buf());
        let first = loader.load(&tmp_yaml, false).unwrap();
        let count = first.node_count().unwrap();
        let second = loader.load(&tmp_yaml, false).unwrap();
        assert_eq!(second.node_count().unwrap(), count);
        assert!(count >= 6); // root + foo + baz + 2 items
    }
}
