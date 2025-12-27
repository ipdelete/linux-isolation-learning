# 06 Unsafe Boundaries

## Goal
- Understand where and why we use `unsafe` in syscall code.

## Prereqs
- You have read `crates/ns-tool/src/main.rs`.

## Build
1) Identify the minimal `unsafe` blocks we add per syscall.
2) Keep each block small and wrap it in a safe function.

## Verify
```bash
cargo run -q -p ns-tool -- proc
```

## Notes
- We avoid `unsafe` unless a libc call or raw pointer is required.
