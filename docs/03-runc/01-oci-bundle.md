# 01 OCI Bundle

## Goal
- Create a minimal OCI bundle layout.

## Prereqs
- Completed namespace and cgroup lessons

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `oci-tool init` to create `config.json` + `rootfs/`.

## Verify
- Automated: `cargo test -p oci-tool`
- Manual:
```bash
cargo run -q -p oci-tool -- init ./bundle
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will use `runc spec` as a reference, not as a dependency.

## Next
- `02-config-json.md`
