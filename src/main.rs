//! vibe-ticket - High-performance ticket management system
//!
//! This is the main entry point for the vibe-ticket CLI application.
//! It handles command-line argument parsing and dispatches to the appropriate
//! command handlers.

use clap::Parser;
use std::process;
use vibe_ticket::cli::{
    Cli, Commands, ConfigCommands, OutputFormatter, SpecCommands, TaskCommands, WorktreeCommands,
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
        handle_error(&e, &formatter);
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
    dispatch_command(cli.command, cli.project, formatter)
}

/// Arguments for the new command dispatcher
struct NewCommandArgs<'a> {
    slug: String,
    title: Option<String>,
    description: Option<String>,
    priority: String,
    tags: Option<String>,
    start: bool,
    project: Option<String>,
    formatter: &'a OutputFormatter,
}

/// Options for list command filtering
#[derive(Copy, Clone)]
#[allow(clippy::struct_excessive_bools)]
struct ListFilterOptions {
    reverse: bool,
    archived: bool,
    open: bool,
    include_done: bool,
}

/// Arguments for the list command dispatcher
struct ListCommandArgs<'a> {
    status: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    sort: String,
    limit: Option<usize>,
    since: Option<String>,
    until: Option<String>,
    filter_options: ListFilterOptions,
    project: Option<String>,
    formatter: &'a OutputFormatter,
}

/// Arguments for the edit command dispatcher
struct EditCommandArgs<'a> {
    ticket: Option<String>,
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    status: Option<String>,
    add_tags: Option<String>,
    remove_tags: Option<String>,
    editor: bool,
    project: Option<String>,
    formatter: &'a OutputFormatter,
}

/// Options for search command filtering
#[derive(Copy, Clone)]
#[allow(clippy::struct_excessive_bools)]
struct SearchOptions {
    title: bool,
    description: bool,
    tags: bool,
    regex: bool,
}

fn dispatch_command(
    command: Commands,
    project: Option<String>,
    formatter: &OutputFormatter,
) -> Result<()> {
    match command {
        Commands::Init { name, description, force, claude_md } => 
            handle_init(name.as_deref(), description.as_deref(), force, claude_md, formatter),
        Commands::New { slug, title, description, priority, tags, start } => 
            dispatch_new_command(NewCommandArgs {
                slug, title, description, priority, tags, start, project, formatter,
            }),
        Commands::List { status, priority, assignee, sort, reverse, limit, 
                        archived, open, since, until, include_done } =>
            dispatch_list_command(ListCommandArgs {
                status, priority, assignee, sort, limit, since, until,
                filter_options: ListFilterOptions {
                    reverse,
                    archived,
                    open,
                    include_done,
                },
                project, formatter,
            }),
        Commands::Open { sort, reverse, limit } => 
            dispatch_open_command(&sort, reverse, limit, project.as_deref(), formatter),
        _ => dispatch_main_commands(command, project.as_deref(), formatter),
    }
}

fn dispatch_main_commands(
    command: Commands,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    match command {
        Commands::Start { ticket, branch, branch_name, worktree, no_worktree } =>
            dispatch_start_command(
                ticket, branch, branch_name, worktree, no_worktree,
                project, formatter,
            ),
        Commands::Close { ticket, message, archive, pr } =>
            dispatch_close_command(ticket, message, archive, pr, project, formatter),
        Commands::Check { detailed, stats } =>
            dispatch_check_command(detailed, stats, project, formatter),
        Commands::Edit { ticket, title, description, priority, status, 
                        add_tags, remove_tags, editor } =>
            dispatch_edit_command(EditCommandArgs {
                ticket, title, description, priority, status,
                add_tags, remove_tags, editor, 
                project: project.map(str::to_string), 
                formatter,
            }),
        _ => dispatch_remaining_commands(command, project, formatter),
    }
}

