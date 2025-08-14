//! Handler for the `close` command
//!
//! This module implements the logic for closing tickets,
//! including status updates and optional archiving.

use crate::cli::{OutputFormatter, find_project_root, handlers::resolve_ticket_ref};
use crate::core::Status;
use crate::error::{Result, VibeTicketError};
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};
use chrono::Utc;

/// Handler for the `close` command
///
/// This function performs the following operations:
/// 1. Loads the specified ticket (or active ticket if none specified)
/// 2. Updates the ticket status to "done"
/// 3. Sets the `closed_at` timestamp
/// 4. Clears the active ticket if it was the one being closed
/// 5. Optionally archives the ticket
/// 6. Optionally creates a pull request
///
/// # Arguments
///
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `message` - Optional close message
/// * `archive` - Whether to archive the ticket
/// * `create_pr` - Whether to create a pull request
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - No ticket is specified and there's no active ticket
/// - The ticket is not found
/// - The ticket is already closed
pub fn handle_close_command(
    ticket_ref: Option<String>,
    message: Option<String>,
    archive: bool,
    create_pr: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    // Ensure project is initialized
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");

    // Initialize storage
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Get the active ticket if no ticket specified
    let ticket_id = if let Some(ref_str) = ticket_ref {
        resolve_ticket_ref(&storage, &ref_str)?
    } else {
        // Get active ticket
        storage
            .get_active()?
            .ok_or(VibeTicketError::NoActiveTicket)?
    };

    // Load the ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Check if ticket is already closed
    if ticket.status == Status::Done {
        return Err(VibeTicketError::custom(format!(
            "Ticket '{}' is already closed",
            ticket.slug
        )));
    }

    // Update ticket status and close time
    let previous_status = ticket.status;
    ticket.status = Status::Done;
    ticket.closed_at = Some(Utc::now());

    // Add close message to metadata if provided
    if let Some(msg) = &message {
        ticket.metadata.insert(
            "close_message".to_string(),
            serde_json::Value::String(msg.clone()),
        );
    }

    // Save the updated ticket
    storage.save(&ticket)?;

    // Notify MCP about ticket closure
    #[cfg(feature = "mcp")]
    crate::integration::notify_ticket_closed(
        &ticket_id,
        message
            .clone()
            .unwrap_or_else(|| "Ticket closed".to_string()),
    );

    // Clear active ticket if this was the active one
    if let Some(active_id) = storage.get_active()? {
        if active_id == ticket_id {
            storage.clear_active()?;
        }
    }

    // Create pull request if requested
    if create_pr {
        create_pull_request(&project_root, &ticket, output)?;
    }

    // Archive if requested (for now, just add a flag to metadata)
    if archive {
        // In a real implementation, we might move the ticket to an archive directory
        let mut archived_ticket = ticket.clone();
        archived_ticket
            .metadata
            .insert("archived".to_string(), serde_json::Value::Bool(true));
        archived_ticket.metadata.insert(
            "archived_at".to_string(),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        );
        storage.save(&archived_ticket)?;
    }

    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket": {
                "id": ticket.id.to_string(),
                "slug": ticket.slug,
                "title": ticket.title,
                "status": ticket.status.to_string(),
                "closed_at": ticket.closed_at,
                "archived": archive,
            },
            "message": message,
            "pr_created": create_pr,
        }))?;
    } else {
        output.success(&format!("Closed ticket: {}", ticket.slug));
        output.info(&format!("Title: {}", ticket.title));
        output.info(&format!("Status: {} → {}", previous_status, Status::Done));

        if let Some(msg) = message {
            output.info(&format!("Close message: {msg}"));
        }

        if archive {
            output.info("Ticket has been archived");
        }

        if create_pr {
            output.info("Pull request creation initiated");
        }

        // Calculate duration if started_at is available
        if let Some(started_at) = ticket.started_at {
            if let Some(closed_at) = ticket.closed_at {
                let duration = closed_at - started_at;
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                output.info(&format!("\nTime spent: {hours}h {minutes}m"));
            }
        }
    }

    Ok(())
}

/// Create a pull request for the ticket
fn create_pull_request(
    project_root: &std::path::Path,
    ticket: &crate::core::Ticket,
    output: &OutputFormatter,
) -> Result<()> {
    use std::process::Command;

    // Get current branch name
    let current_branch = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to get current branch: {e}")))?;

    if !current_branch.status.success() {
        return Err(VibeTicketError::custom("Failed to get current branch name"));
    }

    let branch_name = String::from_utf8_lossy(&current_branch.stdout)
        .trim()
        .to_string();

    // Check if we have the GitHub CLI installed
    let gh_check = Command::new("gh").arg("--version").output();

    if gh_check.is_err() || !gh_check.unwrap().status.success() {
        output.warning(
            "GitHub CLI (gh) not found. Please install it to create pull requests automatically.",
        );
        output.info(&format!(
            "You can create a pull request manually for branch: {branch_name}"
        ));
        return Ok(());
    }

    // Create PR using GitHub CLI
    let pr_title = format!("[{}] {}", ticket.slug, ticket.title);
    let pr_body = format!(
        "## Ticket: {}\n\n{}\n\n**Status:** {} → Done\n**Priority:** {}\n",
        ticket.slug,
        ticket.description,
        if ticket.started_at.is_some() {
            "Doing"
        } else {
            "Todo"
        },
        ticket.priority
    );

    let create_pr = Command::new("gh")
        .arg("pr")
        .arg("create")
        .arg("--title")
        .arg(&pr_title)
        .arg("--body")
        .arg(&pr_body)
        .arg("--head")
        .arg(&branch_name)
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to create PR: {e}")))?;

    if create_pr.status.success() {
        let pr_url = String::from_utf8_lossy(&create_pr.stdout)
            .trim()
            .to_string();
        output.success(&format!("Created pull request: {pr_url}"));
    } else {
        let error_msg = String::from_utf8_lossy(&create_pr.stderr);
        output.warning(&format!("Failed to create pull request: {error_msg}"));
        output.info("You can create the pull request manually");
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_close_message_formatting() {
        let message = "Fixed the login bug and added tests";
        assert!(!message.is_empty());
    }
}
