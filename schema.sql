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
