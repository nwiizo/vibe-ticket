//! Intent-focused work-on command handler
//!
//! Helps users start working on tickets with a focus on getting
//! into the flow quickly rather than remembering command syntax.

use crate::cli::output::OutputFormatter;
use crate::cli::utils;
use crate::core::{Status, Ticket, TicketId};
use crate::error::{Result, VibeTicketError};
use crate::storage::{FileStorage, TicketRepository};
use dialoguer::{Select, theme::ColorfulTheme};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Handle the intent-focused work-on command
///
/// This command helps users:
/// 1. Select a ticket to work on (if not specified)
/// 2. Start work on the ticket
/// 3. Show relevant information
/// 4. Set up the work environment
pub fn handle_work_on_command(
    ticket: Option<String>,
    no_worktree: bool,
    project_dir: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project_dir {
        env::set_current_dir(project_path)?;
    }

    let current_dir = env::current_dir()?;
    let project_root = utils::find_project_root(current_dir.to_str())?;
    let tickets_dir = project_root.join(".vibe-ticket");

    if !tickets_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let storage = FileStorage::new(tickets_dir.clone());

    // Get ticket to work on
    let ticket_id_str = if let Some(t) = ticket {
        t
    } else {
        // Interactive selection
        select_ticket_to_work_on(&storage, formatter)?
    };

    // Parse ticket ID
    let ticket_id = TicketId::parse_str(&ticket_id_str)
        .map_err(|_| VibeTicketError::Custom(format!("Invalid ticket ID: {}", ticket_id_str)))?;

    // Load the ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Check current status
    match ticket.status {
        Status::Done => {
            formatter.warning(&format!(
                "⚠️  Ticket '{}' is already marked as done.",
                ticket.title
            ));
            formatter.info("Consider reopening it or creating a new ticket.");
            return Ok(());
        },
        Status::Blocked => {
            formatter.warning(&format!("⚠️  Ticket '{}' is blocked.", ticket.title));
            formatter.info("Resolve the blocking issue before starting work.");
        },
        _ => {},
    }

    // Update ticket status to "doing"
    if ticket.status != Status::Doing {
        ticket.status = Status::Doing;
        ticket.started_at = Some(chrono::Utc::now());
        storage.save(&ticket)?;
    }

    // Set as active ticket
    let active_ticket_path = tickets_dir.join("active_ticket");
    fs::write(&active_ticket_path, &ticket.id.to_string())?;

    // Create worktree if needed
    if !no_worktree && should_create_worktree(&project_root)? {
        create_worktree_for_ticket(&ticket, &project_root, formatter)?;
    }

    // Display ticket information
    display_work_context(&ticket, formatter)?;

    // Show next steps
    formatter.info("\n📋 Next steps:");
    formatter.info("  • Review the ticket description and requirements");
    formatter.info("  • Check existing tasks with: vibe-ticket task list");
    formatter.info("  • Add new tasks with: vibe-ticket task add <description>");
    formatter.info("  • When done, use: vibe-ticket finish");

    Ok(())
}

/// Select a ticket to work on interactively
fn select_ticket_to_work_on(storage: &FileStorage, formatter: &OutputFormatter) -> Result<String> {
    let tickets = storage.load_all()?;

    // Filter to workable tickets
    let mut workable_tickets: Vec<Ticket> = tickets
        .into_iter()
        .filter(|t| matches!(t.status, Status::Todo | Status::Doing | Status::Blocked))
        .collect();

    if workable_tickets.is_empty() {
        return Err(VibeTicketError::Custom(
            "No open tickets available to work on. Create a new ticket first.".to_string(),
        ));
    }

    // Sort by priority and status
    workable_tickets.sort_by(|a, b| {
        // Doing tickets first, then by priority
        match (&a.status, &b.status) {
            (Status::Doing, Status::Doing) => b.priority.cmp(&a.priority),
            (Status::Doing, _) => std::cmp::Ordering::Less,
            (_, Status::Doing) => std::cmp::Ordering::Greater,
            _ => b.priority.cmp(&a.priority),
        }
    });

    // Create display items
    let items: Vec<String> = workable_tickets
        .iter()
        .map(|t| {
            format!(
                "{} {} - {} [{}]",
                match t.status {
                    Status::Doing => "▶️",
                    Status::Blocked => "🚫",
                    _ => "📋",
                },
                t.slug,
                t.title,
                format!("{:?}", t.priority).to_lowercase()
            )
        })
        .collect();

    // Show selection dialog
    formatter.info("🎯 Select a ticket to work on:\n");

    let theme = ColorfulTheme::default();
    let selection = Select::with_theme(&theme)
        .items(&items)
        .default(0)
        .interact()?;

    Ok(workable_tickets[selection].id.to_string())
}

