# eBPF Tutorial Implementation Plan

Add tracing-focused eBPF tutorials using Aya to the existing Linux isolation learning repository.

## Summary

- **Framework**: Aya (pure Rust, no C toolchain)
- **Scope**: Tracing focus (kprobes, uprobes, tracepoints, perf events)
- **Location**: `docs/04-ebpf/` + `crates/ebpf-tool*/`
- **Lessons**: 9 tutorials following TDD pattern

## How We'll Work (TDD Approach)

- **TDD approach**: Each lesson has learners write tests first, then implement code (red → green).
- Learners write both tests and implementation in `crates/ebpf-tool*/` by following lesson steps.
- Each lesson references specific TODO locations for tests and implementation.
- Test files have `todo!()` stubs with hints; implementation has `todo!()` in match arms.
- Keep lessons small (~30-50 minutes each).

## Success Criteria (Per Lesson)

- One concept + one small deliverable (usually a single subcommand or eBPF program).
- **TDD workflow**: Write test(s) first (red) → implement code (green) → refactor if needed.
- "Verify" proves correctness via automated tests + manual inspection.
- "Common Errors" captures the top 2-4 pitfalls.

## Crate Structure

```
crates/
  ebpf-tool/                    # Userspace CLI (clap-based, workspace member)
    src/main.rs                 # Subcommands with todo!() stubs
    build.rs                    # Builds eBPF programs
    tests/
      check_test.rs             # Tests with todo!() for lesson 00
      kprobe_test.rs            # Tests with todo!() for lesson 01
      ...
  ebpf-tool-ebpf/               # eBPF programs (no_std, NOT a workspace member)
    src/
      main.rs                   # eBPF entry points with todo!() stubs
      kprobe.rs                 # Kprobe programs
      uprobe.rs                 # Uprobe programs
      tracepoint.rs             # Tracepoint programs
      perf.rs                   # Perf event programs
  ebpf-tool-common/             # Shared types (no_std compatible, workspace member)
    src/lib.rs                  # Event structs shared between userspace and eBPF
```

Note: `ebpf-tool-ebpf` is NOT a workspace member (different compilation target).

## Lesson Sequence

| # | Title | Deliverable | Test File | Implementation |
|---|-------|-------------|-----------|----------------|
| 00 | eBPF Setup | `ebpf-tool check` - validate BTF, kernel, permissions | `check_test.rs` | `main.rs` Command::Check |
| 01 | Hello Kprobe | `ebpf-tool kprobe <fn>` - attach and log | `kprobe_test.rs` | `main.rs` + `kprobe.rs` |
| 02 | Reading Kernel Data | Kprobe that reads syscall arguments | (extends kprobe_test.rs) | `kprobe.rs` extensions |
| 03 | eBPF Maps | HashMap counter + `ebpf-tool stats` | `stats_test.rs` | `main.rs` Command::Stats |
| 04 | User-Kernel Comms | PerfEventArray for real-time events | `perf_test.rs` | `perf.rs` |
| 05 | Uprobes | `ebpf-tool uprobe <binary> <fn>` | `uprobe_test.rs` | `main.rs` + `uprobe.rs` |
| 06 | Tracepoints | `ebpf-tool tracepoint <cat> <name>` | `tracepoint_test.rs` | `main.rs` + `tracepoint.rs` |
| 07 | Perf Events | `ebpf-tool perf` - CPU sampling | (extends perf_test.rs) | `perf.rs` extensions |
| 08 | Combining | Full syscall tracer | `tracer_test.rs` | Combined implementation |

## Files to Create

### Phase 1: Scaffolding (with TODOs)

**ebpf-tool-common** (shared types):
- `crates/ebpf-tool-common/Cargo.toml`
- `crates/ebpf-tool-common/src/lib.rs` - Event structs with TODO comments for expansion

**ebpf-tool** (userspace CLI with stubs):
- `crates/ebpf-tool/Cargo.toml`
- `crates/ebpf-tool/build.rs` - Build script for eBPF compilation
- `crates/ebpf-tool/src/main.rs` - Subcommands with `todo!()` stubs

**ebpf-tool tests** (test files with TODOs):
- `crates/ebpf-tool/tests/check_test.rs`
- `crates/ebpf-tool/tests/kprobe_test.rs`
- `crates/ebpf-tool/tests/uprobe_test.rs`
- `crates/ebpf-tool/tests/tracepoint_test.rs`
- `crates/ebpf-tool/tests/perf_test.rs`

**ebpf-tool-ebpf** (eBPF programs with stubs):
- `crates/ebpf-tool-ebpf/Cargo.toml`
- `crates/ebpf-tool-ebpf/src/main.rs`
- `crates/ebpf-tool-ebpf/src/kprobe.rs`
- `crates/ebpf-tool-ebpf/src/uprobe.rs`
- `crates/ebpf-tool-ebpf/src/tracepoint.rs`
- `crates/ebpf-tool-ebpf/src/perf.rs`

### Phase 2: Docs (lessons guide through implementing TODOs)

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

- `Cargo.toml` - Add workspace members + deps (aya, tokio, etc.)
- `.devcontainer/devcontainer.json` - Add llvm, clang, bpf-linker, debug mounts
- `README.md` - Add 04-ebpf section to TOC

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

## Testing Strategy (TDD Approach)

- **Red/Green/Refactor**: Each lesson follows Test-Driven Development.
- **Write tests first**: Lessons instruct learners to write tests before implementing.
- **Test types by layer**:
  - Layer 1: CLI help/args tests (no root)
  - Layer 2: `check` subcommand validation (no root)
  - Layer 3: Load/attach tests (requires root, short duration)
  - Layer 4: Functional tests (root + workload triggering)
- **Test location**: Tests go in `crates/ebpf-tool/tests/` with `todo!()` stubs.
- **Verify section**: Manual checks complement automated tests.

All tests use `assert_cmd` with root privilege checks and early skip pattern.

## Implementation Order

1. **Setup Phase** (scaffolding with TODOs)
   - Update Cargo.toml workspace
   - Update devcontainer.json
   - Create ebpf-tool-common (lib.rs with event structs)
   - Create ebpf-tool scaffolding (main.rs with todo!() stubs)
   - Create test files with todo!() stubs
   - Create ebpf-tool-ebpf scaffolding

2. **Lesson Docs** (guide learners through implementing)
   - Write lesson 00 (setup/check) - guides through check_test.rs + Command::Check
   - Write lessons 01-04 (core lessons)
   - Write lessons 05-07 (probe types)
   - Write lesson 08 (integration)

3. **Finalize**
   - Update README TOC
   - Add appendix entries if needed
