# Feature Specification: {{title}}

**Feature Branch**: `{{branch_name}}`  
**Created**: {{created_date}}  
**Status**: Draft  
**Input**: User description: "{{requirements}}"

## Execution Flow (main)
```
1. Parse user description from Input
   → Parsed: {{parsed_summary}} ✓
2. Extract key concepts from description
   → Actors: {{actors}}
   → Actions: {{actions}}
   → Data: {{data}}
   → Constraints: {{constraints}}
3. For each unclear aspect:
   → [NEEDS CLARIFICATION: {{clarification_items}}]
4. Fill User Scenarios & Testing section ✓
5. Generate Functional Requirements ✓
6. Identify Key Entities ✓
7. Run Review Checklist
   → WARN "Spec has uncertainties" - {{clarification_count}} clarification items marked
8. Return: SUCCESS (spec ready for planning with minor clarifications)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
{{user_story}}

### Acceptance Scenarios
1. **Given** {{given_1}}, **When** {{when_1}}, **Then** {{then_1}}
2. **Given** {{given_2}}, **When** {{when_2}}, **Then** {{then_2}}
3. **Given** {{given_3}}, **When** {{when_3}}, **Then** {{then_3}}

### Edge Cases
{{edge_cases}}

## Requirements *(mandatory)*

### Functional Requirements
{{functional_requirements}}

### Key Entities
{{key_entities}}

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [ ] No implementation details (languages, frameworks, APIs)
- [ ] Focused on user value and business needs
- [ ] Written for non-technical stakeholders
- [ ] All mandatory sections completed

### Requirement Completeness
- [ ] No [NEEDS CLARIFICATION] markers remain
- [ ] Requirements are testable and unambiguous  
- [ ] Success criteria are measurable
- [ ] Scope is clearly bounded
- [ ] Dependencies and assumptions identified

---

## Execution Status
*Updated by main() during processing*

- [ ] User description parsed
- [ ] Key concepts extracted
- [ ] Ambiguities marked
- [ ] User scenarios defined
- [ ] Requirements generated
- [ ] Entities identified
- [ ] Review checklist passed

---