//! Spec-Driven Development (仕様駆動開発) module
//!
//! This module implements a specification-driven development workflow inspired by Kiro.
//! It manages three types of documents:
//! - Requirements Definition (要件定義書)
//! - Technical Design (技術設計書)
//! - Implementation Plan (実装計画書)
//!
//! # Architecture
//!
//! The specs are stored as Markdown files in `.vibe-ticket/specs/` directory
//! with a `spec.json` file tracking the progress and metadata.
//!
//! # Workflow
//!
//! 1. Initialize a new spec
//! 2. Create requirements definition
//! 3. Create technical design based on requirements
//! 4. Create implementation plan based on design
//! 5. Track progress through spec.json

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod manager;
pub mod storage;
pub mod templates;

pub use manager::{SpecManager, delete, get_document_path, list, load, save};
pub use templates::{SpecTemplate, TemplateEngine};

/// Specification metadata and progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    /// Unique spec ID
    pub id: String,

    /// Spec title
    pub title: String,

    /// Brief description
    pub description: String,

    /// Associated ticket ID (if any)
    pub ticket_id: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Progress tracking
    pub progress: SpecProgress,

    /// Version information
    pub version: SpecVersion,

    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Progress tracking for spec documents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct SpecProgress {
    /// Requirements definition completed
    pub requirements_completed: bool,

    /// Technical design completed
    pub design_completed: bool,

    /// Implementation plan completed
    pub tasks_completed: bool,

    /// Requirements approval status
    pub requirements_approved: bool,

    /// Design approval status
    pub design_approved: bool,

    /// Tasks approval status
    pub tasks_approved: bool,

    /// Current phase
    pub current_phase: SpecPhase,

    /// Approval status with additional metadata
    pub approval_status: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Current phase of the specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecPhase {
    /// Initial phase - no documents created yet
    Initial,

    /// Requirements definition phase
    Requirements,

    /// Technical design phase
    Design,

    /// Implementation planning phase
    Implementation,

    /// Tasks phase (alias for Implementation)
    Tasks,

    /// All phases completed
    Completed,
}

/// Version information for spec documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecVersion {
    /// Major version (breaking changes)
    pub major: u32,

    /// Minor version (new features)
    pub minor: u32,

    /// Patch version (bug fixes)
    pub patch: u32,
}

/// Specification document type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecDocumentType {
    /// Requirements definition document
    Requirements,

    /// Technical design document
    Design,

    /// Implementation plan/tasks document
    Tasks,
}

/// A complete specification with all documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specification {
    /// Metadata
    pub metadata: SpecMetadata,

    /// Requirements definition content
    pub requirements: Option<String>,

    /// Technical design content
    pub design: Option<String>,

    /// Implementation plan content
    pub tasks: Option<String>,
}

impl Specification {
    /// Create a new specification
    #[must_use]
    pub fn new(
        title: String,
        description: String,
        ticket_id: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        let mut metadata = SpecMetadata::new(title, description);
        metadata.ticket_id = ticket_id;
        metadata.tags = tags;

        Self {
            metadata,
            requirements: None,
            design: None,
            tasks: None,
        }
    }
}

impl SpecMetadata {
    /// Create new spec metadata
    #[must_use]
    pub fn new(title: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            ticket_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            progress: SpecProgress::default(),
            version: SpecVersion::default(),
            tags: Vec::new(),
        }
    }

    /// Update the current phase based on progress
    pub fn update_phase(&mut self) {
        self.progress.current_phase = match (
            self.progress.requirements_completed,
            self.progress.design_completed,
            self.progress.tasks_completed,
        ) {
            (false, _, _) => SpecPhase::Requirements,
            (true, false, _) => SpecPhase::Design,
            (true, true, false) => SpecPhase::Implementation,
            (true, true, true) => SpecPhase::Completed,
        };
        self.updated_at = Utc::now();
    }
}

impl SpecProgress {
    /// Get the current phase
    #[must_use]
    pub const fn current_phase(&self) -> SpecPhase {
        self.current_phase
    }
}

impl Default for SpecProgress {
    fn default() -> Self {
        Self {
            requirements_completed: false,
            design_completed: false,
            tasks_completed: false,
            requirements_approved: false,
            design_approved: false,
            tasks_approved: false,
            current_phase: SpecPhase::Initial,
            approval_status: None,
        }
    }
}

impl Default for SpecVersion {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 1,
            patch: 0,
        }
    }
}

