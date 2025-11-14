use crate::core::{Ticket, TicketId};
use crate::error::Result;

/// Repository trait for ticket storage operations
///
/// This trait defines the interface for storing and retrieving tickets,
/// allowing for different storage implementations.
pub trait TicketRepository: Send + Sync {
    /// Saves a ticket to the repository
    fn save(&self, ticket: &Ticket) -> Result<()>;

    /// Loads a ticket by ID
    fn load(&self, id: &TicketId) -> Result<Ticket>;

    /// Loads all tickets
    fn load_all(&self) -> Result<Vec<Ticket>>;

    /// Deletes a ticket by ID
    fn delete(&self, id: &TicketId) -> Result<()>;

    /// Checks if a ticket exists by ID
    fn exists(&self, id: &TicketId) -> Result<bool>;

    /// Finds tickets matching a predicate
    fn find<F>(&self, predicate: F) -> Result<Vec<Ticket>>
    where
        F: Fn(&Ticket) -> bool;

    /// Counts tickets matching a predicate
    fn count<F>(&self, predicate: F) -> Result<usize>
    where
        F: Fn(&Ticket) -> bool;
}

/// Repository trait for managing active tickets
///
/// Supports both single active ticket (legacy) and multiple active tickets
pub trait ActiveTicketRepository: Send + Sync {
    /// Sets the active ticket ID (clears other active tickets)
    fn set_active(&self, id: &TicketId) -> Result<()>;

    /// Gets the active ticket ID (returns first active ticket for compatibility)
    fn get_active(&self) -> Result<Option<TicketId>>;

    /// Clears all active tickets
    fn clear_active(&self) -> Result<()>;

    /// Adds a ticket to the list of active tickets
    fn add_active(&self, id: &TicketId) -> Result<()>;

    /// Removes a ticket from the list of active tickets
    fn remove_active(&self, id: &TicketId) -> Result<()>;

    /// Gets all active ticket IDs
    fn get_all_active(&self) -> Result<Vec<TicketId>>;
}

/// Combined repository trait
pub trait Repository: TicketRepository + ActiveTicketRepository {}

/// Implementation of Repository for types that implement both traits
impl<T> Repository for T where T: TicketRepository + ActiveTicketRepository {}

use super::file::FileStorage;

impl TicketRepository for FileStorage {
    fn save(&self, ticket: &Ticket) -> Result<()> {
        self.save_ticket(ticket)
    }

    fn load(&self, id: &TicketId) -> Result<Ticket> {
        self.load_ticket(id)
    }

    fn load_all(&self) -> Result<Vec<Ticket>> {
        self.load_all_tickets()
    }

    fn delete(&self, id: &TicketId) -> Result<()> {
        self.delete_ticket(id)
    }

    fn exists(&self, id: &TicketId) -> Result<bool> {
        match self.load_ticket(id) {
            Ok(_) => Ok(true),
            Err(crate::error::VibeTicketError::TicketNotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn find<F>(&self, predicate: F) -> Result<Vec<Ticket>>
    where
        F: Fn(&Ticket) -> bool,
    {
        let tickets = self.load_all_tickets()?;
        Ok(tickets.into_iter().filter(predicate).collect())
    }

    fn count<F>(&self, predicate: F) -> Result<usize>
    where
        F: Fn(&Ticket) -> bool,
    {
        let tickets = self.load_all_tickets()?;
        Ok(tickets.iter().filter(|t| predicate(t)).count())
    }
}

impl ActiveTicketRepository for FileStorage {
    fn set_active(&self, id: &TicketId) -> Result<()> {
        // Clear all active tickets and set this one
        self.clear_active()?;
        self.add_active_ticket(id)
    }

    fn get_active(&self) -> Result<Option<TicketId>> {
        // For backward compatibility, return the first active ticket
        let active_tickets = self.get_all_active_tickets()?;
        Ok(active_tickets.first().cloned())
    }

    fn clear_active(&self) -> Result<()> {
        // Clear legacy format
        self.clear_active_ticket()?;

        // Clear new format by removing all active tickets
        let all_active = self.get_all_active_tickets()?;
        for id in all_active {
            self.remove_active_ticket(&id)?;
        }

        Ok(())
    }

    fn add_active(&self, id: &TicketId) -> Result<()> {
        self.add_active_ticket(id)
    }

    fn remove_active(&self, id: &TicketId) -> Result<()> {
        self.remove_active_ticket(id)
    }

    fn get_all_active(&self) -> Result<Vec<TicketId>> {
        self.get_all_active_tickets()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Priority, Status, Ticket};
    use crate::storage::FileStorage;
    use tempfile::TempDir;

    fn create_test_ticket(slug: &str) -> Ticket {
        Ticket::new(slug.to_string(), format!("Test ticket {slug}"))
    }

    #[test]
    fn test_ticket_repository_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);

        let ticket = create_test_ticket("test-save");
        let id = ticket.id.clone();

        // Save ticket
        storage.save(&ticket).expect("Failed to save ticket");

        // Load ticket
        let loaded = storage.load(&id).expect("Failed to load ticket");
        assert_eq!(loaded.id, ticket.id);
        assert_eq!(loaded.title, ticket.title);
    }

    #[test]
    fn test_ticket_repository_load_all() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);

