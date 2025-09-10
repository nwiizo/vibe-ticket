//! Base utilities for spec command handlers
//!
//! This module extracts common patterns from spec handlers to reduce
//! code duplication and improve maintainability.

use crate::cli::output::OutputFormatter;
use crate::cli::utils::find_project_root;
use crate::error::{ErrorContext, Result, VibeTicketError};
use crate::specs::{
    SpecManager, SpecMetadata, SpecPhase, SpecProgress, SpecVersion, Specification,
};
use chrono::Utc;
use std::env;
use std::path::PathBuf;
use uuid::Uuid;

/// Context for spec operations
///
/// Encapsulates common initialization and resources for spec handlers
pub struct SpecContext {
    pub project_root: PathBuf,
    pub spec_manager: SpecManager,
    pub formatter: OutputFormatter,
}

impl SpecContext {
    /// Create a new spec context
    ///
    /// Handles project directory resolution and spec manager initialization
    pub fn new(project: Option<&str>, formatter: OutputFormatter) -> Result<Self> {
        // Change to project directory if specified
        if let Some(project_path) = project {
            env::set_current_dir(project_path).context("Failed to change to project directory")?;
        }

        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let project_root = find_project_root(current_dir.to_str())?;
        let spec_dir = project_root.join(".vibe-ticket").join("specs");

        // Ensure spec directory exists
        if !spec_dir.exists() {
            std::fs::create_dir_all(&spec_dir).context("Failed to create specs directory")?;
        }

        let spec_manager = SpecManager::new(spec_dir);

        Ok(Self {
            project_root,
            spec_manager,
            formatter,
        })
    }

    /// Load a specification by ID or get the active one
    pub fn load_spec(&self, spec_id: Option<String>) -> Result<Specification> {
        match spec_id {
            Some(id) => self.spec_manager.load(&id),
            None => {
                let active_id = self.spec_manager.get_active_spec()?
                    .ok_or_else(|| VibeTicketError::Custom(
                        "No active specification. Use --spec to specify one or activate a spec first.".to_string()
                    ))?;
                self.spec_manager.load(&active_id)
            },
        }
    }

    /// Load all specifications with optional filtering
    pub fn load_all_specs(&self, phase_filter: Option<SpecPhase>) -> Result<Vec<SpecMetadata>> {
        let specs = self.spec_manager.list()?;

        if let Some(phase) = phase_filter {
            Ok(specs
                .into_iter()
                .filter(|spec| spec.progress.current_phase == phase)
                .collect())
        } else {
            Ok(specs)
        }
    }

    /// Save a specification and optionally set it as active
    pub fn save_spec(&self, spec: &Specification, set_active: bool) -> Result<()> {
        self.spec_manager.save(spec)?;

        if set_active {
            self.spec_manager.set_active_spec(&spec.metadata.id)?;
        }

        Ok(())
    }

    /// Display success message
    pub fn success(&self, message: &str) {
        self.formatter.success(message);
    }

    /// Display error message
    pub fn error(&self, message: &str) {
        self.formatter.error(message);
    }

    /// Display info message
    pub fn info(&self, message: &str) {
        self.formatter.info(message);
    }

    /// Display warning message
    pub fn warning(&self, message: &str) {
        self.formatter.warning(message);
    }
}

/// Trait for common spec operations
pub trait SpecOperation {
    /// Execute the spec operation
    fn execute(&self, context: &SpecContext) -> Result<()>;

    /// Validate prerequisites for the operation
    fn validate(&self, _context: &SpecContext) -> Result<()> {
        // Default implementation - no validation
        Ok(())
    }

    /// Get operation name for logging
    fn name(&self) -> &str;
}

/// Builder for creating specifications
#[derive(Default)]
pub struct SpecBuilder {
    title: Option<String>,
    description: Option<String>,
    template: Option<String>,
}

impl SpecBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    #[must_use]
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    #[must_use]
    pub fn template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }

    pub fn build(self) -> Result<Specification> {
        let title = self.title.ok_or_else(|| {
            VibeTicketError::Custom("Specification title is required".to_string())
        })?;

        // Create basic specification metadata
        let metadata = SpecMetadata {
            id: Uuid::new_v4().to_string(),
            title: title.clone(),
            description: self.description.unwrap_or_default(),
            ticket_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            progress: SpecProgress::default(),
            version: SpecVersion::default(),
            tags: if let Some(template_name) = self.template {
                vec![format!("template:{}", template_name)]
            } else {
                Vec::new()
            },
        };

        Ok(Specification {
            metadata,
            requirements: None,
            design: None,
            tasks: None,
        })
    }
}

