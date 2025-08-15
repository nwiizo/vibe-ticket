# vibe-ticket CLI-MCP Integration Architecture Specification

## Overview

This document outlines the architectural changes required to integrate vibe-ticket CLI commands with the Model Context Protocol (MCP) service, ensuring bidirectional synchronization between command-line operations and MCP state.

## Goals

1. **Automatic Synchronization**: When a vibe-ticket command is executed via CLI, the corresponding changes should be automatically reflected in the MCP service
2. **Bidirectional Communication**: Changes made through MCP should be accessible via CLI, and vice versa
3. **Consistent State**: Ensure data consistency between CLI operations and MCP state
4. **Minimal Performance Impact**: Integration should not significantly impact CLI performance
5. **Backward Compatibility**: Existing CLI functionality should remain intact

## Current Architecture Analysis

### CLI Layer
- Entry point: `src/main.rs`
- Command handling: `src/cli/commands.rs`
- Ticket operations: `src/ticket/mod.rs`
- Storage: File-based YAML storage in `.vibe-ticket/` directory

### MCP Layer
- Service: `src/mcp/service.rs`
- Handlers: `src/mcp/handlers/`
  - `tickets.rs`: Ticket CRUD operations
  - `tasks.rs`: Task management
  - `search.rs`: Search functionality
  - `config.rs`: Configuration management
  - `worktree.rs`: Git worktree operations

### Storage Layer
- Shared storage: `src/storage/mod.rs`
- Cache: `src/cache/mod.rs`
- Both CLI and MCP use the same underlying storage

## Proposed Architecture

### 1. Event System
Introduce an event system that notifies MCP when CLI operations occur:

```rust
// src/events/mod.rs
pub enum TicketEvent {
    Created(Ticket),
    Updated(Ticket),
    Closed(TicketId),
    TaskAdded(TicketId, Task),
    TaskCompleted(TicketId, usize),
    StatusChanged(TicketId, Status),
}

pub trait EventHandler {
    fn handle_event(&self, event: TicketEvent) -> Result<()>;
}
```

### 2. Integration Points

#### CLI → MCP Flow
1. CLI command executed
2. Operation performed on storage
3. Event emitted
4. MCP event handler receives event
5. MCP updates internal state/cache
6. MCP notifies connected clients (if any)

#### MCP → CLI Flow
- Already working: MCP uses same storage layer
- CLI commands will see MCP changes immediately

### 3. Implementation Strategy

#### Phase 1: Event Infrastructure
- Create event system
- Add event emission to all ticket operations
- Implement MCP event handler

#### Phase 2: CLI Integration
- Modify CLI commands to emit events after successful operations
- Add configuration option to enable/disable MCP integration
- Ensure operations remain atomic

#### Phase 3: MCP Enhancement
- Add real-time notifications to MCP
- Implement caching strategy for better performance
- Add conflict resolution for concurrent operations

#### Phase 4: Testing & Documentation
- Comprehensive integration tests
- Performance benchmarks
- Update documentation

## Technical Details

### Event Emission Points

1. **Ticket Creation** (`vibe-ticket new`)
   - Location: `src/cli/commands.rs::handle_new_command()`
   - Event: `TicketEvent::Created`

2. **Ticket Updates** (`vibe-ticket edit`)
   - Location: `src/cli/commands.rs::handle_edit_command()`
   - Event: `TicketEvent::Updated`

3. **Ticket Status Changes** (`vibe-ticket start/close`)
   - Location: `src/cli/commands.rs::handle_start_command()`, `handle_close_command()`
   - Events: `TicketEvent::StatusChanged`, `TicketEvent::Closed`

4. **Task Operations** (`vibe-ticket task`)
   - Location: `src/cli/commands.rs::handle_task_command()`
   - Events: `TicketEvent::TaskAdded`, `TicketEvent::TaskCompleted`

### MCP Handler Implementation

```rust
// src/mcp/handlers/events.rs
pub struct McpEventHandler {
    service: Arc<VibeTicketMcp>,
}

impl EventHandler for McpEventHandler {
    fn handle_event(&self, event: TicketEvent) -> Result<()> {
        match event {
            TicketEvent::Created(ticket) => {
                // Update MCP cache
                // Notify connected clients
            }
            // ... handle other events
        }
        Ok(())
    }
}
```

### Configuration

Add new configuration options:
```yaml
mcp:
  enabled: true
  auto_sync: true
  conflict_resolution: "cli_priority" # or "mcp_priority", "newest_wins"
```

## Performance Considerations

1. **Asynchronous Event Handling**: Events should be processed asynchronously to not block CLI operations
2. **Batching**: Multiple rapid CLI operations could be batched for MCP updates
3. **Caching**: MCP should maintain an efficient cache to avoid repeated file I/O

## Security Considerations

1. **File Locking**: Ensure proper file locking for concurrent access
2. **Validation**: All events should be validated before processing
3. **Error Handling**: Failed MCP updates should not break CLI functionality

## Migration Path

1. Feature flag for gradual rollout
2. Backward compatibility with existing CLI usage
3. Clear documentation for users

## Success Criteria

1. All CLI operations trigger appropriate MCP updates
2. No performance degradation in CLI operations
3. MCP reflects CLI changes within 100ms
4. Zero data loss or corruption
5. Comprehensive test coverage (>90%)

## Timeline Estimate

- Phase 1: 2-3 days
- Phase 2: 3-4 days
- Phase 3: 2-3 days
- Phase 4: 2 days

Total: ~10-12 days

## Risks and Mitigation

1. **Risk**: Performance impact on CLI
   - **Mitigation**: Async event handling, benchmarking

2. **Risk**: Data consistency issues
   - **Mitigation**: Atomic operations, proper locking

3. **Risk**: Breaking existing functionality
   - **Mitigation**: Feature flags, comprehensive testing