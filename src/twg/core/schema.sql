-- Main storage for hierarchical data
CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,       -- UUID
    parent_id TEXT,            -- UUID, NULL for root
    key TEXT,                  -- JSON key or array index
    value TEXT,                -- Serialized value (for display) or raw primitive
    type TEXT,                 -- object, array, string, integer, float, boolean, null
    rank INTEGER,              -- To maintain order (0, 1, 2...)
    path TEXT,                 -- Materialized path (e.g. .users[0].name) for fast lookups
    is_expanded INTEGER DEFAULT 0
);

-- Index for tree traversal and ordering
CREATE INDEX IF NOT EXISTS idx_parent_rank ON nodes(parent_id, rank);
-- Index for path lookups
CREATE INDEX IF NOT EXISTS idx_path ON nodes(path);

-- FTS5 Search Table
-- content='nodes' makes it a "Contentless" table if we omitted content_rowid, 
-- but here we use an "External Content" table pointing to 'nodes' to save space 
-- while keeping FTS index separate.
-- actually, standard FTS5 with external content is complex to keep in sync.
-- reliable approach: standard FTS5 table, populated manually or via triggers.
-- We will use a separate FTS table and keep it in sync via triggers.

CREATE VIRTUAL TABLE IF NOT EXISTS nodes_search USING fts5(
    key, 
    value, 
    path, 
    content='nodes', 
    content_rowid='rowid'
);

-- Triggers to keep FTS updated
CREATE TRIGGER IF NOT EXISTS nodes_ai AFTER INSERT ON nodes BEGIN
  INSERT INTO nodes_search(rowid, key, value, path) VALUES (new.rowid, new.key, new.value, new.path);
END;

CREATE TRIGGER IF NOT EXISTS nodes_ad AFTER DELETE ON nodes BEGIN
  INSERT INTO nodes_search(nodes_search, rowid, key, value, path) VALUES('delete', old.rowid, old.key, old.value, old.path);
END;

CREATE TRIGGER IF NOT EXISTS nodes_au AFTER UPDATE ON nodes BEGIN
  INSERT INTO nodes_search(nodes_search, rowid, key, value, path) VALUES('delete', old.rowid, old.key, old.value, old.path);
  INSERT INTO nodes_search(rowid, key, value, path) VALUES (new.rowid, new.key, new.value, new.path);
END;
