//! eBPF Tool - CLI for eBPF tracing tutorials
//!
//! This tool provides subcommands for learning eBPF concepts using the Aya framework.
//! Each subcommand corresponds to a lesson in the docs/04-ebpf/ directory.
//!
//! # Architecture
//!
//! ```text
//! crates/
//!   ebpf-tool/           <- You are here (userspace CLI)
//!   ebpf-tool-ebpf/      <- eBPF programs (no_std, BPF target)
//!   ebpf-tool-common/    <- Shared types between userspace and eBPF
//! ```
//!
//! # TDD Workflow
//!
//! 1. Read the lesson doc in docs/04-ebpf/
//! 2. Write tests in tests/*.rs (RED - tests fail)
//! 3. Implement the todo!() stub below (GREEN - tests pass)
//! 4. Refactor as needed

use anyhow::Result;
use clap::{Parser, Subcommand};

// Macro for including compiled eBPF bytecode with proper alignment.
// The eBPF loader requires 8-byte alignment for the bytecode.
#[macro_export]
macro_rules! include_bytes_aligned {
    ($path:expr) => {{
        // Use a static to ensure proper alignment
        #[repr(C, align(8))]
        struct Aligned<T: ?Sized>(T);
        static ALIGNED: &Aligned<[u8]> = &Aligned(*include_bytes!($path));
        &ALIGNED.0
    }};
}

#[derive(Parser)]
#[command(name = "ebpf-tool")]
#[command(about = "eBPF tracing tool for Linux isolation learning")]
#[command(version)]
struct Cli {
    /// Enable verbose logging (set RUST_LOG=debug for maximum detail)
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate eBPF environment (BTF, kernel version, permissions)
    Check,

    /// Attach a kprobe to a kernel function
    Kprobe {
        /// Kernel function name to probe (e.g., "do_sys_openat2")
        function: String,

        /// Duration in seconds to run (0 = until Ctrl+C)
        #[arg(short, long, default_value = "5")]
        duration: u64,
    },

    /// Show eBPF map statistics (HashMap counters)
    Stats,

    /// Attach a uprobe to a userspace function
    Uprobe {
        /// Path to the binary (e.g., "/usr/bin/bash")
        binary: String,

        /// Function name to probe (e.g., "readline")
        function: String,

        /// Duration in seconds to run (0 = until Ctrl+C)
        #[arg(short, long, default_value = "5")]
        duration: u64,
    },

    /// Attach to a kernel tracepoint
    Tracepoint {
        /// Tracepoint category (e.g., "syscalls")
        category: String,

        /// Tracepoint name (e.g., "sys_enter_openat")
        name: String,

        /// Duration in seconds to run (0 = until Ctrl+C)
        #[arg(short, long, default_value = "5")]
        duration: u64,
    },

    /// CPU performance sampling via perf events
    Perf {
        /// Sample frequency in Hz
        #[arg(short, long, default_value = "99")]
        frequency: u64,

        /// Duration in seconds to run (0 = until Ctrl+C)
        #[arg(short, long, default_value = "5")]
        duration: u64,
    },

    /// Full syscall tracer (combines kprobes, maps, and perf events)
    Trace {
        /// Filter by process name (optional)
        #[arg(short, long)]
        process: Option<String>,

        /// Filter by syscall name (optional)
        #[arg(short, long)]
        syscall: Option<String>,

        /// Duration in seconds to run (0 = until Ctrl+C)
        #[arg(short, long, default_value = "10")]
        duration: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity flag
    // Users can also set RUST_LOG=debug for more control
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match cli.command {
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
        // - Check kernel version >= 5.8 for good eBPF support
        // - Verify BTF is available at /sys/kernel/btf/vmlinux
        // - Check CAP_BPF or CAP_SYS_ADMIN capability
        // - Verify bpf() syscall is accessible
        // - Print diagnostic information about the environment
        //
        // Expected output format:
        //   Kernel version: 5.15.0 [OK]
        //   BTF available: /sys/kernel/btf/vmlinux [OK]
        //   Permissions: CAP_BPF [OK]
        //   eBPF syscall: accessible [OK]
        Command::Check => {
            todo!("Implement check subcommand - write tests first!")
        }

        // =========================================================================
        // Lesson 01: Hello Kprobe
        // =========================================================================
        // TODO: Implement kprobe attachment
        // Lesson: docs/04-ebpf/01-hello-kprobe.md
        // Tests: tests/kprobe_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/kprobe_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Load eBPF bytecode using include_bytes_aligned!
        // - Use aya::Bpf::load() to parse the eBPF object
        // - Get the kprobe program: bpf.program_mut("kprobe_fn")
        // - Attach to the specified function: kprobe.attach(&function, 0)
        // - Use aya_log to receive log messages from eBPF program
        // - Run for specified duration or until Ctrl+C
        //
        // eBPF program location: crates/ebpf-tool-ebpf/src/kprobe.rs
        Command::Kprobe { function, duration } => {
            log::info!("Attaching kprobe to function: {}", function);
            log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);
            todo!("Implement kprobe subcommand - write tests first!")
        }

        // =========================================================================
        // Lesson 03: eBPF Maps
        // =========================================================================
        // TODO: Implement map statistics display
        // Lesson: docs/04-ebpf/03-maps.md
        // Tests: tests/stats_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/stats_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Load the eBPF program that populates the HashMap
        // - Get the map: bpf.map("SYSCALL_COUNTS")
        // - Iterate over HashMap entries: map.iter()
        // - Display syscall names and their counts
        // - Consider using a table format for output
        //
        // Expected output format:
        //   Syscall Statistics:
        //   ------------------
        //   openat:    1234
        //   read:      5678
        //   write:     9012
        Command::Stats => {
            todo!("Implement stats subcommand - write tests first!")
        }

        // =========================================================================
        // Lesson 05: Uprobes
        // =========================================================================
        // TODO: Implement uprobe attachment
        // Lesson: docs/04-ebpf/05-uprobes.md
        // Tests: tests/uprobe_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/uprobe_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Load eBPF bytecode for uprobe program
        // - Get the uprobe program: bpf.program_mut("uprobe_fn")
        // - Attach to userspace function: uprobe.attach(Some(&function), 0, &binary, None)
        // - The binary path must be absolute or resolvable
        // - Use aya_log to receive events from the eBPF program
        //
        // eBPF program location: crates/ebpf-tool-ebpf/src/uprobe.rs
        Command::Uprobe {
            binary,
            function,
            duration,
        } => {
            log::info!("Attaching uprobe to {}:{}", binary, function);
            log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);
            todo!("Implement uprobe subcommand - write tests first!")
        }

