# 06 Unsafe Boundaries

## Goal
- Understand where and why we use `unsafe` in syscall code.

## Prereqs
- You have read `crates/ns-tool/src/main.rs`.

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
1) Identify the minimal `unsafe` blocks we add per syscall.
2) Keep each block small and wrap it in a safe function.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
cargo run -q -p ns-tool -- proc
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We avoid `unsafe` unless a libc call or raw pointer is required.

## Next
- Move to namespace lessons: `../01-namespaces/01-pid-namespace.md`
