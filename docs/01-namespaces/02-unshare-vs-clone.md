# 02 unshare() vs clone()

## Goal
- Compare `unshare()` and `clone()` for namespace creation.

## Prereqs
- Completed `01-pid-namespace.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Add a flag to `ns-tool pid` to switch between approaches.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
sudo cargo run -q -p ns-tool -- pid --help
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will keep the process tree small and visible with `ps`.

## Next
- `03-uts-ipc.md`
