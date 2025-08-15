use crate::cli::output::OutputFormatter;
use crate::error::{Result, VibeTicketError, ErrorContext};
use crate::specs::{Specification, SpecManager, SpecPhase, SpecDocumentType};
use std::env;
use std::path::PathBuf;

/// Common context for spec operations
pub struct SpecContext {
    pub spec_manager: SpecManager,
    pub formatter: OutputFormatter,
    pub project_dir: PathBuf,
}

impl SpecContext {
    /// Create a new spec context
    pub fn new(project: Option<&str>, formatter: OutputFormatter) -> Result<Self> {
        // Change to project directory if specified
        if let Some(project_path) = project {
            std::env::set_current_dir(project_path)
                .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
        }

        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let project_dir = current_dir.join(".vibe-ticket");

        if !project_dir.exists() {
            return Err(VibeTicketError::ProjectNotInitialized);
        }

        let spec_manager = SpecManager::new(project_dir.join("specs"));

        Ok(Self {
            spec_manager,
            formatter,
            project_dir,
        })
    }
    
    /// Parse tags from comma-separated string
    pub fn parse_tags(tags: Option<String>) -> Vec<String> {
        tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }
    
    /// Common success output for spec operations
    pub fn output_spec_success(&self, action: &str, spec: &Specification) -> Result<()> {
        if self.formatter.is_json() {
            self.formatter.print_json(&serde_json::json!({
                "status": "success",
                "action": action,
                "spec": {
                    "id": spec.metadata.id,
                    "title": spec.metadata.title,
                    "phase": spec.metadata.progress.current_phase,
                    "tags": spec.metadata.tags,
                    "ticket": spec.metadata.ticket_id,
                }
            }))?;
        } else {
            self.formatter.success(&format!("{} specification: {}", action, spec.metadata.title));
            self.formatter.info(&format!("ID: {}", spec.metadata.id));
            self.formatter.info(&format!("Phase: {:?}", spec.metadata.progress.current_phase));
            if !spec.metadata.tags.is_empty() {
                self.formatter.info(&format!("Tags: {}", spec.metadata.tags.join(", ")));
            }
            if let Some(ticket) = &spec.metadata.ticket_id {
                self.formatter.info(&format!("Linked to ticket: {}", ticket));
            }
        }
        Ok(())
    }
    
    /// Common list output for specifications
    pub fn output_spec_list(&self, specs: Vec<Specification>) -> Result<()> {
        if self.formatter.is_json() {
            self.formatter.print_json(&specs)?;
        } else if specs.is_empty() {
            self.formatter.info("No specifications found");
        } else {
            self.formatter.info(&format!("Found {} specification(s):", specs.len()));
            for spec in specs {
                println!("\n{}", "=".repeat(60));
                println!("Title: {}", spec.metadata.title);
                println!("ID: {}", spec.metadata.id);
                println!("Phase: {:?}", spec.metadata.progress.current_phase);
                if !spec.metadata.description.is_empty() {
                    println!("Description: {}", spec.metadata.description);
                }
                if !spec.metadata.tags.is_empty() {
                    println!("Tags: {}", spec.metadata.tags.join(", "));
                }
                if let Some(ticket) = &spec.metadata.ticket_id {
                    println!("Ticket: {}", ticket);
                }
            }
        }
        Ok(())
    }
}

/// Template for spec phase operations (requirements, design, tasks)
pub trait SpecPhaseHandler {
    fn get_phase(&self) -> SpecPhase;
    fn get_phase_name(&self) -> &str;
    
    fn handle_phase_operation(
        &self,
        spec_id: String,
        editor: Option<String>,
        project: Option<String>,
        formatter: &OutputFormatter,
    ) -> Result<()> {
        let ctx = SpecContext::new(project.as_deref(), formatter.clone())?;
        
        // Load existing spec or create new one
        let mut spec = match ctx.spec_manager.load_spec(&spec_id) {
            Ok(s) => s,
            Err(_) => {
                return Err(VibeTicketError::SpecNotFound {
                    id: spec_id.clone(),
                });
            }
        };
        
        // Update phase
        spec.metadata.progress.current_phase = self.get_phase();
        ctx.spec_manager.save(&spec)?;
        
        // Save phase document - needs spec_id and doc_type
        let doc_type = match self.get_phase() {
            SpecPhase::Requirements => crate::specs::SpecDocumentType::Requirements,
            SpecPhase::Design => crate::specs::SpecDocumentType::Design,
            SpecPhase::Tasks | SpecPhase::Implementation => crate::specs::SpecDocumentType::Tasks,
            _ => crate::specs::SpecDocumentType::Requirements,
        };
        let doc_path = ctx.spec_manager.save_document(&spec_id, doc_type, "")?;
        
        // Open in editor if requested
        if let Some(editor_cmd) = editor.or_else(|| std::env::var("EDITOR").ok()) {
            let _ = editor_cmd; // Use editor_cmd if needed
            // Note: open_in_editor expects just a Path, not editor command
            // This would need to be refactored to properly use the editor
        }
        
        ctx.output_spec_success(&format!("Updated {} for", self.get_phase_name()), &spec)?;
        Ok(())
    }
}

/// Implementation for requirements phase
pub struct RequirementsHandler;

impl SpecPhaseHandler for RequirementsHandler {
    fn get_phase(&self) -> SpecPhase {
        SpecPhase::Requirements
    }
    
    fn get_phase_name(&self) -> &str {
        "requirements"
    }
}

/// Implementation for design phase
pub struct DesignHandler;

impl SpecPhaseHandler for DesignHandler {
    fn get_phase(&self) -> SpecPhase {
        SpecPhase::Design
    }
    
    fn get_phase_name(&self) -> &str {
        "design"
    }
}

/// Implementation for tasks phase
pub struct TasksHandler;

impl SpecPhaseHandler for TasksHandler {
    fn get_phase(&self) -> SpecPhase {
        SpecPhase::Tasks
    }
    
    fn get_phase_name(&self) -> &str {
        "tasks"
    }
}