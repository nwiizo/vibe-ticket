# Vibe-Ticket Slash Commands for Claude Code

This document describes the slash commands available for vibe-ticket when using Claude Code with MCP integration.

## Prerequisites

1. Install vibe-ticket with MCP support:
```bash
cargo install vibe-ticket --features mcp
```

2. Configure Claude Code to use vibe-ticket MCP:
```bash
claude mcp add vibe-ticket ~/.cargo/bin/vibe-ticket --scope project -- mcp serve
```

## Available Slash Commands

### Spec-Driven Development Commands

#### `/specify` - Create Specification from Natural Language
Create a specification from natural language requirements using AI-assisted refinement.

**Usage:**
```
/specify "I want to build a REST API for user management"
```

**MCP Tool:** `vibe-ticket_spec_specify`
**Parameters:**
- `requirements` (required): Natural language description of what you want to build
- `ticket` (optional): Link specification to an existing ticket
- `interactive` (optional): Enable interactive refinement mode

**Example:**
```javascript
// Using MCP tool directly
await callTool('vibe-ticket_spec_specify', {
  requirements: "Build a REST API for user management with authentication",
  interactive: false
});
```

#### `/plan` - Generate Implementation Plan
Generate a detailed implementation plan from a specification.

**Usage:**
```
/plan --tech-stack rust,actix-web,postgresql --architecture microservices
```

**MCP Tool:** `vibe-ticket_spec_plan`
**Parameters:**
- `spec` (optional): Specification ID (uses active spec if not provided)
- `tech_stack` (optional): Array of technologies to use
- `architecture` (optional): Architecture pattern (layered, microservices, event-driven)

**Example:**
```javascript
await callTool('vibe-ticket_spec_plan', {
  tech_stack: ["rust", "actix-web", "postgresql"],
  architecture: "microservices"
});
```

#### `/tasks` - Generate Executable Task List
Create an executable task list from specification and plan.

**Usage:**
```
/tasks --granularity fine --parallel --export-tickets
```

**MCP Tool:** `vibe-ticket_spec_generate_tasks`
**Parameters:**
- `spec` (optional): Specification ID
- `granularity` (optional): Task detail level (fine, medium, coarse)
- `parallel` (optional): Mark tasks that can run in parallel
- `export_tickets` (optional): Export tasks as tickets

**Example:**
```javascript
await callTool('vibe-ticket_spec_generate_tasks', {
  granularity: "fine",
  parallel: true,
  export_tickets: true
});
```

#### `/validate` - Validate Specification
Check specification for completeness and ambiguities.

**Usage:**
```
/validate --check-ambiguities --generate-report
```

**MCP Tool:** `vibe-ticket_spec_validate`
**Parameters:**
- `spec` (optional): Specification ID
- `check_ambiguities` (optional): Check for [NEEDS CLARIFICATION] markers
- `generate_report` (optional): Generate validation report

**Example:**
```javascript
await callTool('vibe-ticket_spec_validate', {
  check_ambiguities: true,
  generate_report: true
});
```

### Existing Spec Management Commands

#### `/spec add` - Add Specification to Ticket
**MCP Tool:** `vibe-ticket_spec_add`

#### `/spec update` - Update Specification
**MCP Tool:** `vibe-ticket_spec_update`

#### `/spec check` - Check Specification Status
**MCP Tool:** `vibe-ticket_spec_check`

## Workflow Example

Here's a complete workflow using slash commands in Claude Code:

```bash
# 1. Create a specification from requirements
/specify "Build a CLI tool for managing todos with categories and priorities"

# 2. Generate implementation plan
/plan --tech-stack rust,clap,serde --architecture layered

# 3. Create executable tasks
/tasks --granularity medium --parallel

# 4. Validate the specification
/validate --check-ambiguities --generate-report

# 5. Export tasks as tickets (optional)
/tasks --export-tickets
```

## Natural Language Usage in Claude Code

When using Claude Code, you can also use natural language to invoke these commands:

- "Create a specification for a REST API"
- "Generate an implementation plan using Rust and PostgreSQL"
- "Create fine-grained tasks that can run in parallel"
- "Validate my specification and check for ambiguities"

Claude Code will automatically map these requests to the appropriate MCP tools.

## Configuration

The spec-driven development features respect the following configuration:

```yaml
# .vibe-ticket/config.yaml
spec:
  default_granularity: medium
  enable_parallel_by_default: false
  auto_export_tickets: false
  templates_dir: templates/
```

## Templates

Templates are stored in the `templates/` directory:
- `spec-template.md` - Specification template
- `plan-template.md` - Implementation plan template
- `task-template.md` - Task list template

Generate templates using:
```bash
vibe-ticket spec template all
```

## Troubleshooting

### MCP Connection Issues
If MCP tools are not available:
1. Check MCP server is running: `vibe-ticket mcp serve`
2. Verify Claude Code configuration: `claude mcp list`
3. Restart Claude Code

### Specification Not Found
If spec commands can't find specifications:
1. Check active spec: `vibe-ticket spec status`
2. List all specs: `vibe-ticket spec list`
3. Set active spec: `vibe-ticket spec activate <spec-id>`

### Template Issues
If templates are not loading:
1. Generate templates: `vibe-ticket spec template all`
2. Check templates directory exists
3. Verify template files are present

## Best Practices

1. **Start with Clear Requirements**: The better your initial requirements, the better the generated specification
2. **Use [NEEDS CLARIFICATION]**: Mark ambiguous requirements for later refinement
3. **Iterate on Specifications**: Use `/validate` frequently to check progress
4. **Leverage Parallel Tasks**: Use `--parallel` flag for tasks that can run concurrently
5. **Export to Tickets**: Use `--export-tickets` to create trackable work items

## Integration with Existing Vibe-Ticket Features

The spec-driven development features integrate seamlessly with existing vibe-ticket functionality:

- Specifications can be linked to tickets
- Tasks generated from specs can be exported as tickets
- Git worktrees can be created for spec-based work
- All standard ticket management features apply

## Future Enhancements

Planned improvements for slash commands:
- AI-powered requirement analysis
- Automatic ambiguity detection
- Smart task dependency resolution
- Integration with CI/CD pipelines
- Real-time collaboration features

---

For more information, see the main [vibe-ticket documentation](README.md) or run `vibe-ticket spec --help`.