//! Interactive selection handler for fuzzy-finding tickets
//!
//! This module provides an interactive ticket selection interface similar to fzf,
//! allowing users to quickly select tickets using keyboard navigation and filtering.

use crate::cli::output::OutputFormatter;
use crate::cli::utils::find_project_root;
use crate::core::{Priority, Status, Ticket};
use crate::error::{Result, VibeTicketError};
use crate::storage::{FileStorage, TicketRepository};
use dialoguer::{FuzzySelect, MultiSelect, Select, theme::ColorfulTheme};

/// Display format for tickets in the selection list
fn format_ticket_for_selection(ticket: &Ticket) -> String {
    let status_icon = match ticket.status {
        Status::Todo => "○",
        Status::Doing => "◐",
        Status::Done => "●",
        Status::Blocked => "⊘",
        Status::Review => "◎",
    };

    let priority_icon = match ticket.priority {
        Priority::Low => "↓",
        Priority::Medium => "→",
        Priority::High => "↑",
        Priority::Critical => "⚡",
    };

    format!(
        "{} {} [{}] {} ({})",
        status_icon,
        priority_icon,
        ticket.id.short(),
        ticket.title,
        ticket.slug
    )
}

/// Handle interactive select command (single selection)
pub fn handle_interactive_select(
    status: Option<String>,
    priority: Option<String>,
    action: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let tickets = load_and_filter_tickets(&storage, status, priority)?;

    if tickets.is_empty() {
        output.warning("No tickets found matching the criteria");
        return Ok(());
    }

    let items: Vec<String> = tickets.iter().map(format_ticket_for_selection).collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a ticket (type to filter)")
        .items(&items)
        .default(0)
        .interact_opt()
        .map_err(|e| VibeTicketError::custom(format!("Selection cancelled: {e}")))?;

    let Some(index) = selection else {
        output.info("Selection cancelled");
        return Ok(());
    };

    let selected_ticket = &tickets[index];

    // Perform the action on the selected ticket
    match action.as_deref() {
        Some("show") | None => show_ticket(selected_ticket, output),
        Some("start") => start_ticket(selected_ticket, project_dir, output),
        Some("edit") => edit_ticket_prompt(selected_ticket, project_dir, output),
        Some("close") => close_ticket(selected_ticket, project_dir, output),
        Some(other) => Err(VibeTicketError::custom(format!(
            "Unknown action: {other}. Valid actions: show, start, edit, close"
        ))),
    }
}

/// Handle interactive multi-select command
pub fn handle_interactive_multi_select(
    status: Option<String>,
    priority: Option<String>,
    action: String,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let tickets = load_and_filter_tickets(&storage, status, priority)?;

    if tickets.is_empty() {
        output.warning("No tickets found matching the criteria");
        return Ok(());
    }

    let items: Vec<String> = tickets.iter().map(format_ticket_for_selection).collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select tickets (space to toggle, enter to confirm)")
        .items(&items)
        .interact_opt()
        .map_err(|e| VibeTicketError::custom(format!("Selection cancelled: {e}")))?;

    let Some(indices) = selections else {
        output.info("Selection cancelled");
        return Ok(());
    };

    if indices.is_empty() {
        output.warning("No tickets selected");
        return Ok(());
    }

    let selected_tickets: Vec<&Ticket> = indices.iter().map(|&i| &tickets[i]).collect();

    output.info(&format!("Selected {} ticket(s)", selected_tickets.len()));

    // Perform the action on all selected tickets
    match action.as_str() {
        "close" => bulk_close_tickets(&selected_tickets, &storage, output),
        "tag" => bulk_tag_tickets(&selected_tickets, &storage, project_dir, output),
        "status" => bulk_status_tickets(&selected_tickets, &storage, project_dir, output),
        other => Err(VibeTicketError::custom(format!(
            "Unknown bulk action: {other}. Valid actions: close, tag, status"
        ))),
    }
}

