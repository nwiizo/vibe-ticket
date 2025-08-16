//! Handler for Git worktree management commands
//!
//! This module provides functionality to manage Git worktrees associated with tickets,
//! enabling parallel development workflows.

use crate::cli::{OutputFormatter, find_project_root};
use crate::config::Config;
use crate::error::{Result, VibeTicketError};
use crate::storage::{FileStorage, TicketRepository};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Handle the worktree list command
#[allow(clippy::needless_pass_by_value)]
pub fn handle_worktree_list(
    all: bool,
    status_filter: Option<String>,
    verbose: bool,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(None)?;
    let config = Config::load_or_default()?;

    // Get all Git worktrees
    let worktrees = list_git_worktrees(&project_root)?;

    // Load ticket information
    let storage = FileStorage::new(project_root.join(".vibe-ticket"));
    let tickets = storage.load_all()?;

    // Create a map of ticket slugs to tickets
    let ticket_map: HashMap<String, _> = tickets.into_iter().map(|t| (t.slug.clone(), t)).collect();

    // Filter and display worktrees
    let mut displayed_count = 0;
    let mut filtered_worktrees = Vec::new();

    for worktree in worktrees {
        // Extract ticket slug from worktree path
        let ticket_slug = extract_ticket_slug(&worktree.path, &config);

        // Check if this is a vibe-ticket worktree
        if !all && ticket_slug.is_none() {
            continue;
        }

        // Apply status filter if provided
        if let Some(ref filter) = status_filter {
            let status = determine_worktree_status(&worktree);
            if !status.eq_ignore_ascii_case(filter) {
                continue;
            }
        }

        // Display worktree information
        if output.is_json() {
            filtered_worktrees.push(worktree);
        } else {
            display_worktree(
                &worktree,
                ticket_slug.as_deref(),
                &ticket_map,
                verbose,
                output,
            );
        }

        displayed_count += 1;
    }

    if output.is_json() {
        output.json(&serde_json::json!({
            "worktrees": filtered_worktrees,
            "count": displayed_count,
        }))?;
    } else {
        output.info(&format!("\nTotal worktrees: {displayed_count}"));
    }

    Ok(())
}

/// Handle the worktree remove command
///
/// # Panics
///
/// Panics if branch name is None when unwrap is called
pub fn handle_worktree_remove(
    worktree_ref: &str,
    force: bool,
    keep_branch: bool,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(None)?;
    let config = Config::load_or_default()?;

    // Resolve worktree path
    let worktree_path = resolve_worktree_path(worktree_ref, &project_root, &config)?;

    // Check for uncommitted changes
    if !force {
        check_uncommitted_changes(&worktree_path)?;
    }

    // Get branch name before removal
    let branch_name = get_worktree_branch(&worktree_path)?;

    // Remove the worktree
    remove_git_worktree(&project_root, &worktree_path, force)?;

    output.success(&format!("Removed worktree: {}", worktree_path.display()));

    // Remove branch if requested
    if !keep_branch && branch_name.is_some() {
        let branch = branch_name.unwrap();
        remove_git_branch(&project_root, &branch)?;
        output.info(&format!("Removed branch: {branch}"));
    }

    Ok(())
}

/// Handle the worktree prune command
pub fn handle_worktree_prune(
    force: bool,
    dry_run: bool,
    remove_branches: bool,
    output: &OutputFormatter,
) -> Result<()> {
    let project_root = find_project_root(None)?;

    // Run git worktree prune
    let mut cmd = Command::new("git");
    cmd.arg("worktree").arg("prune").current_dir(&project_root);

    if dry_run {
        cmd.arg("--dry-run");
    }

    if !force && !dry_run {
        output.warning("This will remove stale worktree information. Use --force to confirm.");
        return Ok(());
    }

    let result = cmd
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to run git worktree prune: {e}")))?;

    if !result.status.success() {
        let error_msg = String::from_utf8_lossy(&result.stderr);
        return Err(VibeTicketError::custom(format!(
            "Failed to prune worktrees: {error_msg}"
        )));
    }

    let output_text = String::from_utf8_lossy(&result.stdout);
    if output_text.is_empty() {
        output.info("No stale worktrees found");
    } else {
        output.success(&format!("Pruned worktrees:\n{output_text}"));

        // TODO: Implement branch removal for pruned worktrees
        if remove_branches && !dry_run {
            output.warning("Branch removal not yet implemented");
        }
    }

    Ok(())
}

