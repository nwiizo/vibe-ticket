use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::cache::TicketCache;
use crate::core::{Ticket, TicketId};

use crate::error::{ErrorContext, Result, VibeTicketError};

/// File-based storage implementation for tickets
///
/// This implementation stores tickets as YAML files in a directory structure
/// within the project's .vibe-ticket directory.
#[derive(Clone)]
pub struct FileStorage {
    /// Base directory for storing ticket data
    base_dir: PathBuf,
    /// Cache for improved performance
    pub(crate) cache: Arc<TicketCache>,
}

impl FileStorage {
    /// Creates a new `FileStorage` instance
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            cache: Arc::new(TicketCache::with_default_ttl()),
        }
    }

    /// Returns the path to the tickets directory
    fn tickets_dir(&self) -> PathBuf {
        self.get_path("tickets")
    }

    /// Returns the path to a specific ticket file
    pub(crate) fn ticket_path(&self, id: &TicketId) -> PathBuf {
        self.tickets_dir().join(format!("{id}.yaml"))
    }

    /// Returns the path to the active ticket file
    fn active_ticket_path(&self) -> PathBuf {
        self.get_path("active_ticket")
    }

    /// Returns the path to the project state file
    fn state_path(&self) -> PathBuf {
        self.get_path("state.yaml")
    }

    /// Helper method to get a path relative to base directory
    fn get_path(&self, name: &str) -> PathBuf {
        self.base_dir.join(name)
    }

    /// Ensures the storage directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(self.tickets_dir()).context("Failed to create tickets directory")?;
        Ok(())
    }

    /// Saves a ticket to storage with file locking for concurrent access protection
    pub fn save_ticket(&self, ticket: &Ticket) -> Result<()> {
        self.ensure_directories()?;

        let path = self.ticket_path(&ticket.id);

        // Acquire lock before modifying the file
        let _lock =
            super::FileLock::acquire(&path, Some("save_ticket".to_string())).map_err(|e| {
                VibeTicketError::custom(format!("Failed to acquire lock for saving ticket: {e}"))
            })?;

        let yaml = serde_yaml::to_string(ticket).context("Failed to serialize ticket")?;

        fs::write(&path, yaml)
            .with_context(|| format!("Failed to write ticket to {}", path.display()))?;

        // Invalidate cache for this ticket
        self.cache.invalidate_ticket(&ticket.id);

        Ok(())
    }

    /// Loads a ticket from storage by ID with read locking
    pub fn load_ticket(&self, id: &TicketId) -> Result<Ticket> {
        // Check cache first
        if let Some(ticket) = self.cache.get_ticket(id) {
            return Ok(ticket);
        }

        let path = self.ticket_path(id);

        if !path.exists() {
            return Err(VibeTicketError::TicketNotFound { id: id.to_string() });
        }

        // Acquire lock for reading to ensure consistency
        let _lock =
            super::FileLock::acquire(&path, Some("load_ticket".to_string())).map_err(|e| {
                VibeTicketError::custom(format!("Failed to acquire lock for loading ticket: {e}"))
            })?;

        let yaml = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read ticket from {}", path.display()))?;

        let ticket: Ticket = serde_yaml::from_str(&yaml).context("Failed to deserialize ticket")?;

        // Cache the loaded ticket
        self.cache.cache_ticket(&ticket);

        Ok(ticket)
    }

    /// Loads all tickets from storage
    pub fn load_all_tickets(&self) -> Result<Vec<Ticket>> {
        // Check cache first
        if let Some(tickets) = self.cache.get_all_tickets() {
            return Ok(tickets);
        }

        let tickets_dir = self.tickets_dir();

        if !tickets_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tickets = Vec::new();

        for entry in fs::read_dir(&tickets_dir).context("Failed to read tickets directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let yaml = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;

                match serde_yaml::from_str::<Ticket>(&yaml) {
                    Ok(ticket) => tickets.push(ticket),
                    Err(e) => {
                        // Log error but continue loading other tickets
                        eprintln!(
                            "Warning: Failed to load ticket from {}: {e}",
                            path.display()
                        );
                    },
                }
            }
        }

        // Cache all loaded tickets
        self.cache.cache_all_tickets(&tickets);

        Ok(tickets)
    }

    /// Deletes a ticket from storage with locking
    pub fn delete_ticket(&self, id: &TicketId) -> Result<()> {
        let path = self.ticket_path(id);

        if !path.exists() {
            return Err(VibeTicketError::TicketNotFound { id: id.to_string() });
        }

        // Acquire lock before deleting
        let _lock =
            super::FileLock::acquire(&path, Some("delete_ticket".to_string())).map_err(|e| {
                VibeTicketError::custom(format!("Failed to acquire lock for deleting ticket: {e}"))
            })?;

        fs::remove_file(&path)
            .with_context(|| format!("Failed to delete ticket at {}", path.display()))?;

        // Invalidate cache for this ticket
        self.cache.invalidate_ticket(id);

        Ok(())
    }

    /// Sets the active ticket with locking
    pub fn set_active_ticket(&self, id: &TicketId) -> Result<()> {
        let path = self.active_ticket_path();

        // Acquire lock for the active ticket file
        let _lock = super::FileLock::acquire(&path, Some("set_active_ticket".to_string()))
            .map_err(|e| {
                VibeTicketError::custom(format!(
                    "Failed to acquire lock for setting active ticket: {e}"
                ))
            })?;

        fs::write(&path, id.to_string()).context("Failed to write active ticket")?;
        Ok(())
    }

    /// Gets the active ticket ID
    pub fn get_active_ticket(&self) -> Result<Option<TicketId>> {
        let path = self.active_ticket_path();

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).context("Failed to read active ticket")?;

        let id = TicketId::parse_str(content.trim()).context("Failed to parse active ticket ID")?;

        Ok(Some(id))
    }

    /// Clears the active ticket with locking
    pub fn clear_active_ticket(&self) -> Result<()> {
        let path = self.active_ticket_path();

        if path.exists() {
            // Acquire lock before removing
            let _lock = super::FileLock::acquire(&path, Some("clear_active_ticket".to_string()))
                .map_err(|e| {
                    VibeTicketError::custom(format!(
                        "Failed to acquire lock for clearing active ticket: {e}"
                    ))
                })?;

            fs::remove_file(&path).context("Failed to clear active ticket")?;
        }

        Ok(())
    }

    /// Checks if a ticket with the given slug already exists
    pub fn ticket_exists_with_slug(&self, slug: &str) -> Result<bool> {
        let tickets = self.load_all_tickets()?;
        Ok(tickets.iter().any(|t| t.slug == slug))
    }

    /// Finds a ticket by its slug
    pub fn find_ticket_by_slug(&self, slug: &str) -> Result<Option<Ticket>> {
        let tickets = self.load_all_tickets()?;
        Ok(tickets.into_iter().find(|t| t.slug == slug))
    }
}

