//! # eBPF Programs for ebpf-tool
//!
//! This crate contains eBPF programs that run inside the Linux kernel's BPF
//! virtual machine. These programs are compiled to BPF bytecode and loaded
//! by the userspace CLI (`ebpf-tool`).
//!
//! ## Why `#![no_std]` and `#![no_main]`?
//!
//! eBPF programs run inside the Linux kernel, not in userspace. This means:
//!
//! - **No standard library**: The kernel doesn't provide Rust's `std` library.
//!   We use `#![no_std]` to indicate we only have access to `core` (the
//!   platform-agnostic foundation of Rust's standard library).
//!
//! - **No main function**: eBPF programs don't have a traditional entry point.
//!   Instead, they define probe functions that the kernel calls when specific
//!   events occur. We use `#![no_main]` to suppress the compiler's expectation
//!   of a `main()` function.
//!
//! - **No heap allocation**: The kernel's BPF VM has strict memory constraints.
//!   We cannot use `Box`, `Vec`, `String`, or other heap-allocated types.
//!
//! - **Custom panic handler**: Without `std`, we must define our own panic
//!   behavior. In eBPF, panics simply loop forever (which the verifier will
//!   reject if reachable, enforcing safe code).
//!
//! ## eBPF Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        Linux Kernel                             │
//! │  ┌──────────────────────────────────────────────────────────┐   │
//! │  │                    BPF Virtual Machine                    │   │
//! │  │  ┌─────────┐  ┌─────────┐  ┌────────────┐  ┌──────────┐  │   │
//! │  │  │ kprobe  │  │ uprobe  │  │ tracepoint │  │   perf   │  │   │
//! │  │  │ programs│  │ programs│  │  programs  │  │ programs │  │   │
//! │  │  └────┬────┘  └────┬────┘  └─────┬──────┘  └────┬─────┘  │   │
//! │  │       │            │             │              │        │   │
//! │  │       └────────────┴──────┬──────┴──────────────┘        │   │
//! │  │                           │                              │   │
//! │  │                    ┌──────▼──────┐                       │   │
//! │  │                    │  BPF Maps   │                       │   │
//! │  │                    │ (shared     │                       │   │
//! │  │                    │  storage)   │                       │   │
//! │  │                    └──────┬──────┘                       │   │
//! │  └───────────────────────────┼──────────────────────────────┘   │
//! │                              │                                  │
//! └──────────────────────────────┼──────────────────────────────────┘
//!                                │ perf buffer / ring buffer
//!                                ▼
//! ┌──────────────────────────────────────────────────────────────────┐
//! │                         Userspace                                │
//! │  ┌────────────────────────────────────────────────────────────┐  │
//! │  │                    ebpf-tool CLI                           │  │
//! │  │  - Loads eBPF programs into kernel                         │  │
//! │  │  - Reads events from BPF maps                              │  │
//! │  │  - Displays output to user                                 │  │
//! │  └────────────────────────────────────────────────────────────┘  │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## How the Build System Works
//!
//! This crate is compiled differently than normal Rust code:
//!
//! 1. **Target**: We compile to `bpfel-unknown-none` (little-endian BPF) or
//!    `bpfeb-unknown-none` (big-endian BPF), not the host architecture.
//!
//! 2. **Toolchain**: We use `cargo-xtask` with `aya-build` to invoke the BPF
//!    compiler with the correct flags and target specification.
//!
//! 3. **Output**: The compiler produces `.o` files containing BPF bytecode
//!    (ELF format with BPF sections).
//!
//! 4. **Loading**: The userspace `ebpf-tool` embeds these `.o` files and uses
//!    the `aya` library to load them into the kernel at runtime.
//!
//! 5. **Verification**: Before execution, the kernel's BPF verifier analyzes
//!    the bytecode to ensure it's safe (terminates, doesn't access invalid
//!    memory, etc.).
//!
//! ## Module Overview
//!
//! Each module contains eBPF programs for a specific probe type:
//!
//! - [`kprobe`]: Kernel function probes - attach to kernel function entry/exit
//!   - Lesson: `docs/04-ebpf/01-first-kprobe.md`
//!   - Lesson: `docs/04-ebpf/02-kprobe-args.md`
//!
//! - [`uprobe`]: Userspace function probes - attach to userspace function entry/exit
//!   - Lesson: `docs/04-ebpf/05-uprobe-basics.md`
//!
//! - [`tracepoint`]: Static kernel tracepoints - attach to predefined kernel events
//!   - Lesson: `docs/04-ebpf/06-tracepoints.md`
//!
//! - [`perf`]: Perf event sampling - sample CPU, memory, and other hardware events
//!   - Lesson: `docs/04-ebpf/07-perf-events.md`
//!
//! ## Getting Started
//!
//! To build and run eBPF programs:
//!
//! ```bash
//! # Build the eBPF programs
//! cargo xtask build-ebpf
//!
//! # Build the userspace CLI
//! cargo build -p ebpf-tool
//!
//! # Run with elevated privileges (required for eBPF)
//! sudo ./target/debug/ebpf-tool <command>
//! ```
//!
//! See `docs/04-ebpf/00-environment-setup.md` for complete setup instructions.

