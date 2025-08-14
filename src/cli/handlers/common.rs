use crate::cli::utils::find_project_root;
use crate::core::Ticket;
use crate::error::{Result, VibeTicketError};
use crate::storage::FileStorage;
use uuid::Uuid;

/// Common context for all handler operations
pub struct HandlerContext {
    pub storage: FileStorage,
}

impl HandlerContext {
    /// Create a new handler context
    pub fn new(project_dir: Option<&str>) -> Result<Self> {
        let project_root = find_project_root(project_dir)?;
        let vibe_ticket_dir = project_root.join(".vibe-ticket");
        let storage = FileStorage::new(&vibe_ticket_dir);
        
        Ok(Self { storage })
    }
    
    /// Get storage reference
    pub fn storage(&self) -> &FileStorage {
        &self.storage
    }
}

/// Common trait for ticket operations
pub trait TicketOperation {
    /// Load a ticket by reference (ID, slug, or active)
    fn load_ticket(&self, ticket_ref: Option<&str>) -> Result<Ticket>;
    
    /// Save a ticket
    fn save_ticket(&self, ticket: &Ticket) -> Result<()>;
    
    /// Resolve ticket reference to ID
    fn resolve_ticket_ref(&self, ticket_ref: &str) -> Result<Uuid>;
    
    /// Get active ticket ID
    fn get_active_ticket_id(&self) -> Result<Uuid>;
}

impl TicketOperation for HandlerContext {
    fn load_ticket(&self, ticket_ref: Option<&str>) -> Result<Ticket> {
        use crate::core::TicketId;
        
        let ticket_id = if let Some(ref_str) = ticket_ref {
            self.resolve_ticket_ref(ref_str)?
        } else {
            self.get_active_ticket_id()?
        };
        
        self.storage.load_ticket(&TicketId::from_uuid(ticket_id))
    }
    
    fn save_ticket(&self, ticket: &Ticket) -> Result<()> {
        self.storage.save_ticket(ticket)
    }
    
    fn resolve_ticket_ref(&self, ticket_ref: &str) -> Result<Uuid> {
        // Try to parse as UUID first
        if let Ok(id) = Uuid::parse_str(ticket_ref) {
            return Ok(id);
        }
        
        // Try to find by slug
        let tickets = self.storage.load_all_tickets()?;
        for ticket in tickets {
            if ticket.slug == ticket_ref {
                return Ok(*ticket.id.as_uuid());
            }
        }
        
        Err(VibeTicketError::TicketNotFound { 
            id: ticket_ref.to_string() 
        })
    }
    
    fn get_active_ticket_id(&self) -> Result<Uuid> {
        let ticket_id = self.storage
            .get_active_ticket()?
            .ok_or(VibeTicketError::NoActiveTicket)?;
        Ok(*ticket_id.as_uuid())
    }
}

/// Helper function to resolve ticket reference using storage
pub fn resolve_ticket_ref(storage: &FileStorage, ticket_ref: &str) -> Result<Uuid> {
    // Try to parse as UUID first
    if let Ok(id) = Uuid::parse_str(ticket_ref) {
        return Ok(id);
    }
    
    // Try to find by slug
    let tickets = storage.load_all_tickets()?;
    for ticket in tickets {
        if ticket.slug == ticket_ref {
            return Ok(*ticket.id.as_uuid());
        }
    }
    
    Err(VibeTicketError::TicketNotFound { 
        id: ticket_ref.to_string() 
    })
}