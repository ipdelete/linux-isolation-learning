# Bug: eBPF docs use `bytes::BytesMut` but `ebpf-tool` does not depend on `bytes`

## Summary
Multiple `docs/04-ebpf/*` lessons include userspace code that allocates buffers via `bytes::BytesMut`, but `crates/ebpf-tool/Cargo.toml` does not list `bytes` as a dependency. Learners following the docs will get unresolved import errors.

## Location
- `docs/04-ebpf/02-reading-data.md` (imports `bytes::BytesMut`)
- `docs/04-ebpf/04-perf-events.md` (imports `bytes::BytesMut`)
- `docs/04-ebpf/07-perf-sampling.md` (imports `bytes::BytesMut`)
- `docs/04-ebpf/08-combining.md` (imports `bytes::BytesMut`)
- `crates/ebpf-tool/Cargo.toml` (missing `bytes` dependency)

## Problem
Docs imply a dependency that isn’t present in the project configuration.

## Steps to reproduce
1. Copy any snippet using `use bytes::BytesMut;` into `crates/ebpf-tool/src/main.rs`.
2. Run `cargo build -p ebpf-tool`.
3. Observe error `failed to resolve: use of undeclared crate or module 'bytes'`.

## Expected
Either:
- Docs use only dependencies already present, or
- Docs explicitly instruct adding `bytes = ...` to `crates/ebpf-tool/Cargo.toml`.

## Actual
Docs assume `bytes` is available when it is not.

## Suggested fix
- Add `bytes` to `crates/ebpf-tool/Cargo.toml` (and workspace deps if preferred), or
- Rewrite snippets to avoid `bytes` and use another buffer strategy consistent with existing deps.