/// Worktree information
#[derive(Debug, Clone, serde::Serialize)]
struct WorktreeInfo {
    path: PathBuf,
    branch: Option<String>,
    commit: String,
    status: String,
}

/// List all Git worktrees
fn list_git_worktrees(project_root: &Path) -> Result<Vec<WorktreeInfo>> {
    let output = Command::new("git")
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to list worktrees: {e}")))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(VibeTicketError::custom(format!(
            "Failed to list worktrees: {error_msg}"
        )));
    }

    let output_text = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_worktree: Option<WorktreeInfo> = None;

    for line in output_text.lines() {
        if line.starts_with("worktree ") {
            if let Some(wt) = current_worktree.take() {
                worktrees.push(wt);
            }
            let path = PathBuf::from(line.strip_prefix("worktree ").unwrap());
            current_worktree = Some(WorktreeInfo {
                path,
                branch: None,
                commit: String::new(),
                status: "active".to_string(),
            });
        } else if line.starts_with("HEAD ") {
            if let Some(ref mut wt) = current_worktree {
                wt.commit = line.strip_prefix("HEAD ").unwrap().to_string();
            }
        } else if line.starts_with("branch ") {
            if let Some(ref mut wt) = current_worktree {
                wt.branch = Some(line.strip_prefix("branch ").unwrap().to_string());
            }
        } else if line.starts_with("detached") {
            if let Some(ref mut wt) = current_worktree {
                wt.status = "detached".to_string();
            }
        }
    }

    if let Some(wt) = current_worktree {
        worktrees.push(wt);
    }

    Ok(worktrees)
}

/// Extract ticket slug from worktree path
fn extract_ticket_slug(path: &Path, config: &Config) -> Option<String> {
    let _path_str = path.to_string_lossy();
    let project_name = &config.project.name;
    let prefix = config
        .git
        .worktree_prefix
        .replace("{project}", project_name);

    // Check if this follows our worktree naming pattern
    if let Some(file_name) = path.file_name() {
        let name = file_name.to_string_lossy();
        let prefix_cleaned = prefix
            .trim_start_matches("../")
            .trim_start_matches("./")
            .trim_end_matches('-');

        if name.starts_with(prefix_cleaned) {
            let slug = name
                .strip_prefix(prefix_cleaned)
                .unwrap()
                .trim_start_matches('-');
            return Some(slug.to_string());
        }
    }

    None
}

/// Determine worktree status
fn determine_worktree_status(worktree: &WorktreeInfo) -> String {
    if !worktree.path.exists() {
        "orphaned".to_string()
    } else if worktree.status == "detached" {
        "detached".to_string()
    } else {
        "active".to_string()
    }
}

/// Display worktree information
fn display_worktree(
    worktree: &WorktreeInfo,
    ticket_slug: Option<&str>,
    ticket_map: &HashMap<String, crate::core::Ticket>,
    verbose: bool,
    output: &OutputFormatter,
) {
    let status = determine_worktree_status(worktree);
    let path_display = worktree.path.display();

    if let Some(slug) = ticket_slug {
        if let Some(ticket) = ticket_map.get(slug) {
            output.info(&format!(
                "{} [{}] - {} ({})",
                path_display, status, ticket.title, ticket.status
            ));
        } else {
            output.info(&format!(
                "{path_display} [{status}] - {slug} (no ticket found)"
            ));
        }
    } else {
        output.info(&format!("{path_display} [{status}]"));
    }

    if verbose {
        if let Some(branch) = &worktree.branch {
            output.info(&format!("  Branch: {branch}"));
        }
        output.info(&format!("  Commit: {}", &worktree.commit[..8]));
    }
}

