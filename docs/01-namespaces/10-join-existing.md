# 10 Join Existing Namespaces

## Goal
- Use `setns()` to join an existing namespace.

## Build
- Implement `ns-tool setns` with a PID target.

## Verify
```bash
sudo cargo run -q -p ns-tool -- setns
```

## Notes
- This is how tools like `nsenter` work.
