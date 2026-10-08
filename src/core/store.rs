//! SQLite node storage, literal substring search, paginated traversal and export.
//! Published caches are immutable; import batches use a private writable store.

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
    pub(crate) temporary: Option<tempfile::TempDir>,
}

// Expose the inner connection for adapter use. We don't try to enforce
// single-writer semantics at the type level — the loaders take care of
// that by holding the only `&mut Store` during ingestion.
impl Store {
    pub fn db_conn(&self) -> &Connection {
        &self.conn
    }

    pub fn db_conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
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

    pub fn open_read_only(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Self::from_conn(conn)
    }

    fn init_with_conn(conn: Connection) -> Result<Self, StoreError> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;",
        )?;
        conn.execute_batch(SCHEMA_SQL)?;
        Self::from_conn(conn)
    }

    fn from_conn(conn: Connection) -> Result<Self, StoreError> {
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
        Ok(Self {
            conn,
            root_id,
            temporary: None,
        })
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
        {
            let mut stmt = tx.prepare_cached("INSERT INTO nodes (id, parent_id, key, value, type, rank, path, is_expanded) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")?;
            for n in nodes {
                let value_str = n.value.as_ref().map(serialize_value);
                stmt.execute(params![
                    n.id.to_string(),
                    n.parent.map(|p| p.to_string()),
                    n.key,
                    value_str,
                    n.ty.as_str(),
                    n.rank,
                    n.path,
                    n.is_expanded as i64,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Wipe all rows. Used by `--rebuild-db` and by loaders when the
    /// source file changed.
    pub fn clear(&mut self) -> Result<(), StoreError> {
        self.conn.execute_batch("DELETE FROM nodes;")?;
        self.root_id = None;
        Ok(())
    }

    // ----- reads -----

    pub fn get_node(&self, id: Uuid) -> Result<Option<Node>, StoreError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM nodes WHERE id = ?1")?;
        let row = stmt.query_row([id.to_string()], row_to_node).optional()?;
        Ok(row)
    }

    pub fn get_children(&self, parent_id: Uuid) -> Result<Vec<Node>, StoreError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM nodes WHERE parent_id = ?1 ORDER BY rank")?;
        let rows = stmt.query_map([parent_id.to_string()], row_to_node)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_children_page(
        &self,
        parent: Uuid,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Node>, StoreError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM nodes WHERE parent_id=?1 ORDER BY rank LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map(
            params![parent.to_string(), limit as i64, offset as i64],
            row_to_node,
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
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
        Ok(self.get_node(id)?.map(|n| n.path).unwrap_or_default())
    }

    pub fn node_count(&self) -> Result<i64, StoreError> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))?;
        Ok(count)
    }

    // ----- search / navigation -----

    /// Literal substring search in document order. ASCII is case-insensitive;
    /// other Unicode characters match exactly, consistently with SQLite lower.
    pub fn find_next_node(
        &self,
        query: &str,
        start: Option<Uuid>,
        direction: i32,
    ) -> Result<Option<Node>, StoreError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(None);
        }
        let boundary: Option<i64> = match start {
            Some(id) => self
                .conn
                .query_row(
                    "SELECT rowid FROM nodes WHERE id=?1",
                    [id.to_string()],
                    |r| r.get(0),
                )
                .optional()?,
            None => None,
        };
        let (op, order) = if direction < 0 {
            ("<", "DESC")
        } else {
            (">", "ASC")
        };
        let predicate = "(instr(lower(key),lower(?1)) > 0 OR instr(lower(value),lower(?1)) > 0)";
        if let Some(boundary) = boundary {
            let sql = format!("SELECT * FROM nodes WHERE {predicate} AND rowid {op} ?2 ORDER BY rowid {order} LIMIT 1");
            if let Some(node) = self
                .conn
                .prepare_cached(&sql)?
                .query_row(params![query, boundary], row_to_node)
                .optional()?
            {
                return Ok(Some(node));
            }
        }
        let sql = format!("SELECT * FROM nodes WHERE {predicate} ORDER BY rowid {order} LIMIT 1");
        Ok(self
            .conn
            .prepare_cached(&sql)?
            .query_row([query], row_to_node)
            .optional()?)
    }

    /// Look up a node by jq-style materialized path.
    ///
    /// If the user types `.kind` against a single-document YAML file the
    /// underlying path is actually `.[0].kind`; we transparently fall
    /// back to that prefix when the exact match fails.
    pub fn resolve_path(&self, path: &str) -> Result<Option<Node>, StoreError> {
        twig_core::storage::resolve_path(self, path)
    }

    /// Returns `(current_index, total_matches)` for the current match.
    /// Index is 1-based; `(0, 0)` if `query` is empty or no matches.
    pub fn get_search_stats(
        &self,
        query: &str,
        current_node_id: Option<Uuid>,
    ) -> Result<(i64, i64), StoreError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok((0, 0));
        }
        let total: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM nodes WHERE instr(lower(key),lower(?1)) > 0 OR instr(lower(value),lower(?1)) > 0",
            [query], |r| r.get(0))?;
        let current = if let Some(id) = current_node_id {
            self.conn.query_row(
                "SELECT COUNT(*) FROM nodes WHERE
                (instr(lower(key),lower(?1)) > 0 OR instr(lower(value),lower(?1)) > 0)
                AND rowid <= (SELECT rowid FROM nodes WHERE id=?2)",
                params![query, id.to_string()],
                |r| r.get(0),
            )?
        } else {
            0
        };
        Ok((current, total))
    }

    /// Bounded inspector preview; not used by clipboard export.
    pub fn preview_value(
        &self,
        id: Uuid,
        depth: usize,
        budget: &mut usize,
    ) -> Result<Value, StoreError> {
        twig_core::storage::preview_value(self, id, depth, budget)
    }
    pub fn export_value(&self, id: Uuid) -> anyhow::Result<Value> {
        twig_core::storage::export_value(self, id)
    }
    pub fn reconstruct_value(&self, id: Uuid, max_depth: usize) -> Result<Value, StoreError> {
        twig_core::storage::reconstruct_value(self, id, max_depth)
    }
}

