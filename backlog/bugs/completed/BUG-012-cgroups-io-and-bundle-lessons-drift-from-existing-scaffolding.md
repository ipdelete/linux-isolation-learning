# BUG-012 COMPLETED: Cgroups I/O and multi-resource lessons instruct scaffolding that already exists

## Summary
In `docs/02-cgroups/04-io.md` and `docs/02-cgroups/06-multi-resource.md`, the "Write Tests (Red)" sections instructed learners to add/create scaffolding that was already present in the repository.

## Fix Applied

### docs/02-cgroups/04-io.md
**Changes**: Updated the "Write Tests (Red)" section to reference existing scaffolding:
- Changed from "Add the IoMax Command Variant" and "Create the Test File" instructions
- Now says: "The test file and command scaffold already exist. Your task is to implement the test functions."
- Removed instructions to manually add `Command::IoMax` variant
- Changed `io_test.rs` creation instructions to "Open the existing test file"
- Updated test implementation guidance to reference existing helper functions like `find_test_block_device()`

**Changes**: Updated the "Build (Green)" section:
- Changed from step "Step 1: Add" to "Step 1: Open the Existing IoMax Match Arm"
- Now says: "The `Command::IoMax` variant already exists in the enum (lines 36-43). Your task is to implement the handler in the match statement."
- Clarified that learners fill in the existing `todo!()` placeholder

### docs/02-cgroups/06-multi-resource.md
**Changes**: Updated the "Write Tests (Red)" section:
- Changed from "Create a new test file for multi-resource scenarios" with full code block
- Now says: "The test file already exists with test function stubs. Your task is to implement the test functions."
- Removed the full test file code block
- Added concise steps: "Step 1: Open the Test File", "Step 2: Implement the Test Functions", "Step 3: Run Tests"
- Focused on directing learners to fill in existing `todo!()` functions in the already-existing `bundle_test.rs`

## Files Updated
1. `/workspaces/linux-isolation-learning/docs/02-cgroups/04-io.md` - Rewrote scaffolding steps to reference existing files/variants
2. `/workspaces/linux-isolation-learning/docs/02-cgroups/06-multi-resource.md` - Simplified to reference existing test file

## Result
Learners will now:
- Open existing test files instead of creating them
- Fill in existing `todo!()` stubs instead of adding code from scratch
- Follow the actual TDD pattern of the codebase (read existing scaffolding → implement TODOs)
- Not encounter compilation errors from duplicate enum variants or file existence conflicts

## Status
COMPLETED - Lessons now match the actual repository structure and guide learners to implement existing scaffolding.
