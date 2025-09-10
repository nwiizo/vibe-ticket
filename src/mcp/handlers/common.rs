use crate::core::Ticket;
use crate::error::Result;
use crate::storage::FileStorage;
use serde_json::Value;
use std::path::Path;

#[cfg(feature = "mcp")]
pub use rmcp::model::{CallToolResult, Tool};

#[cfg(not(feature = "mcp"))]
pub struct CallToolResult {
    pub content: Vec<String>,
}

#[cfg(not(feature = "mcp"))]
pub struct TextContent {
    pub type_: String,
    pub text: String,
}

/// Common MCP handler context
pub struct McpContext {
    pub storage: FileStorage,
}

impl McpContext {
    /// Create a new MCP context
    #[must_use]
    pub fn new(project_dir: &Path) -> Self {
        let vibe_ticket_dir = project_dir.join(".vibe-ticket");
        let storage = FileStorage::new(&vibe_ticket_dir);
        Self { storage }
    }

    /// Create success result
    pub fn success_result(message: impl Into<String>) -> CallToolResult {
        #[cfg(feature = "mcp")]
        {
            CallToolResult {
                content: vec![rmcp::model::Content::text(message.into())],
                is_error: Some(false),
            }
        }
        #[cfg(not(feature = "mcp"))]
        {
            CallToolResult {
                content: vec![message.into()],
            }
        }
    }

    /// Create error result
    pub fn error_result(error: impl std::fmt::Display) -> CallToolResult {
        #[cfg(feature = "mcp")]
        {
            CallToolResult {
                content: vec![rmcp::model::Content::text(format!("Error: {error}"))],
                is_error: Some(true),
            }
        }
        #[cfg(not(feature = "mcp"))]
        {
            CallToolResult {
                content: vec![format!("Error: {}", error)],
            }
        }
    }

    /// Create JSON result
    #[must_use]
    pub fn json_result(value: &Value) -> CallToolResult {
        let json_str = serde_json::to_string_pretty(value)
            .unwrap_or_else(|e| format!("Error serializing JSON: {e}"));

        #[cfg(feature = "mcp")]
        {
            CallToolResult {
                content: vec![rmcp::model::Content::text(json_str)],
                is_error: Some(false),
            }
        }
        #[cfg(not(feature = "mcp"))]
        {
            CallToolResult {
                content: vec![json_str],
            }
        }
    }
}

/// Common trait for MCP search/export operations
pub trait McpDataOperation {
    /// Get the operation name
    fn operation_name(&self) -> &str;

    /// Process tickets with the operation
    fn process_tickets(&self, tickets: Vec<Ticket>) -> Result<Value>;

    /// Execute the operation
    fn execute(&self, ctx: &McpContext, filter: Option<TicketFilter>) -> CallToolResult {
        // Load tickets
        let tickets = match ctx.storage.load_all_tickets() {
            Ok(t) => t,
            Err(e) => return McpContext::error_result(e),
        };

        // Apply filter if provided
        let filtered = if let Some(f) = filter {
            f.apply(tickets)
        } else {
            tickets
        };

        // Process tickets
        match self.process_tickets(filtered) {
            Ok(result) => McpContext::json_result(&result),
            Err(e) => McpContext::error_result(e),
        }
    }
}

/// Common ticket filter
pub struct TicketFilter {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl TicketFilter {
    /// Apply filter to tickets
    #[must_use]
    pub fn apply(self, tickets: Vec<Ticket>) -> Vec<Ticket> {
        tickets
            .into_iter()
            .filter(|t| {
                // Filter by status
                if let Some(ref s) = self.status {
                    if t.status.to_string().to_lowercase() != s.to_lowercase() {
                        return false;
                    }
                }

                // Filter by priority
                if let Some(ref p) = self.priority {
                    if t.priority.to_string().to_lowercase() != p.to_lowercase() {
                        return false;
                    }
                }

                // Filter by assignee
                if let Some(ref a) = self.assignee {
                    if t.assignee.as_ref() != Some(a) {
                        return false;
                    }
                }

                // Filter by tags
                if let Some(ref tags) = self.tags {
                    if !tags.iter().all(|tag| t.tags.contains(tag)) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }
}
