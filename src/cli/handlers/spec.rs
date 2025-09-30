//! Handlers for spec-driven development commands
//!
//! This module implements all handlers for specification management commands,
//! supporting the three-phase spec-driven development workflow.

use crate::cli::output::OutputFormatter;
use crate::error::{ErrorContext, Result, VibeTicketError};
use crate::specs::{
    SpecDocumentType, SpecManager, SpecPhase, SpecTemplate, Specification, TemplateEngine,
};
use chrono::Utc;
use std::env;
use std::fs;
use std::path::Path;

/// Handle spec init command
pub fn handle_spec_init(
    title: &str,
    description: Option<&str>,
    ticket: Option<&str>,
    tags: Option<&str>,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));

    // Parse tags
    let tag_list: Vec<String> = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // Create new specification
    let spec = Specification::new(
        title.to_string(),
        description.unwrap_or_default().to_string(),
        ticket.map(std::string::ToString::to_string),
        tag_list,
    );

    // Save specification
    spec_manager.save(&spec)?;

    formatter.success(&format!(
        "Created new specification '{}' with ID: {}",
        title, spec.metadata.id
    ));

    if formatter.is_json() {
        formatter.json(&serde_json::json!({
            "status": "success",
            "spec_id": spec.metadata.id,
            "title": title,
            "description": description,
            "ticket_id": spec.metadata.ticket_id,
            "tags": spec.metadata.tags,
        }))?;
    } else {
        formatter.info(&format!("Specification ID: {}", spec.metadata.id));
        if let Some(desc) = description {
            formatter.info(&format!("Description: {desc}"));
        }
        if let Some(ticket_id) = &spec.metadata.ticket_id {
            formatter.info(&format!("Associated ticket: {ticket_id}"));
        }
        formatter.info("\nNext steps:");
        formatter.info("  1. Define requirements: vibe-ticket spec requirements");
        formatter.info("  2. Create design: vibe-ticket spec design");
        formatter.info("  3. Plan tasks: vibe-ticket spec tasks");
    }

    Ok(())
}

/// Handle spec requirements command  
pub fn handle_spec_requirements(
    spec_id: String,
    editor: Option<String>,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use super::spec_common::{RequirementsHandler, SpecPhaseHandler};
    let handler = RequirementsHandler;
    handler.handle_phase_operation(spec_id, editor, project, formatter)
}

/// Handle spec design command
pub fn handle_spec_design(
    spec: Option<String>,
    editor: bool,
    complete: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    use super::spec_common::{DesignHandler, SpecPhaseHandler};

    // If using the simplified phase handler
    if let Some(spec_id) = spec.as_ref() {
        if !complete && !editor {
            let handler = DesignHandler;
            return handler.handle_phase_operation(spec_id.clone(), None, project, formatter);
        }
    }

    // Keep existing complex logic for backward compatibility
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));

    // Get spec ID (from parameter or active spec)
    let spec_id = match spec {
        Some(id) => id,
        None => get_active_spec(&project_dir)?,
    };

    // Load specification
    let mut specification = spec_manager.load(&spec_id)?;

    // Check if requirements are complete
    if !specification.metadata.progress.requirements_completed {
        formatter.warning("Requirements phase is not complete. Consider completing it first.");
    }

    if complete {
        // Mark design phase as complete
        specification.metadata.progress.design_completed = true;
        specification.metadata.updated_at = Utc::now();
        spec_manager.save(&specification)?;

        formatter.success(&format!(
            "Marked design phase as complete for spec '{}'",
            specification.metadata.title
        ));
        return Ok(());
    }

    // Get or create design document
    let doc_path = spec_manager.get_document_path(&spec_id, SpecDocumentType::Design);

    if !doc_path.exists() {
        // Create from template with requirements summary
        let requirements_path =
            spec_manager.get_document_path(&spec_id, SpecDocumentType::Requirements);
        let requirements_summary = if requirements_path.exists() {
            // Extract summary from requirements doc
            "See requirements document for details."
        } else {
            "Requirements not yet defined."
        };

        let mut engine = TemplateEngine::new();
        engine.set_variable("spec_id", &spec_id);

        let template = SpecTemplate::for_document_type(
            SpecDocumentType::Design,
            specification.metadata.title,
            Some(requirements_summary.to_string()),
        );

        let content = engine.generate(&template);
        fs::write(&doc_path, content).context("Failed to create design document")?;

        formatter.info(&format!("Created design document: {}", doc_path.display()));
    }

    if editor {
        // Open in editor
        open_in_editor(&doc_path)?;
        formatter.success("Design document saved");
    } else {
        // Display content
        let content = fs::read_to_string(&doc_path).context("Failed to read design document")?;
        formatter.info(&content);
    }

    Ok(())
}

