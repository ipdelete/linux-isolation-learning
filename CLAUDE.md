# Creating Tutorials (TDD Pattern)

This project uses Test-Driven Development for teaching. Learners write tests first (RED), then implement code (GREEN).

## Structure

```
docs/NN-section/           # Lesson docs (guide learners through TDD)
crates/<tool>/src/         # Implementation with todo!() stubs
crates/<tool>/tests/       # Test files with todo!() stubs
backlog/plans/             # Implementation plans
backlog/todos/             # Progress checklists
```

## Creating a New Lesson

### 1. Scaffold the Crate

**main.rs** - Add subcommand with `todo!()` stub:
```rust
Command::NewFeature => {
    // TODO: Implement new-feature subcommand
    // Lesson: docs/NN-section/XX-lesson.md
    // Tests: tests/feature_test.rs
    todo!("Implement new-feature - write tests first!")
}
```

**tests/feature_test.rs** - Create test file with `todo!()` stubs:
```rust
// Tests for the `new-feature` subcommand
// Lesson: docs/NN-section/XX-lesson.md
//
// TDD Workflow:
// 1. Write tests below FIRST (RED)
// 2. Implement code in src/main.rs (GREEN)

#[test]
fn test_feature_success() {
    // TODO: Test that new-feature works correctly
    //
    // Hints:
    // - Use assert_cmd::Command
    // - Check for expected output

    todo!("Implement test for new-feature success case")
}

#[test]
fn test_feature_error_handling() {
    // TODO: Test error cases

    todo!("Implement test for error handling")
}
```

### 2. Write the Lesson Doc

Use `docs/00-foundations/00-lesson-template.md` as reference. Key sections:

```markdown
# Lesson Title

## Goal
One concept + one deliverable.

## Prereqs
- Prior lessons required
- System requirements (sudo, etc.)

## Write Tests (Red)
**Test file**: `crates/<tool>/tests/feature_test.rs`

1. Open the test file
2. Find the TODO
3. Replace todo!() with test implementation
4. Run tests (expect failure)

## Build (Green)
**Implementation file**: `crates/<tool>/src/main.rs`
**TODO location**: Line ~XX

1. Find the todo!() stub
2. Replace with implementation
3. Run tests (expect success)

## Verify
- Automated: `cargo test -p <tool>`
- Manual: Commands to run and expected output

## Common Errors
1. **Error message** - Cause and fix
```

### 3. Update Backlog

**backlog/todos/NN_section_todo.md**:
```markdown
## Phase 2: Crate Scaffolding
- [ ] crates/<tool>/tests/feature_test.rs

## Phase 3: Lesson Docs
- [ ] docs/NN-section/XX-lesson.md (feature_test.rs → Command::NewFeature)
```

## Conventions

- Tests go in `crates/<tool>/tests/` (integration) or inline `#[cfg(test)]` (unit)
- Use `assert_cmd` for CLI testing
- Root-required tests: check `Uid::effective().is_root()` and skip if not
- Shared types crates: define types as scaffolding, tests as `todo!()` stubs
- Keep lessons ~30-50 minutes

## Verification Checklist

Before marking a lesson complete:
- [ ] Crate compiles: `cargo build -p <tool>`
- [ ] Tests fail with `todo!()`: `cargo test -p <tool>` shows RED
- [ ] Lesson doc has: Goal, Prereqs, Write Tests, Build, Verify, Common Errors
- [ ] Backlog updated
