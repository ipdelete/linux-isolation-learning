---
description: Create tutorials for a docs section in parallel
argument-hint: [section-path]
allowed-tools: Read, Write, Edit, Task
---

Read the relevant todo file from @backlog/todos/ and identify all incomplete lessons in the section specified by the user (e.g., "docs/01-namespaces" or "docs/04-ebpf").

For each incomplete lesson in that section:
1. Read the relevant plan from @backlog/plans/ and @docs/00-foundations/00-lesson-template.md for context
2. Orchestrate parallel rust-tutorial-expert agents (one per lesson) to create comprehensive tutorial content
3. Each agent should follow the TDD approach and lesson template structure
4. Include appropriate context about the learning path, codebase structure, and lesson dependencies

After all agents complete:
1. Update the relevant todo file in @backlog/todos/ to mark the completed lessons
2. Provide a summary of what was created

If the user doesn't specify a section, ask which section they want to complete.
