use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::TaskId;

/// Represents a task within a ticket
///
/// Tasks are smaller units of work that can be tracked
/// independently within a ticket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    /// Unique identifier for the task
    pub id: TaskId,

    /// Title describing what needs to be done
    pub title: String,

    /// Whether the task has been completed
    pub completed: bool,

    /// Timestamp when the task was created
    pub created_at: DateTime<Utc>,

    /// Timestamp when the task was completed
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Creates a new task with the given title
    pub fn new(title: impl Into<String>) -> Self {
        use super::TaskBuilder;
        TaskBuilder::new().title(title).build()
    }

    /// Creates a new task with a specific ID (useful for deserialization)
    pub fn with_id(id: TaskId, title: impl Into<String>) -> Self {
        use super::TaskBuilder;
        TaskBuilder::new().id(id).title(title).build()
    }

    /// Marks the task as completed
    pub fn complete(&mut self) {
        if !self.completed {
            self.completed = true;
            self.completed_at = Some(Utc::now());
        }
    }

    /// Marks the task as incomplete
    pub fn uncomplete(&mut self) {
        self.completed = false;
        self.completed_at = None;
    }

    /// Returns the duration since the task was created
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    /// Returns the duration the task took to complete
    pub fn completion_duration(&self) -> Option<chrono::Duration> {
        self.completed_at
            .map(|completed| completed - self.created_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task() {
        let task = Task::new("Test task");
        assert_eq!(task.title, "Test task");
        assert!(!task.completed);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_complete_task() {
        let mut task = Task::new("Test task");
        task.complete();

        assert!(task.completed);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_uncomplete_task() {
        let mut task = Task::new("Test task");
        task.complete();
        task.uncomplete();

        assert!(!task.completed);
        assert!(task.completed_at.is_none());
    }
}
