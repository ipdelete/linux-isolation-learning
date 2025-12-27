//! eBPF Uprobe Programs for Userspace Function Tracing
//!
//! # What are Uprobes?
//!
//! Uprobes (userspace probes) allow you to dynamically trace function calls
//! in userspace applications. Unlike kprobes which trace kernel functions,
//! uprobes attach to specific functions in ELF binaries (executables or
//! shared libraries).
//!
//! # Difference from Kprobes
//!
//! | Aspect          | Kprobes                    | Uprobes                       |
//! |-----------------|----------------------------|-------------------------------|
//! | Target          | Kernel functions           | Userspace functions           |
//! | Location        | Kernel address space       | Process address space         |
//! | Symbols         | kallsyms                   | ELF symbol tables             |
//! | Scope           | System-wide                | Per-binary (all instances)    |
//! | Overhead        | Lower                      | Higher (context switches)     |
//!
//! # Use Cases
//!
//! - **Tracing library calls**: Monitor malloc/free, SSL/TLS functions, etc.
//! - **Debugging applications**: Trace specific function entries without recompiling
//! - **Performance analysis**: Measure function latency with entry/return probes
//! - **Security monitoring**: Detect suspicious API usage patterns
//!
//! # Common Uprobe Targets
//!
//! - `/usr/bin/bash:readline` - Traces bash command input
//! - `/lib/x86_64-linux-gnu/libc.so.6:malloc` - Traces memory allocation
//! - `/lib/x86_64-linux-gnu/libssl.so:SSL_read` - Traces SSL reads
//!
//! # Reference
//!
//! Lesson documentation: `docs/04-ebpf/05-uprobes.md`
//!
//! # TDD Workflow
//!
//! 1. Write tests in `crates/ebpf-tool/tests/uprobe_test.rs` (RED)
//! 2. Implement the uprobe functions below (GREEN)
//! 3. Verify with `cargo test -p ebpf-tool`

use aya_ebpf::{
    macros::uprobe,
    programs::ProbeContext,
};
use aya_log_ebpf::info;

// TODO (Lesson 05): Use FunctionEvent from ebpf-tool-common
// to send structured events to userspace.
//
// See: crates/ebpf-tool-common/src/lib.rs for the struct definition
// You'll need to:
// 1. Define FunctionEvent in ebpf-tool-common
// 2. Create a PerfEventArray map to send events
// 3. Populate and submit the event
//
// Example map definition:
// ```rust
// #[map]
// static UPROBE_EVENTS: PerfEventArray<FunctionEvent> = PerfEventArray::new(0);
// ```

/// Uprobe that traces userspace function calls.
///
/// # Lesson 05: Uprobes
///
/// This uprobe triggers when a specified userspace function is called.
/// The target binary and function symbol are specified at attach time
/// in the userspace loader.
///
/// # TDD Steps
///
/// 1. Write tests in `crates/ebpf-tool/tests/uprobe_test.rs` (RED)
/// 2. Implement this function (GREEN)
///
/// # How Uprobes Work
///
/// - Attach to a specific function in an ELF binary
/// - Trigger when that function is called by any process running that binary
/// - Can read function arguments from CPU registers
/// - Work on dynamically linked libraries (libc, libssl, etc.)
///
/// # Implementation Hints
///
/// - Similar to kprobes but for userspace functions
/// - Binary path and function symbol are specified at attach time
/// - Use `ProbeContext` to read function arguments
/// - Arguments follow calling convention (x86_64: rdi, rsi, rdx, rcx, r8, r9)
///
/// # Example Targets
///
/// ```text
/// /usr/bin/bash:readline     - traces bash readline calls
/// /lib/x86_64-linux-gnu/libc.so.6:malloc  - traces malloc calls
/// /lib/x86_64-linux-gnu/libc.so.6:open    - traces file opens
/// ```
#[uprobe]
pub fn hello_uprobe(ctx: ProbeContext) -> u32 {
    // TODO: Implement in Lesson 05
    // Lesson: docs/04-ebpf/05-uprobes.md
    // Tests: crates/ebpf-tool/tests/uprobe_test.rs
    //
    // Implementation steps:
    //
    // 1. Get process information:
    //    ```rust
    //    let pid = bpf_get_current_pid_tgid() >> 32;
    //    ```
    //
    // 2. Log that the uprobe was triggered:
    //    ```rust
    //    info!(&ctx, "uprobe triggered! pid={}", pid);
    //    ```
    //
    // 3. Read function arguments (optional):
    //    ```rust
    //    // First argument (x86_64: rdi register)
    //    let arg0: u64 = ctx.arg(0).unwrap_or(0);
    //    ```
    //
    // 4. Send event to userspace via PerfEventArray (advanced):
    //    ```rust
    //    let event = FunctionEvent {
    //        pid: pid as u32,
    //        timestamp: bpf_ktime_get_ns(),
    //        // ... other fields
    //    };
    //    UPROBE_EVENTS.output(&ctx, &event, 0);
    //    ```
    //
    // 5. Return 0 for success
    //
    // Common targets for testing:
    // - /usr/bin/bash:readline - traces bash readline calls
    // - /lib/x86_64-linux-gnu/libc.so.6:malloc - traces malloc

    todo!("Implement hello_uprobe - see docs/04-ebpf/05-uprobes.md")
}

