# 04 Container Lifecycle

## Goal
- Use create/start/exec/kill/delete to manage a container.

## Prereqs
- Completed `03-run-basic.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Add a checklist of `runc` commands to run in order.

## Verify
- Automated: `cargo test -p oci-tool`
- Manual:
```bash
runc list
```

## Clean Up
- TBD: How to clean up container state

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will map these steps to Rust helpers later.

## Next
- `05-seccomp.md`