        // =========================================================================
        // Lesson 06: Tracepoints
        // =========================================================================
        // TODO: Implement tracepoint attachment
        // Lesson: docs/04-ebpf/06-tracepoints.md
        // Tests: tests/tracepoint_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/tracepoint_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Load eBPF bytecode for tracepoint program
        // - Get the tracepoint program: bpf.program_mut("tracepoint_fn")
        // - Attach: tracepoint.attach(&category, &name)
        // - Common tracepoints:
        //   - syscalls/sys_enter_openat
        //   - sched/sched_switch
        //   - net/netif_rx
        // - List available: ls /sys/kernel/debug/tracing/events/
        //
        // eBPF program location: crates/ebpf-tool-ebpf/src/tracepoint.rs
        Command::Tracepoint {
            category,
            name,
            duration,
        } => {
            log::info!("Attaching to tracepoint: {}/{}", category, name);
            log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);
            todo!("Implement tracepoint subcommand - write tests first!")
        }

        // =========================================================================
        // Lesson 07: Perf Events
        // =========================================================================
        // TODO: Implement CPU performance sampling
        // Lesson: docs/04-ebpf/07-perf-sampling.md
        // Tests: tests/perf_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/perf_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Load eBPF bytecode for perf event program
        // - Get the perf event program: bpf.program_mut("perf_event_fn")
        // - Create perf event for each CPU: perf_event_open()
        // - Attach: perf_event.attach(perf_fd)
        // - Sample stack traces and aggregate
        // - Display flame graph-style output or top functions
        //
        // eBPF program location: crates/ebpf-tool-ebpf/src/perf.rs
        Command::Perf {
            frequency,
            duration,
        } => {
            log::info!("Starting CPU sampling at {} Hz", frequency);
            log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);
            todo!("Implement perf subcommand - write tests first!")
        }

        // =========================================================================
        // Lesson 08: Combining Everything
        // =========================================================================
        // TODO: Implement full syscall tracer
        // Lesson: docs/04-ebpf/08-combining.md
        // Tests: tests/tracer_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/tracer_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Combines concepts from all previous lessons
        // - Use kprobes/tracepoints to capture syscall entry/exit
        // - Use HashMaps for per-syscall and per-process statistics
        // - Use PerfEventArray for real-time event streaming
        // - Apply optional filters (process name, syscall name)
        // - Display live output with timestamps
        //
        // Expected output format:
        //   [12:34:56.789] bash(1234) openat("/etc/passwd", O_RDONLY) = 3
        //   [12:34:56.790] bash(1234) read(3, ..., 4096) = 1024
        //   [12:34:56.791] bash(1234) close(3) = 0
        Command::Trace {
            process,
            syscall,
            duration,
        } => {
            log::info!("Starting syscall tracer");
            if let Some(ref p) = process {
                log::info!("Filtering by process: {}", p);
            }
            if let Some(ref s) = syscall {
                log::info!("Filtering by syscall: {}", s);
            }
            log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);
            todo!("Implement trace subcommand - write tests first!")
        }
    }
}

// =============================================================================
// Helper functions (implement as needed during lessons)
// =============================================================================

/// Check if the current process has CAP_BPF or CAP_SYS_ADMIN capability.
///
/// This is needed for loading eBPF programs. On modern kernels (5.8+),
/// CAP_BPF is sufficient. On older kernels, CAP_SYS_ADMIN is required.
#[allow(dead_code)]
fn check_bpf_capability() -> bool {
    // TODO: Implement capability check in lesson 00
    // Hint: Use nix::unistd::Uid::effective().is_root() for simple check
    // Or use caps crate for fine-grained capability check
    todo!("Implement capability check")
}

/// Check if BTF (BPF Type Format) is available on the system.
///
/// BTF enables CO-RE (Compile Once, Run Everywhere) which allows
/// eBPF programs to run on different kernel versions without recompilation.
#[allow(dead_code)]
fn check_btf_available() -> bool {
    // TODO: Implement BTF check in lesson 00
    // Hint: Check if /sys/kernel/btf/vmlinux exists
    todo!("Implement BTF availability check")
}

/// Get the kernel version as a tuple (major, minor, patch).
#[allow(dead_code)]
fn get_kernel_version() -> Result<(u32, u32, u32)> {
    // TODO: Implement kernel version parsing in lesson 00
    // Hint: Use nix::sys::utsname::uname() or read /proc/version
    todo!("Implement kernel version check")
}
