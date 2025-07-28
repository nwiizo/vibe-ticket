//! Integration tests for Git worktree functionality

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_worktree_commands_available() {
    let mut cmd = Command::cargo_bin("vibe-ticket").unwrap();
    
    cmd.arg("worktree")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage Git worktrees for tickets"));
}

#[test]
fn test_worktree_list_without_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("vibe-ticket").unwrap();
    
    cmd.current_dir(&temp_dir)
        .arg("worktree")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Project not initialized"));
}

#[test]
fn test_worktree_prune_dry_run() {
    // This test should work even without a git repo in dry-run mode
    let mut cmd = Command::cargo_bin("vibe-ticket").unwrap();
    
    cmd.arg("worktree")
        .arg("prune")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn test_worktree_remove_invalid_reference() {
    let mut cmd = Command::cargo_bin("vibe-ticket").unwrap();
    
    cmd.arg("worktree")
        .arg("remove")
        .arg("non-existent-worktree")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Worktree not found"));
}