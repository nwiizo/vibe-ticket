# Command Structure Refactoring Design

## Current vs. Proposed Command Structure

### Current Structure (Tool-Focused)
```bash
vibe-ticket new <slug>           # Create ticket
vibe-ticket list                 # List tickets  
vibe-ticket start <ticket>       # Start work
vibe-ticket close <ticket>       # Close ticket
vibe-ticket task add <title>     # Add task
vibe-ticket spec init            # Initialize spec
```

### Proposed Structure (Intent-Focused)
```bash
# Primary Intent Commands
vibe-ticket create               # Interactive ticket creation
vibe-ticket work-on [ticket]     # Start working (interactive if no ticket)
vibe-ticket finish [ticket]      # Complete work (interactive if no ticket)
vibe-ticket review               # Review tickets and tasks
vibe-ticket report               # Generate reports

# Specification Workflow (Already intent-focused)
vibe-ticket specify <requirements>  # Create specification from requirements
vibe-ticket plan [spec]            # Create implementation plan
vibe-ticket tasks [spec]           # Generate tasks from plan

# Quick Actions (shortcuts)
vibe-ticket quick-fix <description>    # Create and start a quick fix ticket
vibe-ticket quick-task <description>   # Add task to current ticket
vibe-ticket quick-note <note>          # Add note to current ticket
```

## Implementation Plan

### Phase 1: Add Intent Commands (Non-Breaking)
- Add new intent-focused commands alongside existing ones
- Implement interactive flows for each intent command
- Use existing handlers internally

### Phase 2: Command Aliases
- Map old commands to new structure as aliases
- Add deprecation notices for old commands
- Update documentation to prefer new commands

### Phase 3: Simplify Subcommands
- Reduce nested subcommands
- Move complex operations to interactive mode
- Provide smart defaults

## Intent Command Implementations

### 1. `vibe-ticket create`
```rust
pub fn handle_create_command(
    interactive: bool,
    template: Option<String>,
    formatter: &OutputFormatter,
) -> Result<()> {
    if interactive || template.is_some() {
        // Use interactive mode with templates
        let mode = InteractiveMode::new();
        let ticket_data = mode.create_ticket()?;
        // Create ticket from data
    } else {
        // Quick creation with prompts
        // Title? Description? Priority? Tags?
    }
}
```

### 2. `vibe-ticket work-on`
```rust
pub fn handle_work_on_command(
    ticket: Option<String>,
    formatter: &OutputFormatter,
) -> Result<()> {
    let ticket_id = if let Some(t) = ticket {
        t
    } else {
        // Show list of available tickets
        // Let user select interactively
        interactive_select_ticket()?
    };
    
    // Start work on selected ticket
    // Create worktree if configured
    // Show ticket details and tasks
}
```

### 3. `vibe-ticket finish`
```rust
pub fn handle_finish_command(
    ticket: Option<String>,
    formatter: &OutputFormatter,
) -> Result<()> {
    let ticket_id = ticket.or_else(get_current_ticket)?;
    
    // Check all tasks completed
    // Prompt for closing message
    // Clean up worktree if exists
    // Close ticket
}
```

### 4. `vibe-ticket review`
```rust
pub fn handle_review_command(
    filter: Option<String>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Show dashboard view:
    // - Active tickets
    // - Pending tasks
    // - Recent activity
    // - Blocked items
    
    // Interactive options:
    // - Select ticket to view details
    // - Mark tasks complete
    // - Update status
}
```

### 5. `vibe-ticket report`
```rust
pub fn handle_report_command(
    report_type: Option<String>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Generate reports:
    // - Progress report
    // - Time tracking
    // - Completion stats
    // - Burndown chart
}
```

## Quick Actions

### `quick-fix`
One-line command to create and start a bug fix ticket:
```bash
vibe-ticket quick-fix "Login button not working on mobile"
# Creates ticket with:
# - Auto-generated slug
# - High priority
# - bug tag
# - Starts work immediately
```

### `quick-task`
Add task to current ticket without context switch:
```bash
vibe-ticket quick-task "Add unit tests for auth module"
```

### `quick-note`
Add note/comment to current ticket:
```bash
vibe-ticket quick-note "Discussed with team, proceeding with approach B"
```

## Migration Strategy

1. **v0.4.0** - Add new commands, keep old ones
2. **v0.5.0** - Mark old commands as deprecated
3. **v0.6.0** - Hide old commands (still work but not in help)
4. **v1.0.0** - Remove old commands

## Benefits

1. **Reduced Cognitive Load**
   - Commands match user intent
   - Less memorization needed
   - Natural workflow progression

2. **Improved Discoverability**
   - Intent-based commands are self-documenting
   - Interactive mode guides users
   - Smart defaults reduce decisions

3. **Better UX**
   - Fewer commands to remember
   - Progressive disclosure of complexity
   - Context-aware suggestions

4. **Maintaining Power User Features**
   - All existing functionality preserved
   - Power users can use flags to skip interactive mode
   - Aliases for common workflows