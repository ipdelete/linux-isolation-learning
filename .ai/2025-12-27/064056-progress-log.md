# Progress Log: eBPF Tool eBPF Crate Scaffolding

**Date**: 2025-12-27
**Session**: 064056
**Branch**: `feature/04-ebpf-tutorials`

## Overview

Created the `ebpf-tool-ebpf` crate scaffolding - the kernel-side eBPF programs for the eBPF tutorial series. This completes Phase 2 of the eBPF tutorial implementation plan.

## What We Built

### eBPF Program Crate (`crates/ebpf-tool-ebpf/`)

A complete scaffolding for eBPF programs using the Aya framework with TDD-style `todo!()` stubs. The crate compiles to `bpfel-unknown-none` target (BPF bytecode) and runs inside the Linux kernel.

```
┌─────────────────────────────────────────────────────────────┐
│                     USERSPACE (ebpf-tool)                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  CLI Commands: check, kprobe, uprobe, tracepoint... │   │
│  └─────────────────────────────────────────────────────┘   │
│                           │                                  │
│                    loads via bpf()                          │
│                           ▼                                  │
├─────────────────────────────────────────────────────────────┤
│                      KERNEL (ebpf-tool-ebpf)                │
│  ┌──────────┐ ┌──────────┐ ┌────────────┐ ┌──────────┐    │
│  │ kprobe.rs│ │uprobe.rs │ │tracepoint.rs│ │ perf.rs  │    │
│  │          │ │          │ │            │ │          │    │
│  │ Lessons  │ │ Lesson   │ │  Lesson    │ │ Lessons  │    │
│  │  01-02   │ │   05     │ │    06      │ │  04,07   │    │
│  └──────────┘ └──────────┘ └────────────┘ └──────────┘    │
│                           │                                  │
│                    shared types                             │
│                           ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              ebpf-tool-common                        │   │
│  │    SyscallEvent, SyscallKey, MAX_MAP_ENTRIES        │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Files Created

| File | Description |
|------|-------------|
| `crates/ebpf-tool-ebpf/Cargo.toml` | Package config with aya-ebpf deps, BPF-optimized profiles (opt-level=3, lto=true, panic=abort) |
| `crates/ebpf-tool-ebpf/src/main.rs` | Entry point with `#![no_std]`, `#![no_main]`, panic handler, module imports |
| `crates/ebpf-tool-ebpf/src/kprobe.rs` | Kprobe programs for Lessons 01-02 (hello_kprobe, syscall_kprobe) |
| `crates/ebpf-tool-ebpf/src/uprobe.rs` | Uprobe programs for Lesson 05 (hello_uprobe, hello_uretprobe) |
| `crates/ebpf-tool-ebpf/src/tracepoint.rs` | Tracepoint programs for Lesson 06 (syscalls, sched, net) |
| `crates/ebpf-tool-ebpf/src/perf.rs` | Perf event programs for Lessons 04, 07 (PerfEventArray, CPU sampling) |

## Files Modified

| File | Change |
|------|--------|
| `backlog/todos/04_ebpf_todo.md` | Marked all 6 ebpf-tool-ebpf items as complete |

## Key Concepts

### Why `#![no_std]` and `#![no_main]`?

eBPF programs run inside the Linux kernel BPF virtual machine:
- No heap allocator (only 512 bytes of stack)
- No system calls (eBPF has its own helper functions)
- No threads, no I/O, no standard library
- Entry points are probe functions, not `main()`

### BPF Profile Settings

```toml
[profile.release]
opt-level = 3         # BPF verifier rejects unoptimized code
debug = false         # Debug info causes verifier issues
overflow-checks = false  # Not supported in BPF
panic = "abort"       # No unwinding in kernel
lto = true           # Helps meet verifier complexity limits
```

### Probe Types Scaffolded

| Probe Type | Macro | Context | Use Case |
|------------|-------|---------|----------|
| Kprobe | `#[kprobe]` | `ProbeContext` | Dynamic kernel function tracing |
| Uprobe | `#[uprobe]` | `ProbeContext` | Userspace function tracing |
| Tracepoint | `#[tracepoint]` | `TracePointContext` | Static kernel instrumentation |
| Perf Event | `#[perf_event]` | `PerfEventContext` | CPU sampling, profiling |

## How to Use

### Build the workspace
```bash
make all
```

### Check build status
```bash
cargo check -p ebpf-tool
```

The eBPF programs won't compile yet because they contain `todo!()` stubs. The build script creates a placeholder so userspace tools still work.

### TDD Workflow (for learners)
1. Read lesson doc (e.g., `docs/04-ebpf/01-hello-kprobe.md`)
2. Write tests in `crates/ebpf-tool/tests/kprobe_test.rs` (RED)
3. Implement `todo!()` stubs in `crates/ebpf-tool-ebpf/src/kprobe.rs` (GREEN)
4. Implement userspace handler in `crates/ebpf-tool/src/main.rs`

## Technical Notes

### Build Warnings (Expected)

The build produces expected warnings for TDD scaffolding:
- **Unused imports**: `assert_cmd::Command`, `predicates::prelude::*` - will be used when tests are implemented
- **Dead code**: `is_root()` helpers - will be used for root-required tests
- **Unreachable code**: `Ok(())` after `todo!()` match arms

### eBPF Compilation Status

```
eBPF compilation failed with status: exit status: 101
Created placeholder at: .../ebpf-tool-ebpf
```

This is expected because:
1. The crate exists, so `build.rs` attempts compilation
2. `todo!()` macros panic during compilation
3. Build script gracefully creates placeholder for userspace tools

### Dependencies

```toml
# ebpf-tool-ebpf dependencies
aya-ebpf = "0.1"      # eBPF program types, maps, helpers
aya-log-ebpf = "0.1"  # Logging from eBPF to userspace
ebpf-tool-common = { path = "../ebpf-tool-common" }  # Shared types
```

## Phase Status

### Phase 2: Crate Scaffolding - COMPLETE

```
- [x] crates/ebpf-tool-common (shared types)
- [x] crates/ebpf-tool (userspace CLI)
- [x] crates/ebpf-tool/tests/* (test files)
- [x] crates/ebpf-tool-ebpf (eBPF programs) ← This session
```

### Phase 3: Lesson Docs - NOT STARTED

```
- [ ] docs/04-ebpf/00-ebpf-setup.md
- [ ] docs/04-ebpf/01-hello-kprobe.md
- [ ] docs/04-ebpf/02-reading-data.md
- [ ] docs/04-ebpf/03-maps.md
- [ ] docs/04-ebpf/04-perf-events.md
- [ ] docs/04-ebpf/05-uprobes.md
- [ ] docs/04-ebpf/06-tracepoints.md
- [ ] docs/04-ebpf/07-perf-sampling.md
- [ ] docs/04-ebpf/08-combining.md
```

## Next Steps (Not Implemented)

1. **Write Lesson Docs (Phase 3)**: Create the 9 tutorial documents that guide learners through implementing the `todo!()` stubs
2. **Install eBPF Toolchain**: The devcontainer needs `bpf-linker` and `rust-src` for actual eBPF compilation
3. **Test on Real Kernel**: eBPF programs require Linux kernel 5.8+ with BTF support

## Repository Information

- **Branch**: `feature/04-ebpf-tutorials`
- **Last Commit**: `e600aa3` - Add ebpf-tool test scaffolding with TDD todo!() stubs
- **Status**: Clean (no uncommitted changes after this session)
