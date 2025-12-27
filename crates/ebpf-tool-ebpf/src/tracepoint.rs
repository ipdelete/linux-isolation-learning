// =============================================================================
// eBPF Tracepoint Programs - Lesson 06
// =============================================================================
//
// This module provides eBPF tracepoint programs for monitoring kernel events
// at static instrumentation points.
//
// # What are Tracepoints?
//
// Tracepoints are static instrumentation points placed in the Linux kernel
// source code by kernel developers. Unlike kprobes (which can attach to any
// kernel function), tracepoints are:
//
// - **Stable**: They provide a stable ABI across kernel versions. The kernel
//   developers commit to maintaining the tracepoint interface.
//
// - **Intentional**: They are placed at semantically meaningful points in
//   the kernel, providing well-documented context.
//
// - **Low overhead**: When not in use, tracepoints have minimal overhead
//   (just a branch that's predicted not-taken).
//
// - **Rich context**: Tracepoint arguments are clearly documented and
//   structured, making it easier to access relevant data.
//
// # Tracepoints vs Kprobes
//
// | Feature          | Tracepoints           | Kprobes                |
// |------------------|-----------------------|------------------------|
// | Stability        | Stable ABI            | May break on upgrade   |
// | Attachment       | Named tracepoint      | Any kernel function    |
// | Arguments        | Well-defined format   | Manual extraction      |
// | Discovery        | /sys/kernel/debug/... | kallsyms               |
// | Overhead (idle)  | ~1 nop instruction    | None                   |
// | Flexibility      | Limited to existing   | Attach anywhere        |
//
// # Common Tracepoint Categories
//
// - **syscalls**: System call entry/exit (sys_enter_*, sys_exit_*)
// - **sched**: Scheduler events (sched_switch, sched_process_fork)
// - **net**: Networking events (netif_rx, net_dev_xmit)
// - **block**: Block I/O events (block_rq_issue, block_rq_complete)
// - **irq**: Interrupt handling (irq_handler_entry, softirq_entry)
// - **signal**: Signal delivery (signal_generate, signal_deliver)
// - **raw_syscalls**: Raw syscall entry/exit (fewer args, all syscalls)
//
// # Listing Available Tracepoints
//
// ```bash
// # List all tracepoint categories
// ls /sys/kernel/debug/tracing/events/
//
// # List tracepoints in a category
// ls /sys/kernel/debug/tracing/events/syscalls/
//
// # View tracepoint format (argument layout)
// cat /sys/kernel/debug/tracing/events/syscalls/sys_enter_openat/format
// ```
//
// # Reference
//
// - Lesson: docs/04-ebpf/06-tracepoints.md
// - Tests: crates/ebpf-tool/tests/tracepoint_test.rs
// - Kernel docs: Documentation/trace/tracepoints.rst
//
// =============================================================================

use aya_ebpf::{macros::tracepoint, programs::TracePointContext};
use aya_log_ebpf::info;

// =============================================================================
// Syscall Tracepoints
// =============================================================================

/// Tracepoint for syscall entry events.
///
/// # Lesson 06: Tracepoints
///
/// TDD Steps:
/// 1. Write tests in crates/ebpf-tool/tests/tracepoint_test.rs (RED)
/// 2. Implement this function (GREEN)
///
/// # Tracepoint: syscalls/sys_enter_openat
///
/// This tracepoint fires when the openat syscall is invoked. It provides
/// structured access to syscall arguments without needing to parse registers
/// or stack frames manually.
///
/// The openat syscall is used to open files relative to a directory file
/// descriptor. It's the modern replacement for open() and is used by most
/// file operations in modern Linux.
///
/// # Implementation Hints
///
/// - Use `TracePointContext` instead of `ProbeContext`
/// - Tracepoint args are available via `ctx.read_at::<T>(offset)`
/// - The offset values come from the tracepoint format file
/// - Category and name are specified at attach time in userspace
///
/// # Tracepoint Format (sys_enter_openat)
///
/// ```text
/// field:int __syscall_nr;       offset:8;  size:4;  signed:1;
/// field:int dfd;                offset:16; size:8;  signed:0;
/// field:const char * filename;  offset:24; size:8;  signed:0;
/// field:int flags;              offset:32; size:8;  signed:0;
/// field:umode_t mode;           offset:40; size:8;  signed:0;
/// ```
///
/// # Example Userspace Attachment
///
/// ```rust,ignore
/// let program: &mut TracePoint = bpf.program_mut("sys_enter_tracepoint")?.try_into()?;
/// program.load()?;
/// program.attach("syscalls", "sys_enter_openat")?;
/// ```
#[tracepoint]
pub fn sys_enter_tracepoint(ctx: TracePointContext) -> u32 {
    // TODO: Implement in Lesson 06
    // Lesson: docs/04-ebpf/06-tracepoints.md
    // Tests: crates/ebpf-tool/tests/tracepoint_test.rs
    //
    // Implementation steps:
    //
    // 1. Use match to handle the result of try_sys_enter_tracepoint
    //    - On Ok(ret) -> return ret
    //    - On Err(_) -> return 1 (or appropriate error code)
    //
    // 2. In try_sys_enter_tracepoint helper function:
    //    - Log when the tracepoint fires using info!()
    //    - Read the syscall number from offset 8
    //    - Read the dfd (directory file descriptor) from offset 16
    //    - Optionally read flags from offset 32
    //
    // 3. Common tracepoints to try after sys_enter_openat:
    //    - syscalls/sys_enter_read (file reads)
    //    - syscalls/sys_enter_write (file writes)
    //    - syscalls/sys_enter_execve (program execution)
    //
    // Example reading tracepoint args:
    //   let syscall_nr: i32 = unsafe { ctx.read_at(8)? };
    //   let dfd: i64 = unsafe { ctx.read_at(16)? };
    //   info!(&ctx, "openat syscall: dfd={}", dfd);

    todo!("Implement sys_enter_tracepoint - see docs/04-ebpf/06-tracepoints.md")
}

