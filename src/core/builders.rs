use super::{Priority, Status, Task, TaskId, Ticket, TicketId};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Builder for creating Ticket instances
#[derive(Default)]
pub struct TicketBuilder {
    id: Option<TicketId>,
    slug: Option<String>,
    title: Option<String>,
    description: Option<String>,
    priority: Option<Priority>,
    status: Option<Status>,
    tags: Vec<String>,
    created_at: Option<DateTime<Utc>>,
    started_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
    assignee: Option<String>,
    tasks: Vec<Task>,
    metadata: HashMap<String, serde_json::Value>,
}

impl TicketBuilder {
    /// Create a new ticket builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the ticket ID
    #[must_use]
    pub const fn id(mut self, id: TicketId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the slug
    #[must_use]
    pub fn slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = Some(slug.into());
        self
    }

    /// Set the title
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the description
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the priority
    #[must_use]
    pub const fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set the status
    #[must_use]
    pub const fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    /// Add tags
    #[must_use]
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add a single tag
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set `created_at` timestamp
    #[must_use]
    pub const fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Set `started_at` timestamp
    #[must_use]
    pub const fn started_at(mut self, started_at: DateTime<Utc>) -> Self {
        self.started_at = Some(started_at);
        self
    }

    /// Set `closed_at` timestamp
    #[must_use]
    pub const fn closed_at(mut self, closed_at: DateTime<Utc>) -> Self {
        self.closed_at = Some(closed_at);
        self
    }

    /// Set assignee
    #[must_use]
    pub fn assignee(mut self, assignee: impl Into<String>) -> Self {
        self.assignee = Some(assignee.into());
        self
    }

    /// Add tasks
    #[must_use]
    pub fn tasks(mut self, tasks: Vec<Task>) -> Self {
        self.tasks = tasks;
        self
    }

    /// Add a single task
    #[must_use]
    pub fn task(mut self, task: Task) -> Self {
        self.tasks.push(task);
        self
    }

    /// Set metadata
    #[must_use]
    pub fn metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the ticket
    pub fn build(self) -> Ticket {
        Ticket {
            id: self.id.unwrap_or_default(),
            slug: self.slug.unwrap_or_default(),
            title: self.title.unwrap_or_default(),
            description: self.description.unwrap_or_default(),
            priority: self.priority.unwrap_or_default(),
            status: self.status.unwrap_or_default(),
            tags: self.tags,
            created_at: self.created_at.unwrap_or_else(Utc::now),
            started_at: self.started_at,
            closed_at: self.closed_at,
            assignee: self.assignee,
            tasks: self.tasks,
            metadata: self.metadata,
        }
    }
}

/// Builder for creating Task instances
#[derive(Default)]
pub struct TaskBuilder {
    id: Option<TaskId>,
    title: Option<String>,
    completed: bool,
    created_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

impl TaskBuilder {
    /// Create a new task builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the task ID
    #[must_use]
    pub const fn id(mut self, id: TaskId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the title
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set completion status
    #[must_use]
    pub const fn completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }

    /// Set `created_at` timestamp
    #[must_use]
    pub const fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Set `completed_at` timestamp
    #[must_use]
    pub const fn completed_at(mut self, completed_at: DateTime<Utc>) -> Self {
        self.completed_at = Some(completed_at);
        self
    }

    /// Build the task
    pub fn build(self) -> Task {
        Task {
            id: self.id.unwrap_or_default(),
            title: self.title.unwrap_or_default(),
            completed: self.completed,
            created_at: self.created_at.unwrap_or_else(Utc::now),
            completed_at: self.completed_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_builder() {
        let ticket = TicketBuilder::new()
            .slug("test-ticket")
            .title("Test Ticket")
            .description("A test ticket")
            .priority(Priority::High)
            .tag("test")
            .tag("builder")
            .build();

        assert_eq!(ticket.slug, "test-ticket");
        assert_eq!(ticket.title, "Test Ticket");
        assert_eq!(ticket.description, "A test ticket");
        assert_eq!(ticket.priority, Priority::High);
        assert_eq!(ticket.tags.len(), 2);
    }

    #[test]
    fn test_task_builder() {
        let task = TaskBuilder::new()
            .title("Test Task")
            .completed(true)
            .build();

        assert_eq!(task.title, "Test Task");
        assert!(task.completed);
    }
}
