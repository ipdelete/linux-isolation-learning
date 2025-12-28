# Bug: Memory limits lesson missing cgroup v2 controller/delegation setup

## Summary
The memory limits lesson assumes `memory.max` is writable in `/sys/fs/cgroup/<name>`, but in environments where the memory controller is not enabled/delegated, the command fails with `Permission denied`.

## Location
- `docs/fast-track/06-memory-limits.md`
- `crates/contain/src/cgroup.rs` (memory handler writes `memory.max`)

## Problem
On cgroup v2 systems, a controller must be enabled in the parent subtree (`cgroup.subtree_control`) before it can be used in a child cgroup. In this environment, `cgroup.subtree_control` is empty and cannot be updated (write fails with "Device or resource busy"), so writing `memory.max` is denied.

## Steps to reproduce
1. Run `cargo run -p contain -- cgroup create /sys/fs/cgroup/limited`.
2. Run `cargo run -p contain -- cgroup memory /sys/fs/cgroup/limited 50M`.

## Expected
`memory.max` is updated to `52428800` and the command succeeds.

## Actual
The memory command fails with `Error: Permission denied (os error 13)`.

## Impact
Learners following the lesson cannot set memory limits unless the memory controller is enabled/delegated. The lesson doesn't explain this prerequisite or provide a workaround.

## Resolution
Documented as a DevContainer limitation. Updated:
- `.devcontainer/validation.md` - Added Linux VM setup instructions
- `docs/fast-track/README.md` - Added environment compatibility table
- `docs/fast-track/06-memory-limits.md` - Added VM requirement warning

The fundamental issue is that Docker containers cannot enable cgroup controllers via `subtree_control` when processes exist in the root cgroup (the "no internal processes" rule). A Linux VM with systemd provides proper cgroup delegation.

## Status
RESOLVED (documented limitation with workaround)
