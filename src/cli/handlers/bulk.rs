//! Bulk operations handler for managing multiple tickets at once

use crate::cli::output::OutputFormatter;
use crate::cli::utils::find_project_root;
use crate::core::{Priority, Status, Ticket};
use crate::error::{Result, VibeTicketError};
use crate::storage::{FileStorage, TicketRepository};
use chrono::Utc;

/// Check if a ticket is archived (stored in metadata)
fn is_archived(ticket: &Ticket) -> bool {
    ticket
        .metadata
        .get("archived")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Parse a filter expression into key-value pairs
/// Format: "key:value key2:value2" or "key:value,value2"
fn parse_filter_expression(filter: &str) -> Vec<(String, Vec<String>)> {
    let mut filters = Vec::new();

    for part in filter.split_whitespace() {
        if let Some((key, value)) = part.split_once(':') {
            let values: Vec<String> = value.split(',').map(|s| s.to_string()).collect();
            filters.push((key.to_string(), values));
        }
    }

    filters
}

/// Check if a ticket matches the filter criteria
fn ticket_matches_filter(ticket: &Ticket, filters: &[(String, Vec<String>)]) -> bool {
    for (key, values) in filters {
        let matches = match key.as_str() {
            "status" => values
                .iter()
                .any(|v| format!("{:?}", ticket.status).to_lowercase() == v.to_lowercase()),
            "priority" => values
                .iter()
                .any(|v| format!("{:?}", ticket.priority).to_lowercase() == v.to_lowercase()),
            "assignee" => {
                if values.contains(&"unassigned".to_string()) {
                    ticket.assignee.is_none()
                } else {
                    ticket.assignee.as_ref().is_some_and(|a| {
                        values
                            .iter()
                            .any(|v| a.to_lowercase().contains(&v.to_lowercase()))
                    })
                }
            },
            "tag" | "tags" => values.iter().any(|v| {
                ticket
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&v.to_lowercase()))
            }),
            "slug" => values
                .iter()
                .any(|v| ticket.slug.to_lowercase().contains(&v.to_lowercase())),
            _ => true, // Unknown filter key, ignore
        };

        if !matches {
            return false;
        }
    }

    true
}

/// Handle bulk update command
pub fn handle_bulk_update(
    filter: String,
    status: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    dry_run: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let filters = parse_filter_expression(&filter);
    let tickets = storage.load_all()?;

    let matching: Vec<_> = tickets
        .iter()
        .filter(|t| ticket_matches_filter(t, &filters))
        .collect();

    if matching.is_empty() {
        output.warning("No tickets match the filter");
        return Ok(());
    }

    // Parse new values
    let new_status = status.as_ref().map(|s| parse_status(s)).transpose()?;
    let new_priority = priority.as_ref().map(|p| parse_priority(p)).transpose()?;

    if dry_run {
        output.info(&format!(
            "Would update {} ticket(s) matching filter '{}':",
            matching.len(),
            filter
        ));
        for ticket in &matching {
            output.info(&format!("  - {} ({})", ticket.slug, ticket.id.short()));
        }
        if let Some(s) = &status {
            output.info(&format!("  Set status: {s}"));
        }
        if let Some(p) = &priority {
            output.info(&format!("  Set priority: {p}"));
        }
        if let Some(a) = &assignee {
            output.info(&format!("  Set assignee: {a}"));
        }
        return Ok(());
    }

    let mut updated_count = 0;
    for ticket in matching {
        let mut updated_ticket = ticket.clone();
        let mut changed = false;

        if let Some(s) = new_status {
            if updated_ticket.status != s {
                updated_ticket.status = s;
                changed = true;
            }
        }

        if let Some(p) = new_priority {
            if updated_ticket.priority != p {
                updated_ticket.priority = p;
                changed = true;
            }
        }

        if let Some(a) = &assignee {
            let new_assignee = if a == "unassigned" || a.is_empty() {
                None
            } else {
                Some(a.clone())
            };
            if updated_ticket.assignee != new_assignee {
                updated_ticket.assignee = new_assignee;
                changed = true;
            }
        }

        if changed {
            storage.save(&updated_ticket)?;
            updated_count += 1;
        }
    }

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "updated_count": updated_count,
            "filter": filter,
        }))?;
    } else {
        output.success(&format!("Updated {updated_count} ticket(s)"));
    }

    Ok(())
}

/// Handle bulk tag command
pub fn handle_bulk_tag(
    filter: String,
    add: Option<String>,
    remove: Option<String>,
    dry_run: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let filters = parse_filter_expression(&filter);
    let tickets = storage.load_all()?;

    let matching: Vec<_> = tickets
        .iter()
        .filter(|t| ticket_matches_filter(t, &filters))
        .collect();

    if matching.is_empty() {
        output.warning("No tickets match the filter");
        return Ok(());
    }

    let tags_to_add: Vec<String> = add
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
        .unwrap_or_default();
    let tags_to_remove: Vec<String> = remove
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
        .unwrap_or_default();

    if tags_to_add.is_empty() && tags_to_remove.is_empty() {
        return Err(VibeTicketError::custom(
            "Must specify --add or --remove tags",
        ));
    }

    if dry_run {
        output.info(&format!(
            "Would update tags on {} ticket(s) matching filter '{}':",
            matching.len(),
            filter
        ));
        for ticket in &matching {
            output.info(&format!("  - {} ({})", ticket.slug, ticket.id.short()));
        }
        if !tags_to_add.is_empty() {
            output.info(&format!("  Add tags: {}", tags_to_add.join(", ")));
        }
        if !tags_to_remove.is_empty() {
            output.info(&format!("  Remove tags: {}", tags_to_remove.join(", ")));
        }
        return Ok(());
    }

    let mut updated_count = 0;
    for ticket in matching {
        let mut updated_ticket = ticket.clone();
        let mut changed = false;

        // Add tags
        for tag in &tags_to_add {
            if !updated_ticket.tags.contains(tag) {
                updated_ticket.tags.push(tag.clone());
                changed = true;
            }
        }

        // Remove tags
        for tag in &tags_to_remove {
            if let Some(pos) = updated_ticket.tags.iter().position(|t| t == tag) {
                updated_ticket.tags.remove(pos);
                changed = true;
            }
        }

        if changed {
            storage.save(&updated_ticket)?;
            updated_count += 1;
        }
    }

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "updated_count": updated_count,
            "filter": filter,
            "tags_added": tags_to_add,
            "tags_removed": tags_to_remove,
        }))?;
    } else {
        output.success(&format!("Updated tags on {updated_count} ticket(s)"));
    }

    Ok(())
}

