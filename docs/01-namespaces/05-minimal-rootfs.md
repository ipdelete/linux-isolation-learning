# 05 Minimal Rootfs

## Goal
- Build a minimal root filesystem and enter it.

## Prereqs
- Completed `04-mount-namespace.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Create a rootfs directory and populate it with a shell and libs.
- Use `pivot_root` or `chroot` via `ns-tool mount`.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
sudo cargo run -q -p ns-tool -- mount
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- This lesson is a bridge to OCI bundles.

## Next
- `06-netns-basics.md`
