//! Ticket management MCP tool handlers

use crate::core::{Priority, Status, Ticket, TicketId};
use crate::mcp::handlers::schema_helper::json_to_schema;
use crate::mcp::service::VibeTicketService;
use crate::storage::{ActiveTicketRepository, TicketRepository};
use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{Value, json};
use std::borrow::Cow;
use std::sync::Arc;

/// Register all ticket management tools
#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        create_new_ticket_tool(),
        create_list_tickets_tool(),
        create_show_ticket_tool(),
        create_edit_ticket_tool(),
        create_close_ticket_tool(),
        create_start_ticket_tool(),
        create_check_status_tool(),
    ]
}

fn create_new_ticket_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("vibe-ticket_new"),
        description: Some(Cow::Borrowed("Create a new ticket")),
        input_schema: Arc::new(json_to_schema(json!({
            "type": "object",
            "properties": {
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
            },
            "required": ["slug", "title"]
        }))),
        annotations: None,
    }
}

fn create_list_tickets_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("vibe-ticket_list"),
        description: Some(Cow::Borrowed("List tickets with optional filters")),
        input_schema: Arc::new(json_to_schema(json!({
            "type": "object",
            "properties": {
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
            }
        }))),
        annotations: None,
    }
}

fn create_show_ticket_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("vibe-ticket_show"),
        description: Some(Cow::Borrowed("Show detailed information about a ticket")),
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
    }
}

fn create_edit_ticket_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("vibe-ticket_edit"),
        description: Some(Cow::Borrowed("Edit ticket properties")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug"
                    },
                    "title": {
                        "type": "string",
                        "description": "New title"
                    },
                    "description": {
                        "type": "string",
                        "description": "New description"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["todo", "doing", "done", "blocked", "review"],
                        "description": "New status"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "New priority"
                    },
                    "assignee": {
                        "type": "string",
                        "description": "New assignee"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "New tags (replaces existing)"
                    }
                },
                "required": ["ticket"]
            }))),
        annotations: None,
    }
}

fn create_close_ticket_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("vibe-ticket_close"),
        description: Some(Cow::Borrowed("Close a ticket")),
        input_schema: Arc::new(json_to_schema(json!({
            "type": "object",
            "properties": {
                "ticket": {
                    "type": "string",
                    "description": "Ticket ID or slug"
                },
                "message": {
                    "type": "string",
                    "description": "Closing message"
                }
            },
            "required": ["ticket"]
        }))),
        annotations: None,
    }
}

fn create_start_ticket_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("vibe-ticket_start"),
        description: Some(Cow::Borrowed("Start working on a ticket")),
        input_schema: Arc::new(json_to_schema(json!({
            "type": "object",
            "properties": {
                "ticket": {
                    "type": "string",
                    "description": "Ticket ID or slug"
                },
                "no_worktree": {
                    "type": "boolean",
                    "description": "Skip creating Git worktree",
                    "default": false
                }
            },
            "required": ["ticket"]
        }))),
        annotations: None,
    }
}

fn create_check_status_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("vibe-ticket_check"),
        description: Some(Cow::Borrowed("Check current ticket status")),
        input_schema: Arc::new(json_to_schema(json!({
            "type": "object",
            "properties": {}
        }))),
        annotations: None,
    }
}

/// Helper to resolve ticket reference (ID or slug)
pub fn resolve_ticket_ref(
    service: &VibeTicketService,
    ticket_ref: &str,
) -> Result<TicketId, String> {
    // Try parsing as ID first
    if let Ok(id) = TicketId::parse_str(ticket_ref) {
        return Ok(id);
    }

    // Otherwise, search by slug
    let tickets = service
        .storage
        .load_all()
        .map_err(|e| format!("Failed to list tickets: {e}"))?;

    for ticket in tickets {
        if ticket.slug == ticket_ref {
            return Ok(ticket.id);
        }
    }

    Err(format!("Ticket not found: {ticket_ref}"))
}

/// Handle creating a new ticket
pub fn handle_new(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        slug: String,
        title: String,
        description: Option<String>,
        priority: Option<String>,
        tags: Option<Vec<String>>,
        assignee: Option<String>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let mut ticket = Ticket::new(args.slug.clone(), args.title);

    if let Some(desc) = args.description {
        ticket.description = desc;
    }

    if let Some(priority_str) = args.priority {
        ticket.priority = match priority_str.as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => return Err(format!("Invalid priority: {priority_str}")),
        };
    }

    if let Some(tags) = args.tags {
        ticket.tags = tags;
    }

    if let Some(assignee) = args.assignee {
        ticket.assignee = Some(assignee);
    }

    service
        .storage
        .save(&ticket)
        .map_err(|e| format!("Failed to save ticket: {e}"))?;

    Ok(json!({
        "status": "created",
        "ticket": {
            "id": ticket.id.to_string(),
            "slug": ticket.slug,
            "title": ticket.title,
            "priority": format!("{:?}", ticket.priority).to_lowercase(),
            "status": format!("{:?}", ticket.status).to_lowercase(),
        }
    }))
}

