# vibe-ticket

[![Crates.io](https://img.shields.io/crates/v/vibe-ticket.svg)](https://crates.io/crates/vibe-ticket)
[![Documentation](https://docs.rs/vibe-ticket/badge.svg)](https://docs.rs/vibe-ticket)
[![CI](https://github.com/nwiizo/vibe-ticket/workflows/CI/badge.svg)](https://github.com/nwiizo/vibe-ticket/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance ticket management system for developers, built with Rust. Features Git worktree integration for parallel development workflows.

## Quick Start

### Traditional Workflow
```bash
# Install
cargo install vibe-ticket

# Initialize project
vibe-ticket init

# Create and start a ticket
vibe-ticket new fix-bug --title "Fix login issue" --priority high
vibe-ticket start fix-bug  # Creates Git worktree by default

# Work in the worktree
cd ./my-project-vibeticket-fix-bug/

# Track progress
vibe-ticket task add "Fix null check"
vibe-ticket task complete 1

# Complete ticket
vibe-ticket close fix-bug --message "Fixed login issue"
```

### Spec-Driven Development Workflow (NEW!)
```bash
# Create specification from natural language
vibe-ticket spec specify "Build a REST API for user management"

# Generate implementation plan
vibe-ticket spec plan --tech-stack rust,actix-web

# Create tasks and export as tickets
vibe-ticket spec tasks --parallel --export-tickets

# Start working on generated tickets
vibe-ticket list --open
vibe-ticket start api-t001-initialize
```

## Key Features

- **Git Worktree Support**: Work on multiple tickets simultaneously
- **Concurrent Edit Protection**: Safe multi-user/multi-process ticket access with automatic lock management
- **Spec-Driven Development**: AI-powered specification generation from natural language
  - Create specs from requirements using `/specify`
  - Generate implementation plans with `/plan`
  - Create executable task lists with `/tasks`
  - Validate specifications with `/validate`
- **Task Management**: Break tickets into trackable tasks with parallel execution support
- **Bulk Operations**: Update, tag, close, or archive multiple tickets at once
- **Saved Filters**: Create and reuse custom ticket filters
- **Custom Aliases**: Define shortcuts for frequently used commands
- **Time Tracking**: Log time spent on tickets with detailed reports
- **Custom Hooks**: Execute scripts on ticket lifecycle events
- **Interactive Selection**: fzf-style fuzzy search for quick ticket selection
- **Flexible Search**: Find tickets with powerful filters
- **Export/Import**: JSON, YAML, CSV, and Markdown formats
- **AI Integration**: Full Claude Code support with slash commands
- **MCP Server**: Run as Model Context Protocol server for AI assistants

## Spec-Driven Development (NEW!)

Transform natural language requirements into executable specifications:

```bash
# Create specification from requirements
vibe-ticket spec specify "Build a REST API for user management"

# Generate implementation plan
vibe-ticket spec plan --tech-stack rust,actix-web --architecture microservices

# Create executable task list
vibe-ticket spec tasks --granularity fine --parallel --export-tickets

# Validate specification
vibe-ticket spec validate --check-ambiguities --generate-report
```

### Slash Commands in Claude Code

When using Claude Code with MCP integration:
```
/specify "Build a REST API with authentication"
/plan --tech-stack rust,postgresql
/tasks --parallel --export-tickets
/validate --generate-report
```

## Essential Commands

```bash
vibe-ticket --help              # Show help for any command
vibe-ticket check               # Check current status
vibe-ticket list --open         # Show active tickets
vibe-ticket search "keyword"    # Search tickets
vibe-ticket worktree list       # List all worktrees
vibe-ticket spec list           # List all specifications
```

## Productivity Features (NEW!)

### Bulk Operations
```bash
# Update multiple tickets at once
vibe-ticket bulk update --status doing --priority high tag1,tag2

# Tag multiple tickets
vibe-ticket bulk tag "important,urgent" ticket1 ticket2 ticket3

# Close multiple tickets
vibe-ticket bulk close ticket1 ticket2 --message "Batch close"

# Archive old tickets
vibe-ticket bulk archive --before 2024-01-01
```

### Saved Filters
```bash
# Create a reusable filter
vibe-ticket filter create urgent-bugs --status todo --priority high --tags bug

# List saved filters
vibe-ticket filter list

# Apply a filter
vibe-ticket filter apply urgent-bugs
```

### Custom Aliases
```bash
# Create command shortcuts
vibe-ticket alias create today "list --status doing"
vibe-ticket alias create urgent "list --priority high --priority critical"

# Run an alias
vibe-ticket alias run today
```

### Time Tracking
```bash
# Start/stop timer
vibe-ticket time start my-ticket
vibe-ticket time stop

# Log time manually
vibe-ticket time log my-ticket 2h30m --description "Implemented feature"

# View time report
vibe-ticket time report --period week
vibe-ticket time status
```

### Custom Hooks
```bash
# Create lifecycle hooks
vibe-ticket hook create notify-slack post-close \
  --command 'curl -X POST $SLACK_WEBHOOK -d "{\"text\": \"Ticket $TICKET_SLUG closed\"}"' \
  --description "Notify Slack on ticket close"

# List and manage hooks
vibe-ticket hook list
vibe-ticket hook enable notify-slack
vibe-ticket hook test notify-slack
```

### Interactive Selection (fzf-style)
```bash
# Fuzzy search and select a ticket
vibe-ticket interactive select

# Multi-select for bulk operations
vibe-ticket interactive multi --status todo

# Quick status/priority change
vibe-ticket interactive status my-ticket
vibe-ticket interactive priority my-ticket
```

## Configuration

```yaml
# .vibe-ticket/config.yaml
git:
  worktree_default: true        # Create worktrees by default
  worktree_prefix: "./{project}-vibeticket-"
project:
  default_priority: medium
```

## Documentation

- [Command Reference](docs/commands.md)
- [Configuration](docs/configuration.md)
- [Spec-Driven Development](docs/spec-driven-development.md)
- [Slash Commands Guide](SLASH_COMMANDS.md) - **NEW!**
- [Git Worktree Guide](docs/git-worktree.md)
- [Claude Integration](CLAUDE.md)
- [MCP Integration Guide](docs/mcp-guide.md)
- [Data Formats](docs/data-formats.md)

## AI Assistant Setup

```bash
# Generate CLAUDE.md for AI assistance
vibe-ticket init --claude-md

# Add strict AI rules
curl https://raw.githubusercontent.com/nwiizo/vibe-ticket/main/rules/agent.md >> CLAUDE.md
```

### MCP (Model Context Protocol) Support

vibe-ticket can run as an MCP server for AI assistants like Claude:

```bash
# Install (MCP is now included by default)
cargo install vibe-ticket

# Add to Claude Code (global)
claude mcp add vibe-ticket ls $HOME/.cargo/bin/vibe-ticket --scope local -- mcp serve

# Test the server
vibe-ticket mcp serve
```

#### AI Assistant Integration

When using vibe-ticket with AI assistants via MCP:

1. **All CLI operations are available through MCP** - AI can create tickets, manage tasks, search, and more
2. **Spec-Driven Development with AI** - Use natural language to create specifications and generate implementation plans
3. **Slash Commands in Claude Code** - Direct commands like `/specify`, `/plan`, `/tasks`, `/validate`
4. **Integrated workflow** - AI can seamlessly switch between code editing and ticket management

Example AI interactions:
```
"Create a ticket for the bug we just found"
"Generate a specification for a REST API"
"/specify Build a user authentication system"
"/plan --tech-stack rust,jwt --architecture layered"
"/tasks --granularity fine --parallel"
"Validate the specification and check for ambiguities"
```

Available MCP Tools for Spec-Driven Development:
- `vibe-ticket_spec_specify` - Create specs from natural language
- `vibe-ticket_spec_plan` - Generate implementation plans
- `vibe-ticket_spec_generate_tasks` - Create task lists
- `vibe-ticket_spec_validate` - Validate specifications

See [MCP Integration Guide](docs/mcp-guide.md) for detailed setup and usage.

## Best Practices

### Ticket Management
- Always create a ticket before starting work
- Use meaningful ticket slugs that describe the task
- Update ticket status as work progresses
- Close tickets with descriptive completion messages

### Git Worktree Workflow
- Each ticket gets its own isolated worktree directory
- Work in `./project-vibeticket-<slug>/` directories
- Clean up worktrees after closing tickets: `vibe-ticket worktree remove <ticket>`
- Use `vibe-ticket worktree list` to track active worktrees

### Documentation Testing
- Run `cargo test --doc` regularly to ensure examples work
- Keep documentation examples up-to-date with code changes
- Doc tests prevent documentation drift

### Active Development Tips
- Check current context with `vibe-ticket check`
- Use `vibe-ticket list --status doing` to see active work
- Break complex work into tasks within tickets
- Conduct retrospectives after major tasks

### Concurrent Access Safety
- vibe-ticket automatically handles multiple users/processes accessing tickets
- File locking prevents data corruption during concurrent modifications
- Stale locks are automatically cleaned up after 30 seconds
- Operations retry automatically if a file is temporarily locked

## Installation

### From Source

```bash
git clone https://github.com/nwiizo/vibe-ticket.git
cd vibe-ticket
cargo build --release
cargo install --path .

# With MCP support
cargo build --release --features mcp
cargo install --path . --features mcp
```

### Prerequisites

- Rust 1.70+
- Git (for branch/worktree features)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[MIT License](LICENSE)
