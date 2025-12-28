# Bug: Fast-track mount namespace lesson references missing test and unimplemented code

## Summary
The fast-track mount namespace lesson points to a test file that does not exist and to an implementation that is still `todo!()`, so the documented commands fail out of the box.

## Location
- `docs/fast-track/02-mount-namespace.md`
- `crates/contain/src/ns.rs`
- (missing) `crates/contain/tests/ns_mount_test.rs`

## Problem
The lesson instructs learners to run a test and then implement `NsCommand::Mount`, but the test file is absent and the `NsCommand::Mount` match arm is unimplemented.

## Steps to reproduce
1. Run `cargo test -p contain --test ns_mount_test`.
2. Run `cargo run -p contain -- ns mount`.

## Expected
- `cargo test -p contain --test ns_mount_test` runs the referenced test file (initially failing due to `todo!()`).
- `cargo run -p contain -- ns mount` prints the tmpfs mount output after implementation.

## Actual
- `cargo test -p contain --test ns_mount_test` fails with `error: no test target named 'ns_mount_test'`.
- `cargo run -p contain -- ns mount` panics with `not yet implemented: Implement mount namespace`.

## Impact
Learners cannot follow the lesson as written because the referenced test file is missing and the code path is stubbed.

## Status
OPEN
