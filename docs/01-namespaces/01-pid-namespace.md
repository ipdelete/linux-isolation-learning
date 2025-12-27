# 01 PID Namespace

## Goal
- Create a PID namespace and observe PID 1 behavior.

## Prereqs
- You can build the workspace.

## Build
1) Implement `ns-tool pid` using `unshare(CLONE_NEWPID)` and `fork()`.
2) In the child, `exec` a command (default to `/bin/sh`).

## Verify
```bash
sudo cargo run -q -p ns-tool -- pid -- /bin/sh -c 'echo "pid=$$"; ps -o pid,comm'
```

## Notes
- PID 1 has special signal handling behavior.
