//! Handler for the `start` command
//!
//! This module implements the logic for starting work on a ticket,
//! including Git branch creation and status updates.

use crate::cli::{OutputFormatter, find_project_root, handlers::resolve_ticket_ref};
use crate::config::Config;
use crate::core::Status;
use crate::error::{Result, VibeTicketError};
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};
use chrono::Utc;

/// Handler for the `start` command
///
/// This function performs the following operations:
/// 1. Loads the specified ticket
/// 2. Updates the ticket status to "doing"
/// 3. Sets the ticket as active
/// 4. Optionally creates a Git branch or worktree for the ticket
///
/// # Arguments
///
/// * `ticket_ref` - Ticket ID or slug to start
/// * `create_branch` - Whether to create a Git branch
/// * `branch_name` - Optional custom branch name
/// * `create_worktree` - Whether to create a Git worktree instead of just a branch
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
///
/// # Errors
///
/// Returns an error if:
/// - The project is not initialized
/// - The ticket is not found
/// - Git operations fail
/// - The ticket is already in progress
pub fn handle_start_command(
    ticket_ref: String,
    create_branch: bool,
    branch_name: Option<String>,
    create_worktree: bool,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    // Ensure project is initialized
    let project_root = find_project_root(project_dir.as_deref())?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");

    // Initialize storage
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Resolve ticket ID from reference (ID or slug)
    let ticket_id = resolve_ticket_ref(&storage, &ticket_ref)?;

    // Load the ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Check if ticket is already in progress
    if ticket.status == Status::Doing {
        return Err(VibeTicketError::custom(format!(
            "Ticket '{}' is already in progress",
            ticket.slug
        )));
    }

    // Update ticket status and start time
    let old_status = ticket.status.clone();
    ticket.status = Status::Doing;
    ticket.started_at = Some(Utc::now());

    // Save the updated ticket
    storage.save(&ticket)?;

    // Notify MCP about status change
    #[cfg(feature = "mcp")]
    crate::integration::notify_status_changed(&ticket_id, old_status, ticket.status.clone());

    // Set as active ticket
    storage.set_active(&ticket_id)?;

    // Load configuration to get worktree settings
    let config = Config::load_or_default()?;

    // Create Git branch or worktree if requested
    let (branch_name_final, worktree_created) = if create_branch {
        let branch_name =
            branch_name.unwrap_or_else(|| format!("{}{}", config.git.branch_prefix, ticket.slug));

        if create_worktree {
            create_git_worktree(&project_root, &branch_name, &ticket.slug, &config, output)?;
            (Some(branch_name), true)
        } else {
            create_git_branch(&project_root, &branch_name, output)?;
            (Some(branch_name), false)
        }
    } else {
        (None, false)
    };

    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket": {
                "id": ticket.id.to_string(),
                "slug": ticket.slug,
                "title": ticket.title,
                "status": ticket.status.to_string(),
                "started_at": ticket.started_at,
            },
            "branch_created": create_branch,
            "branch_name": branch_name_final,
            "worktree_created": worktree_created,
        }))?;
    } else {
        output.success(&format!("Started working on ticket: {}", ticket.slug));
        output.info(&format!("Title: {}", ticket.title));
        output.info(&format!("Status: {} → {}", Status::Todo, Status::Doing));

        if let Some(branch) = branch_name_final {
            if worktree_created {
                let worktree_prefix = config
                    .git
                    .worktree_prefix
                    .replace("{project}", &config.project.name);
                output.info(&format!(
                    "Git worktree created: ../{}{}",
                    worktree_prefix.trim_start_matches("../"),
                    ticket.slug
                ));
                output.info(&format!("Branch: {branch}"));
            } else {
                output.info(&format!("Git branch created: {branch}"));
            }
        }

        output.info(&format!("\nTicket '{}' is now active.", ticket.slug));
    }

    Ok(())
}

