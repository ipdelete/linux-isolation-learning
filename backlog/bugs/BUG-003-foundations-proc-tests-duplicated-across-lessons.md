# Bug: `proc` testing work is duplicated across foundations lessons

## Summary
Multiple lessons in `docs/00-foundations/` instruct learners to implement the same `crates/ns-tool/tests/proc_test.rs` tests, causing repetition and a confusing progression.

## Location
- `docs/00-foundations/01-rust-syscall-basics.md`
- `docs/00-foundations/02-cli-patterns.md`
- `docs/00-foundations/03-procfs-intro.md`

## Problem
All three lessons converge on “implement tests for `ns-tool proc`”, even though this is a single deliverable.

## Expected
Each lesson has a distinct deliverable aligned with its topic (syscalls vs clap vs procfs), without re-doing the same test file.

## Actual
Learners are repeatedly told to implement `proc_test.rs` tests in multiple lessons.

## Suggested fix
- Pick one lesson to own “implement `proc_test.rs`”.
- Adjust the other lessons to focus on their unique topic (e.g., clap help/errors in `02`, parsing/sorting/output determinism in `03`).

