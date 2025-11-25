//! Custom hooks handler for running scripts on ticket events
//!
//! Hooks allow users to run custom scripts when certain events occur,
//! such as ticket creation, status changes, or ticket closure.

use crate::cli::output::OutputFormatter;
use crate::cli::utils::find_project_root;
use crate::error::{Result, VibeTicketError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Available hook events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    /// Triggered after a ticket is created
    PostCreate,
    /// Triggered before a ticket status changes
    PreStatusChange,
    /// Triggered after a ticket status changes
    PostStatusChange,
    /// Triggered before a ticket is closed
    PreClose,
    /// Triggered after a ticket is closed
    PostClose,
    /// Triggered when a ticket is started
    PostStart,
    /// Triggered when work on a ticket is finished
    PostFinish,
    /// Triggered after a ticket is edited
    PostEdit,
    /// Triggered after tags are modified
    PostTagChange,
}

impl HookEvent {
    /// Get all available hook events
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::PostCreate,
            Self::PreStatusChange,
            Self::PostStatusChange,
            Self::PreClose,
            Self::PostClose,
            Self::PostStart,
            Self::PostFinish,
            Self::PostEdit,
            Self::PostTagChange,
        ]
    }

    /// Get the event name as a string
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PostCreate => "post_create",
            Self::PreStatusChange => "pre_status_change",
            Self::PostStatusChange => "post_status_change",
            Self::PreClose => "pre_close",
            Self::PostClose => "post_close",
            Self::PostStart => "post_start",
            Self::PostFinish => "post_finish",
            Self::PostEdit => "post_edit",
            Self::PostTagChange => "post_tag_change",
        }
    }

    /// Parse from string
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "post_create" | "post-create" => Some(Self::PostCreate),
            "pre_status_change" | "pre-status-change" => Some(Self::PreStatusChange),
            "post_status_change" | "post-status-change" => Some(Self::PostStatusChange),
            "pre_close" | "pre-close" => Some(Self::PreClose),
            "post_close" | "post-close" => Some(Self::PostClose),
            "post_start" | "post-start" => Some(Self::PostStart),
            "post_finish" | "post-finish" => Some(Self::PostFinish),
            "post_edit" | "post-edit" => Some(Self::PostEdit),
            "post_tag_change" | "post-tag-change" => Some(Self::PostTagChange),
            _ => None,
        }
    }
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    /// Hook name (for identification)
    pub name: String,
    /// Event that triggers this hook
    pub event: HookEvent,
    /// Command to execute
    pub command: String,
    /// Whether the hook is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Description of what this hook does
    pub description: Option<String>,
    /// Whether to abort the operation if hook fails (only for pre-* hooks)
    #[serde(default)]
    pub abort_on_failure: bool,
}

const fn default_enabled() -> bool {
    true
}

/// Collection of hooks
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Hooks {
    pub hooks: HashMap<String, Hook>,
}

impl Hooks {
    /// Load hooks from file
    pub fn load(project_dir: Option<&str>) -> Result<Self> {
        let path = Self::hooks_path(project_dir)?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| VibeTicketError::custom(format!("Failed to read hooks file: {e}")))?;
        let hooks: Self = serde_yaml::from_str(&content)
            .map_err(|e| VibeTicketError::custom(format!("Failed to parse hooks file: {e}")))?;
        Ok(hooks)
    }

    /// Save hooks to file
    pub fn save(&self, project_dir: Option<&str>) -> Result<()> {
        let path = Self::hooks_path(project_dir)?;
        let content = serde_yaml::to_string(self)
            .map_err(|e| VibeTicketError::custom(format!("Failed to serialize hooks: {e}")))?;
        fs::write(&path, content)
            .map_err(|e| VibeTicketError::custom(format!("Failed to write hooks file: {e}")))?;
        Ok(())
    }

    /// Get the path to the hooks file
    fn hooks_path(project_dir: Option<&str>) -> Result<PathBuf> {
        let project_root = find_project_root(project_dir)?;
        Ok(project_root.join(".vibe-ticket").join("hooks.yaml"))
    }

    /// Add a new hook
    pub fn add(&mut self, hook: Hook) {
        self.hooks.insert(hook.name.clone(), hook);
    }

    /// Remove a hook by name
    pub fn remove(&mut self, name: &str) -> Option<Hook> {
        self.hooks.remove(name)
    }

    /// Get a hook by name
    pub fn get(&self, name: &str) -> Option<&Hook> {
        self.hooks.get(name)
    }

    /// Get a mutable hook by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Hook> {
        self.hooks.get_mut(name)
    }

    /// Get all hooks for a specific event
    pub fn get_for_event(&self, event: HookEvent) -> Vec<&Hook> {
        self.hooks
            .values()
            .filter(|h| h.event == event && h.enabled)
            .collect()
    }
}