/// Handle bulk close command
pub fn handle_bulk_close(
    filter: String,
    message: Option<String>,
    archive: bool,
    dry_run: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let filters = parse_filter_expression(&filter);
    let tickets = storage.load_all()?;

    let matching: Vec<_> = tickets
        .iter()
        .filter(|t| ticket_matches_filter(t, &filters))
        .filter(|t| t.status != Status::Done) // Don't close already closed tickets
        .collect();

    if matching.is_empty() {
        output.warning("No open tickets match the filter");
        return Ok(());
    }

    if dry_run {
        output.info(&format!(
            "Would close {} ticket(s) matching filter '{}':",
            matching.len(),
            filter
        ));
        for ticket in &matching {
            output.info(&format!("  - {} ({})", ticket.slug, ticket.id.short()));
        }
        if archive {
            output.info("  (and archive them)");
        }
        return Ok(());
    }

    let mut closed_count = 0;
    for ticket in matching {
        let mut updated_ticket = ticket.clone();
        updated_ticket.status = Status::Done;
        updated_ticket.closed_at = Some(Utc::now());

        if let Some(msg) = &message {
            // Add close message to metadata
            updated_ticket
                .metadata
                .insert("close_message".to_string(), serde_json::json!(msg));
        }

        if archive {
            // Store archived status in metadata
            updated_ticket
                .metadata
                .insert("archived".to_string(), serde_json::json!(true));
        }

        storage.save(&updated_ticket)?;
        closed_count += 1;
    }

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "closed_count": closed_count,
            "filter": filter,
            "archived": archive,
        }))?;
    } else {
        let action = if archive {
            "Closed and archived"
        } else {
            "Closed"
        };
        output.success(&format!("{action} {closed_count} ticket(s)"));
    }

    Ok(())
}

/// Handle bulk archive command
pub fn handle_bulk_archive(
    filter: String,
    dry_run: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let filters = parse_filter_expression(&filter);
    let tickets = storage.load_all()?;

    let matching: Vec<_> = tickets
        .iter()
        .filter(|t| ticket_matches_filter(t, &filters))
        .filter(|t| !is_archived(t)) // Don't archive already archived tickets
        .collect();

    if matching.is_empty() {
        output.warning("No non-archived tickets match the filter");
        return Ok(());
    }

    if dry_run {
        output.info(&format!(
            "Would archive {} ticket(s) matching filter '{}':",
            matching.len(),
            filter
        ));
        for ticket in &matching {
            output.info(&format!("  - {} ({})", ticket.slug, ticket.id.short()));
        }
        return Ok(());
    }

    let mut archived_count = 0;
    for ticket in matching {
        let mut updated_ticket = ticket.clone();
        // Store archived status in metadata
        updated_ticket
            .metadata
            .insert("archived".to_string(), serde_json::json!(true));
        storage.save(&updated_ticket)?;
        archived_count += 1;
    }

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "archived_count": archived_count,
            "filter": filter,
        }))?;
    } else {
        output.success(&format!("Archived {archived_count} ticket(s)"));
    }

    Ok(())
}

fn parse_status(s: &str) -> Result<Status> {
    match s.to_lowercase().as_str() {
        "todo" => Ok(Status::Todo),
        "doing" => Ok(Status::Doing),
        "done" => Ok(Status::Done),
        "blocked" => Ok(Status::Blocked),
        "review" => Ok(Status::Review),
        _ => Err(VibeTicketError::custom(format!(
            "Invalid status: {s}. Valid values: todo, doing, done, blocked, review"
        ))),
    }
}

fn parse_priority(p: &str) -> Result<Priority> {
    match p.to_lowercase().as_str() {
        "low" => Ok(Priority::Low),
        "medium" => Ok(Priority::Medium),
        "high" => Ok(Priority::High),
        "critical" => Ok(Priority::Critical),
        _ => Err(VibeTicketError::custom(format!(
            "Invalid priority: {p}. Valid values: low, medium, high, critical"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filter_expression() {
        let filters = parse_filter_expression("status:todo priority:high");
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].0, "status");
        assert_eq!(filters[0].1, vec!["todo"]);
        assert_eq!(filters[1].0, "priority");
        assert_eq!(filters[1].1, vec!["high"]);
    }

    #[test]
    fn test_parse_filter_expression_multiple_values() {
        let filters = parse_filter_expression("status:todo,doing");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].0, "status");
        assert_eq!(filters[0].1, vec!["todo", "doing"]);
    }

    #[test]
    fn test_parse_status() {
        assert!(parse_status("todo").is_ok());
        assert!(parse_status("DOING").is_ok());
        assert!(parse_status("invalid").is_err());
    }

    #[test]
    fn test_parse_priority() {
        assert!(parse_priority("low").is_ok());
        assert!(parse_priority("CRITICAL").is_ok());
        assert!(parse_priority("invalid").is_err());
    }
}