/// Handle listing tickets
pub fn handle_list(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        status: Option<String>,
        priority: Option<String>,
        assignee: Option<String>,
        open: Option<bool>,
        closed: Option<bool>,
        tags: Option<Vec<String>>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let mut tickets = service
        .storage
        .load_all()
        .map_err(|e| format!("Failed to list tickets: {e}"))?;

    // Apply filters
    if let Some(status_str) = args.status {
        let status = match status_str.as_str() {
            "todo" => Status::Todo,
            "doing" => Status::Doing,
            "done" => Status::Done,
            "blocked" => Status::Blocked,
            "review" => Status::Review,
            _ => return Err(format!("Invalid status: {status_str}")),
        };
        tickets.retain(|t| t.status == status);
    }

    if let Some(priority_str) = args.priority {
        let priority = match priority_str.as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => return Err(format!("Invalid priority: {priority_str}")),
        };
        tickets.retain(|t| t.priority == priority);
    }

    if let Some(assignee) = args.assignee {
        tickets.retain(|t| t.assignee.as_ref() == Some(&assignee));
    }

    if args.open == Some(true) {
        tickets.retain(|t| t.closed_at.is_none());
    }

    if args.closed == Some(true) {
        tickets.retain(|t| t.closed_at.is_some());
    }

    if let Some(tags) = args.tags {
        tickets.retain(|t| tags.iter().any(|tag| t.tags.contains(tag)));
    }

    let ticket_list: Vec<Value> = tickets
        .into_iter()
        .map(|t| {
            json!({
                "id": t.id.to_string(),
                "slug": t.slug,
                "title": t.title,
                "status": format!("{:?}", t.status).to_lowercase(),
                "priority": format!("{:?}", t.priority).to_lowercase(),
                "assignee": t.assignee,
                "tags": t.tags,
                "created_at": t.created_at.to_rfc3339(),
                "closed_at": t.closed_at.map(|dt| dt.to_rfc3339()),
            })
        })
        .collect();

    Ok(json!({
        "tickets": ticket_list,
        "count": ticket_list.len()
    }))
}

/// Handle showing ticket details
pub fn handle_show(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = resolve_ticket_ref(service, &args.ticket)?;
    let ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    Ok(json!({
        "id": ticket.id.to_string(),
        "slug": ticket.slug,
        "title": ticket.title,
        "description": ticket.description,
        "status": format!("{:?}", ticket.status).to_lowercase(),
        "priority": format!("{:?}", ticket.priority).to_lowercase(),
        "assignee": ticket.assignee,
        "tags": ticket.tags,
        "tasks": ticket.tasks.iter().map(|t| json!({
            "id": t.id.to_string(),
            "title": t.title,
            "completed": t.completed,
            "created_at": t.created_at.to_rfc3339(),
            "completed_at": t.completed_at.map(|dt| dt.to_rfc3339()),
        })).collect::<Vec<_>>(),
        "created_at": ticket.created_at.to_rfc3339(),
        "started_at": ticket.started_at.map(|dt| dt.to_rfc3339()),
        "closed_at": ticket.closed_at.map(|dt| dt.to_rfc3339()),
        "metadata": ticket.metadata,
    }))
}

/// Handle editing a ticket
pub fn handle_edit(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        assignee: Option<String>,
        tags: Option<Vec<String>>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = resolve_ticket_ref(service, &args.ticket)?;
    let mut ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    let mut changes = Vec::new();

    if let Some(title) = args.title {
        ticket.title = title;
        changes.push("title");
    }

    if let Some(description) = args.description {
        ticket.description = description;
        changes.push("description");
    }

    if let Some(status_str) = args.status {
        let status = match status_str.as_str() {
            "todo" => Status::Todo,
            "doing" => Status::Doing,
            "done" => Status::Done,
            "blocked" => Status::Blocked,
            "review" => Status::Review,
            _ => return Err(format!("Invalid status: {status_str}")),
        };

        // Handle status transitions
        if ticket.status == Status::Todo && status == Status::Doing && ticket.started_at.is_none() {
            ticket.started_at = Some(chrono::Utc::now());
        }

        ticket.status = status;
        changes.push("status");
    }

    if let Some(priority_str) = args.priority {
        ticket.priority = match priority_str.as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => return Err(format!("Invalid priority: {priority_str}")),
        };
        changes.push("priority");
    }

    if let Some(assignee) = args.assignee {
        ticket.assignee = Some(assignee);
        changes.push("assignee");
    }

    if let Some(tags) = args.tags {
        ticket.tags = tags;
        changes.push("tags");
    }

    if changes.is_empty() {
        return Ok(json!({
            "status": "unchanged",
            "message": "No changes specified"
        }));
    }

    service
        .storage
        .save(&ticket)
        .map_err(|e| format!("Failed to save ticket: {e}"))?;

    Ok(json!({
        "status": "updated",
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "changes": changes
    }))
}