/// Handle interactive status change
pub fn handle_interactive_status(
    ticket_ref: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let ticket = if let Some(ref_str) = ticket_ref {
        let ticket_id = crate::cli::handlers::common::resolve_ticket_ref(&storage, &ref_str)?;
        storage.load(&ticket_id)?
    } else {
        // Select a ticket first
        let tickets = storage.load_all()?;
        if tickets.is_empty() {
            return Err(VibeTicketError::custom("No tickets found"));
        }

        let items: Vec<String> = tickets.iter().map(format_ticket_for_selection).collect();

        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a ticket")
            .items(&items)
            .default(0)
            .interact_opt()
            .map_err(|e| VibeTicketError::custom(format!("Selection cancelled: {e}")))?;

        let Some(index) = selection else {
            output.info("Selection cancelled");
            return Ok(());
        };

        tickets[index].clone()
    };

    // Show status options
    let statuses = vec!["Todo", "Doing", "Done", "Blocked", "Review"];
    let current_index = match ticket.status {
        Status::Todo => 0,
        Status::Doing => 1,
        Status::Done => 2,
        Status::Blocked => 3,
        Status::Review => 4,
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Change status for '{}' (current: {:?})",
            ticket.slug, ticket.status
        ))
        .items(&statuses)
        .default(current_index)
        .interact_opt()
        .map_err(|e| VibeTicketError::custom(format!("Selection cancelled: {e}")))?;

    let Some(index) = selection else {
        output.info("Selection cancelled");
        return Ok(());
    };

    let new_status = match index {
        0 => Status::Todo,
        1 => Status::Doing,
        2 => Status::Done,
        3 => Status::Blocked,
        4 => Status::Review,
        _ => unreachable!(),
    };

    if new_status == ticket.status {
        output.info("Status unchanged");
        return Ok(());
    }

    let mut updated_ticket = ticket.clone();
    updated_ticket.status = new_status;

    if new_status == Status::Done {
        updated_ticket.closed_at = Some(chrono::Utc::now());
    } else if new_status == Status::Doing && updated_ticket.started_at.is_none() {
        updated_ticket.started_at = Some(chrono::Utc::now());
    }

    storage.save(&updated_ticket)?;

    output.success(&format!(
        "Changed status of '{}' from {:?} to {:?}",
        ticket.slug, ticket.status, new_status
    ));

    Ok(())
}

/// Handle interactive priority change
pub fn handle_interactive_priority(
    ticket_ref: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let ticket = if let Some(ref_str) = ticket_ref {
        let ticket_id = crate::cli::handlers::common::resolve_ticket_ref(&storage, &ref_str)?;
        storage.load(&ticket_id)?
    } else {
        // Select a ticket first
        let tickets = storage.load_all()?;
        if tickets.is_empty() {
            return Err(VibeTicketError::custom("No tickets found"));
        }

        let items: Vec<String> = tickets.iter().map(format_ticket_for_selection).collect();

        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a ticket")
            .items(&items)
            .default(0)
            .interact_opt()
            .map_err(|e| VibeTicketError::custom(format!("Selection cancelled: {e}")))?;

        let Some(index) = selection else {
            output.info("Selection cancelled");
            return Ok(());
        };

        tickets[index].clone()
    };

    // Show priority options
    let priorities = vec!["Low", "Medium", "High", "Critical"];
    let current_index = match ticket.priority {
        Priority::Low => 0,
        Priority::Medium => 1,
        Priority::High => 2,
        Priority::Critical => 3,
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Change priority for '{}' (current: {:?})",
            ticket.slug, ticket.priority
        ))
        .items(&priorities)
        .default(current_index)
        .interact_opt()
        .map_err(|e| VibeTicketError::custom(format!("Selection cancelled: {e}")))?;

    let Some(index) = selection else {
        output.info("Selection cancelled");
        return Ok(());
    };

    let new_priority = match index {
        0 => Priority::Low,
        1 => Priority::Medium,
        2 => Priority::High,
        3 => Priority::Critical,
        _ => unreachable!(),
    };

    if new_priority == ticket.priority {
        output.info("Priority unchanged");
        return Ok(());
    }

    let mut updated_ticket = ticket.clone();
    updated_ticket.priority = new_priority;
    storage.save(&updated_ticket)?;

    output.success(&format!(
        "Changed priority of '{}' from {:?} to {:?}",
        ticket.slug, ticket.priority, new_priority
    ));

    Ok(())
}

