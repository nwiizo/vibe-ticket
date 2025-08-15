//! Handler for the `init` command
//!
//! This module implements the initialization of a new vibe-ticket project,
//! creating the necessary directory structure and configuration files.

use crate::cli::output::OutputFormatter;
use crate::config::Config;
use crate::error::{ErrorContext, Result, VibeTicketError};
use crate::storage::{FileStorage, ProjectState};
use std::env;
use std::fs;
use std::path::Path;

/// Handle the init command
///
/// Initializes a new vibe-ticket project in the current directory by:
/// 1. Creating the `.vibe-ticket` directory structure
/// 2. Generating default configuration
/// 3. Setting up the storage repository
/// 4. Creating necessary subdirectories
///
/// # Arguments
///
/// * `name` - Optional project name (defaults to current directory name)
/// * `description` - Optional project description
/// * `force` - Force initialization even if already initialized
/// * `formatter` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is already initialized (unless `force` is true)
/// - File system operations fail
/// - Configuration cannot be saved
///
/// # Example
///
/// ```ignore
/// use vibe_ticket::cli::handlers::init::handle_init;
/// use vibe_ticket::cli::output::OutputFormatter;
///
/// let formatter = OutputFormatter::new(false, false);
/// handle_init(Some("my-project"), None, false, false, &formatter)?;
/// ```
pub fn handle_init(
    name: Option<&str>,
    description: Option<&str>,
    force: bool,
    claude_md: bool,
    formatter: &OutputFormatter,
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    // Check if already initialized
    if project_dir.exists() && !force {
        return Err(VibeTicketError::ProjectAlreadyInitialized { path: project_dir });
    }

    // Determine project name
    let project_name = name.map_or_else(
        || {
            current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("vibe-ticket-project")
                .to_string()
        },
        ToString::to_string,
    );

    // Start progress
    let progress = formatter.progress_bar("Initializing project");
    progress.set_message("Creating directory structure");

    // Create directory structure
    create_directory_structure(&project_dir)?;

    progress.set_message("Creating configuration");

    // Create default configuration
    let mut config = Config::default();
    config.project.name.clone_from(&project_name);
    config.project.description = description.map(ToString::to_string);

    // Save configuration
    let config_path = project_dir.join("config.yaml");
    let config_content =
        serde_yaml::to_string(&config).context("Failed to serialize configuration")?;
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write config to {}", config_path.display()))?;

    progress.set_message("Initializing repository");

    // Initialize storage with project state
    let storage = FileStorage::new(&project_dir);
    let project_state = ProjectState {
        name: project_name.clone(),
        description: description.map(ToString::to_string),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        ticket_count: 0,
    };
    storage.save_state(&project_state)?;

    progress.set_message("Creating default templates");

    // Create default templates
    create_default_templates(&project_dir)?;

    // Create .gitignore if it doesn't exist
    create_gitignore(&current_dir)?;

    progress.finish_with_message("Project initialized successfully");

    // Generate CLAUDE.md if requested
    if claude_md {
        progress.set_message("Generating CLAUDE.md");
        generate_claude_md_for_init(&current_dir, &project_name, description)?;
    }

    progress.finish_with_message("Project initialized successfully");

    // Display success message
    formatter.success(&format!("Initialized vibe-ticket project '{project_name}'"));

    if formatter.is_json() {
        formatter.json(&serde_json::json!({
            "status": "success",
            "project_name": project_name,
            "project_path": current_dir,
            "config_path": config_path,
            "description": description,
            "claude_md": claude_md,
        }))?;
    } else {
        formatter.info(&format!("Project directory: {}", current_dir.display()));
        if let Some(desc) = &description {
            formatter.info(&format!("Description: {desc}"));
        }
        if claude_md {
            formatter.info("Generated CLAUDE.md for AI assistance");
        }
        formatter.info("\nNext steps:");
        formatter.info("  1. Create your first ticket: vibe-ticket new <slug>");
        formatter.info("  2. List tickets: vibe-ticket list");
        formatter.info("  3. Start working: vibe-ticket start <ticket>");
    }

    Ok(())
}

