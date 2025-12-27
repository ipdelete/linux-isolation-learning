# Bug: `oci-tool` scaffold comments reference a non-existent lesson file

## Summary
The `oci-tool` source/tests reference `docs/03-runc/01-bundle.md`, but the actual lesson file is `docs/03-runc/01-oci-bundle.md`. This makes “Lesson:” pointers misleading for learners and maintainers.

## Location
- `crates/oci-tool/src/main.rs` (comments above `Command::Init` and `Command::Show`)
- `crates/oci-tool/tests/init_test.rs` (header comment)
- `crates/oci-tool/tests/show_test.rs` (header comment)

## Problem
The referenced path `docs/03-runc/01-bundle.md` does not exist in the repo.

## Steps to reproduce
1. Open any of the files above.
2. Try to navigate to the referenced lesson path.

## Expected
“Lesson:” references point to existing docs files.

## Actual
The “Lesson:” path is broken.

## Suggested fix
- Update references from `docs/03-runc/01-bundle.md` to `docs/03-runc/01-oci-bundle.md`.

