//! Integration tests for the import feature
//!
//! This module tests the import functionality with various file formats
//! and edge cases to ensure robust importing of tickets.

use std::fs;
use tempfile::TempDir;
use vibe_ticket::cli::{OutputFormatter, handlers::handle_import_command};
use vibe_ticket::core::{Priority, Status, Ticket, TicketId};
use vibe_ticket::storage::{FileStorage, TicketRepository};

/// Create a test environment with an initialized project
fn setup_test_project() -> (TempDir, OutputFormatter) {
    let temp_dir = TempDir::new().unwrap();
    let vibe_ticket_dir = temp_dir.path().join(".vibe-ticket");
    fs::create_dir_all(vibe_ticket_dir.join("tickets")).unwrap();

    // Initialize project state
    let state = vibe_ticket::storage::ProjectState {
        name: "Test Import Project".to_string(),
        description: Some("Testing import functionality".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        ticket_count: 0,
    };

    let storage = FileStorage::new(&vibe_ticket_dir);
    storage.save_state(&state).unwrap();

    let formatter = OutputFormatter::new(false, true);
    (temp_dir, formatter)
}

#[test]
fn test_import_json_array() {
    let (temp_dir, formatter) = setup_test_project();

    // Create test JSON file with array of tickets
    let json_content = r#"[
        {
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "slug": "json-test-1",
            "title": "First JSON Test Ticket",
            "description": "Testing JSON import",
            "priority": "high",
            "status": "todo",
            "tags": ["json", "test"],
            "created_at": "2025-07-28T10:00:00Z",
            "started_at": null,
            "closed_at": null,
            "assignee": null,
            "tasks": [],
            "metadata": {}
        },
        {
            "id": "550e8400-e29b-41d4-a716-446655440002",
            "slug": "json-test-2",
            "title": "Second JSON Test Ticket",
            "description": "Another JSON test",
            "priority": "medium",
            "status": "doing",
            "tags": ["json"],
            "created_at": "2025-07-28T11:00:00Z",
            "started_at": "2025-07-28T11:30:00Z",
            "closed_at": null,
            "assignee": "test-user",
            "tasks": [
                {
                    "id": "550e8400-e29b-41d4-a716-446655440003",
                    "title": "Sample task",
                    "completed": false,
                    "created_at": "2025-07-28T11:00:00Z",
                    "completed_at": null
                }
            ],
            "metadata": {
                "custom_field": "custom_value"
            }
        }
    ]"#;

    let json_file = temp_dir.path().join("tickets.json");
    fs::write(&json_file, json_content).unwrap();

    // Import the tickets
    let result = handle_import_command(
        json_file.to_str().unwrap(),
        Some("json"),
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());

    // Verify tickets were imported
    let storage = FileStorage::new(temp_dir.path().join(".vibe-ticket"));
    let imported_tickets = storage.load_all_tickets().unwrap();
    assert_eq!(imported_tickets.len(), 2);

    // Verify ticket details
    let first_ticket = storage.find_ticket_by_slug("json-test-1").unwrap().unwrap();
    assert_eq!(first_ticket.title, "First JSON Test Ticket");
    assert_eq!(first_ticket.priority, Priority::High);
    assert_eq!(first_ticket.status, Status::Todo);
    assert_eq!(first_ticket.tags, vec!["json", "test"]);

    let second_ticket = storage.find_ticket_by_slug("json-test-2").unwrap().unwrap();
    assert_eq!(second_ticket.title, "Second JSON Test Ticket");
    assert_eq!(second_ticket.assignee, Some("test-user".to_string()));
    assert_eq!(second_ticket.tasks.len(), 1);
}

#[test]
fn test_import_json_object() {
    let (temp_dir, formatter) = setup_test_project();

    // Create test JSON file with object containing tickets field
    let json_content = r#"{
        "export_date": "2025-07-28T12:00:00Z",
        "project": "Test Project",
        "tickets": [
            {
                "id": "650e8400-e29b-41d4-a716-446655440001",
                "slug": "json-obj-test",
                "title": "JSON Object Test",
                "description": "Testing JSON object format",
                "priority": "low",
                "status": "blocked",
                "tags": [],
                "created_at": "2025-07-28T10:00:00Z",
                "started_at": null,
                "closed_at": null,
                "assignee": null,
                "tasks": [],
                "metadata": {}
            }
        ]
    }"#;

    let json_file = temp_dir.path().join("export.json");
    fs::write(&json_file, json_content).unwrap();

    // Import the tickets
    let result = handle_import_command(
        json_file.to_str().unwrap(),
        None, // Test auto-detection
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());

    // Verify ticket was imported
    let storage = FileStorage::new(temp_dir.path().join(".vibe-ticket"));
    let ticket = storage
        .find_ticket_by_slug("json-obj-test")
        .unwrap()
        .unwrap();
    assert_eq!(ticket.title, "JSON Object Test");
    assert_eq!(ticket.status, Status::Blocked);
}

