# 02 config.json

## Goal
- Read and write a minimal OCI `config.json` in Rust.

## Build
- Implement `oci-tool show` to parse and print `config.json`.

## Verify
```bash
cargo run -q -p oci-tool -- show ./bundle
```

## Notes
- We will keep the JSON schema minimal at first.
