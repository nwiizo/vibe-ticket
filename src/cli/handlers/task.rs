//! Handler for the `task` command and its subcommands
//!
//! This module implements the logic for managing tasks within tickets,
//! including adding, completing, listing, and removing tasks.

use crate::cli::OutputFormatter;
use crate::error::Result;

/// Handler for the `task add` subcommand
///
/// Adds a new task to a ticket.
///
/// # Arguments
///
/// * `title` - Title of the task to add
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_add(
    title: String,
    ticket_ref: Option<String>,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    use crate::cli::handlers::task_common::{TaskHandler, TaskOperationResult};
    use crate::core::Task;
    
    let handler = TaskHandler::new(project_dir.as_deref(), output)?;
    
    handler.execute_task_operation(
        ticket_ref.as_deref(),
        None,
        |ticket, _| {
            let task = Task::new(title);
            ticket.tasks.push(task.clone());
            Ok(TaskOperationResult::added(&ticket.slug, task))
        },
    )
}

/// Handler for the `task complete` subcommand
///
/// Marks a task as completed in a ticket.
///
/// # Arguments
///
/// * `task_id` - ID of the task to complete (can be index or UUID)
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_complete(
    task_id: String,
    ticket_ref: Option<String>,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    use crate::cli::handlers::task_common::{TaskHandler, TaskOperationResult};
    
    let handler = TaskHandler::new(project_dir.as_deref(), output)?;
    
    handler.execute_task_operation(
        ticket_ref.as_deref(),
        Some(task_id),
        |ticket, task_index| {
            let index = task_index.unwrap();
            ticket.tasks[index].complete();
            Ok(TaskOperationResult::completed(&ticket.slug, &ticket.tasks[index]))
        },
    )
}

/// Handler for the `task uncomplete` subcommand
///
/// Marks a task as not completed in a ticket.
///
/// # Arguments
///
/// * `task_id` - ID of the task to uncomplete (can be index or UUID)
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_uncomplete(
    task_id: String,
    ticket_ref: Option<String>,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    use crate::cli::handlers::task_common::{TaskHandler, TaskOperationResult};
    
    let handler = TaskHandler::new(project_dir.as_deref(), output)?;
    
    handler.execute_task_operation(
        ticket_ref.as_deref(),
        Some(task_id),
        |ticket, task_index| {
            let index = task_index.unwrap();
            ticket.tasks[index].uncomplete();
            Ok(TaskOperationResult::uncompleted(&ticket.slug, &ticket.tasks[index]))
        },
    )
}

/// Handler for the `task list` subcommand
///
/// Lists all tasks in a ticket.
///
/// # Arguments
///
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `completed_only` - Only show completed tasks
/// * `incomplete_only` - Only show incomplete tasks
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_list(
    ticket_ref: Option<String>,
    completed_only: bool,
    incomplete_only: bool,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    use crate::cli::handlers::task_common::TaskHandler;
    
    let handler = TaskHandler::new(project_dir.as_deref(), output)?;
    handler.list_tasks(ticket_ref.as_deref(), completed_only, incomplete_only)
}

