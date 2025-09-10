//! Specification manager for handling spec lifecycle
//!
//! This module provides the core functionality for managing specifications,
//! including creation, loading, saving, and version control.

use super::{SpecDocumentType, SpecMetadata, SpecPhase, Specification};
use crate::error::{Result, VibeTicketError};
use crate::specs::storage::{DocumentOperations, FileSystemStore};
use std::path::PathBuf;

/// Manages specifications in a project
pub struct SpecManager {
    /// Document operations helper
    ops: DocumentOperations<FileSystemStore>,
}

impl SpecManager {
    /// Create a new spec manager
    #[must_use]
    pub const fn new(specs_dir: PathBuf) -> Self {
        Self {
            ops: DocumentOperations::new(FileSystemStore, specs_dir),
        }
    }

    /// Initialize the specs directory structure
    pub fn initialize(&self) -> Result<()> {
        self.ops.initialize()
    }

    /// Create a new specification
    pub fn create_spec(&self, title: String, description: String) -> Result<SpecMetadata> {
        self.initialize()?;

        let metadata = SpecMetadata::new(title, description);

        // Save initial metadata
        self.save_metadata(&metadata)?;

        Ok(metadata)
    }

    /// Load a specification by ID
    pub fn load_spec(&self, spec_id: &str) -> Result<Specification> {
        let metadata = self.load_metadata(spec_id)?;

        // Load document contents
        let requirements = self.load_document(spec_id, SpecDocumentType::Requirements)?;
        let design = self.load_document(spec_id, SpecDocumentType::Design)?;
        let tasks = self.load_document(spec_id, SpecDocumentType::Tasks)?;

        Ok(Specification {
            metadata,
            requirements,
            design,
            tasks,
        })
    }

    /// Save a document for a specification
    pub fn save_document(
        &self,
        spec_id: &str,
        doc_type: SpecDocumentType,
        content: &str,
    ) -> Result<()> {
        // Save document
        self.ops
            .save_text_in_subdir(spec_id, doc_type.file_name(), content)?;

        // Update metadata
        let mut metadata = self.load_metadata(spec_id)?;
        match doc_type {
            SpecDocumentType::Requirements => {
                metadata.progress.requirements_completed = true;
                metadata.version.bump_patch();
            },
            SpecDocumentType::Design => {
                metadata.progress.design_completed = true;
                metadata.version.bump_patch();
            },
            SpecDocumentType::Tasks => {
                metadata.progress.tasks_completed = true;
                metadata.version.bump_patch();
            },
        }
        metadata.update_phase();
        self.save_metadata(&metadata)?;

        Ok(())
    }

    /// List all specifications
    pub fn list_specs(&self) -> Result<Vec<SpecMetadata>> {
        let spec_dirs = self.ops.list_subdirs()?;
        let mut specs = Vec::new();

        for spec_dir in spec_dirs {
            let Some(dir_name) = spec_dir.file_name() else {
                continue;
            };

            let Some(spec_id) = dir_name.to_str() else {
                continue;
            };

            match self.load_metadata(spec_id) {
                Ok(metadata) => specs.push(metadata),
                Err(e) => eprintln!("Warning: Failed to load spec {spec_id}: {e}"),
            }
        }

        // Sort by creation date (newest first)
        specs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(specs)
    }

    /// Approve a document phase
    pub fn approve_phase(&self, spec_id: &str, phase: SpecPhase) -> Result<()> {
        let mut metadata = self.load_metadata(spec_id)?;

        // Validate phase can be approved
        match phase {
            SpecPhase::Requirements => {
                if !metadata.progress.requirements_completed {
                    return Err(VibeTicketError::custom(
                        "Cannot approve requirements: document not completed",
                    ));
                }
                metadata.progress.requirements_approved = true;
            },
            SpecPhase::Design => {
                if !metadata.progress.design_completed {
                    return Err(VibeTicketError::custom(
                        "Cannot approve design: document not completed",
                    ));
                }
                metadata.progress.design_approved = true;
            },
            SpecPhase::Implementation => {
                if !metadata.progress.tasks_completed {
                    return Err(VibeTicketError::custom(
                        "Cannot approve tasks: document not completed",
                    ));
                }
                metadata.progress.tasks_approved = true;
            },
            _ => {
                return Err(VibeTicketError::custom("Invalid phase for approval"));
            },
        }

        metadata.update_phase();
        self.save_metadata(&metadata)?;

        Ok(())
    }

    /// Get the directory path for a spec
    #[must_use]
    pub fn get_spec_dir(&self, spec_id: &str) -> PathBuf {
        self.ops.get_subdir(spec_id)
    }

    /// Load metadata for a spec
    fn load_metadata(&self, spec_id: &str) -> Result<SpecMetadata> {
        self.ops.load_from_subdir(spec_id, "spec.json")
    }

    /// Save metadata for a spec
    fn save_metadata(&self, metadata: &SpecMetadata) -> Result<()> {
        self.ops.save_in_subdir(&metadata.id, "spec.json", metadata)
    }

    /// Load a document from a spec directory
    fn load_document(&self, spec_id: &str, doc_type: SpecDocumentType) -> Result<Option<String>> {
        self.ops
            .load_text_from_subdir(spec_id, doc_type.file_name())
    }

    /// Find spec by title (partial match)
    pub fn find_spec_by_title(&self, query: &str) -> Result<Option<SpecMetadata>> {
        let specs = self.list_specs()?;
        let query_lower = query.to_lowercase();

        Ok(specs
            .into_iter()
            .find(|spec| spec.title.to_lowercase().contains(&query_lower)))
    }