// Helper functions

fn load_and_filter_tickets(
    storage: &FileStorage,
    status: Option<String>,
    priority: Option<String>,
) -> Result<Vec<Ticket>> {
    let mut tickets = storage.load_all()?;

    // Filter by status if specified
    if let Some(status_str) = status {
        let target_status = parse_status(&status_str)?;
        tickets.retain(|t| t.status == target_status);
    }

    // Filter by priority if specified
    if let Some(priority_str) = priority {
        let target_priority = parse_priority(&priority_str)?;
        tickets.retain(|t| t.priority == target_priority);
    }

    // Sort by priority (critical first) then by created date
    tickets.sort_by(|a, b| {
        let priority_order = |p: &Priority| match p {
            Priority::Critical => 0,
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
        };
        priority_order(&a.priority)
            .cmp(&priority_order(&b.priority))
            .then_with(|| b.created_at.cmp(&a.created_at))
    });

    Ok(tickets)
}

fn parse_status(s: &str) -> Result<Status> {
    match s.to_lowercase().as_str() {
        "todo" => Ok(Status::Todo),
        "doing" => Ok(Status::Doing),
        "done" => Ok(Status::Done),
        "blocked" => Ok(Status::Blocked),
        "review" => Ok(Status::Review),
        _ => Err(VibeTicketError::custom(format!("Invalid status: {s}"))),
    }
}

fn parse_priority(p: &str) -> Result<Priority> {
    match p.to_lowercase().as_str() {
        "low" => Ok(Priority::Low),
        "medium" => Ok(Priority::Medium),
        "high" => Ok(Priority::High),
        "critical" => Ok(Priority::Critical),
        _ => Err(VibeTicketError::custom(format!("Invalid priority: {p}"))),
    }
}

fn show_ticket(ticket: &Ticket, output: &OutputFormatter) -> Result<()> {
    output.success(&format!("Ticket: {}", ticket.slug));
    output.info(&format!("ID: {}", ticket.id));
    output.info(&format!("Title: {}", ticket.title));
    output.info(&format!("Status: {:?}", ticket.status));
    output.info(&format!("Priority: {:?}", ticket.priority));
    if !ticket.tags.is_empty() {
        output.info(&format!("Tags: {}", ticket.tags.join(", ")));
    }
    if !ticket.description.is_empty() {
        output.info("");
        output.info("Description:");
        output.info(&ticket.description);
    }
    Ok(())
}

fn start_ticket(
    ticket: &Ticket,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    use crate::cli::handlers::handle_start_command;
    handle_start_command(
        ticket.slug.clone(),
        true, // create_branch
        None, // branch_name
        true, // create_worktree
        project_dir.map(String::from),
        output,
    )
}

fn edit_ticket_prompt(
    ticket: &Ticket,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let options = vec![
        "Status",
        "Priority",
        "Title",
        "Description",
        "Tags",
        "Cancel",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Edit '{}' - What would you like to change?",
            ticket.slug
        ))
        .items(&options)
        .default(0)
        .interact_opt()
        .map_err(|e| VibeTicketError::custom(format!("Selection cancelled: {e}")))?;

    let Some(index) = selection else {
        output.info("Edit cancelled");
        return Ok(());
    };

    match index {
        0 => handle_interactive_status(Some(ticket.slug.clone()), project_dir, output),
        1 => handle_interactive_priority(Some(ticket.slug.clone()), project_dir, output),
        2..=4 => {
            output.info(&format!(
                "To edit {}, use: vibe-ticket edit {}",
                options[index].to_lowercase(),
                ticket.slug
            ));
            Ok(())
        },
        5 => {
            output.info("Edit cancelled");
            Ok(())
        },
        _ => unreachable!(),
    }
}

fn close_ticket(
    ticket: &Ticket,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    use crate::cli::handlers::handle_close_command;
    handle_close_command(
        Some(ticket.slug.clone()),
        None,  // message
        false, // archive
        false, // create_pr
        project_dir,
        output,
    )
}

