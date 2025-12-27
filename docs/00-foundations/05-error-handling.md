# 05 Error Handling

## Goal
- Use `anyhow` to surface syscall errors clearly.

## Prereqs
- Rust workspace builds.

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
1) Note how `main()` returns `Result<()>`.
2) When a syscall fails, we bubble up the error and let the CLI print it.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
cargo run -q -p ns-tool -- pid
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will add specific context with `anyhow::Context` later.

## Next
- `06-unsafe-boundaries.md`
