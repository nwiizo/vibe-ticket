use clap::{Parser, Subcommand};

/// vibe-ticket: A high-performance ticket management system for Vide Coding
#[derive(Parser, Debug)]
#[command(name = "vibe-ticket")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Use JSON output format
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Disable color output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Path to the project directory
    #[arg(short, long, global = true)]
    pub project: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new vibe-ticket project
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,

        /// Project description
        #[arg(short, long)]
        description: Option<String>,

        /// Force initialization even if already initialized
        #[arg(short, long)]
        force: bool,

        /// Generate CLAUDE.md for AI assistance
        #[arg(long = "claude-md", alias = "claude")]
        claude_md: bool,
    },

    /// Create a new ticket
    New {
        /// Ticket slug (e.g., fix-login-bug)
        slug: String,

        /// Ticket title
        #[arg(short, long)]
        title: Option<String>,

        /// Ticket description
        #[arg(short, long)]
        description: Option<String>,

        /// Priority (low, medium, high, critical)
        #[arg(long, default_value = "medium")]
        priority: String,

        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Start working on the ticket immediately
        #[arg(short, long)]
        start: bool,
    },

    /// List all tickets
    List {
        /// Filter by status (todo, doing, done, blocked, review)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by priority (low, medium, high, critical)
        #[arg(long)]
        priority: Option<String>,

        /// Filter by assignee
        #[arg(short, long)]
        assignee: Option<String>,

        /// Sort by field (created, updated, priority, status, slug)
        #[arg(long, default_value = "slug")]
        sort: String,

        /// Reverse sort order
        #[arg(short, long)]
        reverse: bool,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Show archived tickets
        #[arg(long)]
        archived: bool,

        /// Show only open tickets (todo, doing)
        #[arg(long)]
        open: bool,

        /// Filter tickets created since (e.g., "yesterday", "2 days ago", "2025-07-18")
        #[arg(long)]
        since: Option<String>,

        /// Filter tickets created until (e.g., "today", "1 week ago", "2025-07-20")
        #[arg(long)]
        until: Option<String>,

        /// Include done tickets (by default they are hidden)
        #[arg(long)]
        include_done: bool,
    },

    /// Start working on a ticket
    Start {
        /// Ticket ID or slug
        ticket: String,

        /// Create a new Git branch
        #[arg(short, long, default_value = "true")]
        branch: bool,

        /// Custom branch name (default: ticket-{slug})
        #[arg(long)]
        branch_name: Option<String>,

        /// Create a Git worktree (use --no-worktree to disable)
        #[arg(long, default_value = "true")]
        worktree: bool,

        /// Disable worktree creation and only create a branch
        #[arg(long = "no-worktree", conflicts_with = "worktree")]
        no_worktree: bool,
    },

    /// Show open tickets (alias for list --open)
    Open {
        /// Sort by field (created, updated, priority, status, slug)
        #[arg(long, default_value = "updated")]
        sort: String,

        /// Reverse sort order
        #[arg(short, long)]
        reverse: bool,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Close the current ticket
    Close {
        /// Ticket ID or slug (defaults to active ticket)
        ticket: Option<String>,

        /// Close message
        #[arg(short, long)]
        message: Option<String>,

        /// Archive the ticket
        #[arg(short, long)]
        archive: bool,

        /// Create a merge/pull request
        #[arg(long)]
        pr: bool,
    },

    /// Check the current status
    Check {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Include statistics
        #[arg(short, long)]
        stats: bool,
    },

    /// Edit a ticket
    Edit {
        /// Ticket ID or slug (defaults to active ticket)
        ticket: Option<String>,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New description
        #[arg(long)]
        description: Option<String>,

        /// New priority
        #[arg(long)]
        priority: Option<String>,

        /// New status
        #[arg(long)]
        status: Option<String>,

        /// Add tags (comma-separated)
        #[arg(long)]
        add_tags: Option<String>,

        /// Remove tags (comma-separated)
        #[arg(long)]
        remove_tags: Option<String>,

        /// Open in editor
        #[arg(short, long)]
        editor: bool,
    },

    /// Show ticket details
    Show {
        /// Ticket ID or slug
        ticket: String,

        /// Show tasks
        #[arg(short, long)]
        tasks: bool,

        /// Show history
        #[arg(long)]
        history: bool,

        /// Show in markdown format
        #[arg(short, long)]
        markdown: bool,
    },

    /// Manage tasks within a ticket
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },

    /// Archive or unarchive tickets
    Archive {
        /// Ticket ID or slug
        ticket: String,

        /// Unarchive instead of archive
        #[arg(short, long)]
        unarchive: bool,
    },

    /// Search tickets
    Search {
        /// Search query
        query: String,

        /// Search in title only
        #[arg(long)]
        title: bool,

        /// Search in description only
        #[arg(long)]
        description: bool,

        /// Search in tags only
        #[arg(long)]
        tags: bool,

        /// Use regex
        #[arg(short, long)]
        regex: bool,
    },

    /// Export tickets
    Export {
        /// Output format (json, yaml, csv, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Include archived tickets
        #[arg(long)]
        include_archived: bool,
    },

    /// Import tickets
    Import {
        /// Input file
        file: String,

        /// Input format (json, yaml, csv)
        #[arg(short, long)]
        format: Option<String>,

        /// Skip validation
        #[arg(long)]
        skip_validation: bool,

        /// Dry run (don't actually import)
        #[arg(long)]
        dry_run: bool,
    },

    /// Manage project configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Manage specifications (spec-driven development)
    Spec {
        #[command(subcommand)]
        command: SpecCommands,
    },
    /// Manage Git worktrees for tickets
    Worktree {
        #[command(subcommand)]
        command: WorktreeCommands,
    },

    /// Model Context Protocol (MCP) server
    #[cfg(feature = "mcp")]
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Show specific key
        key: Option<String>,
    },

    /// Set configuration value
    Set {
        /// Configuration key (e.g., project.name, ui.emoji)
        key: String,

        /// Value to set
        value: String,
    },

    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Reset configuration to defaults
    Reset {
        /// Confirm reset
        #[arg(long)]
        force: bool,
    },

    /// Generate or update CLAUDE.md for AI assistance
    Claude {
        /// Append to existing CLAUDE.md instead of overwriting
        #[arg(short, long)]
        append: bool,

        /// Template to use (basic, advanced)
        #[arg(short, long, default_value = "basic")]
        template: String,

        /// Output path for CLAUDE.md (defaults to project root)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// Add a new task to a ticket
    Add {
        /// Task title
        title: String,

        /// Ticket ID or slug (defaults to active ticket)
        #[arg(short, long)]
        ticket: Option<String>,
    },

    /// Complete a task
    Complete {
        /// Task ID
        task: String,

        /// Ticket ID or slug (defaults to active ticket)
        #[arg(short, long)]
        ticket: Option<String>,
    },

    /// Uncomplete a task
    Uncomplete {
        /// Task ID
        task: String,

        /// Ticket ID or slug (defaults to active ticket)
        #[arg(short, long)]
        ticket: Option<String>,
    },

    /// List tasks in a ticket
    List {
        /// Ticket ID or slug (defaults to active ticket)
        #[arg(short, long)]
        ticket: Option<String>,

        /// Show completed tasks only
        #[arg(long)]
        completed: bool,

        /// Show incomplete tasks only
        #[arg(long)]
        incomplete: bool,
    },

    /// Remove a task
    Remove {
        /// Task ID
        task: String,

        /// Ticket ID or slug (defaults to active ticket)
        #[arg(short, long)]
        ticket: Option<String>,

        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum SpecCommands {
    /// Initialize a new specification
    Init {
        /// Specification title
        title: String,

        /// Specification description
        #[arg(short, long)]
        description: Option<String>,

        /// Associated ticket ID
        #[arg(short, long)]
        ticket: Option<String>,

        /// Initial tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },

    /// Create a specification from user requirements (spec-driven development)
    Specify {
        /// Requirements description in natural language
        requirements: String,

        /// Associated ticket ID or slug
        #[arg(short, long)]
        ticket: Option<String>,

        /// Use interactive mode for refinement
        #[arg(short, long)]
        interactive: bool,

        /// Template to use (default: standard)
        #[arg(long, default_value = "standard")]
        template: String,

        /// Output directory for specification
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Create implementation plan from specification
    Plan {
        /// Specification ID or path (defaults to active spec)
        #[arg(short, long)]
        spec: Option<String>,

        /// Technology stack (comma-separated, e.g., "rust,tokio,sqlx")
        #[arg(long)]
        tech_stack: Option<String>,

        /// Architecture pattern (e.g., "mvc", "hexagonal", "layered")
        #[arg(long)]
        architecture: Option<String>,

        /// Open in editor for refinement
        #[arg(short, long)]
        editor: bool,

        /// Output directory for plan
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Generate executable tasks from plan
    Tasks {
        /// Specification ID or path (defaults to active spec)
        #[arg(short, long)]
        spec: Option<String>,

        /// Plan file path (defaults to spec's plan)
        #[arg(long)]
        plan: Option<String>,

        /// Open in editor
        #[arg(short, long)]
        editor: bool,

        /// Mark as complete
        #[arg(long)]
        complete: bool,

        /// Export tasks to tickets
        #[arg(long)]
        export_tickets: bool,

        /// Generate parallel execution markers
        #[arg(long)]
        parallel: bool,

        /// Task granularity (fine, medium, coarse)
        #[arg(long, default_value = "medium")]
        granularity: String,
    },

    /// Validate specification against requirements
    Validate {
        /// Specification ID or path
        #[arg(short, long)]
        spec: Option<String>,

        /// Check for completeness
        #[arg(long)]
        complete: bool,

        /// Check for ambiguities
        #[arg(long)]
        ambiguities: bool,

        /// Generate validation report
        #[arg(long)]
        report: bool,
    },

    /// Create or update requirements document
    Requirements {
        /// Specification ID (defaults to active spec)
        #[arg(short, long)]
        spec: Option<String>,

        /// Open in editor
        #[arg(short, long)]
        editor: bool,

        /// Mark as complete
        #[arg(long)]
        complete: bool,
    },

    /// Create or update design document
    Design {
        /// Specification ID (defaults to active spec)
        #[arg(short, long)]
        spec: Option<String>,

        /// Open in editor
        #[arg(short, long)]
        editor: bool,

        /// Mark as complete
        #[arg(long)]
        complete: bool,
    },

    /// Show specification status
    Status {
        /// Specification ID (defaults to active spec)
        #[arg(short, long)]
        spec: Option<String>,

        /// Show detailed progress
        #[arg(short, long)]
        detailed: bool,

        /// Show validation status
        #[arg(long)]
        validation: bool,
    },

    /// List all specifications
    List {
        /// Filter by status (draft, `in_progress`, completed, approved)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by phase (requirements, design, tasks)
        #[arg(long)]
        phase: Option<String>,

        /// Show archived specs
        #[arg(long)]
        archived: bool,
    },

    /// Show specification details
    Show {
        /// Specification ID
        spec: String,

        /// Show all documents
        #[arg(short, long)]
        all: bool,

        /// Show in markdown format
        #[arg(short, long)]
        markdown: bool,
    },

    /// Delete a specification
    Delete {
        /// Specification ID
        spec: String,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Approve a specification phase
    Approve {
        /// Specification ID
        spec: String,

        /// Phase to approve (requirements, design, tasks)
        phase: String,

        /// Approval message
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Set active specification
    Activate {
        /// Specification ID
        spec: String,
    },

    /// Generate templates for spec-driven development
    Template {
        /// Template type (spec, plan, task, all)
        #[arg(default_value = "all")]
        template_type: String,

        /// Output directory
        #[arg(short, long, default_value = "templates")]
        output: String,

        /// Force overwrite existing templates
        #[arg(short, long)]
        force: bool,
    },
}

#[cfg(feature = "mcp")]
#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// Start MCP server
    Serve {
        /// Host to bind to
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: Option<String>,

        /// Port to listen on  
        #[arg(short = 'P', long, default_value = "3033")]
        port: Option<u16>,

        /// Run as daemon
        #[arg(short, long)]
        daemon: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorktreeCommands {
    /// List all worktrees for vibe-ticket
    List {
        /// Show worktrees for all tickets
        #[arg(short, long)]
        all: bool,

        /// Filter by status (active, stale, orphaned)
        #[arg(short, long)]
        status: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Remove a worktree
    Remove {
        /// Worktree path or ticket ID/slug
        worktree: String,

        /// Force removal even if there are uncommitted changes
        #[arg(short, long)]
        force: bool,

        /// Keep the branch associated with the worktree
        #[arg(long)]
        keep_branch: bool,
    },

    /// Prune stale worktrees
    Prune {
        /// Remove worktrees without confirmation
        #[arg(short, long)]
        force: bool,

        /// Dry run - show what would be removed
        #[arg(short, long)]
        dry_run: bool,

        /// Remove branches for pruned worktrees
        #[arg(long)]
        remove_branches: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Test basic CLI structure parsing
    #[test]
    fn test_cli_parse_basic() {
        let cli = Cli::parse_from(["vibe-ticket", "--version"]);
        assert!(!cli.verbose);
        assert!(!cli.json);
        assert!(!cli.no_color);
        assert!(cli.project.is_none());
    }

    /// Test global flags
    #[test]
    fn test_cli_global_flags() {
        let cli = Cli::parse_from([
            "vibe-ticket",
            "--verbose",
            "--json",
            "--no-color",
            "--project",
            "/path/to/project",
            "list",
        ]);
        assert!(cli.verbose);
        assert!(cli.json);
        assert!(cli.no_color);
        assert_eq!(cli.project, Some("/path/to/project".to_string()));
    }

    /// Test init command parsing
    #[test]
    fn test_init_command() {
        let cli = Cli::parse_from(["vibe-ticket", "init"]);
        match cli.command {
            Commands::Init {
                name,
                description,
                force,
                claude_md,
            } => {
                assert!(name.is_none());
                assert!(description.is_none());
                assert!(!force);
                assert!(!claude_md);
            },
            _ => panic!("Expected Init command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "init",
            "--name",
            "test-project",
            "--description",
            "Test description",
            "--force",
            "--claude-md",
        ]);
        match cli.command {
            Commands::Init {
                name,
                description,
                force,
                claude_md,
            } => {
                assert_eq!(name, Some("test-project".to_string()));
                assert_eq!(description, Some("Test description".to_string()));
                assert!(force);
                assert!(claude_md);
            },
            _ => panic!("Expected Init command"),
        }
    }

    /// Test new command parsing
    #[test]
    fn test_new_command() {
        let cli = Cli::parse_from(["vibe-ticket", "new", "fix-bug"]);
        match cli.command {
            Commands::New {
                slug,
                title,
                description,
                priority,
                tags,
                start,
            } => {
                assert_eq!(slug, "fix-bug");
                assert!(title.is_none());
                assert!(description.is_none());
                assert_eq!(priority, "medium");
                assert!(tags.is_none());
                assert!(!start);
            },
            _ => panic!("Expected New command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "new",
            "feature-auth",
            "--title",
            "Add authentication",
            "--priority",
            "high",
            "--tags",
            "auth,security",
            "--start",
        ]);
        match cli.command {
            Commands::New {
                slug,
                title,
                priority,
                tags,
                start,
                ..
            } => {
                assert_eq!(slug, "feature-auth");
                assert_eq!(title, Some("Add authentication".to_string()));
                assert_eq!(priority, "high");
                assert_eq!(tags, Some("auth,security".to_string()));
                assert!(start);
            },
            _ => panic!("Expected New command"),
        }
    }

    /// Test list command with various filters
    #[test]
    fn test_list_command() {
        let cli = Cli::parse_from(["vibe-ticket", "list"]);
        match cli.command {
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
                ..
            } => {
                assert!(status.is_none());
                assert!(priority.is_none());
                assert!(assignee.is_none());
                assert_eq!(sort, "slug");
                assert!(!reverse);
                assert!(limit.is_none());
                assert!(!archived);
                assert!(!open);
                assert!(since.is_none());
                assert!(until.is_none());
            },
            _ => panic!("Expected List command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "list",
            "--status",
            "doing",
            "--priority",
            "high",
            "--sort",
            "created",
            "--reverse",
            "--limit",
            "10",
            "--open",
            "--since",
            "yesterday",
        ]);
        match cli.command {
            Commands::List {
                status,
                priority,
                sort,
                reverse,
                limit,
                open,
                since,
                ..
            } => {
                assert_eq!(status, Some("doing".to_string()));
                assert_eq!(priority, Some("high".to_string()));
                assert_eq!(sort, "created");
                assert!(reverse);
                assert_eq!(limit, Some(10));
                assert!(open);
                assert_eq!(since, Some("yesterday".to_string()));
            },
            _ => panic!("Expected List command"),
        }
    }

    /// Test start command with worktree options
    #[test]
    fn test_start_command() {
        let cli = Cli::parse_from(["vibe-ticket", "start", "ticket-123"]);
        match cli.command {
            Commands::Start {
                ticket,
                branch,
                branch_name,
                worktree,
                no_worktree,
            } => {
                assert_eq!(ticket, "ticket-123");
                assert!(branch);
                assert!(branch_name.is_none());
                assert!(worktree);
                assert!(!no_worktree);
            },
            _ => panic!("Expected Start command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "start",
            "feature-xyz",
            "--no-worktree",
            "--branch-name",
            "custom-branch",
        ]);
        match cli.command {
            Commands::Start {
                ticket,
                branch_name,
                no_worktree,
                ..
            } => {
                assert_eq!(ticket, "feature-xyz");
                assert_eq!(branch_name, Some("custom-branch".to_string()));
                assert!(no_worktree);
            },
            _ => panic!("Expected Start command"),
        }
    }

    /// Test task subcommands
    #[test]
    fn test_task_commands() {
        let cli = Cli::parse_from(["vibe-ticket", "task", "add", "Write tests"]);
        match cli.command {
            Commands::Task { command } => match command {
                TaskCommands::Add { title, ticket } => {
                    assert_eq!(title, "Write tests");
                    assert!(ticket.is_none());
                },
                _ => panic!("Expected Task Add command"),
            },
            _ => panic!("Expected Task command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "task",
            "complete",
            "1",
            "--ticket",
            "fix-bug",
        ]);
        match cli.command {
            Commands::Task { command } => match command {
                TaskCommands::Complete { task, ticket } => {
                    assert_eq!(task, "1");
                    assert_eq!(ticket, Some("fix-bug".to_string()));
                },
                _ => panic!("Expected Task Complete command"),
            },
            _ => panic!("Expected Task command"),
        }

        let cli = Cli::parse_from(["vibe-ticket", "task", "list", "--completed"]);
        match cli.command {
            Commands::Task { command } => match command {
                TaskCommands::List {
                    ticket,
                    completed,
                    incomplete,
                } => {
                    assert!(ticket.is_none());
                    assert!(completed);
                    assert!(!incomplete);
                },
                _ => panic!("Expected Task List command"),
            },
            _ => panic!("Expected Task command"),
        }
    }

    /// Test config subcommands
    #[test]
    fn test_config_commands() {
        let cli = Cli::parse_from(["vibe-ticket", "config", "show"]);
        match cli.command {
            Commands::Config { command } => match command {
                ConfigCommands::Show { key } => {
                    assert!(key.is_none());
                },
                _ => panic!("Expected Config Show command"),
            },
            _ => panic!("Expected Config command"),
        }

        let cli = Cli::parse_from(["vibe-ticket", "config", "set", "ui.emoji", "true"]);
        match cli.command {
            Commands::Config { command } => match command {
                ConfigCommands::Set { key, value } => {
                    assert_eq!(key, "ui.emoji");
                    assert_eq!(value, "true");
                },
                _ => panic!("Expected Config Set command"),
            },
            _ => panic!("Expected Config command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "config",
            "claude",
            "--append",
            "--template",
            "advanced",
        ]);
        match cli.command {
            Commands::Config { command } => match command {
                ConfigCommands::Claude {
                    append,
                    template,
                    output,
                } => {
                    assert!(append);
                    assert_eq!(template, "advanced");
                    assert!(output.is_none());
                },
                _ => panic!("Expected Config Claude command"),
            },
            _ => panic!("Expected Config command"),
        }
    }

    /// Test spec subcommands
    #[test]
    fn test_spec_commands() {
        let cli = Cli::parse_from(["vibe-ticket", "spec", "init", "New Feature Spec"]);
        match cli.command {
            Commands::Spec { command } => match command {
                SpecCommands::Init {
                    title,
                    description,
                    ticket,
                    tags,
                } => {
                    assert_eq!(title, "New Feature Spec");
                    assert!(description.is_none());
                    assert!(ticket.is_none());
                    assert!(tags.is_none());
                },
                _ => panic!("Expected Spec Init command"),
            },
            _ => panic!("Expected Spec command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "spec",
            "requirements",
            "--editor",
            "--complete",
        ]);
        match cli.command {
            Commands::Spec { command } => match command {
                SpecCommands::Requirements {
                    spec,
                    editor,
                    complete,
                } => {
                    assert!(spec.is_none());
                    assert!(editor);
                    assert!(complete);
                },
                _ => panic!("Expected Spec Requirements command"),
            },
            _ => panic!("Expected Spec command"),
        }
    }

    /// Test worktree subcommands
    #[test]
    fn test_worktree_commands() {
        let cli = Cli::parse_from(["vibe-ticket", "worktree", "list", "--all", "--verbose"]);
        match cli.command {
            Commands::Worktree { command } => match command {
                WorktreeCommands::List {
                    all,
                    status,
                    verbose,
                } => {
                    assert!(all);
                    assert!(status.is_none());
                    assert!(verbose);
                },
                _ => panic!("Expected Worktree List command"),
            },
            _ => panic!("Expected Worktree command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "worktree",
            "remove",
            "fix-bug",
            "--force",
            "--keep-branch",
        ]);
        match cli.command {
            Commands::Worktree { command } => match command {
                WorktreeCommands::Remove {
                    worktree,
                    force,
                    keep_branch,
                } => {
                    assert_eq!(worktree, "fix-bug");
                    assert!(force);
                    assert!(keep_branch);
                },
                _ => panic!("Expected Worktree Remove command"),
            },
            _ => panic!("Expected Worktree command"),
        }
    }

    /// Test edge cases and error scenarios
    #[test]
    fn test_edge_cases() {
        // Test empty tags
        let cli = Cli::parse_from(["vibe-ticket", "new", "test", "--tags", ""]);
        match cli.command {
            Commands::New { tags, .. } => {
                assert_eq!(tags, Some(String::new()));
            },
            _ => panic!("Expected New command"),
        }

        // Test search with regex
        let cli = Cli::parse_from(["vibe-ticket", "search", "bug.*fix", "--regex"]);
        match cli.command {
            Commands::Search { query, regex, .. } => {
                assert_eq!(query, "bug.*fix");
                assert!(regex);
            },
            _ => panic!("Expected Search command"),
        }

        // Test export with custom output
        let cli = Cli::parse_from([
            "vibe-ticket",
            "export",
            "--format",
            "yaml",
            "--output",
            "tickets.yaml",
            "--include-archived",
        ]);
        match cli.command {
            Commands::Export {
                format,
                output,
                include_archived,
            } => {
                assert_eq!(format, "yaml");
                assert_eq!(output, Some("tickets.yaml".to_string()));
                assert!(include_archived);
            },
            _ => panic!("Expected Export command"),
        }
    }

    /// Test command aliases
    #[test]
    fn test_command_aliases() {
        // Test claude-md alias
        let cli = Cli::parse_from(["vibe-ticket", "init", "--claude"]);
        match cli.command {
            Commands::Init { claude_md, .. } => {
                assert!(claude_md);
            },
            _ => panic!("Expected Init command"),
        }
    }

    /// Test complex command combinations
    #[test]
    fn test_complex_commands() {
        // Test close with all options
        let cli = Cli::parse_from([
            "vibe-ticket",
            "close",
            "feature-123",
            "--message",
            "Completed feature",
            "--archive",
            "--pr",
        ]);
        match cli.command {
            Commands::Close {
                ticket,
                message,
                archive,
                pr,
            } => {
                assert_eq!(ticket, Some("feature-123".to_string()));
                assert_eq!(message, Some("Completed feature".to_string()));
                assert!(archive);
                assert!(pr);
            },
            _ => panic!("Expected Close command"),
        }

        // Test edit with multiple tag operations
        let cli = Cli::parse_from([
            "vibe-ticket",
            "edit",
            "--title",
            "New Title",
            "--add-tags",
            "urgent,frontend",
            "--remove-tags",
            "backend",
            "--status",
            "review",
        ]);
        match cli.command {
            Commands::Edit {
                ticket,
                title,
                add_tags,
                remove_tags,
                status,
                ..
            } => {
                assert!(ticket.is_none());
                assert_eq!(title, Some("New Title".to_string()));
                assert_eq!(add_tags, Some("urgent,frontend".to_string()));
                assert_eq!(remove_tags, Some("backend".to_string()));
                assert_eq!(status, Some("review".to_string()));
            },
            _ => panic!("Expected Edit command"),
        }
    }

    /// Test default values
    #[test]
    fn test_default_values() {
        // Test list sort default
        let cli = Cli::parse_from(["vibe-ticket", "list"]);
        match cli.command {
            Commands::List { sort, .. } => {
                assert_eq!(sort, "slug");
            },
            _ => panic!("Expected List command"),
        }

        // Test open sort default
        let cli = Cli::parse_from(["vibe-ticket", "open"]);
        match cli.command {
            Commands::Open { sort, .. } => {
                assert_eq!(sort, "updated");
            },
            _ => panic!("Expected Open command"),
        }

        // Test export format default
        let cli = Cli::parse_from(["vibe-ticket", "export"]);
        match cli.command {
            Commands::Export { format, .. } => {
                assert_eq!(format, "json");
            },
            _ => panic!("Expected Export command"),
        }
    }

    /// Test import command variations
    #[test]
    fn test_import_command() {
        let cli = Cli::parse_from(["vibe-ticket", "import", "data.json"]);
        match cli.command {
            Commands::Import {
                file,
                format,
                skip_validation,
                dry_run,
            } => {
                assert_eq!(file, "data.json");
                assert!(format.is_none());
                assert!(!skip_validation);
                assert!(!dry_run);
            },
            _ => panic!("Expected Import command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "import",
            "tickets.csv",
            "--format",
            "csv",
            "--skip-validation",
            "--dry-run",
        ]);
        match cli.command {
            Commands::Import {
                file,
                format,
                skip_validation,
                dry_run,
            } => {
                assert_eq!(file, "tickets.csv");
                assert_eq!(format, Some("csv".to_string()));
                assert!(skip_validation);
                assert!(dry_run);
            },
            _ => panic!("Expected Import command"),
        }
    }

    /// Test show command variations
    #[test]
    fn test_show_command() {
        let cli = Cli::parse_from(["vibe-ticket", "show", "ABC-123"]);
        match cli.command {
            Commands::Show {
                ticket,
                tasks,
                history,
                markdown,
            } => {
                assert_eq!(ticket, "ABC-123");
                assert!(!tasks);
                assert!(!history);
                assert!(!markdown);
            },
            _ => panic!("Expected Show command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "show",
            "feature-1",
            "--tasks",
            "--history",
            "--markdown",
        ]);
        match cli.command {
            Commands::Show {
                ticket,
                tasks,
                history,
                markdown,
            } => {
                assert_eq!(ticket, "feature-1");
                assert!(tasks);
                assert!(history);
                assert!(markdown);
            },
            _ => panic!("Expected Show command"),
        }
    }

    /// Test check command variations
    #[test]
    fn test_check_command() {
        let cli = Cli::parse_from(["vibe-ticket", "check"]);
        match cli.command {
            Commands::Check { detailed, stats } => {
                assert!(!detailed);
                assert!(!stats);
            },
            _ => panic!("Expected Check command"),
        }

        let cli = Cli::parse_from(["vibe-ticket", "check", "--detailed", "--stats"]);
        match cli.command {
            Commands::Check { detailed, stats } => {
                assert!(detailed);
                assert!(stats);
            },
            _ => panic!("Expected Check command"),
        }
    }

    /// Test archive command
    #[test]
    fn test_archive_command() {
        let cli = Cli::parse_from(["vibe-ticket", "archive", "old-ticket"]);
        match cli.command {
            Commands::Archive { ticket, unarchive } => {
                assert_eq!(ticket, "old-ticket");
                assert!(!unarchive);
            },
            _ => panic!("Expected Archive command"),
        }

        let cli = Cli::parse_from(["vibe-ticket", "archive", "ticket-123", "--unarchive"]);
        match cli.command {
            Commands::Archive { ticket, unarchive } => {
                assert_eq!(ticket, "ticket-123");
                assert!(unarchive);
            },
            _ => panic!("Expected Archive command"),
        }
    }

    /// Test search command filters
    #[test]
    fn test_search_filters() {
        let cli = Cli::parse_from(["vibe-ticket", "search", "auth", "--title"]);
        match cli.command {
            Commands::Search {
                query,
                title,
                description,
                tags,
                ..
            } => {
                assert_eq!(query, "auth");
                assert!(title);
                assert!(!description);
                assert!(!tags);
            },
            _ => panic!("Expected Search command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "search",
            "security",
            "--description",
            "--tags",
        ]);
        match cli.command {
            Commands::Search {
                query,
                title,
                description,
                tags,
                ..
            } => {
                assert_eq!(query, "security");
                assert!(!title);
                assert!(description);
                assert!(tags);
            },
            _ => panic!("Expected Search command"),
        }
    }

    /// Test spec command variations
    #[test]
    fn test_spec_variations() {
        let cli = Cli::parse_from([
            "vibe-ticket",
            "spec",
            "list",
            "--status",
            "draft",
            "--phase",
            "requirements",
            "--archived",
        ]);
        match cli.command {
            Commands::Spec { command } => match command {
                SpecCommands::List {
                    status,
                    phase,
                    archived,
                } => {
                    assert_eq!(status, Some("draft".to_string()));
                    assert_eq!(phase, Some("requirements".to_string()));
                    assert!(archived);
                },
                _ => panic!("Expected Spec List command"),
            },
            _ => panic!("Expected Spec command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "spec",
            "approve",
            "spec-123",
            "design",
            "--message",
            "LGTM",
        ]);
        match cli.command {
            Commands::Spec { command } => match command {
                SpecCommands::Approve {
                    spec,
                    phase,
                    message,
                } => {
                    assert_eq!(spec, "spec-123");
                    assert_eq!(phase, "design");
                    assert_eq!(message, Some("LGTM".to_string()));
                },
                _ => panic!("Expected Spec Approve command"),
            },
            _ => panic!("Expected Spec command"),
        }
    }

    /// Test worktree prune options
    #[test]
    fn test_worktree_prune() {
        let cli = Cli::parse_from(["vibe-ticket", "worktree", "prune"]);
        match cli.command {
            Commands::Worktree { command } => match command {
                WorktreeCommands::Prune {
                    force,
                    dry_run,
                    remove_branches,
                } => {
                    assert!(!force);
                    assert!(!dry_run);
                    assert!(!remove_branches);
                },
                _ => panic!("Expected Worktree Prune command"),
            },
            _ => panic!("Expected Worktree command"),
        }

        let cli = Cli::parse_from([
            "vibe-ticket",
            "worktree",
            "prune",
            "--force",
            "--dry-run",
            "--remove-branches",
        ]);
        match cli.command {
            Commands::Worktree { command } => match command {
                WorktreeCommands::Prune {
                    force,
                    dry_run,
                    remove_branches,
                } => {
                    assert!(force);
                    assert!(dry_run);
                    assert!(remove_branches);
                },
                _ => panic!("Expected Worktree Prune command"),
            },
            _ => panic!("Expected Worktree command"),
        }
    }
}