/// Handle spec tasks command
#[allow(clippy::too_many_arguments)]
pub fn handle_spec_tasks(
    spec: Option<String>,
    plan: Option<String>,
    editor: bool,
    complete: bool,
    export_tickets: bool,
    parallel: bool,
    granularity: String,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));

    // Get spec ID (from parameter or active spec)
    let spec_id = match spec {
        Some(id) => id,
        None => get_active_spec(&project_dir)?,
    };

    // Load specification
    let mut specification = spec_manager.load(&spec_id)?;

    // Check if design is complete
    if !specification.metadata.progress.design_completed {
        formatter.warning("Design phase is not complete. Consider completing it first.");
    }

    if complete {
        // Mark tasks phase as complete
        specification.metadata.progress.tasks_completed = true;
        specification.metadata.updated_at = Utc::now();
        spec_manager.save(&specification)?;

        formatter.success(&format!(
            "Marked tasks phase as complete for spec '{}'",
            specification.metadata.title
        ));
        return Ok(());
    }

    // Get spec directory
    let spec_dir = project_dir
        .join(".vibe-ticket")
        .join("specs")
        .join(&spec_id);

    // Get or create tasks document
    let doc_path = spec_dir.join("tasks.md");

    if !doc_path.exists() {
        // Read plan document if it exists
        let plan_path = if let Some(p) = plan {
            Path::new(&p).to_path_buf()
        } else {
            spec_dir.join("plan.md")
        };

        let plan_content = if plan_path.exists() {
            fs::read_to_string(&plan_path)?
        } else {
            "No plan document found. Creating tasks based on specification.".to_string()
        };

        // Generate tasks based on plan and granularity
        let tasks_content = generate_tasks_document(
            &specification.metadata.title,
            &plan_content,
            &granularity,
            parallel,
        );

        fs::write(&doc_path, tasks_content).context("Failed to create tasks document")?;

        formatter.info(&format!("Created tasks document: {}", doc_path.display()));
    }

    if export_tickets {
        // Export tasks to tickets
        export_tasks_to_tickets(&doc_path, &specification, &project_dir, formatter)?;
    }

    if editor {
        // Open in editor
        open_in_editor(&doc_path)?;
        formatter.success("Tasks document saved");
    } else {
        // Display content
        let content = fs::read_to_string(&doc_path).context("Failed to read tasks document")?;
        formatter.info(&content);
    }

    Ok(())
}

/// Handle spec specify command - create specification from natural language requirements
pub fn handle_spec_specify(
    requirements: &str,
    ticket: Option<&str>,
    interactive: bool,
    _template: &str,
    output: Option<&str>,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));

    // Create specification from requirements
    let title = extract_title_from_requirements(requirements);
    let spec = Specification::new(
        title.clone(),
        requirements.to_string(),
        ticket.map(std::string::ToString::to_string),
        vec!["spec-driven".to_string()],
    );

    // Save initial specification
    spec_manager.save(&spec)?;

    // Determine output directory
    let output_dir = if let Some(out) = output {
        Path::new(out).to_path_buf()
    } else {
        project_dir
            .join(".vibe-ticket")
            .join("specs")
            .join(&spec.metadata.id)
    };

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    // Generate specification document from template
    let mut engine = TemplateEngine::new();
    engine.set_variable("title", &title);
    engine.set_variable("requirements", requirements);
    engine.set_variable("spec_id", &spec.metadata.id);
    engine.set_variable("created_date", &Utc::now().format("%Y-%m-%d").to_string());

    // Create template and generate content
    let spec_template = SpecTemplate::Requirements {
        title: title.clone(),
        description: requirements.to_string(),
    };
    let spec_content = engine.generate(&spec_template);

    // Mark requirements with [NEEDS CLARIFICATION] where ambiguous
    let analyzed_content = analyze_and_mark_ambiguities(&spec_content);

    // Save specification document
    let spec_file = output_dir.join("spec.md");
    fs::write(&spec_file, &analyzed_content)?;

    formatter.success(&format!(
        "Created specification '{}' with ID: {}",
        title, spec.metadata.id
    ));
    formatter.info(&format!("Specification saved to: {}", spec_file.display()));

    if interactive {
        formatter.info("\n💡 Interactive refinement mode:");
        formatter.info("Review the specification and provide clarifications for marked items.");
        formatter.info(
            "The specification contains [NEEDS CLARIFICATION] markers for ambiguous requirements.",
        );

        // Open in editor for refinement
        if let Ok(editor) = env::var("EDITOR") {
            formatter.info(&format!(
                "\nOpening specification in {editor} for refinement..."
            ));
            open_in_editor(&spec_file)?;
        }
    }

    // Check for clarification markers
    let clarification_count = analyzed_content.matches("[NEEDS CLARIFICATION]").count();
    if clarification_count > 0 {
        formatter.warning(&format!(
            "\n⚠️  Found {clarification_count} items that need clarification"
        ));
        formatter.info("Next steps:");
        formatter.info("  1. Review and clarify ambiguous requirements");
        formatter.info("  2. Create implementation plan: vibe-ticket spec plan");
        formatter.info("  3. Generate tasks: vibe-ticket spec tasks");
    } else {
        formatter.info("\n✅ Specification is complete and ready for planning");
        formatter.info("Next steps:");
        formatter.info("  1. Create implementation plan: vibe-ticket spec plan");
        formatter.info("  2. Generate tasks: vibe-ticket spec tasks");
    }

    Ok(())
}

