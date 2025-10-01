//! Handler for the `search` command
//!
//! This module implements the logic for searching tickets
//! by title, description, tags, or using regex patterns.

use crate::cli::{OutputFormatter, find_project_root};
use crate::core::Ticket;
use crate::error::Result;
use crate::storage::{FileStorage, TicketRepository};
use regex::Regex;

/// Handler for the `search` command
///
/// Searches tickets based on various criteria:
/// 1. Full text search across all fields
/// 2. Title-only search
/// 3. Description-only search
/// 4. Tags-only search
/// 5. Regex pattern matching
///
/// # Arguments
///
/// * `query` - Search query or regex pattern
/// * `title_only` - Search only in titles
/// * `description_only` - Search only in descriptions
/// * `tags_only` - Search only in tags
/// * `use_regex` - Treat query as a regex pattern
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
#[allow(clippy::fn_params_excessive_bools)]
pub fn handle_search_command(
    query: &str,
    title_only: bool,
    description_only: bool,
    tags_only: bool,
    use_regex: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    // Ensure project is initialized
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");

    // Initialize storage
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Load all tickets
    let tickets = storage.load_all()?;

    // Compile regex if needed
    let regex = if use_regex {
        Some(Regex::new(query).map_err(|e| {
            crate::error::VibeTicketError::custom(format!("Invalid regex pattern: {e}"))
        })?)
    } else {
        None
    };

    // Search tickets
    let matches = search_tickets(
        &tickets,
        query,
        title_only,
        description_only,
        tags_only,
        use_regex,
        regex.as_ref(),
    );

    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "query": query,
            "regex": use_regex,
            "search_fields": {
                "title": title_only || !description_only && !tags_only,
                "description": description_only || !title_only && !tags_only,
                "tags": tags_only || !title_only && !description_only,
            },
            "results": matches.iter().map(|(ticket, locations)| serde_json::json!({
                "id": ticket.id.to_string(),
                "slug": ticket.slug,
                "title": ticket.title,
                "status": ticket.status.to_string(),
                "priority": ticket.priority.to_string(),
                "matched_in": locations,
            })).collect::<Vec<_>>(),
            "total": matches.len(),
        }))?;
    } else if matches.is_empty() {
        output.info(&format!("No tickets found matching '{query}'"));
    } else {
        output.success(&format!(
            "Found {} ticket{} matching '{}'",
            matches.len(),
            if matches.len() == 1 { "" } else { "s" },
            query
        ));
        output.info("");

        for (ticket, locations) in &matches {
            let status_emoji = match ticket.status {
                crate::core::Status::Todo => "📋",
                crate::core::Status::Doing => "🔄",
                crate::core::Status::Review => "👀",
                crate::core::Status::Blocked => "🚫",
                crate::core::Status::Done => "✅",
            };

            output.info(&format!(
                "{} {} - {}",
                status_emoji, ticket.slug, ticket.title
            ));
            output.info(&format!(
                "   Priority: {} | Status: {} | Matched in: {}",
                ticket.priority,
                ticket.status,
                locations.join(", ")
            ));

            // Show matching context for description
            if locations.contains(&"description".to_string()) && !description_only {
                let excerpt =
                    get_match_excerpt(&ticket.description, query, use_regex, regex.as_ref());
                if let Some(excerpt) = excerpt {
                    output.info(&format!("   Description: ...{excerpt}..."));
                }
            }

            // Show matching tags
            if locations.contains(&"tags".to_string()) && !tags_only && !ticket.tags.is_empty() {
                output.info(&format!("   Tags: {}", ticket.tags.join(", ")));
            }

            output.info("");
        }
    }

    Ok(())
}

