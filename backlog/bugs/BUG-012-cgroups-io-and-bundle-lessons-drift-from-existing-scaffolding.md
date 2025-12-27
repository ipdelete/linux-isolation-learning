# Bug: Cgroups I/O and multi-resource lessons instruct scaffolding that already exists

## Summary
In `docs/02-cgroups/04-io.md` and `docs/02-cgroups/06-multi-resource.md`, the "Write Tests (Red)" sections instruct learners to add/create scaffolding that is already present in the repository, creating confusion and duplicated work.

## Location
- `docs/02-cgroups/04-io.md`
- `docs/02-cgroups/06-multi-resource.md`

## Problem
The lessons are written as if the repo does not yet include:
- `Command::IoMax` + corresponding match arm scaffolding
- `crates/cgroup-tool/tests/io_test.rs`
- `crates/cgroup-tool/tests/bundle_test.rs`

In this repo, those already exist, so a learner following instructions literally will either:
- try to create files that already exist, or
- add duplicate `Command` variants / match arms, causing compile errors.

## Steps to reproduce
1. Open `docs/02-cgroups/04-io.md`.
2. Follow "Step 1: Add the IoMax Command Variant" and "Step 2: Create the Test File".
3. Notice both the enum variant and test file already exist (`crates/cgroup-tool/src/main.rs`, `crates/cgroup-tool/tests/io_test.rs`).
4. Open `docs/02-cgroups/06-multi-resource.md`.
5. Follow "Create a new test file for multi-resource scenarios".
6. Notice `crates/cgroup-tool/tests/bundle_test.rs` already exists.

## Expected
The docs should match the repo state:
- If scaffolding exists, instructions should say "open the existing file and fill in TODOs".
- If scaffolding is expected to be added by the learner, the repo should not pre-contain it.

## Actual
The docs direct learners to add/create scaffolding that is already present.

## Suggested fix
- Update `docs/02-cgroups/04-io.md` to instruct learners to open and implement the existing `io_test.rs` and existing `Command::IoMax` match arm, rather than adding them.
- Update `docs/02-cgroups/06-multi-resource.md` to instruct learners to open and implement the existing `bundle_test.rs` TODOs, and clarify that “Option B” is an optional enhancement (and ensure it aligns with current CLI conventions).

