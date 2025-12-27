# Progress Log: eBPF Tool Userspace CLI Scaffolding

## Header
- **Date**: 2025-12-27
- **Session**: 062544
- **Branch**: `feature/04-ebpf-tutorials`
- **Overview**: Created the `ebpf-tool` userspace CLI crate with TDD scaffolding including Cargo.toml, build.rs, and main.rs with todo!() stubs for all subcommands.

## What We Built

### ebpf-tool Userspace CLI
A complete CLI scaffolding for eBPF tracing tutorials using the Aya framework. The crate follows the TDD pattern where learners write tests first (RED), then implement the todo!() stubs (GREEN).

**Subcommands created:**
| Subcommand | Lesson | Description |
|------------|--------|-------------|
| `check` | 00 | Validate eBPF environment (BTF, kernel, permissions) |
| `kprobe` | 01 | Attach kprobe to kernel function |
| `stats` | 03 | Show eBPF map statistics |
| `uprobe` | 05 | Attach uprobe to userspace function |
| `tracepoint` | 06 | Attach to kernel tracepoint |
| `perf` | 07 | CPU performance sampling |
| `trace` | 08 | Full syscall tracer (combines all concepts) |

## Files Created/Modified

### Created

| File | Description |
|------|-------------|
| `crates/ebpf-tool/Cargo.toml` | Package manifest with Aya dependencies (aya, aya-log, tokio, clap) |
| `crates/ebpf-tool/build.rs` | Build script for compiling eBPF programs from ebpf-tool-ebpf crate |
| `crates/ebpf-tool/src/main.rs` | CLI with 7 subcommands, each with todo!() stubs and TDD guidance |

### Modified

| File | Change |
|------|--------|
| `backlog/todos/04_ebpf_todo.md` | Marked 3 items complete under "crates/ebpf-tool" subsection |

### Existing (Referenced)

| File | Purpose |
|------|---------|
| `crates/ebpf-tool-common/src/lib.rs` | Shared types between userspace and eBPF (already existed) |
| `backlog/plans/04_ebpf_plan.md` | Implementation plan for eBPF tutorials |
| `docs/00-foundations/00-lesson-template.md` | Template for TDD lesson structure |

## Key Concepts Explained

### Architecture

```
crates/
  ebpf-tool/              <- Userspace CLI (this session)
    Cargo.toml            <- Workspace member, Aya dependencies
    build.rs              <- Compiles eBPF programs from sibling crate
    src/main.rs           <- CLI with todo!() stubs
  ebpf-tool-ebpf/         <- eBPF programs (NOT yet created, lesson 01)
    (no_std, BPF target)
  ebpf-tool-common/       <- Shared types (already exists)
    (no_std compatible)
```

### Build Script Flow

```
build.rs executes during `cargo build`:

1. Check if ebpf-tool-ebpf crate exists
   |
   +-- No --> Create placeholder, emit warnings
   |          (allows CLI to compile before lesson 01)
   |
   +-- Yes --> Compile eBPF programs
               |
               +-- cargo +nightly build --target bpfel-unknown-none
               |   -Z build-std=core --release
               |
               +-- Copy compiled binary to OUT_DIR
               |
               +-- Export EBPF_OUT_DIR env var
```

### TDD Pattern in todo!() Stubs

Each subcommand follows this pattern (`main.rs:134-161` example):

```rust
// =========================================================================
// Lesson 00: eBPF Setup
// =========================================================================
// TODO: Implement environment validation
// Lesson: docs/04-ebpf/00-ebpf-setup.md
// Tests: tests/check_test.rs
//
// TDD Steps:
// 1. Write tests in tests/check_test.rs (RED)
// 2. Implement this function to make tests pass (GREEN)
// 3. Refactor as needed
//
// Implementation hints:
// - Check kernel version >= 5.8
// - Verify BTF at /sys/kernel/btf/vmlinux
// - Check CAP_BPF capability
Command::Check => {
    todo!("Implement check subcommand - write tests first!")
}
```

### include_bytes_aligned! Macro

eBPF bytecode requires 8-byte alignment for kernel loading:

```rust
#[macro_export]
macro_rules! include_bytes_aligned {
    ($path:expr) => {{
        #[repr(C, align(8))]
        struct Aligned<T: ?Sized>(T);
        static ALIGNED: &Aligned<[u8]> = &Aligned(*include_bytes!($path));
        &ALIGNED.0
    }};
}
```

## How to Use

### Build the CLI
```bash
cargo build -p ebpf-tool
```

### Run CLI Help
```bash
cargo run -p ebpf-tool -- --help

# Output:
# Commands:
#   check       Validate eBPF environment
#   kprobe      Attach a kprobe to a kernel function
#   stats       Show eBPF map statistics
#   uprobe      Attach a uprobe to a userspace function
#   tracepoint  Attach to a kernel tracepoint
#   perf        CPU performance sampling
#   trace       Full syscall tracer
```

### Run Subcommand (will panic with todo!())
```bash
cargo run -p ebpf-tool -- check
# thread 'main' panicked at 'not yet implemented: Implement check subcommand'
```

### Run Tests (none exist yet)
```bash
cargo test -p ebpf-tool
# running 0 tests
```

## Technical Notes

### Warnings (Expected)
The build script emits warnings when ebpf-tool-ebpf crate doesn't exist:
```
warning: ebpf-tool-ebpf crate not found
warning: eBPF programs will not be compiled until the crate is created
warning: See: docs/04-ebpf/01-hello-kprobe.md for instructions
```

This is by design - learners create the eBPF crate in lesson 01.

### Clippy Fix Applied
Fixed one clippy warning in `build.rs:67`:
```rust
// Before (deprecated pattern):
if path.extension().map_or(false, |ext| ext == "rs")

// After:
if path.extension().is_some_and(|ext| ext == "rs")
```

### Dependencies Added
From workspace (defined in root `Cargo.toml`):
- `aya = "0.13"` - eBPF framework
- `aya-log = "0.2"` - Logging from eBPF programs
- `tokio = { version = "1", features = ["full"] }` - Async runtime
- `clap = { version = "4", features = ["derive"] }` - CLI parsing
- `log = "0.4"` / `env_logger = "0.11"` - Logging

## Next Steps (Not Implemented)

### Immediate (from backlog)
1. **Test files** (`crates/ebpf-tool/tests/`):
   - `check_test.rs` (lesson 00)
   - `kprobe_test.rs` (lessons 01-02)
   - `stats_test.rs` (lesson 03)
   - `perf_test.rs` (lessons 04, 07)
   - `uprobe_test.rs` (lesson 05)
   - `tracepoint_test.rs` (lesson 06)
   - `tracer_test.rs` (lesson 08)

2. **eBPF crate** (`crates/ebpf-tool-ebpf/`):
   - `Cargo.toml`
   - `src/main.rs`
   - `src/kprobe.rs`
   - `src/uprobe.rs`
   - `src/tracepoint.rs`
   - `src/perf.rs`

3. **Lesson docs** (`docs/04-ebpf/`):
   - 9 lesson markdown files (00-08)

### Production Considerations
- eBPF programs require `CAP_BPF` or root privileges
- BTF must be available on the target kernel
- Nightly Rust required for eBPF compilation (`-Z build-std`)

## Repository Information
- **URL**: /workspaces/linux-isolation-learning
- **Branch**: `feature/04-ebpf-tutorials`
- **Last Commit**: `83cdcc2` - Add progress log for eBPF tutorial scaffolding with TDD pattern