/// Handle spec plan command - create implementation plan from specification
pub fn handle_spec_plan(
    spec: Option<String>,
    tech_stack: Option<String>,
    architecture: Option<String>,
    editor: bool,
    output: Option<String>,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));

    // Get spec ID (from parameter or active spec)
    let spec_id = match spec {
        Some(id) => id,
        None => get_active_spec(&project_dir)?,
    };

    // Load specification
    let mut specification = spec_manager.load(&spec_id)?;

    // Check if requirements are complete
    if !specification.metadata.progress.requirements_completed {
        formatter.warning("⚠️  Requirements phase is not complete. Consider completing it first.");
    }

    // Determine output directory
    let output_dir = if let Some(ref out) = output {
        Path::new(out).to_path_buf()
    } else {
        project_dir
            .join(".vibe-ticket")
            .join("specs")
            .join(&spec_id)
    };

    // Read specification document
    let spec_file = output_dir.join("spec.md");
    let spec_content = if spec_file.exists() {
        fs::read_to_string(&spec_file)?
    } else {
        specification.metadata.description.clone()
    };

    // Parse tech stack
    let tech_list: Vec<String> = tech_stack
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // Generate implementation plan
    let mut engine = TemplateEngine::new();
    engine.set_variable("spec_id", &spec_id);
    engine.set_variable("title", &specification.metadata.title);
    engine.set_variable("tech_stack", &tech_list.join(", "));
    engine.set_variable("architecture", architecture.as_deref().unwrap_or("layered"));

    // Create research document
    let research_content =
        generate_research_document(&spec_content, &tech_list, architecture.as_deref());
    let research_file = output_dir.join("research.md");
    fs::write(&research_file, research_content)?;

    // Create data model
    let data_model_content = generate_data_model(&spec_content, &tech_list);
    let data_model_file = output_dir.join("data-model.md");
    fs::write(&data_model_file, data_model_content)?;

    // Create implementation plan
    let plan_content =
        generate_implementation_plan(&spec_content, &tech_list, architecture.as_deref());
    let plan_file = output_dir.join("plan.md");
    fs::write(&plan_file, plan_content)?;

    // Update specification progress
    specification.metadata.progress.design_completed = true;
    specification.metadata.updated_at = Utc::now();
    spec_manager.save(&specification)?;

    formatter.success(&format!(
        "Created implementation plan for specification '{}'",
        specification.metadata.title
    ));
    formatter.info(&format!("Plan saved to: {}", plan_file.display()));
    formatter.info(&format!("Research saved to: {}", research_file.display()));
    formatter.info(&format!(
        "Data model saved to: {}",
        data_model_file.display()
    ));

    if editor {
        formatter.info("\nOpening plan in editor for refinement...");
        open_in_editor(&plan_file)?;
    }

    formatter.info("\n✅ Implementation plan is ready");
    formatter.info("Next step: Generate executable tasks with 'vibe-ticket spec tasks'");

    Ok(())
}

