//! Core business logic for vibe-ticket
//!
//! This module contains the fundamental business logic and domain models for the ticket
//! management system. It is independent of any specific storage or presentation layer.
//!
//! # Architecture
//!
//! The core module follows Domain-Driven Design principles:
//! - Domain entities (Ticket, Project, User, etc.)
//! - Value objects (`TicketId`, Status, Priority, etc.)
//! - Domain services and business rules
//! - Repository traits (interfaces for storage)
//!
//! # Example
//!
//! ```no_run
//! use vibe_ticket::core::{Ticket, Status, Priority};
//!
//! let mut ticket = Ticket::new("fix-login-bug", "Fix login bug");
//! ticket.description = "Users cannot login with special characters".to_string();
//! ticket.priority = Priority::High;
//! ticket.start();
//! ```
//!
//! # Design Principles
//!
//! - The core module should have no dependencies on external crates for business logic
//! - All I/O operations should be abstracted through traits
//! - Business rules should be enforced at this layer

mod builders;
mod id;
mod priority;
mod status;
mod task;
mod ticket;

pub use builders::{TaskBuilder, TicketBuilder};
pub use id::{TaskId, TicketId};
pub use priority::Priority;
pub use status::Status;
pub use task::Task;
pub use ticket::Ticket;