fn dispatch_remaining_commands(
    command: Commands,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    match command {
        Commands::Show {
            ticket,
            tasks,
            history,
            markdown,
        } => dispatch_show_command(&ticket, tasks, history, markdown, project, formatter),
        Commands::Task { command } => handle_task_command(command, project, formatter),
        Commands::Archive { ticket, unarchive } => {
            dispatch_archive_command(&ticket, unarchive, project, formatter)
        },
        Commands::Search {
            query,
            title,
            description,
            tags,
            regex,
        } => dispatch_search_command(&query, SearchOptions { title, description, tags, regex }, project, formatter),
        Commands::Export {
            format,
            output,
            include_archived,
        } => dispatch_export_command(&format, output, include_archived, project, formatter),
        Commands::Import {
            file,
            format,
            skip_validation,
            dry_run,
        } => dispatch_import_command(&file, format.as_deref(), skip_validation, dry_run, project, formatter),
        Commands::Config { command } => dispatch_config_command(command, project, formatter),
        Commands::Spec { command } => dispatch_spec_command(command, project, formatter),
        Commands::Worktree { command } => dispatch_worktree_command(command, formatter),
        #[cfg(feature = "mcp")]
        Commands::Mcp { command } => dispatch_mcp_command(command, project, formatter),
        _ => unreachable!("All commands should be handled"),
    }
}

fn dispatch_new_command(args: NewCommandArgs<'_>) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_new_command;
    let tags_vec = args
        .tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    handle_new_command(
        &args.slug,
        args.title,
        args.description,
        &args.priority,
        tags_vec,
        args.start,
        args.project.as_deref(),
        args.formatter,
    )
}

fn dispatch_list_command(args: ListCommandArgs<'_>) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_list_command;
    handle_list_command(
        args.status,
        args.priority,
        args.assignee,
        &args.sort,
        args.filter_options.reverse,
        args.limit,
        args.filter_options.archived,
        args.filter_options.open,
        args.since,
        args.until,
        args.filter_options.include_done,
        args.project.as_deref(),
        args.formatter,
    )
}

fn dispatch_open_command(
    sort: &str,
    reverse: bool,
    limit: Option<usize>,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_list_command;
    handle_list_command(
        None,
        None,
        None,
        sort,
        reverse,
        limit,
        false,
        true,
        None,
        None,
        false,
        project,
        formatter,
    )
}

fn dispatch_start_command(
    ticket: String,
    branch: bool,
    branch_name: Option<String>,
    worktree: bool,
    no_worktree: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_start_command;
    let use_worktree = if no_worktree { false } else { worktree };
    handle_start_command(
        ticket,
        branch,
        branch_name,
        use_worktree,
        project.map(str::to_string),
        formatter,
    )
}

fn dispatch_close_command(
    ticket: Option<String>,
    message: Option<String>,
    archive: bool,
    pr: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_close_command;
    handle_close_command(ticket, message, archive, pr, project, formatter)
}

fn dispatch_check_command(
    detailed: bool,
    stats: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_check_command;
    handle_check_command(detailed, stats, project, formatter)
}

fn dispatch_edit_command(args: EditCommandArgs<'_>) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_edit_command;
    let add_tags_vec = args
        .add_tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let remove_tags_vec = args
        .remove_tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    handle_edit_command(
        args.ticket,
        args.title,
        args.description,
        args.priority,
        args.status,
        add_tags_vec,
        remove_tags_vec,
        args.editor,
        args.project.as_deref(),
        args.formatter,
    )
}

fn dispatch_show_command(
    ticket: &str,
    tasks: bool,
    history: bool,
    markdown: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_show_command;
    handle_show_command(
        ticket,
        tasks,
        history,
        markdown,
        project,
        formatter,
    )
}

fn dispatch_archive_command(
    ticket: &str,
    unarchive: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_archive_command;
    handle_archive_command(ticket, unarchive, project, formatter)
}

fn dispatch_search_command(
    query: &str,
    options: SearchOptions,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_search_command;
    handle_search_command(
        query,
        options.title,
        options.description,
        options.tags,
        options.regex,
        project,
        formatter,
    )
}

fn dispatch_export_command(
    format: &str,
    output: Option<String>,
    include_archived: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_export_command;
    handle_export_command(
        format,
        output,
        include_archived,
        project,
        formatter,
    )
}

fn dispatch_import_command(
    file: &str,
    format: Option<&str>,
    skip_validation: bool,
    dry_run: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_import_command;
    handle_import_command(
        file,
        format,
        skip_validation,
        dry_run,
        project,
        formatter,
    )
}

