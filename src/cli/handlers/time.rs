//! Time tracking handler for logging work time on tickets

use crate::cli::output::OutputFormatter;
use crate::cli::utils::find_project_root;
use crate::error::{Result, VibeTicketError};
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A time entry for a ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    /// Unique entry ID
    pub id: String,
    /// Ticket ID
    pub ticket_id: String,
    /// Duration in minutes
    pub duration_minutes: i64,
    /// Notes about the work
    pub notes: Option<String>,
    /// Date of the work
    pub date: DateTime<Utc>,
    /// When the entry was created
    pub created_at: DateTime<Utc>,
}

/// Active timer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTimer {
    /// Ticket ID being tracked
    pub ticket_id: String,
    /// Ticket slug for display
    pub ticket_slug: String,
    /// When the timer started
    pub started_at: DateTime<Utc>,
    /// Notes about the work
    pub notes: Option<String>,
}

/// Time tracking data store
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TimeTracking {
    /// Time entries by ticket ID
    pub entries: HashMap<String, Vec<TimeEntry>>,
    /// Currently active timer
    pub active_timer: Option<ActiveTimer>,
}

impl TimeTracking {
    /// Load time tracking data from file
    pub fn load(project_dir: Option<&str>) -> Result<Self> {
        let path = Self::data_path(project_dir)?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            VibeTicketError::custom(format!("Failed to read time tracking file: {e}"))
        })?;
        let data: Self = serde_yaml::from_str(&content).map_err(|e| {
            VibeTicketError::custom(format!("Failed to parse time tracking file: {e}"))
        })?;
        Ok(data)
    }

    /// Save time tracking data to file
    pub fn save(&self, project_dir: Option<&str>) -> Result<()> {
        let path = Self::data_path(project_dir)?;
        let content = serde_yaml::to_string(self).map_err(|e| {
            VibeTicketError::custom(format!("Failed to serialize time tracking: {e}"))
        })?;
        fs::write(&path, content).map_err(|e| {
            VibeTicketError::custom(format!("Failed to write time tracking file: {e}"))
        })?;
        Ok(())
    }

    /// Get the path to the time tracking file
    fn data_path(project_dir: Option<&str>) -> Result<PathBuf> {
        let project_root = find_project_root(project_dir)?;
        Ok(project_root.join(".vibe-ticket").join("time_tracking.yaml"))
    }

    /// Add a time entry
    pub fn add_entry(&mut self, entry: TimeEntry) {
        self.entries
            .entry(entry.ticket_id.clone())
            .or_default()
            .push(entry);
    }

    /// Get total time for a ticket in minutes
    pub fn total_time_for_ticket(&self, ticket_id: &str) -> i64 {
        self.entries
            .get(ticket_id)
            .map(|entries| entries.iter().map(|e| e.duration_minutes).sum())
            .unwrap_or(0)
    }
}

/// Parse time string like "1h30m", "2h", "45m" into minutes
fn parse_time_string(time: &str) -> Result<i64> {
    let time = time.to_lowercase();
    let mut total_minutes: i64 = 0;
    let mut current_num = String::new();

    for c in time.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else if c == 'h' {
            let hours: i64 = current_num
                .parse()
                .map_err(|_| VibeTicketError::custom(format!("Invalid time format: {time}")))?;
            total_minutes += hours * 60;
            current_num.clear();
        } else if c == 'm' {
            let minutes: i64 = current_num
                .parse()
                .map_err(|_| VibeTicketError::custom(format!("Invalid time format: {time}")))?;
            total_minutes += minutes;
            current_num.clear();
        }
    }

    // If there's a remaining number without unit, treat as minutes
    if !current_num.is_empty() {
        let minutes: i64 = current_num
            .parse()
            .map_err(|_| VibeTicketError::custom(format!("Invalid time format: {time}")))?;
        total_minutes += minutes;
    }

    if total_minutes == 0 {
        return Err(VibeTicketError::custom(format!(
            "Invalid time format: {time}. Use format like '1h30m', '2h', or '45m'"
        )));
    }

    Ok(total_minutes)
}

/// Format minutes as human-readable string
fn format_duration(minutes: i64) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 && mins > 0 {
        format!("{hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h")
    } else {
        format!("{mins}m")
    }
}

/// Resolve ticket reference to ID and slug
fn resolve_ticket(
    ticket_ref: Option<String>,
    project_dir: Option<&str>,
) -> Result<(String, String)> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let ticket_id = if let Some(ref_str) = ticket_ref {
        crate::cli::handlers::common::resolve_ticket_ref(&storage, &ref_str)?
    } else {
        storage
            .get_active()?
            .ok_or(VibeTicketError::NoActiveTicket)?
    };

    let ticket = storage.load(&ticket_id)?;
    Ok((ticket_id.to_string(), ticket.slug))
}

