---
description: Create tutorials for a docs section in parallel
argument-hint: [section-path] [subsection]
allowed-tools: Read, Write, Edit, Task
---

Extract the section number from the path (e.g., "docs/04-ebpf" → "04") and read the matching backlog files:
- Todo: @backlog/todos/04_ebpf_todo.md (pattern: {num}_*_todo.md)
- Plan: @backlog/plans/04_ebpf_plan.md (pattern: {num}_*_plan.md)

Identify incomplete items from the todo file. The user may optionally specify a subsection header (e.g., "crates/ebpf-tool-common") to filter to only items under that `## heading`. If a subsection is specified, only process items under that heading.

For each incomplete item in scope:
1. Read the plan file and @docs/00-foundations/00-lesson-template.md for context
2. Orchestrate parallel rust-tutorial-expert agents (one per item) to create comprehensive content
3. Each agent should follow the TDD approach and lesson template structure
4. Include appropriate context about the learning path, codebase structure, and dependencies

After all agents complete:
1. Update the todo file to mark the completed items
2. Provide a summary of what was created

If the user doesn't specify a section, ask which section they want to complete.
If there are many items, ask which subsection(s) to process.
