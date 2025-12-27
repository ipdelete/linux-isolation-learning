# 02 config.json

## Goal
- Read and write a minimal OCI `config.json` in Rust.

## Prereqs
- Completed `01-oci-bundle.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `oci-tool show` to parse and print `config.json`.

## Verify
- Automated: `cargo test -p oci-tool`
- Manual:
```bash
cargo run -q -p oci-tool -- show ./bundle
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will keep the JSON schema minimal at first.

## Next
- `03-run-basic.md`
