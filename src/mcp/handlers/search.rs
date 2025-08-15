//! Search and export MCP tool handlers

use crate::core::Ticket;
use crate::mcp::handlers::schema_helper::json_to_schema;
use crate::mcp::service::VibeTicketService;
use crate::storage::TicketRepository;
use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{Value, json};
use std::borrow::Cow;
use std::sync::Arc;

/// Register all search and export tools
#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        // Search tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_search"),
            description: Some(Cow::Borrowed("Search tickets by keyword")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "in_title": {
                        "type": "boolean",
                        "description": "Search in titles only"
                    },
                    "in_description": {
                        "type": "boolean",
                        "description": "Search in descriptions only"
                    },
                    "in_tasks": {
                        "type": "boolean",
                        "description": "Search in tasks"
                    }
                },
                "required": ["query"]
            }))),
            annotations: None,
        },
        // Export tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_export"),
            description: Some(Cow::Borrowed("Export tickets in various formats")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "enum": ["json", "yaml", "csv", "markdown"],
                        "description": "Export format"
                    },
                    "ticket": {
                        "type": "string",
                        "description": "Specific ticket ID or slug to export (exports all if not specified)"
                    }
                },
                "required": ["format"]
            }))),
            annotations: None,
        },
        // Import tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_import"),
            description: Some(Cow::Borrowed("Import tickets from JSON or YAML")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "data": {
                        "type": "string",
                        "description": "JSON or YAML data to import"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["json", "yaml"],
                        "description": "Data format",
                        "default": "json"
                    }
                },
                "required": ["data"]
            }))),
            annotations: None,
        },
    ]
}

/// Handle searching tickets
pub fn handle_search(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        query: String,
        in_title: Option<bool>,
        in_description: Option<bool>,
        in_tasks: Option<bool>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let query = args.query.to_lowercase();
    let search_all = !args.in_title.unwrap_or(false)
        && !args.in_description.unwrap_or(false)
        && !args.in_tasks.unwrap_or(false);

    let tickets = service
        .storage
        .load_all()
        .map_err(|e| format!("Failed to list tickets: {e}"))?;

    let mut results = Vec::new();

    for ticket in tickets {
        let mut matches = Vec::new();

        // Search in title
        if (search_all || args.in_title.unwrap_or(false))
            && ticket.title.to_lowercase().contains(&query)
        {
            matches.push("title");
        }

        // Search in description
        if (search_all || args.in_description.unwrap_or(false))
            && ticket.description.to_lowercase().contains(&query)
        {
            matches.push("description");
        }

        // Search in tasks
        if search_all || args.in_tasks.unwrap_or(false) {
            for task in &ticket.tasks {
                if task.title.to_lowercase().contains(&query) {
                    matches.push("tasks");
                    break;
                }
            }
        }

        // Search in slug if searching all
        if search_all && ticket.slug.to_lowercase().contains(&query) {
            matches.push("slug");
        }

        // Search in tags if searching all
        if search_all {
            for tag in &ticket.tags {
                if tag.to_lowercase().contains(&query) {
                    matches.push("tags");
                    break;
                }
            }
        }

        if !matches.is_empty() {
            results.push(json!({
                "id": ticket.id.to_string(),
                "slug": ticket.slug,
                "title": ticket.title,
                "status": format!("{:?}", ticket.status).to_lowercase(),
                "priority": format!("{:?}", ticket.priority).to_lowercase(),
                "matched_in": matches,
                "created_at": ticket.created_at.to_rfc3339()
            }));
        }
    }

    Ok(json!({
        "query": args.query,
        "results": results,
        "count": results.len()
    }))
}

/// Handle exporting tickets
pub fn handle_export(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        format: String,
        ticket: Option<String>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let tickets = if let Some(ticket_ref) = args.ticket {
        // Export specific ticket
        let ticket_id = crate::mcp::handlers::tickets::resolve_ticket_ref(service, &ticket_ref)?;
        let ticket = service
            .storage
            .load(&ticket_id)
            .map_err(|e| format!("Failed to load ticket: {e}"))?;
        vec![ticket]
    } else {
        // Export all tickets
        service
            .storage
            .load_all()
            .map_err(|e| format!("Failed to list tickets: {e}"))?
    };

    let exported_data = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&tickets)
            .map_err(|e| format!("Failed to serialize to JSON: {e}"))?,
        "yaml" => serde_yaml::to_string(&tickets)
            .map_err(|e| format!("Failed to serialize to YAML: {e}"))?,
        "csv" => export_to_csv(&tickets)?,
        "markdown" => export_to_markdown(&tickets),
        _ => return Err(format!("Invalid format: {}", args.format)),
    };

    Ok(json!({
        "format": args.format,
        "ticket_count": tickets.len(),
        "data": exported_data
    }))
}