/// Check if we should create a worktree
fn should_create_worktree(project_root: &PathBuf) -> Result<bool> {
    // Check git config for worktree preference
    let config_path = project_root.join(".vibe-ticket").join("config.yaml");

    if config_path.exists() {
        // TODO: Read config to check worktree preference
        // For now, default to true
        Ok(true)
    } else {
        Ok(true)
    }
}

/// Create a git worktree for the ticket
fn create_worktree_for_ticket(
    ticket: &Ticket,
    project_root: &PathBuf,
    formatter: &OutputFormatter,
) -> Result<()> {
    use std::process::Command;

    // Generate worktree name
    let worktree_name = format!(
        "vibe-ticket-vibeticket{}-{}",
        chrono::Utc::now().format("%Y%m%d"),
        ticket.slug
    );

    let worktree_path = project_root
        .parent()
        .unwrap_or(project_root)
        .join(&worktree_name);

    // Check if worktree already exists
    if worktree_path.exists() {
        formatter.info(&format!(
            "📁 Worktree already exists at: {}",
            worktree_path.display()
        ));
        return Ok(());
    }

    // Create branch name
    let branch_name = format!("ticket/{}", ticket.slug);

    // Create worktree
    let output = Command::new("git")
        .args(&["worktree", "add", "-b", &branch_name])
        .arg(&worktree_path)
        .output()?;

    if output.status.success() {
        formatter.success(&format!(
            "📁 Created worktree at: {}",
            worktree_path.display()
        ));
        formatter.info(&format!("🌿 Branch: {}", branch_name));

        // Suggest changing to worktree directory
        formatter.info(&format!(
            "\n💡 To work in the worktree:\n   cd {}",
            worktree_path.display()
        ));
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        formatter.warning(&format!("⚠️  Could not create worktree: {}", error));
    }

    Ok(())
}

/// Display the work context for the ticket
fn display_work_context(ticket: &Ticket, formatter: &OutputFormatter) -> Result<()> {
    formatter.success(&format!(
        "\n🎯 Now working on: {} ({})",
        ticket.title, ticket.slug
    ));

    if !ticket.description.is_empty() {
        formatter.info(&format!("\n📝 Description:\n{}", ticket.description));
    }

    formatter.info(&format!("\n📊 Status: {:?}", ticket.status));
    formatter.info(&format!("🎯 Priority: {:?}", ticket.priority));

    if !ticket.tags.is_empty() {
        formatter.info(&format!("🏷️  Tags: {}", ticket.tags.join(", ")));
    }

    // Show tasks if any
    if !ticket.tasks.is_empty() {
        formatter.info(&format!("\n✅ Tasks ({} total):", ticket.tasks.len()));

        let incomplete_tasks: Vec<_> = ticket.tasks.iter().filter(|t| !t.completed).collect();

        let complete_tasks: Vec<_> = ticket.tasks.iter().filter(|t| t.completed).collect();

        if !incomplete_tasks.is_empty() {
            formatter.info(&format!("  📋 Pending ({}):", incomplete_tasks.len()));
            for task in incomplete_tasks.iter().take(5) {
                formatter.info(&format!("    • {}", task.title));
            }
            if incomplete_tasks.len() > 5 {
                formatter.info(&format!("    ... and {} more", incomplete_tasks.len() - 5));
            }
        }

        if !complete_tasks.is_empty() {
            formatter.info(&format!("  ✓ Completed: {}", complete_tasks.len()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_should_create_worktree() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // Should default to true when no config
        assert!(should_create_worktree(&project_root).unwrap());
    }
}
