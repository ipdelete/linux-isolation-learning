---
description: Generate a detailed progress log for the current chat session
tags: [documentation, logging, progress]
---

Generate a comprehensive progress log for this chat session and save it to `.ai/YYYY-MM-DD/HHMMSS-progress-log.md`.

The progress log should include:

## Structure

### Header
- Date (YYYY-MM-DD format)
- Session timestamp
- Brief overview of what was accomplished

### What We Built
- List all major features/components created
- Include file paths and line numbers for key changes
- Describe the purpose of each component

### Files Created/Modified
Organize into three sections:
- **Created**: New files with brief descriptions
- **Modified**: Changed files with what was modified
- **Existing**: Relevant files that weren't changed

### Key Concepts Explained
- Document any architectural patterns used
- Explain technical concepts that were discussed
- Include code flow diagrams using text/ascii if helpful

### How to Use
- Command examples for running/testing the work
- Configuration steps if needed
- Environment setup requirements

### Technical Notes
- Any warnings, errors, or issues encountered
- Performance considerations
- Type hints or linting issues
- Security considerations

### Next Steps (Not Implemented)
- Potential improvements discussed but not built
- Features that could be added
- Production considerations

### Repository Information (if applicable)
- URL
- Branch name
- Commit hash and message

## Formatting
- Use clear markdown headings
- Include code blocks with language hints
- Use bullet points for lists
- Add diagrams/flow charts where helpful using text/ascii art
- Reference specific files with `file:line` format for easy navigation

## Important
- First run `date +%Y-%m-%d` to get the date folder name, and `date +%H%M%S` for the filename timestamp
- Create `.ai/YYYY-MM-DD/` directory if it doesn't exist (e.g., `.ai/2025-11-26/`)
- Write the log to `.ai/YYYY-MM-DD/HHMMSS-progress-log.md` (e.g., `.ai/2025-11-26/143022-progress-log.md`)
- Make it detailed but concise - focus on what matters
- Include enough context that someone reading it later can understand what was done and why