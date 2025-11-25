//! Command alias handler for creating custom shortcuts

use crate::cli::output::OutputFormatter;
use crate::cli::utils::find_project_root;
use crate::error::{Result, VibeTicketError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A command alias definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAlias {
    /// Alias name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Optional description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Collection of command aliases
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Aliases {
    pub aliases: HashMap<String, CommandAlias>,
}

impl Aliases {
    /// Load aliases from file
    pub fn load(project_dir: Option<&str>) -> Result<Self> {
        let path = Self::aliases_path(project_dir)?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| VibeTicketError::custom(format!("Failed to read aliases file: {e}")))?;
        let aliases: Self = serde_yaml::from_str(&content)
            .map_err(|e| VibeTicketError::custom(format!("Failed to parse aliases file: {e}")))?;
        Ok(aliases)
    }

    /// Save aliases to file
    pub fn save(&self, project_dir: Option<&str>) -> Result<()> {
        let path = Self::aliases_path(project_dir)?;
        let content = serde_yaml::to_string(self)
            .map_err(|e| VibeTicketError::custom(format!("Failed to serialize aliases: {e}")))?;
        fs::write(&path, content)
            .map_err(|e| VibeTicketError::custom(format!("Failed to write aliases file: {e}")))?;
        Ok(())
    }

    /// Get the path to the aliases file
    fn aliases_path(project_dir: Option<&str>) -> Result<PathBuf> {
        let project_root = find_project_root(project_dir)?;
        Ok(project_root.join(".vibe-ticket").join("aliases.yaml"))
    }

    /// Add a new alias
    pub fn add(&mut self, alias: CommandAlias) {
        self.aliases.insert(alias.name.clone(), alias);
    }

    /// Remove an alias by name
    pub fn remove(&mut self, name: &str) -> Option<CommandAlias> {
        self.aliases.remove(name)
    }

    /// Get an alias by name
    pub fn get(&self, name: &str) -> Option<&CommandAlias> {
        self.aliases.get(name)
    }
}

/// Handle alias create command
pub fn handle_alias_create(
    name: String,
    command: String,
    description: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    // Validate alias name
    if name.contains(' ') || name.contains('/') {
        return Err(VibeTicketError::custom(
            "Alias name cannot contain spaces or slashes",
        ));
    }

    // Check for reserved names
    let reserved = [
        "init", "new", "list", "show", "edit", "close", "start", "check", "task", "search",
        "export", "import", "config", "spec", "worktree", "mcp", "bulk", "filter", "alias", "time",
        "board", "review", "approve", "handoff", "archive", "open",
    ];
    if reserved.contains(&name.as_str()) {
        return Err(VibeTicketError::custom(format!(
            "Cannot use reserved command name '{name}' as an alias"
        )));
    }

    let mut aliases = Aliases::load(project_dir)?;

    if aliases.get(&name).is_some() {
        return Err(VibeTicketError::custom(format!(
            "Alias '{name}' already exists. Delete it first or use a different name."
        )));
    }

    let alias = CommandAlias {
        name: name.clone(),
        command: command.clone(),
        description,
        created_at: chrono::Utc::now(),
    };

    aliases.add(alias);
    aliases.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "alias": {
                "name": name,
                "command": command,
            }
        }))?;
    } else {
        output.success(&format!("Created alias '{name}'"));
        output.info(&format!("Command: {command}"));
        output.info("");
        output.info("Usage:");
        output.info(&format!("  vibe-ticket alias run {name}"));
    }

    Ok(())
}

/// Handle alias list command
pub fn handle_alias_list(project_dir: Option<&str>, output: &OutputFormatter) -> Result<()> {
    let aliases = Aliases::load(project_dir)?;

    if aliases.aliases.is_empty() {
        output.info("No aliases defined");
        output.info("");
        output.info("Create one with:");
        output.info("  vibe-ticket alias create <name> <command>");
        output.info("");
        output.info("Examples:");
        output.info("  vibe-ticket alias create urgent \"list --priority critical,high --open\"");
        output.info("  vibe-ticket alias create mine \"list --assignee me --open\"");
        return Ok(());
    }

    if output.is_json() {
        let alias_list: Vec<_> = aliases.aliases.values().collect();
        output.print_json(&serde_json::json!({
            "aliases": alias_list,
            "count": alias_list.len(),
        }))?;
    } else {
        output.info(&format!("Aliases ({}):", aliases.aliases.len()));
        output.info("");

        let mut alias_list: Vec<_> = aliases.aliases.values().collect();
        alias_list.sort_by(|a, b| a.name.cmp(&b.name));

        for alias in alias_list {
            output.info(&format!("  {}", alias.name));
            output.info(&format!("    Command: {}", alias.command));
            if let Some(desc) = &alias.description {
                output.info(&format!("    Description: {desc}"));
            }
            output.info("");
        }
    }

    Ok(())
}

/// Handle alias delete command
pub fn handle_alias_delete(
    name: String,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let mut aliases = Aliases::load(project_dir)?;

    if aliases.get(&name).is_none() {
        return Err(VibeTicketError::custom(format!("Alias '{name}' not found")));
    }

    aliases.remove(&name);
    aliases.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "deleted": name,
        }))?;
    } else {
        output.success(&format!("Deleted alias '{name}'"));
    }

    Ok(())
}

/// Handle alias run command
pub fn handle_alias_run(
    name: String,
    args: Vec<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let aliases = Aliases::load(project_dir)?;

    let alias = aliases
        .get(&name)
        .ok_or_else(|| VibeTicketError::custom(format!("Alias '{name}' not found")))?;

    // Build the full command
    let full_command = if args.is_empty() {
        alias.command.clone()
    } else {
        format!("{} {}", alias.command, args.join(" "))
    };

    output.info(&format!("Running: vibe-ticket {full_command}"));
    output.info("");

    // Parse and execute the command
    // Note: In a real implementation, we'd re-parse and dispatch to the appropriate handler
    // For now, we'll spawn a subprocess to run the command
    let status = std::process::Command::new("vibe-ticket")
        .args(full_command.split_whitespace())
        .status()
        .map_err(|e| VibeTicketError::custom(format!("Failed to run command: {e}")))?;

    if !status.success() {
        return Err(VibeTicketError::custom(format!(
            "Command exited with status: {}",
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Get an alias if it exists (for use during command dispatch)
#[allow(dead_code)]
pub fn get_alias(name: &str, project_dir: Option<&str>) -> Option<String> {
    Aliases::load(project_dir)
        .ok()
        .and_then(|aliases| aliases.get(name).map(|a| a.command.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_serialization() {
        let alias = CommandAlias {
            name: "test".to_string(),
            command: "list --status todo".to_string(),
            description: Some("Test alias".to_string()),
            created_at: chrono::Utc::now(),
        };

        let yaml = serde_yaml::to_string(&alias).unwrap();
        let parsed: CommandAlias = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.name, alias.name);
        assert_eq!(parsed.command, alias.command);
    }
}