/// Handle spec validate command
pub fn handle_spec_validate(
    spec: Option<String>,
    complete: bool,
    ambiguities: bool,
    report: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));

    // Get spec ID (from parameter or active spec)
    let spec_id = match spec {
        Some(id) => id,
        None => get_active_spec(&project_dir)?,
    };

    // Load specification
    let specification = spec_manager.load(&spec_id)?;
    let spec_dir = project_dir
        .join(".vibe-ticket")
        .join("specs")
        .join(&spec_id);

    let mut validation_results: Vec<String> = Vec::new();
    let mut has_errors = false;

    // Check completeness
    if complete || !ambiguities && !report {
        // Check all required documents exist
        let spec_file = spec_dir.join("spec.md");
        if spec_file.exists() {
            validation_results.push("✅ Specification document exists".to_string());
        } else {
            validation_results.push("❌ Missing specification document (spec.md)".to_string());
            has_errors = true;
        }

        // Check progress
        if specification.metadata.progress.requirements_completed {
            validation_results.push("✅ Requirements phase complete".to_string());
        } else {
            validation_results.push("⚠️  Requirements phase not marked as complete".to_string());
        }

        if specification.metadata.progress.design_completed {
            validation_results.push("✅ Design phase complete".to_string());
        } else {
            validation_results.push("⚠️  Design phase not marked as complete".to_string());
        }

        if specification.metadata.progress.tasks_completed {
            validation_results.push("✅ Tasks phase complete".to_string());
        } else {
            validation_results.push("⚠️  Tasks phase not marked as complete".to_string());
        }
    }

    // Check for ambiguities
    if ambiguities || !report {
        let spec_file = spec_dir.join("spec.md");
        if spec_file.exists() {
            let content = fs::read_to_string(&spec_file)?;
            let clarification_count = content.matches("[NEEDS CLARIFICATION]").count();

            if clarification_count > 0 {
                validation_results.push(format!(
                    "⚠️  Found {clarification_count} items marked as [NEEDS CLARIFICATION]"
                ));
                has_errors = true;
            } else {
                validation_results.push("✅ No ambiguities found".to_string());
            }
        }
    }

    // Generate report
    if report {
        let validation_refs: Vec<&str> = validation_results.iter().map(|s| s.as_str()).collect();
        let report_content = generate_validation_report(&specification, &validation_refs);
        let report_file = spec_dir.join("validation-report.md");
        fs::write(&report_file, &report_content)?;
        formatter.info(&format!(
            "Validation report saved to: {}",
            report_file.display()
        ));
    }

    // Display results
    formatter.info(&format!(
        "Validation Results for '{}' ({})",
        specification.metadata.title, spec_id
    ));
    formatter.info("");

    for result in &validation_results {
        formatter.info(result);
    }

    if has_errors {
        formatter.warning("\n⚠️  Specification has validation issues that should be addressed");
    } else {
        formatter.success("\n✅ Specification passed all validation checks");
    }

    Ok(())
}

/// Handle spec template command
pub fn handle_spec_template(
    template_type: &str,
    output: &str,
    force: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let output_dir = Path::new(output);

    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let templates_to_create = match template_type {
        "all" => vec!["spec", "plan", "task"],
        t => vec![t],
    };

    for template in templates_to_create {
        let template_file = output_dir.join(format!("{template}-template.md"));

        if template_file.exists() && !force {
            formatter.warning(&format!(
                "Template {} already exists. Use --force to overwrite.",
                template_file.display()
            ));
            continue;
        }

        let content = match template {
            "spec" => include_str!("../../../templates/spec-template.md"),
            "plan" => include_str!("../../../templates/plan-template.md"),
            "task" => include_str!("../../../templates/task-template.md"),
            _ => {
                formatter.warning(&format!("Unknown template type: {template}"));
                continue;
            },
        };

        fs::write(&template_file, content)?;
        formatter.success(&format!("Created template: {}", template_file.display()));
    }

    formatter.info(&format!(
        "\n✅ Templates created in: {}",
        output_dir.display()
    ));

    Ok(())
}

// Helper functions

#[allow(dead_code)]
fn extract_title_from_requirements(requirements: &str) -> String {
    // Extract first line or first sentence as title
    requirements
        .lines()
        .next()
        .unwrap_or("New Specification")
        .trim()
        .trim_end_matches('.')
        .to_string()
}

#[allow(dead_code)]
fn load_specification_template(template_name: &str) -> Result<String> {
    // For now, use embedded template
    let template = match template_name {
        "standard" => include_str!("../../../templates/spec-template.md"),
        _ => include_str!("../../../templates/spec-template.md"),
    };
    Ok(template.to_string())
}

#[allow(dead_code)]
fn analyze_and_mark_ambiguities(content: &str) -> String {
    // Simple heuristic: mark vague terms and missing details
    let mut result = content.to_string();

    let vague_terms = [
        "various",
        "multiple",
        "several",
        "many",
        "some",
        "appropriate",
        "suitable",
        "proper",
        "adequate",
        "fast",
        "slow",
        "quick",
        "efficient",
        "user-friendly",
        "intuitive",
        "easy",
    ];

    for term in &vague_terms {
        result = result.replace(
            term,
            &format!("{term} [NEEDS CLARIFICATION: Be more specific]"),
        );
    }

    result
}

