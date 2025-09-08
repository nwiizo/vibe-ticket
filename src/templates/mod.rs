//! Template system for vibe-ticket
//! 
//! Provides built-in and custom templates for common ticket types
//! to improve consistency and reduce creation time.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::error::Result;

/// Template field types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    LongText,
    Select(Vec<String>),
    MultiSelect(Vec<String>),
    Number,
    Boolean,
    Date,
    List,
}

/// Template field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default: Option<String>,
    pub help: Option<String>,
}

/// Ticket template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub category: String,
    pub fields: Vec<TemplateField>,
    pub default_priority: Option<String>,
    pub default_tags: Vec<String>,
}

/// Template manager
pub struct TemplateManager {
    templates: HashMap<String, Template>,
    custom_templates_dir: Option<PathBuf>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            custom_templates_dir: None,
        };
        manager.load_builtin_templates();
        manager
    }

    /// Load built-in templates
    fn load_builtin_templates(&mut self) {
        // Bug template
        let bug_template = Template {
            name: "bug".to_string(),
            description: "Report a bug or issue".to_string(),
            category: "issue".to_string(),
            fields: vec![
                TemplateField {
                    name: "summary".to_string(),
                    label: "Brief summary".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    default: None,
                    help: Some("One-line description of the bug".to_string()),
                },
                TemplateField {
                    name: "steps_to_reproduce".to_string(),
                    label: "Steps to reproduce".to_string(),
                    field_type: FieldType::List,
                    required: true,
                    default: None,
                    help: Some("List the steps to reproduce the bug".to_string()),
                },
                TemplateField {
                    name: "expected_behavior".to_string(),
                    label: "Expected behavior".to_string(),
                    field_type: FieldType::LongText,
                    required: true,
                    default: None,
                    help: Some("What should happen?".to_string()),
                },
                TemplateField {
                    name: "actual_behavior".to_string(),
                    label: "Actual behavior".to_string(),
                    field_type: FieldType::LongText,
                    required: true,
                    default: None,
                    help: Some("What actually happens?".to_string()),
                },
                TemplateField {
                    name: "environment".to_string(),
                    label: "Environment".to_string(),
                    field_type: FieldType::Select(vec![
                        "development".to_string(),
                        "staging".to_string(),
                        "production".to_string(),
                    ]),
                    required: false,
                    default: Some("development".to_string()),
                    help: Some("Where did you encounter this bug?".to_string()),
                },
                TemplateField {
                    name: "severity".to_string(),
                    label: "Severity".to_string(),
                    field_type: FieldType::Select(vec![
                        "low".to_string(),
                        "medium".to_string(),
                        "high".to_string(),
                        "critical".to_string(),
                    ]),
                    required: false,
                    default: Some("medium".to_string()),
                    help: Some("How severe is this bug?".to_string()),
                },
            ],
            default_priority: Some("high".to_string()),
            default_tags: vec!["bug".to_string()],
        };
        self.templates.insert("bug".to_string(), bug_template);

        // Feature template
        let feature_template = Template {
            name: "feature".to_string(),
            description: "Propose a new feature".to_string(),
            category: "enhancement".to_string(),
            fields: vec![
                TemplateField {
                    name: "title".to_string(),
                    label: "Feature title".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    default: None,
                    help: Some("Brief, descriptive title".to_string()),
                },
                TemplateField {
                    name: "problem_statement".to_string(),
                    label: "Problem statement".to_string(),
                    field_type: FieldType::LongText,
                    required: true,
                    default: None,
                    help: Some("What problem does this solve?".to_string()),
                },
                TemplateField {
                    name: "proposed_solution".to_string(),
                    label: "Proposed solution".to_string(),
                    field_type: FieldType::LongText,
                    required: true,
                    default: None,
                    help: Some("How should we solve this?".to_string()),
                },
                TemplateField {
                    name: "alternatives".to_string(),
                    label: "Alternatives considered".to_string(),
                    field_type: FieldType::LongText,
                    required: false,
                    default: None,
                    help: Some("What other solutions were considered?".to_string()),
                },
                TemplateField {
                    name: "acceptance_criteria".to_string(),
                    label: "Acceptance criteria".to_string(),
                    field_type: FieldType::List,
                    required: true,
                    default: None,
                    help: Some("How will we know this is complete?".to_string()),
                },
            ],
            default_priority: Some("medium".to_string()),
            default_tags: vec!["feature".to_string(), "enhancement".to_string()],
        };
        self.templates.insert("feature".to_string(), feature_template);

        // Task template
        let task_template = Template {
            name: "task".to_string(),
            description: "General task or work item".to_string(),
            category: "task".to_string(),
            fields: vec![
                TemplateField {
                    name: "title".to_string(),
                    label: "Task title".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    default: None,
                    help: Some("What needs to be done?".to_string()),
                },
                TemplateField {
                    name: "description".to_string(),
                    label: "Description".to_string(),
                    field_type: FieldType::LongText,
                    required: false,
                    default: None,
                    help: Some("Additional details".to_string()),
                },
                TemplateField {
                    name: "checklist".to_string(),
                    label: "Checklist".to_string(),
                    field_type: FieldType::List,
                    required: false,
                    default: None,
                    help: Some("Break down into subtasks".to_string()),
                },
            ],
            default_priority: Some("medium".to_string()),
            default_tags: vec!["task".to_string()],
        };
        self.templates.insert("task".to_string(), task_template);
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// List all available templates
    pub fn list(&self) -> Vec<&Template> {
        self.templates.values().collect()
    }

    /// Load custom templates from a directory
    pub fn load_custom_templates(&mut self, dir: PathBuf) -> Result<()> {
        self.custom_templates_dir = Some(dir);
        // TODO: Implement loading from YAML/JSON files
        Ok(())
    }

    /// Create a ticket from a template with provided values
    pub fn create_from_template(
        &self,
        template_name: &str,
        values: HashMap<String, String>,
    ) -> Result<TicketData> {
        let template = self.get(template_name)
            .ok_or_else(|| crate::error::VibeTicketError::TemplateNotFound(template_name.to_string()))?;

        // Validate required fields
        for field in &template.fields {
            if field.required && !values.contains_key(&field.name) {
                return Err(crate::error::VibeTicketError::MissingRequiredField(field.name.clone()));
            }
        }

        // Build description from template fields
        let mut description = String::new();
        for field in &template.fields {
            if let Some(value) = values.get(&field.name) {
                description.push_str(&format!("## {}\n{}\n\n", field.label, value));
            }
        }

        Ok(TicketData {
            title: values.get("title").or(values.get("summary"))
                .cloned().unwrap_or_else(|| template.name.clone()),
            description: Some(description),
            priority: template.default_priority.clone(),
            tags: template.default_tags.clone(),
        })
    }
}

/// Simplified ticket data structure for template creation
#[derive(Debug, Clone)]
pub struct TicketData {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_templates() {
        let manager = TemplateManager::new();
        
        assert!(manager.get("bug").is_some());
        assert!(manager.get("feature").is_some());
        assert!(manager.get("task").is_some());
        assert!(manager.get("nonexistent").is_none());
    }

    #[test]
    fn test_create_from_template() {
        let manager = TemplateManager::new();
        let mut values = HashMap::new();
        values.insert("summary".to_string(), "Test bug".to_string());
        values.insert("steps_to_reproduce".to_string(), "1. Do this\n2. Do that".to_string());
        values.insert("expected_behavior".to_string(), "Should work".to_string());
        values.insert("actual_behavior".to_string(), "Doesn't work".to_string());

        let result = manager.create_from_template("bug", values);
        assert!(result.is_ok());
        
        let ticket = result.unwrap();
        assert_eq!(ticket.title, "Test bug");
        assert!(ticket.description.is_some());
        assert_eq!(ticket.priority, Some("high".to_string()));
        assert!(ticket.tags.contains(&"bug".to_string()));
    }
}