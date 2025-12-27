//! eBPF Perf Event Programs
//!
//! This module provides eBPF programs for perf event-based tracing using the Aya framework.
//!
//! # What are Perf Events?
//!
//! Perf events are the Linux kernel's performance monitoring infrastructure. They provide
//! access to both hardware performance counters (CPU cycles, cache misses, branch
//! mispredictions) and software events (context switches, page faults, CPU migrations).
//!
//! eBPF programs can attach to perf events in two primary ways:
//!
//! ## 1. PerfEventArray: User-Kernel Communication (Lesson 04)
//!
//! `PerfEventArray` is a specialized eBPF map type for streaming events from eBPF programs
//! to userspace in real-time. Unlike regular maps where userspace polls for data,
//! PerfEventArray uses the perf ring buffer infrastructure for efficient, low-overhead
//! event delivery.
//!
//! Key characteristics:
//! - Per-CPU buffers avoid lock contention
//! - Ring buffer semantics (can drop events if full)
//! - Userspace receives events via epoll/async I/O
//! - Ideal for high-frequency event streaming
//!
//! ```text
//! eBPF Program                    Userspace
//! +-----------+                   +-----------+
//! |  kprobe   |  EVENTS.output()  | AsyncPerf |
//! | tracepoint| ----------------> | EventArray|
//! |  uprobe   |    per-CPU ring   |   .read() |
//! +-----------+                   +-----------+
//! ```
//!
//! ## 2. Perf Sampling: CPU Profiling (Lesson 07)
//!
//! eBPF programs can attach directly to perf events (e.g., CPU cycles at 99Hz) to
//! implement profiling. On each sample, the eBPF program runs and can capture:
//! - Instruction pointer (IP) - where the CPU was executing
//! - Stack trace - the call chain that led here
//! - Process context (PID, TID, comm)
//!
//! This enables building tools like:
//! - CPU profilers (flame graphs)
//! - Off-CPU analysis
//! - Cache miss profiling
//!
//! ```text
//! Perf Event (99Hz)      eBPF Program         Analysis
//! +-------------+        +------------+       +------------+
//! | CPU cycles  | -----> | perf_sample| ----> | Flame Graph|
//! | timer tick  |  fire  | get stack  | send  | Hot spots  |
//! +-------------+        +------------+       +------------+
//! ```
//!
//! # Lessons Covered
//!
//! - **Lesson 04**: [Perf Events](docs/04-ebpf/04-perf-events.md) - PerfEventArray for
//!   streaming syscall events to userspace
//! - **Lesson 07**: [Perf Sampling](docs/04-ebpf/07-perf-sampling.md) - CPU profiling
//!   with stack traces for flame graphs
//!
//! # Safety Considerations
//!
//! All eBPF programs run in a sandboxed environment verified by the kernel. However:
//! - Memory accesses must be bounds-checked (the verifier enforces this)
//! - Stack size is limited to 512 bytes
//! - Loop iterations must be bounded (or use bpf_loop on newer kernels)
//! - Map operations can fail (check return values)

#![allow(unused_imports)] // Allow unused imports during scaffolding

use aya_ebpf::{
    macros::{map, perf_event},
    maps::PerfEventArray,
    programs::PerfEventContext,
    EbpfContext,
};
#[allow(unused_imports)]
use aya_log_ebpf::info;
use ebpf_tool_common::SyscallEvent;

// =============================================================================
// PerfEventArray Map (Lesson 04)
// =============================================================================

