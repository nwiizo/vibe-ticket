//! Handler for the `task` command and its subcommands
//!
//! This module implements the logic for managing tasks within tickets,
//! including adding, completing, listing, and removing tasks.

use crate::cli::{OutputFormatter, find_project_root, handlers::resolve_ticket_ref};
use crate::core::{Task, TaskId};
use crate::error::{Result, VibeTicketError};
use crate::storage::{ActiveTicketRepository, FileStorage, TicketRepository};
use chrono::Utc;

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

/// Handler for the `task complete` subcommand
///
/// Marks a task as completed.
///
/// # Arguments
///
/// * `task_id` - ID of the task to complete
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_complete(
    task_id: String,
    ticket_ref: Option<String>,
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

    // Find and complete the task
    let mut task_found = false;
    for task in &mut ticket.tasks {
        if task.id == task_id {
            if task.completed {
                return Err(VibeTicketError::custom("Task is already completed"));
            }
            task.completed = true;
            task.completed_at = Some(Utc::now());
            task_found = true;
            break;
        }
    }

    if !task_found {
        return Err(VibeTicketError::custom(format!(
            "Task '{task_id}' not found in ticket"
        )));
    }

    // Save the updated ticket
    storage.save(&ticket)?;

    // Calculate completion stats
    let completed_count = ticket.tasks.iter().filter(|t| t.completed).count();
    let total_count = ticket.tasks.len();

    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
            "task_id": task_id.to_string(),
            "progress": {
                "completed": completed_count,
                "total": total_count,
                "percentage": if total_count > 0 { (completed_count * 100) / total_count } else { 0 },
            }
        }))?;
    } else {
        output.success(&format!("Completed task in ticket '{}'", ticket.slug));
        output.info(&format!(
            "Progress: {completed_count}/{total_count} tasks completed"
        ));

        if completed_count == total_count && total_count > 0 {
            output.info("🎉 All tasks completed!");
        }
    }

    Ok(())
}

/// Handler for the `task uncomplete` subcommand
///
/// Marks a completed task as incomplete.
///
/// # Arguments
///
/// * `task_id` - ID of the task to uncomplete
/// * `ticket_ref` - Optional ticket ID or slug (defaults to active ticket)
/// * `project_dir` - Optional project directory path
/// * `output` - Output formatter for displaying results
pub fn handle_task_uncomplete(
    task_id: String,
    ticket_ref: Option<String>,
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

    // Find and uncomplete the task
    let mut task_found = false;
    for task in &mut ticket.tasks {
        if task.id == task_id {
            if !task.completed {
                return Err(VibeTicketError::custom("Task is not completed"));
            }
            task.completed = false;
            task.completed_at = None;
            task_found = true;
            break;
        }
    }

    if !task_found {
        return Err(VibeTicketError::custom(format!(
            "Task '{task_id}' not found in ticket"
        )));
    }

    // Save the updated ticket
    storage.save(&ticket)?;

    // Calculate completion stats
    let completed_count = ticket.tasks.iter().filter(|t| t.completed).count();
    let total_count = ticket.tasks.len();

    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "status": "success",
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
            "task_id": task_id.to_string(),
            "progress": {
                "completed": completed_count,
                "total": total_count,
                "percentage": if total_count > 0 { (completed_count * 100) / total_count } else { 0 },
            }
        }))?;
    } else {
        output.success(&format!(
            "Marked task as incomplete in ticket '{}'",
            ticket.slug
        ));
        output.info(&format!(
            "Progress: {completed_count}/{total_count} tasks completed"
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
    let ticket = storage.load(&ticket_id)?;

    // Filter tasks based on flags
    let mut tasks: Vec<&Task> = ticket.tasks.iter().collect();
    if completed_only {
        tasks.retain(|t| t.completed);
    } else if incomplete_only {
        tasks.retain(|t| !t.completed);
    }

    // Calculate stats
    let total_count = ticket.tasks.len();
    let completed_count = ticket.tasks.iter().filter(|t| t.completed).count();

    // Output results
    if output.is_json() {
        output.print_json(&serde_json::json!({
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
            "ticket_title": ticket.title,
            "progress": {
                "completed": completed_count,
                "total": total_count,
                "percentage": if total_count > 0 { (completed_count * 100) / total_count } else { 0 },
            },
            "tasks": tasks.iter().map(|t| serde_json::json!({
                "id": t.id.to_string(),
                "title": t.title,
                "completed": t.completed,
                "created_at": t.created_at,
                "completed_at": t.completed_at,
            })).collect::<Vec<_>>(),
        }))?;
    } else {
        output.info(&format!("Tasks for ticket: {}", ticket.slug));
        output.info(&format!("Title: {}", ticket.title));
        output.info(&format!(
            "Progress: {completed_count}/{total_count} completed"
        ));

        if tasks.is_empty() {
            output.info("\nNo tasks found");
        } else {
            output.info("\nTasks:");
            for task in tasks {
                let checkbox = if task.completed { "✓" } else { "○" };
                let status = if task.completed { "(completed)" } else { "" };
                output.info(&format!(
                    "  {} [{}] {} {}",
                    checkbox,
                    &task.id.to_string()[..8], // Show first 8 chars of ID
                    task.title,
                    status
                ));
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
