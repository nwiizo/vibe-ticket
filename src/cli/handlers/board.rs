//! Handler for the `board` command
//!
//! This module implements a kanban-style board view for tickets,
//! displaying them in columns organized by status.

use crate::cli::{OutputFormatter, find_project_root};
use crate::core::{Status, Ticket};
use crate::error::Result;
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};
use std::collections::HashMap;

/// Handler for the `board` command
///
/// Displays tickets in a kanban-style board with columns for each status.
///
/// # Arguments
///
/// * `assignee` - Optional assignee filter
/// * `active_only` - Show only active tickets
/// * `compact` - Use compact view with less spacing
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - File I/O operations fail
pub fn handle_board_command(
    assignee: Option<&str>,
    active_only: bool,
    compact: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Load tickets
    let mut tickets = storage.load_all()?;

    // Filter by assignee if specified
    if let Some(assignee_filter) = assignee {
        tickets.retain(|t| {
            t.assignee
                .as_ref()
                .is_some_and(|a| a.contains(assignee_filter))
        });
    }

    // Filter by active status if specified
    if active_only {
        let active_ids = storage.get_all_active()?;
        tickets.retain(|t| active_ids.contains(&t.id));
    }

    // Group tickets by status
    let mut by_status: HashMap<Status, Vec<Ticket>> = HashMap::new();
    for ticket in tickets {
        by_status.entry(ticket.status).or_default().push(ticket);
    }

    // Display board
    if output.is_json() {
        output_json(&by_status, output)?;
    } else {
        output_text(&by_status, compact, output);
    }

    Ok(())
}

/// Output board as JSON
fn output_json(by_status: &HashMap<Status, Vec<Ticket>>, output: &OutputFormatter) -> Result<()> {
    let mut board = HashMap::new();

    for (status, tickets) in by_status {
        let ticket_list: Vec<_> = tickets
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id.to_string(),
                    "slug": t.slug,
                    "title": t.title,
                    "priority": t.priority.to_string(),
                    "assignee": t.assignee,
                    "tasks": {
                        "total": t.tasks.len(),
                        "completed": t.tasks.iter().filter(|task| task.completed).count(),
                    },
                })
            })
            .collect();

        board.insert(status.to_string(), ticket_list);
    }

    output.print_json(&board)
}

/// Output board as text
fn output_text(by_status: &HashMap<Status, Vec<Ticket>>, compact: bool, output: &OutputFormatter) {
    let spacing = if compact { "" } else { "\n" };

    // Define column order
    let columns = [
        Status::Todo,
        Status::Doing,
        Status::Review,
        Status::Blocked,
        Status::Done,
    ];

    // Calculate column width based on content
    let col_width = 30;

    // Print header
    output.info(&format!(
        "{spacing}╔═══════════════════════════════════════════════════════════════════════════════════╗{spacing}"
    ));
    output.info(&format!("║{:^83}║", "KANBAN BOARD"));
    output.info(&format!(
        "╠══════════════════╦══════════════════╦══════════════════╦══════════════════╦════════════════╣{spacing}"
    ));

    // Print column headers
    let header = format!(
        "║ {:<14} ║ {:<14} ║ {:<14} ║ {:<14} ║ {:<14} ║",
        format_status_header(Status::Todo),
        format_status_header(Status::Doing),
        format_status_header(Status::Review),
        format_status_header(Status::Blocked),
        format_status_header(Status::Done),
    );
    output.info(&header);
    output.info(&format!(
        "╠══════════════════╬══════════════════╬══════════════════╬══════════════════╬════════════════╣{spacing}"
    ));

    // Find max number of tickets in any column
    let max_tickets = columns
        .iter()
        .map(|status| by_status.get(status).map_or(0, |v| v.len()))
        .max()
        .unwrap_or(0);

    // Print rows
    for i in 0..max_tickets {
        let mut row_parts = Vec::new();

        for status in &columns {
            let tickets = by_status.get(status);
            if let Some(tickets) = tickets {
                if let Some(ticket) = tickets.get(i) {
                    row_parts.push(format_ticket_cell(ticket, col_width));
                } else {
                    row_parts.push(format!("{:width$}", "", width = col_width - 2));
                }
            } else {
                row_parts.push(format!("{:width$}", "", width = col_width - 2));
            }
        }

        output.info(&format!(
            "║ {:<14} ║ {:<14} ║ {:<14} ║ {:<14} ║ {:<14} ║",
            row_parts[0], row_parts[1], row_parts[2], row_parts[3], row_parts[4]
        ));

        if !compact && i < max_tickets - 1 {
            output.info(&format!(
                "║{:16}║{:16}║{:16}║{:16}║{:16}║",
                "", "", "", "", ""
            ));
        }
    }

    // Print footer
    output.info(&format!(
        "╚══════════════════╩══════════════════╩══════════════════╩══════════════════╩════════════════╝{spacing}"
    ));

    // Print summary
    output.info(spacing);
    output.info("Summary:");
    for status in &columns {
        let count = by_status.get(status).map_or(0, |v| v.len());
        let emoji = match status {
            Status::Todo => "📋",
            Status::Doing => "🔄",
            Status::Review => "👀",
            Status::Blocked => "🚫",
            Status::Done => "✅",
        };
        output.info(&format!("  {emoji} {status}: {count}"));
    }
}

/// Format status header with emoji
fn format_status_header(status: Status) -> String {
    let emoji = match status {
        Status::Todo => "📋",
        Status::Doing => "🔄",
        Status::Review => "👀",
        Status::Blocked => "🚫",
        Status::Done => "✅",
    };
    format!("{emoji} {status}")
}

/// Format a ticket for display in a cell
fn format_ticket_cell(ticket: &Ticket, _width: usize) -> String {
    // Truncate title if too long
    let title = if ticket.title.len() > 12 {
        format!("{}...", &ticket.title[..9])
    } else {
        ticket.title.clone()
    };

    // Add priority indicator
    let priority_indicator = match ticket.priority {
        crate::core::Priority::Critical => "🔴",
        crate::core::Priority::High => "🟠",
        crate::core::Priority::Medium => "🟡",
        crate::core::Priority::Low => "🟢",
    };

    format!("{priority_indicator} {title}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_ticket_cell() {
        let ticket = crate::core::Ticket::new("test".to_string(), "Test Title".to_string());
        let cell = format_ticket_cell(&ticket, 20);
        assert!(!cell.is_empty());
    }

    #[test]
    fn test_format_status_header() {
        let header = format_status_header(Status::Todo);
        assert!(header.contains("📋"));
        assert!(header.contains("Todo"));
    }
}
