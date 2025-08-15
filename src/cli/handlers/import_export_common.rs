use crate::core::Ticket;
use crate::error::{Result, VibeTicketError};
use serde_json::Value;

/// Common format detection and conversion utilities
pub struct FormatUtils;

impl FormatUtils {
    /// Detect format from content
    pub fn detect_format(content: &str) -> Result<DataFormat> {
        let trimmed = content.trim();
        
        // Try JSON first
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if serde_json::from_str::<Value>(trimmed).is_ok() {
                return Ok(DataFormat::Json);
            }
        }
        
        // Try YAML
        if serde_yaml::from_str::<Value>(trimmed).is_ok() {
            return Ok(DataFormat::Yaml);
        }
        
        // Try CSV (simple check for comma-separated values)
        if trimmed.contains(',') && trimmed.lines().count() > 1 {
            return Ok(DataFormat::Csv);
        }
        
        Err(VibeTicketError::InvalidInput(
            "Unable to detect format. Content must be valid JSON, YAML, or CSV".to_string()
        ))
    }
    
    /// Parse JSON content into tickets
    pub fn parse_json(content: &str) -> Result<Vec<Ticket>> {
        serde_json::from_str(content)
            .map_err(|e| VibeTicketError::ParseError(format!("Invalid JSON: {}", e)))
    }
    
    /// Parse YAML content into tickets
    pub fn parse_yaml(content: &str) -> Result<Vec<Ticket>> {
        serde_yaml::from_str(content)
            .map_err(|e| VibeTicketError::ParseError(format!("Invalid YAML: {}", e)))
    }
    
    /// Parse CSV content into tickets
    pub fn parse_csv(content: &str) -> Result<Vec<Ticket>> {
        use csv::ReaderBuilder;
        use crate::core::{TicketId, Priority, Status};
        
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());
        
        let mut tickets = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| VibeTicketError::ParseError(format!("CSV error: {}", e)))?;
            
            // Parse required fields
            let id = TicketId::parse_str(&record[0])
                .map_err(|e| VibeTicketError::ParseError(format!("Invalid ID: {}", e)))?;
            let slug = record[1].to_string();
            let title = record[2].to_string();
            
            // Create ticket with builder
            use crate::core::TicketBuilder;
            let mut builder = TicketBuilder::new()
                .id(id)
                .slug(slug)
                .title(title);
            
            // Add optional fields if present
            if record.len() > 3 && !record[3].is_empty() {
                builder = builder.description(record[3].to_string());
            }
            if record.len() > 4 && !record[4].is_empty() {
                if let Ok(priority) = record[4].parse::<Priority>() {
                    builder = builder.priority(priority);
                }
            }
            if record.len() > 5 && !record[5].is_empty() {
                if let Ok(status) = record[5].parse::<Status>() {
                    builder = builder.status(status);
                }
            }
            
            tickets.push(builder.build());
        }
        
        Ok(tickets)
    }
    
    /// Export tickets to JSON
    pub fn export_json(tickets: &[Ticket]) -> Result<String> {
        serde_json::to_string_pretty(tickets)
            .map_err(|e| VibeTicketError::SerializationError(format!("Failed to serialize to JSON: {}", e)))
    }
    
    /// Export tickets to YAML
    pub fn export_yaml(tickets: &[Ticket]) -> Result<String> {
        serde_yaml::to_string(tickets)
            .map_err(|e| VibeTicketError::SerializationError(format!("Failed to serialize to YAML: {}", e)))
    }
    
    /// Export tickets to CSV
    pub fn export_csv(tickets: &[Ticket]) -> Result<String> {
        use csv::Writer;
        let mut writer = Writer::from_writer(vec![]);
        
        // Write header
        writer.write_record(&["id", "slug", "title", "description", "priority", "status", "tags", "assignee"])
            .map_err(|e| VibeTicketError::SerializationError(format!("Failed to write CSV header: {}", e)))?;
        
        // Write tickets
        for ticket in tickets {
            writer.write_record(&[
                ticket.id.to_string(),
                ticket.slug.clone(),
                ticket.title.clone(),
                ticket.description.clone(),
                ticket.priority.to_string(),
                ticket.status.to_string(),
                ticket.tags.join(","),
                ticket.assignee.clone().unwrap_or_default(),
            ])
            .map_err(|e| VibeTicketError::SerializationError(format!("Failed to write CSV record: {}", e)))?;
        }
        
        writer.flush()
            .map_err(|e| VibeTicketError::SerializationError(format!("Failed to flush CSV: {}", e)))?;
        
        String::from_utf8(writer.into_inner()
            .map_err(|e| VibeTicketError::SerializationError(format!("Failed to get CSV data: {}", e)))?)
            .map_err(|e| VibeTicketError::SerializationError(format!("Invalid UTF-8 in CSV: {}", e)))
    }
    
    /// Export tickets to Markdown
    pub fn export_markdown(tickets: &[Ticket]) -> Result<String> {
        use std::fmt::Write;
        let mut output = String::new();
        
        writeln!(&mut output, "# Tickets Export\n").unwrap();
        writeln!(&mut output, "Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")).unwrap();
        
        for ticket in tickets {
            writeln!(&mut output, "## {} - {}\n", ticket.slug, ticket.title).unwrap();
            writeln!(&mut output, "- **ID**: {}", ticket.id).unwrap();
            writeln!(&mut output, "- **Status**: {}", ticket.status).unwrap();
            writeln!(&mut output, "- **Priority**: {}", ticket.priority).unwrap();
            
            if !ticket.description.is_empty() {
                writeln!(&mut output, "\n### Description\n").unwrap();
                writeln!(&mut output, "{}\n", ticket.description).unwrap();
            }
            
            if !ticket.tasks.is_empty() {
                writeln!(&mut output, "### Tasks\n").unwrap();
                for task in &ticket.tasks {
                    let checkbox = if task.completed { "x" } else { " " };
                    writeln!(&mut output, "- [{}] {}", checkbox, task.title).unwrap();
                }
                writeln!(&mut output).unwrap();
            }
            
            writeln!(&mut output, "---\n").unwrap();
        }
        
        Ok(output)
    }
}