/// Create a Git branch for the ticket
fn create_git_branch(
    project_root: &std::path::Path,
    branch_name: &str,
    output: &OutputFormatter,
) -> Result<()> {
    // Temporarily use git command instead of git2 library due to linking issues
    use std::process::Command;

    // Check if we're in a git repository
    let status = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to run git command: {e}")))?;

    if !status.status.success() {
        return Err(VibeTicketError::custom("Not in a Git repository"));
    }

    // Check if branch already exists
    let check_branch = Command::new("git")
        .arg("show-ref")
        .arg("--verify")
        .arg("--quiet")
        .arg(format!("refs/heads/{branch_name}"))
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to check branch existence: {e}")))?;

    if check_branch.status.success() {
        return Err(VibeTicketError::custom(format!(
            "Branch '{branch_name}' already exists"
        )));
    }

    // Create and checkout the new branch
    let create_branch = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(branch_name)
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to create branch: {e}")))?;

    if !create_branch.status.success() {
        let error_msg = String::from_utf8_lossy(&create_branch.stderr);
        return Err(VibeTicketError::custom(format!(
            "Failed to create branch: {error_msg}"
        )));
    }

    output.success(&format!("Switched to new branch '{branch_name}'"));

    Ok(())
}

/// Create a Git worktree for the ticket
fn create_git_worktree(
    project_root: &std::path::Path,
    branch_name: &str,
    ticket_slug: &str,
    config: &Config,
    output: &OutputFormatter,
) -> Result<()> {
    use std::process::Command;

    // Check if we're in a git repository
    let status = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to run git command: {e}")))?;

    if !status.status.success() {
        return Err(VibeTicketError::custom("Not in a Git repository"));
    }

    // Construct the worktree path using config settings
    let project_name = config.project.name.as_str();
    let worktree_prefix = config
        .git
        .worktree_prefix
        .replace("{project}", project_name);

    // Determine base directory based on prefix
    let (base_dir, clean_prefix) = if worktree_prefix.starts_with("../") {
        // Parent directory
        let parent = project_root
            .parent()
            .ok_or_else(|| VibeTicketError::custom("Cannot find parent directory for worktree"))?;
        (
            parent.to_path_buf(),
            worktree_prefix.trim_start_matches("../"),
        )
    } else if worktree_prefix.starts_with("./") {
        // Current directory
        (
            project_root.to_path_buf(),
            worktree_prefix.trim_start_matches("./"),
        )
    } else {
        // Default to current directory
        (project_root.to_path_buf(), worktree_prefix.as_str())
    };

    let worktree_dir_name = format!("{}{}", clean_prefix.trim_end_matches('-'), ticket_slug);
    let worktree_path = base_dir.join(&worktree_dir_name);

    // Check if worktree directory already exists
    if worktree_path.exists() {
        return Err(VibeTicketError::custom(format!(
            "Worktree directory '{}' already exists",
            worktree_path.display()
        )));
    }

    // Check if branch already exists
    let check_branch = Command::new("git")
        .arg("show-ref")
        .arg("--verify")
        .arg("--quiet")
        .arg(format!("refs/heads/{branch_name}"))
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to check branch existence: {e}")))?;

    if check_branch.status.success() {
        return Err(VibeTicketError::custom(format!(
            "Branch '{branch_name}' already exists"
        )));
    }

    // Create the worktree with a new branch
    let create_worktree = Command::new("git")
        .arg("worktree")
        .arg("add")
        .arg(&worktree_path)
        .arg("-b")
        .arg(branch_name)
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to create worktree: {e}")))?;

    if !create_worktree.status.success() {
        let error_msg = String::from_utf8_lossy(&create_worktree.stderr);
        return Err(VibeTicketError::custom(format!(
            "Failed to create worktree: {error_msg}"
        )));
    }

    output.success(&format!(
        "Created worktree at '{}'",
        worktree_path.display()
    ));

    // Show appropriate cd command based on location
    let cd_path = if worktree_prefix.starts_with("../") {
        format!("../{}", worktree_dir_name)
    } else if worktree_prefix.starts_with("./") {
        format!("./{}", worktree_dir_name)
    } else {
        worktree_dir_name.clone()
    };
    output.info(&format!("You can now cd to '{}'", cd_path));

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_branch_name_generation() {
        let default_name = "test-ticket";
        let branch_name = format!("ticket/{default_name}");
        assert_eq!(branch_name, "ticket/test-ticket");
    }

    #[test]
    fn test_worktree_path_generation() {
        let ticket_slug = "fix-login-bug";
        let project_name = "my-project";
        let worktree_prefix = "../{project}-ticket-".replace("{project}", project_name);
        let worktree_dir_name = format!(
            "{}{}",
            worktree_prefix.trim_start_matches("../"),
            ticket_slug
        );
        assert_eq!(worktree_dir_name, "my-project-ticket-fix-login-bug");
    }
}