#[allow(dead_code)]
fn generate_research_document(
    spec_content: &str,
    tech_stack: &[String],
    architecture: Option<&str>,
) -> String {
    let tech_stack_str = if tech_stack.is_empty() {
        "- No specific technology stack defined".to_string()
    } else {
        tech_stack
            .iter()
            .map(|t| format!("- {t}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r"# Research and Technical Analysis

## Specification Overview
{}

## Technology Stack Analysis
{}

## Architecture Pattern
{}

## Technical Considerations
- Performance requirements
- Scalability needs
- Security requirements
- Integration points

## Dependencies
{}

## Risk Assessment
- Technical risks
- Implementation challenges
- Mitigation strategies

---
Generated on: {}
",
        spec_content.lines().take(5).collect::<Vec<_>>().join("\n"),
        tech_stack_str,
        architecture.unwrap_or("Layered Architecture"),
        if tech_stack.is_empty() {
            "To be determined"
        } else {
            "Based on selected technology stack"
        },
        Utc::now().format("%Y-%m-%d")
    )
}

#[allow(dead_code)]
fn generate_data_model(_spec_content: &str, tech_stack: &[String]) -> String {
    let is_rust = tech_stack.iter().any(|t| t.to_lowercase().contains("rust"));

    format!(
        r"# Data Model

## Core Entities

{}

## Relationships

- One-to-many relationships
- Many-to-many relationships
- Aggregations

## Validation Rules

- Required fields
- Format validations
- Business rules

## Data Types

{}

---
Generated on: {}
",
        "Extract entities from specification...",
        if is_rust {
            "Using Rust type system with strong typing"
        } else {
            "Define appropriate data types for chosen technology"
        },
        Utc::now().format("%Y-%m-%d")
    )
}

#[allow(dead_code)]
fn generate_implementation_plan(
    _spec_content: &str,
    tech_stack: &[String],
    architecture: Option<&str>,
) -> String {
    let tech_stack_str = if tech_stack.is_empty() {
        "To be determined".to_string()
    } else {
        tech_stack.join(", ")
    };

    format!(
        r"# Implementation Plan

## Overview
Implementation plan based on specification and selected technology stack.

## Technology Stack
{}

## Architecture
{}

## Implementation Phases

### Phase 1: Setup and Infrastructure
- Project initialization
- Development environment setup
- Core dependencies installation
- Basic project structure

### Phase 2: Core Implementation
- Data models
- Business logic
- Core functionality

### Phase 3: Integration
- External services
- APIs
- Database connections

### Phase 4: Testing and Validation
- Unit tests
- Integration tests
- Validation against requirements

### Phase 5: Documentation and Deployment
- User documentation
- Deployment preparation
- Final review

## Timeline
- Estimated completion: TBD

---
Generated on: {}
",
        tech_stack_str,
        architecture.unwrap_or("Layered Architecture"),
        Utc::now().format("%Y-%m-%d")
    )
}

#[allow(dead_code)]
fn generate_validation_report(spec: &Specification, results: &[&str]) -> String {
    format!(
        r"# Specification Validation Report

## Specification Details
- **ID**: {}
- **Title**: {}
- **Created**: {}
- **Updated**: {}

## Validation Results

{}

## Progress Status
- Requirements: {}
- Design: {}
- Tasks: {}

## Recommendations
{}

---
Generated on: {}
",
        spec.metadata.id,
        spec.metadata.title,
        spec.metadata.created_at.format("%Y-%m-%d"),
        spec.metadata.updated_at.format("%Y-%m-%d"),
        results.join("\n"),
        if spec.metadata.progress.requirements_completed {
            "✅ Complete"
        } else {
            "⚠️ In Progress"
        },
        if spec.metadata.progress.design_completed {
            "✅ Complete"
        } else {
            "⚠️ In Progress"
        },
        if spec.metadata.progress.tasks_completed {
            "✅ Complete"
        } else {
            "⚠️ In Progress"
        },
        if results.iter().any(|r| r.contains("❌") || r.contains("⚠️")) {
            "Address identified issues before proceeding to next phase"
        } else {
            "Specification is ready for implementation"
        },
        Utc::now().format("%Y-%m-%d")
    )
}

fn generate_tasks_document(
    title: &str,
    plan_content: &str,
    granularity: &str,
    parallel: bool,
) -> String {
    let task_prefix = if parallel { "[P] " } else { "" };

    // Determine task detail level based on granularity
    let (task_count, _task_detail) = match granularity {
        "fine" => (20, "Detailed implementation steps"),
        "coarse" => (5, "High-level milestones"),
        _ => (10, "Standard implementation tasks"),
    };

    format!(
        r"# Tasks: {}

## Overview
Executable tasks generated from implementation plan.

## Task Granularity: {}
- Estimated task count: ~{}
- Parallel execution markers: {}

## Phase 1: Setup and Initialization
- [ ] {}T001: Initialize project structure
- [ ] {}T002: Set up development environment
- [ ] {}T003: Install core dependencies
- [ ] {}T004: Configure build system
- [ ] {}T005: Set up version control

## Phase 2: Core Implementation
- [ ] {}T006: Implement data models
- [ ] {}T007: Create business logic layer
- [ ] {}T008: Develop core functionality
- [ ] {}T009: Implement error handling
- [ ] {}T010: Add logging and monitoring

## Phase 3: Integration and Testing
- [ ] {}T011: Create unit tests
- [ ] {}T012: Implement integration tests
- [ ] {}T013: Set up CI/CD pipeline
- [ ] {}T014: Perform code review
- [ ] {}T015: Fix identified issues

## Phase 4: Documentation and Deployment
- [ ] {}T016: Write user documentation
- [ ] {}T017: Create API documentation
- [ ] {}T018: Prepare deployment scripts
- [ ] {}T019: Perform final testing
- [ ] {}T020: Deploy to production

## Prerequisites
{}

## Notes
- Tasks marked with [P] can be executed in parallel
- Update task status as work progresses
- Export to tickets for team collaboration

---
Generated on: {}
",
        title,
        granularity,
        task_count,
        if parallel { "Enabled" } else { "Disabled" },
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        task_prefix,
        extract_prerequisites_from_plan(plan_content),
        Utc::now().format("%Y-%m-%d")
    )
}

fn extract_prerequisites_from_plan(plan_content: &str) -> String {
    // Simple extraction of technology stack from plan
    if plan_content.contains("Technology Stack") {
        for line in plan_content.lines() {
            if line.contains("Technology Stack") {
                return plan_content
                    .lines()
                    .skip_while(|l| !l.contains("Technology Stack"))
                    .skip(1)
                    .take_while(|l| !l.starts_with('#'))
                    .collect::<Vec<_>>()
                    .join("\n");
            }
        }
    }
    "- Plan document available\n- Requirements completed".to_string()
}

fn export_tasks_to_tickets(
    tasks_path: &Path,
    spec: &Specification,
    project_dir: &Path,
    formatter: &OutputFormatter,
) -> Result<()> {
    use crate::core::Priority;
    use crate::storage::{FileStorage, TicketRepository};

    let content = fs::read_to_string(tasks_path)?;
    let storage = FileStorage::new(project_dir.join(".vibe-ticket"));

    let mut created_count = 0;

    // Parse tasks from markdown
    for line in content.lines() {
        if line.contains("- [ ]") && line.contains("T0") {
            // Extract task ID and description
            let task_text = line.trim_start_matches("- [ ]").trim();
            let parts: Vec<&str> = task_text.splitn(2, ':').collect();

            if parts.len() == 2 {
                let task_id_str = parts[0].replace("[P]", "");
                let task_id = task_id_str.trim();
                let description = parts[1].trim();

                // Create ticket slug from task ID
                let slug = format!("{}-{}", spec.metadata.id, task_id.to_lowercase());

                // Create new ticket using builder
                use crate::core::TicketBuilder;
                let ticket = TicketBuilder::new()
                    .slug(slug.clone())
                    .title(format!("[{task_id}] {description}"))
                    .description(format!("Task from specification: {}", spec.metadata.title))
                    .priority(Priority::Medium)
                    .tags(vec![
                        "spec-driven".to_string(),
                        "auto-generated".to_string(),
                        spec.metadata.id.clone(),
                    ])
                    .build();

                // Save ticket
                if storage.save(&ticket).is_ok() {
                    created_count += 1;
                }
            }
        }
    }

    formatter.success(&format!(
        "Exported {} tasks as tickets from specification '{}'",
        created_count, spec.metadata.title
    ));

    Ok(())
}

/// Handle spec status command
pub fn handle_spec_status(
    spec: Option<String>,
    detailed: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));

    // Get spec ID (from parameter or active spec)
    let spec_id = match spec {
        Some(id) => id,
        None => get_active_spec(&project_dir)?,
    };

    // Load specification
    let specification = spec_manager.load(&spec_id)?;

    if formatter.is_json() {
        formatter.json(&serde_json::json!({
            "spec_id": specification.metadata.id,
            "title": specification.metadata.title,
            "status": format!("{:?}", specification.metadata.progress.current_phase()),
            "progress": {
                "requirements": specification.metadata.progress.requirements_completed,
                "design": specification.metadata.progress.design_completed,
                "tasks": specification.metadata.progress.tasks_completed,
            },
            "approval": specification.metadata.progress.approval_status,
        }))?;
    } else {
        formatter.info(&format!(
            "Specification: {} ({})",
            specification.metadata.title, specification.metadata.id
        ));
        formatter.info(&format!(
            "Current Phase: {:?}",
            specification.metadata.progress.current_phase()
        ));

        formatter.info("\nProgress:");
        formatter.info(&format!(
            "  Requirements: {}",
            if specification.metadata.progress.requirements_completed {
                "✓ Complete"
            } else {
                "○ In Progress"
            }
        ));
        formatter.info(&format!(
            "  Design: {}",
            if specification.metadata.progress.design_completed {
                "✓ Complete"
            } else {
                "○ Pending"
            }
        ));
        formatter.info(&format!(
            "  Tasks: {}",
            if specification.metadata.progress.tasks_completed {
                "✓ Complete"
            } else {
                "○ Pending"
            }
        ));

        if detailed {
            formatter.info(&format!("\nCreated: {}", specification.metadata.created_at));
            formatter.info(&format!("Updated: {}", specification.metadata.updated_at));
            if let Some(ticket_id) = &specification.metadata.ticket_id {
                formatter.info(&format!("Ticket: {ticket_id}"));
            }
            if !specification.metadata.tags.is_empty() {
                formatter.info(&format!("Tags: {}", specification.metadata.tags.join(", ")));
            }
        }
    }

    Ok(())
}

