# Bug: `proc` testing work is duplicated across foundations lessons

## Summary
Multiple lessons in `docs/00-foundations/` were instructing learners to implement the same `crates/ns-tool/tests/proc_test.rs` tests, causing repetition and a confusing progression.

## Status
**FIXED** - Refactored lessons to have distinct deliverables.

## Solution Applied

### Lesson 01: Rust Syscall Basics
**Changed from**: Implementing proc_test.rs tests
**Changed to**: Studying syscall patterns and understanding the decision tree for when to use `nix` vs `libc` vs `std`
- Removed test implementation task
- Focused on understanding `print_proc_ns()` as a reference example
- Emphasized syscall safety patterns and type safety in Rust

### Lesson 02: CLI Patterns with Clap
**Changed from**: Implementing proc_test.rs tests
**Changed to**: Exploring clap's help text generation and error handling
- Removed test implementation task
- Added exercise to add doc comments to the `Command` enum
- Focused on how clap generates help text and validates arguments
- Made CLI patterns the clear deliverable, not proc testing

### Lesson 03: Procfs Intro (PRIMARY OWNER)
**Confirmed**: Owns proc_test.rs implementation
- This lesson naturally aligns with `/proc` filesystem reading
- Students write tests that verify the existing `proc` subcommand works
- Tests demonstrate the TDD pattern: writing tests for working code

## Files Modified
- `docs/00-foundations/01-rust-syscall-basics.md` - Refactored to focus on concepts
- `docs/00-foundations/02-cli-patterns.md` - Refactored to focus on clap patterns
- `docs/00-foundations/03-procfs-intro.md` - Clarified as owner of proc_test.rs
- `crates/ns-tool/tests/proc_test.rs` - Updated comments to clarify lesson ownership

## Result
Each lesson now has a distinct, non-overlapping deliverable aligned with its topic:
1. Lesson 01: Understand syscall patterns (study existing code)
2. Lesson 02: Learn CLI design with clap (explore help/errors)
3. Lesson 03: Test `/proc` filesystem reading (implement proc_test.rs)
