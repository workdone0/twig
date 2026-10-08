//! Storage contract and shared exploration behavior. SQL remains a native optimization.
use crate::model::{DataType, Node};
use serde_json::Value;
use uuid::Uuid;

pub trait NodeSink {
    /// Native storage batches SQL writes; memory storage checks each node immediately.
    fn batch_size(&self) -> usize {
        1024
    }
    fn insert_batch(&mut self, nodes: &[Node]) -> anyhow::Result<()>;
}
pub trait NodeStore {
    type Error: std::error::Error + Send + Sync + 'static;
    fn node(&self, id: Uuid) -> Result<Option<Node>, Self::Error>;
    fn children(&self, id: Uuid, offset: usize, limit: usize) -> Result<Vec<Node>, Self::Error>;
    fn child_count(&self, id: Uuid) -> Result<usize, Self::Error>;
    fn exact_path(&self, path: &str) -> Result<Option<Node>, Self::Error>;
    fn search(
        &self,
        query: &str,
        start: Option<Uuid>,
        direction: i32,
    ) -> Result<Option<Node>, Self::Error>;
}

pub fn resolve_path<S: NodeStore + ?Sized>(
    store: &S,
    path: &str,
) -> Result<Option<Node>, S::Error> {
    let path = path.trim();
    if path.is_empty() {
        return Ok(None);
    }
    let normalized = if path.starts_with('.') {
        path.to_owned()
    } else {
        format!(".{path}")
    };
    if let Some(node) = store.exact_path(&normalized)? {
        return Ok(Some(node));
    }
    let stripped = &normalized[1..];
    let fallback = if stripped.starts_with('[') {
        format!(".[0]{stripped}")
    } else {
        format!(".[0].{stripped}")
    };
    store.exact_path(&fallback)
}

pub fn lineage<S: NodeStore + ?Sized>(store: &S, target: Uuid) -> Result<Vec<Node>, S::Error> {
    let mut nodes = Vec::new();
    let mut current = Some(target);
    while let Some(id) = current {
        let Some(node) = store.node(id)? else {
            break;
        };
        current = node.parent;
        nodes.push(node);
    }
    nodes.reverse();
    Ok(nodes)
}

pub const PAGE_SIZE: usize = 256;
pub fn page_offset(position: usize) -> usize {
    position / PAGE_SIZE * PAGE_SIZE
}

pub fn preview_value<S: NodeStore + ?Sized>(
    store: &S,
    id: Uuid,
    depth: usize,
    budget: &mut usize,
) -> Result<Value, S::Error> {
    if *budget == 0 {
        return Ok(Value::String("…".into()));
    }
    *budget -= 1;
    let Some(node) = store.node(id)? else {
        return Ok(Value::Null);
    };
    if !node.is_container() {
        return Ok(node.value.unwrap_or(Value::Null));
    }
    if depth == 0 {
        return Ok(Value::String("…".into()));
    }
    let children = store.children(id, 0, (*budget).min(30))?;
    if node.ty == DataType::Array {
        let mut values = Vec::new();
        for child in children {
            values.push(preview_value(store, child.id, depth - 1, budget)?);
        }
        Ok(Value::Array(values))
    } else {
        let mut values = serde_json::Map::new();
        for child in children {
            values.insert(
                child.key,
                preview_value(store, child.id, depth - 1, budget)?,
            );
        }
        Ok(Value::Object(values))
    }
}

pub fn export_value<S: NodeStore + ?Sized>(store: &S, id: Uuid) -> anyhow::Result<Value> {
    let mut remaining = 10_000;
    fn count<S: NodeStore + ?Sized>(
        store: &S,
        id: Uuid,
        remaining: &mut usize,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            *remaining > 0,
            "Selection exceeds 10,000 nodes; use --print to export the file"
        );
        *remaining -= 1;
        let n = store.child_count(id)?;
        anyhow::ensure!(
            n <= *remaining,
            "Selection exceeds 10,000 nodes; use --print to export the file"
        );
        for offset in (0..n).step_by(PAGE_SIZE) {
            for child in store.children(id, offset, PAGE_SIZE)? {
                count(store, child.id, remaining)?;
            }
        }
        Ok(())
    }
    count(store, id, &mut remaining)?;
    Ok(reconstruct_value(store, id, 130)?)
}
pub fn reconstruct_value<S: NodeStore + ?Sized>(
    store: &S,
    id: Uuid,
    depth: usize,
) -> Result<Value, S::Error> {
    reconstruct_value_inner(store, id, depth, &mut 0)
}
fn reconstruct_value_inner<S: NodeStore + ?Sized>(
    store: &S,
    node_id: Uuid,
    max_depth: usize,
    current_depth: &mut usize,
) -> Result<Value, S::Error> {
    let node = match store.node(node_id)? {
        Some(n) => n,
        None => return Ok(Value::Null),
    };
    if !node.is_container() {
        return Ok(node.value.unwrap_or(Value::Null));
    }
    if *current_depth >= max_depth {
        return Ok(Value::String("...".to_string()));
    }
    *current_depth += 1;
    let children = store.children(node_id, 0, usize::MAX)?;
    let value = match node.ty {
        DataType::Object => {
            let mut map = serde_json::Map::new();
            for child in children {
                map.insert(
                    child.key.clone(),
                    reconstruct_value_inner(store, child.id, max_depth, current_depth)?,
                );
            }
            Value::Object(map)
        }
        DataType::Array => {
            let mut arr = Vec::with_capacity(children.len());
            for child in children {
                arr.push(reconstruct_value_inner(
                    store,
                    child.id,
                    max_depth,
                    current_depth,
                )?);
            }
            Value::Array(arr)
        }
        _ => unreachable!("non-container branch handled above"),
    };
    *current_depth -= 1;
    Ok(value)
}
