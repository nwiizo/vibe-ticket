//! Handler for the `task` command and its subcommands
//!
//! This module implements the logic for managing tasks within tickets,
//! including adding, completing, listing, and removing tasks.

use crate::cli::{OutputFormatter, find_project_root};
use crate::cli::handlers::common::resolve_ticket_ref;
use crate::core::{Task, TaskId};
use crate::error::{Result, VibeTicketError};
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};

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
    use super::common::{HandlerContext, TicketOperation};
    
    // Create handler context
    let ctx = HandlerContext::new(project_dir.as_deref())?;
    
    // Load the ticket
    let mut ticket = ctx.load_ticket(ticket_ref.as_deref())?;
    
    // Create new task
    let task = Task::new(title);
    ticket.tasks.push(task.clone());
    
    // Save the updated ticket
    ctx.save_ticket(&ticket)?;
    
    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
            "task": {
                "id": task.id.to_string(),
                "title": task.title,
                "completed": task.completed,
            },
            "total_tasks": ticket.tasks.len(),
        }))?;
    } else {
        output.success(&format!("Added task to ticket '{}'", ticket.slug));
        output.info(&format!("Task ID: {}", task.id));
        output.info(&format!("Title: {}", task.title));
        output.info(&format!("Total tasks: {}", ticket.tasks.len()));
    }
    
    Ok(())
}

pub fn handle_task_complete(
    task_id: String,
    ticket_ref: Option<String>,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    use super::common::{HandlerContext, TicketOperation};
    
    // Create handler context
    let ctx = HandlerContext::new(project_dir.as_deref())?;
    
    // Load the ticket
    let mut ticket = ctx.load_ticket(ticket_ref.as_deref())?;
    
    // Parse task ID (could be index or UUID)
    let task_index = if let Ok(index) = task_id.parse::<usize>() {
        if index == 0 || index > ticket.tasks.len() {
            return Err(VibeTicketError::InvalidInput(
                format!("Task index {} is out of range (1-{})", index, ticket.tasks.len())
            ));
        }
        index - 1
    } else {
        // Try to find by UUID
        ticket.tasks.iter().position(|t| t.id.to_string() == task_id)
            .ok_or_else(|| VibeTicketError::TaskNotFound { id: task_id.clone() })?
    };
    
    // Check if task is already completed
    if ticket.tasks[task_index].completed {
        return Err(VibeTicketError::InvalidInput(
            format!("Task '{}' is already completed", ticket.tasks[task_index].title)
        ));
    }
    
    // Mark task as completed
    ticket.tasks[task_index].complete();
    
    // Save the updated ticket
    ctx.save_ticket(&ticket)?;
    
    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
            "task": {
                "id": ticket.tasks[task_index].id.to_string(),
                "title": ticket.tasks[task_index].title.clone(),
                "completed": true,
            },
            "progress": {
                "completed": ticket.completed_tasks_count(),
                "total": ticket.total_tasks_count(),
                "percentage": ticket.completion_percentage(),
            }
        }))?;
    } else {
        output.success(&format!("Completed task in ticket '{}'", ticket.slug));
        output.info(&format!("Task: {}", ticket.tasks[task_index].title));
        output.info(&format!(
            "Progress: {}/{} ({}%)",
            ticket.completed_tasks_count(),
            ticket.total_tasks_count(),
            ticket.completion_percentage()
        ));
    }
    
    Ok(())
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
    use super::common::{HandlerContext, TicketOperation};
    
    // Create handler context
    let ctx = HandlerContext::new(project_dir.as_deref())?;
    
    // Load the ticket
    let mut ticket = ctx.load_ticket(ticket_ref.as_deref())?;
    
    // Parse task ID (could be index or UUID)
    let task_index = if let Ok(index) = task_id.parse::<usize>() {
        if index == 0 || index > ticket.tasks.len() {
            return Err(VibeTicketError::InvalidInput(
                format!("Task index {} is out of range (1-{})", index, ticket.tasks.len())
            ));
        }
        index - 1
    } else {
        // Try to find by UUID
        ticket.tasks.iter().position(|t| t.id.to_string() == task_id)
            .ok_or_else(|| VibeTicketError::TaskNotFound { id: task_id.clone() })?
    };
    
    // Mark task as not completed
    ticket.tasks[task_index].uncomplete();
    
    // Save the updated ticket
    ctx.save_ticket(&ticket)?;
    
    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
            "task": {
                "id": ticket.tasks[task_index].id.to_string(),
                "title": ticket.tasks[task_index].title.clone(),
                "completed": false,
            },
            "progress": {
                "completed": ticket.completed_tasks_count(),
                "total": ticket.total_tasks_count(),
                "percentage": ticket.completion_percentage(),
            }
        }))?;
    } else {
        output.success(&format!("Marked task as incomplete in ticket '{}'", ticket.slug));
        output.info(&format!("Task: {}", ticket.tasks[task_index].title));
        output.info(&format!(
            "Progress: {}/{} ({}%)",
            ticket.completed_tasks_count(),
            ticket.total_tasks_count(),
            ticket.completion_percentage()
        ));
    }
    
    Ok(())
}

