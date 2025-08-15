//! REST API layer for vibe-ticket
//!
//! This module provides a `RESTful` HTTP API for interacting with the ticket system.
//! It is feature-gated and only compiled when the "api" feature is enabled.
//!
//! # Features
//!
//! The API module is only available when the "api" feature is enabled:
//! ```toml
//! [dependencies]
//! vibe-ticket = { version = "0.1", features = ["api"] }
//! ```
//!
//! # API Design
//!
//! The API follows `RESTful` principles:
//! - Resource-based URLs (`/tickets`, `/projects`, `/users`)
//! - HTTP methods for operations (GET, POST, PUT, DELETE)
//! - JSON request/response bodies
//! - Standard HTTP status codes
//! - Pagination, filtering, and sorting support
//!
//! # Authentication
//!
//! The API supports multiple authentication methods:
//! - API tokens (Bearer authentication)
//! - `OAuth2` integration
//! - Basic authentication (for development)
//!
//! # Example Endpoints
//!
//! ```text
//! GET    /api/v1/tickets          - List all tickets
//! POST   /api/v1/tickets          - Create a new ticket
//! GET    /api/v1/tickets/:id      - Get ticket details
//! PUT    /api/v1/tickets/:id      - Update a ticket
//! DELETE /api/v1/tickets/:id      - Delete a ticket
//! ```
//!
//! # `OpenAPI` Documentation
//!
//! The API automatically generates OpenAPI/Swagger documentation
//! available at `/api/docs` when the server is running.
//!
//! # Example Usage
//!
//! ```ignore
//! #[cfg(feature = "api")]
//! use vibe_ticket::api::start_server;
//! use vibe_ticket::config::Config;
//!
//! #[cfg(feature = "api")]
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::load()?;
//!     start_server(config).await?;
//!     Ok(())
//! }
//! ```

// TODO: Add submodules as they are implemented