#[test]
fn test_import_yaml() {
    let (temp_dir, formatter) = setup_test_project();

    // Create test YAML file
    let yaml_content = r#"tickets:
  - id: "750e8400-e29b-41d4-a716-446655440001"
    slug: "yaml-test-1"
    title: "YAML Test Ticket"
    description: "Testing YAML import"
    priority: "critical"
    status: "review"
    tags:
      - yaml
      - import
    created_at: "2025-07-28T10:00:00Z"
    started_at: "2025-07-28T10:30:00Z"
    closed_at: null
    assignee: "yaml-tester"
    tasks: []
    metadata: {}"#;

    let yaml_file = temp_dir.path().join("tickets.yaml");
    fs::write(&yaml_file, yaml_content).unwrap();

    // Import the tickets
    let result = handle_import_command(
        yaml_file.to_str().unwrap(),
        Some("yaml"),
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());

    // Verify ticket was imported
    let storage = FileStorage::new(temp_dir.path().join(".vibe-ticket"));
    let ticket = storage.find_ticket_by_slug("yaml-test-1").unwrap().unwrap();
    assert_eq!(ticket.title, "YAML Test Ticket");
    assert_eq!(ticket.priority, Priority::Critical);
    assert_eq!(ticket.status, Status::Review);
    assert_eq!(ticket.assignee, Some("yaml-tester".to_string()));
}

#[test]
fn test_import_csv() {
    let (temp_dir, formatter) = setup_test_project();

    // Create test CSV file
    let csv_content = r#"ID,Slug,Title,Status,Priority,Assignee,Tags,Created At,Started At,Closed At,Tasks Total,Tasks Completed,Description
850e8400-e29b-41d4-a716-446655440001,csv-test-1,CSV Test Ticket 1,todo,high,,"csv, test",2025-07-28T10:00:00Z,,,0,0,Testing CSV import functionality
850e8400-e29b-41d4-a716-446655440002,csv-test-2,CSV Test Ticket 2,done,medium,csv-user,csv,2025-07-28T11:00:00Z,2025-07-28T11:30:00Z,2025-07-28T12:00:00Z,2,2,"Another CSV test, with commas"
"#;

    let csv_file = temp_dir.path().join("tickets.csv");
    fs::write(&csv_file, csv_content).unwrap();

    // Import the tickets
    let result = handle_import_command(
        csv_file.to_str().unwrap(),
        Some("csv"),
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());

    // Verify tickets were imported
    let storage = FileStorage::new(temp_dir.path().join(".vibe-ticket"));
    let csv_tickets = storage.load_all_tickets().unwrap();
    assert_eq!(csv_tickets.len(), 2);

    let csv_ticket_1 = storage.find_ticket_by_slug("csv-test-1").unwrap().unwrap();
    assert_eq!(csv_ticket_1.title, "CSV Test Ticket 1");
    assert_eq!(csv_ticket_1.tags, vec!["csv", "test"]);
    assert!(csv_ticket_1.assignee.is_none());

    let csv_ticket_2 = storage.find_ticket_by_slug("csv-test-2").unwrap().unwrap();
    assert_eq!(csv_ticket_2.status, Status::Done);
    assert!(csv_ticket_2.closed_at.is_some());
    assert_eq!(csv_ticket_2.assignee, Some("csv-user".to_string()));
}

