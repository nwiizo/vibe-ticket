//\! MCP-specific error types and error handling

use thiserror::Error;

/// MCP-specific error type
#[derive(Debug, Error)]
pub enum McpError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Invalid parameters provided to a tool
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    /// Storage operation failed
    #[error("Storage error: {0}")]
    StorageError(#[from] crate::error::VibeTicketError),

    /// MCP protocol error
    #[error("MCP protocol error: {0}")]
    ProtocolError(String),

    /// Server configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Generic server error
    #[error("Server error: {0}")]
    ServerError(String),
}

/// Result type alias for MCP operations
pub type McpResult<T> = Result<T, McpError>;

impl From<rmcp::service::ServerInitializeError> for McpError {
    fn from(err: rmcp::service::ServerInitializeError) -> Self {
        Self::ProtocolError(err.to_string())
    }
}

impl From<tokio::task::JoinError> for McpError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::ServerError(err.to_string())
    }
}
