# Progress Log: eBPF Tutorial Scaffolding with TDD Pattern

**Date**: 2025-12-27
**Session**: 061411
**Branch**: `feature/04-ebpf-tutorials`

## Overview

Established the TDD (Test-Driven Development) pattern for eBPF tutorials and created the initial `ebpf-tool-common` crate scaffolding. Updated project documentation to guide future tutorial creation.

## What We Built

### 1. ebpf-tool-common Crate
Shared types crate for eBPF programs (no_std compatible).

**Purpose**: Define event structures shared between userspace (`ebpf-tool`) and eBPF programs (`ebpf-tool-ebpf`). These types must match exactly on both sides for perf buffer communication.

**Key types**:
- `SyscallEvent` - Captures syscall invocations (pid, tid, syscall_nr, timestamp, comm)
- `SyscallKey` - HashMap key for syscall counting

**TDD elements**:
- Tests have `todo!()` stubs for learners to implement
- TODO comments for types added in later lessons (FunctionEvent, TracepointEvent, PerfSampleEvent)

### 2. CLAUDE.md Tutorial Creation Guide
Concise guide (120 lines) documenting how to create tutorials following the TDD pattern.

**Sections**:
- Structure overview
- 3-step process: Scaffold → Write docs → Update backlog
- Code examples for `todo!()` stubs
- Conventions and verification checklist

### 3. Updated Plan & Todo
Restructured `04_ebpf_plan.md` and `04_ebpf_todo.md` to reflect TDD workflow.

## Files Created/Modified

### Created
| File | Description |
|------|-------------|
| `CLAUDE.md` | Tutorial creation guide with TDD pattern |
| `crates/ebpf-tool-common/Cargo.toml` | no_std crate config with `user` feature |
| `crates/ebpf-tool-common/src/lib.rs` | Shared types + `todo!()` test stubs |
| `crates/ebpf-tool/Cargo.toml` | Placeholder CLI crate |
| `crates/ebpf-tool/src/main.rs` | Minimal stub (to be expanded) |

### Modified
| File | Changes |
|------|---------|
| `backlog/plans/04_ebpf_plan.md` | Added TDD sections, lesson→test mapping table |
| `backlog/todos/04_ebpf_todo.md` | Restructured into 3 phases |
| `Cargo.lock` | Added ebpf-tool-common |
| `Makefile` | Minor update |

## Key Concepts: TDD Pattern for Tutorials

```
┌─────────────────────────────────────────────────────────────┐
│                    TDD Tutorial Flow                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Phase 2: Scaffolding          Phase 3: Lesson Docs         │
│  ┌─────────────────┐           ┌─────────────────┐         │
│  │ main.rs         │           │ XX-lesson.md    │         │
│  │ ┌─────────────┐ │           │                 │         │
│  │ │ Command::X  │ │◄──────────│ "Write Tests"   │         │
│  │ │ todo!()     │ │           │ section guides  │         │
│  │ └─────────────┘ │           │ learner to      │         │
│  └─────────────────┘           │ implement       │         │
│                                │                 │         │
│  ┌─────────────────┐           │ "Build" section │         │
│  │ tests/x_test.rs │◄──────────│ guides learner  │         │
│  │ ┌─────────────┐ │           │ to implement    │         │
│  │ │ test_x()    │ │           │                 │         │
│  │ │ todo!()     │ │           └─────────────────┘         │
│  │ └─────────────┘ │                                       │
│  └─────────────────┘                                       │
│                                                             │
│  Learner workflow: RED (tests fail) → GREEN (implement)    │
└─────────────────────────────────────────────────────────────┘
```

### Pattern Comparison

**ns-tool (existing pattern)**:
```rust
// crates/ns-tool/src/main.rs:47
Command::Pid => todo!("Implement PID namespace - write tests first!"),

// crates/ns-tool/tests/pid_test.rs
#[test]
fn test_pid_namespace_creation() {
    todo!("Implement test for PID namespace creation")
}
```

**ebpf-tool-common (new, follows pattern)**:
```rust
// crates/ebpf-tool-common/src/lib.rs:138
#[test]
fn test_syscall_event_size_and_alignment() {
    todo!("Verify SyscallEvent size is between 40-48 bytes")
}
```

## How to Use

### Verify Current State
```bash
# ebpf-tool-common compiles
cargo build -p ebpf-tool-common

# Tests fail with todo!() (RED phase - expected)
cargo test -p ebpf-tool-common
# Output: 3 failed, 3 ignored
```

### Check Todo Progress
```bash
cat backlog/todos/04_ebpf_todo.md
```

## Technical Notes

### no_std Compatibility
`ebpf-tool-common` uses `#![no_std]` because eBPF programs run in kernel space without standard library access. All types use:
- `#[repr(C)]` for consistent memory layout
- Fixed-size types (`u32`, `u64`, `[u8; N]`)
- `Copy` trait for perf buffer passing

### Test Stub Pattern
Tests use `todo!()` with hints so learners know what to implement:
```rust
#[test]
fn test_syscall_event_size_and_alignment() {
    // TODO: Verify SyscallEvent has correct size for C interop
    //
    // Hints:
    // - Use core::mem::size_of::<SyscallEvent>()
    // - Expected: 4 + 4 + 8 + 8 + 16 = 40 bytes (may have padding)

    todo!("Verify SyscallEvent size is between 40-48 bytes")
}
```

## Next Steps (Not Implemented)

Per `backlog/todos/04_ebpf_todo.md`:

### Phase 2 Remaining (Scaffolding)
- [ ] `crates/ebpf-tool/src/main.rs` - Full Command enum with `todo!()` stubs
- [ ] `crates/ebpf-tool/build.rs` - eBPF compilation script
- [ ] `crates/ebpf-tool/tests/*.rs` - 7 test files with `todo!()` stubs
- [ ] `crates/ebpf-tool-ebpf/` - eBPF programs crate (6 files)

### Phase 3 (Lesson Docs)
- [ ] 9 lesson documents in `docs/04-ebpf/`

## Repository Information

**URL**: https://github.com/ipdelete/linux-isolation-learning
**Branch**: `feature/04-ebpf-tutorials`
**Commit**: `6e670aa` - Add ebpf-tool-common scaffolding with TDD pattern

### Files in Commit
```
 CLAUDE.md                              | 120 +++
 Cargo.lock                             |   8 +
 Makefile                               |   2 +-
 backlog/plans/04_ebpf_plan.md          | 142 ++-
 backlog/todos/04_ebpf_todo.md          |  61 +-
 crates/ebpf-tool-common/Cargo.toml     |  19 +
 crates/ebpf-tool-common/src/lib.rs     | 194 ++++
 crates/ebpf-tool/Cargo.toml            |  14 +
 crates/ebpf-tool/src/main.rs           |   5 +
```
