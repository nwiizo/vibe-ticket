# vibe-ticket Project: vibe-ticket

A vibe-ticket managed project

## Overview

This project uses vibe-ticket for ticket management. This document provides guidance for Claude Code when working with this codebase.

## Common vibe-ticket Commands

### Getting Started
```bash
# Create your first ticket
vibe-ticket new fix-bug --title "Fix login issue" --priority high

# List all tickets
vibe-ticket list

# Start working on a ticket (creates worktree by default)
vibe-ticket start fix-bug
# This creates: ./vibe-ticket-vibeticket-fix-bug/

# Start without worktree (branch only)
vibe-ticket start fix-bug --no-worktree

# Show current status
vibe-ticket check
```

### Working with Tickets
```bash
# Show ticket details
vibe-ticket show <ticket>

# Update ticket
vibe-ticket edit <ticket> --status review

# Add tasks to ticket
vibe-ticket task add "Write unit tests"
vibe-ticket task add "Update documentation"

# Complete tasks
vibe-ticket task complete 1

# Close ticket
vibe-ticket close <ticket> --message "Fixed the login issue"
```

### Search and Filter
```bash
# Search tickets
vibe-ticket search "login"

# Filter by status
vibe-ticket list --status doing

# Filter by priority
vibe-ticket list --priority high
```

### Git Worktree Management
```bash
# List all worktrees for tickets
vibe-ticket worktree list

# List all worktrees (including non-ticket ones)
vibe-ticket worktree list --all

# Remove a worktree
vibe-ticket worktree remove fix-bug

# Prune stale worktrees
vibe-ticket worktree prune
```

### Configuration
```bash
# View configuration
vibe-ticket config show

# Set configuration values
vibe-ticket config set project.default_priority medium
vibe-ticket config set git.auto_branch true
vibe-ticket config set git.worktree_default false  # Disable default worktree creation

# Generate this file
vibe-ticket config claude
```

## Using vibe-ticket with MCP (Model Context Protocol)

vibe-ticket now supports MCP, allowing you to manage tickets through AI assistants and other MCP-compatible tools.

### Quick MCP Setup

```bash
# Install with MCP support
cargo install vibe-ticket --features mcp

# Add to Claude Code
claude mcp add vibe-ticket ~/.cargo/bin/vibe-ticket --scope global -- mcp serve

# Verify installation
claude mcp list | grep vibe-ticket
```

For detailed setup options, see [MCP Integration Guide](docs/mcp-integration.md).

### Available MCP Tools

When using vibe-ticket through MCP, the following tools are available:

#### Ticket Management
- `vibe-ticket_new` - Create a new ticket
- `vibe-ticket_list` - List tickets with filters
- `vibe-ticket_show` - Show ticket details
- `vibe-ticket_edit` - Edit ticket properties
- `vibe-ticket_close` - Close a ticket
- `vibe-ticket_start` - Start working on a ticket
- `vibe-ticket_check` - Check current status

#### Task Management
- `vibe-ticket_task_add` - Add a task to a ticket
- `vibe-ticket_task_complete` - Complete a task
- `vibe-ticket_task_list` - List tasks for a ticket
- `vibe-ticket_task_remove` - Remove a task

#### Specification Management (New)
- `vibe-ticket_spec_add` - Add specifications to a ticket
- `vibe-ticket_spec_update` - Update specifications for a ticket
- `vibe-ticket_spec_check` - Check specification status

#### Worktree Management
- `vibe-ticket_worktree_list` - List worktrees
- `vibe-ticket_worktree_remove` - Remove a worktree
- `vibe-ticket_worktree_prune` - Prune stale worktrees

#### Other Operations
- `vibe-ticket_search` - Search tickets
- `vibe-ticket_export` - Export tickets
- `vibe-ticket_import` - Import tickets
- `vibe-ticket_config_show` - Show configuration
- `vibe-ticket_config_set` - Set configuration values

### Using MCP Tools in Your Sessions

When I have access to vibe-ticket MCP tools, I can:
- Create and manage tickets directly
- Update ticket status and properties
- Add and complete tasks
- Search and filter tickets
- Manage Git worktrees

Example: "Create a ticket for implementing user authentication with high priority"
→ I'll use `vibe-ticket_new` with appropriate arguments

### Key Points for MCP Usage

- **Ticket References**: Use either slug or ID
- **Status Values**: `todo`, `doing`, `done`, `blocked`, `review`
- **Priority Values**: `low`, `medium`, `high`, `critical`
- **Synchronization**: CLI and MCP share the same data instantly

## Project Configuration

The project has been initialized with default settings. You can customize them using the config commands above.

### Git Worktree Configuration
```yaml
git:
  worktree_enabled: true              # Enable worktree support
  worktree_default: true              # Create worktree by default when starting tickets
  worktree_prefix: "./{project}-vibeticket-"  # Directory naming pattern
  worktree_cleanup_on_close: false   # Auto-remove worktree when closing ticket
```

## Workflow Guidelines

1. Create a ticket before starting any work
2. Use descriptive ticket slugs (e.g., fix-login-bug, add-search-feature)
3. When starting a ticket, a Git worktree is created automatically
   - Work in the worktree directory: `./vibe-ticket-vibeticket-<slug>/`
   - Each ticket has its own isolated working directory
4. Break down complex work into tasks within tickets
5. Keep ticket status updated as work progresses
6. Close tickets with meaningful completion messages
7. Remove worktrees when done: `vibe-ticket worktree remove <ticket>`

