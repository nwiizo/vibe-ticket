//! Spec-driven development MCP tool handlers

use crate::mcp::handlers::schema_helper::json_to_schema;
use crate::mcp::service::VibeTicketService;
use crate::storage::TicketRepository;
use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{Value, json};
use std::borrow::Cow;
use std::sync::Arc;

/// Register all spec-driven development tools
#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        // Add spec tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_add"),
            description: Some(Cow::Borrowed("Add specifications to a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug"
                    },
                    "spec_type": {
                        "type": "string",
                        "enum": ["requirements", "design", "tasks"],
                        "description": "Type of specification to add"
                    },
                    "content": {
                        "type": "object",
                        "description": "Specification content"
                    }
                },
                "required": ["ticket", "spec_type", "content"]
            }))),
            annotations: None,
        },
        // Update spec tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_update"),
            description: Some(Cow::Borrowed("Update specifications for a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug"
                    },
                    "spec_type": {
                        "type": "string",
                        "enum": ["requirements", "design", "tasks"],
                        "description": "Type of specification to update"
                    },
                    "content": {
                        "type": "object",
                        "description": "Updated specification content"
                    }
                },
                "required": ["ticket", "spec_type", "content"]
            }))),
            annotations: None,
        },
        // Check spec tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_check"),
            description: Some(Cow::Borrowed("Check specification status for a ticket")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "ticket": {
                        "type": "string",
                        "description": "Ticket ID or slug"
                    }
                },
                "required": ["ticket"]
            }))),
            annotations: None,
        },
        // Specify tool - Create specification from natural language
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_specify"),
            description: Some(Cow::Borrowed("Create a specification from natural language requirements (spec-driven development)")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "requirements": {
                        "type": "string",
                        "description": "Natural language requirements description"
                    },
                    "ticket": {
                        "type": "string",
                        "description": "Optional ticket ID or slug to link specification to"
                    },
                    "interactive": {
                        "type": "boolean",
                        "description": "Enable interactive mode for refinement",
                        "default": false
                    }
                },
                "required": ["requirements"]
            }))),
            annotations: None,
        },
        // Plan tool - Generate implementation plan
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_plan"),
            description: Some(Cow::Borrowed("Generate implementation plan from specification")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "spec": {
                        "type": "string",
                        "description": "Specification ID (uses active spec if not provided)"
                    },
                    "tech_stack": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Technology stack for the implementation"
                    },
                    "architecture": {
                        "type": "string",
                        "description": "Architecture pattern (e.g., layered, microservices, event-driven)"
                    }
                },
                "required": []
            }))),
            annotations: None,
        },
        // Generate tasks tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_generate_tasks"),
            description: Some(Cow::Borrowed("Generate executable task list from specification and plan")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "spec": {
                        "type": "string",
                        "description": "Specification ID (uses active spec if not provided)"
                    },
                    "granularity": {
                        "type": "string",
                        "enum": ["fine", "medium", "coarse"],
                        "description": "Task granularity level",
                        "default": "medium"
                    },
                    "parallel": {
                        "type": "boolean",
                        "description": "Mark tasks that can be executed in parallel",
                        "default": false
                    },
                    "export_tickets": {
                        "type": "boolean",
                        "description": "Export tasks as tickets",
                        "default": false
                    }
                },
                "required": []
            }))),
            annotations: None,
        },
        // Validate tool
        Tool {
            name: Cow::Borrowed("vibe-ticket_spec_validate"),
            description: Some(Cow::Borrowed("Validate specification for completeness and ambiguities")),
            input_schema: Arc::new(json_to_schema(json!({
                "type": "object",
                "properties": {
                    "spec": {
                        "type": "string",
                        "description": "Specification ID (uses active spec if not provided)"
                    },
                    "check_ambiguities": {
                        "type": "boolean",
                        "description": "Check for [NEEDS CLARIFICATION] markers",
                        "default": true
                    },
                    "generate_report": {
                        "type": "boolean",
                        "description": "Generate validation report",
                        "default": false
                    }
                },
                "required": []
            }))),
            annotations: None,
        },
    ]
}