/// Context passed to hooks during execution
#[derive(Debug, Clone, Serialize)]
pub struct HookContext {
    /// Ticket ID
    pub ticket_id: String,
    /// Ticket slug
    pub ticket_slug: String,
    /// Event that triggered the hook
    pub event: String,
    /// Previous status (for status change events)
    pub previous_status: Option<String>,
    /// New status (for status change events)
    pub new_status: Option<String>,
    /// Additional data as JSON
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Execute hooks for a given event
pub fn execute_hooks(
    event: HookEvent,
    context: &HookContext,
    project_dir: Option<&str>,
) -> Result<bool> {
    let hooks = Hooks::load(project_dir)?;
    let event_hooks = hooks.get_for_event(event);

    if event_hooks.is_empty() {
        return Ok(true);
    }

    let context_json = serde_json::to_string(context).unwrap_or_else(|_| "{}".to_string());

    for hook in event_hooks {
        let result = execute_hook(hook, &context_json);

        if let Err(e) = result {
            eprintln!("Hook '{}' failed: {}", hook.name, e);
            if hook.abort_on_failure && event.as_str().starts_with("pre_") {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Execute a single hook
fn execute_hook(hook: &Hook, context_json: &str) -> Result<()> {
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };

    let shell_arg = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };

    let output = Command::new(shell)
        .arg(shell_arg)
        .arg(&hook.command)
        .env("VIBE_TICKET_CONTEXT", context_json)
        .env("VIBE_TICKET_EVENT", hook.event.as_str())
        .output()
        .map_err(|e| VibeTicketError::custom(format!("Failed to execute hook: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VibeTicketError::custom(format!(
            "Hook command failed: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

/// Handle hook create command
pub fn handle_hook_create(
    name: String,
    event: String,
    command: String,
    description: Option<String>,
    abort_on_failure: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let hook_event = HookEvent::parse(&event).ok_or_else(|| {
        VibeTicketError::custom(format!(
            "Invalid event: {}. Valid events: {}",
            event,
            HookEvent::all()
                .iter()
                .map(|e| e.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;

    let mut hooks = Hooks::load(project_dir)?;

    if hooks.get(&name).is_some() {
        return Err(VibeTicketError::custom(format!(
            "Hook '{name}' already exists. Delete it first or use a different name."
        )));
    }

    let hook = Hook {
        name: name.clone(),
        event: hook_event,
        command: command.clone(),
        enabled: true,
        description,
        abort_on_failure,
    };

    hooks.add(hook);
    hooks.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "hook": {
                "name": name,
                "event": event,
                "command": command,
            }
        }))?;
    } else {
        output.success(&format!("Created hook '{name}'"));
        output.info(&format!("Event: {event}"));
        output.info(&format!("Command: {command}"));
        if abort_on_failure {
            output.info("Will abort operation on failure");
        }
    }

    Ok(())
}

/// Handle hook list command
pub fn handle_hook_list(project_dir: Option<&str>, output: &OutputFormatter) -> Result<()> {
    let hooks = Hooks::load(project_dir)?;

    if hooks.hooks.is_empty() {
        output.info("No hooks defined");
        output.info("");
        output.info("Create one with:");
        output.info("  vibe-ticket hook create <name> <event> <command>");
        output.info("");
        output.info("Available events:");
        for event in HookEvent::all() {
            output.info(&format!("  - {}", event.as_str()));
        }
        return Ok(());
    }

    if output.is_json() {
        let hook_list: Vec<_> = hooks.hooks.values().collect();
        output.print_json(&serde_json::json!({
            "hooks": hook_list,
            "count": hook_list.len(),
        }))?;
    } else {
        output.info(&format!("Hooks ({}):", hooks.hooks.len()));
        output.info("");

        let mut hook_list: Vec<_> = hooks.hooks.values().collect();
        hook_list.sort_by(|a, b| a.name.cmp(&b.name));

        for hook in hook_list {
            let status = if hook.enabled { "✓" } else { "✗" };
            output.info(&format!("  {} {}", status, hook.name));
            output.info(&format!("    Event: {}", hook.event));
            output.info(&format!("    Command: {}", hook.command));
            if let Some(desc) = &hook.description {
                output.info(&format!("    Description: {desc}"));
            }
            if hook.abort_on_failure {
                output.info("    Abort on failure: yes");
            }
            output.info("");
        }
    }

    Ok(())
}

/// Handle hook delete command
pub fn handle_hook_delete(
    name: String,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let mut hooks = Hooks::load(project_dir)?;

    if hooks.get(&name).is_none() {
        return Err(VibeTicketError::custom(format!("Hook '{name}' not found")));
    }

    hooks.remove(&name);
    hooks.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "deleted": name,
        }))?;
    } else {
        output.success(&format!("Deleted hook '{name}'"));
    }

    Ok(())
}

/// Handle hook enable command
pub fn handle_hook_enable(
    name: String,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let mut hooks = Hooks::load(project_dir)?;

    let hook = hooks
        .get_mut(&name)
        .ok_or_else(|| VibeTicketError::custom(format!("Hook '{name}' not found")))?;

    hook.enabled = true;
    hooks.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "hook": name,
            "enabled": true,
        }))?;
    } else {
        output.success(&format!("Enabled hook '{name}'"));
    }