/// Perf event array for sending events to userspace.
///
/// # Lesson 04: User-Kernel Communication
///
/// This map is the primary mechanism for real-time event streaming from eBPF
/// programs to userspace. Each CPU has its own buffer to avoid lock contention,
/// making it suitable for high-frequency events.
///
/// # How It Works
///
/// 1. eBPF program populates a `SyscallEvent` struct
/// 2. Calls `EVENTS.output(&ctx, &event, 0)` to send it
/// 3. Event goes into the per-CPU ring buffer
/// 4. Userspace receives via `AsyncPerfEventArray::read()`
///
/// # Usage from eBPF
///
/// ```ignore
/// let event = SyscallEvent { pid, tid, syscall_nr, timestamp_ns, comm };
/// EVENTS.output(&ctx, &event, 0);
/// ```
///
/// # Usage from Userspace
///
/// ```ignore
/// // Get the map from the loaded eBPF object
/// let mut perf_array = AsyncPerfEventArray::try_from(bpf.take_map("EVENTS")?)?;
///
/// // Open a buffer for each online CPU
/// for cpu_id in online_cpus()? {
///     let mut buf = perf_array.open(cpu_id, None)?;
///
///     // Spawn async task to read events
///     tokio::spawn(async move {
///         let mut buffers = (0..10)
///             .map(|_| BytesMut::with_capacity(1024))
///             .collect::<Vec<_>>();
///
///         loop {
///             let events = buf.read_events(&mut buffers).await?;
///             for buf in buffers.iter().take(events.read) {
///                 let event: &SyscallEvent = unsafe { ... };
///                 // Process event
///             }
///         }
///     });
/// }
/// ```
///
/// # Performance Considerations
///
/// - The `0` flag uses the current CPU's buffer (most common)
/// - Use `BPF_F_CURRENT_CPU` for explicit current CPU
/// - Events are dropped if the ring buffer is full
/// - Userspace should read quickly to avoid drops
#[map]
static EVENTS: PerfEventArray<SyscallEvent> = PerfEventArray::new(0);

// =============================================================================
// Perf Event Program (Lesson 07)
// =============================================================================

/// Perf event program for CPU sampling.
///
/// # Lesson 07: Perf Sampling
///
/// This program is triggered by perf events (typically CPU cycles at a fixed
/// frequency like 99Hz). It's used for CPU profiling to identify hot spots
/// and generate flame graphs.
///
/// # TDD Steps
///
/// 1. Write tests in `crates/ebpf-tool/tests/perf_test.rs` (RED)
/// 2. Implement this function (GREEN)
/// 3. Run `cargo test -p ebpf-tool` to verify
///
/// # How Perf Sampling Works
///
/// ```text
/// 1. Userspace: perf_event_open(PERF_TYPE_SOFTWARE, PERF_COUNT_SW_CPU_CLOCK, 99Hz)
/// 2. Userspace: attach this eBPF program to the perf event fd
/// 3. Kernel: Every ~10ms (99Hz), fires this program on the sampled CPU
/// 4. eBPF: Captures IP, stack trace, PID, CPU, timestamp
/// 5. eBPF: Sends sample to userspace via PerfEventArray
/// 6. Userspace: Aggregates samples, generates flame graph
/// ```
///
/// # Implementation Hints
///
/// The `PerfEventContext` provides access to sample data:
///
/// ```ignore
/// // Get sample metadata
/// let cpu = unsafe { bpf_get_smp_processor_id() };
/// let pid = ctx.pid();
/// let tgid = ctx.tgid();
///
/// // Get instruction pointer (where CPU was executing)
/// // This requires reading from the perf sample data
/// let sample_addr = unsafe { (*ctx.as_ptr()).sample_addr };
///
/// // Capture stack trace (requires StackTraceMap)
/// let stack_id = unsafe {
///     bpf_get_stackid(ctx.as_ptr(), &STACKS as *const _ as *mut _, 0)
/// };
///
/// // Build and send event
/// let sample = PerfSampleEvent { pid, cpu, ip, stack_id, timestamp };
/// PERF_SAMPLES.output(&ctx, &sample, 0);
/// ```
///
/// # Why 99Hz?
///
/// We use 99Hz instead of 100Hz to avoid "lockstep" with timer interrupts.
/// Many systems have 100Hz or 250Hz timers, and sampling at exactly those
/// frequencies would bias samples toward timer handling code.
///
/// # Return Value
///
/// - Returns `0` on success
/// - Non-zero return values are logged but don't affect program execution
#[perf_event]
pub fn perf_sample(ctx: PerfEventContext) -> u32 {
    // TODO: Implement in Lesson 07
    // Lesson: docs/04-ebpf/07-perf-sampling.md
    // Tests: crates/ebpf-tool/tests/perf_test.rs
    //
    // Implementation checklist:
    //
    // [ ] Get current CPU: bpf_get_smp_processor_id()
    // [ ] Get PID/TID from context
    // [ ] Get instruction pointer from perf sample data
    // [ ] Capture stack trace with bpf_get_stackid() (requires STACKS map)
    // [ ] Build PerfSampleEvent struct (define in ebpf-tool-common)
    // [ ] Send event to userspace via PerfEventArray
    //
    // Example workflow:
    //
    // 1. Userspace opens perf event:
    //    ```
    //    let attr = perf_event_attr {
    //        type_: PERF_TYPE_SOFTWARE,
    //        config: PERF_COUNT_SW_CPU_CLOCK,
    //        sample_period: 1_000_000_000 / 99, // 99 Hz
    //        ...
    //    };
    //    let fd = perf_event_open(&attr, -1, cpu, -1, 0);
    //    ```
    //
    // 2. Attaches this eBPF program to the perf event fd
    //
    // 3. On each sample, this function runs with context about where
    //    the CPU was executing
    //
    // 4. We collect IP + stack, send to userspace
    //
    // 5. Userspace aggregates samples and generates flame graph
    //
    // Suppress unused variable warning during scaffolding
    let _ = &ctx;

    todo!("Implement perf_sample - see docs/04-ebpf/07-perf-sampling.md")
}

