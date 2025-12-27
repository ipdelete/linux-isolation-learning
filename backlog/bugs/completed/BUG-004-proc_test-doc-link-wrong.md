# Bug: `proc_test.rs` points to a non-existent lesson filename

## Summary
The header comment in `crates/ns-tool/tests/proc_test.rs` references `docs/00-foundations/01-setup.md`, which does not exist.

## Location
- `crates/ns-tool/tests/proc_test.rs`

## Problem
The lesson reference in the test scaffolding is wrong, making it harder to navigate the tutorial-to-code mapping.

## Expected
The test file references the correct lesson, likely `docs/00-foundations/00-setup-rust.md` or `docs/00-foundations/01-rust-syscall-basics.md` depending on intended ownership.

## Actual
It references `docs/00-foundations/01-setup.md`.

## Suggested fix
- Update the lesson reference comment to the correct file.
- Optionally, align it with whichever foundation lesson “owns” `proc_test.rs` after de-duplication.

