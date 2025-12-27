# eBPF Tutorial TODO (Aya-based, tracing focus)

## Setup
- [x] Update Cargo.toml (workspace members + dependencies)
- [x] Update .devcontainer/devcontainer.json (llvm, bpf-linker, mounts)
- [x] Update README.md (add 04-ebpf section to TOC)

## crates/ebpf-tool-common
- [ ] crates/ebpf-tool-common/Cargo.toml
- [ ] crates/ebpf-tool-common/src/lib.rs

## crates/ebpf-tool-ebpf
- [ ] crates/ebpf-tool-ebpf/Cargo.toml
- [ ] crates/ebpf-tool-ebpf/src/main.rs
- [ ] crates/ebpf-tool-ebpf/src/kprobe.rs
- [ ] crates/ebpf-tool-ebpf/src/uprobe.rs
- [ ] crates/ebpf-tool-ebpf/src/tracepoint.rs
- [ ] crates/ebpf-tool-ebpf/src/perf.rs

## crates/ebpf-tool
- [ ] crates/ebpf-tool/Cargo.toml
- [ ] crates/ebpf-tool/build.rs
- [ ] crates/ebpf-tool/src/main.rs
- [ ] crates/ebpf-tool/tests/check_test.rs
- [ ] crates/ebpf-tool/tests/kprobe_test.rs
- [ ] crates/ebpf-tool/tests/uprobe_test.rs
- [ ] crates/ebpf-tool/tests/tracepoint_test.rs
- [ ] crates/ebpf-tool/tests/perf_test.rs

## docs/04-ebpf
- [ ] docs/04-ebpf/00-ebpf-setup.md
- [ ] docs/04-ebpf/01-hello-kprobe.md
- [ ] docs/04-ebpf/02-reading-data.md
- [ ] docs/04-ebpf/03-maps.md
- [ ] docs/04-ebpf/04-perf-events.md
- [ ] docs/04-ebpf/05-uprobes.md
- [ ] docs/04-ebpf/06-tracepoints.md
- [ ] docs/04-ebpf/07-perf-sampling.md
- [ ] docs/04-ebpf/08-combining.md