// =============================================================================
// Helper Functions (Lesson 04)
// =============================================================================

/// Helper to send an event through the perf event array.
///
/// # Lesson 04: Perf Events
///
/// This function demonstrates how to send structured events from eBPF to userspace.
/// It wraps the `PerfEventArray::output()` call with error handling guidance.
///
/// # Arguments
///
/// * `ctx` - The eBPF context (from kprobe, tracepoint, etc.)
/// * `event` - The event data to send
///
/// # Returns
///
/// * `Ok(())` - Event was successfully queued
/// * `Err(errno)` - Failed to send (usually -ENOENT if no userspace reader)
///
/// # Example
///
/// ```ignore
/// #[kprobe]
/// pub fn my_kprobe(ctx: ProbeContext) -> u32 {
///     let event = SyscallEvent {
///         pid: ctx.pid(),
///         tid: ctx.tgid(),
///         syscall_nr: 59, // execve
///         timestamp_ns: unsafe { bpf_ktime_get_ns() },
///         comm: [0u8; 16],
///     };
///
///     if let Err(e) = send_event(&ctx, &event) {
///         // Log error but don't fail the probe
///         info!(&ctx, "Failed to send event: {}", e);
///     }
///     0
/// }
/// ```
///
/// # Common Errors
///
/// - `-ENOENT` (-2): No userspace program is reading from the buffer
/// - `-ENOSPC` (-28): Ring buffer is full (userspace not reading fast enough)
#[allow(dead_code)]
fn send_event<C: EbpfContext>(ctx: &C, event: &SyscallEvent) -> Result<(), i64> {
    // TODO: Implement in Lesson 04
    // Lesson: docs/04-ebpf/04-perf-events.md
    //
    // Implementation:
    //
    // [ ] Call EVENTS.output(ctx, event, 0)
    //     - The 0 is flags (0 = use current CPU's buffer)
    //     - This is the most common usage pattern
    //
    // [ ] Handle the Result
    //     - output() returns Result<(), i64>
    //     - i64 is the errno on failure
    //
    // [ ] Consider logging on error (optional)
    //     - aya_log_ebpf::error!(&ctx, "send failed: {}", errno);
    //
    // Example implementation:
    // ```
    // EVENTS.output(ctx, event, 0)
    // ```
    //
    // That's it! The PerfEventArray handles the complexity of:
    // - Finding the right per-CPU buffer
    // - Copying data to the ring buffer
    // - Waking up userspace if needed
    //
    // Suppress unused variable warnings during scaffolding
    let _ = (ctx, event);

    todo!("Implement send_event - see docs/04-ebpf/04-perf-events.md")
}

// =============================================================================
// Stack Trace Map (Lesson 07)
// =============================================================================

