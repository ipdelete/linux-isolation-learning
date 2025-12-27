//! eBPF Kprobe Programs for Kernel Function Tracing
//!
//! This module contains eBPF programs that use kprobes (kernel probes) to dynamically
//! trace kernel function invocations. Kprobes are a powerful debugging mechanism that
//! allow you to install handlers that execute when specific kernel functions are called.
//!
//! # What are Kprobes?
//!
//! Kprobes (Kernel Probes) are a Linux kernel mechanism for dynamic tracing:
//!
//! - **Dynamic**: Probes can be inserted at runtime without recompiling the kernel
//! - **Non-intrusive**: When not active, there is zero overhead
//! - **Flexible**: Can probe almost any kernel function (with some exceptions)
//!
//! # How Kprobes Work
//!
//! 1. **Breakpoint Insertion**: When you attach a kprobe to a kernel function, the
//!    kernel replaces the first instruction with a breakpoint (INT3 on x86).
//!
//! 2. **Handler Execution**: When the CPU hits the breakpoint:
//!    - The kernel saves register state
//!    - Your eBPF handler executes
//!    - The original instruction is executed
//!    - Normal execution resumes
//!
//! 3. **Context Access**: Your handler receives a `ProbeContext` that provides:
//!    - Access to function arguments
//!    - Register values (via `pt_regs`)
//!    - Ability to read kernel/user memory
//!
//! # Common Use Cases
//!
//! - **System Call Tracing**: Monitor which processes call which syscalls
//! - **File System Monitoring**: Track file opens, reads, writes
//! - **Network Debugging**: Trace packet processing in the network stack
//! - **Performance Analysis**: Measure function call latencies
//! - **Security Monitoring**: Detect suspicious kernel activity
//!
//! # Lessons in This Module
//!
//! - **Lesson 01**: Hello Kprobe - Basic kprobe that logs when triggered
//! - **Lesson 02**: Reading Kernel Data - Extract syscall arguments and process info
//!
//! # References
//!
//! - [Aya Book: Kprobes](https://aya-rs.dev/book/programs/kprobes/)
//! - [Linux Kprobes Documentation](https://www.kernel.org/doc/html/latest/trace/kprobes.html)
//! - Lesson Docs: `docs/04-ebpf/01-hello-kprobe.md`, `docs/04-ebpf/02-reading-data.md`
//!
//! # Safety
//!
//! Kprobe handlers run in kernel context with significant power. The BPF verifier
//! ensures memory safety, but you must still:
//!
//! - Validate all pointer accesses
//! - Handle errors gracefully (return 0 on success)
//! - Keep handlers short to minimize latency impact
//! - Be aware that you're running with interrupts disabled

// =============================================================================
// Required Imports
// =============================================================================
//
// TODO: These imports are used in Lessons 01-02
// Uncomment as you progress through the lessons

use aya_ebpf::{
    macros::kprobe,
    programs::ProbeContext,
    // TODO (Lesson 02): Add these imports for reading kernel data
    // helpers::{bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_ktime_get_ns},
};

// TODO (Lesson 01): Uncomment for logging support
// use aya_log_ebpf::info;

// TODO (Lesson 02): Uncomment for sending events to userspace
// use aya_ebpf::{
//     macros::map,
//     maps::PerfEventArray,
// };
// use ebpf_tool_common::SyscallEvent;

// =============================================================================
// eBPF Maps (Lesson 02+)
// =============================================================================
//
// Maps are shared data structures between eBPF and userspace.
// Uncomment when implementing Lesson 02.

// TODO (Lesson 02): Add perf event array for sending events to userspace
//
// #[map]
// static EVENTS: PerfEventArray<SyscallEvent> = PerfEventArray::new(0);

// =============================================================================
// Lesson 01: Hello Kprobe - Basic Kernel Function Tracing
// =============================================================================

