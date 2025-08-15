/// Tests for concurrent access to storage
#[cfg(test)]
mod tests {
    use crate::core::{Priority, Status, Ticket};
    use crate::storage::FileStorage;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_ticket(title: &str) -> Ticket {
        Ticket {
            id: Uuid::new_v4().into(),
            slug: format!("test-{}", title.to_lowercase().replace(' ', "-")),
            title: title.to_string(),
            description: format!("Description for {title}"),
            priority: Priority::Medium,
            status: Status::Todo,
            tags: vec!["test".to_string()],
            created_at: chrono::Utc::now(),
            started_at: None,
            closed_at: None,
            assignee: None,
            tasks: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_concurrent_ticket_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FileStorage::new(temp_dir.path()));
        let barrier = Arc::new(Barrier::new(10));

        let mut handles = vec![];

        // Spawn 10 threads that try to create tickets simultaneously
        for i in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Try to create a ticket
                let ticket = create_test_ticket(&format!("Concurrent Ticket {i}"));
                storage_clone.save_ticket(&ticket).unwrap();
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all tickets were created
        let tickets = storage.load_all_tickets().unwrap();
        assert_eq!(tickets.len(), 10);
    }

    #[test]
    fn test_concurrent_ticket_modification() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FileStorage::new(temp_dir.path()));

        // Create initial ticket
        let ticket = create_test_ticket("Concurrent Modification Test");
        let ticket_id = ticket.id.clone();
        storage.save_ticket(&ticket).unwrap();

        let barrier = Arc::new(Barrier::new(5));
        let mut handles = vec![];

        // Spawn 5 threads that try to modify the same ticket
        for i in 0..5 {
            let storage_clone = Arc::clone(&storage);
            let barrier_clone = Arc::clone(&barrier);
            let id = ticket_id.clone();

            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Load, modify, and save the ticket
                let mut ticket = storage_clone.load_ticket(&id).unwrap();
                ticket.description = format!("Modified by thread {i}");
                ticket.tags.push(format!("thread-{i}"));
                storage_clone.save_ticket(&ticket).unwrap();
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify the ticket was modified
        let final_ticket = storage.load_ticket(&ticket_id).unwrap();
        assert!(final_ticket.description.starts_with("Modified by thread"));
        // Should have at least one thread tag
        assert!(
            final_ticket
                .tags
                .iter()
                .any(|tag| tag.starts_with("thread-"))
        );
    }

    #[test]
    fn test_concurrent_active_ticket_changes() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FileStorage::new(temp_dir.path()));

        // Create multiple tickets
        let mut ticket_ids = vec![];
        for i in 0..5 {
            let ticket = create_test_ticket(&format!("Active Test {i}"));
            let id = ticket.id.clone();
            storage.save_ticket(&ticket).unwrap();
            ticket_ids.push(id);
        }

        let barrier = Arc::new(Barrier::new(5));
        let mut handles = vec![];

        // Spawn threads that try to set different active tickets
        for (i, id) in ticket_ids.iter().enumerate() {
            let storage_clone = Arc::clone(&storage);
            let barrier_clone = Arc::clone(&barrier);
            let ticket_id = id.clone();

            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Try to set this ticket as active
                storage_clone.set_active_ticket(&ticket_id).unwrap();

                // Small delay to create more contention
                thread::sleep(std::time::Duration::from_millis(10));

                // Try to clear if it's an even thread
                if i % 2 == 0 {
                    let _ = storage_clone.clear_active_ticket();
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // The active ticket should be one of the tickets (or none)
        let active = storage.get_active_ticket().ok().flatten();
        if let Some(active_id) = active {
            assert!(ticket_ids.contains(&active_id));
        }
    }

    #[test]
    fn test_lock_timeout_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // Create a ticket
        let ticket = create_test_ticket("Lock Timeout Test");
        let ticket_id = ticket.id.clone();
        let ticket_path = storage.ticket_path(&ticket_id);

        // Ensure the ticket directory exists
        storage.ensure_directories().unwrap();

        // Manually create a stale lock without saving the ticket first
        let lock_path = ticket_path.with_extension("yaml.lock");
        let stale_info = crate::storage::lock::LockInfo {
            holder_id: "stale-process".to_string(),
            pid: 99999,
            acquired_at: 0, // Very old timestamp
            operation: Some("stale operation".to_string()),
        };
        std::fs::write(&lock_path, serde_json::to_string(&stale_info).unwrap()).unwrap();

        // Should still be able to save the ticket (stale lock should be removed)
        storage.save_ticket(&ticket).unwrap();

        // Verify the ticket was saved
        let loaded = storage.load_ticket(&ticket_id).unwrap();
        assert_eq!(loaded.title, "Lock Timeout Test");
    }
}
