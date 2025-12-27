# Bug: eBPF docs use incorrect `include_bytes_aligned!` paths instead of `OUT_DIR`

## Summary
Several `docs/04-ebpf/*` lessons show userspace code embedding eBPF artifacts via hard-coded paths like `../../target/bpfel-unknown-none/...` or `../../ebpf-tool-ebpf/target/...`. In this repo, the intended build flow is `crates/ebpf-tool/build.rs`, which places the compiled eBPF ELF in the userspace crate’s `OUT_DIR` and exports `EBPF_OUT_DIR`.

## Location
- `docs/04-ebpf/01-hello-kprobe.md` (hard-coded `target/bpfel-unknown-none/...` path)
- `docs/04-ebpf/02-reading-data.md` (hard-coded `target/bpfel-unknown-none/...` path)
- `docs/04-ebpf/04-perf-events.md` (hard-coded `../../ebpf-tool-ebpf/target/...` path)
- `docs/04-ebpf/05-uprobes.md` (hard-coded `.../debug/uprobe` / `.../release/uprobe` paths)
- `docs/04-ebpf/06-tracepoints.md` (hard-coded `.../release/tracepoint` path)
- `docs/04-ebpf/07-perf-sampling.md` (hard-coded `../../ebpf-tool-ebpf/target/...` path)

## Problem
These file paths do not exist (or do not match how artifacts are produced) in the current repo. The userspace embedding should refer to the artifact produced by `build.rs`, typically:

- `include_bytes_aligned!(concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf"))`

## Steps to reproduce
1. Follow any of the lessons above and paste the `include_bytes_aligned!` snippet as written.
2. Run `cargo build -p ebpf-tool`.
3. Observe compilation fails because the referenced file path is missing.

## Expected
Docs instruct embedding the eBPF artifact using the repo’s actual build output location (the userspace crate `OUT_DIR`).

## Actual
Docs point to nonexistent or repo-inconsistent paths.

## Suggested fix
- Update all userspace snippets to use `OUT_DIR` (and/or `EBPF_OUT_DIR`) consistently.
- Remove “per-subcommand” artifact path guidance unless the repo actually produces separate `uprobe`, `tracepoint`, etc. ELF files.

