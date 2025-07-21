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
//! ```no_run
//! use vide_ticket::cli::handlers::init::handle_init;
//! use vide_ticket::cli::output::OutputFormatter;
//!
//! let formatter = OutputFormatter::new(false, false);
//! handle_init(Some("my-project".to_string()), None, false, &formatter)?;
//! ```

mod archive;
mod check;
mod close;
mod config;
mod edit;
mod export;
mod import;
mod init;
mod list;
mod new;
mod search;
mod show;
mod spec;
mod start;
mod task;

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
pub use new::handle_new_command;
pub use search::handle_search_command;
pub use show::handle_show_command;
pub use start::handle_start_command;
pub use spec::{
    handle_spec_activate, handle_spec_approve, handle_spec_delete, handle_spec_design,
    handle_spec_init, handle_spec_list, handle_spec_requirements, handle_spec_show,
    handle_spec_status, handle_spec_tasks,
};
pub use task::{
    handle_task_add, handle_task_complete, handle_task_list, handle_task_remove,
    handle_task_uncomplete,
};

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
/// This function checks if the current directory contains a vide-ticket project
/// and returns an error if not. Many commands require an initialized project.
///
/// # Errors
///
/// Returns `VideTicketError::ProjectNotInitialized` if no project is found.
pub fn ensure_project_initialized() -> Result<()> {
    use crate::config::Config;
    use crate::error::VideTicketError;
    use std::path::Path;

    let config_path = Path::new(".vide-ticket/config.yaml");
    if !config_path.exists() {
        return Err(VideTicketError::ProjectNotInitialized);
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
/// Returns `VideTicketError::NoActiveTicket` if no ticket is active.
pub fn get_active_ticket() -> Result<String> {
    use crate::error::VideTicketError;
    use crate::storage::FileStorage;

    ensure_project_initialized()?;

    let storage = FileStorage::new(".vide-ticket");
    if let Some(ticket_id) = storage.get_active_ticket()? {
        Ok(ticket_id.to_string())
    } else {
        Err(VideTicketError::NoActiveTicket)
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
            let storage = FileStorage::new(".vide-ticket");

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

            Err(crate::error::VideTicketError::TicketNotFound { id: ref_str })
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

/// Validate a slug format
///
/// Ensures the slug contains only lowercase letters, numbers, and hyphens.
///
/// # Errors
///
/// Returns `VideTicketError::InvalidSlug` if the slug format is invalid.
pub fn validate_slug(slug: &str) -> Result<()> {
    use crate::error::VideTicketError;

    if slug.is_empty() {
        return Err(VideTicketError::InvalidSlug {
            slug: slug.to_string(),
        });
    }

    let valid = slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !valid || slug.starts_with('-') || slug.ends_with('-') || slug.contains("--") {
        return Err(VideTicketError::InvalidSlug {
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
        assert_eq!(parse_tags(Some("".to_string())), Vec::<String>::new());
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