// =============================================================================
// Scheduler Tracepoints
// =============================================================================

/// Tracepoint for scheduler events.
///
/// # Tracepoint: sched/sched_switch
///
/// Fires on context switches - useful for understanding CPU scheduling
/// behavior, measuring process runtime, and detecting scheduling anomalies.
///
/// # Use Cases
///
/// - **Latency analysis**: Measure time between process activations
/// - **CPU utilization**: Track which processes use CPU time
/// - **Priority inversion**: Detect scheduling priority issues
/// - **Performance debugging**: Understand context switch overhead
///
/// # Tracepoint Format (sched_switch)
///
/// ```text
/// field:char prev_comm[16];     offset:8;  size:16; signed:0;
/// field:pid_t prev_pid;         offset:24; size:4;  signed:1;
/// field:int prev_prio;          offset:28; size:4;  signed:1;
/// field:long prev_state;        offset:32; size:8;  signed:1;
/// field:char next_comm[16];     offset:40; size:16; signed:0;
/// field:pid_t next_pid;         offset:56; size:4;  signed:1;
/// field:int next_prio;          offset:60; size:4;  signed:1;
/// ```
///
/// # Example Userspace Attachment
///
/// ```rust,ignore
/// let program: &mut TracePoint = bpf.program_mut("sched_tracepoint")?.try_into()?;
/// program.load()?;
/// program.attach("sched", "sched_switch")?;
/// ```
#[tracepoint]
pub fn sched_tracepoint(ctx: TracePointContext) -> u32 {
    // TODO: Implement in Lesson 06 (optional extension)
    // Lesson: docs/04-ebpf/06-tracepoints.md
    // Tests: crates/ebpf-tool/tests/tracepoint_test.rs
    //
    // Implementation hints:
    //
    // 1. Read prev_pid (offset 24) and next_pid (offset 56)
    // 2. Log the context switch: "switch: pid {} -> pid {}"
    // 3. Optional: Use a BPF map to track per-process CPU time
    // 4. Optional: Calculate time between switches using bpf_ktime_get_ns()
    //
    // Advanced extensions:
    // - Build per-CPU statistics using a PerCpuArray map
    // - Track scheduling latency (time from wake to run)
    // - Detect runaway processes hogging CPU
    //
    // Example:
    //   let prev_pid: i32 = unsafe { ctx.read_at(24)? };
    //   let next_pid: i32 = unsafe { ctx.read_at(56)? };
    //   info!(&ctx, "context switch: {} -> {}", prev_pid, next_pid);

    todo!("Implement sched_tracepoint - see docs/04-ebpf/06-tracepoints.md")
}

/// Tracepoint for process execution events.
///
/// # Tracepoint: sched/sched_process_exec
///
/// Fires when a process calls execve() to replace its image with a new
/// program. This is invaluable for security monitoring and auditing.
///
/// # Use Cases
///
/// - **Security monitoring**: Track all program executions
/// - **Audit logging**: Record who ran what and when
/// - **Container escapes**: Detect unexpected process execution
/// - **Malware detection**: Identify suspicious programs
#[tracepoint]
pub fn exec_tracepoint(ctx: TracePointContext) -> u32 {
    // TODO: Implement in Lesson 06 (optional extension)
    // Lesson: docs/04-ebpf/06-tracepoints.md
    // Tests: crates/ebpf-tool/tests/tracepoint_test.rs
    //
    // This tracepoint can capture:
    // - The filename being executed
    // - The PID of the process
    // - The old comm (process name) being replaced
    //
    // Check the format file for exact offsets:
    //   cat /sys/kernel/debug/tracing/events/sched/sched_process_exec/format

    todo!("Implement exec_tracepoint - see docs/04-ebpf/06-tracepoints.md")
}

