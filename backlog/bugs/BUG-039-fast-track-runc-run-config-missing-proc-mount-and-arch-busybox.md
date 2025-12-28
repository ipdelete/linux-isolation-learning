# Bug: runc run lesson missing /proc mount and hardcodes x86_64 busybox

## Summary
The runc run lesson's minimal `config.json` omits a `/proc` mount, causing `runc run` to fail. The setup also downloads an x86_64 busybox binary, which fails on arm64 environments.

## Location
- `docs/fast-track/09-runc-run.md`

## Problem
1) The non-interactive example `config.json` lacks a `mounts` entry for `/proc`, which runc expects. This leads to `error closing exec fds: open /proc/self/fd: no such file or directory`.
2) The setup step downloads `busybox` for `x86_64-linux-musl`, which doesn't run on `aarch64` systems.

## Steps to reproduce
1. Follow the "Setup" steps to create `/tmp/testcontainer` and rootfs.
2. Apply the non-interactive `config.json` from the lesson.
3. Run `sudo runc run testrun`.

## Expected
`runc run` prints:
```
Hello from container
PID   USER     COMMAND
    1 root     /bin/sh -c echo Hello from container && ps aux
    2 root     ps aux
```

## Actual
- `runc run` fails with `error closing exec fds: open /proc/self/fd: no such file or directory`.
- On arm64, the busybox binary is incompatible.

## Impact
Learners cannot complete the runc run lesson without adding a `/proc` mount and using an architecture-appropriate busybox binary.

## Status
OPEN