/// Handler for the `task list` subcommand
///
/// Lists all tasks in a ticket.
///
/// # Arguments
///
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `completed_only` - Show only completed tasks
/// * `incomplete_only` - Show only incomplete tasks
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_list(
    ticket_ref: Option<String>,
    completed_only: bool,
    incomplete_only: bool,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    use super::common::{HandlerContext, TicketOperation};
    
    // Create handler context
    let ctx = HandlerContext::new(project_dir.as_deref())?;
    
    // Load the ticket
    let ticket = ctx.load_ticket(ticket_ref.as_deref())?;
    
    // Filter tasks
    let tasks: Vec<_> = ticket.tasks.iter().enumerate()
        .filter(|(_, task)| {
            if completed_only {
                task.completed
            } else if incomplete_only {
                !task.completed
            } else {
                true
            }
        })
        .collect();
    
    // Output results
    if output.is_json() {
        let tasks_json: Vec<_> = tasks.iter()
            .map(|(idx, task)| serde_json::json!({
                "index": idx + 1,
                "id": task.id.to_string(),
                "title": task.title.clone(),
                "completed": task.completed,
                "created_at": task.created_at,
                "completed_at": task.completed_at,
            }))
            .collect();
        
        output.print_json(&serde_json::json!({
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
            "tasks": tasks_json,
            "total": tasks.len(),
            "completed": ticket.completed_tasks_count(),
            "percentage": ticket.completion_percentage(),
        }))?;
    } else if tasks.is_empty() {
        let filter_msg = if completed_only {
            " (completed)"
        } else if incomplete_only {
            " (incomplete)"
        } else {
            ""
        };
        output.info(&format!("No tasks{} in ticket '{}'", filter_msg, ticket.slug));
    } else {
        output.info(&format!("Tasks in ticket '{}':", ticket.slug));
        output.info(&format!(
            "Progress: {}/{} ({}%)\n",
            ticket.completed_tasks_count(),
            ticket.total_tasks_count(),
            ticket.completion_percentage()
        ));
        
        for (idx, task) in tasks {
            let status = if task.completed { "✓" } else { "○" };
            println!("{} {}. {} - {}", status, idx + 1, task.title, task.id);
            if task.completed {
                if let Some(completed_at) = task.completed_at {
                    println!("     Completed: {}", completed_at.format("%Y-%m-%d %H:%M"));
                }
            }
        }
    }
    
    Ok(())
}

/// Handler for the `task remove` subcommand
///
/// Removes a task from a ticket.
///
/// # Arguments
///
/// * `task_id` - ID of the task to remove
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `force` - Skip confirmation
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_remove(
    task_id: String,
    ticket_ref: Option<String>,
    force: bool,
    project_dir: Option<String>,
    output: &OutputFormatter,
) -> Result<()> {
    // Ensure project is initialized
    let project_root = find_project_root(project_dir.as_deref())?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");

    // Initialize storage
    let storage = FileStorage::new(&vibe_ticket_dir);

    // Get the active ticket if no ticket specified
    let ticket_id = if let Some(ref_str) = ticket_ref {
        resolve_ticket_ref(&storage, &ref_str)?
    } else {
        // Get active ticket
        storage
            .get_active()?
            .ok_or(VibeTicketError::NoActiveTicket)?
    };

    // Load the ticket
    let mut ticket = storage.load(&ticket_id)?;

    // Parse task ID
    let task_id = TaskId::parse_str(&task_id)
        .map_err(|_| VibeTicketError::custom(format!("Invalid task ID: {task_id}")))?;

    // Find the task
    let task_index = ticket
        .tasks
        .iter()
        .position(|t| t.id == task_id)
        .ok_or_else(|| VibeTicketError::custom(format!("Task '{task_id}' not found in ticket")))?;

    let task = &ticket.tasks[task_index];

    // Confirm removal if not forced
    if !force {
        output.warning(&format!(
            "Are you sure you want to remove task: '{}'?",
            task.title
        ));
        output.info("Use --force to skip this confirmation");
        return Ok(());
    }

    // Remove the task
    let removed_task = ticket.tasks.remove(task_index);

    // Save the updated ticket
    storage.save(&ticket)?;

    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
            "removed_task": {
                "id": removed_task.id.to_string(),
                "title": removed_task.title,
                "was_completed": removed_task.completed,
            },
            "remaining_tasks": ticket.tasks.len(),
        }))?;
    } else {
        output.success(&format!("Removed task from ticket '{}'", ticket.slug));
        output.info(&format!("Removed: {}", removed_task.title));
        output.info(&format!("Remaining tasks: {}", ticket.tasks.len()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::output::OutputFormatter;
    use crate::core::Ticket;
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
        task.complete();
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
        task.complete();
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
        completed_task.complete();
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
        completed_task.complete();
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
            true, // force
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
            false, // no force
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
