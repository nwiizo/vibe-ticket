//! Task management MCP tool handlers

use crate::core::{Task, TaskId};
use crate::mcp::handlers::schema_helper::json_to_schema;
use crate::mcp::service::VibeTicketService;
use crate::storage::{ActiveTicketRepository, TicketRepository};
use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{Value, json};
use std::borrow::Cow;
use std::sync::Arc;

/// Register all task management tools
#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        // Add task tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_task_add"),
            description: Some(Cow::Borrowed("Add a task to a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Task title"
                    },
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug (defaults to active ticket)"
                    }
                },
                "required": ["title"]
            }))),
            title: None,
            output_schema: None,
            icons: None,
            annotations: None,
            meta: None,
        },
        // Complete task tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_task_complete"),
            description: Some(Cow::Borrowed("Mark a task as completed")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to complete"
                    },
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug (defaults to active ticket)"
                    }
                },
                "required": ["task_id"]
            }))),
            title: None,
            output_schema: None,
            icons: None,
            annotations: None,
            meta: None,
        },
        // List tasks tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_task_list"),
            description: Some(Cow::Borrowed("List tasks in a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug (defaults to active ticket)"
                    },
                    "completed_only": {
                        "type": "boolean",
                        "description": "Show only completed tasks"
                    },
                    "incomplete_only": {
                        "type": "boolean",
                        "description": "Show only incomplete tasks"
                    }
                }
            }))),
            title: None,
            output_schema: None,
            icons: None,
            annotations: None,
            meta: None,
        },
        // Remove task tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_task_remove"),
            description: Some(Cow::Borrowed("Remove a task from a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "Task ID to remove"
                    },
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug (defaults to active ticket)"
                    }
                },
                "required": ["task_id"]
            }))),
            title: None,
            output_schema: None,
            icons: None,
            annotations: None,
            meta: None,
        },
    ]
}

/// Helper to resolve ticket reference
fn resolve_ticket_ref(
    service: &VibeTicketService,
    ticket_ref: Option<&str>,
) -> Result<crate::core::TicketId, String> {
    if let Some(ref_str) = ticket_ref {
        crate::mcp::handlers::tickets::resolve_ticket_ref(service, ref_str)
    } else {
        // Get active ticket
        service
            .storage
            .get_active()
            .map_err(|e| format!("Failed to get active ticket: {e}"))?
            .ok_or_else(|| "No active ticket. Please specify a ticket ID or slug.".to_string())
    }
}

/// Handle adding a task
pub fn handle_add(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        title: String,
        ticket: Option<String>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = resolve_ticket_ref(service, args.ticket.as_deref())?;
    let mut ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    let task = Task::new(args.title);
    let task_id = task.id.clone();
    let task_title = task.title.clone();
    ticket.tasks.push(task);

    service
        .storage
        .save(&ticket)
        .map_err(|e| format!("Failed to save ticket: {e}"))?;

    Ok(json!({
        "status": "added",
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "task": {
            "id": task_id.to_string(),
            "title": task_title,
            "completed": false
        },
        "total_tasks": ticket.tasks.len()
    }))
}

/// Handle completing a task
pub fn handle_complete(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        task_id: String,
        ticket: Option<String>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = resolve_ticket_ref(service, args.ticket.as_deref())?;
    let mut ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    let task_id = TaskId::parse_str(&args.task_id)
        .map_err(|_| format!("Invalid task ID: {}", args.task_id))?;

    // Find and complete the task
    let mut task_found = false;
    let mut task_title = String::new();
    for task in &mut ticket.tasks {
        if task.id == task_id {
            if task.completed {
                return Err("Task is already completed".to_string());
            }
            task.completed = true;
            task.completed_at = Some(chrono::Utc::now());
            task_title.clone_from(&task.title);
            task_found = true;
            break;
        }
    }

    if !task_found {
        return Err(format!("Task '{}' not found in ticket", args.task_id));
    }

    service
        .storage
        .save(&ticket)
        .map_err(|e| format!("Failed to save ticket: {e}"))?;

    let completed_count = ticket.tasks.iter().filter(|t| t.completed).count();
    let total_count = ticket.tasks.len();

    Ok(json!({
        "status": "completed",
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "task": {
            "id": task_id.to_string(),
            "title": task_title
        },
        "progress": {
            "completed": completed_count,
            "total": total_count,
            "percentage": if total_count > 0 { (completed_count * 100) / total_count } else { 0 }
        }
    }))
}

/// Handle listing tasks
pub fn handle_list(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: Option<String>,
        completed_only: Option<bool>,
        incomplete_only: Option<bool>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = resolve_ticket_ref(service, args.ticket.as_deref())?;
    let ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    let mut tasks: Vec<&Task> = ticket.tasks.iter().collect();

    if args.completed_only.unwrap_or(false) {
        tasks.retain(|t| t.completed);
    } else if args.incomplete_only.unwrap_or(false) {
        tasks.retain(|t| !t.completed);
    }

    let total_count = ticket.tasks.len();
    let completed_count = ticket.tasks.iter().filter(|t| t.completed).count();

    Ok(json!({
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "ticket_title": ticket.title,
        "progress": {
            "completed": completed_count,
            "total": total_count,
            "percentage": if total_count > 0 { (completed_count * 100) / total_count } else { 0 }
        },
        "tasks": tasks.iter().map(|t| json!({
            "id": t.id.to_string(),
            "title": t.title,
            "completed": t.completed,
            "created_at": t.created_at.to_rfc3339(),
            "completed_at": t.completed_at.map(|dt| dt.to_rfc3339())
        })).collect::<Vec<_>>()
    }))
}

/// Handle removing a task
pub fn handle_remove(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        task_id: String,
        ticket: Option<String>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = resolve_ticket_ref(service, args.ticket.as_deref())?;
    let mut ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    let task_id = TaskId::parse_str(&args.task_id)
        .map_err(|_| format!("Invalid task ID: {}", args.task_id))?;

    // Find and remove the task
    let task_index = ticket
        .tasks
        .iter()
        .position(|t| t.id == task_id)
        .ok_or_else(|| format!("Task '{}' not found in ticket", args.task_id))?;

    let removed_task = ticket.tasks.remove(task_index);

    service
        .storage
        .save(&ticket)
        .map_err(|e| format!("Failed to save ticket: {e}"))?;

    Ok(json!({
        "status": "removed",
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "removed_task": {
            "id": removed_task.id.to_string(),
            "title": removed_task.title,
            "was_completed": removed_task.completed
        },
        "remaining_tasks": ticket.tasks.len()
    }))
}
