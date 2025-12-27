# Bug: Unsafe-boundaries lesson depends on missing `syscall` module scaffolding

## Summary
The unsafe-boundaries lesson walks through creating `crates/ns-tool/src/syscall.rs`, `src/lib.rs`, and `unsafe_wrapper_test.rs`, but those files aren’t present in the repo and `ns-tool` is currently a binary-only crate structure.

## Location
- `docs/00-foundations/06-unsafe-boundaries.md`

## Problem
Learners can’t follow the lesson in a TDD manner without additional repo scaffolding (library target + module exports), and the lesson’s steps are too big a jump compared to existing stubs.

## Expected
Either:
- Repo includes the needed scaffolding (stubs + TODO tests) before the lesson, or
- The lesson is moved later / rewritten to match the repo’s current layout and teaching progression.

## Actual
`crates/ns-tool/src/syscall.rs` and `crates/ns-tool/src/lib.rs` are not present; the lesson assumes they exist or should be added.

## Suggested fix
- Add minimal scaffolding ahead of time (empty `lib.rs` exporting `syscall` + TODO tests) and keep the lesson focused on filling in TODOs, or
- Re-scope the lesson to explain unsafe boundaries using existing code patterns without requiring a crate layout change.