/// Resolve worktree path from reference
fn resolve_worktree_path(
    worktree_ref: &str,
    project_root: &Path,
    config: &Config,
) -> Result<PathBuf> {
    // Check if it's a direct path
    let path = Path::new(worktree_ref);
    if path.is_absolute() || path.exists() {
        return Ok(path.to_path_buf());
    }

    // Try to resolve as ticket slug
    let project_name = &config.project.name;
    let prefix = config
        .git
        .worktree_prefix
        .replace("{project}", project_name);

    // Determine base directory based on prefix
    let (base_dir, clean_prefix) = if prefix.starts_with("../") {
        // Parent directory
        let parent = project_root
            .parent()
            .ok_or_else(|| VibeTicketError::custom("Cannot find parent directory"))?;
        (parent.to_path_buf(), prefix.trim_start_matches("../"))
    } else if prefix.starts_with("./") {
        // Current directory
        (project_root.to_path_buf(), prefix.trim_start_matches("./"))
    } else {
        // Default to current directory
        (project_root.to_path_buf(), prefix.as_str())
    };

    let worktree_name = format!("{clean_prefix}{worktree_ref}");
    let worktree_path = base_dir.join(&worktree_name);
    if worktree_path.exists() {
        return Ok(worktree_path);
    }

    Err(VibeTicketError::custom(format!(
        "Worktree not found: {worktree_ref}"
    )))
}

/// Check for uncommitted changes in worktree
fn check_uncommitted_changes(worktree_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(worktree_path)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to check git status: {e}")))?;

    if !output.status.success() {
        // Might not be a git repository, which is fine
        return Ok(());
    }

    let output_text = String::from_utf8_lossy(&output.stdout);
    if !output_text.trim().is_empty() {
        return Err(VibeTicketError::custom(
            "Worktree has uncommitted changes. Use --force to remove anyway",
        ));
    }

    Ok(())
}

/// Get branch name for worktree
fn get_worktree_branch(worktree_path: &Path) -> Result<Option<String>> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(worktree_path)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to get branch name: {e}")))?;

    if !output.status.success() {
        return Ok(None);
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch == "HEAD" {
        Ok(None)
    } else {
        Ok(Some(branch))
    }
}

/// Remove a Git worktree
fn remove_git_worktree(project_root: &Path, worktree_path: &Path, force: bool) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("worktree")
        .arg("remove")
        .arg(worktree_path)
        .current_dir(project_root);

    if force {
        cmd.arg("--force");
    }

    let output = cmd
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to remove worktree: {e}")))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(VibeTicketError::custom(format!(
            "Failed to remove worktree: {error_msg}"
        )));
    }

    Ok(())
}

