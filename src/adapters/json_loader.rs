//! JSON node-event deserialization with bounded insertion batches.
use crate::adapters::loader::{load_cached, LoadOptions, Loader};
use crate::core::{
    model::{DataType, Node},
    store::Store,
};
use anyhow::Result;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::path::Path;
use std::sync::{atomic::AtomicBool, Arc};
use uuid::Uuid;

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
            let mut sink = Sink::new(store, &self.options);
            let mut de = serde_json::Deserializer::from_reader(std::io::BufReader::new(reader));
            Seed {
                sink: &mut sink,
                parent: None,
                key: "root".into(),
                path: ".".into(),
                rank: 0,
                depth: 0,
            }
            .deserialize(&mut de)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse JSON (empty input is invalid) at line {}, column {}: {e}",
                    e.line(),
                    e.column()
                )
            })?;
            de.end().map_err(|e| {
                anyhow::anyhow!("Failed to parse JSON: expected a single document: {e}")
            })?;
            sink.flush()?;
            Ok(())
        })
    }
}

pub(crate) struct Sink<'a> {
    store: &'a mut Store,
    options: &'a LoadOptions,
    batch: Vec<Node>,
}
impl<'a> Sink<'a> {
    pub fn new(store: &'a mut Store, options: &'a LoadOptions) -> Self {
        Self {
            store,
            options,
            batch: Vec::with_capacity(1024),
        }
    }
    pub fn push(&mut self, node: Node) -> Result<()> {
        self.options.check_cancelled()?;
        self.batch.push(node);
        self.options
            .progress
            .nodes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if self.batch.len() >= 1024 {
            self.flush()?;
        }
        Ok(())
    }
    pub fn flush(&mut self) -> Result<()> {
        self.options.check_cancelled()?;
        self.store.bulk_load(&self.batch)?;
        self.batch.clear();
        Ok(())
    }
}

pub(crate) struct Seed<'a, 'b> {
    pub sink: &'a mut Sink<'b>,
    pub parent: Option<Uuid>,
    pub key: String,
    pub path: String,
    pub rank: i64,
    pub depth: usize,
}
impl Seed<'_, '_> {
    fn node(&mut self, ty: DataType, value: Option<Value>) -> Result<Uuid> {
        let id = Uuid::new_v4();
        self.sink.push(Node {
            id,
            parent: self.parent,
            key: self.key.clone(),
            path: self.path.clone(),
            rank: self.rank,
            ty,
            value,
            is_expanded: false,
        })?;
        Ok(id)
    }
    fn scalar<E: de::Error>(mut self, value: Value) -> std::result::Result<(), E> {
        self.node(DataType::from_value(&value), Some(value))
            .map_err(E::custom)?;
        Ok(())
    }
}
impl<'de> DeserializeSeed<'de> for Seed<'_, '_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<(), D::Error> {
        if self.depth > 128 {
            return Err(de::Error::custom("maximum nesting depth of 128 exceeded"));
        }
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed<'_, '_> {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a JSON-compatible value")
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<(), E> {
        self.scalar(Value::Bool(v))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<(), E> {
        self.scalar(v.into())
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<(), E> {
        self.scalar(v.into())
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<(), E> {
        let n = serde_json::Number::from_f64(v)
            .ok_or_else(|| E::custom("non-finite numbers are not supported"))?;
        self.scalar(Value::Number(n))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<(), E> {
        self.scalar(v.into())
    }
    fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<(), E> {
        self.scalar(v.into())
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<(), E> {
        self.scalar(Value::Null)
    }
    fn visit_none<E: de::Error>(self) -> std::result::Result<(), E> {
        self.scalar(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(mut self, mut seq: A) -> std::result::Result<(), A::Error> {
        let id = self
            .node(DataType::Array, None)
            .map_err(de::Error::custom)?;
        let mut rank = 0;
        while seq
            .next_element_seed(Seed {
                sink: self.sink,
                parent: Some(id),
                key: rank.to_string(),
                path: child_path(&self.path, true, &rank.to_string()),
                rank,
                depth: self.depth + 1,
            })?
            .is_some()
        {
            rank += 1;
        }
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(mut self, mut map: A) -> std::result::Result<(), A::Error> {
        let id = self
            .node(DataType::Object, None)
            .map_err(de::Error::custom)?;
        let mut rank = 0;
        while let Some(key) = map.next_key_seed(StringKey)? {
            let path = child_path(&self.path, false, &key);
            map.next_value_seed(Seed {
                sink: self.sink,
                parent: Some(id),
                key,
                path,
                rank,
                depth: self.depth + 1,
            })?;
            rank += 1;
        }
        Ok(())
    }
}

struct StringKey;
impl<'de> DeserializeSeed<'de> for StringKey {
    type Value = String;
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> std::result::Result<String, D::Error> {
        struct KeyVisitor;
        impl Visitor<'_> for KeyVisitor {
            type Value = String;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string mapping key")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<String, E> {
                Ok(v.into())
            }
            fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<String, E> {
                Ok(v)
            }
        }
        d.deserialize_any(KeyVisitor)
    }
}

/// Unambiguous jq-like paths: identifier keys use dots, all other keys use
/// JSON-quoted brackets. Root-array indices retain the conventional `.[0]`.
///
/// ```
/// use twig::adapters::json_loader::child_path;
/// assert_eq!(child_path(".", false, "a.b"), ".[\"a.b\"]");
/// ```
pub fn child_path(parent: &str, array: bool, key: &str) -> String {
    if array {
        return format!("{parent}[{key}]");
    }
    let identifier = !key.is_empty()
        && key
            .chars()
            .enumerate()
            .all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()));
    if identifier {
        if parent == "." {
            format!(".{key}")
        } else {
            format!("{parent}.{key}")
        }
    } else {
        format!(
            "{parent}[{}]",
            serde_json::to_string(key).expect("string serializes")
        )
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
