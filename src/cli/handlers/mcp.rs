//! MCP server command handler

use crate::cli::output::OutputFormatter;
use crate::config::Config;
use crate::mcp::{McpConfig, McpServer};
use crate::storage::FileStorage;
use std::path::PathBuf;

pub fn handle_mcp_serve(
    _config: Config,
    host: Option<String>,
    port: Option<u16>,
    daemon: bool,
    project_path: Option<&str>,
    formatter: &OutputFormatter,
) -> anyhow::Result<()> {
    use tracing::error;

    // Create MCP configuration
    let mut mcp_config = McpConfig::default();

    if let Some(host) = host {
        mcp_config.server.host = host;
    }

    if let Some(port) = port {
        mcp_config.server.port = port;
    }

    // Get storage path
    let storage_path = project_path.map_or_else(
        || PathBuf::from(".vibe-ticket"),
        |path| PathBuf::from(path).join(".vibe-ticket")
    );

    mcp_config.storage_path = storage_path.clone();

    // Create storage
    let storage = FileStorage::new(storage_path);

    // Create and start server
    let server = McpServer::new(mcp_config.clone(), storage);

    if daemon {
        formatter.info("Starting MCP server in daemon mode...");
        // TODO: Implement daemon mode
        return Err(anyhow::anyhow!("Daemon mode not yet implemented"));
    }

    formatter.info(&format!(
        "Starting MCP server on {}:{}",
        mcp_config.server.host, mcp_config.server.port
    ));

    // Run server
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        if let Err(e) = Box::pin(server.start()).await {
            error!("MCP server error: {}", e);
            return Err(anyhow::anyhow!("MCP server error: {}", e));
        }
        Ok(())
    })
}