/// Handle time log command
pub fn handle_time_log(
    time: String,
    ticket: Option<String>,
    notes: Option<String>,
    date: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let (ticket_id, ticket_slug) = resolve_ticket(ticket, project_dir)?;
    let duration_minutes = parse_time_string(&time)?;

    let entry_date = if let Some(date_str) = date {
        // Parse date string - simplified for now
        chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|_| {
                VibeTicketError::custom(format!("Invalid date format: {date_str}. Use YYYY-MM-DD"))
            })
            .and_then(|d| {
                d.and_hms_opt(12, 0, 0)
                    .map(|dt| dt.and_utc())
                    .ok_or_else(|| {
                        VibeTicketError::custom("Failed to create date time".to_string())
                    })
            })?
    } else {
        Utc::now()
    };

    let entry = TimeEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ticket_id: ticket_id.clone(),
        duration_minutes,
        notes: notes.clone(),
        date: entry_date,
        created_at: Utc::now(),
    };

    let mut tracking = TimeTracking::load(project_dir)?;
    tracking.add_entry(entry);
    tracking.save(project_dir)?;

    let total = tracking.total_time_for_ticket(&ticket_id);

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket_id": ticket_id,
            "ticket_slug": ticket_slug,
            "logged": format_duration(duration_minutes),
            "total": format_duration(total),
        }))?;
    } else {
        output.success(&format!(
            "Logged {} on ticket '{}'",
            format_duration(duration_minutes),
            ticket_slug
        ));
        if let Some(n) = notes {
            output.info(&format!("Notes: {n}"));
        }
        output.info(&format!("Total time: {}", format_duration(total)));
    }

    Ok(())
}

/// Handle time start command
pub fn handle_time_start(
    ticket: Option<String>,
    notes: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let (ticket_id, ticket_slug) = resolve_ticket(ticket, project_dir)?;

    let mut tracking = TimeTracking::load(project_dir)?;

    if tracking.active_timer.is_some() {
        return Err(VibeTicketError::custom(
            "Timer already running. Stop it first with 'vibe-ticket time stop'",
        ));
    }

    tracking.active_timer = Some(ActiveTimer {
        ticket_id,
        ticket_slug: ticket_slug.clone(),
        started_at: Utc::now(),
        notes,
    });

    tracking.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "action": "started",
            "ticket_slug": ticket_slug,
            "started_at": Utc::now().to_rfc3339(),
        }))?;
    } else {
        output.success(&format!("Started timer for ticket '{ticket_slug}'"));
        output.info(&format!("Started at: {}", Utc::now().format("%H:%M:%S")));
    }

    Ok(())
}

/// Handle time stop command
pub fn handle_time_stop(
    notes: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let mut tracking = TimeTracking::load(project_dir)?;

    let timer = tracking
        .active_timer
        .take()
        .ok_or_else(|| VibeTicketError::custom("No timer running"))?;

    let duration = Utc::now().signed_duration_since(timer.started_at);
    let duration_minutes = duration.num_minutes();

    // Round up to at least 1 minute
    let duration_minutes = if duration_minutes < 1 {
        1
    } else {
        duration_minutes
    };

    let final_notes = notes.or(timer.notes);

    let entry = TimeEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ticket_id: timer.ticket_id.clone(),
        duration_minutes,
        notes: final_notes.clone(),
        date: Utc::now(),
        created_at: Utc::now(),
    };

    tracking.add_entry(entry);
    tracking.save(project_dir)?;

    let total = tracking.total_time_for_ticket(&timer.ticket_id);

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "action": "stopped",
            "ticket_slug": timer.ticket_slug,
            "logged": format_duration(duration_minutes),
            "total": format_duration(total),
        }))?;
    } else {
        output.success(&format!("Stopped timer for ticket '{}'", timer.ticket_slug));
        output.info(&format!(
            "Time logged: {}",
            format_duration(duration_minutes)
        ));
        if let Some(n) = final_notes {
            output.info(&format!("Notes: {n}"));
        }
        output.info(&format!("Total time on ticket: {}", format_duration(total)));
    }

    Ok(())
}

