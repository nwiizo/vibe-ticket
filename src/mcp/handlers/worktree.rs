//! Worktree management MCP tool handlers

use crate::mcp::handlers::schema_helper::json_to_schema;
use crate::mcp::service::VibeTicketService;
use crate::storage::TicketRepository;
use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{Value, json};
use std::borrow::Cow;
use std::sync::Arc;

/// Register all worktree management tools
#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        // List worktrees tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_worktree_list"),
            description: Some(Cow::Borrowed("List Git worktrees")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "all": {
                        "type": "boolean",
                        "description": "Show all worktrees (not just ticket worktrees)"
                    }
                }
            }))),
            title: None,
            output_schema: None,
            icons: None,
            annotations: None,
            meta: None,
        },
        // Remove worktree tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_worktree_remove"),
            description: Some(Cow::Borrowed("Remove a Git worktree")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug for the worktree to remove"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Force removal even if there are uncommitted changes"
                    }
                },
                "required": ["ticket"]
            }))),
            title: None,
            output_schema: None,
            icons: None,
            annotations: None,
            meta: None,
        },
        // Prune worktrees tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_worktree_prune"),
            description: Some(Cow::Borrowed("Remove stale worktree information")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {}
            }))),
            title: None,
            output_schema: None,
            icons: None,
            annotations: None,
            meta: None,
        },
    ]
}

/// Handle listing worktrees
///
/// # Panics
///
/// Panics if worktree object parsing fails
pub fn handle_list(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        all: Option<bool>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    // Execute git worktree list
    let output = std::process::Command::new("git")
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .current_dir(&service.project_root)
        .output()
        .map_err(|e| format!("Failed to execute git worktree list: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Git worktree list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_worktree = json!({});

    for line in output_str.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if !current_worktree.as_object().unwrap().is_empty() {
                worktrees.push(current_worktree);
                current_worktree = json!({});
            }
            current_worktree["path"] = json!(path.trim());
        } else if let Some(head) = line.strip_prefix("HEAD ") {
            current_worktree["head"] = json!(head.trim());
        } else if let Some(branch) = line.strip_prefix("branch ") {
            current_worktree["branch"] = json!(branch.trim());
        } else if line == "detached" {
            current_worktree["detached"] = json!(true);
        }
    }

    if !current_worktree.as_object().unwrap().is_empty() {
        worktrees.push(current_worktree);
    }

    // Filter for ticket worktrees if not showing all
    if !args.all.unwrap_or(false) {
        worktrees.retain(|w| {
            w.get("path")
                .and_then(|p| p.as_str())
                .is_some_and(|path| path.contains("vibeticket"))
        });
    }

    Ok(json!({
        "worktrees": worktrees,
        "count": worktrees.len()
    }))
}

/// Handle removing a worktree
pub fn handle_remove(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
        force: Option<bool>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    // Resolve ticket to get the worktree path
    let ticket_id = crate::mcp::handlers::tickets::resolve_ticket_ref(service, &args.ticket)?;
    let ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    // Construct worktree path pattern
    let worktree_pattern = format!("vibeticket-{}", ticket.slug);

    // First, list worktrees to find the exact path
    let list_output = std::process::Command::new("git")
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .current_dir(&service.project_root)
        .output()
        .map_err(|e| format!("Failed to list worktrees: {e}"))?;

    let output_str = String::from_utf8_lossy(&list_output.stdout);
    let mut worktree_path = None;

    for line in output_str.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if path.contains(&worktree_pattern) {
                worktree_path = Some(path.to_string());
                break;
            }
        }
    }

    let path =
        worktree_path.ok_or_else(|| format!("No worktree found for ticket '{}'", args.ticket))?;

    // Remove the worktree
    let mut remove_cmd = std::process::Command::new("git");
    remove_cmd.arg("worktree").arg("remove");

    if args.force.unwrap_or(false) {
        remove_cmd.arg("--force");
    }

    remove_cmd.arg(&path);
    remove_cmd.current_dir(&service.project_root);

    let output = remove_cmd
        .output()
        .map_err(|e| format!("Failed to remove worktree: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to remove worktree: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(json!({
        "status": "removed",
        "worktree_path": path,
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug
    }))
}

/// Handle pruning stale worktrees
pub fn handle_prune(_service: &VibeTicketService, _arguments: Value) -> Result<Value, String> {
    // Execute git worktree prune
    let output = std::process::Command::new("git")
        .arg("worktree")
        .arg("prune")
        .arg("--verbose")
        .output()
        .map_err(|e| format!("Failed to prune worktrees: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Git worktree prune failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let pruned_count = output_str
        .lines()
        .filter(|l| l.contains("Removing"))
        .count();

    Ok(json!({
        "status": "pruned",
        "pruned_count": pruned_count,
        "message": if pruned_count > 0 {
            format!("Pruned {pruned_count} stale worktree(s)")
        } else {
            "No stale worktrees found".to_string()
        }
    }))
}
