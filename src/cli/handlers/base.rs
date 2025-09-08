//! Base handler utilities for common operations
//!
//! This module provides shared functionality used across all command handlers
//! to reduce code duplication and ensure consistency.

use crate::cli::output::OutputFormatter;
use crate::cli::utils;
use crate::core::{Ticket, TicketId};
use crate::error::{Result, VibeTicketError};
use crate::storage::{FileStorage, TicketRepository};
use std::env;
use std::path::PathBuf;

/// Context for handler operations
///
/// Encapsulates common initialization and resources needed by handlers
pub struct HandlerContext {
    pub project_root: PathBuf,
    pub tickets_dir: PathBuf,
    pub storage: FileStorage,
    pub formatter: OutputFormatter,
}

impl HandlerContext {
    /// Create a new handler context
    ///
    /// Handles project directory resolution, storage initialization,
    /// and common error checking.
    pub fn new(project_dir: Option<&str>, formatter: OutputFormatter) -> Result<Self> {
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

        Ok(Self {
            project_root,
            tickets_dir,
            storage,
            formatter,
        })
    }

    /// Get the currently active ticket ID
    pub fn get_active_ticket_id(&self) -> Result<TicketId> {
        let active_ticket_path = self.tickets_dir.join("active_ticket");
        
        if !active_ticket_path.exists() {
            return Err(VibeTicketError::Custom(
                "No active ticket. Use 'vibe-ticket work-on' to select a ticket.".to_string()
            ));
        }

        let content = std::fs::read_to_string(&active_ticket_path)?;
        let id_str = content.trim();
        
        TicketId::parse_str(id_str)
            .map_err(|_| VibeTicketError::Custom(format!("Invalid active ticket ID: {}", id_str)))
    }

    /// Set the active ticket
    pub fn set_active_ticket(&self, ticket_id: &TicketId) -> Result<()> {
        let active_ticket_path = self.tickets_dir.join("active_ticket");
        std::fs::write(&active_ticket_path, ticket_id.to_string())?;
        Ok(())
    }

    /// Clear the active ticket
    pub fn clear_active_ticket(&self) -> Result<()> {
        let active_ticket_path = self.tickets_dir.join("active_ticket");
        if active_ticket_path.exists() {
            std::fs::remove_file(&active_ticket_path)?;
        }
        Ok(())
    }

    /// Resolve a ticket reference (ID, slug, or active)
    pub fn resolve_ticket_ref(&self, ticket_ref: Option<&str>) -> Result<TicketId> {
        if let Some(ref_str) = ticket_ref {
            // Try to parse as ID first
            if let Ok(id) = TicketId::parse_str(ref_str) {
                return Ok(id);
            }

            // Try to find by slug
            let tickets = self.storage.load_all()?;
            for ticket in tickets {
                if ticket.slug == ref_str {
                    return Ok(ticket.id);
                }
            }

            Err(VibeTicketError::Custom(format!(
                "Ticket not found: {}",
                ref_str
            )))
        } else {
            // Get active ticket
            self.get_active_ticket_id()
        }
    }

    /// Load a ticket by reference
    pub fn load_ticket_by_ref(&self, ticket_ref: Option<&str>) -> Result<Ticket> {
        let ticket_id = self.resolve_ticket_ref(ticket_ref)?;
        self.storage.load(&ticket_id)
    }

    /// Display formatted success message
    pub fn success(&self, message: &str) {
        self.formatter.success(message);
    }

    /// Display formatted error message
    pub fn error(&self, message: &str) {
        self.formatter.error(message);
    }

    /// Display formatted info message
    pub fn info(&self, message: &str) {
        self.formatter.info(message);
    }

    /// Display formatted warning message
    pub fn warning(&self, message: &str) {
        self.formatter.warning(message);
    }
}

/// Common validation functions
pub mod validation {
    use crate::core::Priority;
    use crate::error::{Result, VibeTicketError};

    /// Validate and parse priority string
    pub fn parse_priority(priority: &str) -> Result<Priority> {
        match priority.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(VibeTicketError::Custom(format!(
                "Invalid priority: {}. Must be one of: low, medium, high, critical",
                priority
            ))),
        }
    }

    /// Validate ticket title
    pub fn validate_title(title: &str) -> Result<()> {
        if title.trim().is_empty() {
            return Err(VibeTicketError::Custom(
                "Ticket title cannot be empty".to_string()
            ));
        }
        if title.len() > 200 {
            return Err(VibeTicketError::Custom(
                "Ticket title cannot exceed 200 characters".to_string()
            ));
        }
        Ok(())
    }

    /// Validate tags
    pub fn parse_tags(tags_str: Option<&str>) -> Vec<String> {
        tags_str
            .map(|s| {
                s.split(',')
                    .map(|tag| tag.trim().to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_handler_context_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir(&tickets_dir).unwrap();

        let formatter = OutputFormatter::default();
        let context = HandlerContext::new(
            Some(temp_dir.path().to_str().unwrap()),
            formatter
        );

        assert!(context.is_ok());
    }

    #[test]
    fn test_priority_validation() {
        use validation::parse_priority;

        assert!(parse_priority("low").is_ok());
        assert!(parse_priority("HIGH").is_ok());
        assert!(parse_priority("invalid").is_err());
    }

    #[test]
    fn test_title_validation() {
        use validation::validate_title;

        assert!(validate_title("Valid title").is_ok());
        assert!(validate_title("").is_err());
        assert!(validate_title("   ").is_err());
        
        let long_title = "a".repeat(201);
        assert!(validate_title(&long_title).is_err());
    }

    #[test]
    fn test_parse_tags() {
        use validation::parse_tags;

        let tags = parse_tags(Some("bug, feature, urgent"));
        assert_eq!(tags, vec!["bug", "feature", "urgent"]);

        let tags = parse_tags(Some("  tag1  ,  , tag2  "));
        assert_eq!(tags, vec!["tag1", "tag2"]);

        let tags = parse_tags(None);
        assert!(tags.is_empty());
    }
}