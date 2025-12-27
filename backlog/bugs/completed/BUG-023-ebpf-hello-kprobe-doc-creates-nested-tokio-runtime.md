# Bug: `01-hello-kprobe.md` suggests creating a nested Tokio runtime

## Summary
`docs/04-ebpf/01-hello-kprobe.md` includes a userspace implementation snippet that constructs a Tokio runtime via `tokio::runtime::Builder` and calls `block_on`. In this repo, `crates/ebpf-tool/src/main.rs` already uses `#[tokio::main]`, so creating a nested runtime inside `main` will panic at runtime.

## Location
- `docs/04-ebpf/01-hello-kprobe.md` (userspace implementation snippet around the runtime builder)
- `crates/ebpf-tool/src/main.rs` (`#[tokio::main] async fn main()`)

## Problem
Tokio does not allow starting a runtime from within an existing runtime by default; the documented snippet conflicts with the actual application structure.

## Steps to reproduce
1. Implement the `Command::Kprobe` match arm following the doc snippet (including `tokio::runtime::Builder::new_current_thread()`).
2. Run `cargo run -p ebpf-tool -- kprobe do_sys_openat2 -d 1`.
3. Observe a Tokio panic about starting a runtime from within a runtime.

## Expected
Docs should use the existing async context (await timers, `tokio::select!`, etc.) without constructing a new runtime.

## Actual
Docs suggest a pattern that will panic in this repo.

## Suggested fix
- Update the snippet to use async control flow directly inside the `#[tokio::main]` runtime (e.g., `tokio::select! { ... }`).

