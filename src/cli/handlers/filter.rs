//! Saved filters (views) handler for managing reusable filter expressions

use crate::cli::output::OutputFormatter;
use crate::cli::utils::find_project_root;
use crate::error::{Result, VibeTicketError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A saved filter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFilter {
    /// Filter name
    pub name: String,
    /// Filter expression (e.g., "status:todo priority:high")
    pub expression: String,
    /// Optional description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Collection of saved filters
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavedFilters {
    pub filters: HashMap<String, SavedFilter>,
}

impl SavedFilters {
    /// Load saved filters from file
    pub fn load(project_dir: Option<&str>) -> Result<Self> {
        let path = Self::filters_path(project_dir)?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| VibeTicketError::custom(format!("Failed to read filters file: {e}")))?;
        let filters: Self = serde_yaml::from_str(&content)
            .map_err(|e| VibeTicketError::custom(format!("Failed to parse filters file: {e}")))?;
        Ok(filters)
    }

    /// Save filters to file
    pub fn save(&self, project_dir: Option<&str>) -> Result<()> {
        let path = Self::filters_path(project_dir)?;
        let content = serde_yaml::to_string(self)
            .map_err(|e| VibeTicketError::custom(format!("Failed to serialize filters: {e}")))?;
        fs::write(&path, content)
            .map_err(|e| VibeTicketError::custom(format!("Failed to write filters file: {e}")))?;
        Ok(())
    }

    /// Get the path to the filters file
    fn filters_path(project_dir: Option<&str>) -> Result<PathBuf> {
        let project_root = find_project_root(project_dir)?;
        Ok(project_root.join(".vibe-ticket").join("filters.yaml"))
    }

    /// Add a new filter
    pub fn add(&mut self, filter: SavedFilter) {
        self.filters.insert(filter.name.clone(), filter);
    }

    /// Remove a filter by name
    pub fn remove(&mut self, name: &str) -> Option<SavedFilter> {
        self.filters.remove(name)
    }

    /// Get a filter by name
    pub fn get(&self, name: &str) -> Option<&SavedFilter> {
        self.filters.get(name)
    }
}

/// Handle filter create command
pub fn handle_filter_create(
    name: String,
    expression: String,
    description: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let mut filters = SavedFilters::load(project_dir)?;

    if filters.get(&name).is_some() {
        return Err(VibeTicketError::custom(format!(
            "Filter '{name}' already exists. Use a different name or delete the existing filter."
        )));
    }

    let filter = SavedFilter {
        name: name.clone(),
        expression: expression.clone(),
        description,
        created_at: chrono::Utc::now(),
    };

    filters.add(filter);
    filters.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "filter": {
                "name": name,
                "expression": expression,
            }
        }))?;
    } else {
        output.success(&format!("Created filter '{name}'"));
        output.info(&format!("Expression: {expression}"));
        output.info("");
        output.info("Usage:");
        output.info(&format!("  vibe-ticket filter apply {name}"));
        output.info(&format!("  vibe-ticket list --filter @{name}"));
    }

    Ok(())
}

/// Handle filter list command
pub fn handle_filter_list(project_dir: Option<&str>, output: &OutputFormatter) -> Result<()> {
    let filters = SavedFilters::load(project_dir)?;

    if filters.filters.is_empty() {
        output.info("No saved filters");
        output.info("");
        output.info("Create one with:");
        output.info("  vibe-ticket filter create <name> <expression>");
        return Ok(());
    }

    if output.is_json() {
        let filter_list: Vec<_> = filters.filters.values().collect();
        output.print_json(&serde_json::json!({
            "filters": filter_list,
            "count": filter_list.len(),
        }))?;
    } else {
        output.info(&format!("Saved filters ({}):", filters.filters.len()));
        output.info("");

        let mut filter_list: Vec<_> = filters.filters.values().collect();
        filter_list.sort_by(|a, b| a.name.cmp(&b.name));

        for filter in filter_list {
            output.info(&format!("  @{}", filter.name));
            output.info(&format!("    Expression: {}", filter.expression));
            if let Some(desc) = &filter.description {
                output.info(&format!("    Description: {desc}"));
            }
            output.info("");
        }
    }

    Ok(())
}

/// Handle filter show command
pub fn handle_filter_show(
    name: String,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let filters = SavedFilters::load(project_dir)?;

    let filter = filters
        .get(&name)
        .ok_or_else(|| VibeTicketError::custom(format!("Filter '{name}' not found")))?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "filter": filter,
        }))?;
    } else {
        output.info(&format!("Filter: @{}", filter.name));
        output.info(&format!("Expression: {}", filter.expression));
        if let Some(desc) = &filter.description {
            output.info(&format!("Description: {desc}"));
        }
        output.info(&format!(
            "Created: {}",
            filter.created_at.format("%Y-%m-%d %H:%M")
        ));
    }

    Ok(())
}

/// Handle filter delete command
pub fn handle_filter_delete(
    name: String,
    force: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let mut filters = SavedFilters::load(project_dir)?;

    if filters.get(&name).is_none() {
        return Err(VibeTicketError::custom(format!(
            "Filter '{name}' not found"
        )));
    }

    if !force {
        // In a real implementation, we'd prompt for confirmation
        // For now, just proceed
    }

    filters.remove(&name);
    filters.save(project_dir)?;

    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "deleted": name,
        }))?;
    } else {
        output.success(&format!("Deleted filter '{name}'"));
    }

    Ok(())
}

/// Handle filter apply command
pub fn handle_filter_apply(
    name: String,
    additional: Option<String>,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    let filters = SavedFilters::load(project_dir)?;

    let filter = filters
        .get(&name)
        .ok_or_else(|| VibeTicketError::custom(format!("Filter '{name}' not found")))?;

    // Combine with additional filter if provided
    let combined_expression = if let Some(additional_expr) = additional {
        format!("{} {}", filter.expression, additional_expr)
    } else {
        filter.expression.clone()
    };

    output.info(&format!("Applying filter '@{name}':"));
    output.info(&format!("Expression: {combined_expression}"));
    output.info("");

    // Parse the filter expression and call list with appropriate params
    // Parse status and priority from expression
    let mut status_filter = None;
    let mut priority_filter = None;

    for part in combined_expression.split_whitespace() {
        if let Some((key, value)) = part.split_once(':') {
            match key.to_lowercase().as_str() {
                "status" => status_filter = Some(value.to_string()),
                "priority" => priority_filter = Some(value.to_string()),
                _ => {}, // Ignore other filters for now
            }
        }
    }

    use crate::cli::handlers::list::handle_list_command;

    handle_list_command(
        status_filter,
        priority_filter,
        None, // assignee
        "slug",
        false, // reverse
        None,  // limit
        false, // archived
        false, // open
        None,  // since
        None,  // until
        false, // include_done
        project_dir,
        output,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saved_filter_serialization() {
        let filter = SavedFilter {
            name: "test".to_string(),
            expression: "status:todo".to_string(),
            description: Some("Test filter".to_string()),
            created_at: chrono::Utc::now(),
        };

        let yaml = serde_yaml::to_string(&filter).unwrap();
        let parsed: SavedFilter = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.name, filter.name);
        assert_eq!(parsed.expression, filter.expression);
    }
}
