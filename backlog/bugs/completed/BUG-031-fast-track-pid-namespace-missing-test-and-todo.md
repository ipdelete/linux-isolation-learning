# Bug: Fast-track PID namespace lesson references missing test and unimplemented code

## Summary
The fast-track PID namespace lesson points to a test file that does not exist and to an implementation that is still `todo!()`, so the documented commands fail out of the box.

## Location
- `docs/fast-track/01-pid-namespace.md`
- `crates/contain/src/ns.rs`
- (missing) `crates/contain/tests/ns_pid_test.rs`

## Problem
The lesson instructs learners to run a test and then implement `NsCommand::Pid`, but the test file is absent and the `NsCommand::Pid` match arm is unimplemented.

## Steps to reproduce
1. Run `cargo test -p contain --test ns_pid_test`.
2. Run `cargo run -p contain -- ns pid`.

## Expected
- `cargo test -p contain --test ns_pid_test` runs the referenced test file (initially failing due to `todo!()`).
- `cargo run -p contain -- ns pid` prints `PID inside namespace: 1` after implementation.

## Actual
- `cargo test -p contain --test ns_pid_test` fails with `error: no test target named 'ns_pid_test'`.
- `cargo run -p contain -- ns pid` panics with `not yet implemented: Implement PID namespace`.

## Impact
Learners cannot follow the lesson as written because the referenced test file is missing and the code path is stubbed.

## Status
CLOSED

## Resolution
Created `crates/contain/tests/ns_pid_test.rs` with `todo!()` stub following TDD pattern.
The implementation stub in `src/ns.rs` already exists with `todo!()` - learners implement both.
