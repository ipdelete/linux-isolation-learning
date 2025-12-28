# Bug: CPU limits lesson missing cgroup v2 controller/delegation setup

## Summary
The CPU limits lesson assumes `cpu.max` is writable in `/sys/fs/cgroup/<name>`, but in environments where the CPU controller is not enabled/delegated, the command fails with `Permission denied`.

## Location
- `docs/fast-track/07-cpu-limits.md`
- `crates/contain/src/cgroup.rs` (CPU handler writes `cpu.max`)

## Problem
On cgroup v2 systems, a controller must be enabled in the parent subtree (`cgroup.subtree_control`) before it can be used in a child cgroup. In this environment, `cgroup.subtree_control` is empty and cannot be updated (write fails with "Device or resource busy"), so writing `cpu.max` is denied.

## Steps to reproduce
1. Run `cargo run -p contain -- cgroup create /sys/fs/cgroup/cpulimit`.
2. Run `cargo run -p contain -- cgroup cpu /sys/fs/cgroup/cpulimit 50000`.

## Expected
`cpu.max` is updated to `50000 100000` and the command succeeds.

## Actual
The CPU command fails with `Error: Permission denied (os error 13)`.

## Impact
Learners following the lesson cannot set CPU limits unless the CPU controller is enabled/delegated. The lesson doesn't explain this prerequisite or provide a workaround.

## Status
OPEN