    /// Delete a specification
    pub fn delete_spec(&self, spec_id: &str) -> Result<()> {
        let spec_dir = self.get_spec_dir(spec_id);

        if !spec_dir.exists() {
            return Err(VibeTicketError::custom(format!(
                "Specification not found: {spec_id}"
            )));
        }

        std::fs::remove_dir_all(&spec_dir)
            .map_err(|e| VibeTicketError::io_error("delete", &spec_dir, e))?;

        Ok(())
    }

    /// Set active specification
    pub fn set_active_spec(&self, spec_id: &str) -> Result<()> {
        // Verify spec exists
        self.load_metadata(spec_id)?;

        let active_file = self
            .ops
            .base_dir()
            .parent()
            .ok_or_else(|| VibeTicketError::custom("Invalid specs directory structure"))?
            .join(".active_spec");

        std::fs::write(&active_file, spec_id)
            .map_err(|e| VibeTicketError::io_error("write", &active_file, e))?;

        Ok(())
    }

    /// Get active specification ID
    pub fn get_active_spec(&self) -> Result<Option<String>> {
        let active_file = self
            .ops
            .base_dir()
            .parent()
            .ok_or_else(|| VibeTicketError::custom("Invalid specs directory structure"))?
            .join(".active_spec");

        if !active_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&active_file)
            .map_err(|e| VibeTicketError::io_error("read", &active_file, e))?;

        let spec_id = content.trim();
        if spec_id.is_empty() {
            Ok(None)
        } else {
            Ok(Some(spec_id.to_string()))
        }
    }

    // Compatibility methods

    /// Save a complete specification
    pub fn save(&self, spec: &Specification) -> Result<()> {
        // Save metadata
        self.save_metadata(&spec.metadata)?;

        // Save documents if present
        if let Some(ref requirements) = spec.requirements {
            self.save_document(
                &spec.metadata.id,
                SpecDocumentType::Requirements,
                requirements,
            )?;
        }
        if let Some(ref design) = spec.design {
            self.save_document(&spec.metadata.id, SpecDocumentType::Design, design)?;
        }
        if let Some(ref tasks) = spec.tasks {
            self.save_document(&spec.metadata.id, SpecDocumentType::Tasks, tasks)?;
        }

        Ok(())
    }

    /// Load a specification (alias for `load_spec`)
    pub fn load(&self, spec_id: &str) -> Result<Specification> {
        self.load_spec(spec_id)
    }

    /// List specifications (alias for `list_specs`)
    pub fn list(&self) -> Result<Vec<SpecMetadata>> {
        self.list_specs()
    }

    /// Delete a specification (alias for `delete_spec`)
    pub fn delete(&self, spec_id: &str) -> Result<()> {
        self.delete_spec(spec_id)
    }

    /// Get document path
    #[must_use]
    pub fn get_document_path(&self, spec_id: &str, doc_type: SpecDocumentType) -> PathBuf {
        self.get_spec_dir(spec_id).join(doc_type.file_name())
    }
}

// Standalone functions for compatibility

/// Save a specification document
pub fn save(
    specs_dir: &std::path::Path,
    spec_id: &str,
    doc_type: SpecDocumentType,
    content: &str,
) -> Result<()> {
    let manager = SpecManager::new(specs_dir.to_path_buf());
    manager.save_document(spec_id, doc_type, content)
}

/// Load a specification
pub fn load(specs_dir: &std::path::Path, spec_id: &str) -> Result<Specification> {
    let manager = SpecManager::new(specs_dir.to_path_buf());
    manager.load_spec(spec_id)
}

/// List all specifications
pub fn list(specs_dir: &std::path::Path) -> Result<Vec<SpecMetadata>> {
    let manager = SpecManager::new(specs_dir.to_path_buf());
    manager.list_specs()
}

/// Delete a specification
pub fn delete(specs_dir: &std::path::Path, spec_id: &str) -> Result<()> {
    let manager = SpecManager::new(specs_dir.to_path_buf());
    manager.delete_spec(spec_id)
}

/// Get document path for a specification
#[must_use]
pub fn get_document_path(
    specs_dir: &std::path::Path,
    spec_id: &str,
    doc_type: SpecDocumentType,
) -> PathBuf {
    specs_dir.join(spec_id).join(doc_type.file_name())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (SpecManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        let manager = SpecManager::new(specs_dir);
        (manager, temp_dir)
    }

    #[test]
    fn test_spec_manager_creation() {
        let (manager, _temp) = create_test_manager();
        assert!(manager.initialize().is_ok());
    }

    #[test]
    fn test_create_and_load_spec() {
        let (manager, _temp) = create_test_manager();

        let metadata = manager
            .create_spec("Test Spec".to_string(), "Test description".to_string())
            .unwrap();

        let loaded = manager.load_spec(&metadata.id).unwrap();
        assert_eq!(loaded.metadata.title, "Test Spec");
        assert_eq!(loaded.metadata.description, "Test description");
    }

    #[test]
    fn test_save_and_load_documents() {
        let (manager, _temp) = create_test_manager();

        let metadata = manager
            .create_spec("Test Spec".to_string(), "Test description".to_string())
            .unwrap();

        // Save documents
        manager
            .save_document(
                &metadata.id,
                SpecDocumentType::Requirements,
                "Test requirements",
            )
            .unwrap();

        manager
            .save_document(&metadata.id, SpecDocumentType::Design, "Test design")
            .unwrap();

        // Load spec and verify documents
        let spec = manager.load_spec(&metadata.id).unwrap();
        assert_eq!(spec.requirements, Some("Test requirements".to_string()));
        assert_eq!(spec.design, Some("Test design".to_string()));
        assert!(spec.metadata.progress.requirements_completed);
        assert!(spec.metadata.progress.design_completed);
    }
}