/// Basic kprobe that logs when a kernel function is called.
///
/// This is your first eBPF kprobe program. It demonstrates the minimal
/// structure needed to attach to a kernel function and execute code when
/// that function is called.
///
/// # Lesson 01: Hello Kprobe
///
/// **Goal**: Understand kprobe basics by creating a program that logs
/// when a kernel function is invoked.
///
/// ## TDD Workflow
///
/// 1. **Write tests** in `crates/ebpf-tool/tests/kprobe_test.rs` (RED)
/// 2. **Implement this function** (GREEN)
/// 3. **Verify** with `sudo -E cargo test -p ebpf-tool`
///
/// ## Implementation Hints
///
/// - Use the `info!` macro from `aya_log_ebpf` to log messages
/// - Messages are sent to userspace via a perf buffer
/// - Return `0` for success, non-zero for failure
/// - The kernel function name is specified when attaching from userspace
///
/// ## Example Implementation
///
/// ```ignore
/// // Uncomment aya_log_ebpf::info import at top of file first!
/// match try_hello_kprobe(ctx) {
///     Ok(ret) => ret,
///     Err(ret) => ret as u32,
/// }
/// ```
///
/// See helper function `try_hello_kprobe` below for the actual logic.
///
/// ## What Happens When This Runs
///
/// 1. Userspace attaches this probe to a kernel function (e.g., `do_sys_openat2`)
/// 2. Every time that function is called, this handler executes
/// 3. The `info!` log message is sent to userspace via perf buffer
/// 4. Userspace reads and displays the messages
///
/// # Errors
///
/// Returns non-zero if logging fails, but this is rare in practice.
#[kprobe]
pub fn hello_kprobe(ctx: ProbeContext) -> u32 {
    // TODO: Implement in Lesson 01
    // Lesson: docs/04-ebpf/01-hello-kprobe.md
    // Tests: crates/ebpf-tool/tests/kprobe_test.rs
    //
    // Implementation steps:
    // 1. Uncomment the aya_log_ebpf::info import at the top
    // 2. Call try_hello_kprobe(ctx) and handle the Result
    // 3. Return 0 on success, error code on failure
    //
    // Starter code:
    //   match try_hello_kprobe(ctx) {
    //       Ok(ret) => ret,
    //       Err(ret) => ret as u32,
    //   }

    // Suppress unused variable warning until implementation
    let _ = ctx;

    todo!("Implement hello_kprobe - see docs/04-ebpf/01-hello-kprobe.md")
}

/// Helper function for hello_kprobe with proper error handling.
///
/// Separating the logic into a helper that returns `Result` makes error
/// handling cleaner and is a common pattern in Aya programs.
///
/// # Lesson 01 Implementation
///
/// ```ignore
/// fn try_hello_kprobe(ctx: ProbeContext) -> Result<u32, i64> {
///     // Log that the kprobe was triggered
///     info!(&ctx, "kprobe triggered!");
///
///     // Return success
///     Ok(0)
/// }
/// ```
#[allow(dead_code)]
fn try_hello_kprobe(_ctx: ProbeContext) -> Result<u32, i64> {
    // TODO: Implement in Lesson 01
    // Lesson: docs/04-ebpf/01-hello-kprobe.md
    //
    // Hints:
    // - Use info!(&ctx, "kprobe triggered!") to log
    // - Return Ok(0) for success
    //
    // Example:
    //   info!(&ctx, "kprobe triggered!");
    //   Ok(0)

    todo!("Implement try_hello_kprobe - log a message and return Ok(0)")
}

// =============================================================================
// Lesson 02: Reading Kernel Data from Kprobe Context
// =============================================================================

/// Kprobe that reads syscall arguments and sends events to userspace.
///
/// This extends the basic kprobe to extract meaningful data from the kernel:
/// - Process ID and thread ID of the calling process
/// - Process name (comm)
/// - System call number or function arguments
/// - Timestamp of the event
///
/// # Lesson 02: Reading Kernel Data
///
/// **Goal**: Learn to extract data from kernel context and send structured
/// events to userspace.
///
/// ## TDD Workflow
///
/// 1. **Write tests** in `crates/ebpf-tool/tests/kprobe_test.rs`:
///    - Enable `test_kprobe_reads_process_info` (remove `#[ignore]`)
///    - Enable `test_kprobe_reads_function_arguments` (remove `#[ignore]`)
/// 2. **Implement this function** (GREEN)
/// 3. **Verify** with `sudo -E cargo test -p ebpf-tool`
///
/// ## Key BPF Helpers
///
/// - `bpf_get_current_pid_tgid()`: Returns (PID << 32 | TID)
/// - `bpf_get_current_comm()`: Gets process command name (up to 16 chars)
/// - `bpf_ktime_get_ns()`: High-resolution timestamp
/// - `ctx.arg::<T>(n)`: Read the nth function argument
///
/// ## Implementation Hints
///
/// ```ignore
/// // Get PID and TID from combined value
/// let pid_tgid = bpf_get_current_pid_tgid();
/// let pid = (pid_tgid >> 32) as u32;  // Process ID
/// let tid = pid_tgid as u32;          // Thread ID
///
/// // Get process name
/// let mut comm = [0u8; 16];
/// let _ = bpf_get_current_comm(&mut comm);
///
/// // Create and send event
/// let event = SyscallEvent {
///     pid,
///     tid,
///     syscall_nr: 0,  // Populated if probing syscall entry
///     timestamp_ns: bpf_ktime_get_ns(),
///     comm,
/// };
/// EVENTS.output(&ctx, &event, 0);
/// ```
///
/// ## Data Layout Considerations
///
/// - `SyscallEvent` is defined in `ebpf-tool-common`
/// - Must be `#[repr(C)]` for correct memory layout
/// - Userspace must read with matching struct definition
#[kprobe]
pub fn syscall_kprobe(ctx: ProbeContext) -> u32 {
    // TODO: Implement in Lesson 02
    // Lesson: docs/04-ebpf/02-reading-data.md
    // Tests: crates/ebpf-tool/tests/kprobe_test.rs
    //
    // Implementation steps:
    // 1. Uncomment the helper imports at the top of this file
    // 2. Uncomment the EVENTS map definition above
    // 3. Call try_syscall_kprobe(ctx) and handle the Result
    // 4. Return 0 on success, error code on failure
    //
    // Starter code:
    //   match try_syscall_kprobe(ctx) {
    //       Ok(ret) => ret,
    //       Err(_) => 0,  // Silently ignore errors in kprobe
    //   }

    // Suppress unused variable warning until implementation
    let _ = ctx;

    todo!("Implement syscall_kprobe - see docs/04-ebpf/02-reading-data.md")
}

