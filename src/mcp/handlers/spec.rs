//! Spec-driven development MCP tool handlers

use crate::mcp::handlers::schema_helper::json_to_schema;
use crate::mcp::service::VibeTicketService;
use crate::storage::TicketRepository;
use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{Value, json};
use std::borrow::Cow;
use std::sync::Arc;

/// Register all spec-driven development tools
#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        // Add spec tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_add"),
            description: Some(Cow::Borrowed("Add specifications to a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug"
                    },
                    "spec_type": {
                        "type": "string",
                        "enum": ["requirements", "design", "tasks"],
                        "description": "Type of specification to add"
                    },
                    "content": {
                        "type": "object",
                        "description": "Specification content"
                    }
                },
                "required": ["ticket", "spec_type", "content"]
            }))),
            annotations: None,
        },
        // Update spec tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_update"),
            description: Some(Cow::Borrowed("Update specifications for a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug"
                    },
                    "spec_type": {
                        "type": "string",
                        "enum": ["requirements", "design", "tasks"],
                        "description": "Type of specification to update"
                    },
                    "content": {
                        "type": "object",
                        "description": "Updated specification content"
                    }
                },
                "required": ["ticket", "spec_type", "content"]
            }))),
            annotations: None,
        },
        // Check spec tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_check"),
            description: Some(Cow::Borrowed("Check specification status for a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug"
                    }
                },
                "required": ["ticket"]
            }))),
            annotations: None,
        },
    ]
}

/// Handle adding specifications
pub async fn handle_add(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
        spec_type: String,
        content: Value,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id =
        crate::mcp::handlers::tickets::resolve_ticket_ref(service, &args.ticket).await?;
    let mut ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    // Store specification in metadata
    let spec_key = format!("spec_{}", args.spec_type);
    ticket.metadata.insert(
        spec_key.clone(),
        Value::String(
            serde_json::to_string(&args.content)
                .map_err(|e| format!("Failed to serialize spec: {e}"))?,
        ),
    );

    ticket.metadata.insert(
        format!("{spec_key}_updated_at"),
        Value::String(chrono::Utc::now().to_rfc3339()),
    );

    service
        .storage
        .save(&ticket)
        .map_err(|e| format!("Failed to save ticket: {e}"))?;

    Ok(json!({
        "status": "added",
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "spec_type": args.spec_type,
    }))
}

/// Handle updating specifications
pub async fn handle_update(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    // Update uses the same logic as add
    handle_add(service, arguments).await
}

/// Handle checking specification status
pub async fn handle_check(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id =
        crate::mcp::handlers::tickets::resolve_ticket_ref(service, &args.ticket).await?;
    let ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    let mut specs = json!({});

    // Check for each spec type
    for spec_type in ["requirements", "design", "tasks"] {
        let spec_key = format!("spec_{spec_type}");
        if let Some(spec_json) = ticket.metadata.get(&spec_key) {
            specs[spec_type] = json!({
                "exists": true,
                "updated_at": ticket.metadata.get(&format!("{spec_key}_updated_at")),
                "content": spec_json.as_str().and_then(|s| serde_json::from_str::<Value>(s).ok())
            });
        } else {
            specs[spec_type] = json!({
                "exists": false
            });
        }
    }

    Ok(json!({
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "specifications": specs
    }))
}
