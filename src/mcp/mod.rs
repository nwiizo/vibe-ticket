//! Model Context Protocol (MCP) server implementation for vibe-ticket
//!
//! This module provides MCP server functionality to expose vibe-ticket
//! operations through the Model Context Protocol, enabling AI assistants
//! to interact with the ticket management system.

pub mod auth;
pub mod config;
pub mod error;
pub mod event_bridge;
pub mod handlers;
pub mod server;
pub mod service;

pub use config::McpConfig;
pub use error::{McpError, McpResult};
pub use server::McpServer;
