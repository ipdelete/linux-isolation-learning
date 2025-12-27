//! Shared types between ebpf-tool userspace and eBPF programs
//!
//! This crate provides data structures shared between the userspace application
//! (`ebpf-tool`) and eBPF programs (`ebpf-tool-ebpf`). Because eBPF programs run
//! in the kernel, this crate must be `no_std` compatible.
//!
//! # Design Constraints
//!
//! - **No heap**: Use fixed-size types only (`u32`, `u64`, `[u8; N]`)
//! - **`#[repr(C)]`**: Required for consistent memory layout across boundaries
//! - **`Copy`**: Events are passed by value through perf buffers

#![no_std]

/// Maximum length of process command name (TASK_COMM_LEN in Linux kernel).
pub const COMM_LEN: usize = 16;

/// Maximum entries in syscall counter maps.
pub const MAX_MAP_ENTRIES: u32 = 10240;

// =============================================================================
// Syscall Event (Lessons 02-04, 08)
// =============================================================================

/// Event generated when a system call is invoked.
///
/// Used in kprobe-based syscall tracing. The eBPF program populates this
/// struct and sends it to userspace via a perf event array.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallEvent {
    /// Process ID (tgid in kernel terms)
    pub pid: u32,
    /// Thread ID (pid in kernel terms)
    pub tid: u32,
    /// System call number (architecture-dependent)
    pub syscall_nr: u64,
    /// Timestamp in nanoseconds (from bpf_ktime_get_ns)
    pub timestamp_ns: u64,
    /// Process command name (null-padded)
    pub comm: [u8; COMM_LEN],
}

impl SyscallEvent {
    /// Create a zeroed event (for initialization in eBPF programs).
    pub const fn new() -> Self {
        Self {
            pid: 0,
            tid: 0,
            syscall_nr: 0,
            timestamp_ns: 0,
            comm: [0u8; COMM_LEN],
        }
    }
}

impl Default for SyscallEvent {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Map Key (Lesson 03)
// =============================================================================

/// Key for syscall counting HashMap.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyscallKey {
    /// Process ID (0 for system-wide)
    pub pid: u32,
    pub _pad: u32,
    /// System call number
    pub syscall_nr: u64,
}

impl SyscallKey {
    pub const fn new(pid: u32, syscall_nr: u64) -> Self {
        Self {
            pid,
            _pad: 0,
            syscall_nr,
        }
    }
}

impl Default for SyscallKey {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

// =============================================================================
// TODO: Add more event types as you progress through lessons
// =============================================================================

// TODO (Lesson 05 - Uprobes): Add FunctionEvent struct
// Hints:
// - pid, tid, timestamp_ns (like SyscallEvent)
// - ip: u64 (instruction pointer)
// - is_return: u8 (0 = entry, 1 = return)
// - comm: [u8; COMM_LEN]
//
// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct FunctionEvent {
//     todo!("Define fields for uprobe events")
// }

// TODO (Lesson 06 - Tracepoints): Add TracepointEvent struct
// Hints:
// - Basic fields: pid, tid, timestamp_ns, comm
// - category: [u8; 32] (e.g., "sched", "syscalls")
// - name: [u8; 64] (e.g., "sched_process_exec")

// TODO (Lesson 07 - Perf Sampling): Add PerfSampleEvent struct
// Hints:
// - pid, tid, timestamp_ns, comm
// - cpu: u32 (which CPU the sample was taken on)
// - ip: u64 (instruction pointer at sample time)

// =============================================================================
// Tests - Learners implement these as they progress
// =============================================================================

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_syscall_event_size_and_alignment() {
        // TODO: Verify SyscallEvent has correct size for C interop
        //
        // Hints:
        // - Use core::mem::size_of::<SyscallEvent>()
        // - Expected: 4 + 4 + 8 + 8 + 16 = 40 bytes (may have padding)
        // - Use core::mem::align_of::<SyscallEvent>() to check alignment
        //
        // Why this matters: eBPF and userspace must agree on struct layout

        todo!("Verify SyscallEvent size is between 40-48 bytes")
    }

    #[test]
    fn test_syscall_event_is_copy() {
        // TODO: Verify SyscallEvent implements Copy
        //
        // Hints:
        // - Create a helper: fn assert_copy<T: Copy>() {}
        // - Call it with the type
        //
        // Why this matters: Events are passed by value through perf buffers

        todo!("Verify SyscallEvent implements Copy trait")
    }

    #[test]
    fn test_syscall_key_new() {
        // TODO: Test SyscallKey::new() creates correct key
        //
        // Hints:
        // - let key = SyscallKey::new(1234, 59); // PID 1234, execve
        // - assert_eq!(key.pid, 1234);
        // - assert_eq!(key.syscall_nr, 59);

        todo!("Test SyscallKey construction")
    }

    #[test]
    #[ignore] // Enable after implementing FunctionEvent in Lesson 05
    fn test_function_event() {
        // TODO (Lesson 05): Test FunctionEvent struct
        //
        // Hints:
        // - Verify size and alignment
        // - Test is_return field (0 or 1)

        todo!("Test FunctionEvent after implementing in Lesson 05")
    }

    #[test]
    #[ignore] // Enable after implementing TracepointEvent in Lesson 06
    fn test_tracepoint_event() {
        // TODO (Lesson 06): Test TracepointEvent struct

        todo!("Test TracepointEvent after implementing in Lesson 06")
    }

    #[test]
    #[ignore] // Enable after implementing PerfSampleEvent in Lesson 07
    fn test_perf_sample_event() {
        // TODO (Lesson 07): Test PerfSampleEvent struct

        todo!("Test PerfSampleEvent after implementing in Lesson 07")
    }
}