        // Save multiple tickets
        let tickets: Vec<_> = (0..3)
            .map(|i| create_test_ticket(&format!("test-{i}")))
            .collect();

        for ticket in &tickets {
            storage.save(ticket).expect("Failed to save ticket");
        }

        // Load all tickets
        let loaded = storage.load_all().expect("Failed to load all tickets");
        assert_eq!(loaded.len(), 3);
    }

    #[test]
    fn test_ticket_repository_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);

        let ticket = create_test_ticket("test-delete");
        let id = ticket.id.clone();

        // Save and delete
        storage.save(&ticket).expect("Failed to save ticket");
        assert!(storage.exists(&id).expect("Failed to check existence"));

        storage.delete(&id).expect("Failed to delete ticket");
        assert!(!storage.exists(&id).expect("Failed to check existence"));
    }

    #[test]
    fn test_ticket_repository_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);

        let ticket = create_test_ticket("test-exists");
        let id = ticket.id.clone();
        let non_existent_id = TicketId::new();

        // Check non-existent
        assert!(
            !storage
                .exists(&non_existent_id)
                .expect("Failed to check existence")
        );

        // Save and check exists
        storage.save(&ticket).expect("Failed to save ticket");
        assert!(storage.exists(&id).expect("Failed to check existence"));
    }

    #[test]
    fn test_ticket_repository_find() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);

        // Create tickets with different priorities
        let mut high_priority = create_test_ticket("high-priority");
        high_priority.priority = Priority::High;

        let mut low_priority = create_test_ticket("low-priority");
        low_priority.priority = Priority::Low;

        storage.save(&high_priority).expect("Failed to save ticket");
        storage.save(&low_priority).expect("Failed to save ticket");

        // Find high priority tickets
        let found = storage
            .find(|t| t.priority == Priority::High)
            .expect("Failed to find tickets");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].slug, "high-priority");
    }

    #[test]
    fn test_ticket_repository_count() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);

        // Create tickets with different statuses
        let mut todo = create_test_ticket("todo");
        todo.status = Status::Todo;

        let mut doing = create_test_ticket("doing");
        doing.status = Status::Doing;

        let mut done = create_test_ticket("done");
        done.status = Status::Done;

        storage.save(&todo).expect("Failed to save ticket");
        storage.save(&doing).expect("Failed to save ticket");
        storage.save(&done).expect("Failed to save ticket");

        // Count open tickets
        let open_count = storage
            .count(|t| matches!(t.status, Status::Todo | Status::Doing))
            .expect("Failed to count tickets");
        assert_eq!(open_count, 2);
    }

    #[test]
    fn test_active_ticket_repository() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);

        let ticket_id = TicketId::new();

        // Initially no active ticket
        assert!(
            storage
                .get_active()
                .expect("Failed to get active")
                .is_none()
        );

        // Set active ticket
        storage
            .set_active(&ticket_id)
            .expect("Failed to set active");
        let active = storage.get_active().expect("Failed to get active");
        assert_eq!(active, Some(ticket_id.clone()));

        // Clear active ticket
        storage.clear_active().expect("Failed to clear active");
        assert!(
            storage
                .get_active()
                .expect("Failed to get active")
                .is_none()
        );
    }

    #[test]
    fn test_combined_repository() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);

        // Test both ticket and active ticket operations
        let ticket = create_test_ticket("combined-test");
        let id = ticket.id.clone();

        // Ticket operations
        storage.save(&ticket).expect("Failed to save ticket");
        assert!(storage.exists(&id).expect("Failed to check existence"));

        // Active ticket operations
        storage.set_active(&id).expect("Failed to set active");
        assert_eq!(
            storage.get_active().expect("Failed to get active"),
            Some(id)
        );
    }
}
