use crate::model::{DataType, Node};
use anyhow::Result;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use uuid::Uuid;

/// Native loading supplies cancellation/progress; browsers cancel their worker.
pub trait ParseControl {
    fn check_cancelled(&self) -> Result<()> {
        Ok(())
    }
    fn node_emitted(&self) {}
}
impl ParseControl for () {}

pub fn parse_json(
    reader: &mut dyn std::io::Read,
    store: &mut dyn crate::storage::NodeSink,
    control: &dyn ParseControl,
) -> Result<()> {
    let mut sink = Sink::new(store, control);
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
    de.end()
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON: expected a single document: {e}"))?;
    sink.flush()?;
    Ok(())
}
pub fn parse_yaml(
    reader: &mut dyn std::io::Read,
    store: &mut dyn crate::storage::NodeSink,
    control: &dyn ParseControl,
) -> Result<()> {
    let mut sink = Sink::new(store, control);
    let root = sink.id();
    sink.push(Node {
        id: root,
        parent: None,
        key: "root".into(),
        path: ".".into(),
        ty: DataType::Array,
        value: None,
        rank: 0,
        is_expanded: false,
    })?;
    for (rank, doc) in serde_norway::Deserializer::from_reader(reader).enumerate() {
        Seed {
            sink: &mut sink,
            parent: Some(root),
            key: rank.to_string(),
            path: format!(".[{rank}]"),
            rank: rank as i64,
            depth: 0,
        }
        .deserialize(doc)
        .map_err(|e| anyhow::anyhow!("YAML parse error: {e}"))?;
    }
    sink.flush()?;
    Ok(())
}
pub(crate) struct Sink<'a> {
    store: &'a mut dyn crate::storage::NodeSink,
    options: &'a dyn ParseControl,
    batch: Vec<Node>,
    next_id: u128,
}
impl<'a> Sink<'a> {
    pub fn new(store: &'a mut dyn crate::storage::NodeSink, options: &'a dyn ParseControl) -> Self {
        Self {
            store,
            options,
            batch: Vec::with_capacity(1024),
            next_id: 0,
        }
    }
    fn id(&mut self) -> Uuid {
        self.next_id += 1;
        Uuid::from_u128(self.next_id)
    }
    pub fn push(&mut self, node: Node) -> Result<()> {
        self.options.check_cancelled()?;
        self.batch.push(node);
        self.options.node_emitted();
        if self.batch.len() >= self.store.batch_size() {
            self.flush()?;
        }
        Ok(())
    }
    pub fn flush(&mut self) -> Result<()> {
        self.options.check_cancelled()?;
        self.store.insert_batch(&self.batch)?;
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
        let id = self.sink.id();
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
/// use twig_core::parser::child_path;
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
