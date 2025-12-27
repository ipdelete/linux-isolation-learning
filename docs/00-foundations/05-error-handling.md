# 05 Error Handling

## Goal
- Use `anyhow` to surface syscall errors clearly.

## Prereqs
- Rust workspace builds.

## Build
1) Note how `main()` returns `Result<()>`.
2) When a syscall fails, we bubble up the error and let the CLI print it.

## Verify
```bash
cargo run -q -p ns-tool -- pid
```

## Notes
- We will add specific context with `anyhow::Context` later.
