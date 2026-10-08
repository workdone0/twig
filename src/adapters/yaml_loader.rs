//! YAML document stream to JSON-compatible SQLite nodes.
//! The YAML library may buffer document/parser state; unlike JSON, bounded
//! parser memory is not guaranteed. Emitted node batches are bounded.
use crate::adapters::loader::{load_cached, LoadOptions, Loader};
use crate::core::store::Store;
use anyhow::Result;
use std::path::Path;

pub struct YamlLoader {
    options: LoadOptions,
}
impl Default for YamlLoader {
    fn default() -> Self {
        Self::new()
    }
}
impl YamlLoader {
    pub fn new() -> Self {
        Self::with_options(LoadOptions::default())
    }
    pub fn with_options(options: LoadOptions) -> Self {
        Self { options }
    }
    pub fn with_cache_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.options.cache_dir = Some(dir);
        self
    }
}
impl Loader for YamlLoader {
    fn load(&self, file: &Path, force: bool) -> Result<Store> {
        load_cached(file, force, &self.options, |reader, store| {
            twig_core::parser::parse_yaml(reader, store, &self.options)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
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