## Best Practices for This Project

- Follow the established ticket naming conventions
- Use appropriate priority levels (low, medium, high, critical)
- Tag tickets for better organization
- Document decisions in ticket descriptions
- Link related tickets when applicable

## Tips for Claude Code

When helping with this project:
1. Always check for active tickets before suggesting new work
2. Reference ticket IDs in commit messages
3. Update ticket status as implementation progresses
4. Use `vibe-ticket check` to understand current context
5. Generate new tickets for bugs or features discovered during development
6. **USE MCP TOOLS**: When users describe repetitive tasks or workflow improvements, proactively suggest using vibe-ticket MCP tools to automate them
7. **IMPORTANT**: After completing each major task or work session, provide a retrospective that includes:
   - What was accomplished
   - What challenges were encountered
   - What could be improved for next time
   - Any vibe-tickets that should be created for follow-up work
   - Lessons learned that could benefit future development

## MCP Integration for AI Assistants

vibe-ticket provides full MCP support, allowing AI assistants to:
- Create and manage tickets programmatically
- Automate repetitive ticket operations
- Generate reports and analytics
- Integrate ticket management with code changes

**When to suggest MCP tools:**
- User mentions creating multiple similar tickets
- Repetitive task management is needed
- Bulk operations on tickets would save time
- Workflow automation opportunities arise

Example: "I notice you're creating several similar tickets. Would you like me to use the vibe-ticket MCP tools to automate this process?"

## Lessons Learned from Development

### Active Ticket Management
- Always update `.vibe-ticket/active_ticket` file when switching between tickets
- Use `vibe-ticket start <id> --no-worktree` when a worktree already exists
- Check ticket ID format carefully - use full UUID when needed

### Worktree Best Practices
- Each ticket has its own worktree directory for isolated development
- Worktree names follow pattern: `vibe-ticket-vibeticket<date>-<slug>`
- Always verify worktree existence before creating new ones
- Clean up worktrees after closing tickets to save disk space

### Documentation Testing
- Run `cargo test --doc` to verify all documentation examples
- Doc tests ensure code examples in documentation remain accurate
- Keeping doc tests passing prevents documentation drift

## Work Retrospectives

### Why Retrospectives Matter
Retrospectives help improve the development process by:
- Identifying recurring issues before they become major problems
- Documenting solutions for future reference
- Creating actionable tickets for improvements
- Building institutional knowledge

### When to Conduct Retrospectives
- After completing any release preparation
- When finishing a complex feature implementation
- After resolving critical bugs
- At the end of each work session involving multiple tasks
- When encountering unexpected challenges

### Retrospective Template
```markdown
## Retrospective: [Task/Feature Name] - [Date]

### Summary
Brief overview of what was worked on.

### What Went Well
- List successes and smooth processes
- Note effective tools or techniques used

### Challenges Encountered
- Document specific problems faced
- Include error messages or unexpected behaviors

### Improvements for Next Time
- Concrete suggestions for process improvements
- Tools or automation that could help

### Follow-up Tickets Created
- List any vibe-tickets created as a result
- Include ticket IDs and brief descriptions

### Lessons Learned
- Key insights that will help future development
- Patterns to watch for or avoid
```

## Retrospective: Fix Documentation Tests - 2025-07-31

### Summary
Fixed failing documentation tests that were preventing clean builds. All doc tests now pass successfully.

### What Went Well
- Quick identification of the issue through `cargo test --doc`
- Existing worktree made it easy to isolate work
- Tests were already well-structured, just needed minor fixes

### Challenges Encountered
- Initial confusion with ticket ID format (short vs full UUID)
- Active ticket file needed manual update when switching contexts
- Some doc tests were marked as ignored, which may need review

### Improvements for Next Time
- Create helper command to switch active tickets more easily
- Consider automation for active ticket management
- Review ignored doc tests to ensure they're still relevant

### Follow-up Tickets Created
None - all documentation tests are now passing

### Lessons Learned
- Documentation tests are crucial for maintaining accurate examples
- Worktree workflow is effective for isolated development
- Active ticket management could benefit from better tooling

---
Generated on: 2025-07-22

## Retrospective: MCP Task Management - 2025-08-14

### Summary
Successfully managed tickets through MCP tools, added tasks to previously task-less tickets, and identified areas for improvement.

### What Went Well
- MCP tools worked seamlessly for ticket management
- Successfully added meaningful tasks to 4 tickets that lacked them
- Completed and closed tickets efficiently through MCP
- Created new tickets for discovered issues

### Challenges Encountered
- Some tickets have many tasks but unclear completion criteria
- Manual task status updates could be automated
- Batch operations for tasks would improve efficiency

### Improvements for Next Time
- Implement batch task operations for efficiency
- Add automatic status updates when all tasks complete
- Improve error handling in MCP tools

### Follow-up Tickets Created
- `improve-mcp-error-handling` - MCPツールのエラーハンドリング改善
- `add-batch-task-operations` - タスクの一括操作機能追加
- `auto-status-update` - チケットステータスの自動更新機能

### Lessons Learned
- MCP integration significantly speeds up ticket management
- Task definitions are crucial for tracking progress
- Automation opportunities exist in workflow management

---
Generated on: 2025-08-14


## Project Initialization

This project was initialized with:
```bash
vibe-ticket init --claude-md
```

To regenerate or update this file:
```bash
# Regenerate with basic template
vibe-ticket config claude

# Append with advanced features
vibe-ticket config claude --template advanced --append
```