// TODO (Lesson 07): Add a StackTraceMap for capturing call stacks
//
// StackTraceMap is a specialized BPF map that stores kernel and userspace
// stack traces. It's used with bpf_get_stackid() to capture call chains.
//
// Uncomment and implement in Lesson 07:
//
// use aya_ebpf::maps::StackTraceMap;
//
// /// Stack trace storage for CPU profiling.
// ///
// /// # How Stack Traces Work
// ///
// /// When a perf event fires, we can capture the stack trace:
// ///
// /// ```text
// /// bpf_get_stackid()
// ///        |
// ///        v
// ///   +----------+     +-----------------+
// ///   | Stack ID | --> | STACKS map      |
// ///   | (hash)   |     | [id] -> [frames]|
// ///   +----------+     +-----------------+
// /// ```
// ///
// /// The stack_id is a hash of the stack frames. Identical stacks get the
// /// same ID, enabling efficient aggregation. Userspace can later read the
// /// actual frame addresses from the map.
// ///
// /// # Map Size
// ///
// /// 10,000 entries is enough for most profiling sessions. Each entry stores
// /// up to 127 stack frames (PERF_MAX_STACK_DEPTH).
// ///
// /// # Flags for bpf_get_stackid()
// ///
// /// - `0`: Kernel stack only
// /// - `BPF_F_USER_STACK`: User stack only
// /// - `BPF_F_FAST_STACK_CMP`: Faster but may have more collisions
// ///
// /// # Usage
// ///
// /// ```ignore
// /// let kernel_stack_id = unsafe {
// ///     bpf_get_stackid(ctx.as_ptr(), &STACKS as *const _ as *mut _, 0)
// /// };
// ///
// /// let user_stack_id = unsafe {
// ///     bpf_get_stackid(
// ///         ctx.as_ptr(),
// ///         &STACKS as *const _ as *mut _,
// ///         BPF_F_USER_STACK
// ///     )
// /// };
// /// ```
// #[map]
// static STACKS: StackTraceMap = StackTraceMap::with_max_entries(10000, 0);
//
// Usage in perf_sample():
//
// ```ignore
// let stack_id = unsafe {
//     bpf_get_stackid(
//         ctx.as_ptr() as *mut _,
//         &STACKS as *const _ as *mut _,
//         0  // 0 = kernel stack, BPF_F_USER_STACK = user stack
//     )
// };
//
// // stack_id is now a unique identifier for this stack trace
// // Userspace can read STACKS[stack_id] to get the actual frames
// ```

// =============================================================================
// PerfSampleEvent Type (Lesson 07)
// =============================================================================

// TODO (Lesson 07): Define PerfSampleEvent in ebpf-tool-common
//
// Before implementing perf_sample(), add this struct to
// crates/ebpf-tool-common/src/lib.rs:
//
// ```rust
// /// Event generated during CPU sampling.
// ///
// /// Used for profiling and flame graph generation. The eBPF perf_event
// /// program populates this on each sample and sends it to userspace.
// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct PerfSampleEvent {
//     /// Process ID (tgid in kernel terms)
//     pub pid: u32,
//     /// Thread ID (pid in kernel terms)
//     pub tid: u32,
//     /// CPU where the sample was taken
//     pub cpu: u32,
//     /// Padding for alignment
//     pub _pad: u32,
//     /// Instruction pointer at sample time
//     pub ip: u64,
//     /// Kernel stack ID (from STACKS map, -1 if unavailable)
//     pub kernel_stack_id: i64,
//     /// User stack ID (from STACKS map, -1 if unavailable)
//     pub user_stack_id: i64,
//     /// Timestamp in nanoseconds (from bpf_ktime_get_ns)
//     pub timestamp_ns: u64,
//     /// Process command name (null-padded)
//     pub comm: [u8; 16],
// }
// ```
//
// Then update the EVENTS map type or add a separate PerfEventArray for samples.

// =============================================================================
// Module Tests
// =============================================================================

// Note: eBPF program testing is done via integration tests in ebpf-tool.
// The eBPF code itself cannot be unit tested in the traditional sense because
// it runs in the kernel, not userspace.
//
// Test strategy:
// 1. Compile eBPF programs successfully (build.rs handles this)
// 2. Load and attach in integration tests (tests/perf_test.rs)
// 3. Verify events are received in userspace
// 4. Check that stack traces are captured correctly
//
// See: crates/ebpf-tool/tests/perf_test.rs for the integration tests.
