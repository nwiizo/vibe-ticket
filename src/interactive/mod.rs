//! Interactive mode for vibe-ticket
//!
//! Provides guided, interactive interfaces for common operations
//! to improve user experience and reduce errors.

use crate::error::Result;
use crate::templates::{FieldType, TemplateManager};
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use std::collections::HashMap;

/// Interactive ticket creation
pub struct InteractiveMode {
    theme: ColorfulTheme,
    template_manager: TemplateManager,
}

impl InteractiveMode {
    /// Create a new interactive mode handler
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
            template_manager: TemplateManager::new(),
        }
    }

    /// Run interactive ticket creation
    pub fn create_ticket(&self) -> Result<InteractiveTicketData> {
        println!("🎫 Welcome to vibe-ticket interactive mode!\n");

        // Choose ticket type
        let ticket_types = vec![
            "Feature - New functionality",
            "Bug - Something isn't working",
            "Task - General work item",
            "Refactor - Code improvement",
            "Custom - No template",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("What would you like to create?")
            .items(&ticket_types)
            .default(0)
            .interact()?;

        let template_name = match selection {
            0 => Some("feature"),
            1 => Some("bug"),
            2 => Some("task"),
            3 => Some("task"), // Use task template for refactor
            _ => None,
        };

        if let Some(template_name) = template_name {
            self.create_from_template(template_name)
        } else {
            self.create_custom()
        }
    }

    /// Create ticket from template
    fn create_from_template(&self, template_name: &str) -> Result<InteractiveTicketData> {
        let template = self.template_manager.get(template_name).ok_or_else(|| {
            crate::error::VibeTicketError::TemplateNotFound(template_name.to_string())
        })?;

        println!("\n📝 Using {} template\n", template.name);

        let mut values = HashMap::new();

        // Collect values for each field
        for field in &template.fields {
            let value = match &field.field_type {
                FieldType::Text => {
                    let mut input =
                        Input::<String>::with_theme(&self.theme).with_prompt(&field.label);

                    if let Some(default) = &field.default {
                        input = input.default(default.clone());
                    }

                    if let Some(help) = &field.help {
                        println!("ℹ️  {}", help);
                    }

                    if field.required {
                        input.interact()?
                    } else {
                        input.allow_empty(true).interact()?
                    }
                },
                FieldType::LongText => {
                    println!("📝 {} (Press Enter twice when done)", field.label);
                    if let Some(help) = &field.help {
                        println!("ℹ️  {}", help);
                    }
                    self.read_multiline()?
                },
                FieldType::Select(options) => {
                    let selection = Select::with_theme(&self.theme)
                        .with_prompt(&field.label)
                        .items(options)
                        .default(0)
                        .interact()?;
                    options[selection].clone()
                },
                FieldType::MultiSelect(options) => {
                    let selections = MultiSelect::with_theme(&self.theme)
                        .with_prompt(&field.label)
                        .items(options)
                        .interact()?;
                    selections
                        .iter()
                        .map(|&i| options[i].clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                },
                FieldType::List => {
                    println!("📝 {} (Enter items, blank line when done)", field.label);
                    if let Some(help) = &field.help {
                        println!("ℹ️  {}", help);
                    }
                    self.read_list()?
                },
                _ => String::new(), // TODO: Implement other field types
            };

            if !value.is_empty() || !field.required {
                values.insert(field.name.clone(), value);
            }
        }

        // Get additional metadata
        let priority = self.select_priority()?;
        let tags = self.input_tags()?;
        let start_now = self.confirm_start()?;

        // Create ticket data from template
        let ticket_data = self
            .template_manager
            .create_from_template(template_name, values)?;

        Ok(InteractiveTicketData {
            title: ticket_data.title,
            description: ticket_data.description,
            priority: ticket_data.priority.unwrap_or(priority),
            tags: if tags.is_empty() {
                ticket_data.tags
            } else {
                tags
            },
            start_now,
            template_used: Some(template_name.to_string()),
        })
    }

    /// Create custom ticket without template
    fn create_custom(&self) -> Result<InteractiveTicketData> {
        println!("\n📝 Creating custom ticket\n");

        let title = Input::<String>::with_theme(&self.theme)
            .with_prompt("Title")
            .interact()?;

        let description = if Confirm::with_theme(&self.theme)
            .with_prompt("Add a detailed description?")
            .default(true)
            .interact()?
        {
            println!("📝 Description (Press Enter twice when done)");
            Some(self.read_multiline()?)
        } else {
            None
        };

        let priority = self.select_priority()?;
        let tags = self.input_tags()?;
        let start_now = self.confirm_start()?;

        Ok(InteractiveTicketData {
            title,
            description,
            priority,
            tags,
            start_now,
            template_used: None,
        })
    }

    /// Select priority interactively
    fn select_priority(&self) -> Result<String> {
        let priorities = vec!["Low", "Medium", "High", "Critical"];
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Priority")
            .items(&priorities)
            .default(1) // Default to Medium
            .interact()?;

        Ok(priorities[selection].to_lowercase())
    }

    /// Input tags interactively
    fn input_tags(&self) -> Result<Vec<String>> {
        let tags_input = Input::<String>::with_theme(&self.theme)
            .with_prompt("Tags (comma-separated, optional)")
            .allow_empty(true)
            .interact()?;

        Ok(if tags_input.is_empty() {
            Vec::new()
        } else {
            tags_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }

    /// Confirm whether to start working on ticket immediately
    fn confirm_start(&self) -> Result<bool> {
        Confirm::with_theme(&self.theme)
            .with_prompt("Start working on this ticket now?")
            .default(false)
            .interact()
            .map_err(Into::into)
    }

    /// Read multiline input
    fn read_multiline(&self) -> Result<String> {
        let mut lines = Vec::new();
        let stdin = std::io::stdin();
        let mut empty_lines = 0;

        loop {
            let mut line = String::new();
            stdin.read_line(&mut line)?;

            if line.trim().is_empty() {
                empty_lines += 1;
                if empty_lines >= 2 {
                    break;
                }
                lines.push(String::new());
            } else {
                empty_lines = 0;
                lines.push(line.trim_end().to_string());
            }
        }

        // Remove trailing empty lines
        while lines.last().map_or(false, |l| l.is_empty()) {
            lines.pop();
        }

        Ok(lines.join("\n"))
    }

    /// Read list items
    fn read_list(&self) -> Result<String> {
        let mut items = Vec::new();
        let stdin = std::io::stdin();
        let mut counter = 1;

        loop {
            print!("  {}. ", counter);
            use std::io::Write;
            std::io::stdout().flush()?;

            let mut line = String::new();
            stdin.read_line(&mut line)?;

            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }

            items.push(format!("{}. {}", counter, trimmed));
            counter += 1;
        }

        Ok(items.join("\n"))
    }

    /// Interactive workflow selector
    pub fn select_workflow(&self) -> Result<String> {
        let workflows = vec![
            "work-on - Start working on a ticket",
            "finish - Complete current ticket",
            "review - Review tickets",
            "report - Generate reports",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select a workflow")
            .items(&workflows)
            .interact()?;

        Ok(match selection {
            0 => "work-on",
            1 => "finish",
            2 => "review",
            _ => "report",
        }
        .to_string())
    }

    /// Interactive ticket selector
    pub fn select_ticket(&self, tickets: Vec<(String, String)>) -> Result<String> {
        if tickets.is_empty() {
            return Err(crate::error::VibeTicketError::NoTicketsFound);
        }

        let items: Vec<String> = tickets
            .iter()
            .map(|(id, title)| format!("{} - {}", id, title))
            .collect();

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select a ticket")
            .items(&items)
            .interact()?;

        Ok(tickets[selection].0.clone())
    }
}

/// Data structure for interactive ticket creation
#[derive(Debug)]
pub struct InteractiveTicketData {
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub tags: Vec<String>,
    pub start_now: bool,
    pub template_used: Option<String>,
}

/// Interactive command prompt for continuous interaction
pub struct InteractivePrompt {
    theme: ColorfulTheme,
}

impl InteractivePrompt {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

    /// Run the interactive prompt
    pub fn run(&self) -> Result<()> {
        println!("🎫 vibe-ticket Interactive Mode");
        println!("Type 'help' for commands, 'exit' to quit\n");

        loop {
            let input = Input::<String>::with_theme(&self.theme)
                .with_prompt("vibe-ticket>")
                .interact()?;

            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "help" => self.show_help(),
                "create" => {
                    let mode = InteractiveMode::new();
                    match mode.create_ticket() {
                        Ok(data) => {
                            println!("✅ Ticket created: {}", data.title);
                            // TODO: Actually create the ticket
                        },
                        Err(e) => println!("❌ Error: {}", e),
                    }
                },
                "list" => println!("📋 Listing tickets..."), // TODO: Implement
                "work-on" => println!("🔧 Starting work..."), // TODO: Implement
                "exit" | "quit" => {
                    println!("👋 Goodbye!");
                    break;
                },
                _ => println!("❓ Unknown command. Type 'help' for available commands."),
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("\n📚 Available Commands:");
        println!("  create   - Create a new ticket interactively");
        println!("  list     - List all tickets");
        println!("  work-on  - Start working on a ticket");
        println!("  finish   - Complete current ticket");
        println!("  help     - Show this help message");
        println!("  exit     - Exit interactive mode\n");
    }
}
