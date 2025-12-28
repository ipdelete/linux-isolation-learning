# Bug: Fast-track network namespace lesson references missing test and unimplemented code

## Summary
The fast-track network namespace lesson points to a test file that does not exist and to an implementation that is still `todo!()`, so the documented commands fail out of the box.

## Location
- `docs/fast-track/03-network-namespace.md`
- `crates/contain/src/net.rs`
- (missing) `crates/contain/tests/net_test.rs`

## Problem
The lesson instructs learners to run a test and then implement `NetCommand` handlers, but the test file is absent and the match arms are unimplemented.

## Steps to reproduce
1. Run `cargo test -p contain --test net_test`.
2. Run `cargo run -p contain -- net create mynet`.

## Expected
- `cargo test -p contain --test net_test` runs the referenced test file (initially failing due to `todo!()`).
- `cargo run -p contain -- net create mynet` creates the namespace after implementation.

## Actual
- `cargo test -p contain --test net_test` fails with `error: no test target named 'net_test'`.
- `cargo run -p contain -- net create mynet` panics with `not yet implemented: Implement network namespace creation`.

## Impact
Learners cannot follow the lesson as written because the referenced test file is missing and the code paths are stubbed.

## Status
CLOSED

## Resolution
Created `crates/contain/tests/net_test.rs` with `todo!()` stub following TDD pattern.
The implementation stubs in `src/net.rs` already exist with `todo!()` - learners implement both.
