//! In-memory representations of tree nodes.
//!
//! `Node` mirrors the Python `@dataclass Node` and `DataType` mirrors the
//! Python `Enum` of the same name. The store reads these rows out of SQLite
//! rather than keeping a live `Value` tree, so this struct stays cheap.

use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Object,
    Array,
    String,
    Integer,
    Float,
    Boolean,
    Null,
}

impl DataType {
    /// Stable lowercase name used both in SQLite and on the wire.
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::Object => "object",
            DataType::Array => "array",
            DataType::String => "string",
            DataType::Integer => "integer",
            DataType::Float => "float",
            DataType::Boolean => "boolean",
            DataType::Null => "null",
        }
    }

    /// Inverse of `as_str`; falls back to `String` for unknown rows so a
    /// corrupted cache can still be read.
    pub fn parse(s: &str) -> Self {
        match s {
            "object" => DataType::Object,
            "array" => DataType::Array,
            "string" => DataType::String,
            "integer" => DataType::Integer,
            "float" => DataType::Float,
            "boolean" => DataType::Boolean,
            "null" => DataType::Null,
            _ => DataType::String,
        }
    }

    /// Mirror of the Python `DataType.from_value` helper. Note that
    /// `bool` is checked before `int` because in Python (and now in
    /// `serde_json::Value`) `true.is_instance(int)` is true; we keep the
    /// same precedence so JSON `true` stays a boolean.
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Null => DataType::Null,
            Value::Bool(_) => DataType::Boolean,
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    DataType::Integer
                } else {
                    DataType::Float
                }
            }
            Value::String(_) => DataType::String,
            Value::Array(_) => DataType::Array,
            Value::Object(_) => DataType::Object,
        }
    }
}

/// Lightweight view of a single row in the `nodes` table.
///
/// `value` is always `None` for containers (we don't store them) and holds
/// a primitive `serde_json::Value` for scalars.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: Uuid,
    pub key: String,
    pub value: Option<Value>,
    pub ty: DataType,
    pub parent: Option<Uuid>,
    pub path: String,
    pub is_expanded: bool,
    /// Sibling position under `parent`. Used for `ORDER BY rank` queries.
    pub rank: i64,
}

impl Node {
    /// Containers (`object` / `array`) hold children in the `nodes` table
    /// rather than embedding them in the value column.
    pub fn is_container(&self) -> bool {
        matches!(self.ty, DataType::Object | DataType::Array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_bool_beats_int() {
        let v = Value::Bool(true);
        assert_eq!(DataType::from_value(&v), DataType::Boolean);
    }

    #[test]
    fn from_value_distinguishes_int_and_float() {
        assert_eq!(DataType::from_value(&Value::from(42i64)), DataType::Integer);
        assert_eq!(
            DataType::from_value(&serde_json::json!(1.5)),
            DataType::Float
        );
    }

    #[test]
    fn from_value_handles_null_string_object_array() {
        assert_eq!(DataType::from_value(&Value::Null), DataType::Null);
        assert_eq!(
            DataType::from_value(&Value::String("x".into())),
            DataType::String
        );
        assert_eq!(
            DataType::from_value(&serde_json::json!({})),
            DataType::Object
        );
        assert_eq!(
            DataType::from_value(&serde_json::json!([])),
            DataType::Array
        );
    }

    #[test]
    fn round_trip_through_as_str() {
        for ty in [
            DataType::Object,
            DataType::Array,
            DataType::String,
            DataType::Integer,
            DataType::Float,
            DataType::Boolean,
            DataType::Null,
        ] {
            assert_eq!(DataType::parse(ty.as_str()), ty);
        }
    }

    #[test]
    fn unknown_type_falls_back_to_string() {
        assert_eq!(DataType::parse("not-a-type"), DataType::String);
    }
}