/// Helper function for syscall_kprobe with proper error handling.
///
/// # Lesson 02 Implementation
///
/// This function should:
/// 1. Get PID/TID using `bpf_get_current_pid_tgid()`
/// 2. Get process name using `bpf_get_current_comm()`
/// 3. Get timestamp using `bpf_ktime_get_ns()`
/// 4. Optionally read syscall arguments from context
/// 5. Create a `SyscallEvent` and send via `EVENTS` perf array
#[allow(dead_code)]
fn try_syscall_kprobe(_ctx: ProbeContext) -> Result<u32, i64> {
    // TODO: Implement in Lesson 02
    // Lesson: docs/04-ebpf/02-reading-data.md
    //
    // Implementation outline:
    //
    // 1. Get process info:
    //    let pid_tgid = unsafe { bpf_get_current_pid_tgid() };
    //    let pid = (pid_tgid >> 32) as u32;
    //    let tid = pid_tgid as u32;
    //
    // 2. Get process name:
    //    let mut comm = [0u8; 16];
    //    unsafe { bpf_get_current_comm(&mut comm) }
    //        .map_err(|e| e as i64)?;
    //
    // 3. Get timestamp:
    //    let timestamp_ns = unsafe { bpf_ktime_get_ns() };
    //
    // 4. Read syscall number (optional, depends on probe target):
    //    let syscall_nr = try_read_syscall_args(&ctx)?;
    //
    // 5. Build and send event:
    //    let event = SyscallEvent {
    //        pid,
    //        tid,
    //        syscall_nr,
    //        timestamp_ns,
    //        comm,
    //    };
    //    EVENTS.output(&ctx, &event, 0);
    //
    // 6. Return success:
    //    Ok(0)

    todo!("Implement try_syscall_kprobe - read kernel data and send event")
}

// =============================================================================
// Helper Functions for Reading Kernel Data
// =============================================================================

/// Helper to safely read syscall arguments from kprobe context.
///
/// When probing syscall entry points, the first argument is typically
/// the syscall number (on x86_64, in the `orig_rax` register).
///
/// # Safety
///
/// This function accesses kernel memory through the probe context.
/// The BPF verifier ensures safety, but we wrap in `unsafe` to be explicit.
///
/// # Arguments
///
/// * `ctx` - The probe context containing register state
///
/// # Returns
///
/// * `Ok(syscall_nr)` - The system call number
/// * `Err(errno)` - Error code if reading fails
///
/// # Example
///
/// ```ignore
/// let syscall_nr = unsafe { try_read_syscall_args(&ctx)? };
/// ```
#[allow(dead_code)]
unsafe fn try_read_syscall_args(_ctx: &ProbeContext) -> Result<u64, i64> {
    // TODO: Implement in Lesson 02
    // Lesson: docs/04-ebpf/02-reading-data.md
    //
    // Hints:
    // - Use ctx.arg::<u64>(0) to read first argument
    // - Different kernel functions have different argument layouts
    // - For syscall entry points, argument 0 is often the syscall number
    //
    // Example:
    //   let arg0: u64 = ctx.arg(0).ok_or(-1i64)?;
    //   Ok(arg0)
    //
    // Note: The exact method depends on which kernel function you're probing.
    // When probing sys_enter, you may need to access pt_regs differently.

    todo!("Read syscall arguments from ProbeContext")
}

/// Helper to get the current CPU ID.
///
/// Useful for per-CPU maps and understanding scheduling behavior.
///
/// # Lesson 02+ Implementation
///
/// ```ignore
/// use aya_ebpf::helpers::bpf_get_smp_processor_id;
///
/// fn get_cpu_id() -> u32 {
///     unsafe { bpf_get_smp_processor_id() }
/// }
/// ```
#[allow(dead_code)]
fn get_cpu_id() -> u32 {
    // TODO: Implement when needed
    // Hints:
    // - Use bpf_get_smp_processor_id() helper
    // - Returns the current CPU number (0-indexed)

    todo!("Get current CPU ID using bpf_get_smp_processor_id")
}

// =============================================================================
// Note: Panic handler is defined in main.rs (crate root)
// =============================================================================
