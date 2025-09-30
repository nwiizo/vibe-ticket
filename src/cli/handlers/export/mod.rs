//! Export handlers refactored with trait-based approach
//!
//! This module provides a unified interface for exporting tickets
//! to various formats using the Exporter trait.

mod csv;
mod json;
mod markdown;
mod yaml;

use crate::cli::{OutputFormatter, find_project_root};
use crate::core::Ticket;
use crate::error::{Result, VibeTicketError};
use crate::storage::{FileStorage, TicketRepository};
use chrono::{DateTime, Utc};
use serde::Serialize;

pub use self::csv::CsvExporter;
pub use self::json::JsonExporter;
pub use self::markdown::MarkdownExporter;
pub use self::yaml::YamlExporter;

/// Common metadata structure for JSON and YAML exports
#[derive(Debug, Serialize)]
pub struct ExportMetadata {
    pub tickets: Vec<Ticket>,
    pub exported_at: DateTime<Utc>,
    pub total: usize,
}

impl ExportMetadata {
    /// Create new export metadata with the given tickets
    pub fn new(tickets: Vec<Ticket>) -> Self {
        let total = tickets.len();
        Self {
            tickets,
            exported_at: Utc::now(),
            total,
        }
    }
}

/// Trait for ticket exporters
pub trait Exporter {
    /// Export tickets to the target format
    fn export(&self, tickets: &[Ticket]) -> Result<String>;

    /// Get the format name for display
    fn format_name(&self) -> &'static str;
}

/// Handler for the `export` command
///
/// Exports tickets to various formats using the appropriate exporter
pub fn handle_export_command(
    format: &str,
    output_path: Option<String>,
    include_archived: bool,
    project_dir: Option<&str>,
    output: &OutputFormatter,
) -> Result<()> {
    // Get exporter for the format
    let exporter: Box<dyn Exporter> = match format.to_lowercase().as_str() {
        "json" => Box::new(JsonExporter),
        "yaml" => Box::new(YamlExporter),
        "csv" => Box::new(CsvExporter),
        "markdown" | "md" => Box::new(MarkdownExporter),
        _ => {
            return Err(VibeTicketError::custom(format!(
                "Unsupported export format: {format}. Supported formats: json, yaml, csv, markdown"
            )));
        },
    };

    // Load and filter tickets
    let tickets = load_tickets(project_dir, include_archived)?;

    // Export using the appropriate exporter
    let content = exporter.export(&tickets)?;

    // Output results
    output_results(
        content,
        output_path,
        tickets.len(),
        exporter.format_name(),
        include_archived,
        output,
    )
}

/// Load tickets from storage
fn load_tickets(project_dir: Option<&str>, include_archived: bool) -> Result<Vec<Ticket>> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);

    let mut tickets = storage.load_all()?;

    // Filter out archived tickets if not included
    if !include_archived {
        tickets.retain(|t| {
            !t.metadata
                .get("archived")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        });
    }

    // Sort tickets by creation date
    tickets.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    Ok(tickets)
}

/// Output export results
fn output_results(
    content: String,
    output_path: Option<String>,
    ticket_count: usize,
    format_name: &str,
    include_archived: bool,
    output: &OutputFormatter,
) -> Result<()> {
    if let Some(path) = output_path {
        std::fs::write(&path, content)
            .map_err(|e| VibeTicketError::io_error("write", std::path::Path::new(&path), e))?;

        output.success(&format!("Exported {ticket_count} tickets to {path}"));
        output.info(&format!("Format: {format_name}"));
        if !include_archived {
            output.info(
                "Note: Archived tickets were excluded. Use --include-archived to include them.",
            );
        }
    } else {
        // Output to stdout
        println!("{content}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Priority, Status, TicketId};
    use chrono::Utc;

    fn create_test_ticket() -> Ticket {
        Ticket {
            id: TicketId::new(),
            slug: "test-ticket".to_string(),
            title: "Test Ticket".to_string(),
            description: "Test description".to_string(),
            status: Status::Todo,
            priority: Priority::Medium,
            tags: vec!["test".to_string()],
            assignee: None,
            tasks: vec![],
            metadata: std::collections::HashMap::default(),
            created_at: Utc::now(),
            started_at: None,
            closed_at: None,
        }
    }

    /// Test macro to reduce boilerplate for exporter tests
    macro_rules! test_exporter {
        ($name:ident, $exporter:expr, $contains:expr) => {
            #[test]
            fn $name() {
                let tickets = vec![create_test_ticket()];
                let result = $exporter.export(&tickets);
                assert!(result.is_ok(), "Export should succeed");
                let output = result.unwrap();
                assert!(
                    output.contains($contains),
                    "Expected output to contain '{}', but got: {}",
                    $contains,
                    output
                );
            }
        };
    }

    test_exporter!(test_json_exporter, JsonExporter, "\"total\": 1");
    test_exporter!(test_csv_exporter, CsvExporter, "test-ticket");
    test_exporter!(test_yaml_exporter, YamlExporter, "total: 1");
    test_exporter!(test_markdown_exporter, MarkdownExporter, "# Ticket Export");
}
