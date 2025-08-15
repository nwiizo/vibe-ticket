//! vibe-ticket - High-performance ticket management system
//!
//! This is the main entry point for the vibe-ticket CLI application.
//! It handles command-line argument parsing and dispatches to the appropriate
//! command handlers.

use clap::Parser;
use std::process;
use vibe_ticket::cli::{
    Cli, Commands, OutputFormatter, SpecCommands, TaskCommands, WorktreeCommands,
    handlers::handle_init,
};
use vibe_ticket::error::Result;

/// Main entry point for the vibe-ticket CLI
///
/// Parses command-line arguments and executes the requested command.
/// Handles errors gracefully and provides helpful error messages to users.
fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Configure output formatter based on flags
    let formatter = OutputFormatter::new(cli.json, cli.no_color);

    // Execute the command and handle errors
    if let Err(e) = run(cli, &formatter) {
        handle_error(e, &formatter);
        process::exit(1);
    }
}

/// Run the CLI application with the parsed arguments
///
/// This function dispatches to the appropriate command handler based on
/// the parsed command. Each handler is responsible for its own business logic.
///
/// # Arguments
///
/// * `cli` - Parsed CLI arguments
/// * `formatter` - Output formatter for displaying results
///
/// # Errors
///
/// Returns any error that occurs during command execution
fn run(cli: Cli, formatter: &OutputFormatter) -> Result<()> {
    // Set up logging if verbose mode is enabled
    if cli.verbose {
        tracing_subscriber::fmt().with_env_filter("debug").init();
    }

    // Change to project directory if specified
    if let Some(project_path) = &cli.project {
        std::env::set_current_dir(project_path).map_err(vibe_ticket::error::VibeTicketError::Io)?;
    }

    // Dispatch to command handler
    match cli.command {
        Commands::Init {
            name,
            description,
            force,
            claude_md,
        } => handle_init(
            name.as_deref(),
            description.as_deref(),
            force,
            claude_md,
            formatter,
        ),

        Commands::New {
            slug,
            title,
            description,
            priority,
            tags,
            start,
        } => {
            use vibe_ticket::cli::handlers::handle_new_command;
            handle_new_command(
                &slug,
                title,
                description,
                &priority,
                tags,
                start,
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::List {
            status,
            priority,
            assignee,
            sort,
            reverse,
            limit,
            archived,
            open,
            since,
            until,
            include_done,
        } => {
            use vibe_ticket::cli::handlers::handle_list_command;
            handle_list_command(
                status,
                priority,
                assignee,
                &sort,
                reverse,
                limit,
                archived,
                open,
                since,
                until,
                include_done,
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::Open {
            sort,
            reverse,
            limit,
        } => {
            use vibe_ticket::cli::handlers::handle_list_command;
            // Call list handler with open filter set to true
            handle_list_command(
                None, // status
                None, // priority
                None, // assignee
                &sort,
                reverse,
                limit,
                false, // archived
                true,  // open
                None,  // since
                None,  // until
                false, // include_done
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::Start {
            ticket,
            branch,
            branch_name,
            worktree,
            no_worktree,
        } => {
            use vibe_ticket::cli::handlers::handle_start_command;
            // If no_worktree is true, override worktree to false
            let use_worktree = if no_worktree { false } else { worktree };
            handle_start_command(
                ticket,
                branch,
                branch_name,
                use_worktree,
                cli.project,
                formatter,
            )
        },

        Commands::Close {
            ticket,
            message,
            archive,
            pr,
        } => {
            use vibe_ticket::cli::handlers::handle_close_command;
            handle_close_command(
                ticket,
                message,
                archive,
                pr,
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::Check { detailed, stats } => {
            use vibe_ticket::cli::handlers::handle_check_command;
            handle_check_command(detailed, stats, cli.project.as_deref(), formatter)
        },

        Commands::Edit {
            ticket,
            title,
            description,
            priority,
            status,
            add_tags,
            remove_tags,
            editor,
        } => {
            use vibe_ticket::cli::handlers::handle_edit_command;
            handle_edit_command(
                ticket,
                title,
                description,
                priority,
                status,
                add_tags,
                remove_tags,
                editor,
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::Show {
            ticket,
            tasks,
            history,
            markdown,
        } => {
            use vibe_ticket::cli::handlers::handle_show_command;
            handle_show_command(
                &ticket,
                tasks,
                history,
                markdown,
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::Task { command } => match command {
            TaskCommands::Add { title, ticket } => {
                use vibe_ticket::cli::handlers::handle_task_add;
                handle_task_add(title, ticket, cli.project, formatter)
            },
            TaskCommands::Complete { task, ticket } => {
                use vibe_ticket::cli::handlers::handle_task_complete;
                handle_task_complete(task, ticket, cli.project, formatter)
            },
            TaskCommands::Uncomplete { task, ticket } => {
                use vibe_ticket::cli::handlers::handle_task_uncomplete;
                handle_task_uncomplete(task, ticket, cli.project, formatter)
            },
            TaskCommands::List {
                ticket,
                completed,
                incomplete,
            } => {
                use vibe_ticket::cli::handlers::handle_task_list;
                handle_task_list(ticket, completed, incomplete, cli.project, formatter)
            },
            TaskCommands::Remove {
                task,
                ticket,
                force,
            } => {
                use vibe_ticket::cli::handlers::handle_task_remove;
                handle_task_remove(task, ticket, force, cli.project, formatter)
            },
        },

        Commands::Archive { ticket, unarchive } => {
            use vibe_ticket::cli::handlers::handle_archive_command;
            handle_archive_command(&ticket, unarchive, cli.project.as_deref(), formatter)
        },

        Commands::Search {
            query,
            title,
            description,
            tags,
            regex,
        } => {
            use vibe_ticket::cli::handlers::handle_search_command;
            handle_search_command(
                &query,
                title,
                description,
                tags,
                regex,
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::Export {
            format,
            output,
            include_archived,
        } => {
            use vibe_ticket::cli::handlers::handle_export_command;
            handle_export_command(
                &format,
                output,
                include_archived,
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::Import {
            file,
            format,
            skip_validation,
            dry_run,
        } => {
            use vibe_ticket::cli::handlers::handle_import_command;
            handle_import_command(
                &file,
                format.as_deref(),
                skip_validation,
                dry_run,
                cli.project.as_deref(),
                formatter,
            )
        },

        Commands::Config { command } => {
            use vibe_ticket::cli::handlers::handle_config_command;
            handle_config_command(command, cli.project.as_deref(), formatter)
        },

        Commands::Spec { command } => match command {
            SpecCommands::Init {
                title,
                description,
                ticket,
                tags,
            } => {
                use vibe_ticket::cli::handlers::handle_spec_init;
                handle_spec_init(title, description, ticket, tags, cli.project, formatter)
            },
            SpecCommands::Requirements {
                spec,
                editor,
                complete: _,
            } => {
                use vibe_ticket::cli::handlers::handle_spec_requirements;
                // The function signature has changed after refactoring
                // Need to adjust the call to match the new signature
                let spec_id = spec.unwrap_or_default();
                let editor_opt = if editor { Some("".to_string()) } else { None };
                handle_spec_requirements(spec_id, editor_opt, cli.project, formatter)
            },
            SpecCommands::Design {
                spec,
                editor,
                complete,
            } => {
                use vibe_ticket::cli::handlers::handle_spec_design;
                handle_spec_design(spec, editor, complete, cli.project, formatter)
            },
            SpecCommands::Tasks {
                spec,
                editor,
                complete,
                export_tickets,
            } => {
                use vibe_ticket::cli::handlers::handle_spec_tasks;
                handle_spec_tasks(
                    spec,
                    editor,
                    complete,
                    export_tickets,
                    cli.project,
                    formatter,
                )
            },
            SpecCommands::Status { spec, detailed } => {
                use vibe_ticket::cli::handlers::handle_spec_status;
                handle_spec_status(spec, detailed, cli.project, formatter)
            },
            SpecCommands::List {
                status,
                phase,
                archived,
            } => {
                use vibe_ticket::cli::handlers::handle_spec_list;
                handle_spec_list(status, phase, archived, cli.project, formatter)
            },
            SpecCommands::Show {
                spec,
                all,
                markdown,
            } => {
                use vibe_ticket::cli::handlers::handle_spec_show;
                handle_spec_show(spec, all, markdown, cli.project, formatter)
            },
            SpecCommands::Delete { spec, force } => {
                use vibe_ticket::cli::handlers::handle_spec_delete;
                handle_spec_delete(spec, force, cli.project, formatter)
            },
            SpecCommands::Approve {
                spec,
                phase,
                message,
            } => {
                use vibe_ticket::cli::handlers::handle_spec_approve;
                handle_spec_approve(spec, phase, message, cli.project, formatter)
            },
            SpecCommands::Activate { spec } => {
                use vibe_ticket::cli::handlers::handle_spec_activate;
                handle_spec_activate(spec, cli.project, formatter)
            },
        },
        Commands::Worktree { command } => match command {
            WorktreeCommands::List {
                all,
                status,
                verbose,
            } => {
                use vibe_ticket::cli::handlers::handle_worktree_list;
                handle_worktree_list(all, status, verbose, formatter)
            },
            WorktreeCommands::Remove {
                worktree,
                force,
                keep_branch,
            } => {
                use vibe_ticket::cli::handlers::handle_worktree_remove;
                handle_worktree_remove(&worktree, force, keep_branch, formatter)
            },
            WorktreeCommands::Prune {
                force,
                dry_run,
                remove_branches,
            } => {
                use vibe_ticket::cli::handlers::handle_worktree_prune;
                handle_worktree_prune(force, dry_run, remove_branches, formatter)
            },
        },
        #[cfg(feature = "mcp")]
        Commands::Mcp { command } => match command {
            vibe_ticket::cli::McpCommands::Serve { host, port, daemon } => {
                use vibe_ticket::cli::handlers::handle_mcp_serve;
                let config = vibe_ticket::config::Config::load_or_default()?;
                handle_mcp_serve(
                    config,
                    host,
                    port,
                    daemon,
                    cli.project.as_deref(),
                    formatter,
                )
                .map_err(|e| vibe_ticket::error::VibeTicketError::custom(e.to_string()))
            },
        },
    }
}

/// Handle errors and display them to the user
///
/// This function formats errors in a user-friendly way, including:
/// - The main error message
/// - Any suggestions for fixing the error
/// - Additional context in verbose mode
///
/// # Arguments
///
/// * `error` - The error to handle
/// * `formatter` - Output formatter for displaying the error
fn handle_error(error: vibe_ticket::error::VibeTicketError, formatter: &OutputFormatter) {
    // Display the main error message
    formatter.error(&error.user_message());

    // Display suggestions if available
    let suggestions = error.suggestions();
    if !suggestions.is_empty() {
        formatter.info("\nSuggestions:");
        for suggestion in &suggestions {
            formatter.info(&format!("  • {suggestion}"));
        }
    }

    // In JSON mode, output error as JSON
    if formatter.is_json() {
        let _ = formatter.json(&serde_json::json!({
            "status": "error",
            "error": error.to_string(),
            "error_type": format!("{:?}", error),
            "suggestions": suggestions,
            "recoverable": error.is_recoverable(),
            "is_config_error": error.is_config_error(),
        }));
    }

    // In verbose mode, show the full error chain
    if tracing::enabled!(tracing::Level::DEBUG) {
        eprintln!("\nDebug information:");
        eprintln!("{error:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test that the CLI can be parsed with various commands
        let _cli = Cli::parse_from(["vibe-ticket", "init"]);
        let _cli = Cli::parse_from(["vibe-ticket", "list"]);
        let _cli = Cli::parse_from(["vibe-ticket", "new", "test-ticket"]);
    }
}
