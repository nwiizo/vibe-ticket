//! Handler for the `archive` command
//!
//! This module implements the logic for archiving and unarchiving tickets.

use crate::cli::handlers::common::resolve_ticket_ref;
use crate::cli::{OutputFormatter, find_project_root};
use crate::error::{Result, VibeTicketError};
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};

/// Handler for the `archive` command
///
/// Archives or unarchives a ticket:
/// 1. Archives completed tickets to clean up the active list
/// 2. Adds metadata to mark the ticket as archived
/// 3. Can unarchive tickets to restore them to the active list
///
/// # Arguments
///
/// * `ticket_ref` - Ticket ID or slug to archive/unarchive
/// * `unarchive` - Whether to unarchive instead of archive
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - The ticket is not found
/// - Trying to archive an active ticket
pub fn handle_archive_command(
    ticket_ref: &str,
    unarchive: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    // Ensure project is initialized
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");

    // Initialize storage
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Resolve ticket ID
    let ticket_id = resolve_ticket_ref(&storage, ticket_ref)?;

    // Load the ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Check if already in desired state
    let is_archived = ticket
        .metadata
        .get("archived")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if unarchive {
        // Unarchiving
        if !is_archived {
            return Err(VibeTicketError::custom("Ticket is not archived"));
        }

        // Remove archive metadata
        ticket.metadata.remove("archived");
        ticket.metadata.remove("archived_at");

        // Save the updated ticket
        storage.save(&ticket)?;

        // Output results
        if output.is_json() {
            output.print_json(&serde_json::json!({
                "status": "success",
                "action": "unarchived",
                "ticket": {
                    "id": ticket.id.to_string(),
                    "slug": ticket.slug,
                    "title": ticket.title,
                    "status": ticket.status.to_string(),
                }
            }))?;
        } else {
            output.success(&format!("Unarchived ticket: {}", ticket.slug));
            output.info(&format!("Title: {}", ticket.title));
            output.info(&format!("Status: {}", ticket.status));
        }
    } else {
        // Archiving
        if is_archived {
            return Err(VibeTicketError::custom("Ticket is already archived"));
        }

        // Check if ticket is the active ticket
        if let Ok(Some(active_id)) = storage.get_active() {
            if active_id == ticket.id {
                return Err(VibeTicketError::custom(
                    "Cannot archive the active ticket. Close or switch to another ticket first.",
                ));
            }
        }

        // Check if ticket is in progress
        if ticket.status == crate::core::Status::Doing
            || ticket.status == crate::core::Status::Review
        {
            output.warning(&format!(
                "Warning: Archiving a ticket with status '{}'. Consider closing it first.",
                ticket.status
            ));
        }

        // Add archive metadata
        ticket
            .metadata
            .insert("archived".to_string(), serde_json::Value::Bool(true));
        ticket.metadata.insert(
            "archived_at".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );

        // Save the updated ticket
        storage.save(&ticket)?;

        // Output results
        if output.is_json() {
            output.print_json(&serde_json::json!({
                "status": "success",
                "action": "archived",
                "ticket": {
                    "id": ticket.id.to_string(),
                    "slug": ticket.slug,
                    "title": ticket.title,
                    "status": ticket.status.to_string(),
                }
            }))?;
        } else {
            output.success(&format!("Archived ticket: {}", ticket.slug));
            output.info(&format!("Title: {}", ticket.title));
            output.info(&format!("Status: {}", ticket.status));
            output.info("\nThe ticket has been archived and will not appear in regular listings.");
            output.info("Use --archived flag with list command to see archived tickets.");
            output.info("Use --unarchive flag to restore this ticket.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archive_metadata() {
        use serde_json::json;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("archived".to_string(), json!(true));

        let is_archived = metadata
            .get("archived")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        assert!(is_archived);
    }
}
