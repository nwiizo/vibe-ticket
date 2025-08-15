//! Command handlers for the CLI
//!
//! This module contains the implementation of all CLI command handlers.
//! Each command has its own handler module that encapsulates the business logic
//! for executing that specific command.
//!
//! # Architecture
//!
//! Handlers follow a consistent pattern:
//! - Each handler receives parsed command arguments
//! - Handlers interact with the storage layer to perform operations
//! - Results are formatted and displayed using the output module
//! - Errors are properly propagated with context
//!
//! # Example
//!
//! Handlers are typically called from the main CLI entry point and handle
//! specific commands like `init`, `new`, `list`, etc.

mod archive;
mod check;
mod close;
mod common;
mod config;
mod edit;
mod export;
mod import;
mod init;
mod list;
#[cfg(feature = "mcp")]
mod mcp;
mod new;
mod search;
mod show;
mod spec;
mod spec_common;
mod start;
mod task;
mod worktree;

// Re-export handlers
pub use archive::handle_archive_command;
pub use check::handle_check_command;
pub use close::handle_close_command;
pub use config::handle_config_command;
pub use edit::handle_edit_command;
pub use export::handle_export_command;
pub use import::handle_import_command;
pub use init::handle_init;
pub use list::handle_list_command;
#[cfg(feature = "mcp")]
pub use mcp::handle_mcp_serve;
pub use new::handle_new_command;
pub use search::handle_search_command;
pub use show::handle_show_command;
pub use spec::{
    handle_spec_activate, handle_spec_approve, handle_spec_delete, handle_spec_design,
    handle_spec_init, handle_spec_list, handle_spec_requirements, handle_spec_show,
    handle_spec_status, handle_spec_tasks,
};
pub use start::handle_start_command;
pub use task::{
    handle_task_add, handle_task_complete, handle_task_list, handle_task_remove,
    handle_task_uncomplete,
};
pub use worktree::{handle_worktree_list, handle_worktree_prune, handle_worktree_remove};

use crate::cli::output::OutputFormatter;
use crate::error::Result;

/// Common trait for command handlers
///
/// This trait provides a consistent interface for all command handlers,
/// ensuring they follow the same pattern for execution and error handling.
pub trait CommandHandler {
    /// Execute the command with the given formatter
    fn execute(&self, formatter: &OutputFormatter) -> Result<()>;
}

/// Helper function to ensure a project is initialized
///
/// This function checks if the current directory contains a vibe-ticket project
/// and returns an error if not. Many commands require an initialized project.
///
/// # Errors
///
/// Returns `VibeTicketError::ProjectNotInitialized` if no project is found.
pub fn ensure_project_initialized() -> Result<()> {
    use crate::config::Config;
    use crate::error::VibeTicketError;
    use std::path::Path;

    let config_path = Path::new(".vibe-ticket/config.yaml");
    if !config_path.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    // Try to load config to ensure it's valid
    Config::load_or_default()?;

    Ok(())
}

/// Helper function to get the active ticket ID
///
/// Returns the ID of the currently active ticket, if any.
///
/// # Errors
///
/// Returns `VibeTicketError::NoActiveTicket` if no ticket is active.
pub fn get_active_ticket() -> Result<String> {
    use crate::error::VibeTicketError;
    use crate::storage::FileStorage;

    ensure_project_initialized()?;

    let storage = FileStorage::new(".vibe-ticket");
    if let Some(ticket_id) = storage.get_active_ticket()? {
        Ok(ticket_id.to_string())
    } else {
        Err(VibeTicketError::NoActiveTicket)
    }
}

/// Helper function to resolve a ticket identifier
///
/// Takes a ticket ID or slug and returns the actual ticket ID.
/// If None is provided, returns the active ticket ID.
///
/// # Arguments
///
/// * `ticket_ref` - Optional ticket ID or slug
///
/// # Errors
///
/// Returns an error if the ticket is not found or if no active ticket exists
/// when `ticket_ref` is None.
pub fn resolve_ticket_id(ticket_ref: Option<String>) -> Result<String> {
    match ticket_ref {
        Some(ref_str) => {
            use crate::core::TicketId;
            use crate::storage::FileStorage;

            ensure_project_initialized()?;
            let storage = FileStorage::new(".vibe-ticket");

            // First try to parse as ticket ID
            if let Ok(ticket_id) = TicketId::parse_str(&ref_str) {
                // Try to load the ticket to verify it exists
                if storage.load_ticket(&ticket_id).is_ok() {
                    return Ok(ticket_id.to_string());
                }
            }

            // Then try to find by slug
            if let Some(ticket) = storage.find_ticket_by_slug(&ref_str)? {
                return Ok(ticket.id.to_string());
            }

            Err(crate::error::VibeTicketError::TicketNotFound { id: ref_str })
        },
        None => get_active_ticket(),
    }
}

