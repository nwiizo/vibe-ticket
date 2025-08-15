//! Authentication middleware for MCP server

use crate::mcp::{config::AuthConfig, error::McpError};
use std::future::Future;
use std::pin::Pin;

/// Authentication middleware
pub struct AuthMiddleware {
    config: AuthConfig,
}

impl AuthMiddleware {
    /// Create new authentication middleware
    #[must_use]
    pub const fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    /// Authenticate a request
    #[must_use]
    pub fn authenticate(
        &self,
        _api_key: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), McpError>> + Send + '_>> {
        Box::pin(async move {
            if !self.config.enabled {
                return Ok(());
            }

            // TODO: Implement actual authentication
            Ok(())
        })
    }
}
