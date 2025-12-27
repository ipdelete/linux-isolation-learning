# Bug: eBPF docs reference `cargo xtask build-ebpf` but no `xtask` exists

## Summary
Multiple `docs/04-ebpf/*` lessons instruct learners to run `cargo xtask build-ebpf`, but the workspace has no `xtask` crate and no `cargo-xtask` integration. This blocks learners from building the eBPF programs using the documented path.

## Location
- `docs/04-ebpf/01-hello-kprobe.md` (build instructions)
- `docs/04-ebpf/02-reading-data.md` (build instructions)
- `docs/04-ebpf/03-maps.md` (build instructions)
- `docs/04-ebpf/04-perf-events.md` (build instructions)
- `docs/04-ebpf/05-uprobes.md` (build instructions)
- `docs/04-ebpf/06-tracepoints.md` (build instructions)
- `docs/04-ebpf/08-combining.md` (build instructions)

## Problem
The repo’s root `Cargo.toml` does not list an `xtask` workspace member, and there is no `crates/xtask` (or similar) providing a `build-ebpf` subcommand.

## Steps to reproduce
1. Run `cargo xtask build-ebpf`.
2. Observe Cargo fails because there is no `xtask` binary/command in the workspace.

## Expected
Docs only reference build commands that exist in the repository, or the repository includes the referenced `xtask` implementation.

## Actual
Docs instruct a non-existent command, blocking progress.

## Suggested fix
- Either add an `xtask` crate implementing `cargo xtask build-ebpf`, or
- Update docs to match the repo’s actual build flow (currently `crates/ebpf-tool/build.rs` compiling `crates/ebpf-tool-ebpf` and copying the artifact into `OUT_DIR`).

