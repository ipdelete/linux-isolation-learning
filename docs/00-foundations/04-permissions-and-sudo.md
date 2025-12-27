# 04 Permissions and sudo

## Goal
- Understand when root is required and how to verify permission errors.

## Prereqs
- `cargo run -q -p ns-tool -- proc` works.

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
1) Run a command that should fail without root.
2) Re-run it with `sudo`.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
cargo run -q -p ns-tool -- pid
sudo cargo run -q -p ns-tool -- pid
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- Many namespace and cgroup operations require root.
- We will keep cleanup steps explicit to avoid lingering state.

## Next
- `05-error-handling.md`