/// Uretprobe that traces userspace function returns.
///
/// # What is a Uretprobe?
///
/// A uretprobe (userspace return probe) fires when a function returns,
/// complementing the entry uprobe. This allows you to:
///
/// - Capture return values
/// - Measure function execution duration (paired with entry probe)
/// - Track function call success/failure patterns
///
/// # TDD Steps
///
/// 1. Write tests in `crates/ebpf-tool/tests/uprobe_test.rs` (RED)
/// 2. Implement this function (GREEN)
///
/// # How Uretprobes Work
///
/// - Install a trampoline at function entry
/// - Replace return address to redirect through the trampoline
/// - Fire when the function returns (before returning to caller)
/// - Can read the return value from the appropriate register
///
/// # Return Value Access
///
/// The return value is in a register that depends on the architecture:
/// - x86_64: `rax` register
/// - ARM64: `x0` register
///
/// # Use Cases
///
/// - Track malloc/free return values to detect allocation failures
/// - Measure function latency when paired with entry probe
/// - Monitor API call success/failure rates
#[uprobe]
pub fn hello_uretprobe(ctx: ProbeContext) -> u32 {
    // TODO: Implement in Lesson 05 (optional extension)
    // Lesson: docs/04-ebpf/05-uprobes.md
    // Tests: crates/ebpf-tool/tests/uprobe_test.rs
    //
    // This is a uretprobe - it triggers on function return.
    //
    // Implementation steps:
    //
    // 1. Get process information:
    //    ```rust
    //    let pid = bpf_get_current_pid_tgid() >> 32;
    //    ```
    //
    // 2. Read the return value (architecture-dependent):
    //    ```rust
    //    // On x86_64, return value is in rax
    //    let ret_val: u64 = ctx.ret().unwrap_or(0);
    //    ```
    //
    // 3. Log the return:
    //    ```rust
    //    info!(&ctx, "function returned: {} (pid={})", ret_val, pid);
    //    ```
    //
    // 4. For duration tracking, use a HashMap to store entry timestamps:
    //    ```rust
    //    // Entry probe stores: ENTRY_TIMES.insert(&pid, &timestamp, 0);
    //    // Return probe reads and calculates duration
    //    if let Some(entry_time) = ENTRY_TIMES.get(&pid) {
    //        let duration = bpf_ktime_get_ns() - *entry_time;
    //        info!(&ctx, "function took {} ns", duration);
    //    }
    //    ```
    //
    // 5. Return 0 for success

    todo!("Implement hello_uretprobe - see docs/04-ebpf/05-uprobes.md")
}

// =============================================================================
// Advanced: Structured Event Reporting (for Lesson 05 extension)
// =============================================================================
//
// To send structured events to userspace, you'll need:
//
// 1. Define FunctionEvent in ebpf-tool-common/src/lib.rs:
//    ```rust
//    #[repr(C)]
//    pub struct FunctionEvent {
//        pub pid: u32,
//        pub tid: u32,
//        pub timestamp: u64,
//        pub function_addr: u64,
//        pub arg0: u64,
//        pub ret_val: u64,
//        pub duration_ns: u64,
//        pub comm: [u8; 16],
//    }
//    ```
//
// 2. Create a PerfEventArray map in this file:
//    ```rust
//    use aya_ebpf::maps::PerfEventArray;
//
//    #[map]
//    static UPROBE_EVENTS: PerfEventArray<FunctionEvent> = PerfEventArray::new(0);
//    ```
//
// 3. For duration tracking, use a HashMap:
//    ```rust
//    use aya_ebpf::maps::HashMap;
//
//    #[map]
//    static ENTRY_TIMES: HashMap<u32, u64> = HashMap::with_max_entries(10240, 0);
//    ```
//
// 4. In the userspace program, receive events via the perf buffer.

// =============================================================================
// Helper function examples (uncomment when implementing)
// =============================================================================
//
// /// Try to execute the uprobe logic, returning a Result for cleaner error handling.
// fn try_hello_uprobe(ctx: &ProbeContext) -> Result<(), i64> {
//     let pid = bpf_get_current_pid_tgid() >> 32;
//     info!(ctx, "uprobe triggered! pid={}", pid);
//     Ok(())
// }
//
// /// Try to execute the uretprobe logic.
// fn try_hello_uretprobe(ctx: &ProbeContext) -> Result<(), i64> {
//     let pid = bpf_get_current_pid_tgid() >> 32;
//     let ret_val: u64 = ctx.ret().unwrap_or(0);
//     info!(ctx, "function returned: {} (pid={})", ret_val, pid);
//     Ok(())
// }
