# Bug: Fast-track combine namespaces lesson references missing test and unimplemented code

## Summary
The fast-track combined namespaces lesson points to a test file that does not exist and to an implementation that is still `todo!()`, so the documented commands fail out of the box.

## Location
- `docs/fast-track/04-combine.md`
- `crates/contain/src/ns.rs`
- (missing) `crates/contain/tests/ns_container_test.rs`

## Problem
The lesson instructs learners to run a test and then implement `NsCommand::Container`, but the test file is absent and the match arm is unimplemented.

## Steps to reproduce
1. Run `cargo test -p contain --test ns_container_test`.
2. Run `cargo run -p contain -- ns container -- /bin/sh -c 'echo PID:$$ && hostname'`.

## Expected
- `cargo test -p contain --test ns_container_test` runs the referenced test file (initially failing due to `todo!()`).
- `cargo run -p contain -- ns container -- /bin/sh -c 'echo PID:$$ && hostname'` prints `PID:1` and `container` after implementation.

## Actual
- `cargo test -p contain --test ns_container_test` fails with `error: no test target named 'ns_container_test'`.
- `cargo run -p contain -- ns container -- /bin/sh -c 'echo PID:$$ && hostname'` panics with `not yet implemented: Implement mini-container`.

## Impact
Learners cannot follow the lesson as written because the referenced test file is missing and the code path is stubbed.

## Status
OPEN
