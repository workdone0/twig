//! Embedded SQLite schema and FTS5 triggers.

/// Verbatim port of `src/twg/core/schema.sql` from the Python project.
///
/// Pulled in via `include_str!` so the SQL lives next to its
/// declaration but is baked into the binary at compile time.
pub const SCHEMA_SQL: &str = include_str!("../../schema.sql");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_contains_expected_statements() {
        assert!(SCHEMA_SQL.contains("CREATE TABLE IF NOT EXISTS nodes"));
        assert!(SCHEMA_SQL.contains("CREATE VIRTUAL TABLE IF NOT EXISTS nodes_search USING fts5"));
        assert!(SCHEMA_SQL.contains("nodes_ai"));
        assert!(SCHEMA_SQL.contains("nodes_ad"));
        assert!(SCHEMA_SQL.contains("nodes_au"));
    }
}