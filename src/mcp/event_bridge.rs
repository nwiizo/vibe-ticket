//! Event bridge that connects integration events to MCP handlers

use crate::events::{EventHandler, TicketEvent};
use crate::integration::{integration, IntegrationEvent};
use tokio::sync::broadcast;
use tracing::{error, info};

/// Start the event bridge that forwards integration events to MCP handlers
pub async fn start_event_bridge<H: EventHandler + Send + Sync + 'static>(handler: H) {
    // Get the integration service
    let Some(integration_service) = integration() else {
        error!("Integration service not initialized");
        return;
    };

    // Subscribe to integration events
    let mut receiver = integration_service.subscribe();

    // Spawn a task to handle events
    tokio::spawn(async move {
        info!("MCP event bridge started");
        
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    // Convert IntegrationEvent to TicketEvent
                    let ticket_event = match event {
                        IntegrationEvent::TicketCreated { ticket } => {
                            TicketEvent::Created(ticket)
                        }
                        IntegrationEvent::TicketUpdated { ticket } => {
                            TicketEvent::Updated(ticket)
                        }
                        IntegrationEvent::TicketClosed { ticket_id, message } => {
                            TicketEvent::Closed(ticket_id, message)
                        }
                        IntegrationEvent::StatusChanged {
                            ticket_id,
                            old_status,
                            new_status,
                        } => {
                            TicketEvent::StatusChanged(ticket_id, old_status, new_status)
                        }
                    };

                    // Forward to MCP handler
                    if let Err(e) = handler.handle_event(ticket_event).await {
                        error!("Error handling event in MCP: {}", e);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    error!("MCP event bridge lagged by {} messages", count);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("Integration event channel closed, stopping MCP event bridge");
                    break;
                }
            }
        }
    });
}