/// Project state stored in the .vibe-ticket directory
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectState {
    /// Project name
    pub name: String,

    /// Project description
    pub description: Option<String>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last modified timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Total number of tickets created (for ID generation)
    pub ticket_count: u64,
}

impl FileStorage {
    /// Saves the project state
    pub fn save_state(&self, state: &ProjectState) -> Result<()> {
        let path = self.state_path();
        let yaml = serde_yaml::to_string(state).context("Failed to serialize project state")?;

        fs::write(&path, yaml).context("Failed to write project state")?;

        Ok(())
    }

    /// Loads the project state
    pub fn load_state(&self) -> Result<ProjectState> {
        let path = self.state_path();

        if !path.exists() {
            return Err(VibeTicketError::ProjectNotInitialized);
        }

        let yaml = fs::read_to_string(&path).context("Failed to read project state")?;

        let state: ProjectState =
            serde_yaml::from_str(&yaml).context("Failed to deserialize project state")?;

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (FileStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        (storage, temp_dir)
    }

    #[test]
    fn test_save_and_load_ticket() {
        let (storage, _temp) = create_test_storage();
        let mut ticket = Ticket::new("test-ticket", "Test Ticket");
        ticket.description = "Test description".to_string();

        storage.save_ticket(&ticket).unwrap();
        let loaded = storage.load_ticket(&ticket.id).unwrap();

        assert_eq!(loaded.slug, ticket.slug);
        assert_eq!(loaded.title, ticket.title);
        assert_eq!(loaded.description, ticket.description);
    }

    #[test]
    fn test_load_all_tickets() {
        let (storage, _temp) = create_test_storage();

        let first_ticket = Ticket::new("ticket-1", "Ticket 1");
        let second_ticket = Ticket::new("ticket-2", "Ticket 2");

        storage.save_ticket(&first_ticket).unwrap();
        storage.save_ticket(&second_ticket).unwrap();

        let all_tickets = storage.load_all_tickets().unwrap();
        assert_eq!(all_tickets.len(), 2);
    }

    #[test]
    fn test_active_ticket() {
        let (storage, _temp) = create_test_storage();
        let ticket = Ticket::new("test", "Test");

        storage.save_ticket(&ticket).unwrap();
        storage.set_active_ticket(&ticket.id).unwrap();

        let active_id = storage.get_active_ticket().unwrap();
        assert_eq!(active_id, Some(ticket.id));

        storage.clear_active_ticket().unwrap();
        let active_id = storage.get_active_ticket().unwrap();
        assert_eq!(active_id, None);
    }
}
// Include concurrent tests
#[cfg(test)]
#[path = "concurrent_tests.rs"]
mod concurrent_tests;
