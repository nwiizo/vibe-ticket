# vibe-ticket Project: vibe-ticket

A vibe-ticket managed project with spec-driven development support.

## Quick Reference

| Feature | Command | Slash Command | MCP Tool |
|---------|---------|---------------|----------|
| Create spec from requirements | `vibe-ticket spec specify "..."` | `/specify "..."` | `vibe-ticket_spec_specify` |
| Generate plan | `vibe-ticket spec plan --tech-stack ...` | `/plan --tech-stack ...` | `vibe-ticket_spec_plan` |
| Create tasks | `vibe-ticket spec tasks --parallel` | `/tasks --parallel` | `vibe-ticket_spec_generate_tasks` |
| Validate spec | `vibe-ticket spec validate` | `/validate` | `vibe-ticket_spec_validate` |

## Overview

This project uses vibe-ticket for ticket management with advanced spec-driven development capabilities. This document provides guidance for Claude Code when working with this codebase.

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

#### Specification Management (Spec-Driven Development)
- `vibe-ticket_spec_add` - Add specifications to a ticket
- `vibe-ticket_spec_update` - Update specifications for a ticket
- `vibe-ticket_spec_check` - Check specification status
- `vibe-ticket_spec_specify` - Create specification from natural language (NEW!)
- `vibe-ticket_spec_plan` - Generate implementation plan (NEW!)
- `vibe-ticket_spec_generate_tasks` - Create executable task list (NEW!)
- `vibe-ticket_spec_validate` - Validate specification completeness (NEW!)

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

### Slash Commands for Spec-Driven Development (NEW!)

When using Claude Code, you can use these slash commands for spec-driven development:

- `/specify "requirements"` - Create specification from natural language
- `/plan --tech-stack rust,actix --architecture microservices` - Generate implementation plan
- `/tasks --granularity fine --parallel` - Create executable task list
- `/validate --check-ambiguities` - Validate specification

Example workflow:
```
/specify "Build a REST API for user management"
/plan --tech-stack rust,postgresql
/tasks --parallel --export-tickets
/validate --generate-report
```

For detailed slash command documentation, see [SLASH_COMMANDS.md](SLASH_COMMANDS.md).

### Using MCP Tools in Your Sessions

When I have access to vibe-ticket MCP tools, I can:
- Create and manage tickets directly
- Update ticket status and properties
- Generate specifications from natural language requirements
- Create implementation plans and task lists automatically
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

## Spec-Driven Development Workflow

### Overview
vibe-ticket now supports spec-driven development, inspired by GitHub's spec-kit. This approach transforms natural language requirements into executable specifications that generate implementations.

### Workflow Steps

1. **Specify** - Create specification from natural language
   ```bash
   vibe-ticket spec specify "Build a REST API for user management"
   # Or in Claude Code: /specify "Build a REST API for user management"
   ```

2. **Plan** - Generate implementation plan
   ```bash
   vibe-ticket spec plan --tech-stack rust,actix-web --architecture microservices
   # Or in Claude Code: /plan --tech-stack rust,actix-web
   ```

3. **Tasks** - Create executable task list
   ```bash
   vibe-ticket spec tasks --granularity fine --parallel --export-tickets
   # Or in Claude Code: /tasks --parallel --export-tickets
   ```

4. **Validate** - Check specification completeness
   ```bash
   vibe-ticket spec validate --check-ambiguities --generate-report
   # Or in Claude Code: /validate --generate-report
   ```

### Key Concepts

- **[NEEDS CLARIFICATION]** markers: Identify ambiguous requirements
- **[P]** markers: Tasks that can execute in parallel
- **Three-phase approach**: Requirements → Design → Tasks
- **Granularity levels**: fine (20+ tasks), medium (10 tasks), coarse (5 tasks)

## Workflow Guidelines

1. **For new features**: Start with spec-driven development
   - Use `/specify` to create specification from requirements
   - Generate plan and tasks automatically
   - Export tasks as tickets for tracking

2. **For bug fixes**: Use traditional ticket workflow
   - Create ticket directly with `vibe-ticket new`
   - Start work with automatic worktree creation

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
7. **USE SLASH COMMANDS**: Leverage spec-driven development commands:
   - `/specify` when user describes new features
   - `/plan` to generate implementation strategies
   - `/tasks` to break down work into manageable pieces
   - `/validate` to ensure specifications are complete
8. **IMPORTANT**: After completing each major task or work session, provide a retrospective that includes:
   - What was accomplished
   - What challenges were encountered
   - What could be improved for next time
   - Any vibe-tickets that should be created for follow-up work
   - Lessons learned that could benefit future development

### Proactive Suggestions

When you see opportunities, suggest:
- "Would you like me to create a specification for this feature using `/specify`?"
- "I can generate an implementation plan with `/plan --tech-stack <tech>`"
- "Should I create parallel tasks for this work using `/tasks --parallel`?"
- "Let me validate the specification to check for ambiguities"

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

## Complete Workflow Example

Here's how to use vibe-ticket for a new feature development:

```bash
# Step 1: Create specification from requirements
vibe-ticket spec specify "Build a user authentication system with JWT tokens"

# Step 2: Generate implementation plan
vibe-ticket spec plan --tech-stack rust,jwt,redis --architecture layered

# Step 3: Create tasks with parallel execution
vibe-ticket spec tasks --granularity fine --parallel --export-tickets

# Step 4: Validate specification
vibe-ticket spec validate --check-ambiguities --generate-report

# Step 5: Start working on generated tickets
vibe-ticket list --status todo
vibe-ticket start auth-t001-initialize

# Step 6: Track progress
vibe-ticket task add "Set up JWT library"
vibe-ticket task complete 1

# Step 7: Complete and close
vibe-ticket close auth-t001-initialize --message "Initialized authentication module"
```

### Using Slash Commands in Claude Code

```
User: "I need to build a REST API for managing blog posts"

Claude: I'll help you create a specification for that using spec-driven development.

/specify "Build a REST API for managing blog posts with CRUD operations"

[Specification created successfully]

Now let me generate an implementation plan:

/plan --tech-stack rust,actix-web,postgresql --architecture rest

[Plan generated]

Let's create tasks that can be executed in parallel:

/tasks --granularity medium --parallel --export-tickets

[12 tasks created and exported as tickets]

Finally, let me validate the specification:

/validate --check-ambiguities

[Validation complete - no issues found]

You now have a complete specification with an implementation plan and 12 tickets ready to work on. 
You can start with: vibe-ticket start api-t001-initialize
```

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
