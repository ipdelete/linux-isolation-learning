# Bug: Mount-namespace lesson uses `rmount` instead of `umount` in cleanup snippet

## Summary
`docs/01-namespaces/04-mount-namespace.md` includes a cleanup snippet that calls `sudo rmount ...`, which is not a standard Linux command (likely a typo for `umount`).

## Location
- `docs/01-namespaces/04-mount-namespace.md`

## Problem
Learners copying the cleanup snippet will get a command-not-found error and may leave behind mount points/directories.

## Steps to reproduce
1. Open `docs/01-namespaces/04-mount-namespace.md`.
2. Find the cleanup example that includes `sudo rmount /mnt/isolated_test ...`.
3. Run it on a typical Linux system.

## Expected
Cleanup examples should use valid commands (typically `umount`), e.g.:
- `sudo umount /mnt/isolated_test 2>/dev/null || true`

## Actual
The doc uses `sudo rmount ...`, which fails.

## Suggested fix
- Replace `rmount` with `umount` in the cleanup snippet(s).
- Consider using `umount -l` where the lesson discusses busy mounts (optional).