/// Handle spec list command
#[allow(clippy::needless_pass_by_value)]
pub fn handle_spec_list(
    status: Option<String>,
    phase: Option<String>,
    _archived: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));
    let specs = spec_manager.list()?;

    // Filter specs
    let filtered_specs: Vec<_> = specs
        .into_iter()
        .filter(|spec| {
            // Filter by status if provided
            if let Some(ref status_filter) = status {
                let current_status = format!("{:?}", spec.progress.current_phase()).to_lowercase();
                if !current_status.contains(&status_filter.to_lowercase()) {
                    return false;
                }
            }

            // Filter by phase if provided
            if let Some(ref phase_filter) = phase {
                match phase_filter.to_lowercase().as_str() {
                    "requirements" => {
                        if spec.progress.requirements_completed {
                            return false;
                        }
                    },
                    "design" => {
                        if !spec.progress.requirements_completed || spec.progress.design_completed {
                            return false;
                        }
                    },
                    "tasks" => {
                        if !spec.progress.design_completed || spec.progress.tasks_completed {
                            return false;
                        }
                    },
                    _ => {},
                }
            }

            true
        })
        .collect();

    if formatter.is_json() {
        let specs_json: Vec<_> = filtered_specs
            .iter()
            .map(|spec| {
                serde_json::json!({
                    "id": spec.id,
                    "title": spec.title,
                    "description": spec.description,
                    "phase": format!("{:?}", spec.progress.current_phase()),
                    "created_at": spec.created_at,
                    "updated_at": spec.updated_at,
                })
            })
            .collect();
        formatter.json(&serde_json::json!(specs_json))?;
    } else if filtered_specs.is_empty() {
        formatter.info("No specifications found");
    } else {
        formatter.info(&format!(
            "Found {} specification(s):\n",
            filtered_specs.len()
        ));

        for spec in &filtered_specs {
            formatter.info(&format!(
                "{} - {} ({:?})",
                spec.id,
                spec.title,
                spec.progress.current_phase()
            ));
        }
    }

    Ok(())
}

