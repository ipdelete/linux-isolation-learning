# 06 Network Integration

## Goal
- Attach an OCI container to a pre-made network namespace.

## Prereqs
- Completed `05-seccomp.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Use `netns-tool` to prepare network and update `config.json`.

## Verify
- Automated: `cargo test -p oci-tool`
- Manual:
```bash
# TODO: add once network namespace integration is in place
```

## Clean Up
- TBD: How to clean up network resources

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- This mirrors how container runtimes wire networking.

## Next
- `07-cgroups-integration.md`
