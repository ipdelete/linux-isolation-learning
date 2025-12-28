# Bug: Fast-track cgroup basics lesson references missing test and unimplemented code

## Summary
The fast-track cgroup basics lesson points to a test file that does not exist and to implementations that are still `todo!()`, so the documented commands fail out of the box.

## Location
- `docs/fast-track/05-cgroup-basics.md`
- `crates/contain/src/cgroup.rs`
- (missing) `crates/contain/tests/cgroup_test.rs`

## Problem
The lesson instructs learners to run a test and then implement `CgroupCommand` handlers, but the test file is absent and the match arms are unimplemented.

## Steps to reproduce
1. Run `cargo test -p contain --test cgroup_test`.
2. Run `cargo run -p contain -- cgroup create /sys/fs/cgroup/mygroup`.

## Expected
- `cargo test -p contain --test cgroup_test` runs the referenced test file (initially failing due to `todo!()`).
- `cargo run -p contain -- cgroup create /sys/fs/cgroup/mygroup` creates the cgroup after implementation.

## Actual
- `cargo test -p contain --test cgroup_test` fails with `error: no test target named 'cgroup_test'`.
- `cargo run -p contain -- cgroup create /sys/fs/cgroup/mygroup` panics with `not yet implemented: Implement cgroup creation`.

## Impact
Learners cannot follow the lesson as written because the referenced test file is missing and the code paths are stubbed.

## Status
CLOSED

## Resolution
Created `crates/contain/tests/cgroup_test.rs` with `todo!()` stub following TDD pattern.
The implementation stubs in `src/cgroup.rs` already exist with `todo!()` - learners implement both.
