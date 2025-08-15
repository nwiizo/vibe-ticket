# Refactoring Plan - vibe-ticket

Based on similarity-rs analysis performed on 2025-07-21

## Summary

The codebase has significant code duplication that needs refactoring. The analysis found:
- **Critical duplications (>99%)**: Export handlers, specs manager methods
- **High duplications (95-99%)**: Core module methods, error handling patterns
- **Medium duplications (90-95%)**: Test utilities and helper functions

## Refactoring Priority

### 1. Critical Priority - Export Module (99% duplication)

**Files affected**: `src/cli/handlers/export.rs`

**Issues**:
- `handle_export_command` <-> `export_markdown`: 99.06% similarity
- `handle_export_command` <-> `export_csv`: 98.81% similarity  
- `export_markdown` <-> `output_ticket_markdown`: 99.74% similarity

**Refactoring approach**:
1. Extract common export logic into a trait `Exporter`
2. Create specific implementations for each format (CSV, JSON, Markdown)
3. Use template method pattern for shared formatting logic
4. Consolidate ticket output formatting into shared utilities

### 2. Critical Priority - Specs Manager (98-99% duplication)

**Files affected**: `src/specs/manager.rs`

**Issues**:
- Multiple methods with 98-99% similarity
- `save_document`, `load_document`, `save_metadata` have nearly identical patterns
- Test functions duplicate production code patterns

**Refactoring approach**:
1. Extract common file I/O operations into generic methods
2. Use type parameters for different document types
3. Create a `DocumentStore` trait for persistence operations
4. Consolidate error handling patterns

### 3. High Priority - Core Module Methods

**Files affected**: 
- `src/core/priority.rs` (100% duplication in some methods)
- `src/core/task.rs` (97% duplication between constructors)
- `src/core/id.rs` (100% duplication in test functions)

**Issues**:
- `Priority::emoji` <-> `Priority::color`: 100% similarity
- `Task::new` <-> `Task::with_id`: 97.37% similarity
- Multiple test functions are identical

**Refactoring approach**:
1. Use macros for repetitive enum match patterns
2. Consolidate constructors using builder pattern
3. Extract test utilities into shared test module

### 4. Medium Priority - Error Handling

**Files affected**: `src/error.rs`

**Issues**:
- `user_message` <-> `with_context`: 94.81% similarity
- Multiple error methods have similar structure

**Refactoring approach**:
1. Create error builder with fluent interface
2. Use chain of responsibility for error context building
3. Extract common error formatting logic

### 5. Low Priority - Test Utilities

**Files affected**: Multiple test modules

**Issues**:
- Test setup code is duplicated across modules
- Similar assertion patterns repeated

**Refactoring approach**:
1. Create `tests/common/mod.rs` with shared test utilities
2. Use test fixtures for common setup
3. Create assertion helper macros

## Implementation Plan

### Phase 1: Export Module Refactoring (Week 1)
- [ ] Design `Exporter` trait
- [ ] Implement format-specific exporters
- [ ] Migrate existing export functions
- [ ] Update tests

### Phase 2: Specs Manager Refactoring (Week 2)
- [ ] Extract `DocumentStore` trait
- [ ] Implement generic document operations
- [ ] Consolidate file I/O logic
- [ ] Update dependent code

### Phase 3: Core Module Cleanup (Week 3)
- [ ] Implement builder pattern for Task
- [ ] Create macros for Priority methods
- [ ] Extract shared test utilities
- [ ] Update all tests

### Phase 4: Error Handling Improvements (Week 4)
- [ ] Design error builder API
- [ ] Implement fluent error construction
- [ ] Migrate existing error handling
- [ ] Add comprehensive error tests

## Success Metrics

- Reduce total code duplication by >50%
- Improve test coverage to >90%
- Decrease build time by consolidating duplicates
- Improve maintainability score

## Risks and Mitigations

1. **Risk**: Breaking existing functionality
   - **Mitigation**: Comprehensive test coverage before refactoring
   
2. **Risk**: API changes affecting users
   - **Mitigation**: Maintain backward compatibility, deprecate old APIs gradually

3. **Risk**: Performance regression
   - **Mitigation**: Benchmark critical paths before and after changes

## Notes

- Use `cargo clippy` and `cargo fmt` after each refactoring step
- Run `similarity-rs` periodically to track progress
- Update documentation as APIs change
- Consider creating a `CHANGELOG.md` for tracking changes