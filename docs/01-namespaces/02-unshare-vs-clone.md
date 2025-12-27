# 02 unshare() vs clone()

## Goal
- Compare `unshare()` and `clone()` for namespace creation.

## Build
- Add a flag to `ns-tool pid` to switch between approaches.

## Verify
```bash
sudo cargo run -q -p ns-tool -- pid --help
```

## Notes
- We will keep the process tree small and visible with `ps`.
