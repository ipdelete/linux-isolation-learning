# Bug: `cgroup-tool` scaffolding references non-existent lesson filenames

## Summary
`cgroup-tool` source and test headers reference lesson files that do not exist in `docs/02-cgroups/`, which breaks traceability between code/test TODOs and the tutorial docs.

## Location
- `crates/cgroup-tool/src/main.rs`
- `crates/cgroup-tool/tests/create_test.rs`
- `crates/cgroup-tool/tests/attach_test.rs`
- `crates/cgroup-tool/tests/delete_test.rs`
- `crates/cgroup-tool/tests/pids_test.rs`

## Problem
The repository has these cgroup lesson docs:
- `docs/02-cgroups/01-cgv2-basics.md`
- `docs/02-cgroups/05-pids.md`

But the scaffolding comments reference:
- `docs/02-cgroups/01-create-attach.md` (does not exist)
- `docs/02-cgroups/04-pids.md` (does not exist; PIDs is `05-pids.md`)

This makes it hard for learners (and maintainers) to find the corresponding lesson from the code/test file they’re editing.

## Steps to reproduce
1. Open `crates/cgroup-tool/src/main.rs`.
2. Locate the `// Lesson:` comment for `Create`, `Attach`, `Delete`, or `PidsMax`.
3. Try to open the referenced doc path; it does not exist.

## Expected
`// Lesson:` references should point to the correct, existing lesson docs.

## Actual
Several `// Lesson:` references point to non-existent doc files.

## Suggested fix
- Update `// Lesson:` references to:
  - `docs/02-cgroups/01-cgv2-basics.md` for create/attach/delete
  - `docs/02-cgroups/05-pids.md` for PIDs
- (Optional) Add a quick consistency check (script or CI) that validates `// Lesson:` paths exist.

