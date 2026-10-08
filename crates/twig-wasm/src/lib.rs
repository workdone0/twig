//! Narrow string-based bridge: numeric scalar values never round-trip through JS numbers.
use serde::Serialize;
use twig_core::{
    memory::{MemoryStore, MAX_INPUT_BYTES},
    model::Node,
    parser,
    storage::{self, NodeStore},
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

fn error(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}
fn id(s: &str) -> Result<Uuid, JsValue> {
    Uuid::parse_str(s).map_err(error)
}
fn json(value: impl Serialize) -> Result<String, JsValue> {
    serde_json::to_string(&value).map_err(error)
}
fn clip(text: &str, max: usize) -> String {
    let mut chars = text.chars();
    let mut result: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}
#[derive(Serialize)]
struct Row {
    id: String,
    parent: Option<String>,
    key: String,
    path: String,
    kind: &'static str,
    rank: i64,
    value: String,
    container: bool,
}
impl From<Node> for Row {
    fn from(n: Node) -> Self {
        Self {
            id: n.id.to_string(),
            parent: n.parent.map(|id| id.to_string()),
            key: clip(&n.key, 256),
            path: clip(&n.path, 2048),
            kind: n.ty.as_str(),
            rank: n.rank,
            value: n
                .value
                .as_ref()
                .map(|v| clip(&v.to_string(), 256))
                .unwrap_or_default(),
            container: n.is_container(),
        }
    }
}
#[wasm_bindgen]
pub struct Document {
    store: MemoryStore,
}
#[wasm_bindgen]
impl Document {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8], format: &str) -> Result<Document, JsValue> {
        if bytes.len() > MAX_INPUT_BYTES {
            return Err(error("File exceeds 20 MiB. Install Twig for larger files."));
        }
        let mut store = MemoryStore::default();
        let mut input = bytes;
        match format {
            "yaml" => parser::parse_yaml(&mut input, &mut store, &()),
            "json" | "har" => parser::parse_json(&mut input, &mut store, &()),
            _ => return Err(error("Unsupported format")),
        }
        .map_err(error)?;
        Ok(Self { store })
    }
    pub fn info(&self) -> Result<String, JsValue> {
        let root = self
            .store
            .root()
            .and_then(|id| self.store.node(id).ok().flatten())
            .map(Row::from);
        json(
            serde_json::json!({"root": root, "nodes": self.store.len(), "estimatedBytes": self.store.estimated_bytes()}),
        )
    }
    pub fn children(&self, parent: &str, offset: usize) -> Result<String, JsValue> {
        let parent = id(parent)?;
        let node = self
            .store
            .node(parent)
            .unwrap()
            .ok_or_else(|| error("Node not found"))?;
        let (rows, total) = if node.is_container() {
            (
                self.store
                    .children(parent, offset, storage::PAGE_SIZE)
                    .unwrap(),
                self.store.child_count(parent).unwrap(),
            )
        } else {
            (if offset == 0 { vec![node] } else { vec![] }, 1)
        };
        json(
            serde_json::json!({"rows": rows.into_iter().map(Row::from).collect::<Vec<_>>(), "total": total, "offset": offset}),
        )
    }
    pub fn search(
        &self,
        query: &str,
        start: Option<String>,
        direction: i32,
    ) -> Result<String, JsValue> {
        let start = start.as_deref().map(id).transpose()?;
        json(
            self.store
                .search(query, start, direction)
                .unwrap()
                .map(Row::from),
        )
    }
    pub fn resolve(&self, path: &str) -> Result<String, JsValue> {
        json(
            storage::resolve_path(&self.store, path)
                .unwrap()
                .map(Row::from),
        )
    }
    pub fn lineage(&self, target: &str) -> Result<String, JsValue> {
        json(
            storage::lineage(&self.store, id(target)?)
                .unwrap()
                .into_iter()
                .map(Row::from)
                .collect::<Vec<_>>(),
        )
    }
    pub fn preview(&self, target: &str) -> Result<String, JsValue> {
        let value = storage::preview_value(&self.store, id(target)?, 4, &mut 200).unwrap();
        Ok(clip(
            &serde_json::to_string_pretty(&value).map_err(error)?,
            64 * 1024,
        ))
    }
    pub fn export(&self, target: &str) -> Result<String, JsValue> {
        let value = storage::export_value(&self.store, id(target)?).map_err(error)?;
        let text = serde_json::to_string_pretty(&value).map_err(error)?;
        if text.len() > 4 * 1024 * 1024 {
            return Err(error(
                "Selection exceeds the 4 MiB browser clipboard limit. Install Twig to export it.",
            ));
        }
        Ok(text)
    }
    pub fn path(&self, target: &str) -> Result<String, JsValue> {
        let node = self
            .store
            .node(id(target)?)
            .unwrap()
            .ok_or_else(|| error("Node not found"))?;
        if node.path.len() > 4 * 1024 * 1024 {
            return Err(error("Path exceeds the browser clipboard limit"));
        }
        Ok(node.path)
    }
}
