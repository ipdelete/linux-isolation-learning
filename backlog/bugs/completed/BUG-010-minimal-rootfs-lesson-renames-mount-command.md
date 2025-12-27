# Bug: Minimal-rootfs lesson instructs renaming/removing `Mount` subcommand, conflicting with prior lesson

## Summary
`docs/01-namespaces/05-minimal-rootfs.md` suggests removing or commenting out the existing `Mount` command and introducing a new `chroot` command, which can conflict with `docs/01-namespaces/04-mount-namespace.md` where learners implement `ns-tool mount`.

## Location
- `docs/01-namespaces/04-mount-namespace.md`
- `docs/01-namespaces/05-minimal-rootfs.md`
- `crates/ns-tool/src/main.rs` (implied by the lesson steps)

## Problem
The lesson sequence is unclear: following lesson 05 as written may invalidate or remove the earlier `mount` subcommand, breaking continuity and potentially invalidating earlier tests/docs.

## Steps to reproduce
1. Follow `docs/01-namespaces/04-mount-namespace.md` and implement `ns-tool mount`.
2. Start `docs/01-namespaces/05-minimal-rootfs.md`.
3. Observe the instruction to remove/comment out `Mount` in the `Command` enum.

## Expected
Lesson 05 should build on lesson 04 without requiring deleting/renaming earlier subcommands, or should explicitly state that lesson 05 supersedes lesson 04 and update the earlier lesson/tests accordingly.

## Actual
Lesson 05 instructs changing the CLI shape (removing `Mount`) while lesson 04 assumes it exists.

## Suggested fix
- Keep `Mount` and add `Chroot` (or `Rootfs`) as an additional subcommand, or
- If renaming is intentional, update lesson 04 and any tests/docs to match the new command name and ordering.
- Ensure the "Next" pointers and test filenames align with the final command names.