/// Create the vibe-ticket directory structure
///
/// Creates all necessary subdirectories for the project:
/// - `.vibe-ticket/` - Main project directory
/// - `.vibe-ticket/tickets/` - Ticket storage
/// - `.vibe-ticket/templates/` - Custom templates
/// - `.vibe-ticket/plugins/` - Plugin directory
/// - `.vibe-ticket/backups/` - Backup directory
fn create_directory_structure(project_dir: &Path) -> Result<()> {
    let directories = [
        project_dir,
        &project_dir.join("tickets"),
        &project_dir.join("templates"),
        &project_dir.join("plugins"),
        &project_dir.join("backups"),
    ];

    for dir in directories {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory {}", dir.display()))?;
    }

    Ok(())
}

/// Create default templates
///
/// Creates starter templates for tickets and other documents
fn create_default_templates(project_dir: &Path) -> Result<()> {
    let templates_dir = project_dir.join("templates");

    // Default ticket template
    let ticket_template = r"# {{ title }}

## Description
{{ description }}

## Acceptance Criteria
- [ ] 

## Technical Notes


## Related Tickets
- 

---
Created: {{ created_at }}
Status: {{ status }}
Priority: {{ priority }}
";

    fs::write(templates_dir.join("ticket.md"), ticket_template)
        .context("Failed to create ticket template")?;

    // Default PR template
    let pr_template = r"## Summary
{{ summary }}

## Related Ticket
Closes #{{ ticket_id }} - {{ ticket_title }}

## Changes
- 

## Testing
- [ ] Unit tests pass
- [ ] Manual testing completed
- [ ] Documentation updated

## Screenshots (if applicable)

";

    fs::write(templates_dir.join("pull_request.md"), pr_template)
        .context("Failed to create PR template")?;

    Ok(())
}

/// Create or update .gitignore file
///
/// Adds vibe-ticket specific entries to .gitignore
fn create_gitignore(project_dir: &Path) -> Result<()> {
    let gitignore_path = project_dir.join(".gitignore");
    let vibe_entries = [
        "# vibe-ticket",
        ".vibe-ticket/backups/",
        ".vibe-ticket/tmp/",
        ".vibe-ticket/*.log",
        "",
        "# vibe-ticket worktree directories",
        "*-vibeticket-*/",
        "",
    ];

    if gitignore_path.exists() {
        // Read existing content
        let mut content =
            fs::read_to_string(&gitignore_path).context("Failed to read .gitignore")?;

        // Check if vibe-ticket entries already exist
        if !content.contains("# vibe-ticket") {
            // Append our entries
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&vibe_entries.join("\n"));

            fs::write(&gitignore_path, content).context("Failed to update .gitignore")?;
        }
    } else {
        // Create new .gitignore
        fs::write(&gitignore_path, vibe_entries.join("\n"))
            .context("Failed to create .gitignore")?;
    }

    Ok(())
}

