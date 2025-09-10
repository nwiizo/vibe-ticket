//! Intent-focused create command handler
//!
//! Provides a user-friendly interface for creating tickets with
//! focus on what the user wants to accomplish rather than the mechanics.

use crate::cli::output::OutputFormatter;
use crate::cli::utils;
use crate::core::{Priority, Status, TicketBuilder};
use crate::error::Result;
use crate::interactive::{InteractiveMode, InteractiveTicketData};
use crate::storage::{FileStorage, TicketRepository};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::env;

/// Parameters for creating a ticket
pub struct CreateParams {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<String>,
    pub template: Option<String>,
    pub interactive: bool,
    pub quick: bool,
    pub project_dir: Option<String>,
}

/// Handle the intent-focused create command
///
/// This provides multiple ways to create a ticket:
/// 1. Interactive mode with templates
/// 2. Quick mode with minimal prompts
/// 3. From command line arguments
pub fn handle_create_command(params: CreateParams, formatter: &OutputFormatter) -> Result<()> {
    // Change to project directory if specified
    if let Some(ref project_path) = params.project_dir {
        env::set_current_dir(project_path)?;
    }

    let current_dir = env::current_dir()?;
    let project_root = utils::find_project_root(current_dir.to_str())?;
    let tickets_dir = project_root.join(".vibe-ticket");

    if !tickets_dir.exists() {
        return Err(crate::error::VibeTicketError::ProjectNotInitialized);
    }

    // Determine creation mode
    let ticket_data = if params.interactive || params.template.is_some() {
        // Use full interactive mode
        create_interactive(params.template)?
    } else if params.quick || (params.title.is_some() && params.description.is_some()) {
        // Quick creation with provided data
        create_quick(
            params.title,
            params.description,
            params.priority,
            params.tags,
        )?
    } else {
        // Guided creation with prompts
        create_guided()?
    };

    // Create the ticket
    let storage = FileStorage::new(tickets_dir);
    let ticket = build_ticket_from_data(ticket_data);
    storage.save(&ticket)?;

    // Success message
    formatter.success(&format!(
        "✅ Created ticket '{}' ({})",
        ticket.title, ticket.slug
    ));

    // Ask if user wants to start working on it
    let theme = ColorfulTheme::default();
    if Confirm::with_theme(&theme)
        .with_prompt("Start working on this ticket now?")
        .default(true)
        .interact()?
    {
        // Start work on the ticket
        crate::cli::handlers::start::handle_start_command(
            ticket.slug,
            true, // create_branch
            None, // branch_name
            true, // create_worktree
            params.project_dir,
            formatter,
        )?;
    }

    Ok(())
}

/// Create ticket using full interactive mode
fn create_interactive(_template: Option<String>) -> Result<InteractiveTicketData> {
    let mode = InteractiveMode::new();
    // TODO: Load specific template if provided
    mode.create_ticket()
}

/// Create ticket quickly with minimal interaction
fn create_quick(
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    tags: Option<String>,
) -> Result<InteractiveTicketData> {
    let theme = ColorfulTheme::default();

    // Get title if not provided
    let title = if let Some(t) = title {
        t
    } else {
        Input::<String>::with_theme(&theme)
            .with_prompt("Title")
            .interact()?
    };

    // Parse priority
    let priority = priority
        .unwrap_or_else(|| "medium".to_string())
        .to_lowercase();

    // Parse tags
    let tags = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    Ok(InteractiveTicketData {
        title,
        description,
        priority,
        tags,
        start_now: false,
        template_used: None,
    })
}

