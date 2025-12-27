# 01 OCI Bundle

## Goal
- Create a minimal OCI bundle layout.

## Build
- Implement `oci-tool init` to create `config.json` + `rootfs/`.

## Verify
```bash
cargo run -q -p oci-tool -- init ./bundle
```

## Notes
- We will use `runc spec` as a reference, not as a dependency.