/// Handle adding specifications
pub fn handle_add(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
        spec_type: String,
        content: Value,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = crate::mcp::handlers::tickets::resolve_ticket_ref(service, &args.ticket)?;
    let mut ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    // Store specification in metadata
    let spec_key = format!("spec_{}", args.spec_type);
    ticket.metadata.insert(
        spec_key.clone(),
        Value::String(
            serde_json::to_string(&args.content)
                .map_err(|e| format!("Failed to serialize spec: {e}"))?,
        ),
    );

    ticket.metadata.insert(
        format!("{spec_key}_updated_at"),
        Value::String(chrono::Utc::now().to_rfc3339()),
    );

    service
        .storage
        .save(&ticket)
        .map_err(|e| format!("Failed to save ticket: {e}"))?;

    Ok(json!({
        "status": "added",
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "spec_type": args.spec_type,
    }))
}

/// Handle updating specifications
pub fn handle_update(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    // Update uses the same logic as add
    handle_add(service, arguments)
}

/// Handle checking specification status
pub fn handle_check(service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Args {
        ticket: String,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    let ticket_id = crate::mcp::handlers::tickets::resolve_ticket_ref(service, &args.ticket)?;
    let ticket = service
        .storage
        .load(&ticket_id)
        .map_err(|e| format!("Failed to load ticket: {e}"))?;

    let mut specs = json!({});

    // Check for each spec type
    for spec_type in ["requirements", "design", "tasks"] {
        let spec_key = format!("spec_{spec_type}");
        if let Some(spec_json) = ticket.metadata.get(&spec_key) {
            specs[spec_type] = json!({
                "exists": true,
                "updated_at": ticket.metadata.get(&format!("{spec_key}_updated_at")),
                "content": spec_json.as_str().and_then(|s| serde_json::from_str::<Value>(s).ok())
            });
        } else {
            specs[spec_type] = json!({
                "exists": false
            });
        }
    }

    Ok(json!({
        "ticket_id": ticket.id.to_string(),
        "ticket_slug": ticket.slug,
        "specifications": specs
    }))
}

/// Handle creating specification from natural language
pub fn handle_specify(_service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    use crate::specs::{SpecManager, Specification};
    use std::path::PathBuf;
    
    #[derive(Deserialize)]
    struct Args {
        requirements: String,
        ticket: Option<String>,
        #[allow(dead_code)]
        interactive: Option<bool>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    // Get project directory
    let project_dir = PathBuf::from(".vibe-ticket");
    if !project_dir.exists() {
        return Err("Project not initialized. Run 'vibe-ticket init' first.".to_string());
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));
    
    // Extract title from requirements
    let title = args.requirements
        .lines()
        .next()
        .unwrap_or(&args.requirements)
        .chars()
        .take(100)
        .collect::<String>()
        .trim_end_matches('.')
        .to_string();
    
    // Create specification
    let spec = Specification::new(
        title.clone(),
        args.requirements.clone(),
        args.ticket,
        vec!["spec-driven".to_string()],
    );
    
    // Save specification
    spec_manager.save(&spec)
        .map_err(|e| format!("Failed to save specification: {e}"))?;
    
    Ok(json!({
        "status": "created",
        "spec_id": spec.metadata.id,
        "title": spec.metadata.title,
        "description": spec.metadata.description,
        "progress": json!({
            "requirements_completed": spec.metadata.progress.requirements_completed,
            "design_completed": spec.metadata.progress.design_completed,
            "tasks_completed": spec.metadata.progress.tasks_completed
        }),
        "message": "Specification created successfully. Use 'spec_plan' to generate implementation plan."
    }))
}

/// Handle generating implementation plan
pub fn handle_plan(_service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    use crate::specs::SpecManager;
    use std::path::PathBuf;
    
    #[derive(Deserialize)]
    struct Args {
        spec: Option<String>,
        tech_stack: Option<Vec<String>>,
        architecture: Option<String>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    // Get project directory
    let project_dir = PathBuf::from(".vibe-ticket");
    if !project_dir.exists() {
        return Err("Project not initialized. Run 'vibe-ticket init' first.".to_string());
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));
    
    // Get spec ID (from parameter or active spec)
    let spec_id = match args.spec {
        Some(id) => id,
        None => spec_manager.get_active_spec()
            .map_err(|e| format!("Failed to get active spec: {e}"))?
            .ok_or("No active specification. Use 'spec activate' to set one.")?,
    };
    
    // Load specification
    let spec = spec_manager.load(&spec_id)
        .map_err(|e| format!("Failed to load specification: {e}"))?;
    
    // Generate plan document
    let tech_stack_str = args.tech_stack
        .as_ref()
        .map(|ts| ts.join(", "))
        .unwrap_or_else(|| "To be determined".to_string());
    
    let plan_content = format!(
        r#"# Implementation Plan: {}

## Technology Stack
{}

## Architecture Pattern
{}

## Implementation Phases

### Phase 1: Setup and Infrastructure
- Project initialization
- Development environment setup
- Core dependencies installation

### Phase 2: Core Implementation
- Data models
- Business logic
- Core functionality

### Phase 3: Integration and Testing
- Unit tests
- Integration tests
- Validation against requirements

### Phase 4: Documentation and Deployment
- User documentation
- Deployment preparation
- Final review

---
Generated on: {}
"#,
        spec.metadata.title,
        tech_stack_str,
        args.architecture.as_deref().unwrap_or("Layered Architecture"),
        chrono::Utc::now().format("%Y-%m-%d")
    );
    
    // Save plan document
    let spec_dir = spec_manager.get_spec_dir(&spec_id);
    std::fs::write(spec_dir.join("plan.md"), &plan_content)
        .map_err(|e| format!("Failed to save plan: {e}"))?;
    
    Ok(json!({
        "status": "created",
        "spec_id": spec_id,
        "title": spec.metadata.title,
        "tech_stack": args.tech_stack,
        "architecture": args.architecture,
        "message": "Implementation plan created. Use 'spec_generate_tasks' to create task list."
    }))
}

/// Handle generating tasks
pub fn handle_generate_tasks(_service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    use crate::specs::SpecManager;
    use std::path::PathBuf;
    
    #[derive(Deserialize)]
    struct Args {
        spec: Option<String>,
        granularity: Option<String>,
        parallel: Option<bool>,
        export_tickets: Option<bool>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    // Get project directory
    let project_dir = PathBuf::from(".vibe-ticket");
    if !project_dir.exists() {
        return Err("Project not initialized. Run 'vibe-ticket init' first.".to_string());
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));
    
    // Get spec ID
    let spec_id = match args.spec {
        Some(id) => id,
        None => spec_manager.get_active_spec()
            .map_err(|e| format!("Failed to get active spec: {e}"))?
            .ok_or("No active specification. Use 'spec activate' to set one.")?,
    };
    
    // Load specification
    let spec = spec_manager.load(&spec_id)
        .map_err(|e| format!("Failed to load specification: {e}"))?;
    
    let granularity = args.granularity.as_deref().unwrap_or("medium");
    let parallel = args.parallel.unwrap_or(false);
    let task_prefix = if parallel { "[P] " } else { "" };
    
    // Generate tasks document
    let tasks_content = format!(
        r#"# Tasks: {}

## Task Granularity: {}
- Parallel execution: {}

## Phase 1: Setup
- [ ] {}T001: Initialize project structure
- [ ] {}T002: Set up development environment
- [ ] {}T003: Install dependencies

## Phase 2: Implementation
- [ ] {}T004: Implement data models
- [ ] {}T005: Create business logic
- [ ] {}T006: Develop core functionality

## Phase 3: Testing
- [ ] {}T007: Write unit tests
- [ ] {}T008: Create integration tests
- [ ] {}T009: Perform validation

## Phase 4: Deployment
- [ ] {}T010: Prepare documentation
- [ ] {}T011: Configure deployment
- [ ] {}T012: Deploy to production

---
Generated on: {}
"#,
        spec.metadata.title,
        granularity,
        if parallel { "Enabled" } else { "Disabled" },
        task_prefix, task_prefix, task_prefix,
        task_prefix, task_prefix, task_prefix,
        task_prefix, task_prefix, task_prefix,
        task_prefix, task_prefix, task_prefix,
        chrono::Utc::now().format("%Y-%m-%d")
    );
    
    // Save tasks document
    let spec_dir = spec_manager.get_spec_dir(&spec_id);
    std::fs::write(spec_dir.join("tasks.md"), &tasks_content)
        .map_err(|e| format!("Failed to save tasks: {e}"))?;
    
    let mut message = "Task list generated successfully.".to_string();
    
    // Export to tickets if requested
    if args.export_tickets.unwrap_or(false) {
        // TODO: Implement ticket export
        message.push_str(" (Ticket export not yet implemented in MCP)");
    }
    
    Ok(json!({
        "status": "created",
        "spec_id": spec_id,
        "title": spec.metadata.title,
        "granularity": granularity,
        "parallel": parallel,
        "task_count": 12,
        "message": message
    }))
}

/// Handle validating specification
pub fn handle_validate(_service: &VibeTicketService, arguments: Value) -> Result<Value, String> {
    use crate::specs::SpecManager;
    use std::path::PathBuf;
    
    #[derive(Deserialize)]
    struct Args {
        spec: Option<String>,
        check_ambiguities: Option<bool>,
        generate_report: Option<bool>,
    }

    let args: Args =
        serde_json::from_value(arguments).map_err(|e| format!("Invalid arguments: {e}"))?;

    // Get project directory
    let project_dir = PathBuf::from(".vibe-ticket");
    if !project_dir.exists() {
        return Err("Project not initialized. Run 'vibe-ticket init' first.".to_string());
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));
    
    // Get spec ID
    let spec_id = match args.spec {
        Some(id) => id,
        None => spec_manager.get_active_spec()
            .map_err(|e| format!("Failed to get active spec: {e}"))?
            .ok_or("No active specification. Use 'spec activate' to set one.")?,
    };
    
    // Load specification
    let spec = spec_manager.load(&spec_id)
        .map_err(|e| format!("Failed to load specification: {e}"))?;
    
    let mut validation_results = Vec::new();
    let mut has_issues = false;
    
    // Check completeness
    if !spec.metadata.progress.requirements_completed {
        validation_results.push("⚠️  Requirements phase not complete".to_string());
        has_issues = true;
    } else {
        validation_results.push("✅ Requirements phase complete".to_string());
    }
    
    if !spec.metadata.progress.design_completed {
        validation_results.push("⚠️  Design phase not complete".to_string());
        has_issues = true;
    } else {
        validation_results.push("✅ Design phase complete".to_string());
    }
    
    if !spec.metadata.progress.tasks_completed {
        validation_results.push("⚠️  Tasks phase not complete".to_string());
        has_issues = true;
    } else {
        validation_results.push("✅ Tasks phase complete".to_string());
    }
    
    // Check for ambiguities if requested
    if args.check_ambiguities.unwrap_or(true) {
        let spec_dir = spec_manager.get_spec_dir(&spec_id);
        let spec_file = spec_dir.join("spec.md");
        
        if spec_file.exists() {
            let content = std::fs::read_to_string(&spec_file)
                .map_err(|e| format!("Failed to read spec file: {e}"))?;
            
            let clarification_count = content.matches("[NEEDS CLARIFICATION]").count();
            if clarification_count > 0 {
                validation_results.push(format!("⚠️  Found {} items marked as [NEEDS CLARIFICATION]", clarification_count));
                has_issues = true;
            } else {
                validation_results.push("✅ No ambiguities found".to_string());
            }
        }
    }
    
    // Generate report if requested
    if args.generate_report.unwrap_or(false) {
        let report_content = format!(
            r#"# Specification Validation Report

## Specification: {}
## ID: {}

## Progress
- Requirements: {}
- Design: {}
- Tasks: {}

## Validation Results
{}

## Summary
Status: {}

---
Generated on: {}
"#,
            spec.metadata.title,
            spec.metadata.id,
            if spec.metadata.progress.requirements_completed { "✅ Complete" } else { "⚠️ Incomplete" },
            if spec.metadata.progress.design_completed { "✅ Complete" } else { "⚠️ Incomplete" },
            if spec.metadata.progress.tasks_completed { "✅ Complete" } else { "⚠️ Incomplete" },
            validation_results.join("\n"),
            if has_issues { "Has Issues" } else { "Valid" },
            chrono::Utc::now().format("%Y-%m-%d")
        );
        
        let spec_dir = spec_manager.get_spec_dir(&spec_id);
        std::fs::write(spec_dir.join("validation-report.md"), report_content)
            .map_err(|e| format!("Failed to save report: {e}"))?;
    }
    
    Ok(json!({
        "status": if has_issues { "has_issues" } else { "valid" },
        "spec_id": spec_id,
        "title": spec.metadata.title,
        "validation_results": validation_results,
        "has_issues": has_issues,
        "message": if has_issues { 
            "Specification has validation issues that should be addressed" 
        } else { 
            "Specification passed all validation checks" 
        }
    }))
}