/// Handle importing tickets
pub fn handle_import(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        data: String,
        format: Option<String>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let format = args.format.unwrap_or_else(|| "json".to_string());

    let tickets: Vec<Ticket> =
        match format.as_str() {
            "json" => serde_json::from_str(&args.data)
                .map_err(|e| format!("Failed to parse JSON: {e}"))?,
            "yaml" => serde_yaml::from_str(&args.data)
                .map_err(|e| format!("Failed to parse YAML: {e}"))?,
            _ => return Err(format!("Invalid format: {format}")),
        };

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut errors = Vec::new();

    for ticket in tickets {
        // Check if ticket already exists
        if service
            .storage
            .load_all()
            .map(|existing| {
                existing
                    .iter()
                    .any(|t| t.id == ticket.id || t.slug == ticket.slug)
            })
            .unwrap_or(false)
        {
            skipped_count += 1;
            continue;
        }

        match service.storage.save(&ticket) {
            Ok(()) => imported_count += 1,
            Err(e) => errors.push(format!("Failed to import '{}': {}", ticket.slug, e)),
        }
    }

    let mut response = json!({
        "imported": imported_count,
        "skipped": skipped_count,
        "total": imported_count + skipped_count
    });

    if !errors.is_empty() {
        response["errors"] = json!(errors);
    }

    Ok(response)
}

/// Export tickets to CSV format
fn export_to_csv(tickets: &[Ticket]) -> Result<String, String> {
    use std::io::Write as IoWrite;

    let mut csv_data = Vec::new();

    // Write header
    writeln!(&mut csv_data, "ID,Slug,Title,Status,Priority,Assignee,Tags,Created,Started,Closed,Tasks Total,Tasks Completed")
        .map_err(|e| format!("Failed to write CSV header: {e}"))?;

    // Write ticket rows
    for ticket in tickets {
        let tags = ticket.tags.join(";");
        let tasks_total = ticket.tasks.len();
        let tasks_completed = ticket.tasks.iter().filter(|t| t.completed).count();

        writeln!(
            &mut csv_data,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            ticket.id,
            ticket.slug,
            escape_csv(&ticket.title),
            format!("{:?}", ticket.status).to_lowercase(),
            format!("{:?}", ticket.priority).to_lowercase(),
            ticket.assignee.as_deref().unwrap_or(""),
            tags,
            ticket.created_at.to_rfc3339(),
            ticket
                .started_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            ticket
                .closed_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            tasks_total,
            tasks_completed
        )
        .map_err(|e| format!("Failed to write CSV row: {e}"))?;
    }

    String::from_utf8(csv_data).map_err(|e| format!("Failed to convert CSV to string: {e}"))
}

/// Export tickets to Markdown format
fn export_to_markdown(tickets: &[Ticket]) -> String {
    use std::fmt::Write as FmtWrite;

    let mut md = String::new();

    writeln!(&mut md, "# Tickets Export\n").unwrap();
    writeln!(
        &mut md,
        "Generated on: {}\n",
        chrono::Utc::now().to_rfc3339()
    )
    .unwrap();

    for ticket in tickets {
        writeln!(&mut md, "## {} - {}", ticket.slug, ticket.title).unwrap();
        writeln!(&mut md).unwrap();
        writeln!(&mut md, "- **ID**: {}", ticket.id).unwrap();
        writeln!(&mut md, "- **Status**: {:?}", ticket.status).unwrap();
        writeln!(&mut md, "- **Priority**: {:?}", ticket.priority).unwrap();

        if let Some(assignee) = &ticket.assignee {
            writeln!(&mut md, "- **Assignee**: {assignee}").unwrap();
        }

        if !ticket.tags.is_empty() {
            writeln!(&mut md, "- **Tags**: {}", ticket.tags.join(", ")).unwrap();
        }

        writeln!(&mut md, "- **Created**: {}", ticket.created_at.to_rfc3339()).unwrap();

        if let Some(started) = ticket.started_at {
            writeln!(&mut md, "- **Started**: {}", started.to_rfc3339()).unwrap();
        }

        if let Some(closed) = ticket.closed_at {
            writeln!(&mut md, "- **Closed**: {}", closed.to_rfc3339()).unwrap();
        }

        if !ticket.description.is_empty() {
            writeln!(&mut md, "\n### Description\n").unwrap();
            writeln!(&mut md, "{}", ticket.description).unwrap();
        }

        if !ticket.tasks.is_empty() {
            writeln!(&mut md, "\n### Tasks\n").unwrap();
            for task in &ticket.tasks {
                let checkbox = if task.completed { "x" } else { " " };
                writeln!(&mut md, "- [{}] {}", checkbox, task.title).unwrap();
            }
        }

        writeln!(&mut md, "\n---\n").unwrap();
    }

    md
}

/// Escape CSV field if needed
fn escape_csv(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
