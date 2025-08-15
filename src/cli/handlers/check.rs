//! Handler for the `check` command
//!
//! This module implements the logic for checking the current project status,
//! including active ticket information and project statistics.

use crate::cli::{OutputFormatter, find_project_root};
use crate::core::{Status, Ticket};
use crate::error::Result;
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};
use chrono::{DateTime, Local, Utc};

/// Handler for the `check` command
///
/// This function displays:
/// 1. Project information
/// 2. Active ticket details (if any)
/// 3. Current Git branch
/// 4. Project statistics (optional)
/// 5. Recent tickets (in detailed mode)
///
/// # Arguments
///
/// * `detailed` - Whether to show detailed information
/// * `stats` - Whether to include statistics
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - File I/O operations fail
pub fn handle_check_command(
    detailed: bool,
    stats: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    // Gather all data
    let check_data = gather_check_data(detailed, stats, project_dir)?;

    // Output results
    if output.is_json() {
        output_json(&check_data, output)?;
    } else {
        output_text(&check_data, detailed, output);
    }

    Ok(())
}

/// Data structure for check command
struct CheckData {
    project_root: std::path::PathBuf,
    project_state: crate::storage::ProjectState,
    active_ticket: Option<Ticket>,
    current_branch: Option<String>,
    statistics: Option<Statistics>,
    recent_tickets: Vec<Ticket>,
}

/// Gather all data needed for check command
fn gather_check_data(detailed: bool, stats: bool, project_dir: Option<&str>) -> Result<CheckData> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let project_state = storage.load_state()?;
    let active_ticket_id = storage.get_active()?;
    let active_ticket = if let Some(id) = &active_ticket_id {
        Some(storage.load(id)?)
    } else {
        None
    };

    let current_branch = get_current_git_branch(&project_root);
    let statistics = if stats || detailed {
        Some(calculate_statistics(&storage)?)
    } else {
        None
    };

    let recent_tickets = if detailed {
        get_recent_tickets(&storage, 5)?
    } else {
        vec![]
    };

    Ok(CheckData {
        project_root,
        project_state,
        active_ticket,
        current_branch,
        statistics,
        recent_tickets,
    })
}

/// Output check data as JSON
fn output_json(data: &CheckData, output: &OutputFormatter) -> Result<()> {
    output.print_json(&serde_json::json!({
        "project": {
            "name": data.project_state.name,
            "description": data.project_state.description,
            "created_at": data.project_state.created_at,
            "path": data.project_root,
        },
        "active_ticket": data.active_ticket.as_ref().map(|t| serde_json::json!({
            "id": t.id.to_string(),
            "slug": t.slug,
            "title": t.title,
            "status": t.status.to_string(),
            "priority": t.priority.to_string(),
            "started_at": t.started_at,
        })),
        "git_branch": data.current_branch,
        "statistics": data.statistics,
        "recent_tickets": data.recent_tickets.iter().map(|t| serde_json::json!({
            "id": t.id.to_string(),
            "slug": t.slug,
            "title": t.title,
            "status": t.status.to_string(),
        })).collect::<Vec<_>>(),
    }))
}

/// Output check data as text
fn output_text(data: &CheckData, detailed: bool, output: &OutputFormatter) {
    // Project info
    output.info(&format!("Project: {}", data.project_state.name));
    if let Some(desc) = &data.project_state.description {
        output.info(&format!("Description: {desc}"));
    }
    output.info(&format!("Path: {}", data.project_root.display()));
    output.info(&format!(
        "Created: {}",
        format_datetime(data.project_state.created_at)
    ));

    if let Some(branch) = &data.current_branch {
        output.info(&format!("Git branch: {branch}"));
    }

    output.info("");

    // Active ticket
    if let Some(ticket) = &data.active_ticket {
        display_active_ticket(ticket, output);
    } else {
        output.info("No active ticket");
    }

    // Statistics
    if let Some(stats) = &data.statistics {
        display_statistics(stats, detailed, output);
    }

    // Recent tickets
    if detailed && !data.recent_tickets.is_empty() {
        display_recent_tickets(&data.recent_tickets, output);
    }
}