/// Handle time status command
pub fn handle_time_status(project_dir: Option<&str>, output: &OutputFormatter) -> Result<()> {
    let tracking = TimeTracking::load(project_dir)?;

    if let Some(timer) = &tracking.active_timer {
        let elapsed = Utc::now().signed_duration_since(timer.started_at);
        let elapsed_str = format_duration(elapsed.num_minutes().max(0));

        if output.is_json() {
            output.print_json(&serde_json::json!({
                "status": "running",
                "ticket_id": timer.ticket_id,
                "ticket_slug": timer.ticket_slug,
                "started_at": timer.started_at.to_rfc3339(),
                "elapsed": elapsed_str,
                "notes": timer.notes,
            }))?;
        } else {
            output.success("Timer is running");
            output.info(&format!("Ticket: {}", timer.ticket_slug));
            output.info(&format!(
                "Started: {}",
                timer.started_at.format("%Y-%m-%d %H:%M:%S")
            ));
            output.info(&format!("Elapsed: {elapsed_str}"));
            if let Some(notes) = &timer.notes {
                output.info(&format!("Notes: {notes}"));
            }
        }
    } else if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "stopped",
        }))?;
    } else {
        output.info("No timer running");
        output.info("");
        output.info("Start one with:");
        output.info("  vibe-ticket time start");
    }

    Ok(())
}

/// Handle time report command
pub fn handle_time_report(
    ticket: Option<String>,
    all: bool,
    _since: Option<String>,
    _until: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let tracking = TimeTracking::load(project_dir)?;

    if all {
        // Show summary for all tickets
        let mut total_minutes: i64 = 0;
        let mut ticket_totals: Vec<(String, i64)> = Vec::new();

        for (ticket_id, entries) in &tracking.entries {
            let ticket_total: i64 = entries.iter().map(|e| e.duration_minutes).sum();
            total_minutes += ticket_total;
            ticket_totals.push((ticket_id.clone(), ticket_total));
        }

        ticket_totals.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by time, descending

        if output.is_json() {
            let report: Vec<_> = ticket_totals
                .iter()
                .map(|(id, mins)| {
                    serde_json::json!({
                        "ticket_id": id,
                        "total": format_duration(*mins),
                        "minutes": mins,
                    })
                })
                .collect();
            output.print_json(&serde_json::json!({
                "tickets": report,
                "total": format_duration(total_minutes),
                "total_minutes": total_minutes,
            }))?;
        } else {
            output.info("Time Report (All Tickets)");
            output.info(&format!("Total: {}", format_duration(total_minutes)));
            output.info("");

            for (ticket_id, mins) in ticket_totals {
                // Get short ID
                let short_id = if ticket_id.len() > 8 {
                    &ticket_id[..8]
                } else {
                    &ticket_id
                };
                output.info(&format!("  {}: {}", short_id, format_duration(mins)));
            }
        }
    } else {
        // Show report for specific ticket
        let (ticket_id, ticket_slug) = resolve_ticket(ticket, project_dir)?;

        let entries = tracking
            .entries
            .get(&ticket_id)
            .cloned()
            .unwrap_or_default();
        let total: i64 = entries.iter().map(|e| e.duration_minutes).sum();

        if output.is_json() {
            let entry_list: Vec<_> = entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "duration": format_duration(e.duration_minutes),
                        "minutes": e.duration_minutes,
                        "date": e.date.format("%Y-%m-%d").to_string(),
                        "notes": e.notes,
                    })
                })
                .collect();
            output.print_json(&serde_json::json!({
                "ticket_id": ticket_id,
                "ticket_slug": ticket_slug,
                "entries": entry_list,
                "total": format_duration(total),
                "total_minutes": total,
            }))?;
        } else {
            output.info(&format!("Time Report for '{ticket_slug}'"));
            output.info(&format!("Total: {}", format_duration(total)));
            output.info("");

            if entries.is_empty() {
                output.info("No time entries");
            } else {
                for entry in entries.iter().rev().take(10) {
                    let date = entry.date.format("%Y-%m-%d").to_string();
                    let duration = format_duration(entry.duration_minutes);
                    let notes = entry.notes.as_deref().unwrap_or("-");
                    output.info(&format!("  {date} - {duration} - {notes}"));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_string() {
        assert_eq!(parse_time_string("1h").unwrap(), 60);
        assert_eq!(parse_time_string("30m").unwrap(), 30);
        assert_eq!(parse_time_string("1h30m").unwrap(), 90);
        assert_eq!(parse_time_string("2h15m").unwrap(), 135);
        assert!(parse_time_string("invalid").is_err());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(60), "1h");
        assert_eq!(format_duration(30), "30m");
        assert_eq!(format_duration(90), "1h 30m");
        assert_eq!(format_duration(135), "2h 15m");
    }
}