fn bulk_close_tickets(
    tickets: &[&Ticket],
    storage: &FileStorage,
    output: &OutputFormatter,
) -> Result<()> {
    let mut closed = 0;
    for ticket in tickets {
        if ticket.status != Status::Done {
            let mut updated = (*ticket).clone();
            updated.status = Status::Done;
            updated.closed_at = Some(chrono::Utc::now());
            storage.save(&updated)?;
            closed += 1;
        }
    }
    output.success(&format!("Closed {closed} ticket(s)"));
    Ok(())
}

fn bulk_tag_tickets(
    tickets: &[&Ticket],
    storage: &FileStorage,
    _project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let tag_input: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter tags to add (comma-separated)")
        .interact_text()
        .map_err(|e| VibeTicketError::custom(format!("Input cancelled: {e}")))?;

    let tags: Vec<String> = tag_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tags.is_empty() {
        output.warning("No tags specified");
        return Ok(());
    }

    let mut updated_count = 0;
    for ticket in tickets {
        let mut updated = (*ticket).clone();
        for tag in &tags {
            if !updated.tags.contains(tag) {
                updated.tags.push(tag.clone());
            }
        }
        if updated.tags.len() != ticket.tags.len() {
            storage.save(&updated)?;
            updated_count += 1;
        }
    }

    output.success(&format!(
        "Added tags '{}' to {updated_count} ticket(s)",
        tags.join(", ")
    ));
    Ok(())
}

fn bulk_status_tickets(
    tickets: &[&Ticket],
    storage: &FileStorage,
    _project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let statuses = vec!["Todo", "Doing", "Done", "Blocked", "Review"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select new status for all selected tickets")
        .items(&statuses)
        .default(0)
        .interact_opt()
        .map_err(|e| VibeTicketError::custom(format!("Selection cancelled: {e}")))?;

    let Some(index) = selection else {
        output.info("Selection cancelled");
        return Ok(());
    };

    let new_status = match index {
        0 => Status::Todo,
        1 => Status::Doing,
        2 => Status::Done,
        3 => Status::Blocked,
        4 => Status::Review,
        _ => unreachable!(),
    };

    let mut updated_count = 0;
    for ticket in tickets {
        if ticket.status != new_status {
            let mut updated = (*ticket).clone();
            updated.status = new_status;

            if new_status == Status::Done {
                updated.closed_at = Some(chrono::Utc::now());
            } else if new_status == Status::Doing && updated.started_at.is_none() {
                updated.started_at = Some(chrono::Utc::now());
            }

            storage.save(&updated)?;
            updated_count += 1;
        }
    }

    output.success(&format!(
        "Changed status to {new_status:?} for {updated_count} ticket(s)"
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_ticket_for_selection() {
        use crate::core::TicketId;
        use chrono::Utc;
        use std::collections::HashMap;

        let ticket = Ticket {
            id: TicketId::new(),
            slug: "test-ticket".to_string(),
            title: "Test Ticket".to_string(),
            description: String::new(),
            priority: Priority::High,
            status: Status::Doing,
            tags: vec![],
            created_at: Utc::now(),
            started_at: None,
            closed_at: None,
            assignee: None,
            tasks: vec![],
            metadata: HashMap::new(),
        };

        let formatted = format_ticket_for_selection(&ticket);
        assert!(formatted.contains("Test Ticket"));
        assert!(formatted.contains("test-ticket"));
        assert!(formatted.contains("◐")); // Doing icon
        assert!(formatted.contains("↑")); // High priority icon
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(parse_status("todo").unwrap(), Status::Todo);
        assert_eq!(parse_status("DOING").unwrap(), Status::Doing);
        assert!(parse_status("invalid").is_err());
    }

    #[test]
    fn test_parse_priority() {
        assert_eq!(parse_priority("low").unwrap(), Priority::Low);
        assert_eq!(parse_priority("CRITICAL").unwrap(), Priority::Critical);
        assert!(parse_priority("invalid").is_err());
    }
}
