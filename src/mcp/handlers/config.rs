//! Configuration management MCP tool handlers

use crate::config::Config;
use crate::mcp::handlers::schema_helper::json_to_schema;
use crate::mcp::service::VibeTicketService;
use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::sync::Arc;

// Simple ConfigManager for MCP
struct ConfigManager;

impl ConfigManager {
    fn new() -> Self {
        Self
    }
    
    fn load_from_path(&self, path: &std::path::Path) -> Result<Config, String> {
        Config::load_from_path(path)
            .map_err(|e| format!("Failed to load config: {}", e))
    }
    
    fn save_to_path(&self, config: &Config, path: &std::path::Path) -> Result<(), String> {
        config.save_to_path(path)
            .map_err(|e| format!("Failed to save config: {}", e))
    }
}

/// Register all configuration management tools
pub fn register_tools() -> Vec<Tool> {
    vec![
        // Show config tool
        Tool {
            name: Cow::Borrowed("vibe-ticket.config.show"),
            description: Some(Cow::Borrowed("Show current configuration")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Specific configuration key to show (shows all if not specified)"
                    }
                }
            }))),
            annotations: None,
        },
        // Set config tool
        Tool {
            name: Cow::Borrowed("vibe-ticket.config.set"),
            description: Some(Cow::Borrowed("Set a configuration value")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Configuration key (e.g., 'project.default_priority')"
                    },
                    "value": {
                        "description": "Configuration value"
                    }
                },
                "required": ["key", "value"]
            }))),
            annotations: None,
        },
    ]
}

/// Handle showing configuration
pub async fn handle_show(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        key: Option<String>,
    }

    let args: Args = serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid arguments: {}", e))?;

    let config_path = service.project_root.join(".vibe-ticket").join("config.yaml");
    let config_manager = ConfigManager::new();
    let config = config_manager.load_from_path(&config_path)
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    if let Some(key) = args.key {
        // Show specific key
        let value = match key.as_str() {
            "project.name" => json!(config.project.name),
            "project.description" => json!(config.project.description),
            "project.default_priority" => json!(config.project.default_priority),
            "project.default_assignee" => json!(config.project.default_assignee),
            
            "git.auto_branch" => json!(config.git.auto_branch),
            "git.branch_prefix" => json!(config.git.branch_prefix),
            "git.commit_template" => json!(config.git.commit_template),
            "git.worktree_enabled" => json!(config.git.worktree_enabled),
            "git.worktree_default" => json!(config.git.worktree_default),
            "git.worktree_prefix" => json!(config.git.worktree_prefix),
            "git.worktree_cleanup_on_close" => json!(config.git.worktree_cleanup_on_close),
            
            "ui.date_format" => json!(config.ui.date_format),
            
            _ => return Err(format!("Unknown configuration key: {}", key))
        };
        
        Ok(json!({
            "key": key,
            "value": value
        }))
    } else {
        // Show all configuration
        Ok(json!({
            "project": {
                "name": config.project.name,
                "description": config.project.description,
                "default_priority": config.project.default_priority,
                "default_assignee": config.project.default_assignee,
            },
            "git": {
                "auto_branch": config.git.auto_branch,
                "branch_prefix": config.git.branch_prefix,
                "commit_template": config.git.commit_template,
                "worktree_enabled": config.git.worktree_enabled,
                "worktree_default": config.git.worktree_default,
                "worktree_prefix": config.git.worktree_prefix,
                "worktree_cleanup_on_close": config.git.worktree_cleanup_on_close,
            },
            "ui": {
                "date_format": config.ui.date_format,
            }
        }))
    }
}

/// Handle setting configuration
pub async fn handle_set(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        key: String,
        value: Value,
    }

    let args: Args = serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid arguments: {}", e))?;

    let config_path = service.project_root.join(".vibe-ticket").join("config.yaml");
    let config_manager = ConfigManager::new();
    let mut config = config_manager.load_from_path(&config_path)
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Parse and set the value
    match args.key.as_str() {
        "project.name" => {
            config.project.name = args.value.as_str()
                .ok_or("Value must be a string")?
                .to_string();
        },
        "project.description" => {
            config.project.description = args.value.as_str()
                .map(|s| s.to_string());
        },
        "project.default_priority" => {
            config.project.default_priority = args.value.as_str()
                .ok_or("Value must be a string")?
                .to_string();
        },
        "project.default_assignee" => {
            config.project.default_assignee = args.value.as_str()
                .map(|s| s.to_string());
        },
        
        "git.auto_branch" => {
            config.git.auto_branch = args.value.as_bool()
                .ok_or("Value must be a boolean")?;
        },
        "git.branch_prefix" => {
            config.git.branch_prefix = args.value.as_str()
                .ok_or("Value must be a string")?
                .to_string();
        },
        "git.commit_template" => {
            config.git.commit_template = args.value.as_str()
                .map(|s| s.to_string());
        },
        "git.worktree_enabled" => {
            config.git.worktree_enabled = args.value.as_bool()
                .ok_or("Value must be a boolean")?;
        },
        "git.worktree_default" => {
            config.git.worktree_default = args.value.as_bool()
                .ok_or("Value must be a boolean")?;
        },
        "git.worktree_prefix" => {
            config.git.worktree_prefix = args.value.as_str()
                .ok_or("Value must be a string")?
                .to_string();
        },
        "git.worktree_cleanup_on_close" => {
            config.git.worktree_cleanup_on_close = args.value.as_bool()
                .ok_or("Value must be a boolean")?;
        },
        
        "ui.date_format" => {
            config.ui.date_format = args.value.as_str()
                .ok_or("Value must be a string")?
                .to_string();
        },
        
        _ => return Err(format!("Unknown configuration key: {}", args.key))
    }

    // Save the updated configuration
    config_manager.save_to_path(&config, &config_path)
        .map_err(|e| format!("Failed to save configuration: {}", e))?;

    Ok(json!({
        "status": "updated",
        "key": args.key,
        "value": args.value
    }))
}