/// Handler for the `task remove` subcommand
///
/// Removes a task from a ticket.
///
/// # Arguments
///
/// * `task_id` - ID of the task to remove (can be index or UUID)
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_remove(
    task_id: String,
    ticket_ref: Option<String>,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    use crate::cli::handlers::task_common::{TaskHandler, TaskOperationResult};
    
    let handler = TaskHandler::new(project_dir.as_deref(), output)?;
    
    handler.execute_task_operation(
        ticket_ref.as_deref(),
        Some(task_id),
        |ticket, task_index| {
            let index = task_index.unwrap();
            let task_title = ticket.tasks[index].title.clone();
            ticket.tasks.remove(index);
            Ok(TaskOperationResult::removed(&ticket.slug, &task_title))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handlers::resolve_ticket_ref;
    use crate::cli::output::OutputFormatter;
    use crate::core::{Task, Ticket};
    use crate::error::VibeTicketError;
    use crate::storage::{FileStorage, TicketRepository, ActiveTicketRepository};
    use chrono::Utc;
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, FileStorage, OutputFormatter) {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(storage_path.join("tickets")).unwrap();
        let storage = FileStorage::new(storage_path);
        let formatter = OutputFormatter::new(false, false);
        (temp_dir, storage, formatter)
    }

    fn create_test_ticket(storage: &FileStorage) -> (crate::core::TicketId, Ticket) {
        let ticket = Ticket::new("test-ticket".to_string(), "Test Ticket".to_string());
        let ticket_id = ticket.id.clone();
        storage.save(&ticket).unwrap();
        storage.set_active(&ticket_id).unwrap();
        (ticket_id, ticket)
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test task".to_string());
        assert_eq!(task.title, "Test task");
        assert!(!task.completed);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_handle_task_add_to_active_ticket() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (ticket_id, _) = create_test_ticket(&storage);

        // Add task to active ticket
        let result = handle_task_add(
            "New task".to_string(),
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());

        // Verify task was added
        let ticket = storage.load(&ticket_id).unwrap();
        assert_eq!(ticket.tasks.len(), 1);
        assert_eq!(ticket.tasks[0].title, "New task");
        assert!(!ticket.tasks[0].completed);
    }

    #[test]
    fn test_handle_task_add_to_specific_ticket() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let ticket = Ticket::new("other-ticket".to_string(), "Other Ticket".to_string());
        let ticket_id = ticket.id.clone();
        storage.save(&ticket).unwrap();

        // Add task to specific ticket
        let result = handle_task_add(
            "Specific task".to_string(),
            Some("other-ticket".to_string()),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());

        // Verify task was added
        let ticket = storage.load(&ticket_id).unwrap();
        assert_eq!(ticket.tasks.len(), 1);
        assert_eq!(ticket.tasks[0].title, "Specific task");
    }

    #[test]
    fn test_handle_task_complete() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (ticket_id, mut ticket) = create_test_ticket(&storage);

        // Add a task
        let task = Task::new("Task to complete".to_string());
        let task_id = task.id.to_string();
        ticket.tasks.push(task);
        storage.save(&ticket).unwrap();

        // Complete the task
        let result = handle_task_complete(
            task_id,
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());

        // Verify task was completed
        let ticket = storage.load(&ticket_id).unwrap();
        assert!(ticket.tasks[0].completed);
        assert!(ticket.tasks[0].completed_at.is_some());
    }

    #[test]
    fn test_handle_task_complete_already_completed() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (_, mut ticket) = create_test_ticket(&storage);

        // Add a completed task
        let mut task = Task::new("Already completed".to_string());
        task.completed = true;
        task.completed_at = Some(Utc::now());
        let task_id = task.id.to_string();
        ticket.tasks.push(task);
        storage.save(&ticket).unwrap();

        // Try to complete again
        let result = handle_task_complete(
            task_id,
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("already completed")
        );
    }

    #[test]
    fn test_handle_task_uncomplete() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (ticket_id, mut ticket) = create_test_ticket(&storage);

        // Add a completed task
        let mut task = Task::new("Completed task".to_string());
        task.completed = true;
        task.completed_at = Some(Utc::now());
        let task_id_str = task.id.to_string();
        ticket.tasks.push(task);
        storage.save(&ticket).unwrap();

        // Uncomplete the task
        let result = handle_task_uncomplete(
            task_id_str,
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());

        // Verify task was uncompleted
        let ticket = storage.load(&ticket_id).unwrap();
        assert!(!ticket.tasks[0].completed);
        assert!(ticket.tasks[0].completed_at.is_none());
    }

    #[test]
    fn test_handle_task_list() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (_, mut ticket) = create_test_ticket(&storage);

        // Add multiple tasks
        ticket.tasks.push(Task::new("Task 1".to_string()));
        ticket.tasks.push(Task::new("Task 2".to_string()));
        let mut completed_task = Task::new("Completed Task".to_string());
        completed_task.completed = true;
        completed_task.completed_at = Some(Utc::now());
        ticket.tasks.push(completed_task);
        storage.save(&ticket).unwrap();

        // List all tasks
        let result = handle_task_list(
            None,
            false,
            false,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_task_list_completed_only() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (_, mut ticket) = create_test_ticket(&storage);

        // Add mixed tasks
        ticket.tasks.push(Task::new("Pending Task".to_string()));
        let mut completed_task = Task::new("Completed Task".to_string());
        completed_task.completed = true;
        completed_task.completed_at = Some(Utc::now());
        ticket.tasks.push(completed_task);
        storage.save(&ticket).unwrap();

        // List only completed tasks
        let result = handle_task_list(
            None,
            true,
            false,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_task_remove() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (ticket_id, mut ticket) = create_test_ticket(&storage);

        // Add multiple tasks
        ticket.tasks.push(Task::new("Task 1".to_string()));
        let task_to_remove = Task::new("Task 2".to_string());
        let task_id_str = task_to_remove.id.to_string();
        ticket.tasks.push(task_to_remove);
        ticket.tasks.push(Task::new("Task 3".to_string()));
        storage.save(&ticket).unwrap();

        // Remove task 2
        let result = handle_task_remove(
            task_id_str,
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());

        // Verify task was removed
        let ticket = storage.load(&ticket_id).unwrap();
        assert_eq!(ticket.tasks.len(), 2);
        assert_eq!(ticket.tasks[0].title, "Task 1");
        assert_eq!(ticket.tasks[1].title, "Task 3");
    }

    #[test]
    fn test_handle_task_remove_with_confirmation() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (_, mut ticket) = create_test_ticket(&storage);

        // Add a task
        let task = Task::new("Task to remove".to_string());
        let task_id_str = task.id.to_string();
        ticket.tasks.push(task);
        storage.save(&ticket).unwrap();

        // Try to remove without force (should ask for confirmation)
        let result = handle_task_remove(
            task_id_str,
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());

        // Task should still be there
        let ticket = storage.load(&ticket.id).unwrap();
        assert_eq!(ticket.tasks.len(), 1);
    }

    #[test]
    fn test_task_add_no_active_ticket() {
        let (temp_dir, _, formatter) = setup_test_env();

        // Try to add task without active ticket
        let result = handle_task_add(
            "New task".to_string(),
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VibeTicketError::NoActiveTicket
        ));
    }

    #[test]
    fn test_task_complete_invalid_id() {
        let (temp_dir, storage, formatter) = setup_test_env();
        let (_, _) = create_test_ticket(&storage);

        // Try to complete non-existent task
        let result = handle_task_complete(
            "invalid-id".to_string(),
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_ticket_ref_by_id() {
        let (_, storage, _) = setup_test_env();
        let ticket = Ticket::new("test-slug".to_string(), "Test".to_string());
        let ticket_id = ticket.id.clone();
        storage.save(&ticket).unwrap();

        let resolved = resolve_ticket_ref(&storage, &ticket_id.to_string()).unwrap();
        assert_eq!(resolved, ticket_id);
    }

    #[test]
    fn test_resolve_ticket_ref_by_slug() {
        let (_, storage, _) = setup_test_env();
        let ticket = Ticket::new("test-slug".to_string(), "Test".to_string());
        let ticket_id = ticket.id.clone();
        storage.save(&ticket).unwrap();

        let resolved = resolve_ticket_ref(&storage, "test-slug").unwrap();
        assert_eq!(resolved, ticket_id);
    }

    #[test]
    fn test_resolve_ticket_ref_not_found() {
        let (_, storage, _) = setup_test_env();

        let result = resolve_ticket_ref(&storage, "non-existent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VibeTicketError::TicketNotFound { .. }
        ));
    }

    #[test]
    fn test_json_output_format() {
        let (temp_dir, storage, _json_formatter) = setup_test_env();
        let formatter = OutputFormatter::new(true, false); // JSON output
        let (_, _) = create_test_ticket(&storage);

        // Add task with JSON output
        let result = handle_task_add(
            "JSON task".to_string(),
            None,
            Some(temp_dir.path().to_str().unwrap().to_string()),
            &formatter,
        );

        assert!(result.is_ok());
    }
}