/// Format tags from a comma-separated string
///
/// Takes a string of comma-separated tags and returns a vector of trimmed tags.
///
/// # Example
///
/// ```
/// use vibe_ticket::cli::handlers::parse_tags;
///
/// let tags = parse_tags(Some("bug, ui, urgent".to_string()));
/// assert_eq!(tags, vec!["bug", "ui", "urgent"]);
/// ```
pub fn parse_tags(tags_str: Option<String>) -> Vec<String> {
    tags_str
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Resolve a ticket reference (ID, partial ID, or slug) to a ticket ID
///
/// This function attempts to find a ticket by:
/// 1. Full UUID match
/// 2. Exact slug match
/// 3. Partial UUID match (prefix)
///
/// # Arguments
///
/// * `storage` - The storage instance to use
/// * `ticket_ref` - The ticket reference (UUID, partial UUID, or slug)
///
/// # Errors
///
/// Returns an error if:
/// - No ticket is found
/// - Multiple tickets match a partial UUID
pub fn resolve_ticket_ref(
    storage: &crate::storage::FileStorage,
    ticket_ref: &str,
) -> Result<crate::core::TicketId> {
    use crate::core::TicketId;
    use crate::error::VibeTicketError;
    use crate::storage::TicketRepository;

    // First try to parse as full ticket ID
    if let Ok(ticket_id) = TicketId::parse_str(ticket_ref) {
        // Verify the ticket exists
        if storage.load(&ticket_id).is_ok() {
            return Ok(ticket_id);
        }
    }

    // Try to find by partial ID or slug
    let all_tickets = storage.load_all()?;
    let mut matches = Vec::new();

    for ticket in all_tickets {
        // Check if it matches the slug exactly
        if ticket.slug == ticket_ref {
            return Ok(ticket.id);
        }

        // Check if it's a partial ID match
        let id_str = ticket.id.to_string();
        if id_str.starts_with(ticket_ref) {
            matches.push(ticket);
        }
    }

    // If we found exactly one match with partial ID, use it
    if matches.len() == 1 {
        return Ok(matches[0].id.clone());
    } else if matches.len() > 1 {
        // Multiple matches found, return error with suggestions
        let suggestions: Vec<String> = matches
            .iter()
            .map(|t| format!("{} ({})", t.id, t.slug))
            .collect();
        return Err(VibeTicketError::custom(format!(
            "Multiple tickets found matching '{}': {}",
            ticket_ref,
            suggestions.join(", ")
        )));
    }

    Err(VibeTicketError::TicketNotFound {
        id: ticket_ref.to_string(),
    })
}

/// Validate a slug format
///
/// Ensures the slug contains only lowercase letters, numbers, and hyphens.
///
/// # Errors
///
/// Returns `VibeTicketError::InvalidSlug` if the slug format is invalid.
pub fn validate_slug(slug: &str) -> Result<()> {
    use crate::error::VibeTicketError;

    if slug.is_empty() {
        return Err(VibeTicketError::InvalidSlug {
            slug: slug.to_string(),
        });
    }

    let valid = slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !valid || slug.starts_with('-') || slug.ends_with('-') || slug.contains("--") {
        return Err(VibeTicketError::InvalidSlug {
            slug: slug.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tags() {
        assert_eq!(parse_tags(None), Vec::<String>::new());
        assert_eq!(parse_tags(Some(String::new())), Vec::<String>::new());
        assert_eq!(
            parse_tags(Some("bug, ui, urgent".to_string())),
            vec!["bug", "ui", "urgent"]
        );
        assert_eq!(
            parse_tags(Some("  bug  ,  ui  ".to_string())),
            vec!["bug", "ui"]
        );
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("fix-login-bug").is_ok());
        assert!(validate_slug("feature-123").is_ok());
        assert!(validate_slug("test").is_ok());

        assert!(validate_slug("").is_err());
        assert!(validate_slug("Fix-Login").is_err()); // uppercase
        assert!(validate_slug("-start").is_err()); // starts with hyphen
        assert!(validate_slug("end-").is_err()); // ends with hyphen
        assert!(validate_slug("double--hyphen").is_err()); // double hyphen
        assert!(validate_slug("special@char").is_err()); // special char
    }
}
