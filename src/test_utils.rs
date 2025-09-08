//! Test utilities for vibe-ticket
//!
//! This module provides common test fixtures and utilities to reduce
//! duplication in test code across the codebase.

#![cfg(test)]

use crate::core::{Priority, Status, Task, Ticket, TicketId};
use crate::storage::{FileStorage, TicketRepository};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for creating a temporary project
pub struct TestProject {
    pub temp_dir: TempDir,
    pub project_root: PathBuf,
    pub tickets_dir: PathBuf,
    pub storage: FileStorage,
}

impl TestProject {
    /// Create a new test project with initialized vibe-ticket directory
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_root = temp_dir.path().to_path_buf();
        let tickets_dir = project_root.join(".vibe-ticket");

        std::fs::create_dir(&tickets_dir).expect("Failed to create tickets dir");

        let storage = FileStorage::new(tickets_dir.clone());

        Self {
            temp_dir,
            project_root,
            tickets_dir,
            storage,
        }
    }

    /// Create a test project with sample tickets
    pub fn with_sample_tickets() -> Self {
        let mut project = Self::new();

        // Add some sample tickets
        let tickets = vec![
            create_test_ticket("Fix login bug", Priority::High, Status::Todo),
            create_test_ticket("Add search feature", Priority::Medium, Status::Doing),
            create_test_ticket("Update documentation", Priority::Low, Status::Done),
        ];

        for ticket in tickets {
            project
                .storage
                .save(&ticket)
                .expect("Failed to save ticket");
        }

        project
    }

    /// Get the path to the project root
    pub fn root_path(&self) -> &PathBuf {
        &self.project_root
    }

    /// Get the path as a string
    pub fn root_path_str(&self) -> &str {
        self.project_root.to_str().expect("Invalid path")
    }

    /// Create and save a ticket
    pub fn create_ticket(&self, title: &str) -> Ticket {
        let ticket = create_test_ticket(title, Priority::Medium, Status::Todo);
        self.storage.save(&ticket).expect("Failed to save ticket");
        ticket
    }

    /// Set the active ticket
    pub fn set_active(&self, ticket_id: &TicketId) {
        let active_path = self.tickets_dir.join("active_ticket");
        std::fs::write(active_path, ticket_id.to_string()).expect("Failed to set active ticket");
    }
}

/// Create a test ticket with default values
pub fn create_test_ticket(title: &str, priority: Priority, status: Status) -> Ticket {
    Ticket {
        id: TicketId::new(),
        slug: format!("test-{}", title.to_lowercase().replace(' ', "-")),
        title: title.to_string(),
        description: format!("Description for {}", title),
        priority,
        status,
        tags: vec!["test".to_string()],
        created_at: Utc::now(),
        started_at: if status == Status::Doing {
            Some(Utc::now())
        } else {
            None
        },
        closed_at: if status == Status::Done {
            Some(Utc::now())
        } else {
            None
        },
        assignee: None,
        tasks: vec![],
        metadata: HashMap::new(),
    }
}

/// Create a test ticket with tasks
pub fn create_ticket_with_tasks(title: &str, task_count: usize) -> Ticket {
    let mut ticket = create_test_ticket(title, Priority::Medium, Status::Todo);

    for i in 1..=task_count {
        let task = Task::new(format!("Task {}", i));
        ticket.tasks.push(task);
    }

    ticket
}

/// Create a completed test ticket
pub fn create_completed_ticket(title: &str) -> Ticket {
    let mut ticket = create_test_ticket(title, Priority::Low, Status::Done);
    ticket.closed_at = Some(Utc::now());

    // Add some completed tasks
    for i in 1..=3 {
        let mut task = Task::new(format!("Completed task {}", i));
        task.completed = true;
        task.completed_at = Some(Utc::now());
        ticket.tasks.push(task);
    }

    ticket
}

/// Assert that two tickets are equal (ignoring timestamps)
pub fn assert_tickets_equal(left: &Ticket, right: &Ticket) {
    assert_eq!(left.id, right.id, "Ticket IDs don't match");
    assert_eq!(left.slug, right.slug, "Ticket slugs don't match");
    assert_eq!(left.title, right.title, "Ticket titles don't match");
    assert_eq!(
        left.description, right.description,
        "Ticket descriptions don't match"
    );
    assert_eq!(
        left.priority, right.priority,
        "Ticket priorities don't match"
    );
    assert_eq!(left.status, right.status, "Ticket statuses don't match");
    assert_eq!(left.tags, right.tags, "Ticket tags don't match");
    assert_eq!(
        left.assignee, right.assignee,
        "Ticket assignees don't match"
    );
    assert_eq!(
        left.tasks.len(),
        right.tasks.len(),
        "Task counts don't match"
    );
}

/// Test data builder for complex scenarios
pub struct TestDataBuilder {
    tickets: Vec<Ticket>,
}

impl TestDataBuilder {
    pub fn new() -> Self {
        Self {
            tickets: Vec::new(),
        }
    }

    /// Add a ticket with specific properties
    pub fn with_ticket(mut self, title: &str, priority: Priority, status: Status) -> Self {
        self.tickets
            .push(create_test_ticket(title, priority, status));
        self
    }

    /// Add multiple tickets with the same status
    pub fn with_tickets_in_status(mut self, status: Status, count: usize) -> Self {
        for i in 1..=count {
            self.tickets.push(create_test_ticket(
                &format!("{:?} ticket {}", status, i),
                Priority::Medium,
                status.clone(),
            ));
        }
        self
    }

    /// Build and return the tickets
    pub fn build(self) -> Vec<Ticket> {
        self.tickets
    }

    /// Build and save to a test project
    pub fn build_in_project(self) -> TestProject {
        let project = TestProject::new();

        for ticket in self.tickets {
            project
                .storage
                .save(&ticket)
                .expect("Failed to save ticket");
        }

        project
    }
}

/// Macro for quickly creating test tickets
#[macro_export]
macro_rules! test_ticket {
    ($title:expr) => {
        $crate::test_utils::create_test_ticket($title, Priority::Medium, Status::Todo)
    };
    ($title:expr, $priority:expr) => {
        $crate::test_utils::create_test_ticket($title, $priority, Status::Todo)
    };
    ($title:expr, $priority:expr, $status:expr) => {
        $crate::test_utils::create_test_ticket($title, $priority, $status)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = TestProject::new();
        assert!(project.tickets_dir.exists());
    }

    #[test]
    fn test_project_with_sample_tickets() {
        let project = TestProject::with_sample_tickets();
        let tickets = project.storage.load_all().unwrap();
        assert_eq!(tickets.len(), 3);
    }

    #[test]
    fn test_data_builder() {
        let tickets = TestDataBuilder::new()
            .with_ticket("Bug fix", Priority::High, Status::Todo)
            .with_tickets_in_status(Status::Doing, 2)
            .build();

        assert_eq!(tickets.len(), 3);
        assert_eq!(tickets[0].title, "Bug fix");
        assert_eq!(tickets[1].status, Status::Doing);
    }
}
