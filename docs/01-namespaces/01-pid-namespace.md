# 01 PID Namespace

## Goal
- Create a PID namespace and observe PID 1 behavior.

## Prereqs
- You can build the workspace.

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
1) Implement `ns-tool pid` using `unshare(CLONE_NEWPID)` and `fork()`.
2) In the child, `exec` a command (default to `/bin/sh`).

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
sudo cargo run -q -p ns-tool -- pid -- /bin/sh -c 'echo "pid=$$"; ps -o pid,comm'
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- PID 1 has special signal handling behavior.

## Next
- `02-unshare-vs-clone.md`