/// Remove a Git branch
fn remove_git_branch(project_root: &Path, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("branch")
        .arg("-d")
        .arg(branch)
        .current_dir(project_root)
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to remove branch: {e}")))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        // Try force delete if regular delete fails
        let force_output = Command::new("git")
            .arg("branch")
            .arg("-D")
            .arg(branch)
            .current_dir(project_root)
            .output()
            .map_err(|e| VibeTicketError::custom(format!("Failed to force remove branch: {e}")))?;

        if !force_output.status.success() {
            return Err(VibeTicketError::custom(format!(
                "Failed to remove branch: {error_msg}"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::output::OutputFormatter;
    use crate::config::{GitConfig, ProjectConfig};
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        Config {
            project: ProjectConfig {
                name: "test-project".to_string(),
                description: None,
                default_assignee: None,
                default_priority: "medium".to_string(),
            },
            ui: crate::config::UiConfig {
                theme: "auto".to_string(),
                emoji: true,
                page_size: 20,
                date_format: "%Y-%m-%d %H:%M".to_string(),
            },
            git: GitConfig {
                enabled: true,
                auto_branch: true,
                branch_prefix: "ticket/".to_string(),
                commit_template: None,
                worktree_enabled: true,
                worktree_default: true,
                worktree_prefix: "./{project}-vibeticket-".to_string(),
                worktree_cleanup_on_close: false,
            },
            plugins: crate::config::PluginsConfig {
                enabled: vec![],
                directory: ".vibe-ticket/plugins".to_string(),
            },
        }
    }

    #[test]
    fn test_extract_ticket_slug() {
        let config = create_test_config();

        // Test standard worktree path
        let path = PathBuf::from("./test-project-vibeticket-fix-bug");
        let slug = extract_ticket_slug(&path, &config);
        assert_eq!(slug, Some("fix-bug".to_string()));

        // Test path without prefix
        let path = PathBuf::from("./some-other-dir");
        let slug = extract_ticket_slug(&path, &config);
        assert_eq!(slug, None);

        // Test with parent directory prefix
        let mut config_parent = config.clone();
        config_parent.git.worktree_prefix = "../{project}-vibeticket-".to_string();
        let path = PathBuf::from("../test-project-vibeticket-feature");
        let slug = extract_ticket_slug(&path, &config_parent);
        assert_eq!(slug, Some("feature".to_string()));
    }

    #[test]
    fn test_determine_worktree_status() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("worktree");
        std::fs::create_dir(&path).unwrap();

        // Test active worktree
        let worktree = WorktreeInfo {
            path: path.clone(),
            branch: Some("feature".to_string()),
            commit: "abc123".to_string(),
            status: "active".to_string(),
        };
        assert_eq!(determine_worktree_status(&worktree), "active");

        // Test detached worktree
        let mut detached = worktree.clone();
        detached.status = "detached".to_string();
        assert_eq!(determine_worktree_status(&detached), "detached");

        // Test orphaned worktree (non-existent path)
        let orphaned = WorktreeInfo {
            path: PathBuf::from("/non/existent/path"),
            branch: Some("feature".to_string()),
            commit: "abc123".to_string(),
            status: "active".to_string(),
        };
        assert_eq!(determine_worktree_status(&orphaned), "orphaned");
    }

    #[test]
    fn test_resolve_worktree_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let config = create_test_config();

        // Create test worktree directory
        let worktree_name = "test-project-vibeticket-test-ticket";
        let worktree_path = project_root.join(worktree_name);
        std::fs::create_dir(&worktree_path).unwrap();

        // Test resolving by ticket slug
        let resolved = resolve_worktree_path("test-ticket", project_root, &config).unwrap();
        assert_eq!(resolved, worktree_path);

        // Test absolute path
        let resolved =
            resolve_worktree_path(worktree_path.to_str().unwrap(), project_root, &config).unwrap();
        assert_eq!(resolved, worktree_path);

        // Test non-existent worktree
        let result = resolve_worktree_path("non-existent", project_root, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_worktree_info_parsing() {
        // Test the WorktreeInfo structure
        let worktree = WorktreeInfo {
            path: PathBuf::from("/path/to/worktree"),
            branch: Some("feature/test".to_string()),
            commit: "abc123def456".to_string(),
            status: "active".to_string(),
        };

        assert_eq!(worktree.path.to_str().unwrap(), "/path/to/worktree");
        assert_eq!(worktree.branch.as_ref().unwrap(), "feature/test");
        assert_eq!(worktree.commit, "abc123def456");
        assert_eq!(worktree.status, "active");
    }

    #[test]
    fn test_display_worktree() {
        use crate::core::Ticket;
        let formatter = OutputFormatter::new(false, false);

        let worktree = WorktreeInfo {
            path: PathBuf::from("./test-worktree"),
            branch: Some("feature/test".to_string()),
            commit: "abc123def456".to_string(),
            status: "active".to_string(),
        };

        // Test with ticket
        let ticket = Ticket::new("test-ticket".to_string(), "Test Ticket".to_string());
        let mut ticket_map = HashMap::new();
        ticket_map.insert("test-ticket".to_string(), ticket);

        display_worktree(
            &worktree,
            Some("test-ticket"),
            &ticket_map,
            false,
            &formatter,
        );

        // Test without ticket
        display_worktree(&worktree, None, &ticket_map, false, &formatter);

        // Test verbose mode
        display_worktree(
            &worktree,
            Some("test-ticket"),
            &ticket_map,
            true,
            &formatter,
        );
    }

    #[test]
    fn test_check_uncommitted_changes_no_git() {
        let temp_dir = TempDir::new().unwrap();

        // Test with non-git directory (should succeed)
        let result = check_uncommitted_changes(temp_dir.path());
        assert!(result.is_ok());
    }
}
