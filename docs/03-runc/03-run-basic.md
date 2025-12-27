# 03 Run Basic Container

## Goal
- Use `runc` to run the bundle you created.

## Build
- Add a simple rootfs (busybox) and run with `runc run`.

## Verify
```bash
sudo runc run test-container
```

## Notes
- We will keep the rootfs minimal.
