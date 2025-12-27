# 01 PID Namespace

## Goal
- Create a PID namespace and observe PID 1 behavior.

## Build
- Implement `ns-tool pid` with `clone()` or `unshare()`.

## Verify
```bash
sudo cargo run -q -p ns-tool -- pid
```

## Notes
- PID 1 has special signal handling behavior.