/// Display active ticket information
fn display_active_ticket(ticket: &Ticket, output: &OutputFormatter) {
    output.success("Active Ticket:");
    output.info(&format!("  ID: {}", ticket.id));
    output.info(&format!("  Slug: {}", ticket.slug));
    output.info(&format!("  Title: {}", ticket.title));
    output.info(&format!("  Status: {}", ticket.status));
    output.info(&format!("  Priority: {}", ticket.priority));

    if let Some(started_at) = ticket.started_at {
        let duration = Utc::now() - started_at;
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        output.info(&format!("  Time spent: {hours}h {minutes}m"));
    }

    if !ticket.tasks.is_empty() {
        let completed = ticket.tasks.iter().filter(|t| t.completed).count();
        output.info(&format!("  Tasks: {}/{}", completed, ticket.tasks.len()));
    }
}

/// Display statistics
fn display_statistics(stats: &Statistics, detailed: bool, output: &OutputFormatter) {
    output.info("");
    output.info("Statistics:");
    output.info(&format!("  Total tickets: {}", stats.total));
    output.info(&format!("  Todo: {}", stats.todo));
    output.info(&format!("  In progress: {}", stats.doing));
    output.info(&format!("  In review: {}", stats.review));
    output.info(&format!("  Blocked: {}", stats.blocked));
    output.info(&format!("  Done: {}", stats.done));

    if detailed {
        output.info("");
        output.info("Priority breakdown:");
        output.info(&format!("  Critical: {}", stats.critical));
        output.info(&format!("  High: {}", stats.high));
        output.info(&format!("  Medium: {}", stats.medium));
        output.info(&format!("  Low: {}", stats.low));
    }
}

/// Display recent tickets
fn display_recent_tickets(tickets: &[Ticket], output: &OutputFormatter) {
    output.info("");
    output.info("Recent tickets:");
    for ticket in tickets {
        let status_emoji = match ticket.status {
            Status::Todo => "📋",
            Status::Doing => "🔄",
            Status::Review => "👀",
            Status::Blocked => "🚫",
            Status::Done => "✅",
        };
        output.info(&format!(
            "  {} {} - {} ({})",
            status_emoji, ticket.slug, ticket.title, ticket.priority
        ));
    }
}

/// Project statistics
#[derive(Debug, serde::Serialize)]
struct Statistics {
    total: usize,
    todo: usize,
    doing: usize,
    review: usize,
    blocked: usize,
    done: usize,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
}

/// Calculate project statistics
fn calculate_statistics(storage: &FileStorage) -> Result<Statistics> {
    let tickets = storage.load_all()?;

    let mut stats = Statistics {
        total: tickets.len(),
        todo: 0,
        doing: 0,
        review: 0,
        blocked: 0,
        done: 0,
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
    };

    for ticket in &tickets {
        // Count by status
        match ticket.status {
            Status::Todo => stats.todo += 1,
            Status::Doing => stats.doing += 1,
            Status::Review => stats.review += 1,
            Status::Blocked => stats.blocked += 1,
            Status::Done => stats.done += 1,
        }

        // Count by priority
        match ticket.priority {
            crate::core::Priority::Critical => stats.critical += 1,
            crate::core::Priority::High => stats.high += 1,
            crate::core::Priority::Medium => stats.medium += 1,
            crate::core::Priority::Low => stats.low += 1,
        }
    }

    Ok(stats)
}

/// Get recent tickets sorted by creation date
fn get_recent_tickets(storage: &FileStorage, limit: usize) -> Result<Vec<Ticket>> {
    let mut tickets = storage.load_all()?;

    // Sort by creation date (descending)
    tickets.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Take the specified limit
    tickets.truncate(limit);

    Ok(tickets)
}

/// Get current Git branch name
fn get_current_git_branch(project_root: &std::path::Path) -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(project_root)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
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
    }
}
