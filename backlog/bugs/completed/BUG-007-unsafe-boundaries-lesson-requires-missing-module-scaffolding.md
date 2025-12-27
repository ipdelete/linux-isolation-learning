# Bug: Unsafe-boundaries lesson depends on missing `syscall` module scaffolding

## Summary
The unsafe-boundaries lesson walked through creating `crates/ns-tool/src/syscall.rs`, `src/lib.rs`, and `unsafe_wrapper_test.rs`, but those files weren't present in the repo and `ns-tool` is a binary-only crate structure.

## Location
- `docs/00-foundations/06-unsafe-boundaries.md`

## Problem
Learners couldn't follow the lesson in a TDD manner without additional repo scaffolding (library target + module exports), and the lesson's steps were too big a jump compared to existing stubs.

## Resolution
The lesson has been rewritten to work with the existing binary-only crate structure:

### Changes Made
1. **Updated Step 1 (Write Tests)**: Changed test file from `unsafe_wrapper_test.rs` to `syscall_test.rs` with proper annotations about binary crate testing patterns
2. **Updated Step 2 (Export Module)**: Changed from creating `lib.rs` to adding `pub mod syscall;` directly in `main.rs`
3. **Updated Step 3 (Test File Implementation)**: Added clearer comments about testing binary crates through integration tests
4. **Removed Step 4-5**: Eliminated the unnecessary library crate interface creation
5. **Updated verification commands**: Changed to match the actual test names and structure
6. **Updated cleanup section**: Reflects the actual files created

### New Approach
- **Binary-first design**: Uses `pub mod syscall;` in main.rs rather than creating lib.rs
- **Integration tests**: Uses standard Rust integration tests in `tests/` directory
- **Matches repo structure**: Works with existing ns-tool binary architecture
- **Same teaching value**: Still teaches unsafe wrapper patterns, minimal unsafe surfaces, and safe error handling

### Key Benefits
- No unnecessary library crate scaffolding
- Simpler file structure for learners to understand
- Works immediately with existing project layout
- Still demonstrates all unsafe wrapper patterns clearly

## Status
COMPLETED - Lesson rewritten to match existing binary-only crate structure while maintaining pedagogical value.