/// Handle closing a ticket
pub fn handle_close(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
        message: Option<String>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = resolve_ticket_ref(service, &args.ticket)?;
    let mut ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    if ticket.closed_at.is_some() {
        return Err("Ticket is already closed".to_string());
    }

    ticket.status = Status::Done;
    ticket.closed_at = Some(chrono::Utc::now());

    if let Some(message) = args.message {
        ticket
            .metadata
            .insert("closing_message".to_string(), Value::String(message));
    }

    service
        .storage
        .save(&ticket)
        .map_err(|e| format!("Failed to save ticket: {e}"))?;

    // Clear active ticket if this was it
    if let Ok(Some(active_id)) = service.storage.get_active() {
        if active_id == ticket_id {
            let _ = service.storage.clear_active();
        }
    }

    Ok(json!({
        "status": "closed",
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "closed_at": ticket.closed_at.unwrap().to_rfc3339()
    }))
}

/// Handle starting work on a ticket
pub fn handle_start(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
        no_worktree: Option<bool>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = resolve_ticket_ref(service, &args.ticket)?;
    let mut ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    // Update ticket status if needed
    if ticket.status == Status::Todo {
        ticket.status = Status::Doing;
        if ticket.started_at.is_none() {
            ticket.started_at = Some(chrono::Utc::now());
        }
        service
            .storage
            .save(&ticket)
            .map_err(|e| format!("Failed to save ticket: {e}"))?;
    }

    // Set as active ticket
    service
        .storage
        .set_active(&ticket_id)
        .map_err(|e| format!("Failed to set active ticket: {e}"))?;

    let mut response = json!({
        "status": "started",
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
    });

    // Handle worktree creation if not disabled
    if !args.no_worktree.unwrap_or(false) {
        // Load config to check worktree settings
        let config = crate::config::Config::load_or_default()
            .map_err(|e| format!("Failed to load config: {e}"))?;

        if config.git.enabled && config.git.worktree_enabled && config.git.worktree_default {
            // Create branch name
            let branch_name = format!("{}{}", config.git.branch_prefix, ticket.slug);

            // Try to create worktree
            match create_git_worktree_mcp(&branch_name, &ticket.slug, &config) {
                Ok(worktree_path) => {
                    response["worktree_created"] = json!(true);
                    response["worktree_path"] = json!(worktree_path);
                    response["branch_name"] = json!(branch_name);
                },
                Err(e) => {
                    response["worktree_note"] = json!(format!("Worktree creation failed: {}", e));
                },
            }
        } else {
            response["worktree_note"] = json!("Worktree creation disabled in config");
        }
    }

    Ok(response)
}

/// Create a Git worktree for the ticket (MCP version)
fn create_git_worktree_mcp(
    branch_name: &str,
    ticket_slug: &str,
    config: &crate::config::Config,
) -> Result<String, String> {
    use std::process::Command;

    // Get project root
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["status"])
        .output()
        .map_err(|e| format!("Failed to run git status: {e}"))?;

    if !status.status.success() {
        return Err("Not in a git repository".to_string());
    }

    // Check if branch already exists
    let branch_exists = Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch_name}"),
        ])
        .output()
        .map_err(|e| format!("Failed to check if branch exists: {e}"))?
        .status
        .success();

    // Create worktree directory path
    let worktree_prefix = config
        .git
        .worktree_prefix
        .replace("{project}", &config.project.name);
    let worktree_dir = format!("{worktree_prefix}{ticket_slug}");
    let worktree_path = project_root
        .parent()
        .unwrap_or(&project_root)
        .join(&worktree_dir);

    // Check if worktree already exists
    if worktree_path.exists() {
        return Err(format!(
            "Worktree directory already exists: {}",
            worktree_path.display()
        ));
    }

    // Create worktree with new or existing branch
    let mut cmd = Command::new("git");
    cmd.arg("worktree").arg("add");

    if !branch_exists {
        cmd.arg("-b");
    }

    cmd.arg(&worktree_path).arg(branch_name);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to create worktree: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create worktree: {stderr}"));
    }

    Ok(worktree_path.to_string_lossy().to_string())
}

/// Handle checking current status
pub fn handle_check(service: &VibeTicketService, _arguments: Value) -> Result<Value, String> {
    let active_ticket = if let Ok(Some(ticket_id)) = service.storage.get_active() {
        if let Ok(ticket) = service.storage.load(&ticket_id) {
            Some(json!({
                "id": ticket.id.to_string(),
                "slug": ticket.slug,
                "title": ticket.title,
                "status": format!("{:?}", ticket.status).to_lowercase(),
                "priority": format!("{:?}", ticket.priority).to_lowercase(),
                "tasks": {
                    "total": ticket.tasks.len(),
                    "completed": ticket.tasks.iter().filter(|t| t.completed).count(),
                }
            }))
        } else {
            None
        }
    } else {
        None
    };

    let ticket_count = service
        .storage
        .load_all()
        .map(|tickets| tickets.len())
        .unwrap_or(0);

    Ok(json!({
        "active_ticket": active_ticket,
        "statistics": {
            "total_tickets": ticket_count,
        }
    }))
}
