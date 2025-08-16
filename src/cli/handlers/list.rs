use crate::cli::{OutputFormatter, find_project_root};
use crate::core::{Priority, Status, Ticket};
use crate::error::{Result, VibeTicketError};
use crate::storage::{FileStorage, TicketRepository};
use chrono::{DateTime, Duration, Local, NaiveDate, Utc};

/// Handler for the `list` command
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub fn handle_list_command(
    status: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    sort: &str,
    reverse: bool,
    limit: Option<usize>,
    archived: bool,
    open: bool,
    since: Option<String>,
    until: Option<String>,
    include_done: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    // Ensure project is initialized
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");

    // Initialize storage
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Load all tickets
    let mut tickets = storage.load_all()?;

    // Parse date filters
    let since_date = since.map(|s| parse_date_filter(&s)).transpose()?;
    let until_date = until.map(|s| parse_date_filter(&s)).transpose()?;

    // Apply filters
    tickets = filter_tickets(
        tickets,
        status,
        priority,
        assignee,
        archived,
        open,
        since_date,
        until_date,
        include_done,
    )?;

    // Sort tickets
    sort_tickets(&mut tickets, sort, reverse);

    // Apply limit
    if let Some(limit) = limit {
        tickets.truncate(limit);
    }

    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "tickets": tickets,
            "count": tickets.len(),
        }))?;
    } else if tickets.is_empty() {
        output.info("No tickets found matching the criteria.");
    } else {
        output.print_tickets(&tickets)?;
    }

    Ok(())
}

/// Parse date filter strings
fn parse_date_filter(date_str: &str) -> Result<DateTime<Utc>> {
    let date_str = date_str.trim().to_lowercase();

    // Handle relative dates
    if date_str == "today" {
        return Ok(Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc));
    } else if date_str == "yesterday" {
        return Ok((Local::now() - Duration::days(1))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc));
    } else if date_str == "tomorrow" {
        return Ok((Local::now() + Duration::days(1))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc));
    }

    // Handle "X hours ago" format
    if let Some(captures) = regex::Regex::new(r"^(\d+)\s+hours?\s+ago$")
        .unwrap()
        .captures(&date_str)
    {
        if let Some(hours_str) = captures.get(1) {
            if let Ok(hours) = hours_str.as_str().parse::<i64>() {
                return Ok((Local::now() - Duration::hours(hours)).with_timezone(&Utc));
            }
        }
    }

    // Handle "X days ago" format
    if let Some(captures) = regex::Regex::new(r"^(\d+)\s+days?\s+ago$")
        .unwrap()
        .captures(&date_str)
    {
        if let Some(days_str) = captures.get(1) {
            if let Ok(days) = days_str.as_str().parse::<i64>() {
                return Ok((Local::now() - Duration::days(days))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc));
            }
        }
    }

    // Handle "X weeks ago" format
    if let Some(captures) = regex::Regex::new(r"^(\d+)\s+weeks?\s+ago$")
        .unwrap()
        .captures(&date_str)
    {
        if let Some(weeks_str) = captures.get(1) {
            if let Ok(weeks) = weeks_str.as_str().parse::<i64>() {
                return Ok((Local::now() - Duration::weeks(weeks))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc));
            }
        }
    }

    // Try parsing as ISO date (YYYY-MM-DD)
    if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }

    Err(VibeTicketError::custom(format!(
        "Invalid date format: '{date_str}'. Use formats like 'yesterday', '2 hours ago', '3 days ago', or 'YYYY-MM-DD'"
    )))
}

/// Filter tickets based on criteria
#[allow(clippy::too_many_arguments)]
fn filter_tickets(
    tickets: Vec<Ticket>,
    status: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    archived: bool,
    open: bool,
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
    include_done: bool,
) -> Result<Vec<Ticket>> {
    let mut filtered = tickets;

    // Filter by status
    if let Some(status_str) = status {
        let status = Status::try_from(status_str.as_str())
            .map_err(|_| VibeTicketError::InvalidStatus { status: status_str })?;
        filtered.retain(|t| t.status == status);
    } else if !include_done {
        // If no status filter is provided and include_done is false, exclude DONE tickets
        filtered.retain(|t| t.status != Status::Done);
    }

    // Filter by priority
    if let Some(priority_str) = priority {
        let priority = Priority::try_from(priority_str.as_str()).map_err(|_| {
            VibeTicketError::InvalidPriority {
                priority: priority_str,
            }
        })?;
        filtered.retain(|t| t.priority == priority);
    }

    // Filter by assignee
    if let Some(assignee) = assignee {
        filtered.retain(|t| t.assignee.as_ref() == Some(&assignee));
    }

    // Filter by archived status
    if !archived {
        // Filter out archived tickets
        filtered.retain(|t| {
            !t.metadata
                .get("archived")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        });
    }

    // Filter by open status (todo, doing)
    if open {
        filtered.retain(|t| matches!(t.status, Status::Todo | Status::Doing));
    }

    // Filter by date range
    if let Some(since) = since {
        filtered.retain(|t| t.created_at >= since);
    }

    if let Some(until) = until {
        filtered.retain(|t| t.created_at <= until);
    }

    Ok(filtered)
}

/// Sort tickets based on the specified field
fn sort_tickets(tickets: &mut [Ticket], sort_by: &str, reverse: bool) {
    match sort_by {
        "created" => {
            tickets.sort_by_key(|t| t.created_at);
        },
        "updated" => {
            // For now, sort by created_at as we don't have updated_at
            tickets.sort_by_key(|t| t.created_at);
        },
        "priority" => {
            tickets.sort_by_key(|t| t.priority);
        },
        "status" => {
            tickets.sort_by(|a, b| {
                // Custom sort order for status
                let order = |s: &Status| match s {
                    Status::Doing => 0,
                    Status::Review => 1,
                    Status::Blocked => 2,
                    Status::Todo => 3,
                    Status::Done => 4,
                };
                order(&a.status).cmp(&order(&b.status))
            });
        },
        "slug" => {
            // Default to slug sort (which will be chronological due to timestamp prefix)
            tickets.sort_by(|a, b| a.slug.cmp(&b.slug));
        },
        _ => {
            tickets.sort_by(|a, b| a.slug.cmp(&b.slug));
        },
    }

    if reverse {
        tickets.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_filter() {
        // Test "yesterday" - should be some time yesterday
        let yesterday_result = parse_date_filter("yesterday").unwrap();
        let now = Utc::now();
        let diff = now.signed_duration_since(yesterday_result);
        // Should be between 1 and 2 days ago
        assert!(
            diff.num_hours() >= 24 && diff.num_hours() <= 48,
            "Expected yesterday to be 24-48 hours ago, got {} hours",
            diff.num_hours()
        );

        // Test "X days ago" - should be X days ago at midnight
        let three_days_ago = parse_date_filter("3 days ago").unwrap();
        let diff = now.signed_duration_since(three_days_ago);
        // Should be between 3 and 4 days ago (72-96 hours)
        assert!(
            diff.num_hours() >= 72 && diff.num_hours() <= 96,
            "Expected 3 days ago to be 72-96 hours ago, got {} hours",
            diff.num_hours()
        );

        // Test ISO date
        let iso_date = parse_date_filter("2025-07-15").unwrap();
        assert_eq!(iso_date.format("%Y-%m-%d").to_string(), "2025-07-15");

        // Test invalid format
        assert!(parse_date_filter("invalid").is_err());
    }
}
