//! Helper functions for creating MCP tool schemas

use serde_json::{Map, Value, json};
use std::borrow::Cow;
use std::sync::Arc;

#[cfg(feature = "mcp")]
use rmcp::model::Tool;

#[cfg(not(feature = "mcp"))]
use crate::mcp::handlers::common::Tool;

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

/// Create a basic MCP tool with the given parameters
#[must_use]
pub fn create_tool(name: &'static str, description: &'static str, schema: Value) -> Tool {
    Tool {
        name: Cow::Borrowed(name),
        description: Some(Cow::Borrowed(description)),
        input_schema: Arc::new(json_to_schema(schema)),
        title: None,
        output_schema: None,
        icons: None,
        annotations: None,
    }
}

/// Create common ticket properties schema
#[must_use]
pub fn ticket_properties_schema() -> Value {
    json!({
        "slug": {
            "type": "string",
            "description": "Unique identifier slug for the ticket"
        },
        "title": {
            "type": "string",
            "description": "Title of the ticket"
        },
        "description": {
            "type": "string",
            "description": "Detailed description of the ticket"
        },
        "priority": {
            "type": "string",
            "enum": ["low", "medium", "high", "critical"],
            "description": "Priority level",
            "default": "medium"
        },
        "tags": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Tags for categorization"
        },
        "assignee": {
            "type": "string",
            "description": "Assignee for the ticket"
        }
    })
}

/// Create common filter properties schema
#[must_use]
pub fn filter_properties_schema() -> Value {
    json!({
        "status": {
            "type": "string",
            "enum": ["todo", "doing", "done", "blocked", "review"],
            "description": "Filter by status"
        },
        "priority": {
            "type": "string",
            "enum": ["low", "medium", "high", "critical"],
            "description": "Filter by priority"
        },
        "assignee": {
            "type": "string",
            "description": "Filter by assignee"
        },
        "open": {
            "type": "boolean",
            "description": "Show only open tickets"
        },
        "closed": {
            "type": "boolean",
            "description": "Show only closed tickets"
        },
        "tags": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Filter by tags"
        }
    })
}