/// Generate CLAUDE.md for a newly initialized project
fn generate_claude_md_for_init(
    project_dir: &Path,
    project_name: &str,
    description: Option<&str>,
) -> Result<()> {
    let claude_content = format!(
        r#"# vibe-ticket Project: {}

{}

## Overview

This project uses vibe-ticket for ticket management. This document provides guidance for Claude Code when working with this codebase.

## Common vibe-ticket Commands

### Getting Started
```bash
# Create your first ticket
vibe-ticket new fix-bug --title "Fix login issue" --priority high

# List all tickets
vibe-ticket list

# Start working on a ticket
vibe-ticket start fix-bug

# Show current status
vibe-ticket check
```

### Working with Tickets
```bash
# Show ticket details
vibe-ticket show <ticket>

# Update ticket
vibe-ticket edit <ticket> --status review

# Add tasks to ticket
vibe-ticket task add "Write unit tests"
vibe-ticket task add "Update documentation"

# Complete tasks
vibe-ticket task complete 1

# Close ticket
vibe-ticket close <ticket> --message "Fixed the login issue"
```

### Search and Filter
```bash
# Search tickets
vibe-ticket search "login"

# Filter by status
vibe-ticket list --status doing

# Filter by priority
vibe-ticket list --priority high
```

### Configuration
```bash
# View configuration
vibe-ticket config show

# Set configuration values
vibe-ticket config set project.default_priority medium
vibe-ticket config set git.auto_branch true

# Generate this file
vibe-ticket config claude
```

## Project Configuration

The project has been initialized with default settings. You can customize them using the config commands above.

## Workflow Guidelines

1. Create a ticket before starting any work
2. Use descriptive ticket slugs (e.g., fix-login-bug, add-search-feature)
3. Break down complex work into tasks within tickets
4. Keep ticket status updated as work progresses
5. Close tickets with meaningful completion messages

## Best Practices for This Project

- Follow the established ticket naming conventions
- Use appropriate priority levels (low, medium, high, critical)
- Tag tickets for better organization
- Document decisions in ticket descriptions
- Link related tickets when applicable

## Tips for Claude Code

When helping with this project:
1. Always check for active tickets before suggesting new work
2. Reference ticket IDs in commit messages
3. Update ticket status as implementation progresses
4. Use `vibe-ticket check` to understand current context
5. Generate new tickets for bugs or features discovered during development

---
Generated on: {}
"#,
        project_name,
        description.unwrap_or("A vibe-ticket managed project"),
        chrono::Local::now().format("%Y-%m-%d")
    );

    let claude_path = project_dir.join("CLAUDE.md");
    fs::write(&claude_path, &claude_content)?;

    // Also update the generated CLAUDE.md to mention the init command
    let additional_content = r"

## Project Initialization

This project was initialized with:
```bash
vibe-ticket init --claude-md
```

To regenerate or update this file:
```bash
# Regenerate with basic template
vibe-ticket config claude

# Append with advanced features
vibe-ticket config claude --template advanced --append
```
"
    .to_string();

    let full_content = format!("{claude_content}{additional_content}");
    fs::write(&claude_path, full_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".vibe-ticket");

        create_directory_structure(&project_dir).unwrap();

        assert!(project_dir.exists());
        assert!(project_dir.join("tickets").exists());
        assert!(project_dir.join("templates").exists());
        assert!(project_dir.join("plugins").exists());
        assert!(project_dir.join("backups").exists());
    }

    #[test]
    fn test_create_default_templates() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".vibe-ticket");
        create_directory_structure(&project_dir).unwrap();

        create_default_templates(&project_dir).unwrap();

        assert!(project_dir.join("templates/ticket.md").exists());
        assert!(project_dir.join("templates/pull_request.md").exists());
    }

    #[test]
    fn test_create_gitignore() {
        let temp_dir = TempDir::new().unwrap();

        create_gitignore(temp_dir.path()).unwrap();

        let gitignore_path = temp_dir.path().join(".gitignore");
        assert!(gitignore_path.exists());

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("# vibe-ticket"));
        assert!(content.contains(".vibe-ticket/backups/"));
    }

    #[test]
    fn test_generate_claude_md_for_init() {
        let temp_dir = TempDir::new().unwrap();

        generate_claude_md_for_init(temp_dir.path(), "Test Project", Some("Test description"))
            .unwrap();

        let claude_path = temp_dir.path().join("CLAUDE.md");
        assert!(claude_path.exists());

        let content = fs::read_to_string(&claude_path).unwrap();
        assert!(content.contains("# vibe-ticket Project: Test Project"));
        assert!(content.contains("Test description"));
        assert!(content.contains("## Common vibe-ticket Commands"));
    }
}
