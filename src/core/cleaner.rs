//! JSON repair for the `--fix` CLI mode.
//!
//! Delegates the heavy lifting to the `jsonrepair` crate, then
//! post-processes the result to:
//! - sanitize `NaN` / `±Infinity` floats (not valid JSON) into `null`
//! - pretty-print with 2-space indentation, matching the Python
//!   `repair_json` output exactly.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

/// Repairs malformed JSON content.
///
/// Handles trailing commas, single quotes, unquoted keys, missing
/// braces / brackets, comments, unexpected characters, etc. via
/// `jsonrepair`, then walks the parsed tree replacing non-finite floats
/// with `null` and re-serializes with `serde_json::to_string_pretty`.
pub fn repair_json(content: &str) -> Result<String> {
    if content.trim().is_empty() {
        return Ok(content.to_string());
    }

    let opts = jsonrepair::Options::default();
    let repaired = jsonrepair::repair_to_string(content, &opts)
        .map_err(|e| anyhow!("jsonrepair failed: {e}"))
        .context("running jsonrepair")?;

    // Parse then sanitize so we can drop NaN / Infinity safely.
    let parsed: Value =
        serde_json::from_str(&repaired).map_err(|e| anyhow!("repaired JSON still invalid: {e}"))?;
    let sanitized = sanitize(parsed);

    serde_json::to_string_pretty(&sanitized)
        .map_err(|e| anyhow!("failed to serialize repaired JSON: {e}"))
}

fn sanitize(value: Value) -> Value {
    match value {
        Value::Number(n) => match n.as_f64() {
            Some(f) if f.is_finite() => Value::Number(n),
            _ => Value::Null,
        },
        Value::Array(arr) => Value::Array(arr.into_iter().map(sanitize).collect()),
        Value::Object(map) => {
            Value::Object(map.into_iter().map(|(k, v)| (k, sanitize(v))).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_valid_json_is_idempotent() {
        let input = r#"{"a": 1}"#;
        let out = repair_json(input).unwrap();
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed, serde_json::json!({"a": 1}));
    }

    #[test]
    fn repair_trailing_comma() {
        let input = r#"{"a": 1,}"#;
        let parsed: Value = serde_json::from_str(&repair_json(input).unwrap()).unwrap();
        assert_eq!(parsed, serde_json::json!({"a": 1}));
    }

    #[test]
    fn repair_single_quotes() {
        let input = "{'a': 1}";
        let parsed: Value = serde_json::from_str(&repair_json(input).unwrap()).unwrap();
        assert_eq!(parsed, serde_json::json!({"a": 1}));
    }

    #[test]
    fn repair_unquoted_keys() {
        let input = "{a: 1}";
        let parsed: Value = serde_json::from_str(&repair_json(input).unwrap()).unwrap();
        assert_eq!(parsed, serde_json::json!({"a": 1}));
    }

    #[test]
    fn repair_missing_closing_brace() {
        let input = r#"{"a": 1"#;
        let parsed: Value = serde_json::from_str(&repair_json(input).unwrap()).unwrap();
        assert_eq!(parsed, serde_json::json!({"a": 1}));
    }

    #[test]
    fn repair_missing_closing_bracket_and_brace() {
        let input = r#"{"a": [1, 2"#;
        let parsed: Value = serde_json::from_str(&repair_json(input).unwrap()).unwrap();
        assert_eq!(parsed, serde_json::json!({"a": [1, 2]}));
    }

    #[test]
    fn repair_mixed_errors() {
        let input = "{'a': [1, 2, ], c: 3";
        let parsed: Value = serde_json::from_str(&repair_json(input).unwrap()).unwrap();
        assert_eq!(parsed, serde_json::json!({"a": [1, 2], "c": 3}));
    }

    #[test]
    fn repair_sanitizes_nan_and_infinity() {
        // Build a value containing non-finite floats by going through
        // jsonrepair directly with a known-broken input.
        let input = r#"{"x": NaN, "y": Infinity, "z": -Infinity}"#;
        let parsed: Value = serde_json::from_str(&repair_json(input).unwrap()).unwrap();
        assert_eq!(parsed, serde_json::json!({"x": null, "y": null, "z": null}));
    }

    #[test]
    fn empty_input_is_returned_verbatim() {
        let out = repair_json("   ").unwrap();
        assert_eq!(out, "   ");
    }
}