// =============================================================================
// Network Tracepoints
// =============================================================================

/// Tracepoint for network packet receive events.
///
/// # Tracepoint: net/netif_rx
///
/// Fires when a network packet is received by a network interface.
/// Useful for monitoring network activity without the complexity of XDP.
///
/// # Use Cases
///
/// - **Traffic monitoring**: Count packets per interface
/// - **Debugging**: Trace packet flow through the network stack
/// - **Performance**: Measure packet processing latency
#[tracepoint]
pub fn net_rx_tracepoint(ctx: TracePointContext) -> u32 {
    // TODO: Implement in Lesson 06 (optional extension)
    // Lesson: docs/04-ebpf/06-tracepoints.md
    // Tests: crates/ebpf-tool/tests/tracepoint_test.rs
    //
    // Implementation hints:
    // - Read packet length from the tracepoint arguments
    // - Track packets per interface using a HashMap
    // - Calculate packet rates using timestamps
    //
    // Check the format file for exact offsets:
    //   cat /sys/kernel/debug/tracing/events/net/netif_rx/format

    todo!("Implement net_rx_tracepoint - see docs/04-ebpf/06-tracepoints.md")
}

// =============================================================================
// Understanding Tracepoint Format
// =============================================================================
//
// Each tracepoint has a format file that describes its arguments. This is
// crucial for knowing what data is available and at what offsets.
//
// Location: /sys/kernel/debug/tracing/events/<category>/<tracepoint>/format
//
// # Example: sys_enter_openat Format
//
// ```text
// name: sys_enter_openat
// ID: 614
// format:
//     field:unsigned short common_type;       offset:0;  size:2; signed:0;
//     field:unsigned char common_flags;       offset:2;  size:1; signed:0;
//     field:unsigned char common_preempt_cnt; offset:3;  size:1; signed:0;
//     field:int common_pid;                   offset:4;  size:4; signed:1;
//
//     field:int __syscall_nr;                 offset:8;  size:4; signed:1;
//     field:int dfd;                          offset:16; size:8; signed:0;
//     field:const char * filename;            offset:24; size:8; signed:0;
//     field:int flags;                        offset:32; size:8; signed:0;
//     field:umode_t mode;                     offset:40; size:8; signed:0;
//
// print fmt: "dfd: 0x%08lx, filename: 0x%08lx, flags: 0x%08lx, mode: 0x%08lx",
//            ((unsigned long)(REC->dfd)), ...
// ```
//
// # Reading Tracepoint Arguments
//
// Use ctx.read_at::<T>(offset) to read fields at their documented offsets:
//
// ```rust,ignore
// // Read the syscall number (offset 8, size 4, type i32)
// let syscall_nr: i32 = unsafe { ctx.read_at(8)? };
//
// // Read the directory file descriptor (offset 16, size 8, type i64)
// let dfd: i64 = unsafe { ctx.read_at(16)? };
//
// // Read the flags (offset 32, size 8, type i64)
// let flags: i64 = unsafe { ctx.read_at(32)? };
// ```
//
// # Common Pitfalls
//
// 1. **Pointer arguments**: Fields like `filename` contain pointers, not
//    the actual string. Use bpf_probe_read_user_str() to read the string.
//
// 2. **Architecture differences**: Some offsets may differ between 32-bit
//    and 64-bit systems. Always check the format file on your target.
//
// 3. **Kernel version changes**: While tracepoints are stable, the format
//    can occasionally change. Re-check after kernel upgrades.
//
// =============================================================================

// =============================================================================
// Helper Functions (to be implemented in Lesson 06)
// =============================================================================

// TODO (Lesson 06): Implement helper functions for common operations
//
// Suggested helpers:
//
// fn try_sys_enter_tracepoint(ctx: TracePointContext) -> Result<u32, i64> {
//     // Safe wrapper that handles errors properly
//     // Returns Result instead of panicking
// }
//
// fn read_syscall_nr(ctx: &TracePointContext) -> Result<i32, i64> {
//     // Read syscall number from standard offset
//     unsafe { ctx.read_at(8) }
// }
//
// fn get_current_pid_tgid() -> u64 {
//     // Get current PID/TGID for filtering
//     unsafe { aya_ebpf::helpers::bpf_get_current_pid_tgid() }
// }
