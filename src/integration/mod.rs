//! CLI-MCP integration module

use crate::core::{Status, Ticket, TicketId};
use crate::storage::FileStorage;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Event types for CLI-MCP communication
#[derive(Debug, Clone)]
pub enum IntegrationEvent {
    TicketCreated {
        ticket: Ticket,
    },
    TicketUpdated {
        ticket: Ticket,
    },
    TicketClosed {
        ticket_id: TicketId,
        message: String,
    },
    StatusChanged {
        ticket_id: TicketId,
        old_status: Status,
        new_status: Status,
    },
}

/// Integration service that bridges CLI and MCP
pub struct IntegrationService {
    event_sender: broadcast::Sender<IntegrationEvent>,
}

impl std::fmt::Debug for IntegrationService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntegrationService")
            .field("storage", &"Arc<FileStorage>")
            .field("event_sender", &"broadcast::Sender<IntegrationEvent>")
            .finish()
    }
}

impl IntegrationService {
    /// Create a new integration service
    pub fn new(_storage: Arc<FileStorage>) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self { event_sender }
    }

    /// Get an event receiver
    pub fn subscribe(&self) -> broadcast::Receiver<IntegrationEvent> {
        self.event_sender.subscribe()
    }

    /// Notify about a ticket creation
    pub fn notify_ticket_created(&self, ticket: &Ticket) {
        let _ = self.event_sender.send(IntegrationEvent::TicketCreated {
            ticket: ticket.clone(),
        });
        tracing::info!("Integration: Ticket created - {}", ticket.slug);
    }

    /// Notify about a ticket update
    pub fn notify_ticket_updated(&self, ticket: &Ticket) {
        let _ = self.event_sender.send(IntegrationEvent::TicketUpdated {
            ticket: ticket.clone(),
        });
        tracing::info!("Integration: Ticket updated - {}", ticket.slug);
    }

    /// Notify about a ticket closure
    pub fn notify_ticket_closed(&self, ticket_id: &TicketId, message: String) {
        let _ = self.event_sender.send(IntegrationEvent::TicketClosed {
            ticket_id: ticket_id.clone(),
            message,
        });
        tracing::info!("Integration: Ticket closed - {}", ticket_id.short());
    }

    /// Notify about a status change
    pub fn notify_status_changed(
        &self,
        ticket_id: &TicketId,
        old_status: Status,
        new_status: Status,
    ) {
        let _ = self.event_sender.send(IntegrationEvent::StatusChanged {
            ticket_id: ticket_id.clone(),
            old_status,
            new_status,
        });
        tracing::info!(
            "Integration: Status changed - {} from {:?} to {:?}",
            ticket_id.short(),
            old_status,
            new_status
        );
    }
}

/// Global integration service instance
#[cfg(not(test))]
static INTEGRATION: once_cell::sync::OnceCell<Arc<IntegrationService>> =
    once_cell::sync::OnceCell::new();

/// Global integration service instance for tests (allows replacement)
#[cfg(test)]
static INTEGRATION: std::sync::RwLock<Option<Arc<IntegrationService>>> = 
    std::sync::RwLock::new(None);

/// Initialize the integration service
pub fn init_integration(storage: Arc<FileStorage>) {
    let service = Arc::new(IntegrationService::new(storage));
    
    #[cfg(not(test))]
    {
        INTEGRATION
            .set(service)
            .expect("Integration already initialized");
    }
    
    #[cfg(test)]
    {
        let mut integration = INTEGRATION.write().unwrap();
        *integration = Some(service);
    }
}

/// Get the integration service
#[cfg(not(test))]
pub fn integration() -> Option<&'static Arc<IntegrationService>> {
    INTEGRATION.get()
}

/// Get the integration service (test version)
#[cfg(test)]
pub fn integration() -> Option<Arc<IntegrationService>> {
    let integration = INTEGRATION.read().unwrap();
    integration.clone()
}

/// Helper function to notify about ticket creation
pub fn notify_ticket_created(ticket: &Ticket) {
    if let Some(integration) = integration() {
        integration.notify_ticket_created(ticket);
    }
}

/// Helper function to notify about ticket update
pub fn notify_ticket_updated(ticket: &Ticket) {
    if let Some(integration) = integration() {
        integration.notify_ticket_updated(ticket);
    }
}

/// Helper function to notify about ticket closure
pub fn notify_ticket_closed(ticket_id: &TicketId, message: String) {
    if let Some(integration) = integration() {
        integration.notify_ticket_closed(ticket_id, message);
    }
}

/// Helper function to notify about status change
pub fn notify_status_changed(ticket_id: &TicketId, old_status: Status, new_status: Status) {
    if let Some(integration) = integration() {
        integration.notify_status_changed(ticket_id, old_status, new_status);
    }
}
