# 03 Procfs Intro

## Goal
- Read namespace data from `/proc` using Rust.

## Prereqs
- `ns-tool` builds.

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
1) Run the `ns-tool proc` command.
2) Compare the output with a raw `readlink`.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
cargo run -q -p ns-tool -- proc
readlink /proc/$$/ns/pid
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- The symlink inode values are how we compare namespaces.

## Next
- `04-permissions-and-sudo.md`
