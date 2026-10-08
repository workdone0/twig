//! Session-only browser storage. Budgets bound retained nodes and estimated allocations.
use crate::{
    model::Node,
    storage::{NodeSink, NodeStore},
};
use std::collections::HashMap;
use uuid::Uuid;

pub const MAX_INPUT_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_NODES: usize = 250_000;
pub const MAX_ESTIMATED_BYTES: usize = 128 * 1024 * 1024;

#[derive(Default)]
pub struct MemoryStore {
    nodes: Vec<Node>,
    ids: HashMap<Uuid, usize>,
    paths: HashMap<String, usize>,
    children: HashMap<Uuid, Vec<usize>>,
    estimated_bytes: usize,
}
impl MemoryStore {
    pub fn root(&self) -> Option<Uuid> {
        self.nodes.first().map(|n| n.id)
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }
}
impl NodeSink for MemoryStore {
    fn batch_size(&self) -> usize {
        1
    }
    fn insert_batch(&mut self, nodes: &[Node]) -> anyhow::Result<()> {
        for node in nodes {
            // Account for node, indexes, duplicated paths, scalar and allocator overhead.
            let bytes = 512
                + node.key.len()
                + node.path.len() * 2
                + node
                    .value
                    .as_ref()
                    .map_or(0, |v| v.as_str().map_or(32, str::len));
            anyhow::ensure!(
                self.nodes.len() < MAX_NODES
                    && self.estimated_bytes.saturating_add(bytes) <= MAX_ESTIMATED_BYTES,
                "Browser resource limit reached. Install Twig for larger files."
            );
            anyhow::ensure!(
                !self.paths.contains_key(&node.path),
                "Duplicate key or ambiguous path: {}",
                node.path
            );
            self.estimated_bytes += bytes;
            let index = self.nodes.len();
            self.ids.insert(node.id, index);
            self.paths.insert(node.path.clone(), index);
            if let Some(parent) = node.parent {
                self.children.entry(parent).or_default().push(index);
            }
            self.nodes.push(node.clone());
        }
        Ok(())
    }
}
impl NodeStore for MemoryStore {
    type Error = std::convert::Infallible;
    fn node(&self, id: Uuid) -> Result<Option<Node>, Self::Error> {
        Ok(self.ids.get(&id).map(|&i| self.nodes[i].clone()))
    }
    fn children(&self, id: Uuid, offset: usize, limit: usize) -> Result<Vec<Node>, Self::Error> {
        Ok(self
            .children
            .get(&id)
            .map(|items| {
                items
                    .iter()
                    .skip(offset)
                    .take(limit)
                    .map(|&i| self.nodes[i].clone())
                    .collect()
            })
            .unwrap_or_default())
    }
    fn child_count(&self, id: Uuid) -> Result<usize, Self::Error> {
        Ok(self.children.get(&id).map_or(0, Vec::len))
    }
    fn exact_path(&self, path: &str) -> Result<Option<Node>, Self::Error> {
        Ok(self.paths.get(path).map(|&i| self.nodes[i].clone()))
    }
    fn search(
        &self,
        query: &str,
        start: Option<Uuid>,
        direction: i32,
    ) -> Result<Option<Node>, Self::Error> {
        let query = query.trim().to_ascii_lowercase();
        let len = self.nodes.len();
        if query.is_empty() || len == 0 {
            return Ok(None);
        }
        let boundary = start.and_then(|id| self.ids.get(&id).copied());
        let first = if direction < 0 {
            boundary.map_or(len - 1, |i| (i + len - 1) % len)
        } else {
            boundary.map_or(0, |i| (i + 1) % len)
        };
        for step in 0..len {
            let i = if direction < 0 {
                (first + len - step) % len
            } else {
                (first + step) % len
            };
            let node = &self.nodes[i];
            let value = node
                .value
                .as_ref()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
                .unwrap_or_default();
            if node.key.to_ascii_lowercase().contains(&query)
                || value.to_ascii_lowercase().contains(&query)
            {
                return Ok(Some(node.clone()));
            }
        }
        Ok(None)
    }
}
