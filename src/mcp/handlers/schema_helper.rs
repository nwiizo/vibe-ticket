//! Helper functions for creating MCP tool schemas

use serde_json::{Map, Value, json};

/// Convert a JSON value to a Map for use as tool schema
#[must_use]
pub fn json_to_schema(value: Value) -> Map<String, Value> {
    if let Value::Object(map) = value {
        map
    } else {
        let mut map = Map::new();
        map.insert("type".to_string(), json!("object"));
        map.insert("properties".to_string(), Value::Object(Map::new()));
        map
    }
}