/// Supported data formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataFormat {
    Json,
    Yaml,
    Csv,
    Markdown,
}

impl DataFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Csv => "csv",
            Self::Markdown => "md",
        }
    }
    
    /// Parse content based on format
    pub fn parse(&self, content: &str) -> Result<Vec<Ticket>> {
        match self {
            Self::Json => FormatUtils::parse_json(content),
            Self::Yaml => FormatUtils::parse_yaml(content),
            Self::Csv => FormatUtils::parse_csv(content),
            Self::Markdown => Err(VibeTicketError::InvalidInput(
                "Cannot import from Markdown format".to_string()
            )),
        }
    }
    
    /// Export tickets based on format
    pub fn export(&self, tickets: &[Ticket]) -> Result<String> {
        match self {
            Self::Json => FormatUtils::export_json(tickets),
            Self::Yaml => FormatUtils::export_yaml(tickets),
            Self::Csv => FormatUtils::export_csv(tickets),
            Self::Markdown => FormatUtils::export_markdown(tickets),
        }
    }
}

/// Validate imported tickets
pub fn validate_tickets(tickets: &[Ticket]) -> Result<()> {
    if tickets.is_empty() {
        return Err(VibeTicketError::InvalidInput(
            "No tickets found in import data".to_string()
        ));
    }
    
    // Check for duplicate IDs
    let mut seen_ids = std::collections::HashSet::new();
    for ticket in tickets {
        if !seen_ids.insert(ticket.id) {
            return Err(VibeTicketError::DuplicateTicket {
                slug: ticket.slug.clone()
            });
        }
    }
    
    // Check for duplicate slugs
    let mut seen_slugs = std::collections::HashSet::new();
    for ticket in tickets {
        if !seen_slugs.insert(&ticket.slug) {
            return Err(VibeTicketError::DuplicateTicket {
                slug: ticket.slug.clone()
            });
        }
    }
    
    Ok(())
}