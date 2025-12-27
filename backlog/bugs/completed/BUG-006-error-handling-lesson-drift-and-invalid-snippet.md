# Bug: Error-handling lesson is out of sync with repo and contains a non-compiling snippet

## Summary
`ns-tool` already has `crates/ns-tool/src/error.rs` and `crates/ns-tool/tests/error_test.rs`, but the lesson reads like these should be created. Also, one sample uses `.pipe(Ok)` which won’t compile as written.

## Location
- `docs/00-foundations/05-error-handling.md`

## Problem
The lesson describes creating files that already exist and includes at least one invalid snippet, which will confuse learners who compare instructions to the repo.

## Expected
Lesson should either:
- guide learners to fill in TODO scaffolding (if intended), or
- treat `error.rs` as a reference and focus on extending/modifying it.
All code snippets should compile.

## Actual
- Repo already contains `crates/ns-tool/src/error.rs` with unit tests.
- Repo already contains `crates/ns-tool/tests/error_test.rs`.
- Snippet includes `.pipe(Ok)` (not in std) and doesn’t compile.

## Suggested fix
- Update lesson to match the current scaffolding state.
- Replace `.pipe(Ok)` with standard Rust (e.g., `Ok(...)`) or remove the snippet.

