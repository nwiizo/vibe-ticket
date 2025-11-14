//! Handlers for workflow commands (review, approve, request-changes, handoff)
//!
//! These commands facilitate AI agent collaboration and ticket handoff workflows.

use crate::cli::{OutputFormatter, find_project_root};
use crate::core::{Status, TicketId};
use crate::error::Result;
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};

/// Handler for the `review` command
///
/// Marks a ticket as ready for review by changing its status to Review.
///
/// # Arguments
///
/// * `ticket` - Optional ticket ID or slug (defaults to active ticket)
/// * `notes` - Optional review notes
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - The ticket is not found
/// - File I/O operations fail
pub fn handle_review_command(
    ticket: Option<String>,
    notes: Option<&str>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Resolve ticket ID
    let ticket_id = resolve_ticket(&storage, ticket)?;

    // Load ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Check if already in review
    if ticket.status == Status::Review {
        output.warning(&format!("Ticket '{}' is already in review", ticket.title));
        return Ok(());
    }

    // Update status
    let old_status = ticket.status;
    ticket.status = Status::Review;

    // Add notes to description if provided
    if let Some(review_notes) = notes {
        ticket.description.push_str("\n\n## Review Notes\n\n");
        ticket.description.push_str(review_notes);
    }

    // Save
    storage.save(&ticket)?;

    output.success(&format!(
        "✅ Ticket '{}' moved to review (was: {})",
        ticket.title, old_status
    ));

    if notes.is_some() {
        output.info("Review notes added to ticket description");
    }

    Ok(())
}

/// Handler for the `approve` command
///
/// Approves a ticket and marks it as done.
///
/// # Arguments
///
/// * `ticket` - Optional ticket ID or slug (defaults to active ticket)
/// * `message` - Optional approval message
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - The ticket is not found
/// - File I/O operations fail
pub fn handle_approve_command(
    ticket: Option<String>,
    message: Option<&str>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Resolve ticket ID
    let ticket_id = resolve_ticket(&storage, ticket.clone())?;

    // Load ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Check if already done
    if ticket.status == Status::Done {
        output.warning(&format!("Ticket '{}' is already done", ticket.title));
        return Ok(());
    }

    // Update status
    let old_status = ticket.status;
    ticket.status = Status::Done;
    ticket.closed_at = Some(chrono::Utc::now());

    // Add approval message to description if provided
    if let Some(approval_msg) = message {
        ticket.description.push_str("\n\n## Approval\n\n");
        ticket.description.push_str(approval_msg);
    }

    // Save
    storage.save(&ticket)?;

    // Remove from active tickets
    storage.remove_active(&ticket_id)?;

    output.success(&format!(
        "✅ Ticket '{}' approved and marked as done (was: {})",
        ticket.title, old_status
    ));

    if message.is_some() {
        output.info("Approval message added to ticket description");
    }

    Ok(())
}

/// Handler for the `request-changes` command
///
/// Requests changes on a ticket and moves it back to doing status.
///
/// # Arguments
///
/// * `ticket` - Optional ticket ID or slug (defaults to active ticket)
/// * `changes` - Description of requested changes
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - The ticket is not found
/// - File I/O operations fail
pub fn handle_request_changes_command(
    ticket: Option<String>,
    changes: &str,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Resolve ticket ID
    let ticket_id = resolve_ticket(&storage, ticket)?;

    // Load ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Update status
    let old_status = ticket.status;
    ticket.status = Status::Doing;

    // Add changes to description
    use std::fmt::Write;
    ticket.description.push_str("\n\n## Changes Requested\n\n");
    ticket.description.push_str(changes);
    let _ = write!(
        &mut ticket.description,
        "\n\n*Requested at: {}*",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Save
    storage.save(&ticket)?;

    output.warning(&format!(
        "🔄 Changes requested for ticket '{}' (was: {})",
        ticket.title, old_status
    ));
    output.info("Changes added to ticket description");

    Ok(())
}

/// Handler for the `handoff` command
///
/// Hands off a ticket to another agent or person.
///
/// # Arguments
///
/// * `ticket` - Optional ticket ID or slug (defaults to active ticket)
/// * `assignee` - New assignee name
/// * `notes` - Optional handoff notes
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - The ticket is not found
/// - File I/O operations fail
pub fn handle_handoff_command(
    ticket: Option<String>,
    assignee: &str,
    notes: Option<&str>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Resolve ticket ID
    let ticket_id = resolve_ticket(&storage, ticket)?;

    // Load ticket
    let mut ticket = storage.load(&ticket_id)?;

    let old_assignee = ticket.assignee.clone();
    ticket.assignee = Some(assignee.to_string());

    // Add handoff notes to description if provided
    if let Some(handoff_notes) = notes {
        use std::fmt::Write;
        ticket.description.push_str("\n\n## Handoff Notes\n\n");
        ticket.description.push_str(handoff_notes);
        let _ = write!(
            &mut ticket.description,
            "\n\n*Handed off from {} to {} at {}*",
            old_assignee.as_deref().unwrap_or("unassigned"),
            assignee,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    // Save
    storage.save(&ticket)?;

    output.success(&format!(
        "🤝 Ticket '{}' handed off from {} to {}",
        ticket.title,
        old_assignee.as_deref().unwrap_or("unassigned"),
        assignee
    ));

    if notes.is_some() {
        output.info("Handoff notes added to ticket description");
    }

    Ok(())
}

/// Helper function to resolve ticket ID from optional reference
fn resolve_ticket(storage: &FileStorage, ticket: Option<String>) -> Result<TicketId> {
    use crate::error::VibeTicketError;

    if let Some(ticket_ref) = ticket {
        // Try to parse as ID first
        if let Ok(id) = TicketId::parse_str(&ticket_ref) {
            if storage.load(&id).is_ok() {
                return Ok(id);
            }
        }

        // Try to find by slug
        if let Some(ticket) = storage.find_ticket_by_slug(&ticket_ref)? {
            return Ok(ticket.id);
        }

        Err(VibeTicketError::TicketNotFound { id: ticket_ref })
    } else {
        // Use active ticket
        storage.get_active()?.ok_or(VibeTicketError::NoActiveTicket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Ticket;
    use tempfile::TempDir;

    fn setup_test_storage() -> (TempDir, FileStorage) {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);
        (temp_dir, storage)
    }

    #[test]
    fn test_review_command() {
        let (_temp, storage) = setup_test_storage();
        let mut ticket = Ticket::new("test".to_string(), "Test".to_string());
        ticket.status = Status::Doing;
        storage.save(&ticket).unwrap();

        storage.set_active(&ticket.id).unwrap();

        let output = OutputFormatter::new(false, false);
        handle_review_command(
            None,
            Some("Ready for review"),
            Some(_temp.path().to_str().unwrap()),
            &output,
        )
        .unwrap();

        let updated = storage.load(&ticket.id).unwrap();
        assert_eq!(updated.status, Status::Review);
    }

    #[test]
    fn test_approve_command() {
        let (_temp, storage) = setup_test_storage();
        let mut ticket = Ticket::new("test".to_string(), "Test".to_string());
        ticket.status = Status::Review;
        storage.save(&ticket).unwrap();

        storage.set_active(&ticket.id).unwrap();

        let output = OutputFormatter::new(false, false);
        handle_approve_command(
            None,
            Some("Looks good!"),
            Some(_temp.path().to_str().unwrap()),
            &output,
        )
        .unwrap();

        let updated = storage.load(&ticket.id).unwrap();
        assert_eq!(updated.status, Status::Done);
        assert!(updated.closed_at.is_some());
    }
}
