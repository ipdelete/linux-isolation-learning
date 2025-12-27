# 03 Run Basic Container

## Goal
- Use `runc` to run the bundle you created.

## Prereqs
- Completed `02-config-json.md`
- `runc` installed on the system

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Add a simple rootfs (busybox) and run with `runc run`.

## Verify
- Automated: `cargo test -p oci-tool`
- Manual:
```bash
sudo runc run test-container
```

## Clean Up
- TBD: How to stop and remove the container

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will keep the rootfs minimal.

## Next
- `04-lifecycle.md`