/// Search tickets for matches
///
/// # Panics
///
/// Panics if `use_regex` is true but `regex` is None
#[allow(clippy::fn_params_excessive_bools)]
fn search_tickets(
    tickets: &[Ticket],
    query: &str,
    title_only: bool,
    description_only: bool,
    tags_only: bool,
    use_regex: bool,
    regex: Option<&Regex>,
) -> Vec<(Ticket, Vec<String>)> {
    let mut matches = Vec::new();

    for ticket in tickets {
        let match_locations = if use_regex {
            search_ticket_regex(
                ticket,
                title_only,
                description_only,
                tags_only,
                regex.unwrap(),
            )
        } else {
            search_ticket_text(ticket, query, title_only, description_only, tags_only)
        };

        if !match_locations.is_empty() {
            matches.push((ticket.clone(), match_locations));
        }
    }

    // Sort matches by creation date (newest first)
    matches.sort_by(|a, b| b.0.created_at.cmp(&a.0.created_at));
    matches
}

/// Search ticket with regex
fn search_ticket_regex(
    ticket: &Ticket,
    title_only: bool,
    description_only: bool,
    tags_only: bool,
    regex: &Regex,
) -> Vec<String> {
    let mut match_locations = Vec::new();

    if !title_only && !description_only && !tags_only {
        // Search all fields
        if regex.is_match(&ticket.title) {
            match_locations.push("title".to_string());
        }
        if regex.is_match(&ticket.description) {
            match_locations.push("description".to_string());
        }
        if ticket.tags.iter().any(|tag| regex.is_match(tag)) {
            match_locations.push("tags".to_string());
        }
    } else {
        // Search specific fields
        if title_only && regex.is_match(&ticket.title) {
            match_locations.push("title".to_string());
        }
        if description_only && regex.is_match(&ticket.description) {
            match_locations.push("description".to_string());
        }
        if tags_only && ticket.tags.iter().any(|tag| regex.is_match(tag)) {
            match_locations.push("tags".to_string());
        }
    }

    match_locations
}

/// Search ticket with text
fn search_ticket_text(
    ticket: &Ticket,
    query: &str,
    title_only: bool,
    description_only: bool,
    tags_only: bool,
) -> Vec<String> {
    let mut match_locations = Vec::new();
    let query_lower = query.to_lowercase();

    if !title_only && !description_only && !tags_only {
        // Search all fields
        if ticket.title.to_lowercase().contains(&query_lower) {
            match_locations.push("title".to_string());
        }
        if ticket.description.to_lowercase().contains(&query_lower) {
            match_locations.push("description".to_string());
        }
        if ticket
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query_lower))
        {
            match_locations.push("tags".to_string());
        }
    } else {
        // Search specific fields
        if title_only && ticket.title.to_lowercase().contains(&query_lower) {
            match_locations.push("title".to_string());
        }
        if description_only && ticket.description.to_lowercase().contains(&query_lower) {
            match_locations.push("description".to_string());
        }
        if tags_only
            && ticket
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&query_lower))
        {
            match_locations.push("tags".to_string());
        }
    }

    match_locations
}

/// Extract a short excerpt around the match
fn get_match_excerpt(
    text: &str,
    query: &str,
    use_regex: bool,
    regex: Option<&Regex>,
) -> Option<String> {
    const CONTEXT_CHARS: usize = 30;

    let match_pos = if use_regex {
        regex?.find(text).map(|m| m.start())
    } else {
        text.to_lowercase().find(&query.to_lowercase())
    };

    match_pos.map(|pos| {
        let start = pos.saturating_sub(CONTEXT_CHARS);
        let end = (pos + query.len() + CONTEXT_CHARS).min(text.len());

        let excerpt = &text[start..end];
        excerpt.replace('\n', " ").trim().to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_excerpt() {
        let text = "This is a long text with some interesting content in the middle of it.";
        let query = "interesting";
        let excerpt = get_match_excerpt(text, query, false, None);
        assert!(excerpt.is_some());
        assert!(excerpt.unwrap().contains("interesting"));
    }

    #[test]
    fn test_regex_validation() {
        assert!(Regex::new("test.*pattern").is_ok());
        // Test valid regex patterns work
        assert!(Regex::new(r"test\d+").is_ok());
    }
}
