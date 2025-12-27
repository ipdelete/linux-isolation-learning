# 05 Minimal Rootfs

## Goal
- Build a minimal root filesystem and enter it.

## Build
- Create a rootfs directory and populate it with a shell and libs.
- Use `pivot_root` or `chroot` via `ns-tool mount`.

## Verify
```bash
sudo cargo run -q -p ns-tool -- mount
```

## Notes
- This lesson is a bridge to OCI bundles.