/// Handle spec show command
#[allow(clippy::needless_pass_by_value)]
pub fn handle_spec_show(
    spec: String,
    all: bool,
    markdown: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));
    let specification = spec_manager.load(&spec)?;

    if formatter.is_json() {
        formatter.json(&serde_json::json!(specification))?;
    } else {
        formatter.info(&format!(
            "# Specification: {}",
            specification.metadata.title
        ));
        formatter.info(&format!("ID: {}", specification.metadata.id));
        formatter.info(&format!(
            "Description: {}",
            specification.metadata.description
        ));
        formatter.info(&format!(
            "Phase: {:?}",
            specification.metadata.progress.current_phase()
        ));

        if all || markdown {
            // Show all documents
            let doc_types = [
                SpecDocumentType::Requirements,
                SpecDocumentType::Design,
                SpecDocumentType::Tasks,
            ];

            for doc_type in &doc_types {
                let doc_path = spec_manager.get_document_path(&spec, *doc_type);
                if doc_path.exists() {
                    formatter.info(&format!("\n## {doc_type:?} Document\n"));
                    let content =
                        fs::read_to_string(&doc_path).context("Failed to read document")?;
                    formatter.info(&content);
                }
            }
        }
    }

    Ok(())
}

/// Handle spec delete command
#[allow(clippy::needless_pass_by_value)]
pub fn handle_spec_delete(
    spec: String,
    force: bool,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));

    if !force {
        // Confirm deletion
        formatter.warning(&format!(
            "Are you sure you want to delete specification '{spec}'?"
        ));
        formatter.warning("This will delete all associated documents and cannot be undone.");
        formatter.info("Use --force to skip this confirmation.");
        return Ok(());
    }

    spec_manager.delete(&spec)?;
    formatter.success(&format!("Deleted specification '{spec}'"));

    Ok(())
}

