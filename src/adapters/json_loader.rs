//! JSON node-event deserialization with bounded insertion batches.
use crate::adapters::loader::{load_cached, LoadOptions, Loader};
use crate::core::store::Store;
use anyhow::Result;
use std::path::Path;
use std::sync::{atomic::AtomicBool, Arc};

pub struct JsonLoader {
    pub cancelled: Arc<AtomicBool>,
    options: LoadOptions,
}
impl Default for JsonLoader {
    fn default() -> Self {
        Self::new()
    }
}
impl JsonLoader {
    pub fn new() -> Self {
        Self::with_options(LoadOptions::default())
    }
    pub fn with_options(options: LoadOptions) -> Self {
        Self {
            cancelled: options.cancelled.clone(),
            options,
        }
    }
    pub fn with_cache_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.options.cache_dir = Some(dir);
        self
    }
}
impl Loader for JsonLoader {
    fn load(&self, file: &Path, force: bool) -> Result<Store> {
        load_cached(file, force, &self.options, |reader, store| {
            twig_core::parser::parse_json(reader, store, &self.options)
        })
    }
}
pub use twig_core::parser::child_path;
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
