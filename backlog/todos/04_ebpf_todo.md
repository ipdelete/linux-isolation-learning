# eBPF Tutorial TODO (Aya-based, TDD approach)

## Phase 1: Setup & Configuration
- [x] Update Cargo.toml (workspace members + dependencies)
- [x] Update .devcontainer/devcontainer.json (llvm, bpf-linker, mounts)
- [x] Update README.md (add 04-ebpf section to TOC)

## Phase 2: Crate Scaffolding (with TODOs for learners)

### crates/ebpf-tool-common (shared types)
- [x] crates/ebpf-tool-common/Cargo.toml
- [x] crates/ebpf-tool-common/src/lib.rs (event structs, no_std)

### crates/ebpf-tool (userspace CLI with todo!() stubs)
- [x] crates/ebpf-tool/Cargo.toml
- [x] crates/ebpf-tool/build.rs
- [x] crates/ebpf-tool/src/main.rs (Command enum with todo!() for each subcommand)

### crates/ebpf-tool tests (test files with todo!() stubs)
- [x] crates/ebpf-tool/tests/check_test.rs (lesson 00)
- [x] crates/ebpf-tool/tests/kprobe_test.rs (lessons 01-02)
- [x] crates/ebpf-tool/tests/stats_test.rs (lesson 03)
- [x] crates/ebpf-tool/tests/perf_test.rs (lessons 04, 07)
- [x] crates/ebpf-tool/tests/uprobe_test.rs (lesson 05)
- [x] crates/ebpf-tool/tests/tracepoint_test.rs (lesson 06)
- [x] crates/ebpf-tool/tests/tracer_test.rs (lesson 08)

### crates/ebpf-tool-ebpf (eBPF programs with todo!() stubs)
- [ ] crates/ebpf-tool-ebpf/Cargo.toml
- [ ] crates/ebpf-tool-ebpf/src/main.rs
- [ ] crates/ebpf-tool-ebpf/src/kprobe.rs
- [ ] crates/ebpf-tool-ebpf/src/uprobe.rs
- [ ] crates/ebpf-tool-ebpf/src/tracepoint.rs
- [ ] crates/ebpf-tool-ebpf/src/perf.rs

## Phase 3: Lesson Docs (guide learners through TDD)

Each lesson follows: Write Tests (Red) → Build (Green) → Verify

- [ ] docs/04-ebpf/00-ebpf-setup.md (check_test.rs → Command::Check)
- [ ] docs/04-ebpf/01-hello-kprobe.md (kprobe_test.rs → kprobe.rs)
- [ ] docs/04-ebpf/02-reading-data.md (extend kprobe tests → extend kprobe.rs)
- [ ] docs/04-ebpf/03-maps.md (stats_test.rs → Command::Stats + maps)
- [ ] docs/04-ebpf/04-perf-events.md (perf_test.rs → perf.rs)
- [ ] docs/04-ebpf/05-uprobes.md (uprobe_test.rs → uprobe.rs)
- [ ] docs/04-ebpf/06-tracepoints.md (tracepoint_test.rs → tracepoint.rs)
- [ ] docs/04-ebpf/07-perf-sampling.md (extend perf_test.rs → extend perf.rs)
- [ ] docs/04-ebpf/08-combining.md (tracer_test.rs → combined tracer)
