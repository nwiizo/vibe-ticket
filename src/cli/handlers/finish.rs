//! Intent-focused finish command handler
//!
//! Helps users complete their work on a ticket with proper cleanup
//! and documentation of what was accomplished.

use crate::cli::output::OutputFormatter;
use crate::cli::utils;
use crate::core::{Status, Ticket, TicketId};
use crate::error::{Result, VibeTicketError};
use crate::storage::{FileStorage, TicketRepository};
use chrono::Utc;
use dialoguer::{Confirm, Editor, Input, MultiSelect, theme::ColorfulTheme};
use std::env;
use std::fs;
use std::path::Path;

/// Handle the intent-focused finish command
///
/// This command helps users:
/// 1. Complete all remaining tasks
/// 2. Document what was accomplished
/// 3. Clean up the work environment
/// 4. Close the ticket properly
pub fn handle_finish_command(
    ticket: Option<String>,
    message: Option<String>,
    keep_worktree: bool,
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

    // Get ticket to finish
    let ticket_id_str = if let Some(t) = ticket {
        t
    } else {
        // Try to get active ticket
        get_active_ticket(&tickets_dir)?
    };

    // Parse ticket ID
    let ticket_id = TicketId::parse_str(&ticket_id_str)
        .map_err(|_| VibeTicketError::Custom(format!("Invalid ticket ID: {ticket_id_str}")))?;

    // Load the ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Check if already done
    if ticket.status == Status::Done {
        formatter.info(&format!(
            "✅ Ticket '{}' is already completed.",
            ticket.title
        ));
        return Ok(());
    }

    // Check and handle incomplete tasks
    let has_incomplete_tasks = ticket.tasks.iter().any(|t| !t.completed);

    if has_incomplete_tasks {
        handle_incomplete_tasks(&mut ticket, formatter)?;
    }

    // Get closing message
    let closing_message = if let Some(msg) = message {
        msg
    } else {
        get_closing_message(&ticket, formatter)?
    };

    // Update ticket
    ticket.status = Status::Done;
    ticket.closed_at = Some(Utc::now());

    // Add closing message to metadata
    ticket.metadata.insert(
        "closing_message".to_string(),
        closing_message.clone().into(),
    );

    // Save ticket
    storage.save(&ticket)?;

    // Clear active ticket
    let active_ticket_path = tickets_dir.join("active_ticket");
    if active_ticket_path.exists() {
        fs::remove_file(&active_ticket_path)?;
    }

    // Handle worktree cleanup
    if !keep_worktree {
        cleanup_worktree(&ticket, &project_root, formatter)?;
    }

    // Success message
    formatter.success(&format!(
        "🎉 Completed ticket '{}' ({})",
        ticket.title, ticket.slug
    ));

    // Show summary
    show_completion_summary(&ticket, &closing_message, formatter)?;

    // Suggest next actions
    formatter.info("\n💡 What's next?");
    formatter.info("  • Review other tickets: vibe-ticket review");
    formatter.info("  • Create a new ticket: vibe-ticket create");
    formatter.info("  • Generate a report: vibe-ticket report");

    Ok(())
}

/// Get the currently active ticket
fn get_active_ticket(tickets_dir: &Path) -> Result<String> {
    let active_ticket_path = tickets_dir.join("active_ticket");

    if active_ticket_path.exists() {
        let content = fs::read_to_string(&active_ticket_path)?;
        Ok(content.trim().to_string())
    } else {
        Err(VibeTicketError::Custom(
            "No active ticket. Specify a ticket ID or use 'vibe-ticket work-on' first.".to_string(),
        ))
    }
}

/// Handle incomplete tasks
fn handle_incomplete_tasks(ticket: &mut Ticket, formatter: &OutputFormatter) -> Result<()> {
    // Collect incomplete tasks info first
    let incomplete_tasks: Vec<(crate::core::TaskId, String)> = ticket
        .tasks
        .iter()
        .filter(|t| !t.completed)
        .map(|t| (t.id.clone(), t.title.clone()))
        .collect();

    if incomplete_tasks.is_empty() {
        return Ok(());
    }
    formatter.warning(&format!(
        "⚠️  There are {} incomplete tasks:",
        incomplete_tasks.len()
    ));

    for (_, title) in incomplete_tasks.iter().take(5) {
        formatter.info(&format!("  • {title}"));
    }

    if incomplete_tasks.len() > 5 {
        formatter.info(&format!("  ... and {} more", incomplete_tasks.len() - 5));
    }

    let theme = ColorfulTheme::default();

    // Ask what to do
    if Confirm::with_theme(&theme)
        .with_prompt("Mark all tasks as complete?")
        .default(false)
        .interact()?
    {
        // Mark all as complete
        for task in &mut ticket.tasks {
            if !task.completed {
                task.completed = true;
                task.completed_at = Some(Utc::now());
            }
        }
        formatter.success("✅ All tasks marked as complete.");
    } else if incomplete_tasks.len() > 1 {
        // Select which tasks to complete
        let task_titles: Vec<String> = incomplete_tasks
            .iter()
            .map(|(_, title)| title.clone())
            .collect();

        let selections = MultiSelect::with_theme(&theme)
            .with_prompt("Select tasks to mark as complete")
            .items(&task_titles)
            .interact()?;

        if !selections.is_empty() {
            let selected_ids: Vec<_> = selections
                .iter()
                .map(|&i| incomplete_tasks[i].0.clone())
                .collect();

            for task in &mut ticket.tasks {
                if selected_ids.contains(&task.id) {
                    task.completed = true;
                    task.completed_at = Some(Utc::now());
                }
            }

            formatter.success(&format!(
                "✅ Marked {} tasks as complete.",
                selections.len()
            ));
        }
    }

    Ok(())
}