/// Common formatting for spec output
pub struct SpecFormatter;

impl SpecFormatter {
    /// Format specification status
    pub fn format_status(spec: &Specification, formatter: &OutputFormatter) {
        formatter.success(&format!("📋 Specification: {}", spec.metadata.title));
        formatter.info(&format!("   ID: {}", spec.metadata.id));
        formatter.info(&format!(
            "   Phase: {:?}",
            spec.metadata.progress.current_phase
        ));
        formatter.info(&format!(
            "   Version: {}.{}.{}",
            spec.metadata.version.major, spec.metadata.version.minor, spec.metadata.version.patch
        ));

        if !spec.metadata.tags.is_empty() {
            formatter.info(&format!("   Tags: {}", spec.metadata.tags.join(", ")));
        }
    }

    /// Format specification progress
    pub fn format_progress(spec: &Specification, formatter: &OutputFormatter) {
        let progress = &spec.metadata.progress;

        formatter.info("\n📊 Progress:");

        // Requirements phase
        Self::format_phase_status("Requirements", progress.requirements_completed, formatter);
        Self::format_phase_status("Design", progress.design_completed, formatter);
        Self::format_phase_status("Tasks", progress.tasks_completed, formatter);

        formatter.info("\n✅ Approvals:");
        Self::format_phase_status(
            "Requirements Approved",
            progress.requirements_approved,
            formatter,
        );
        Self::format_phase_status("Design Approved", progress.design_approved, formatter);
        Self::format_phase_status("Tasks Approved", progress.tasks_approved, formatter);
    }

    fn format_phase_status(phase: &str, complete: bool, formatter: &OutputFormatter) {
        let status = if complete { "✅" } else { "⏳" };
        formatter.info(&format!("   {status} {phase}"));
    }

    /// Format specification summary for lists
    #[must_use]
    pub fn format_summary(spec: &SpecMetadata) -> String {
        format!(
            "{} - {} [{:?}]",
            spec.id.chars().take(8).collect::<String>(),
            spec.title,
            spec.progress.current_phase
        )
    }
}

/// Common validation functions for spec operations
pub mod validation {
    use super::*;

    /// Validate that a specification exists
    pub fn spec_exists(spec_id: &str, context: &SpecContext) -> Result<()> {
        context
            .spec_manager
            .load(spec_id)
            .map(|_| ())
            .map_err(|_| VibeTicketError::Custom(format!("Specification '{spec_id}' not found")))
    }

    /// Validate that no active specification exists
    pub fn no_active_spec(context: &SpecContext) -> Result<()> {
        if context.spec_manager.get_active_spec()?.is_some() {
            Err(VibeTicketError::Custom(
                "An active specification already exists. Use --force to override.".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    /// Validate specification phase transition
    pub fn can_transition_phase(spec: &Specification, to_phase: SpecPhase) -> Result<()> {
        let progress = &spec.metadata.progress;
        let current = progress.current_phase;

        // Define valid transitions
        let valid_transition = match (current, to_phase) {
            (SpecPhase::Requirements, SpecPhase::Design) => progress.requirements_completed,
            (SpecPhase::Design, SpecPhase::Implementation) => progress.design_completed,
            (SpecPhase::Implementation, SpecPhase::Completed) => progress.tasks_completed,
            (SpecPhase::Tasks, SpecPhase::Completed) => progress.tasks_completed,
            _ => false,
        };

        if valid_transition {
            Ok(())
        } else {
            Err(VibeTicketError::Custom(format!(
                "Cannot transition from {current:?} to {to_phase:?} phase"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_spec_context_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let vibe_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir(&vibe_dir).unwrap();

        let formatter = OutputFormatter::new(false, false);
        let context = SpecContext::new(Some(temp_dir.path().to_str().unwrap()), formatter);

        assert!(context.is_ok());
    }

    #[test]
    fn test_spec_builder() {
        let spec = SpecBuilder::new()
            .title("Test Specification".to_string())
            .description("Test description".to_string())
            .build();

        assert!(spec.is_ok());
        let spec = spec.unwrap();
        assert_eq!(spec.metadata.title, "Test Specification");
        assert_eq!(spec.metadata.description, "Test description");
    }

    #[test]
    fn test_spec_builder_requires_title() {
        let spec = SpecBuilder::new()
            .description("Test description".to_string())
            .build();

        assert!(spec.is_err());
    }
}
