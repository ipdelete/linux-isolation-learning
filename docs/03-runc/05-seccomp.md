# 05 Seccomp

## Goal
- Add a minimal seccomp profile and see a blocked syscall.

## Prereqs
- Completed `04-lifecycle.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Modify `config.json` to deny a syscall and run the container.

## Verify
- Automated: `cargo test -p oci-tool`
- Manual:
```bash
# TODO: add a verification step once the profile is defined
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will build the JSON in Rust later.

## Next
- `06-network-integration.md`