    Ok(())
}

/// Handle hook disable command
pub fn handle_hook_disable(
    name: String,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let mut hooks = Hooks::load(project_dir)?;

    let hook = hooks
        .get_mut(&name)
        .ok_or_else(|| VibeTicketError::custom(format!("Hook '{name}' not found")))?;

    hook.enabled = false;
    hooks.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "hook": name,
            "enabled": false,
        }))?;
    } else {
        output.success(&format!("Disabled hook '{name}'"));
    }

    Ok(())
}

/// Handle hook test command
pub fn handle_hook_test(
    name: String,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let hooks = Hooks::load(project_dir)?;

    let hook = hooks
        .get(&name)
        .ok_or_else(|| VibeTicketError::custom(format!("Hook '{name}' not found")))?;

    output.info(&format!("Testing hook '{name}'..."));

    let test_context = HookContext {
        ticket_id: "test-id".to_string(),
        ticket_slug: "test-ticket".to_string(),
        event: hook.event.to_string(),
        previous_status: Some("todo".to_string()),
        new_status: Some("doing".to_string()),
        extra: HashMap::new(),
    };

    let context_json = serde_json::to_string(&test_context).unwrap_or_else(|_| "{}".to_string());

    match execute_hook(hook, &context_json) {
        Ok(()) => {
            if output.is_json() {
                output.print_json(&serde_json::json!({
                    "status": "success",
                    "hook": name,
                    "message": "Hook executed successfully",
                }))?;
            } else {
                output.success("Hook executed successfully");
            }
        },
        Err(e) => {
            if output.is_json() {
                output.print_json(&serde_json::json!({
                    "status": "error",
                    "hook": name,
                    "message": e.to_string(),
                }))?;
            } else {
                output.error(&format!("Hook failed: {e}"));
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_parsing() {
        assert_eq!(HookEvent::parse("post_create"), Some(HookEvent::PostCreate));
        assert_eq!(HookEvent::parse("post-create"), Some(HookEvent::PostCreate));
        assert_eq!(
            HookEvent::parse("PRE_STATUS_CHANGE"),
            Some(HookEvent::PreStatusChange)
        );
        assert_eq!(HookEvent::parse("invalid"), None);
    }

    #[test]
    fn test_hook_serialization() {
        let hook = Hook {
            name: "test".to_string(),
            event: HookEvent::PostCreate,
            command: "echo test".to_string(),
            enabled: true,
            description: Some("Test hook".to_string()),
            abort_on_failure: false,
        };

        let yaml = serde_yaml::to_string(&hook).unwrap();
        let parsed: Hook = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.name, hook.name);
        assert_eq!(parsed.event, hook.event);
    }
}
