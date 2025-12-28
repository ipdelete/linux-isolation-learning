# Bug: Fast-track OCI bundle lesson references missing test, implementation, and tempfile dev-dependency

## Summary
The OCI bundle lesson points to a test file that does not exist, uses `tempfile` without a dev-dependency, and `OciCommand::Init` is unimplemented, so the documented commands fail out of the box.

## Location
- `docs/fast-track/08-oci-bundle.md`
- `crates/contain/src/oci.rs`
- (missing) `crates/contain/tests/oci_test.rs`
- `crates/contain/Cargo.toml` (missing `tempfile` dev-dependency)

## Problem
The lesson instructs learners to run `cargo test -p contain --test oci_test`, but the test file is absent and the implementation is a `todo!()`. The test also uses `tempfile` without declaring it.

## Steps to reproduce
1. Run `cargo test -p contain --test oci_test`.
2. Run `cargo run -p contain -- oci init /tmp/mybundle`.

## Expected
- The test compiles and runs, creating `config.json` and `rootfs`.
- The manual command creates a valid bundle layout.

## Actual
- The test target is missing and the crate fails to compile due to missing `tempfile` dependency.
- The command panics with `not yet implemented: Implement OCI bundle init`.

## Impact
Learners cannot follow the lesson as written because the referenced test file is missing, the implementation is stubbed, and the dev-dependency is absent.

## Status
OPEN
