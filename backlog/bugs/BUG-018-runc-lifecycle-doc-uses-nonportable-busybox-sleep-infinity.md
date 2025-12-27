# Bug: Lifecycle lesson uses `sleep infinity`, which is not portable to BusyBox

## Summary
`docs/03-runc/04-lifecycle.md` instructs setting the container init process to `["sleep", "infinity"]`. BusyBox `sleep` commonly does not accept `infinity`, so the container may exit immediately instead of staying running for lifecycle exercises.

## Location
- `docs/03-runc/04-lifecycle.md` (config.json modification steps; uses `sleep infinity`)

## Problem
The lesson assumes GNU coreutils semantics. In many minimal rootfs setups (BusyBox), `sleep infinity` fails (invalid time interval), breaking the create/start/exec exercises.

## Steps to reproduce
1. Build the rootfs in `docs/03-runc/04-lifecycle.md` using BusyBox.
2. Set `process.args` to `["sleep","infinity"]`.
3. `sudo runc start ...` then check state / container status.

## Expected
The container remains running long enough to practice `runc exec`, `runc state`, and signals.

## Actual
The process may exit immediately due to `sleep` rejecting `infinity`.

## Suggested fix
- Use a BusyBox-friendly long-running command, e.g.:
  - `["sh", "-c", "while true; do sleep 3600; done"]`, or
  - `["sleep", "2147483647"]` (large value), with a note about portability.

