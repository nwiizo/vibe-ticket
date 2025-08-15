//! MCP event handler for CLI operations

use crate::Result;
use crate::events::{EventHandler, TicketEvent};
use crate::mcp::service::VibeTicketService;
use std::sync::Arc;

/// MCP event handler that processes CLI events
pub struct McpEventHandler {}

impl McpEventHandler {
    /// Create a new MCP event handler
    #[must_use]
    pub fn new(_service: Arc<VibeTicketService>) -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl EventHandler for McpEventHandler {
    async fn handle_event(&self, event: TicketEvent) -> Result<()> {
        match event {
            TicketEvent::Created(ticket) => {
                // Clear any cached ticket lists
                tracing::info!("MCP: Ticket created via CLI: {}", ticket.id);
                // In the future, we could notify connected MCP clients here
            },
            TicketEvent::Updated(ticket) => {
                tracing::info!("MCP: Ticket updated via CLI: {}", ticket.id);
                // Update any internal caches if needed
            },
            TicketEvent::Closed(ticket_id, message) => {
                tracing::info!("MCP: Ticket closed via CLI: {} - {}", ticket_id, message);
            },
            TicketEvent::TaskAdded(ticket_id, task) => {
                tracing::info!(
                    "MCP: Task added via CLI to ticket {}: {}",
                    ticket_id,
                    task.title
                );
            },
            TicketEvent::TaskCompleted(ticket_id, task_id) => {
                tracing::info!(
                    "MCP: Task completed via CLI: {} in ticket {}",
                    task_id,
                    ticket_id
                );
            },
            TicketEvent::TaskRemoved(ticket_id, task_id) => {
                tracing::info!(
                    "MCP: Task removed via CLI: {} from ticket {}",
                    task_id,
                    ticket_id
                );
            },
            TicketEvent::StatusChanged(ticket_id, old_status, new_status) => {
                tracing::info!(
                    "MCP: Ticket {} status changed via CLI: {:?} -> {:?}",
                    ticket_id,
                    old_status,
                    new_status
                );
            },
        }

        Ok(())
    }
}