#![no_std]
#![no_main]

// =============================================================================
// Probe Modules
// =============================================================================
//
// Each module contains eBPF programs for a specific probe type. The programs
// are annotated with Aya macros that define their type and attachment point.

/// Kernel function probes (kprobes and kretprobes).
///
/// Kprobes allow you to dynamically attach to almost any kernel function and
/// inspect its arguments, return values, and execution context.
///
/// # Lessons
/// - `docs/04-ebpf/01-first-kprobe.md` - Your first kprobe program
/// - `docs/04-ebpf/02-kprobe-args.md` - Accessing function arguments
///
/// # TODO
/// Implement the following probes:
/// - `kprobe_execve`: Trace process execution (sys_execve)
/// - `kretprobe_execve`: Capture execve return values
///
/// See the lesson docs for step-by-step implementation guides.
mod kprobe;

/// Userspace function probes (uprobes and uretprobes).
///
/// Uprobes allow you to attach to functions in userspace binaries and shared
/// libraries. This is useful for tracing application behavior without modifying
/// the source code.
///
/// # Lessons
/// - `docs/04-ebpf/05-uprobe-basics.md` - Tracing userspace functions
///
/// # TODO
/// Implement the following probes:
/// - `uprobe_malloc`: Trace memory allocation calls
/// - `uretprobe_malloc`: Capture allocation sizes and return addresses
///
/// See the lesson docs for step-by-step implementation guides.
mod uprobe;

/// Static kernel tracepoints.
///
/// Tracepoints are predefined instrumentation points in the kernel that provide
/// a stable ABI. They're more reliable than kprobes because they don't change
/// across kernel versions (usually).
///
/// # Lessons
/// - `docs/04-ebpf/06-tracepoints.md` - Using kernel tracepoints
///
/// # TODO
/// Implement the following probes:
/// - `tracepoint_sched_process_exec`: Trace process execution via scheduler
/// - `tracepoint_syscalls_enter`: Trace system call entry
///
/// See the lesson docs for step-by-step implementation guides.
mod tracepoint;

/// Perf event sampling.
///
/// Perf events allow you to sample hardware performance counters (CPU cycles,
/// cache misses, etc.) and software events (page faults, context switches).
///
/// # Lessons
/// - `docs/04-ebpf/07-perf-events.md` - Hardware and software event sampling
///
/// # TODO
/// Implement the following probes:
/// - `perf_cpu_cycles`: Sample CPU cycles for profiling
/// - `perf_cache_misses`: Monitor cache performance
///
/// See the lesson docs for step-by-step implementation guides.
mod perf;

// =============================================================================
// Required no_std Infrastructure
// =============================================================================

/// Panic handler for the eBPF environment.
///
/// Since we're running in `#![no_std]` mode, we must provide our own panic
/// handler. In the eBPF context, panics shouldn't occur because:
///
/// 1. The BPF verifier rejects programs that could panic
/// 2. We use checked arithmetic and bounds checking
/// 3. All error conditions are handled explicitly
///
/// If a panic somehow occurs, we enter an infinite loop. The BPF verifier will
/// reject any program where this loop is reachable, ensuring our code is safe.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
