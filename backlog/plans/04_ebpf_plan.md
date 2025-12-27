# eBPF Tutorial Implementation Plan

Add tracing-focused eBPF tutorials using Aya to the existing Linux isolation learning repository.

## Summary

- **Framework**: Aya (pure Rust, no C toolchain)
- **Scope**: Tracing focus (kprobes, uprobes, tracepoints, perf events)
- **Location**: `docs/04-ebpf/` + `crates/ebpf-tool*/`
- **Lessons**: 9 tutorials following TDD pattern

## Crate Structure

```
crates/
  ebpf-tool/                    # Userspace CLI (clap-based)
  ebpf-tool-ebpf/               # eBPF programs (no_std, compiles to BPF)
  ebpf-tool-common/             # Shared types (no_std compatible)
```

Note: `ebpf-tool-ebpf` is NOT a workspace member (different compilation target).

## Lesson Sequence

| # | Title | Deliverable |
|---|-------|-------------|
| 00 | eBPF Setup | `ebpf-tool check` - validate BTF, kernel, permissions |
| 01 | Hello Kprobe | `ebpf-tool kprobe <fn>` - attach and log |
| 02 | Reading Kernel Data | Kprobe that reads syscall arguments |
| 03 | eBPF Maps | HashMap counter + `ebpf-tool stats` |
| 04 | User-Kernel Comms | PerfEventArray for real-time events |
| 05 | Uprobes | `ebpf-tool uprobe <binary> <fn>` |
| 06 | Tracepoints | `ebpf-tool tracepoint <cat> <name>` |
| 07 | Perf Events | `ebpf-tool perf` - CPU sampling |
| 08 | Combining Techniques | Full syscall tracer |

## Files to Create

### Crates
- `crates/ebpf-tool/Cargo.toml`
- `crates/ebpf-tool/src/main.rs`
- `crates/ebpf-tool/build.rs`
- `crates/ebpf-tool/tests/check_test.rs` (+ one per lesson)
- `crates/ebpf-tool-ebpf/Cargo.toml`
- `crates/ebpf-tool-ebpf/src/main.rs`
- `crates/ebpf-tool-ebpf/src/kprobe.rs` (+ uprobe.rs, tracepoint.rs, perf.rs)
- `crates/ebpf-tool-common/Cargo.toml`
- `crates/ebpf-tool-common/src/lib.rs`

### Docs
- `docs/04-ebpf/00-ebpf-setup.md`
- `docs/04-ebpf/01-hello-kprobe.md`
- `docs/04-ebpf/02-reading-data.md`
- `docs/04-ebpf/03-maps.md`
- `docs/04-ebpf/04-perf-events.md`
- `docs/04-ebpf/05-uprobes.md`
- `docs/04-ebpf/06-tracepoints.md`
- `docs/04-ebpf/07-perf-sampling.md`
- `docs/04-ebpf/08-combining.md`

## Files to Modify

- `/workspaces/linux-isolation-learning/Cargo.toml` - Add workspace members + deps (aya, tokio, etc.)
- `/workspaces/linux-isolation-learning/.devcontainer/devcontainer.json` - Add llvm, clang, bpf-linker, debug mounts
- `/workspaces/linux-isolation-learning/README.md` - Add 04-ebpf section to TOC

## Key Dependencies

```toml
# Add to [workspace.dependencies]
aya = "0.13"
aya-log = "0.2"
tokio = { version = "1", features = ["full"] }
log = "0.4"
env_logger = "0.11"
```

## Devcontainer Changes

```json
"postCreateCommand": "... && rustup component add rust-src && cargo install bpf-linker",
"mounts": [
  "source=/sys/kernel/debug,target=/sys/kernel/debug,type=bind",
  "source=/sys/kernel/btf,target=/sys/kernel/btf,type=bind,readonly"
]
```

## Implementation Order

1. **Setup Phase**
   - Create crate scaffolding (ebpf-tool, ebpf-tool-ebpf, ebpf-tool-common)
   - Update Cargo.toml workspace
   - Update devcontainer.json
   - Write lesson 00 (setup/check)

2. **Core Lessons** (01-04)
   - Each lesson: write test file with TODOs, write doc, implement code

3. **Probe Types** (05-07)
   - Uprobe, tracepoint, perf event lessons

4. **Integration** (08)
   - Combining lesson
   - Update README TOC
   - Add appendix entries if needed

## Testing Strategy

- **Layer 1**: CLI help/args tests (no root)
- **Layer 2**: `check` subcommand (no root)
- **Layer 3**: Load/attach tests (requires root, short duration)
- **Layer 4**: Functional tests (root + workload triggering)

All tests use `assert_cmd` with root privilege checks and early skip pattern.
