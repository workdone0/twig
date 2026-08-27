//! SQLite-backed store with FTS5 search.
//!
//! `Store` is the Rust analogue of the Python `SQLiteModel`. It owns a
//! single `rusqlite::Connection`, applies the same PRAGMAs the Python
//! version used (WAL, synchronous=NORMAL, cache_size=-64000, temp_store
//! =MEMORY), and exposes the read API the UI needs:
//!
//! - `get_node`, `get_children`, `get_children_count`, `get_path`
//! - `find_next_node`, `resolve_path`, `get_search_stats`
//! - `reconstruct_value` (depth-limited rebuild of the native
//!   `serde_json::Value` tree)
//!
//! Writes go through `Store::bulk_load`, which is used by the streaming
//! adapters during ingestion.

use std::path::Path;
use std::str::FromStr;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::Value;
use uuid::Uuid;

use crate::core::model::{DataType, Node};
use crate::core::schema::SCHEMA_SQL;

#[derive(Debug)]
pub struct Store {
    pub conn: Connection,
    pub root_id: Option<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid uuid in row: {0}")]
    BadUuid(String),
}

impl Store {
    /// Open a fresh in-memory store with the schema applied.
    pub fn in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        Self::init_with_conn(conn)
    }

    /// Open a store at the given path. If the file does not exist it is
    /// created and the schema applied; if it does, it is left untouched.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        Self::init_with_conn(conn)
    }

    fn init_with_conn(conn: Connection) -> Result<Self, StoreError> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;",
        )?;
        conn.execute_batch(SCHEMA_SQL)?;
        let root_id = conn
            .query_row(
                "SELECT id FROM nodes WHERE parent_id IS NULL LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let root_id = match root_id {
            Some(s) => Some(Uuid::from_str(&s).map_err(|e| StoreError::BadUuid(e.to_string()))?),
            None => None,
        };
        Ok(Self { conn, root_id })
    }

    // ----- writes -----

    /// Bulk-load a batch of nodes inside a single transaction.
    ///
    /// This is the only write API; loaders are expected to call it once
    /// per chunk (typically 10k rows). The caller is responsible for
    /// dropping indexes / rebuilding them around the bulk load — the
    /// `Loader` impls orchestrate that dance.
    pub fn bulk_load(&mut self, nodes: &[Node]) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        for n in nodes {
            let value_str = n.value.as_ref().map(serialize_value);
            tx.execute(
                "INSERT INTO nodes (id, parent_id, key, value, type, rank, path, is_expanded)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    n.id.to_string(),
                    n.parent.map(|p| p.to_string()),
                    n.key,
                    value_str,
                    n.ty.as_str(),
                    n.rank,
                    n.path,
                    n.is_expanded as i64,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Wipe all rows. Used by `--rebuild-db` and by loaders when the
    /// source file changed.
    pub fn clear(&mut self) -> Result<(), StoreError> {
        // Two steps: drop everything in `nodes` (triggers cascade the
        // deletion into the FTS mirror), then make sure no orphan FTS
        // rows remain by issuing the contentless-style `'delete-all'`
        // command against `nodes_search`.
        self.conn.execute_batch(
            "DELETE FROM nodes;
             INSERT INTO nodes_search(nodes_search) VALUES('delete-all');",
        )?;
        Ok(())
    }

    // ----- reads -----

    pub fn get_node(&self, id: Uuid) -> Result<Option<Node>, StoreError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM nodes WHERE id = ?1")?;
        let row = stmt
            .query_row([id.to_string()], row_to_node)
            .optional()?;
        Ok(row)
    }

    pub fn get_children(&self, parent_id: Uuid) -> Result<Vec<Node>, StoreError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM nodes WHERE parent_id = ?1 ORDER BY rank")?;
        let rows = stmt.query_map([parent_id.to_string()], row_to_node)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_children_count(&self, parent_id: Uuid) -> Result<i64, StoreError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT COUNT(*) FROM nodes WHERE parent_id = ?1")?;
        let count = stmt.query_row([parent_id.to_string()], |row| row.get::<_, i64>(0))?;
        Ok(count)
    }

    /// jq-style materialized path from the `path` column.
    pub fn get_path(&self, id: Uuid) -> Result<String, StoreError> {
        Ok(self
            .get_node(id)?
            .map(|n| n.path)
            .unwrap_or_default())
    }

    pub fn node_count(&self) -> Result<i64, StoreError> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))?;
        Ok(count)
    }
}

