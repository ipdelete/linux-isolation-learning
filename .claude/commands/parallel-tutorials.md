---
description: Create tutorials for a docs section in parallel
argument-hint: [section-path]
allowed-tools: Read, Write, Edit, Task
---

Extract the section number from the path (e.g., "docs/04-ebpf" → "04") and read the matching backlog files:
- Todo: @backlog/todos/04_ebpf_todo.md (pattern: {num}_*_todo.md)
- Plan: @backlog/plans/04_ebpf_plan.md (pattern: {num}_*_plan.md)

Identify all incomplete lessons in the specified section from the todo file.

For each incomplete lesson in that section:
1. Read the plan file and @docs/00-foundations/00-lesson-template.md for context
2. Orchestrate parallel rust-tutorial-expert agents (one per lesson) to create comprehensive tutorial content
3. Each agent should follow the TDD approach and lesson template structure
4. Include appropriate context about the learning path, codebase structure, and lesson dependencies

After all agents complete:
1. Update the todo file to mark the completed lessons
2. Provide a summary of what was created

If the user doesn't specify a section, ask which section they want to complete.