/// Handle spec approve command
#[allow(clippy::needless_pass_by_value)]
pub fn handle_spec_approve(
    spec: String,
    phase: String,
    message: Option<String>,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    let spec_manager = SpecManager::new(project_dir.join("specs"));
    let mut specification = spec_manager.load(&spec)?;

    // Parse phase
    let phase_enum = match phase.to_lowercase().as_str() {
        "requirements" => SpecPhase::Requirements,
        "design" => SpecPhase::Design,
        "tasks" => SpecPhase::Tasks,
        _ => {
            return Err(VibeTicketError::InvalidInput(
                "Invalid phase. Must be one of: requirements, design, tasks".to_string(),
            ));
        },
    };

    // Update approval status
    if specification.metadata.progress.approval_status.is_none() {
        specification.metadata.progress.approval_status = Some(std::collections::HashMap::new());
    }

    if let Some(ref mut approvals) = specification.metadata.progress.approval_status {
        approvals.insert(
            format!("{phase_enum:?}"),
            serde_json::json!({
                "approved": true,
                "approved_at": Utc::now(),
                "message": message,
            }),
        );
    }

    specification.metadata.updated_at = Utc::now();
    spec_manager.save(&specification)?;

    formatter.success(&format!(
        "Approved {} phase for specification '{}'",
        phase, specification.metadata.title
    ));

    Ok(())
}

/// Handle spec activate command
#[allow(clippy::needless_pass_by_value)]
pub fn handle_spec_activate(
    spec: String,
    project: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    // Change to project directory if specified
    if let Some(project_path) = project {
        std::env::set_current_dir(project_path)
            .with_context(|| format!("Failed to change to project directory: {project_path}"))?;
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let project_dir = current_dir.join(".vibe-ticket");

    if !project_dir.exists() {
        return Err(VibeTicketError::ProjectNotInitialized);
    }

    // Verify spec exists
    let spec_manager = SpecManager::new(project_dir.join("specs"));
    let specification = spec_manager.load(&spec)?;

    // Save active spec
    let active_spec_path = project_dir.join(".active_spec");
    fs::write(&active_spec_path, &spec).context("Failed to set active specification")?;

    formatter.success(&format!(
        "Set active specification to '{}' ({})",
        specification.metadata.title, spec
    ));

    Ok(())
}

/// Get the active specification ID
fn get_active_spec(project_dir: &Path) -> Result<String> {
    let active_spec_path = project_dir.join(".active_spec");

    if !active_spec_path.exists() {
        return Err(VibeTicketError::NoActiveSpec);
    }

    fs::read_to_string(&active_spec_path)
        .context("Failed to read active specification")
        .map(|s| s.trim().to_string())
}

/// Open a file in the default editor
fn open_in_editor(path: &Path) -> Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    std::process::Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("Failed to open editor: {editor}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_formatter() -> OutputFormatter {
        OutputFormatter::new(false, false)
    }

    #[test]
    fn test_spec_init() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let formatter = create_test_formatter();
        let result = handle_spec_init(
            "Test Spec",
            Some("Test description"),
            None,
            Some("test,spec"),
            None,
            &formatter,
        );

        assert!(result.is_ok());

        // Verify spec was created
        let specs_dir = project_dir.join("specs");
        assert!(specs_dir.exists());

        // Check that at least one spec directory was created
        assert!(
            !std::fs::read_dir(&specs_dir)
                .unwrap()
                .filter_map(std::result::Result::ok)
                .next()
                .is_none()
        );
    }

    #[test]
    fn test_spec_init_no_project() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let formatter = create_test_formatter();
        let result = handle_spec_init("Test Spec", None, None, None, None, &formatter);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VibeTicketError::ProjectNotInitialized
        ));
    }

    #[test]
    fn test_get_active_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Test no active spec
        let result = get_active_spec(&project_dir).unwrap_err();
        assert!(matches!(result, VibeTicketError::NoActiveSpec));

        // Test with active spec
        let active_spec_path = project_dir.join(".active_spec");
        std::fs::write(&active_spec_path, "test-spec-id").unwrap();

        let active_spec = get_active_spec(&project_dir).unwrap();
        assert_eq!(active_spec, "test-spec-id");
    }

    #[test]
    fn test_spec_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let formatter = create_test_formatter();

        // Initialize spec
        let result = handle_spec_init(
            "Lifecycle Test",
            Some("Testing spec lifecycle"),
            None,
            None,
            None,
            &formatter,
        );
        assert!(result.is_ok());

        // List specs
        let list_result = handle_spec_list(None, None, false, None, &formatter);
        assert!(list_result.is_ok());

        // Test status command (should fail without active spec)
        let status_result = handle_spec_status(None, false, None, &formatter);
        assert!(status_result.is_err());
    }

    #[test]
    fn test_spec_delete_without_force() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let formatter = create_test_formatter();

        // Try delete without force (should just show warning)
        let result = handle_spec_delete("test-spec".to_string(), false, None, &formatter);
        assert!(result.is_ok()); // Doesn't actually delete without force
    }

    #[test]
    fn test_spec_approve_invalid_phase() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".vibe-ticket");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let formatter = create_test_formatter();

        // Create a spec first
        handle_spec_init("Approve Test", None, None, None, None, &formatter).unwrap();

        // Try to approve with invalid phase
        let result = handle_spec_approve(
            "test-spec".to_string(),
            "invalid-phase".to_string(),
            None,
            None,
            &formatter,
        );

        assert!(result.is_err());
    }
}
