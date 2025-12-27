# 01 Rust Syscall Basics

## Goal
- Understand how we call Linux syscalls from Rust.

## Prereqs
- `cargo build -q` succeeds.

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
1) Open `crates/ns-tool/src/main.rs`.
2) Notice we use:
   - `libc` for raw syscalls and constants
   - `nix` for safer wrappers when possible
3) Find `print_proc_ns()` and how it reads `/proc`.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
cargo run -q -p ns-tool -- proc
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We keep unsafe blocks tiny and isolated.
- Prefer `nix` wrappers until we need a raw syscall.

## Next
- `02-cli-patterns.md`