/// Get closing message from user
fn get_closing_message(ticket: &Ticket, formatter: &OutputFormatter) -> Result<String> {
    formatter.info("\n📝 Document what was accomplished:");
    formatter.info("  (Press Enter to open editor, or type inline)");

    let theme = ColorfulTheme::default();

    // First try inline input
    let inline = Input::<String>::with_theme(&theme)
        .with_prompt("Summary")
        .allow_empty(true)
        .interact()?;

    if !inline.is_empty() {
        Ok(inline)
    } else {
        // Open editor for longer message
        let template = format!(
            "# Closing ticket: {}\n\
            #\n\
            # Please describe what was accomplished.\n\
            # Lines starting with '#' will be ignored.\n\
            #\n\
            # Ticket: {}\n\
            # Priority: {:?}\n\
            # Started: {}\n\
            #\n\
            \n",
            ticket.title,
            ticket.slug,
            ticket.priority,
            ticket
                .started_at
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        );

        if let Some(message) = Editor::new().edit(&template)? {
            // Remove comment lines
            let cleaned: String = message
                .lines()
                .filter(|line| !line.trim_start().starts_with('#'))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            if cleaned.is_empty() {
                Ok("Ticket completed.".to_string())
            } else {
                Ok(cleaned)
            }
        } else {
            Ok("Ticket completed.".to_string())
        }
    }
}

/// Clean up the worktree for the ticket
fn cleanup_worktree(
    ticket: &Ticket,
    _project_root: &Path,
    formatter: &OutputFormatter,
) -> Result<()> {
    use std::process::Command;

    // Check for worktree
    let output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .output()?;

    if !output.status.success() {
        return Ok(()); // Not a git repo or worktree not available
    }

    let worktree_list = String::from_utf8_lossy(&output.stdout);
    let ticket_worktree_pattern = format!("vibeticket.*-{}", ticket.slug);

    // Find matching worktree
    let mut worktree_path = None;

    for line in worktree_list.lines() {
        if line.starts_with("worktree ") {
            let path = line.strip_prefix("worktree ").unwrap_or("");
            if path.contains(&ticket_worktree_pattern) {
                worktree_path = Some(path.to_string());
            }
        }
    }

    if let Some(path) = worktree_path {
        let theme = ColorfulTheme::default();

        if Confirm::with_theme(&theme)
            .with_prompt(&format!("Remove worktree at {path}?"))
            .default(true)
            .interact()?
        {
            // Remove the worktree
            let output = Command::new("git")
                .args(&["worktree", "remove", &path, "--force"])
                .output()?;

            if output.status.success() {
                formatter.success("🧹 Removed worktree.");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                formatter.warning(&format!("⚠️  Could not remove worktree: {error}"));
            }
        }
    }

    Ok(())
}

/// Show completion summary
fn show_completion_summary(
    ticket: &Ticket,
    closing_message: &str,
    formatter: &OutputFormatter,
) -> Result<()> {
    formatter.info("\n📊 Completion Summary:");
    formatter.info(&format!("  • Title: {}", ticket.title));
    formatter.info(&format!("  • Duration: {}", calculate_duration(ticket)));

    let completed_tasks = ticket.tasks.iter().filter(|t| t.completed).count();
    let total_tasks = ticket.tasks.len();

    if total_tasks > 0 {
        formatter.info(&format!(
            "  • Tasks: {}/{} completed",
            completed_tasks, total_tasks
        ));
    }

    formatter.info(&format!("  • Message: {}", closing_message));

    Ok(())
}

/// Calculate work duration
fn calculate_duration(ticket: &Ticket) -> String {
    if let Some(started) = ticket.started_at {
        let duration = Utc::now() - started;

        if duration.num_days() > 0 {
            format!("{} days", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours", duration.num_hours())
        } else {
            format!("{} minutes", duration.num_minutes())
        }
    } else {
        "Unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TicketBuilder;

    #[test]
    fn test_calculate_duration() {
        let ticket = TicketBuilder::new().slug("test").title("Test").build();

        assert_eq!(calculate_duration(&ticket), "Unknown");

        let mut ticket_with_start = ticket.clone();
        ticket_with_start.started_at = Some(Utc::now() - chrono::Duration::hours(3));
        assert!(calculate_duration(&ticket_with_start).contains("hours"));
    }
}
