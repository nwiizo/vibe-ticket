# Code Duplication Refactoring Plan

Based on `similarity-rs` analysis, here are the identified code duplications and refactoring recommendations:

## Critical Duplications (>95% similarity)

### 1. CLI Handlers Common Pattern (Multiple files)
**Problem**: Most CLI handlers have identical initialization boilerplate
- Files affected: `close.rs`, `edit.rs`, `new.rs`, `start.rs`, `list.rs`, `show.rs`, `task.rs`
- Similarity: 98-99%
- Pattern:
  ```rust
  // Ensure project is initialized
  let project_root = find_project_root(project_dir)?;
  let vibe_ticket_dir = project_root.join(".vibe-ticket");
  
  // Initialize storage
  let storage = FileStorage::new(&vibe_ticket_dir);
  ```

**Solution**: Extract to common function
```rust
// src/cli/handlers/common.rs
pub fn init_handler_context(project_dir: Option<&str>) -> Result<(PathBuf, FileStorage)> {
    let project_root = find_project_root(project_dir)?;
    let vibe_ticket_dir = project_root.join(".vibe-ticket");
    let storage = FileStorage::new(&vibe_ticket_dir);
    Ok((project_root, storage))
}
```

### 2. Export Module Functions (export/markdown.rs)
**Problem**: Multiple small functions with nearly identical structure (98-99% similarity)
- Functions: `write_summary`, `write_header`, `group_by_status`, `count_by_status`, `write_status_section`
- All follow same pattern of string building

**Solution**: Create a builder pattern
```rust
pub struct MarkdownBuilder {
    content: String,
}

impl MarkdownBuilder {
    pub fn add_header(&mut self, title: &str, level: usize) -> &mut Self
    pub fn add_table(&mut self, headers: &[&str], rows: Vec<Vec<String>>) -> &mut Self
    pub fn add_summary(&mut self, tickets: &[Ticket]) -> &mut Self
}
```

### 3. MCP Service Methods (mcp/service.rs, mcp/service_with_events.rs)
**Problem**: Duplicate service initialization and tool registration (95-97% similarity)
- Methods: `new`, `get_tools`, `list_tools`, `server_info`

**Solution**: Use trait with default implementations
```rust
trait MpcServiceBase {
    fn get_storage(&self) -> &FileStorage;
    fn get_project_root(&self) -> &PathBuf;
    
    fn default_tools(&self) -> Vec<Tool> {
        // Common tool registration
    }
    
    fn default_server_info(&self) -> ServerInfo {
        // Common server info
    }
}
```

### 4. Search Handler Functions (mcp/handlers/search.rs)
**Problem**: Export functions are 99% similar
- Functions: `handle_search`, `handle_export`, `handle_import`, `export_to_csv`, `export_to_markdown`

**Solution**: Generic export handler
```rust
trait ExportFormat {
    fn file_extension(&self) -> &str;
    fn content_type(&self) -> &str;
    fn format_tickets(&self, tickets: &[Ticket]) -> Result<String>;
}

fn handle_export_generic<F: ExportFormat>(
    format: F,
    tickets: &[Ticket],
    output_path: Option<String>
) -> Result<String> {
    // Common export logic
}
```

### 5. Task Handler Operations (cli/handlers/task.rs)
**Problem**: `handle_task_uncomplete` and `handle_task_remove` are 99.69% similar
- Both load ticket, find task, modify, and save

**Solution**: Generic task operation
```rust
fn handle_task_operation<F>(
    task_id: String,
    ticket_ref: Option<String>,
    project_dir: Option<String>,
    output: &OutputFormatter,
    operation: F,
) -> Result<()> 
where 
    F: FnOnce(&mut Ticket, usize) -> Result<String>
{
    // Common task operation logic
}
```

## Medium Priority Duplications (85-95% similarity)

### 6. Storage Repository Methods (storage/repository.rs)
**Problem**: `exists`, `find`, and `count` methods share 97-98% similarity
**Solution**: Implement in terms of a common internal method

### 7. Spec Handler Functions (cli/handlers/spec.rs)
**Problem**: `handle_spec_show` and `handle_spec_approve` are 99.17% similar
**Solution**: Extract common spec loading and validation logic

## Implementation Priority

1. **High Priority** (Implement immediately):
   - CLI handler initialization (affects 15+ files)
   - Task operations refactoring (high duplication %)
   
2. **Medium Priority** (Next sprint):
   - Export module builder pattern
   - MCP service trait extraction
   
3. **Low Priority** (As time permits):
   - Storage repository optimization
   - Spec handler consolidation

## Benefits

- **Code Reduction**: ~500-700 lines removed
- **Maintainability**: Single source of truth for common patterns
- **Testing**: Easier to test common functionality once
- **Performance**: Potential for optimization in centralized functions
- **Consistency**: Ensures uniform behavior across handlers

## Risks & Mitigation

- **Risk**: Over-abstraction making code harder to understand
  - **Mitigation**: Keep abstractions simple and well-documented
  
- **Risk**: Breaking existing functionality
  - **Mitigation**: Comprehensive test coverage before refactoring

## Next Steps

1. Create `src/cli/handlers/common.rs` for shared handler utilities
2. Write tests for the new common functions
3. Incrementally refactor handlers one at a time
4. Run integration tests after each refactor
5. Update documentation to reflect new patterns