/// Create ticket with guided prompts (middle ground)
fn create_guided() -> Result<InteractiveTicketData> {
    let theme = ColorfulTheme::default();

    println!("🎫 Create a new ticket\n");

    // Title (required)
    let title = Input::<String>::with_theme(&theme)
        .with_prompt("What needs to be done?")
        .interact()?;

    // Description (optional but encouraged)
    let description = if Confirm::with_theme(&theme)
        .with_prompt("Add more details?")
        .default(true)
        .interact()?
    {
        Some(
            Input::<String>::with_theme(&theme)
                .with_prompt("Description")
                .allow_empty(true)
                .interact()?,
        )
    } else {
        None
    };

    // Priority (with smart default)
    let priorities = vec!["Low", "Medium", "High", "Critical"];
    let default_priority = guess_priority(&title, description.as_ref());
    let priority_index = Select::with_theme(&theme)
        .with_prompt("Priority")
        .items(&priorities)
        .default(default_priority)
        .interact()?;
    let priority = priorities[priority_index].to_lowercase();

    // Tags (optional)
    let tags_input = Input::<String>::with_theme(&theme)
        .with_prompt("Tags (comma-separated, press Enter to skip)")
        .allow_empty(true)
        .interact()?;

    let tags = if tags_input.is_empty() {
        suggest_tags(&title, description.as_ref())
    } else {
        tags_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    Ok(InteractiveTicketData {
        title,
        description,
        priority,
        tags,
        start_now: false,
        template_used: None,
    })
}

/// Build a ticket from interactive data
fn build_ticket_from_data(data: InteractiveTicketData) -> crate::core::Ticket {
    let slug = utils::generate_slug(&data.title);
    let priority = match data.priority.as_str() {
        "low" => Priority::Low,
        "high" => Priority::High,
        "critical" => Priority::Critical,
        _ => Priority::Medium,
    };

    let mut builder = TicketBuilder::new()
        .slug(slug)
        .title(data.title)
        .priority(priority)
        .status(Status::Todo)
        .tags(data.tags);

    if let Some(desc) = data.description {
        builder = builder.description(desc);
    }

    builder.build()
}

/// Guess priority based on title and description
fn guess_priority(title: &str, description: Option<&String>) -> usize {
    let text = format!(
        "{} {}",
        title.to_lowercase(),
        description.unwrap_or(&String::new()).to_lowercase()
    );

    match () {
        () if text.contains("critical") || text.contains("urgent") || text.contains("asap") => 3,
        () if text.contains("bug") || text.contains("error") || text.contains("broken") => 2,
        () if text.contains("minor") || text.contains("typo") || text.contains("cleanup") => 0,
        () => 1,
    }
}

/// Suggest tags based on title and description
fn suggest_tags(title: &str, description: Option<&String>) -> Vec<String> {
    let text = format!(
        "{} {}",
        title.to_lowercase(),
        description.unwrap_or(&String::new()).to_lowercase()
    );

    let mut tags = Vec::new();

    // Detect type
    if text.contains("bug") || text.contains("fix") || text.contains("error") {
        tags.push("bug".to_string());
    } else if text.contains("feature") || text.contains("add") || text.contains("implement") {
        tags.push("feature".to_string());
    } else if text.contains("refactor") || text.contains("cleanup") || text.contains("improve") {
        tags.push("refactor".to_string());
    } else if text.contains("doc") || text.contains("readme") {
        tags.push("documentation".to_string());
    } else if text.contains("test") {
        tags.push("testing".to_string());
    }

    // Detect area
    if text.contains("ui") || text.contains("frontend") || text.contains("css") {
        tags.push("frontend".to_string());
    } else if text.contains("api") || text.contains("backend") || text.contains("server") {
        tags.push("backend".to_string());
    } else if text.contains("database") || text.contains("sql") || text.contains("migration") {
        tags.push("database".to_string());
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_priority() {
        assert_eq!(guess_priority("Fix critical bug", None), 3);
        assert_eq!(guess_priority("Add new feature", None), 1);
        assert_eq!(guess_priority("Fix typo", None), 0);
        let urgent_desc = "This is urgent!".to_string();
        assert_eq!(guess_priority("Normal task", Some(&urgent_desc)), 3);
    }

    #[test]
    fn test_suggest_tags() {
        let tags = suggest_tags("Fix login bug", None);
        assert!(tags.contains(&"bug".to_string()));

        let tags = suggest_tags("Add new feature to API", None);
        assert!(tags.contains(&"feature".to_string()));
        assert!(tags.contains(&"backend".to_string()));

        let tags = suggest_tags("Update README documentation", None);
        assert!(tags.contains(&"documentation".to_string()));
    }
}