impl std::fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl SpecVersion {
    /// Increment patch version
    pub const fn bump_patch(&mut self) {
        self.patch += 1;
    }

    /// Increment minor version (resets patch)
    pub const fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    /// Increment major version (resets minor and patch)
    pub const fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }
}

impl SpecDocumentType {
    /// Get file name for this document type
    #[must_use]
    pub const fn file_name(&self) -> &'static str {
        match self {
            Self::Requirements => "requirements.md",
            Self::Design => "design.md",
            Self::Tasks => "tasks.md",
        }
    }

    /// Get display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Requirements => "Requirements Definition",
            Self::Design => "Technical Design",
            Self::Tasks => "Implementation Plan",
        }
    }
}

impl std::fmt::Display for SpecPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initial => write!(f, "Initial"),
            Self::Requirements => write!(f, "Requirements Definition"),
            Self::Design => write!(f, "Technical Design"),
            Self::Implementation => write!(f, "Implementation Planning"),
            Self::Tasks => write!(f, "Tasks"),
            Self::Completed => write!(f, "Completed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_spec_metadata_new() {
        let metadata = SpecMetadata::new("Test Spec".to_string(), "Test Description".to_string());

        assert_eq!(metadata.title, "Test Spec");
        assert_eq!(metadata.description, "Test Description");
        assert!(metadata.ticket_id.is_none());
        assert!(metadata.tags.is_empty());
        assert_eq!(metadata.progress.current_phase, SpecPhase::Initial);
        assert_eq!(metadata.version.to_string(), "0.1.0");
    }

    #[test]
    fn test_spec_metadata_update_phase() {
        let mut metadata = SpecMetadata::new("Test".to_string(), "Description".to_string());

        // Initial phase
        assert_eq!(metadata.progress.current_phase, SpecPhase::Initial);

        // Complete requirements
        metadata.progress.requirements_completed = true;
        metadata.update_phase();
        assert_eq!(metadata.progress.current_phase, SpecPhase::Design);

        // Complete design
        metadata.progress.design_completed = true;
        metadata.update_phase();
        assert_eq!(metadata.progress.current_phase, SpecPhase::Implementation);

        // Complete tasks
        metadata.progress.tasks_completed = true;
        metadata.update_phase();
        assert_eq!(metadata.progress.current_phase, SpecPhase::Completed);
    }

    #[test]
    fn test_specification_new() {
        let spec = Specification::new(
            "Test Spec".to_string(),
            "Description".to_string(),
            Some("ticket-123".to_string()),
            vec!["tag1".to_string(), "tag2".to_string()],
        );

        assert_eq!(spec.metadata.title, "Test Spec");
        assert_eq!(spec.metadata.description, "Description");
        assert_eq!(spec.metadata.ticket_id, Some("ticket-123".to_string()));
        assert_eq!(
            spec.metadata.tags,
            vec!["tag1".to_string(), "tag2".to_string()]
        );
        assert!(spec.requirements.is_none());
        assert!(spec.design.is_none());
        assert!(spec.tasks.is_none());
    }

    #[test]
    fn test_spec_progress_default() {
        let progress = SpecProgress::default();

        assert!(!progress.requirements_completed);
        assert!(!progress.design_completed);
        assert!(!progress.tasks_completed);
        assert!(!progress.requirements_approved);
        assert!(!progress.design_approved);
        assert!(!progress.tasks_approved);
        assert_eq!(progress.current_phase, SpecPhase::Initial);
        assert!(progress.approval_status.is_none());
    }

    #[test]
    fn test_spec_progress_current_phase() {
        let progress = SpecProgress::default();
        assert_eq!(progress.current_phase(), SpecPhase::Initial);
    }

    #[test]
    fn test_spec_version_default() {
        let version = SpecVersion::default();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 0);
        assert_eq!(version.to_string(), "0.1.0");
    }

    #[test]
    fn test_spec_version_bump() {
        let mut version = SpecVersion::default();

        // Bump patch
        version.bump_patch();
        assert_eq!(version.to_string(), "0.1.1");

        // Bump minor (resets patch)
        version.bump_minor();
        assert_eq!(version.to_string(), "0.2.0");

        // Bump major (resets minor and patch)
        version.bump_major();
        assert_eq!(version.to_string(), "1.0.0");
    }

    #[test]
    fn test_spec_document_type_file_name() {
        assert_eq!(
            SpecDocumentType::Requirements.file_name(),
            "requirements.md"
        );
        assert_eq!(SpecDocumentType::Design.file_name(), "design.md");
        assert_eq!(SpecDocumentType::Tasks.file_name(), "tasks.md");
    }

    #[test]
    fn test_spec_document_type_display_name() {
        assert_eq!(
            SpecDocumentType::Requirements.display_name(),
            "Requirements Definition"
        );
        assert_eq!(SpecDocumentType::Design.display_name(), "Technical Design");
        assert_eq!(
            SpecDocumentType::Tasks.display_name(),
            "Implementation Plan"
        );
    }

    #[test]
    fn test_spec_phase_display() {
        assert_eq!(SpecPhase::Initial.to_string(), "Initial");
        assert_eq!(
            SpecPhase::Requirements.to_string(),
            "Requirements Definition"
        );
        assert_eq!(SpecPhase::Design.to_string(), "Technical Design");
        assert_eq!(
            SpecPhase::Implementation.to_string(),
            "Implementation Planning"
        );
        assert_eq!(SpecPhase::Tasks.to_string(), "Tasks");
        assert_eq!(SpecPhase::Completed.to_string(), "Completed");
    }

    #[test]
    fn test_spec_phase_equality() {
        assert_eq!(SpecPhase::Initial, SpecPhase::Initial);
        assert_ne!(SpecPhase::Initial, SpecPhase::Requirements);
        assert_ne!(SpecPhase::Tasks, SpecPhase::Implementation);
    }

    #[test]
    fn test_spec_progress_with_approval_status() {
        let mut progress = SpecProgress::default();

        // Add approval status
        let mut approval_status = HashMap::new();
        approval_status.insert(
            "approved_by".to_string(),
            serde_json::json!("reviewer@example.com"),
        );
        approval_status.insert(
            "approved_at".to_string(),
            serde_json::json!("2024-01-01T00:00:00Z"),
        );

        progress.approval_status = Some(approval_status);

        assert!(progress.approval_status.is_some());
        let status = progress.approval_status.as_ref().unwrap();
        assert_eq!(
            status.get("approved_by").unwrap(),
            &serde_json::json!("reviewer@example.com")
        );
    }

    #[test]
    fn test_spec_metadata_with_all_fields() {
        let mut metadata = SpecMetadata::new("Complete Spec".to_string(), "Full test".to_string());

        metadata.ticket_id = Some("ticket-456".to_string());
        metadata.tags = vec!["backend".to_string(), "api".to_string()];
        metadata.version.bump_minor();

        // Complete all phases
        metadata.progress.requirements_completed = true;
        metadata.progress.requirements_approved = true;
        metadata.progress.design_completed = true;
        metadata.progress.design_approved = true;
        metadata.progress.tasks_completed = true;
        metadata.progress.tasks_approved = true;
        metadata.update_phase();

        assert_eq!(metadata.progress.current_phase, SpecPhase::Completed);
        assert_eq!(metadata.version.to_string(), "0.2.0");
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_spec_serialization() {
        let spec = Specification::new(
            "Serialization Test".to_string(),
            "Test serialization".to_string(),
            None,
            vec![],
        );

        // Serialize to JSON
        let json = serde_json::to_string(&spec).unwrap();

        // Deserialize back
        let deserialized: Specification = serde_json::from_str(&json).unwrap();

        assert_eq!(spec.metadata.id, deserialized.metadata.id);
        assert_eq!(spec.metadata.title, deserialized.metadata.title);
        assert_eq!(spec.metadata.description, deserialized.metadata.description);
    }

    #[test]
    fn test_spec_with_documents() {
        let mut spec = Specification::new(
            "Doc Test".to_string(),
            "Test with documents".to_string(),
            None,
            vec![],
        );

        // Add documents
        spec.requirements = Some("# Requirements\n\nTest requirements".to_string());
        spec.design = Some("# Design\n\nTest design".to_string());
        spec.tasks = Some("# Tasks\n\n- [ ] Task 1\n- [ ] Task 2".to_string());

        assert!(spec.requirements.is_some());
        assert!(spec.design.is_some());
        assert!(spec.tasks.is_some());

        assert!(spec.requirements.as_ref().unwrap().contains("Requirements"));
        assert!(spec.design.as_ref().unwrap().contains("Design"));
        assert!(spec.tasks.as_ref().unwrap().contains("Task 1"));
    }
}
