# 07 Cgroups Integration

## Goal
- Attach an OCI container to a pre-made cgroup.

## Prereqs
- Completed `06-network-integration.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Use `cgroup-tool` to prepare limits and update `config.json`.

## Verify
- Automated: `cargo test -p oci-tool`
- Manual:
```bash
# TODO: add once cgroup integration is in place
```

## Clean Up
- TBD: How to clean up cgroup resources

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will keep limits small while testing.

## Next
- Lessons complete! Explore the appendix: `../90-appendix/01-rust-syscall-cheatsheet.md`
