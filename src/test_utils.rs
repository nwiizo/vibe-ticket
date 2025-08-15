//! Common test utilities for vibe-ticket
//!
//! This module provides shared test helpers and fixtures to reduce duplication
//! across test modules.

#[cfg(test)]
pub mod test {
    use crate::core::{Priority, Status, Task, Ticket, TicketId};
    use chrono::Utc;
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Creates a test ticket with default values
    #[must_use]
    pub fn create_test_ticket() -> Ticket {
        create_test_ticket_with_id(TicketId::new())
    }

    /// Creates a test ticket with a specific ID
    #[must_use]
    pub fn create_test_ticket_with_id(id: TicketId) -> Ticket {
        Ticket {
            id,
            slug: "test-ticket".to_string(),
            title: "Test Ticket".to_string(),
            description: "Test description".to_string(),
            status: Status::Todo,
            priority: Priority::Medium,
            tags: vec!["test".to_string()],
            assignee: None,
            tasks: vec![],
            metadata: HashMap::new(),
            created_at: Utc::now(),
            started_at: None,
            closed_at: None,
        }
    }

    /// Creates a test ticket with custom fields
    pub struct TestTicketBuilder {
        ticket: Ticket,
    }

    impl Default for TestTicketBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TestTicketBuilder {
        #[must_use]
        pub fn new() -> Self {
            Self {
                ticket: create_test_ticket(),
            }
        }

        #[must_use]
        pub fn with_slug(mut self, slug: &str) -> Self {
            self.ticket.slug = slug.to_string();
            self
        }

        #[must_use]
        pub fn with_title(mut self, title: &str) -> Self {
            self.ticket.title = title.to_string();
            self
        }

        #[must_use]
        pub fn with_status(mut self, status: Status) -> Self {
            self.ticket.status = status;
            self
        }

        #[must_use]
        pub fn with_priority(mut self, priority: Priority) -> Self {
            self.ticket.priority = priority;
            self
        }

        pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
            self.ticket.tags = tags
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect();
            self
        }

        #[must_use]
        pub fn with_task(mut self, title: &str) -> Self {
            self.ticket.tasks.push(Task::new(title.to_string()));
            self
        }

        #[must_use]
        pub fn build(self) -> Ticket {
            self.ticket
        }
    }

    /// Creates a temporary directory for testing
    #[must_use]
    pub fn create_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    /// Creates a test project directory structure
    #[must_use]
    pub fn create_test_project() -> (TempDir, std::path::PathBuf) {
        let temp_dir = create_temp_dir();
        let project_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&project_dir).expect("Failed to create project directory");
        (temp_dir, project_dir)
    }

    /// Asserts that two tickets are equal, ignoring timestamps
    pub fn assert_tickets_equal_ignore_timestamps(left: &Ticket, right: &Ticket) {
        assert_eq!(left.id, right.id);
        assert_eq!(left.slug, right.slug);
        assert_eq!(left.title, right.title);
        assert_eq!(left.description, right.description);
        assert_eq!(left.status, right.status);
        assert_eq!(left.priority, right.priority);
        assert_eq!(left.tags, right.tags);
        assert_eq!(left.assignee, right.assignee);
        assert_eq!(left.tasks.len(), right.tasks.len());
        // Don't compare timestamps as they may differ slightly
    }

    /// Common test assertions for ID types
    #[macro_export]
    macro_rules! assert_id_properties {
        ($id:expr) => {
            assert!(!$id.to_string().is_empty());
            assert_eq!($id.short().len(), 8);
            assert!($id.to_string().starts_with(&$id.short()));
        };
    }

    /// Common test assertions for enum properties
    #[macro_export]
    macro_rules! assert_enum_properties {
        ($enum_type:ty, $values:expr) => {
            let all = <$enum_type>::all();
            assert_eq!(all.len(), $values.len());
            for (i, value) in $values.iter().enumerate() {
                assert_eq!(all[i], *value);
            }
        };
    }
}
