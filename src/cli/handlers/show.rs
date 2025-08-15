//! Handler for the `show` command
//!
//! This module implements the logic for displaying detailed information
//! about a specific ticket, including tasks and history.

use crate::cli::handlers::common::resolve_ticket_ref;
use crate::cli::{OutputFormatter, find_project_root};
use crate::error::Result;
use crate::storage::{FileStorage, TicketRepository};
use chrono::{DateTime, Local, Utc};

/// Handler for the `show` command
///
/// This function displays comprehensive information about a ticket:
/// 1. Basic ticket information (ID, slug, title, etc.)
/// 2. Full description
/// 3. Status and priority
/// 4. Timestamps (created, started, closed)
/// 5. Tags
/// 6. Tasks (if requested)
/// 7. History (if available and requested)
/// 8. Metadata
///
/// # Arguments
///
/// * `ticket_ref` - Ticket ID or slug to display
/// * `show_tasks` - Whether to show task details
/// * `show_history` - Whether to show ticket history
/// * `markdown` - Whether to format output as markdown
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - The ticket is not found
pub fn handle_show_command(
    ticket_ref: &str,
    show_tasks: bool,
    show_history: bool,
    markdown: bool,
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
    let ticket = storage.load(&ticket_id)?;

    // Output results
    if output.is_json() {
        let mut json_output = serde_json::json!({
            "ticket": {
                "id": ticket.id.to_string(),
                "slug": ticket.slug,
                "title": ticket.title,
                "description": ticket.description,
                "status": ticket.status.to_string(),
                "priority": ticket.priority.to_string(),
                "tags": ticket.tags,
                "assignee": ticket.assignee,
                "created_at": ticket.created_at,
                "started_at": ticket.started_at,
                "closed_at": ticket.closed_at,
                "metadata": ticket.metadata,
            }
        });

        if show_tasks {
            json_output["tasks"] = serde_json::json!(ticket.tasks);
        }

        output.print_json(&json_output)?;
    } else if markdown {
        output_markdown(&ticket, show_tasks, output);
    } else {
        output_plain(&ticket, show_tasks, show_history, output);
    }

    Ok(())
}

/// Output ticket information in plain text format
fn output_plain(
    ticket: &crate::core::Ticket,
    show_tasks: bool,
    show_history: bool,
    output: &OutputFormatter,
) {
    // Header
    output.success(&format!("Ticket: {}", ticket.slug));
    output.info(&format!("ID: {}", ticket.id));
    output.info(&format!("Title: {}", ticket.title));
    output.info(&format!("Status: {}", ticket.status));
    output.info(&format!("Priority: {}", ticket.priority));

    // Assignee
    if let Some(assignee) = &ticket.assignee {
        output.info(&format!("Assignee: {assignee}"));
    }

    // Tags
    if !ticket.tags.is_empty() {
        output.info(&format!("Tags: {}", ticket.tags.join(", ")));
    }

    // Timestamps
    output.info("");
    output.info("Timeline:");
    output.info(&format!(
        "  Created: {}",
        format_datetime(ticket.created_at)
    ));

    if let Some(started_at) = ticket.started_at {
        output.info(&format!("  Started: {}", format_datetime(started_at)));

        // Calculate time spent
        let end_time = ticket.closed_at.unwrap_or_else(Utc::now);
        let duration = end_time - started_at;
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        output.info(&format!("  Time spent: {hours}h {minutes}m"));
    }

    if let Some(closed_at) = ticket.closed_at {
        output.info(&format!("  Closed: {}", format_datetime(closed_at)));
    }

    // Description
    output.info("");
    output.info("Description:");
    for line in ticket.description.lines() {
        output.info(&format!("  {line}"));
    }

    // Tasks
    if show_tasks && !ticket.tasks.is_empty() {
        output.info("");
        output.info("Tasks:");
        let completed = ticket.tasks.iter().filter(|t| t.completed).count();
        output.info(&format!("  Progress: {}/{}", completed, ticket.tasks.len()));
        output.info("");

        for task in &ticket.tasks {
            let checkbox = if task.completed { "✓" } else { "○" };
            output.info(&format!("  {} {}", checkbox, task.title));
            if task.completed {
                if let Some(completed_at) = task.completed_at {
                    output.info(&format!(
                        "      Completed: {}",
                        format_datetime(completed_at)
                    ));
                }
            }
        }
    }

    // Metadata
    if !ticket.metadata.is_empty() {
        output.info("");
        output.info("Metadata:");
        // Show close message if present
        if let Some(msg) = ticket
            .metadata
            .get("close_message")
            .and_then(|v| v.as_str())
        {
            output.info(&format!("  Close message: {msg}"));
        }

        // Show archived status if present
        if ticket
            .metadata
            .get("archived")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            output.info("  Status: Archived");
            if let Some(date_str) = ticket.metadata.get("archived_at").and_then(|v| v.as_str()) {
                output.info(&format!("  Archived at: {date_str}"));
            }
        }
    }

    // History (placeholder for future implementation)
    if show_history {
        output.info("");
        output.info("History:");
        output.info("  (History tracking not yet implemented)");
    }
}

/// Output ticket information in markdown format
fn output_markdown(ticket: &crate::core::Ticket, show_tasks: bool, _output: &OutputFormatter) {
    // Title and metadata
    println!("# {}", ticket.title);
    println!();
    println!("**ID**: `{}`", ticket.id);
    println!("**Slug**: `{}`", ticket.slug);
    println!("**Status**: {}", ticket.status);
    println!("**Priority**: {}", ticket.priority);

    if let Some(assignee) = &ticket.assignee {
        println!("**Assignee**: {assignee}");
    }

    if !ticket.tags.is_empty() {
        println!(
            "**Tags**: {}",
            ticket
                .tags
                .iter()
                .map(|t| format!("`{t}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    println!();

    // Timeline
    println!("## Timeline");
    println!();
    println!("- **Created**: {}", format_datetime(ticket.created_at));

    if let Some(started_at) = ticket.started_at {
        println!("- **Started**: {}", format_datetime(started_at));

        let end_time = ticket.closed_at.unwrap_or_else(Utc::now);
        let duration = end_time - started_at;
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        println!("- **Time spent**: {hours}h {minutes}m");
    }

    if let Some(closed_at) = ticket.closed_at {
        println!("- **Closed**: {}", format_datetime(closed_at));
    }

    println!();

    // Description
    println!("## Description");
    println!();
    println!("{}", ticket.description);
    println!();

    // Tasks
    if show_tasks && !ticket.tasks.is_empty() {
        println!("## Tasks");
        println!();
        let completed = ticket.tasks.iter().filter(|t| t.completed).count();
        println!("Progress: {}/{}", completed, ticket.tasks.len());
        println!();

        for task in &ticket.tasks {
            let checkbox = if task.completed { "[x]" } else { "[ ]" };
            println!("- {} {}", checkbox, task.title);
        }
        println!();
    }
}

/// Format datetime for display
fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_datetime() {
        let dt = Utc::now();
        let formatted = format_datetime(dt);
        assert!(!formatted.is_empty());
        assert!(formatted.contains('-'));
        assert!(formatted.contains(':'));
    }
}