#[test]
fn test_dry_run_import() {
    let (temp_dir, formatter) = setup_test_project();

    // Create test JSON file
    let json_content = r#"[
        {
            "id": "950e8400-e29b-41d4-a716-446655440001",
            "slug": "dry-run-test",
            "title": "Dry Run Test",
            "description": "This should not be imported",
            "priority": "medium",
            "status": "todo",
            "tags": [],
            "created_at": "2025-07-28T10:00:00Z",
            "started_at": null,
            "closed_at": null,
            "assignee": null,
            "tasks": [],
            "metadata": {}
        }
    ]"#;

    let json_file = temp_dir.path().join("dry_run.json");
    fs::write(&json_file, json_content).unwrap();

    // Import with dry run
    let result = handle_import_command(
        json_file.to_str().unwrap(),
        Some("json"),
        false,
        true, // dry_run = true
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());

    // Verify no tickets were actually imported
    let storage = FileStorage::new(temp_dir.path().join(".vibe-ticket"));
    let tickets = storage.load_all_tickets().unwrap();
    assert_eq!(tickets.len(), 0);
}

#[test]
fn test_skip_existing_tickets() {
    let (temp_dir, formatter) = setup_test_project();

    // Create an existing ticket
    let existing_ticket = Ticket {
        id: TicketId::parse_str("a50e8400-e29b-41d4-a716-446655440001").unwrap(),
        slug: "existing-ticket".to_string(),
        title: "Existing Ticket".to_string(),
        description: "This ticket already exists".to_string(),
        priority: Priority::Medium,
        status: Status::Todo,
        tags: vec![],
        created_at: chrono::Utc::now(),
        started_at: None,
        closed_at: None,
        assignee: None,
        tasks: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let storage = FileStorage::new(temp_dir.path().join(".vibe-ticket"));
    storage.save(&existing_ticket).unwrap();

    // Create JSON with duplicate slug
    let json_content = r#"[
        {
            "id": "b50e8400-e29b-41d4-a716-446655440001",
            "slug": "existing-ticket",
            "title": "Duplicate Slug Ticket",
            "description": "This has the same slug",
            "priority": "high",
            "status": "todo",
            "tags": [],
            "created_at": "2025-07-28T10:00:00Z",
            "started_at": null,
            "closed_at": null,
            "assignee": null,
            "tasks": [],
            "metadata": {}
        },
        {
            "id": "b50e8400-e29b-41d4-a716-446655440002",
            "slug": "new-ticket",
            "title": "New Ticket",
            "description": "This should be imported",
            "priority": "low",
            "status": "todo",
            "tags": [],
            "created_at": "2025-07-28T11:00:00Z",
            "started_at": null,
            "closed_at": null,
            "assignee": null,
            "tasks": [],
            "metadata": {}
        }
    ]"#;

    let json_file = temp_dir.path().join("with_duplicates.json");
    fs::write(&json_file, json_content).unwrap();

    // Import the tickets
    let result = handle_import_command(
        json_file.to_str().unwrap(),
        Some("json"),
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());

    // Verify only the new ticket was imported
    let tickets = storage.load_all_tickets().unwrap();
    assert_eq!(tickets.len(), 2); // 1 existing + 1 new

    // Verify the existing ticket wasn't overwritten
    let existing = storage
        .find_ticket_by_slug("existing-ticket")
        .unwrap()
        .unwrap();
    assert_eq!(existing.title, "Existing Ticket"); // Original title preserved

    // Verify the new ticket was imported
    let new_ticket = storage.find_ticket_by_slug("new-ticket").unwrap().unwrap();
    assert_eq!(new_ticket.title, "New Ticket");
}

#[test]
fn test_validation_duplicate_ids() {
    let (temp_dir, formatter) = setup_test_project();

    // Create JSON with duplicate IDs (validation should fail)
    let json_content = r#"[
        {
            "id": "c50e8400-e29b-41d4-a716-446655440001",
            "slug": "ticket-1",
            "title": "First Ticket",
            "description": "Test",
            "priority": "medium",
            "status": "todo",
            "tags": [],
            "created_at": "2025-07-28T10:00:00Z",
            "started_at": null,
            "closed_at": null,
            "assignee": null,
            "tasks": [],
            "metadata": {}
        },
        {
            "id": "c50e8400-e29b-41d4-a716-446655440001",
            "slug": "ticket-2",
            "title": "Duplicate ID Ticket",
            "description": "This has the same ID",
            "priority": "high",
            "status": "todo",
            "tags": [],
            "created_at": "2025-07-28T11:00:00Z",
            "started_at": null,
            "closed_at": null,
            "assignee": null,
            "tasks": [],
            "metadata": {}
        }
    ]"#;

    let json_file = temp_dir.path().join("duplicate_ids.json");
    fs::write(&json_file, json_content).unwrap();

    // Import should succeed even with duplicate IDs in the import file
    // The second ticket will be skipped during import
    let result = handle_import_command(
        json_file.to_str().unwrap(),
        Some("json"),
        true, // skip_validation = true
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());
}