fn dispatch_config_command(
    command: ConfigCommands,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use vibe_ticket::cli::handlers::handle_config_command;
    handle_config_command(command, project, formatter)
}

fn dispatch_spec_command(
    command: SpecCommands,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    match command {
        SpecCommands::Init {
            title,
            description,
            ticket,
            tags,
        } => {
            use vibe_ticket::cli::handlers::handle_spec_init;
            handle_spec_init(
                &title,
                description.as_deref(),
                ticket.as_deref(),
                tags.as_deref(),
                project,
                formatter,
            )
        },
        SpecCommands::Requirements {
            spec,
            editor,
            complete: _,
        } => {
            use vibe_ticket::cli::handlers::handle_spec_requirements;
            let spec_id = spec.unwrap_or_default();
            let editor_opt = if editor { Some(String::new()) } else { None };
            handle_spec_requirements(spec_id, editor_opt, project, formatter)
        },
        SpecCommands::Design {
            spec,
            editor,
            complete,
        } => {
            use vibe_ticket::cli::handlers::handle_spec_design;
            handle_spec_design(spec, editor, complete, project, formatter)
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
                project,
                formatter,
            )
        },
        SpecCommands::Status { spec, detailed } => {
            use vibe_ticket::cli::handlers::handle_spec_status;
            handle_spec_status(spec, detailed, project, formatter)
        },
        SpecCommands::List {
            status,
            phase,
            archived,
        } => {
            use vibe_ticket::cli::handlers::handle_spec_list;
            handle_spec_list(status, phase, archived, project, formatter)
        },
        SpecCommands::Show {
            spec,
            all,
            markdown,
        } => {
            use vibe_ticket::cli::handlers::handle_spec_show;
            handle_spec_show(spec, all, markdown, project, formatter)
        },
        SpecCommands::Delete { spec, force } => {
            use vibe_ticket::cli::handlers::handle_spec_delete;
            handle_spec_delete(spec, force, project, formatter)
        },
        SpecCommands::Approve {
            spec,
            phase,
            message,
        } => {
            use vibe_ticket::cli::handlers::handle_spec_approve;
            handle_spec_approve(spec, phase, message, project, formatter)
        },
        SpecCommands::Activate { spec } => {
            use vibe_ticket::cli::handlers::handle_spec_activate;
            handle_spec_activate(spec, project, formatter)
        },
    }
}

fn dispatch_worktree_command(command: WorktreeCommands, formatter: &OutputFormatter) -> Result<()> {
    match command {
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
    }
}

#[cfg(feature = "mcp")]
fn dispatch_mcp_command(
    command: vibe_ticket::cli::McpCommands,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    match command {
        vibe_ticket::cli::McpCommands::Serve { host, port, daemon } => {
            use vibe_ticket::cli::handlers::handle_mcp_serve;
            let config = vibe_ticket::config::Config::load_or_default()?;
            handle_mcp_serve(config, host, port, daemon, project, formatter)
                .map_err(|e| vibe_ticket::error::VibeTicketError::custom(e.to_string()))
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
fn handle_error(error: &vibe_ticket::error::VibeTicketError, formatter: &OutputFormatter) {
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

fn handle_task_command(
    command: TaskCommands,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    match command {
        TaskCommands::Add { title, ticket } => {
            use vibe_ticket::cli::handlers::handle_task_add;
            handle_task_add(title, ticket, project.map(str::to_string), formatter)
        },
        TaskCommands::Complete { task, ticket } => {
            use vibe_ticket::cli::handlers::handle_task_complete;
            handle_task_complete(task, ticket, project.map(str::to_string), formatter)
        },
        TaskCommands::Uncomplete { task, ticket } => {
            use vibe_ticket::cli::handlers::handle_task_uncomplete;
            handle_task_uncomplete(task, ticket, project.map(str::to_string), formatter)
        },
        TaskCommands::List {
            ticket,
            completed,
            incomplete,
        } => {
            use vibe_ticket::cli::handlers::handle_task_list;
            handle_task_list(ticket, completed, incomplete, project.map(str::to_string), formatter)
        },
        TaskCommands::Remove {
            task,
            ticket,
            force,
        } => {
            use vibe_ticket::cli::handlers::handle_task_remove;
            handle_task_remove(task, ticket, force, project.map(str::to_string), formatter)
        },
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
