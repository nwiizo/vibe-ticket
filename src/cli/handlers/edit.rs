//! Handler for the `edit` command
//!
//! This module implements the logic for editing ticket properties,
//! including title, description, priority, status, and tags.

use crate::cli::handlers::common::resolve_ticket_ref;
use crate::cli::{OutputFormatter, find_project_root};
use crate::core::{Priority, Status};
use crate::error::{Result, VibeTicketError};
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};

/// Parameters for updating a ticket
struct UpdateParams {
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    status: Option<String>,
    add_tags: Option<String>,
    remove_tags: Option<String>,
}

/// Handler for the `edit` command
///
/// This function allows editing various properties of a ticket:
/// 1. Title
/// 2. Description
/// 3. Priority
/// 4. Status
/// 5. Tags (add/remove)
/// 6. Opens in editor if requested
///
/// # Arguments
///
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `title` - New title for the ticket
/// * `description` - New description for the ticket
/// * `priority` - New priority for the ticket
/// * `status` - New status for the ticket
/// * `add_tags` - Tags to add (comma-separated)
/// * `remove_tags` - Tags to remove (comma-separated)
/// * `editor` - Whether to open in the default editor
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - No ticket is specified and there's no active ticket
/// - The ticket is not found
/// - Invalid priority or status values are provided
#[allow(clippy::too_many_arguments)]
pub fn handle_edit_command(
    ticket_ref: Option<String>,
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    status: Option<String>,
    add_tags: Option<String>,
    remove_tags: Option<String>,
    editor: bool,
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

    // Track what was changed
    let mut changes = Vec::new();

    // Open in editor if requested
    if editor {
        edit_in_editor(&mut ticket, &storage, output)?;
        return Ok(());
    }

    // Apply updates
    let update_params = UpdateParams {
        title,
        description,
        priority,
        status,
        add_tags,
        remove_tags,
    };
    apply_ticket_updates(&mut ticket, &mut changes, update_params)?;

    // Check if any changes were made
    if changes.is_empty() {
        output.warning("No changes specified");
        return Ok(());
    }

    // Save the updated ticket
    storage.save(&ticket)?;

    // Notify MCP about ticket update
    #[cfg(feature = "mcp")]
    crate::integration::notify_ticket_updated(&ticket);

    // Output results
    format_edit_output(output, &ticket, &changes)?;

    Ok(())
}

/// Apply updates to a ticket
fn apply_ticket_updates(
    ticket: &mut crate::core::Ticket,
    changes: &mut Vec<String>,
    params: UpdateParams,
) -> Result<()> {
    // Update title if provided
    if let Some(new_title) = params.title {
        let old_title = ticket.title.clone();
        ticket.title.clone_from(&new_title);
        changes.push(format!("Title: {old_title} → {new_title}"));
    }

    // Update description if provided
    if let Some(new_description) = params.description {
        ticket.description = new_description;
        changes.push("Description updated".to_string());
    }

    // Update priority if provided
    if let Some(priority_str) = params.priority {
        let new_priority = Priority::try_from(priority_str.as_str()).map_err(|_| {
            VibeTicketError::InvalidPriority {
                priority: priority_str,
            }
        })?;
        let old_priority = ticket.priority;
        ticket.priority = new_priority;
        changes.push(format!("Priority: {old_priority} → {new_priority}"));
    }

    // Update status if provided
    if let Some(status_str) = params.status {
        let new_status = Status::try_from(status_str.as_str())
            .map_err(|_| VibeTicketError::InvalidStatus { status: status_str })?;
        let old_status = ticket.status;
        ticket.status = new_status;
        changes.push(format!("Status: {old_status} → {new_status}"));

        // Update timestamps based on status changes
        match (old_status, new_status) {
            (Status::Todo, Status::Doing) => {
                ticket.started_at = Some(chrono::Utc::now());
            },
            (_, Status::Done) if old_status != Status::Done => {
                ticket.closed_at = Some(chrono::Utc::now());
            },
            _ => {},
        }
    }

    // Handle tags
    handle_tag_updates(ticket, changes, params.add_tags, params.remove_tags);

    Ok(())
}

/// Handle tag updates
fn handle_tag_updates(
    ticket: &mut crate::core::Ticket,
    changes: &mut Vec<String>,
    add_tags: Option<String>,
    remove_tags: Option<String>,
) {
    // Add tags if provided
    if let Some(tags_str) = add_tags {
        let new_tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for tag in new_tags {
            if !ticket.tags.contains(&tag) {
                ticket.tags.push(tag);
            }
        }
        changes.push("Tags added".to_string());
    }

    // Remove tags if provided
    if let Some(tags_str) = remove_tags {
        let tags_to_remove: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        ticket.tags.retain(|tag| !tags_to_remove.contains(tag));
        changes.push("Tags removed".to_string());
    }
}

/// Format and display edit command output
fn format_edit_output(
    output: &OutputFormatter,
    ticket: &crate::core::Ticket,
    changes: &[String],
) -> Result<()> {
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket": {
                "id": ticket.id.to_string(),
                "slug": ticket.slug,
                "title": ticket.title,
                "description": ticket.description,
                "status": ticket.status.to_string(),
                "priority": ticket.priority.to_string(),
                "tags": ticket.tags,
            },
            "changes": changes,
        }))?;
    } else {
        output.success(&format!("Updated ticket: {}", ticket.slug));
        for change in changes {
            output.info(&format!("  • {change}"));
        }

        // Show current state
        output.info("");
        output.info("Current state:");
        output.info(&format!("  Title: {}", ticket.title));
        output.info(&format!("  Status: {}", ticket.status));
        output.info(&format!("  Priority: {}", ticket.priority));
        if !ticket.tags.is_empty() {
            output.info(&format!("  Tags: {}", ticket.tags.join(", ")));
        }
    }
    Ok(())
}

/// Edit ticket in the default editor
fn edit_in_editor(
    ticket: &mut crate::core::Ticket,
    storage: &FileStorage,
    output: &OutputFormatter,
) -> Result<()> {
    use std::io::Write as IoWrite;
    use std::process::Command;

    // Create a temporary file with the ticket content
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("vibe-ticket-{}.yaml", ticket.id));

    // Serialize ticket to YAML
    let yaml_content = serde_yaml::to_string(&ticket)
        .map_err(|e| VibeTicketError::custom(format!("Failed to serialize YAML: {e}")))?;

    // Write to temporary file
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| VibeTicketError::io_error("create", &temp_file, e))?;
    file.write_all(yaml_content.as_bytes())
        .map_err(|e| VibeTicketError::io_error("write", &temp_file, e))?;

    // Get editor from environment
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    // Open editor
    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()
        .map_err(|e| VibeTicketError::custom(format!("Failed to launch editor: {e}")))?;

    if !status.success() {
        return Err(VibeTicketError::custom("Editor exited with error"));
    }

    // Read the edited content
    let edited_content = std::fs::read_to_string(&temp_file)
        .map_err(|e| VibeTicketError::io_error("read", &temp_file, e))?;

    // Parse the edited ticket
    let edited_ticket: crate::core::Ticket = serde_yaml::from_str(&edited_content)
        .map_err(|e| VibeTicketError::deserialization_error("YAML ticket", e))?;

    // Update the original ticket
    *ticket = edited_ticket;

    // Save the updated ticket
    storage.save(ticket)?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    output.success(&format!("Updated ticket: {}", ticket.slug));

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_tag_parsing() {
        let tags_str = "bug, ui, urgent";
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(tags, vec!["bug", "ui", "urgent"]);
    }
}
