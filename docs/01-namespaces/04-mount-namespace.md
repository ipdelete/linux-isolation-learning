# 04 Mount Namespace

## Goal
- Create a mount namespace and verify mount isolation.

## Prereqs
- Completed `03-uts-ipc.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `ns-tool mount` with a simple tmpfs mount.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
sudo cargo run -q -p ns-tool -- mount
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will keep mount propagation private.

## Next
- `05-minimal-rootfs.md`
