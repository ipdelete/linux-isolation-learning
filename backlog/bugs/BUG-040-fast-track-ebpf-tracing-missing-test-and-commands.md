# Bug: Fast-track eBPF tracing lesson references missing test and unimplemented commands

## Summary
The eBPF tracing fast-track lesson references a `trace_test` and a `trace check` command that don't exist, and `trace syscalls` is unimplemented, so the documented steps fail.

## Location
- `docs/fast-track/10-ebpf-tracing.md`
- `crates/contain/src/trace.rs`
- (missing) `crates/contain/tests/trace_test.rs`

## Problem
The lesson instructs learners to run `cargo test -p contain --test trace_test` and `contain trace check`, but there is no `trace_test` file and no `check` subcommand in the `trace` CLI. The `trace syscalls` command panics due to `todo!()`.

## Steps to reproduce
1. Run `cargo test -p contain --test trace_test`.
2. Run `cargo run -p contain -- trace check`.
3. Run `cargo run -p contain -- trace syscalls`.

## Expected
- `trace_test` exists and passes.
- `contain trace check` succeeds.
- `contain trace syscalls` starts tracing.

## Actual
- `cargo test -p contain --test trace_test` fails with `no test target named 'trace_test'`.
- `contain trace check` is not a valid subcommand.
- `contain trace syscalls` panics with `not yet implemented: Implement syscall tracing`.

## Impact
Learners cannot complete the eBPF fast-track lesson as written.

## Status
OPEN
