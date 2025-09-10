//! vibe-ticket - A high-performance ticket management system for developers
//!
//! This crate provides a comprehensive ticket management solution with features including:
//! - Git worktree integration for parallel development
//! - Concurrent access protection with automatic file locking
//! - Spec-driven development with three-phase workflow
//! - Task management within tickets
//! - Multiple export/import formats

// Allow missing error documentation for internal implementations
#![allow(clippy::missing_errors_doc)]
// Allow some pedantic lints that don't improve code quality
#![allow(clippy::option_if_let_else)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unused_self)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::single_match_else)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::map_unwrap_or)]

//! # Concurrent Safety
//!
//! All operations in vibe-ticket are safe for concurrent access. The storage layer
//! automatically handles file locking to prevent data corruption when multiple
//! processes or users access tickets simultaneously. Lock files are created
//! transparently and cleaned up automatically, with built-in retry logic for
//! smooth operation under contention.
//!
//! # Example
//!
//! ```rust,ignore
//! use vibe_ticket::storage::{FileStorage, TicketRepository};
//! use vibe_ticket::core::Ticket;
//!
//! // Initialize storage
//! let storage = FileStorage::new(".vibe-ticket");
//!
//! // Create a ticket (automatically locked during write)
//! let ticket = Ticket::new("fix-bug".to_string(), "Fix login bug".to_string());
//! storage.save(&ticket)?;
//!
//! // Multiple processes can safely access tickets
//! let loaded = storage.load(&ticket.id)?;
//! ```

pub mod cache;
pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod events;
pub mod integration;
pub mod interactive;
pub mod plugins;
pub mod specs;
pub mod storage;
pub mod templates;

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(test)]
pub mod test_utils;

// Re-export commonly used types
pub use error::{Result, VibeTicketError};
