# 04 Permissions and sudo

## Goal
- Understand when root is required and how to verify permission errors.

## Prereqs
- `cargo run -q -p ns-tool -- proc` works.

## Build
1) Run a command that should fail without root.
2) Re-run it with `sudo`.

## Verify
```bash
cargo run -q -p ns-tool -- pid
sudo cargo run -q -p ns-tool -- pid
```

## Notes
- Many namespace and cgroup operations require root.
- We will keep cleanup steps explicit to avoid lingering state.
