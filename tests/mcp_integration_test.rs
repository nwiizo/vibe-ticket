//! Integration tests for MCP-CLI synchronization

#[cfg(feature = "mcp")]
mod mcp_tests {
    use serial_test::serial;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::time::sleep;
    use vibe_ticket::{
        cli::{OutputFormatter, handlers::handle_new_command},
        core::{Priority, Status},
        storage::FileStorage,
    };

    #[tokio::test]
    #[serial]
    async fn test_cli_to_mcp_notification() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let vibe_ticket_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&vibe_ticket_dir).unwrap();

        // Initialize storage
        let storage = FileStorage::new(&vibe_ticket_dir);

        // Initialize project state
        let state = vibe_ticket::storage::ProjectState {
            name: "Test Project".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            ticket_count: 0,
        };
        storage.save_state(&state).unwrap();
        storage.ensure_directories().unwrap();

        // Integration service initialization would go here
        // For now, skipping since integration module isn't exported

        // Mock receiver for testing
        let (_tx, mut receiver) = tokio::sync::broadcast::channel(100);

        // Create output formatter
        let output = OutputFormatter::new(false, false);

        // Create a ticket via CLI
        let result = handle_new_command(
            "test-integration",
            Some("Test Integration".to_string()),
            Some("Testing CLI-MCP integration".to_string()),
            "high",
            Some("integration,test".to_string()),
            false,
            Some(temp_dir.path().to_str().unwrap()),
            &output,
        );

        assert!(result.is_ok(), "Creating ticket should succeed");

        // Wait for the event to be processed
        sleep(Duration::from_millis(100)).await;

        // Check if we received the event
        use vibe_ticket::integration::IntegrationEvent;
        match receiver.try_recv() {
            Ok(IntegrationEvent::TicketCreated { ticket }) => {
                assert!(ticket.slug.ends_with("-test-integration"));
                assert_eq!(ticket.title, "Test Integration");
                assert_eq!(ticket.priority, Priority::High);
                assert_eq!(ticket.tags, vec!["integration", "test"]);
            },
            Ok(other) => panic!("Expected TicketCreated event, got {other:?}"),
            Err(e) => panic!("Failed to receive event: {e:?}"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_status_change_notification() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let vibe_ticket_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&vibe_ticket_dir).unwrap();

        // Initialize storage
        let storage = FileStorage::new(&vibe_ticket_dir);

        // Initialize project state
        let state = vibe_ticket::storage::ProjectState {
            name: "Test Project".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            ticket_count: 0,
        };
        storage.save_state(&state).unwrap();
        storage.ensure_directories().unwrap();

        // Integration service initialization would go here
        // For now, skipping since integration module isn't exported

        // Mock receiver for testing
        let (_tx, mut receiver) = tokio::sync::broadcast::channel(100);

        // Create output formatter
        let output = OutputFormatter::new(false, false);

        // Create a ticket with --start flag to trigger status change
        let result = handle_new_command(
            "test-status",
            Some("Test Status Change".to_string()),
            None,
            "medium",
            None,
            true, // Start immediately
            Some(temp_dir.path().to_str().unwrap()),
            &output,
        );

        assert!(
            result.is_ok(),
            "Creating and starting ticket should succeed"
        );

        // Wait for events to be processed
        sleep(Duration::from_millis(100)).await;

        // We should receive two events: TicketCreated and StatusChanged
        use vibe_ticket::integration::IntegrationEvent;

        // First event should be TicketCreated
        match receiver.try_recv() {
            Ok(IntegrationEvent::TicketCreated { ticket }) => {
                assert!(ticket.slug.ends_with("-test-status"));
            },
            Ok(other) => panic!("Expected TicketCreated event first, got {other:?}"),
            Err(e) => panic!("Failed to receive first event: {e:?}"),
        }

        // Second event should be StatusChanged
        match receiver.try_recv() {
            Ok(IntegrationEvent::StatusChanged {
                ticket_id: _,
                old_status,
                new_status,
            }) => {
                assert_eq!(old_status, Status::Todo);
                assert_eq!(new_status, Status::Doing);
            },
            Ok(other) => panic!("Expected StatusChanged event second, got {other:?}"),
            Err(e) => panic!("Failed to receive second event: {e:?}"),
        }
    }
}
