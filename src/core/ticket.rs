use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Priority, Status, Task, TaskId, TicketId};

/// Represents a ticket in the vibe-ticket system
///
/// A ticket encapsulates a unit of work with associated metadata,
/// tasks, and lifecycle information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ticket {
    /// Unique identifier for the ticket
    pub id: TicketId,

    /// URL-friendly slug derived from the title
    pub slug: String,

    /// Human-readable title of the ticket
    pub title: String,

    /// Detailed description of the work to be done
    pub description: String,

    /// Priority level of the ticket
    pub priority: Priority,

    /// Current status of the ticket
    pub status: Status,

    /// Tags for categorization and filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Timestamp when the ticket was created
    pub created_at: DateTime<Utc>,

    /// Timestamp when work started on the ticket
    pub started_at: Option<DateTime<Utc>>,

    /// Timestamp when the ticket was closed
    pub closed_at: Option<DateTime<Utc>>,

    /// Username of the person assigned to the ticket
    pub assignee: Option<String>,

    /// List of tasks associated with this ticket
    #[serde(default)]
    pub tasks: Vec<Task>,

    /// Additional metadata for extensibility
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Ticket {
    /// Creates a new ticket with the given slug and title
    pub fn new(slug: impl Into<String>, title: impl Into<String>) -> Self {
        use super::TicketBuilder;
        TicketBuilder::new().slug(slug).title(title).build()
    }

    /// Creates a new ticket with a specific ID (useful for deserialization)
    pub fn with_id(id: TicketId, slug: impl Into<String>, title: impl Into<String>) -> Self {
        use super::TicketBuilder;
        TicketBuilder::new().id(id).slug(slug).title(title).build()
    }

    /// Starts work on the ticket, updating status and timestamp
    pub fn start(&mut self) {
        self.status = Status::Doing;
        self.started_at = Some(Utc::now());
    }

    /// Closes the ticket, updating status and timestamp
    pub fn close(&mut self) {
        self.status = Status::Done;
        self.closed_at = Some(Utc::now());
    }

    /// Adds a task to the ticket
    pub fn add_task(&mut self, title: impl Into<String>) -> TaskId {
        let task = Task::new(title);
        let task_id = task.id.clone();
        self.tasks.push(task);
        task_id
    }

    /// Marks a task as completed
    pub fn complete_task(&mut self, task_id: &TaskId) -> Result<(), String> {
        self.tasks
            .iter_mut()
            .find(|task| &task.id == task_id)
            .ok_or_else(|| format!("Task with ID {task_id} not found"))?
            .complete();
        Ok(())
    }

    /// Returns the number of completed tasks
    #[must_use]
    pub fn completed_tasks_count(&self) -> usize {
        self.tasks.iter().filter(|task| task.completed).count()
    }

    /// Returns the total number of tasks
    #[must_use]
    pub fn total_tasks_count(&self) -> usize {
        self.tasks.len()
    }

    /// Calculates the completion percentage
    #[must_use]
    pub fn completion_percentage(&self) -> f32 {
        if self.tasks.is_empty() {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                (self.completed_tasks_count() as f32 / self.total_tasks_count() as f32) * 100.0
            }
        }
    }

    /// Returns the duration the ticket has been open
    pub fn duration(&self) -> chrono::Duration {
        let end_time = self.closed_at.unwrap_or_else(Utc::now);
        end_time - self.created_at
    }

    /// Returns the working duration (from start to close/now)
    #[must_use]
    pub fn working_duration(&self) -> Option<chrono::Duration> {
        self.started_at.map(|start| {
            let end_time = self.closed_at.unwrap_or_else(Utc::now);
            end_time - start
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ticket() {
        let ticket = Ticket::new("test-ticket", "Test Ticket");
        assert_eq!(ticket.slug, "test-ticket");
        assert_eq!(ticket.title, "Test Ticket");
        assert_eq!(ticket.status, Status::Todo);
        assert!(ticket.started_at.is_none());
        assert!(ticket.closed_at.is_none());
    }

    #[test]
    fn test_start_ticket() {
        let mut ticket = Ticket::new("test", "Test");
        ticket.start();
        assert_eq!(ticket.status, Status::Doing);
        assert!(ticket.started_at.is_some());
    }

    #[test]
    fn test_close_ticket() {
        let mut ticket = Ticket::new("test", "Test");
        ticket.close();
        assert_eq!(ticket.status, Status::Done);
        assert!(ticket.closed_at.is_some());
    }

    #[test]
    fn test_task_management() {
        let mut ticket = Ticket::new("test", "Test");
        let task_id = ticket.add_task("Task 1");

        assert_eq!(ticket.total_tasks_count(), 1);
        assert_eq!(ticket.completed_tasks_count(), 0);

        ticket.complete_task(&task_id).unwrap();
        assert_eq!(ticket.completed_tasks_count(), 1);
        assert!((ticket.completion_percentage() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_with_id() {
        let id = TicketId::new();
        let ticket = Ticket::with_id(id.clone(), "custom-slug", "Custom Title");
        assert_eq!(ticket.id, id);
        assert_eq!(ticket.slug, "custom-slug");
        assert_eq!(ticket.title, "Custom Title");
        assert_eq!(ticket.description, "");
        assert_eq!(ticket.priority, Priority::default());
        assert_eq!(ticket.status, Status::default());
        assert!(ticket.tags.is_empty());
        assert!(ticket.assignee.is_none());
        assert!(ticket.tasks.is_empty());
        assert!(ticket.metadata.is_empty());
    }

    #[test]
    fn test_complete_nonexistent_task() {
        let mut ticket = Ticket::new("test", "Test");
        let fake_id = TaskId::new();

        let result = ticket.complete_task(&fake_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_completion_percentage_empty() {
        let ticket = Ticket::new("test", "Test");
        assert!(ticket.completion_percentage().abs() < f32::EPSILON);
    }

    #[test]
    fn test_completion_percentage_partial() {
        let mut ticket = Ticket::new("test", "Test");
        let task1 = ticket.add_task("Task 1");
        let _task2 = ticket.add_task("Task 2");
        ticket.add_task("Task 3");
        ticket.add_task("Task 4");

        ticket.complete_task(&task1).unwrap();
        assert_eq!(ticket.completed_tasks_count(), 1);
        assert_eq!(ticket.total_tasks_count(), 4);
        assert!((ticket.completion_percentage() - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_duration() {
        let mut ticket = Ticket::new("test", "Test");
        let creation_time = ticket.created_at;

        // Test open ticket duration
        let duration = ticket.duration();
        assert!(duration.num_seconds() >= 0);

        // Test closed ticket duration
        std::thread::sleep(std::time::Duration::from_millis(10));
        ticket.close();
        let closed_duration = ticket.duration();
        assert!(closed_duration.num_milliseconds() >= 10);
        assert_eq!(closed_duration, ticket.closed_at.unwrap() - creation_time);
    }

    #[test]
    fn test_working_duration() {
        let mut ticket = Ticket::new("test", "Test");

        // No working duration before start
        assert!(ticket.working_duration().is_none());

        // Start the ticket
        ticket.start();
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Working duration should exist
        let working_duration = ticket.working_duration();
        assert!(working_duration.is_some());
        assert!(working_duration.unwrap().num_milliseconds() >= 10);

        // Close the ticket
        ticket.close();
        let final_duration = ticket.working_duration().unwrap();
        assert_eq!(
            final_duration,
            ticket.closed_at.unwrap() - ticket.started_at.unwrap()
        );
    }

    #[test]
    fn test_ticket_with_tags_and_metadata() {
        let mut ticket = Ticket::new("test", "Test");

        // Add tags
        ticket.tags.push("bug".to_string());
        ticket.tags.push("urgent".to_string());
        assert_eq!(ticket.tags.len(), 2);
        assert!(ticket.tags.contains(&"bug".to_string()));

        // Add metadata
        ticket.metadata.insert(
            "reporter".to_string(),
            serde_json::json!("user@example.com"),
        );
        ticket
            .metadata
            .insert("component".to_string(), serde_json::json!("backend"));
        assert_eq!(ticket.metadata.len(), 2);
        assert_eq!(
            ticket.metadata.get("reporter").unwrap(),
            &serde_json::json!("user@example.com")
        );
    }

    #[test]
    fn test_ticket_assignee() {
        let mut ticket = Ticket::new("test", "Test");
        assert!(ticket.assignee.is_none());

        ticket.assignee = Some("alice".to_string());
        assert_eq!(ticket.assignee, Some("alice".to_string()));
    }

    #[test]
    fn test_ticket_priority_and_status() {
        let mut ticket = Ticket::new("test", "Test");

        // Test priority changes
        ticket.priority = Priority::High;
        assert_eq!(ticket.priority, Priority::High);

        ticket.priority = Priority::Critical;
        assert_eq!(ticket.priority, Priority::Critical);

        // Test status changes
        assert_eq!(ticket.status, Status::Todo);

        ticket.status = Status::Review;
        assert_eq!(ticket.status, Status::Review);

        ticket.status = Status::Blocked;
        assert_eq!(ticket.status, Status::Blocked);
    }

    #[test]
    fn test_ticket_serde() {
        let mut ticket = Ticket::new("test-serde", "Test Serialization");
        ticket.description = "Testing serialization and deserialization".to_string();
        ticket.priority = Priority::Medium;
        ticket.tags.push("test".to_string());
        ticket.assignee = Some("bob".to_string());

        let task_id = ticket.add_task("Serialize me");
        ticket.complete_task(&task_id).unwrap();

        // Serialize
        let json = serde_json::to_string(&ticket).unwrap();

        // Deserialize
        let deserialized: Ticket = serde_json::from_str(&json).unwrap();

        // Verify
        assert_eq!(ticket.id, deserialized.id);
        assert_eq!(ticket.slug, deserialized.slug);
        assert_eq!(ticket.title, deserialized.title);
        assert_eq!(ticket.description, deserialized.description);
        assert_eq!(ticket.priority, deserialized.priority);
        assert_eq!(ticket.status, deserialized.status);
        assert_eq!(ticket.tags, deserialized.tags);
        assert_eq!(ticket.assignee, deserialized.assignee);
        assert_eq!(ticket.tasks.len(), deserialized.tasks.len());
        assert_eq!(ticket.tasks[0].completed, deserialized.tasks[0].completed);
    }

    #[test]
    fn test_ticket_equality() {
        let ticket1 = Ticket::new("test", "Test");
        let mut ticket2 = ticket1.clone();

        // Should be equal after clone
        assert_eq!(ticket1, ticket2);

        // Should not be equal after modification
        ticket2.title = "Different".to_string();
        assert_ne!(ticket1, ticket2);
    }
}
