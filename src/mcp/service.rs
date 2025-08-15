//! MCP service implementation for vibe-ticket

use crate::storage::FileStorage;
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    model::{ServerCapabilities, ServerInfo, Tool},
    service::RequestContext,
};
use serde_json::Value;
use std::borrow::Cow;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

/// MCP service implementation
#[derive(Clone)]
pub struct VibeTicketService {
    pub storage: Arc<FileStorage>,
    pub project_root: PathBuf,
}

impl VibeTicketService {
    /// Create a new service instance
    #[must_use]
    pub fn new(storage: FileStorage, project_root: PathBuf) -> Self {
        Self {
            storage: Arc::new(storage),
            project_root,
        }
    }

    /// Get all available tools
    #[must_use]
    pub fn get_tools() -> Vec<Tool> {
        use crate::mcp::handlers;
        let mut tools = Vec::new();

        // Ticket operations
        tools.extend(handlers::tickets::register_tools());
        tools.extend(handlers::tasks::register_tools());
        tools.extend(handlers::worktree::register_tools());
        tools.extend(handlers::search::register_tools());
        tools.extend(handlers::config::register_tools());
        tools.extend(handlers::spec::register_tools());

        tools
    }
}

// Implement ServerHandler trait for MCP protocol
#[allow(refining_impl_trait_reachable)]
impl ServerHandler for VibeTicketService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "vibe-ticket MCP server provides comprehensive ticket management capabilities. \
                 Use the available tools to create, manage, and track tickets, tasks, and worktrees."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _pagination: Option<rmcp::model::PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListToolsResult {
            tools: Self::get_tools(),
            next_cursor: None,
        })
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<rmcp::model::CallToolResult, rmcp::ErrorData>>
                + Send
                + 'static,
        >,
    > {
        let service = self.clone();
        let name = request.name.clone();
        let arguments = Value::Object(request.arguments.unwrap_or_default());

        Box::pin(async move {
            let result = match name.as_ref() {
                // Ticket operations
                "vibe-ticket_new" => crate::mcp::handlers::tickets::handle_new(&service, arguments),
                "vibe-ticket_list" => {
                    crate::mcp::handlers::tickets::handle_list(&service, arguments)
                },
                "vibe-ticket_show" => {
                    crate::mcp::handlers::tickets::handle_show(&service, arguments)                },
                "vibe-ticket_edit" => {
                    crate::mcp::handlers::tickets::handle_edit(&service, arguments)                },
                "vibe-ticket_close" => {
                    crate::mcp::handlers::tickets::handle_close(&service, arguments)                },
                "vibe-ticket_start" => {
                    crate::mcp::handlers::tickets::handle_start(&service, arguments)                },
                "vibe-ticket_check" => {
                    crate::mcp::handlers::tickets::handle_check(&service, arguments)
                },

                // Task operations
                "vibe-ticket_task_add" => {
                    crate::mcp::handlers::tasks::handle_add(&service, arguments)                },
                "vibe-ticket_task_complete" => {
                    crate::mcp::handlers::tasks::handle_complete(&service, arguments)                },
                "vibe-ticket_task_list" => {
                    crate::mcp::handlers::tasks::handle_list(&service, arguments)                },
                "vibe-ticket_task_remove" => {
                    crate::mcp::handlers::tasks::handle_remove(&service, arguments)                },

                // Worktree operations
                "vibe-ticket_worktree_list" => {
                    crate::mcp::handlers::worktree::handle_list(&service, arguments)
                },
                "vibe-ticket_worktree_remove" => {
                    crate::mcp::handlers::worktree::handle_remove(&service, arguments)                },
                "vibe-ticket_worktree_prune" => {
                    crate::mcp::handlers::worktree::handle_prune(&service, arguments)
                },

                // Search and export
                "vibe-ticket_search" => {
                    crate::mcp::handlers::search::handle_search(&service, arguments)
                },
                "vibe-ticket_export" => {
                    crate::mcp::handlers::search::handle_export(&service, arguments)                },
                "vibe-ticket_import" => {
                    crate::mcp::handlers::search::handle_import(&service, arguments)
                },

                // Config operations
                "vibe-ticket_config_show" => {
                    crate::mcp::handlers::config::handle_show(&service, arguments)
                },
                "vibe-ticket_config_set" => {
                    crate::mcp::handlers::config::handle_set(&service, arguments)
                },

                // Spec operations
                "vibe-ticket_spec_add" => {
                    crate::mcp::handlers::spec::handle_add(&service, arguments)                },
                "vibe-ticket_spec_update" => {
                    crate::mcp::handlers::spec::handle_update(&service, arguments)                },
                "vibe-ticket_spec_check" => {
                    crate::mcp::handlers::spec::handle_check(&service, arguments)                },

                _ => Err(format!("Unknown tool: {name}")),
            };

            match result {
                Ok(content) => Ok(rmcp::model::CallToolResult {
                    content: vec![rmcp::model::Content::text(
                        serde_json::to_string_pretty(&content)
                            .unwrap_or_else(|_| content.to_string()),
                    )],
                    is_error: None,
                }),
                Err(e) => Err(ErrorData {
                    code: rmcp::model::ErrorCode(-32603), // Internal error code
                    message: Cow::Borrowed("Internal error"),
                    data: Some(serde_json::json!({ "error": e })),
                }),
            }
        })
    }
}
