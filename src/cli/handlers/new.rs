use crate::cli::{OutputFormatter, find_project_root, validate_slug};
use crate::core::{Priority, Ticket};
use crate::error::{Result, VibeTicketError};
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};

use super::parse_tags;

/// Handler for the `new` command
#[allow(clippy::too_many_arguments)]
pub fn handle_new_command(
    slug: &str,
    title: Option<String>,
    description: Option<String>,
    priority: &str,
    tags: Option<String>,
    start: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    // Ensure project is initialized
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");

    // Initialize storage
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Generate timestamp prefix for the slug
    let now = chrono::Local::now();
    let timestamp_prefix = now.format("%Y%m%d%H%M").to_string();

    // Validate and normalize the slug
    let base_slug = slug.trim();
    validate_slug(base_slug)?;

    // Combine timestamp and slug
    let slug = format!("{timestamp_prefix}-{base_slug}");

    // Check if a ticket with this slug already exists
    if storage.ticket_exists_with_slug(&slug)? {
        return Err(VibeTicketError::DuplicateTicket { slug });
    }

    // Parse priority
    let priority = Priority::try_from(priority).map_err(|_| VibeTicketError::InvalidPriority {
        priority: priority.to_string(),
    })?;

    // Parse tags
    let tags = tags.map(|t| parse_tags(Some(t))).unwrap_or_default();

    // Create title from base slug if not provided
    let title = title.unwrap_or_else(|| {
        base_slug
            .split('-')
            .map(|word| {
                let mut chars = word.chars();
                chars.next().map_or_else(String::new, |first| {
                    first.to_uppercase().collect::<String>() + chars.as_str()
                })
            })
            .collect::<Vec<_>>()
            .join(" ")
    });

    // Create the ticket
    let mut ticket = Ticket::new(&slug, &title);
    ticket.description = description.unwrap_or_default();
    ticket.priority = priority;
    ticket.tags = tags;

    // Save the ticket
    storage.save(&ticket)?;

    // Notify MCP about ticket creation
    #[cfg(feature = "mcp")]
    crate::integration::notify_ticket_created(&ticket);

    // If --start flag is provided, start working on the ticket immediately
    if start {
        #[cfg(feature = "mcp")]
        let old_status = ticket.status;
        
        ticket.start();
        storage.save(&ticket)?;
        storage.set_active(&ticket.id)?;

        // Notify MCP about status change
        #[cfg(feature = "mcp")]
        crate::integration::notify_status_changed(&ticket.id, old_status, ticket.status);

        if output.is_json() {
            output.print_json(&serde_json::json!({
                "success": true,
                "message": "Created and started ticket",
                "ticket": ticket,
            }))?;
        } else {
            output.success(&format!(
                "Created ticket '{}' (ID: {})",
                ticket.slug,
                ticket.id.short()
            ));
            output.info(&format!("Started working on ticket '{}'", ticket.slug));

            // TODO: Create Git branch when Git integration is implemented
            output.info("Note: Git branch creation will be available in future version");
        }
    } else if output.is_json() {
        output.print_json(&serde_json::json!({
            "success": true,
            "message": "Created ticket",
            "ticket": ticket,
        }))?;
    } else {
        output.success(&format!(
            "Created ticket '{}' (ID: {})",
            ticket.slug,
            ticket.id.short()
        ));
        output.info(&format!("Title: {}", ticket.title));
        output.info(&format!("Priority: {}", ticket.priority));
        if !ticket.tags.is_empty() {
            output.info(&format!("Tags: {}", ticket.tags.join(", ")));
        }
        output.info("");
        output.info("To start working on this ticket:");
        output.info(&format!("  vibe-ticket start {}", ticket.slug));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_ticket_from_slug() {
        let temp_dir = TempDir::new().unwrap();
        let vibe_ticket_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&vibe_ticket_dir).unwrap();

        // Initialize project state
        let state = crate::storage::ProjectState {
            name: "Test Project".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            ticket_count: 0,
        };

        let storage = FileStorage::new(&vibe_ticket_dir);
        storage.save_state(&state).unwrap();
        storage.ensure_directories().unwrap();

        // Create output formatter
        let output = OutputFormatter::new(false, false);

        // Test creating a ticket
        let result = handle_new_command(
            "fix-login-bug",
            None,
            Some("Users cannot login".to_string()),
            "high",
            Some("bug,auth".to_string()),
            false,
            Some(temp_dir.path().to_str().unwrap()),
            &output,
        );

        assert!(result.is_ok());

        // Verify ticket was created
        let tickets = storage.load_all().unwrap();
        assert_eq!(tickets.len(), 1);

        let ticket = &tickets[0];
        // Check that slug has timestamp prefix and base slug
        assert!(
            ticket.slug.ends_with("-fix-login-bug"),
            "Expected slug to end with '-fix-login-bug', got: {}",
            ticket.slug
        );
        assert_eq!(ticket.slug.len(), 26); // YYYYMMDDHHMM (12) + "-" (1) + "fix-login-bug" (13) = 26
        assert_eq!(ticket.title, "Fix Login Bug");
        assert_eq!(ticket.description, "Users cannot login");
        assert_eq!(ticket.priority, Priority::High);
        assert_eq!(ticket.tags, vec!["bug", "auth"]);
    }
}