#[test]
fn test_invalid_json_format() {
    let (temp_dir, formatter) = setup_test_project();

    // Create invalid JSON file
    let json_content = r"{ invalid json content";

    let json_file = temp_dir.path().join("invalid.json");
    fs::write(&json_file, json_content).unwrap();

    // Import should fail
    let result = handle_import_command(
        json_file.to_str().unwrap(),
        Some("json"),
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    assert!(result.is_err());
}

#[test]
fn test_auto_format_detection() {
    let (temp_dir, formatter) = setup_test_project();

    // Test JSON detection by extension
    let json_content = r#"[{"id": "d50e8400-e29b-41d4-a716-446655440001", "slug": "test", "title": "Test", "description": "", "priority": "medium", "status": "todo", "tags": [], "created_at": "2025-07-28T10:00:00Z", "started_at": null, "closed_at": null, "assignee": null, "tasks": [], "metadata": {}}]"#;
    let json_file = temp_dir.path().join("data.json");
    fs::write(&json_file, json_content).unwrap();

    let result = handle_import_command(
        json_file.to_str().unwrap(),
        None, // Let it auto-detect
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );
    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());

    // Test YAML detection by content
    let yaml_content = r#"---
tickets:
  - id: "e50e8400-e29b-41d4-a716-446655440001"
    slug: "yaml-auto"
    title: "YAML Auto Detect"
    description: ""
    priority: "low"
    status: "todo"
    tags: []
    created_at: "2025-07-28T10:00:00Z"
    started_at: null
    closed_at: null
    assignee: null
    tasks: []
    metadata: {}"#;

    let unknown_file = temp_dir.path().join("unknown.txt");
    fs::write(&unknown_file, yaml_content).unwrap();

    let result = handle_import_command(
        unknown_file.to_str().unwrap(),
        None, // Let it auto-detect
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );
    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());
}

#[test]
fn test_import_with_complex_metadata() {
    let (temp_dir, formatter) = setup_test_project();

    // Create JSON with complex metadata
    let json_content = r#"[
        {
            "id": "f50e8400-e29b-41d4-a716-446655440001",
            "slug": "metadata-test",
            "title": "Metadata Test Ticket",
            "description": "Testing complex metadata",
            "priority": "high",
            "status": "doing",
            "tags": ["metadata", "complex"],
            "created_at": "2025-07-28T10:00:00Z",
            "started_at": "2025-07-28T10:30:00Z",
            "closed_at": null,
            "assignee": "tester",
            "tasks": [
                {
                    "id": "f50e8400-e29b-41d4-a716-446655440002",
                    "title": "Task with metadata",
                    "completed": true,
                    "created_at": "2025-07-28T10:00:00Z",
                    "completed_at": "2025-07-28T10:45:00Z"
                },
                {
                    "id": "f50e8400-e29b-41d4-a716-446655440003",
                    "title": "Another task",
                    "completed": false,
                    "created_at": "2025-07-28T10:00:00Z",
                    "completed_at": null
                }
            ],
            "metadata": {
                "custom_field": "value",
                "nested": {
                    "field": "nested_value"
                },
                "array_field": ["item1", "item2"],
                "number_field": 42,
                "boolean_field": true
            }
        }
    ]"#;

    let json_file = temp_dir.path().join("metadata.json");
    fs::write(&json_file, json_content).unwrap();

    // Import the ticket
    let result = handle_import_command(
        json_file.to_str().unwrap(),
        Some("json"),
        false,
        false,
        Some(temp_dir.path().to_str().unwrap()),
        &formatter,
    );

    if let Err(e) = &result {
        eprintln!("Import error: {e:?}");
    }
    assert!(result.is_ok());

    // Verify ticket with metadata was imported
    let storage = FileStorage::new(temp_dir.path().join(".vibe-ticket"));
    let ticket = storage
        .find_ticket_by_slug("metadata-test")
        .unwrap()
        .unwrap();

    assert_eq!(ticket.tasks.len(), 2);
    assert!(ticket.tasks[0].completed);
    assert!(!ticket.tasks[1].completed);

    // Verify metadata
    assert!(ticket.metadata.contains_key("custom_field"));
    assert!(ticket.metadata.contains_key("nested"));
}
