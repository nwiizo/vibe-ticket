//! CLI module for vibe-ticket
//!
//! This module provides the command-line interface implementation for the ticket management system.
//! It handles command parsing, user interaction, and presentation of data to the terminal.
//!
//! # Structure
//!
//! The CLI module is organized as follows:
//! - Command parsing and argument handling
//! - Interactive prompts and user input
//! - Output formatting and display
//! - Terminal UI components
//!
//! # Example
//!
//! ```no_run
//! use vibe_ticket::cli::{Cli, Commands};
//! use clap::Parser;
//!
//! let cli = Cli::parse();
//! match cli.command {
//!     Commands::Init { .. } => {
//!         // Handle init command
//!     }
//!     _ => {}
//! }
//! ```

mod commands;
pub mod handlers;
mod output;
mod utils;

#[cfg(feature = "mcp")]
pub use commands::McpCommands;
pub use commands::{
    AliasCommands, BulkCommands, Cli, Commands, ConfigCommands, FilterCommands, SpecCommands,
    TaskCommands, TimeCommands, WorktreeCommands,
};
pub use output::{OutputFormatter, ProgressBar};
pub use utils::*;
