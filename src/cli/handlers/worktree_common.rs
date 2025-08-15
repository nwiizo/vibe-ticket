use crate::cli::output::OutputFormatter;
use crate::error::{Result, VibeTicketError};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git worktree information
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_locked: bool,
    pub prunable: bool,
}

/// Common worktree operations
pub struct WorktreeOperations;

impl WorktreeOperations {
    /// List all git worktrees
    pub fn list_all() -> Result<Vec<WorktreeInfo>> {
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .output()
            .map_err(|e| VibeTicketError::GitError(format!("Failed to list worktrees: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VibeTicketError::GitError(format!("git worktree list failed: {}", error)));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_worktree_list(&stdout)
    }
    
    /// Parse git worktree list output
    fn parse_worktree_list(output: &str) -> Result<Vec<WorktreeInfo>> {
        let mut worktrees = Vec::new();
        let mut current = None;
        let mut path = PathBuf::new();
        let mut commit = String::new();
        let mut branch = String::new();
        let mut is_bare = false;
        let mut is_detached = false;
        let mut is_locked = false;
        let mut prunable = false;
        
        for line in output.lines() {
            if line.is_empty() {
                if let Some(p) = current.take() {
                    worktrees.push(WorktreeInfo {
                        path: p,
                        commit: commit.clone(),
                        branch: branch.clone(),
                        is_bare,
                        is_detached,
                        is_locked,
                        prunable,
                    });
                    // Reset for next worktree
                    commit.clear();
                    branch.clear();
                    is_bare = false;
                    is_detached = false;
                    is_locked = false;
                    prunable = false;
                }
            } else if let Some(p) = line.strip_prefix("worktree ") {
                path = PathBuf::from(p);
                current = Some(path.clone());
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                commit = h.to_string();
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                branch = b.to_string();
            } else if line == "bare" {
                is_bare = true;
            } else if line == "detached" {
                is_detached = true;
            } else if line.starts_with("locked") {
                is_locked = true;
            } else if line.starts_with("prunable") {
                prunable = true;
            }
        }
        
        // Handle last worktree if any
        if let Some(p) = current {
            worktrees.push(WorktreeInfo {
                path: p,
                commit,
                branch,
                is_bare,
                is_detached,
                is_locked,
                prunable,
            });
        }
        
        Ok(worktrees)
    }
    
    /// Remove a git worktree
    pub fn remove(path: &Path, force: bool) -> Result<()> {
        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        args.push(path.to_str().ok_or_else(|| {
            VibeTicketError::InvalidInput("Invalid worktree path".to_string())
        })?);
        
        let output = Command::new("git")
            .args(&args)
            .output()
            .map_err(|e| VibeTicketError::GitError(format!("Failed to remove worktree: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VibeTicketError::GitError(format!("git worktree remove failed: {}", error)));
        }
        
        Ok(())
    }
    
    /// Prune stale worktree entries
    pub fn prune() -> Result<()> {
        let output = Command::new("git")
            .args(["worktree", "prune"])
            .output()
            .map_err(|e| VibeTicketError::GitError(format!("Failed to prune worktrees: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VibeTicketError::GitError(format!("git worktree prune failed: {}", error)));
        }
        
        Ok(())
    }
    
    /// Check for uncommitted changes in a worktree
    pub fn has_uncommitted_changes(path: &Path) -> Result<bool> {
        let output = Command::new("git")
            .args(["-C", path.to_str().unwrap_or("."), "status", "--porcelain"])
            .output()
            .map_err(|e| VibeTicketError::GitError(format!("Failed to check git status: {}", e)))?;
        
        if !output.status.success() {
            return Ok(false); // Assume no changes if status fails
        }
        
        Ok(!output.stdout.is_empty())
    }
    
    /// Get the current branch of a worktree
    pub fn get_branch(path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["-C", path.to_str().unwrap_or("."), "branch", "--show-current"])
            .output()
            .map_err(|e| VibeTicketError::GitError(format!("Failed to get current branch: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VibeTicketError::GitError(format!("Failed to get branch: {}", error)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

/// Common output formatting for worktree operations
pub struct WorktreeFormatter;

impl WorktreeFormatter {
    /// Format worktree list for display
    pub fn format_list(worktrees: &[WorktreeInfo], formatter: &OutputFormatter) -> Result<()> {
        if formatter.is_json() {
            let json_worktrees: Vec<_> = worktrees.iter()
                .map(|w| serde_json::json!({
                    "path": w.path.display().to_string(),
                    "branch": w.branch,
                    "commit": w.commit,
                    "bare": w.is_bare,
                    "detached": w.is_detached,
                    "locked": w.is_locked,
                    "prunable": w.prunable,
                }))
                .collect();
            formatter.print_json(&serde_json::json!(json_worktrees))?;
        } else {
            if worktrees.is_empty() {
                formatter.info("No worktrees found");
            } else {
                formatter.info(&format!("Found {} worktree(s):", worktrees.len()));
                for w in worktrees {
                    let status = if w.is_locked {
                        " [LOCKED]"
                    } else if w.is_detached {
                        " [DETACHED]"
                    } else if w.prunable {
                        " [PRUNABLE]"
                    } else {
                        ""
                    };
                    
                    println!("{} ({}){}", 
                        w.path.display(), 
                        if w.branch.is_empty() { &w.commit[..8] } else { &w.branch },
                        status
                    );
                }
            }
        }
        Ok(())
    }
}