# 06 Multi-Resource Cgroup

## Goal
- Combine memory, CPU, I/O, and PID limits.

## Prereqs
- Completed `05-pids.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Add a helper subcommand that applies a bundle of limits.

## Verify
- Automated: `cargo test -p cgroup-tool`
- Manual:
```bash
# TODO: add command once implemented
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- Keep limits low while testing.

## Next
- Move to OCI/runc lessons: `../03-runc/01-oci-bundle.md`
