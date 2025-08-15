use crate::cli::OutputFormatter;
use crate::core::{Task, Ticket};
use crate::error::{Result, VibeTicketError};
use crate::cli::handlers::common::{HandlerContext, TicketOperation};
use serde_json::json;

/// Common task operation handler
pub struct TaskHandler<'a> {
    ctx: HandlerContext,
    output: &'a OutputFormatter,
}

impl<'a> TaskHandler<'a> {
    /// Create a new task handler
    pub fn new(project_dir: Option<&str>, output: &'a OutputFormatter) -> Result<Self> {
        let ctx = HandlerContext::new(project_dir)?;
        Ok(Self { ctx, output })
    }
    
    /// Execute a task operation
    pub fn execute_task_operation<F>(
        &self,
        ticket_ref: Option<&str>,
        task_id: Option<String>,
        operation: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut Ticket, Option<usize>) -> Result<TaskOperationResult>,
    {
        // Load the ticket
        let mut ticket = self.ctx.load_ticket(ticket_ref)?;
        
        // Resolve task index if task_id is provided
        let task_index = if let Some(id) = task_id {
            Some(self.resolve_task_index(&ticket, &id)?)
        } else {
            None
        };
        
        // Execute the operation
        let result = operation(&mut ticket, task_index)?;
        
        // Save the ticket if modified
        if result.modified {
            self.ctx.save_ticket(&ticket)?;
        }
        
        // Output the result
        self.output_result(&ticket, &result)?;
        
        Ok(())
    }
    
    /// Resolve task ID to index
    fn resolve_task_index(&self, ticket: &Ticket, task_id: &str) -> Result<usize> {
        if let Ok(index) = task_id.parse::<usize>() {
            if index == 0 || index > ticket.tasks.len() {
                return Err(VibeTicketError::InvalidInput(
                    format!("Task index {} is out of range (1-{})", index, ticket.tasks.len())
                ));
            }
            Ok(index - 1)
        } else {
            // Try to find by UUID
            ticket.tasks.iter().position(|t| t.id.to_string() == task_id)
                .ok_or_else(|| VibeTicketError::TaskNotFound { id: task_id.to_string() })
        }
    }
    
    /// Output operation result
    fn output_result(&self, ticket: &Ticket, result: &TaskOperationResult) -> Result<()> {
        if self.output.is_json() {
            self.output.print_json(&result.to_json(ticket))?;
        } else {
            self.output.success(&result.success_message);
            if let Some(ref info) = result.info_message {
                self.output.info(info);
            }
            if result.show_progress {
                self.output.info(&format!(
                    "Progress: {}/{} ({}%)",
                    ticket.completed_tasks_count(),
                    ticket.total_tasks_count(),
                    ticket.completion_percentage()
                ));
            }
        }
        Ok(())
    }
    
    /// List tasks with filtering
    pub fn list_tasks(
        &self,
        ticket_ref: Option<&str>,
        completed_only: bool,
        incomplete_only: bool,
    ) -> Result<()> {
        let ticket = self.ctx.load_ticket(ticket_ref)?;
        
        let tasks: Vec<&Task> = ticket.tasks.iter()
            .filter(|task| {
                if completed_only && !task.completed {
                    return false;
                }
                if incomplete_only && task.completed {
                    return false;
                }
                true
            })
            .collect();
        
        if self.output.is_json() {
            let json_tasks: Vec<_> = tasks.iter().enumerate()
                .map(|(i, task)| json!({
                    "index": i + 1,
                    "id": task.id.to_string(),
                    "title": task.title,
                    "completed": task.completed,
                    "created_at": task.created_at,
                    "completed_at": task.completed_at,
                }))
                .collect();
            
            self.output.print_json(&json!({
                "ticket_id": ticket.id.to_string(),
                "ticket_slug": ticket.slug,
                "tasks": json_tasks,
                "summary": {
                    "total": ticket.total_tasks_count(),
                    "completed": ticket.completed_tasks_count(),
                    "incomplete": ticket.total_tasks_count() - ticket.completed_tasks_count(),
                    "percentage": ticket.completion_percentage(),
                }
            }))?;
        } else {
            if tasks.is_empty() {
                let filter_msg = if completed_only {
                    " (no completed tasks)"
                } else if incomplete_only {
                    " (no incomplete tasks)"
                } else {
                    ""
                };
                self.output.info(&format!("No tasks found for ticket '{}'{}", ticket.slug, filter_msg));
            } else {
                self.output.info(&format!("Tasks for ticket '{}':", ticket.slug));
                for (i, task) in tasks.iter().enumerate() {
                    let checkbox = if task.completed { "✓" } else { " " };
                    println!("  {}. [{}] {}", i + 1, checkbox, task.title);
                }
                
                if !completed_only && !incomplete_only {
                    println!();
                    self.output.info(&format!(
                        "Progress: {}/{} ({}%)",
                        ticket.completed_tasks_count(),
                        ticket.total_tasks_count(),
                        ticket.completion_percentage()
                    ));
                }
            }
        }
        
        Ok(())
    }
}

/// Result of a task operation
pub struct TaskOperationResult {
    pub modified: bool,
    pub success_message: String,
    pub info_message: Option<String>,
    pub show_progress: bool,
    pub task: Option<Task>,
}

impl TaskOperationResult {
    /// Create a new result for task addition
    pub fn added(ticket_slug: &str, task: Task) -> Self {
        Self {
            modified: true,
            success_message: format!("Added task to ticket '{}'", ticket_slug),
            info_message: Some(format!("Task: {}", task.title)),
            show_progress: true,
            task: Some(task),
        }
    }
    
    /// Create a new result for task completion
    pub fn completed(ticket_slug: &str, task: &Task) -> Self {
        Self {
            modified: true,
            success_message: format!("Completed task in ticket '{}'", ticket_slug),
            info_message: Some(format!("Task: {}", task.title)),
            show_progress: true,
            task: Some(task.clone()),
        }
    }
    
    /// Create a new result for task uncompletion
    pub fn uncompleted(ticket_slug: &str, task: &Task) -> Self {
        Self {
            modified: true,
            success_message: format!("Marked task as incomplete in ticket '{}'", ticket_slug),
            info_message: Some(format!("Task: {}", task.title)),
            show_progress: true,
            task: Some(task.clone()),
        }
    }
    
    /// Create a new result for task removal
    pub fn removed(ticket_slug: &str, task_title: &str) -> Self {
        Self {
            modified: true,
            success_message: format!("Removed task from ticket '{}'", ticket_slug),
            info_message: Some(format!("Task: {}", task_title)),
            show_progress: true,
            task: None,
        }
    }
    
    /// Convert to JSON
    pub fn to_json(&self, ticket: &Ticket) -> serde_json::Value {
        let mut result = json!({
            "status": "success",
            "ticket_id": ticket.id.to_string(),
            "ticket_slug": ticket.slug,
        });
        
        if let Some(ref task) = self.task {
            result["task"] = json!({
                "id": task.id.to_string(),
                "title": task.title,
                "completed": task.completed,
            });
        }
        
        if self.show_progress {
            result["progress"] = json!({
                "completed": ticket.completed_tasks_count(),
                "total": ticket.total_tasks_count(),
                "percentage": ticket.completion_percentage(),
            });
        }
        
        result
    }
}