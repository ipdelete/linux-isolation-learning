# 04 Mount Namespace

## Goal
- Create a mount namespace and verify mount isolation.

## Build
- Implement `ns-tool mount` with a simple tmpfs mount.

## Verify
```bash
sudo cargo run -q -p ns-tool -- mount
```

## Notes
- We will keep mount propagation private.