fn row_to_node(row: &Row<'_>) -> rusqlite::Result<Node> {
    let id_str: String = row.get("id")?;
    let id = Uuid::from_str(&id_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, e.to_string().into()))?;
    let parent_str: Option<String> = row.get("parent_id")?;
    let parent = parent_str
        .map(|s| {
            Uuid::from_str(&s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    e.to_string().into(),
                )
            })
        })
        .transpose()?;
    let key: String = row.get("key")?;
    let value_str: Option<String> = row.get("value")?;
    let ty_str: String = row.get("type")?;
    let ty = DataType::parse(&ty_str);
    let value = value_str.and_then(|s| deserialize_value(ty, &s));
    let path: String = row.get("path")?;
    let is_expanded: i64 = row.get("is_expanded")?;
    let rank: i64 = row.get("rank")?;
    Ok(Node {
        id,
        key,
        value,
        ty,
        parent,
        path,
        is_expanded: is_expanded != 0,
        rank,
    })
}

fn serialize_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}

fn deserialize_value(ty: DataType, raw: &str) -> Option<Value> {
    Some(match ty {
        DataType::Null => Value::Null,
        DataType::Boolean => Value::Bool(matches!(raw.to_ascii_lowercase().as_str(), "true")),
        DataType::Integer => raw
            .parse::<i64>()
            .map(Value::from)
            .unwrap_or(Value::String(raw.into())),
        DataType::Float => match raw.parse::<f64>() {
            Ok(f) if f.is_finite() => serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::String(raw.into())),
            _ => Value::String(raw.into()),
        },
        DataType::String => Value::String(raw.to_string()),
        DataType::Object | DataType::Array => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_node(parent: Option<Uuid>, key: &str, ty: DataType, value: Option<Value>, rank: i64) -> Node {
        Node {
            id: Uuid::new_v4(),
            key: key.to_string(),
            value,
            ty,
            parent,
            path: format!(".{}", key),
            is_expanded: false,
            rank,
        }
    }

    #[test]
    fn in_memory_store_round_trip() {
        let mut store = Store::in_memory().unwrap();
        assert_eq!(store.node_count().unwrap(), 0);

        let root = make_node(None, "root", DataType::Object, None, 0);
        let child = make_node(Some(root.id), "name", DataType::String, Some(json!("twig")), 0);
        store.bulk_load(&[root.clone(), child.clone()]).unwrap();

        let root_back = store.get_node(root.id).unwrap().unwrap();
        assert_eq!(root_back.key, "root");
        assert_eq!(root_back.ty, DataType::Object);
        assert!(root_back.value.is_none());

        let children = store.get_children(root.id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].key, "name");
        assert_eq!(children[0].value.as_ref().unwrap(), &json!("twig"));

        assert_eq!(store.get_children_count(root.id).unwrap(), 1);
        assert_eq!(store.get_path(child.id).unwrap(), ".name");
    }

    #[test]
    fn bulk_load_preserves_scalar_types() {
        let mut store = Store::in_memory().unwrap();
        let root = make_node(None, "root", DataType::Object, None, 0);
        let kids = [
            ("s", json!("hello"), DataType::String),
            ("i", json!(42), DataType::Integer),
            ("f", json!(1.5), DataType::Float),
            ("b", json!(true), DataType::Boolean),
            ("n", json!(null), DataType::Null),
        ];
        let mut rows = vec![root.clone()];
        for (i, (k, v, ty)) in kids.iter().enumerate() {
            rows.push(make_node(Some(root.id), k, *ty, Some(v.clone()), i as i64));
        }
        store.bulk_load(&rows).unwrap();

        let children = store.get_children(root.id).unwrap();
        let by_key: std::collections::HashMap<_, _> =
            children.iter().map(|n| (n.key.as_str(), n)).collect();
        assert_eq!(by_key["s"].value.as_ref().unwrap(), &json!("hello"));
        assert_eq!(by_key["i"].value.as_ref().unwrap(), &json!(42));
        assert_eq!(by_key["f"].value.as_ref().unwrap(), &json!(1.5));
        assert_eq!(by_key["b"].value.as_ref().unwrap(), &json!(true));
        assert_eq!(by_key["n"].value.as_ref().unwrap(), &json!(null));
    }

    #[test]
    fn clear_resets_state() {
        let mut store = Store::in_memory().unwrap();
        let root = make_node(None, "root", DataType::Object, None, 0);
        store.bulk_load(&[root]).unwrap();
        assert_eq!(store.node_count().unwrap(), 1);
        store.clear().unwrap();
        assert_eq!(store.node_count().unwrap(), 0);
    }
}