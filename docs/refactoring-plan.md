# Code Duplication Refactoring Plan

Based on semantic similarity analysis using `similarity-rs`, we've identified significant code duplication patterns that should be refactored to improve maintainability and reduce technical debt.

## Critical Duplication Patterns (>90% similarity)

### 1. **Spec Handler Functions (92-95% similarity)**
**Location**: `src/cli/handlers/spec.rs`
- Multiple handler functions share 90%+ similar code structure
- Functions affected:
  - `handle_spec_init`, `handle_spec_status`, `handle_spec_list`, `handle_spec_show`
  - `handle_spec_tasks`, `handle_spec_validate`
  - `handle_spec_delete`, `handle_spec_activate`, `handle_spec_approve`

**Refactoring Strategy**:
```rust
// Create a base trait for spec operations
trait SpecOperation {
    fn validate_project(&self) -> Result<(PathBuf, SpecManager)>;
    fn load_spec(&self, spec_id: Option<String>) -> Result<Specification>;
    fn format_output(&self, spec: &Specification, formatter: &OutputFormatter);
}

// Extract common initialization pattern
fn init_spec_context(project_dir: Option<&str>) -> Result<(PathBuf, SpecManager)> {
    // Common project directory setup
    // Common spec manager initialization
}
```

### 2. **Task Handler Functions (94-99% similarity)**
**Location**: `src/cli/handlers/task.rs`
- Task operations share nearly identical code
- Functions affected:
  - `handle_task_complete` and `handle_task_uncomplete` (99% similar!)
  - `handle_task_add`, `handle_task_list` (95% similar)

**Refactoring Strategy**:
```rust
// Create generic task operation handler
fn handle_task_operation<F>(
    task_ref: Option<String>,
    project_dir: Option<&str>,
    operation: F,
    formatter: &OutputFormatter,
) -> Result<()>
where
    F: FnOnce(&mut Task, &mut Ticket) -> Result<()>
{
    // Common task loading and validation
    // Apply operation
    // Common save and output
}
```

### 3. **MCP Tool Creation Functions (97-98% similarity)**
**Location**: `src/mcp/handlers/tickets.rs`
- Tool creation functions are nearly identical
- Functions affected:
  - `create_new_ticket_tool`, `create_list_tickets_tool`, `create_edit_ticket_tool`
  - `create_close_ticket_tool`, `create_start_ticket_tool`, `create_check_status_tool`

**Refactoring Strategy**:
```rust
// Use a builder pattern for tool creation
struct ToolBuilder {
    name: String,
    description: String,
    parameters: Vec<Parameter>,
}

impl ToolBuilder {
    fn build_ticket_tool(operation: &str) -> Tool {
        // Common tool structure with operation-specific parameters
    }
}
```

### 4. **Search Functions (96% similarity)**
**Location**: `src/cli/handlers/search.rs`
- `search_ticket_regex` and `search_ticket_text` are nearly identical

**Refactoring Strategy**:
```rust
// Create a generic search function with strategy pattern
enum SearchStrategy {
    Regex(Regex),
    Text(String),
}

fn search_tickets(tickets: &[Ticket], strategy: SearchStrategy) -> Vec<Ticket> {
    // Common search logic with strategy-specific matching
}
```

## Medium Priority Duplications (85-90% similarity)

### 5. **Common Initialization Pattern**
**Repeated in**: All handler files
- Project directory setup
- Storage initialization
- Error handling

**Refactoring Strategy**:
```rust
// Create a HandlerContext struct
pub struct HandlerContext {
    pub project_root: PathBuf,
    pub storage: FileStorage,
    pub formatter: OutputFormatter,
}

impl HandlerContext {
    pub fn new(project_dir: Option<&str>, formatter: OutputFormatter) -> Result<Self> {
        // Common initialization logic
    }
}
```

### 6. **Test Setup Functions**
**Location**: Multiple test modules
- Test fixture creation is duplicated

**Refactoring Strategy**:
```rust
// Create test utilities module
mod test_utils {
    pub fn setup_test_project() -> (TempDir, FileStorage) {
        // Common test setup
    }
    
    pub fn create_test_ticket(title: &str) -> Ticket {
        // Common ticket creation for tests
    }
}
```

## Implementation Plan

### Phase 1: Extract Common Patterns (Week 1)
1. Create `src/cli/handlers/base.rs` with `HandlerContext`
2. Create `src/cli/handlers/spec_base.rs` for spec operations
3. Create `src/test_utils.rs` for test fixtures

### Phase 2: Refactor High Similarity Functions (Week 2)
1. Refactor task handlers to use generic operation handler
2. Refactor MCP tool creation to use builder pattern
3. Consolidate search functions

### Phase 3: Refactor Spec Handlers (Week 3)
1. Extract common spec handler logic
2. Implement SpecOperation trait
3. Refactor all spec handlers to use common base

### Phase 4: Testing and Validation (Week 4)
1. Ensure all tests pass
2. Add integration tests for refactored code
3. Performance testing to ensure no regression

## Benefits

1. **Code Reduction**: Estimated 30-40% reduction in lines of code
2. **Maintainability**: Single source of truth for common operations
3. **Consistency**: Uniform error handling and validation
4. **Testability**: Easier to test common logic once
5. **Bug Prevention**: Fix once, apply everywhere

## Metrics

### Before Refactoring:
- Total functions with >90% similarity: 47
- Total functions with >85% similarity: 83
- Estimated duplicate lines: ~2,500

### After Refactoring (Target):
- Functions with >90% similarity: <5
- Functions with >85% similarity: <20
- Lines saved: ~1,500

## Risk Mitigation

1. **Incremental Refactoring**: Refactor one module at a time
2. **Comprehensive Testing**: Add tests before refactoring
3. **Feature Flags**: Use feature flags for gradual rollout
4. **Code Review**: Each refactoring PR should be thoroughly reviewed
5. **Performance Monitoring**: Benchmark before and after

## Next Steps

1. Review and approve this plan
2. Create tracking tickets for each phase
3. Begin with Phase 1 extraction of common patterns
4. Set up metrics tracking for code quality improvement