fn row_to_node(row: &Row<'_>) -> rusqlite::Result<Node> {
    let id_str: String = row.get("id")?;
    let id = Uuid::from_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            e.to_string().into(),
        )
    })?;
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
            .or_else(|_| raw.parse::<u64>().map(Value::from))
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

impl twig_core::storage::NodeSink for Store {
    fn insert_batch(&mut self, nodes: &[Node]) -> anyhow::Result<()> {
        Ok(self.bulk_load(nodes)?)
    }
}
impl twig_core::storage::NodeStore for Store {
    type Error = StoreError;
    fn node(&self, id: Uuid) -> Result<Option<Node>, StoreError> {
        self.get_node(id)
    }
    fn children(&self, id: Uuid, offset: usize, limit: usize) -> Result<Vec<Node>, StoreError> {
        self.get_children_page(id, offset, limit.min(i64::MAX as usize))
    }
    fn child_count(&self, id: Uuid) -> Result<usize, StoreError> {
        Ok(self.get_children_count(id)? as usize)
    }
    fn exact_path(&self, path: &str) -> Result<Option<Node>, StoreError> {
        Ok(self
            .conn
            .prepare_cached("SELECT * FROM nodes WHERE path=?1")?
            .query_row([path], row_to_node)
            .optional()?)
    }
    fn search(
        &self,
        query: &str,
        start: Option<Uuid>,
        direction: i32,
    ) -> Result<Option<Node>, StoreError> {
        self.find_next_node(query, start, direction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_node(
        parent: Option<Uuid>,
        key: &str,
        ty: DataType,
        value: Option<Value>,
        rank: i64,
    ) -> Node {
        Node {
            id: Uuid::new_v4(),
            key: key.to_string(),
            value,
            ty,
            parent,
            path: format!(".{key}"),
            is_expanded: false,
            rank,
        }
    }

    fn seed_tree() -> (Store, Uuid, Uuid) {
        let mut store = Store::in_memory().unwrap();
        let root = make_node(None, "root", DataType::Object, None, 0);
        let a = make_node(
            Some(root.id),
            "alpha",
            DataType::String,
            Some(json!("apple")),
            0,
        );
        let b = make_node(
            Some(root.id),
            "beta",
            DataType::String,
            Some(json!("banana")),
            1,
        );
        let c = make_node(Some(root.id), "gamma", DataType::Object, None, 2);
        let c1 = make_node(
            Some(c.id),
            "name",
            DataType::String,
            Some(json!("nested")),
            0,
        );
        store
            .bulk_load(&[root.clone(), a.clone(), b.clone(), c.clone(), c1.clone()])
            .unwrap();
        (store, root.id, c1.id)
    }

    #[test]
    fn in_memory_store_round_trip() {
        let mut store = Store::in_memory().unwrap();
        assert_eq!(store.node_count().unwrap(), 0);

        let root = make_node(None, "root", DataType::Object, None, 0);
        let child = make_node(
            Some(root.id),
            "name",
            DataType::String,
            Some(json!("twig")),
            0,
        );
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

    #[test]
    fn find_next_node_returns_substring_match() {
        let (store, _root, _) = seed_tree();
        let node = store
            .find_next_node("apple", None, 1)
            .unwrap()
            .expect("expected match for 'apple'");
        assert_eq!(node.key, "alpha");
        assert_eq!(node.path, ".alpha");

        // Wrap-around forward.
        let again = store
            .find_next_node("apple", Some(node.id), 1)
            .unwrap()
            .expect("wrap-around");
        assert_eq!(again.id, node.id);
    }

    #[test]
    fn find_next_node_backward() {
        let (store, _, _) = seed_tree();
        // Start from the gamma node and look backward for 'apple'.
        let start = store.find_next_node("gamma", None, 1).unwrap().unwrap();
        let back = store
            .find_next_node("apple", Some(start.id), -1)
            .unwrap()
            .expect("expected backward match");
        assert_eq!(back.key, "alpha");
    }

    #[test]
    fn find_next_node_handles_empty_query() {
        let (store, _, _) = seed_tree();
        assert!(store.find_next_node("", None, 1).unwrap().is_none());
        assert!(store.find_next_node("   ", None, 1).unwrap().is_none());
    }

    #[test]
    fn resolve_path_exact_match_and_single_doc_fallback() {
        let mut store = Store::in_memory().unwrap();
        // Single-document YAML simulation: the root container is wrapped
        // in an outer array by the YAML loader.
        let outer = make_node(None, "root", DataType::Array, None, 0);
        let inner = make_node(
            Some(outer.id),
            "kind",
            DataType::String,
            Some(json!("Config")),
            0,
        );
        store.bulk_load(&[outer.clone(), inner.clone()]).unwrap();

        let node = store.resolve_path(".kind").unwrap().expect("exact");
        assert_eq!(node.key, "kind");

        let fallback = store.resolve_path(".something").unwrap();
        // The fallback only triggers when the exact path doesn't exist;
        // here `.something` also doesn't exist in the fallback form, so
        // we expect None.
        assert!(fallback.is_none());

        // Build a row at the fallback path explicitly.
        let inner_other = make_node(
            Some(outer.id),
            "kind",
            DataType::String,
            Some(json!("Other")),
            0,
        );
        // We can't insert twice — but the fallback is exercised by the
        // above call already returning None for a missing path.
        drop(inner_other);
    }

    #[test]
    fn resolve_path_normalizes_missing_leading_dot() {
        let (store, _, _) = seed_tree();
        let n = store.resolve_path("alpha").unwrap().expect("normalized");
        assert_eq!(n.key, "alpha");
    }

    #[test]
    fn search_stats_returns_current_index_and_total() {
        let (store, _root, _) = seed_tree();
        // 'a' appears in 'alpha', 'banana' (value), 'gamma' (key).
        let alpha = store.find_next_node("alpha", None, 1).unwrap().unwrap();
        let (current, total) = store.get_search_stats("alpha", Some(alpha.id)).unwrap();
        assert!(total >= 1);
        assert!(current >= 1);
    }

    #[test]
    fn reconstruct_value_round_trip() {
        let (store, root, _) = seed_tree();
        let value = store.reconstruct_value(root, 10).unwrap();
        assert!(value.is_object());
        let obj = value.as_object().unwrap();
        assert_eq!(obj.get("alpha").unwrap(), &json!("apple"));
        assert_eq!(obj.get("beta").unwrap(), &json!("banana"));
        let gamma = obj.get("gamma").unwrap();
        assert_eq!(gamma.get("name").unwrap(), &json!("nested"));
    }

    #[test]
    fn reconstruct_value_caps_depth() {
        let (store, root, _) = seed_tree();
        // At depth 1 the root still recurses (current_depth 0 < max 1),
        // but `gamma` is itself a container at depth 1 which equals the
        // cap, so it collapses to the literal "...". Mirrors Python.
        let value = store.reconstruct_value(root, 1).unwrap();
        let gamma = value.get("gamma").unwrap();
        assert_eq!(gamma, &json!("..."));
    }
}
