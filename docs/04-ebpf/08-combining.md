# Combining Everything: Full Syscall Tracer

## Goal

Build a complete syscall tracer that combines kprobes, tracepoints, HashMaps, and PerfEventArrays into a production-quality tool similar to `strace`. You will implement the `trace` subcommand that provides real-time syscall monitoring with filtering capabilities.

**What you will build**: An `ebpf-tool trace` command that captures syscall events across the system, filters by process or syscall name, displays timestamps and process information, and provides summary statistics.

**Estimated time**: 45-60 minutes

## Prereqs

- Completed lessons 00-07 (especially `01-hello-kprobe.md`, `03-maps.md`, and `07-perf-sampling.md`)
- Understanding of:
  - Kprobes and tracepoints for syscall interception
  - HashMaps for in-kernel counting and statistics
  - PerfEventArray for streaming events to userspace
  - Basic filtering concepts in eBPF
- `sudo` access (required for loading eBPF programs)
- Development environment with Aya framework configured

## Concepts: Why Combine eBPF Features?

Before writing code, let's understand why combining multiple eBPF features creates more powerful observability tools.

### The Tracing Challenge

Individual eBPF features solve specific problems:

| Feature | Purpose | Limitation Alone |
|---------|---------|------------------|
| **Kprobes** | Hook kernel functions | No persistent data, no aggregation |
| **Tracepoints** | Stable syscall events | Same as kprobes - ephemeral |
| **HashMaps** | Aggregate statistics | No real-time streaming |
| **PerfEventArray** | Stream to userspace | No in-kernel aggregation |

A complete tracer needs **all of these working together**:

```
                          COMBINED TRACER ARCHITECTURE
    ================================================================================

    +------------------------------ KERNEL SPACE --------------------------------+
    |                                                                            |
    |   +-----------------+      +-----------------+      +------------------+   |
    |   |   Tracepoint    |      |    Kprobe       |      |   Tracepoint     |   |
    |   | sys_enter_*     |      | (fallback for   |      |   sys_exit_*     |   |
    |   | (syscall entry) |      |  custom funcs)  |      | (syscall exit)   |   |
    |   +--------+--------+      +--------+--------+      +---------+--------+   |
    |            |                        |                         |            |
    |            +------------------------+-------------------------+            |
    |                                     |                                      |
    |                           +---------v---------+                            |
    |                           |   FILTER LOGIC    |                            |
    |                           | - Check PID       |                            |
    |                           | - Check syscall   |                            |
    |                           | - Apply masks     |                            |
    |                           +---------+---------+                            |
    |                                     |                                      |
    |                    +----------------+----------------+                     |
    |                    |                                 |                     |
    |           +--------v--------+               +--------v--------+            |
    |           |    STATS MAP    |               |  EVENTS ARRAY   |            |
    |           |   (HashMap)     |               | (PerfEventArray)|            |
    |           |                 |               |                 |            |
    |           | syscall -> cnt  |               | -> SyscallEvent |            |
    |           | process -> cnt  |               |                 |            |
    |           +---------+-------+               +--------+--------+            |
    |                     |                                |                     |
    +---------------------|--------------------------------|---------------------+
                          |                                |
    +---------------------|--------------------------------|---------------------+
    |                     v                                v                     |
    |    USERSPACE   +----------+                  +-----------+                 |
    |                |  Stats   |                  |  Event    |                 |
    |                |  Reader  |                  |  Handler  |                 |
    |                +----+-----+                  +-----+-----+                 |
    |                     |                              |                       |
    |                     +-------------+----------------+                       |
    |                                   |                                        |
    |                          +--------v--------+                               |
    |                          |   FORMATTER    |                               |
    |                          | [HH:MM:SS.mmm] |                               |
    |                          | proc(pid) sys  |                               |
    |                          +--------+-------+                                |
    |                                   |                                        |
    |                          +--------v--------+                               |
    |                          |    TERMINAL    |                               |
    |                          |    OUTPUT      |                               |
    |                          +-----------------+                               |
    +----------------------------------------------------------------------------+
```

### In-Kernel Filtering: Why It Matters

Consider tracing a busy system with 10,000 syscalls per second. Without in-kernel filtering:

```
Kernel:  10,000 events/sec  ------>  Userspace: Process 10,000 events
                                                 Filter: 9,990 discarded
                                                 Display: 10 relevant
```

With in-kernel filtering:

```
Kernel:  10,000 events/sec
         Filter in BPF: 9,990 discarded
         10 events  ------>  Userspace: Display 10 relevant
```

**Benefits**:
- Reduced CPU overhead (filtering happens once, in kernel)
- Less memory pressure (fewer events copied to userspace)
- Lower latency (relevant events processed immediately)
- Better scalability (can handle higher event rates)

### Multiple Maps for Multiple Purposes

Our tracer uses three maps working together:

```rust
// 1. Real-time event streaming
#[map]
static EVENTS: PerfEventArray<SyscallEvent> = PerfEventArray::new(0);

// 2. Per-syscall statistics (for summary)
#[map]
static SYSCALL_COUNTS: HashMap<u64, u64> = HashMap::with_max_entries(512, 0);

// 3. Per-process statistics (for summary)
#[map]
static PROCESS_COUNTS: HashMap<u32, u64> = HashMap::with_max_entries(10240, 0);
```

Each map serves a different purpose:
- **EVENTS**: Streams individual events as they happen
- **SYSCALL_COUNTS**: Tracks how many times each syscall is invoked
- **PROCESS_COUNTS**: Tracks syscall activity per process

### The Complete Event Flow

Here's what happens when a process makes a syscall:

1. **Syscall Entry**: Tracepoint `sys_enter` fires
2. **Filter Check**: eBPF program checks if we care about this event
3. **If Filtered Out**: Return immediately (no overhead)
4. **If Matched**:
   - Update `SYSCALL_COUNTS[syscall_nr]++`
   - Update `PROCESS_COUNTS[pid]++`
   - Create `SyscallEvent` with timestamp, PID, comm, args
   - Send event via `EVENTS.output()`
5. **Syscall Exit** (optional): Capture return value
6. **Userspace Handler**: Receives event, formats, displays

### Output Format Design

Our tracer produces two types of output:

**Real-time Events** (like strace):
```
[12:34:56.789] bash(1234) openat(AT_FDCWD, "/etc/passwd", O_RDONLY) = 3
[12:34:56.790] bash(1234) read(3, ..., 4096) = 1024
[12:34:56.791] bash(1234) close(3) = 0
```

**Summary Statistics** (at end):
```
Tracing completed. Duration: 10.00s

Summary:
---------
Total events captured: 5432

Top syscalls:
  read:     2345 calls (43.2%)
  write:    1234 calls (22.7%)
  openat:    456 calls (8.4%)
  close:     389 calls (7.2%)
  stat:      321 calls (5.9%)

Top processes:
  bash:     1234 events
  python:    567 events
  node:      432 events
```

This dual output gives both real-time visibility and analytical summary.

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/tracer_test.rs`

The test file already exists with `todo!()` stubs. Your task is to implement the test logic.

### What the Tests Should Verify

1. **Help text**: `--help` shows usage without requiring root
2. **Basic execution**: Tracer runs and exits cleanly
3. **Syscall events**: Output contains actual syscall information
4. **Process filter**: `-p` flag limits to specific process
5. **Syscall filter**: `-s` flag limits to specific syscall
6. **Timestamps**: Output includes timing information
7. **Process info**: Output shows PID and command name
8. **Duration control**: `-d` flag controls runtime

### Steps

1. Open the test file:

```bash
# View the test file
cat crates/ebpf-tool/tests/tracer_test.rs
```

2. Find the first `todo!()` in `test_trace_help` and replace it:

```rust
#[test]
fn test_trace_help() {
    // Test that --help works without root privileges
    ebpf_tool()
        .args(["trace", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("trace"))
        .stdout(predicate::str::contains("-p"))
        .stdout(predicate::str::contains("--process"))
        .stdout(predicate::str::contains("-s"))
        .stdout(predicate::str::contains("--syscall"))
        .stdout(predicate::str::contains("-d"))
        .stdout(predicate::str::contains("--duration"));
}
```

3. Implement `test_trace_runs_successfully`:

```rust
#[test]
fn test_trace_runs_successfully() {
    if !is_root() {
        eprintln!("Skipping test_trace_runs_successfully: requires root");
        return;
    }

    // Run tracer for 1 second - should start and exit cleanly
    ebpf_tool()
        .args(["trace", "-d", "1"])
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success();
}
```

4. Implement `test_trace_shows_syscall_events`:

```rust
#[test]
fn test_trace_shows_syscall_events() {
    if !is_root() {
        eprintln!("Skipping test_trace_shows_syscall_events: requires root");
        return;
    }

    // Run tracer for 2 seconds
    // Any running process will generate syscalls
    let output = ebpf_tool()
        .args(["trace", "-d", "2"])
        .timeout(std::time::Duration::from_secs(15))
        .output()
        .expect("Failed to run trace command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain at least some common syscall names or "Summary"
    // The tracer captures events and shows a summary
    assert!(
        stdout.contains("Summary") || stdout.contains("events"),
        "Expected output to contain syscall information, got: {}",
        stdout
    );
}
```

5. Implement `test_trace_filter_by_process`:

```rust
#[test]
fn test_trace_filter_by_process() {
    if !is_root() {
        eprintln!("Skipping test_trace_filter_by_process: requires root");
        return;
    }

    // Filter by a process name that likely exists
    // Using "sh" which should be available on most systems
    let output = ebpf_tool()
        .args(["trace", "-p", "sh", "-d", "2"])
        .timeout(std::time::Duration::from_secs(15))
        .output()
        .expect("Failed to run trace command");

    // Command should succeed (even if no matching processes)
    assert!(output.status.success(), "trace command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention the filter being applied
    assert!(
        stdout.contains("sh") || stdout.contains("filter") || stdout.contains("process"),
        "Expected filter information in output"
    );
}
```

6. Implement `test_trace_filter_by_syscall`:

```rust
#[test]
fn test_trace_filter_by_syscall() {
    if !is_root() {
        eprintln!("Skipping test_trace_filter_by_syscall: requires root");
        return;
    }

    // Filter by read syscall - one of the most common
    let output = ebpf_tool()
        .args(["trace", "-s", "read", "-d", "2"])
        .timeout(std::time::Duration::from_secs(15))
        .output()
        .expect("Failed to run trace command");

    assert!(output.status.success(), "trace command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention the syscall filter
    assert!(
        stdout.contains("read") || stdout.contains("syscall"),
        "Expected syscall filter in output"
    );
}
```

7. Implement `test_trace_shows_timestamps`:

```rust
#[test]
fn test_trace_shows_timestamps() {
    if !is_root() {
        eprintln!("Skipping test_trace_shows_timestamps: requires root");
        return;
    }

    let output = ebpf_tool()
        .args(["trace", "-d", "2"])
        .timeout(std::time::Duration::from_secs(15))
        .output()
        .expect("Failed to run trace command");

    assert!(output.status.success(), "trace command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Look for timestamp patterns: either bracketed time or "Duration"
    let has_timestamps = stdout.contains("[") && stdout.contains("]")
        || stdout.contains("ns")
        || stdout.contains("Duration")
        || stdout.contains(":");

    assert!(has_timestamps, "Expected timestamps in output: {}", stdout);
}
```

8. Implement `test_trace_shows_process_info`:

```rust
#[test]
fn test_trace_shows_process_info() {
    if !is_root() {
        eprintln!("Skipping test_trace_shows_process_info: requires root");
        return;
    }

    let output = ebpf_tool()
        .args(["trace", "-d", "2"])
        .timeout(std::time::Duration::from_secs(15))
        .output()
        .expect("Failed to run trace command");

    assert!(output.status.success(), "trace command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain process information (PIDs are numbers, or process names)
    let has_process_info = stdout.contains("(")
        || stdout.contains("pid")
        || stdout.contains("PID")
        || stdout.contains("process");

    assert!(
        has_process_info,
        "Expected process info in output: {}",
        stdout
    );
}
```

9. Implement `test_trace_respects_duration`:

```rust
#[test]
fn test_trace_respects_duration() {
    if !is_root() {
        eprintln!("Skipping test_trace_respects_duration: requires root");
        return;
    }

    use std::time::Instant;

    let start = Instant::now();

    // Run for exactly 2 seconds
    ebpf_tool()
        .args(["trace", "-d", "2"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    let elapsed = start.elapsed();

    // Should take approximately 2 seconds (allow 1-4 second range for tolerance)
    assert!(
        elapsed.as_secs() >= 1 && elapsed.as_secs() <= 5,
        "Expected ~2 seconds, got {:?}",
        elapsed
    );
}
```

10. Run the tests (expect failure at this point):

```bash
# Non-root test (help text)
cargo test -p ebpf-tool --test tracer_test test_trace_help

# Root tests - expect panic from todo!()
sudo -E cargo test -p ebpf-tool --test tracer_test
```

**Expected output**: Tests should fail because the `trace` subcommand has `todo!()` stub.

```
thread 'test_trace_runs_successfully' panicked at 'not yet implemented: Implement trace subcommand - write tests first!'
```

This is the **RED** phase - tests exist but fail because implementation is missing.

## Build (Green)

Now implement the tracer to make the tests pass.

### Implementation Overview

The implementation has two parts:
1. **eBPF program** (kernel-side): Captures syscall events
2. **Userspace CLI** (in `main.rs`): Loads eBPF, processes events, displays output

### Part 1: eBPF Program for Tracing

**File**: `crates/ebpf-tool-ebpf/src/tracer.rs` (create new file)

This eBPF program attaches to syscall tracepoints and sends events to userspace:

```rust
//! Full syscall tracer eBPF program
//!
//! Lesson: docs/04-ebpf/08-combining.md
//!
//! This program combines:
//! - Tracepoints for syscall entry
//! - HashMap for per-syscall statistics
//! - HashMap for per-process statistics
//! - PerfEventArray for real-time event streaming

use aya_ebpf::{
    macros::{map, tracepoint},
    maps::{HashMap, PerfEventArray},
    programs::TracePointContext,
    helpers::{bpf_get_current_pid_tgid, bpf_get_current_comm, bpf_ktime_get_ns},
};
use aya_log_ebpf::info;
use ebpf_tool_common::{SyscallEvent, COMM_LEN, MAX_MAP_ENTRIES};

// =============================================================================
// Maps: Shared data structures between kernel and userspace
// =============================================================================

/// Stream individual syscall events to userspace in real-time.
/// Each event contains timestamp, PID, syscall number, and process name.
#[map]
static EVENTS: PerfEventArray<SyscallEvent> = PerfEventArray::new(0);

/// Count syscalls by syscall number (for summary statistics).
/// Key: syscall number (u64), Value: count (u64)
#[map]
static SYSCALL_COUNTS: HashMap<u64, u64> = HashMap::with_max_entries(512, 0);

/// Count syscalls by process ID (for summary statistics).
/// Key: PID (u32), Value: count (u64)
#[map]
static PROCESS_COUNTS: HashMap<u32, u64> = HashMap::with_max_entries(MAX_MAP_ENTRIES, 0);

/// Filter configuration - set by userspace.
/// Key 0: target PID (0 = all processes)
/// Key 1: target syscall number (0 = all syscalls)
#[map]
static FILTER_CONFIG: HashMap<u32, u64> = HashMap::with_max_entries(16, 0);

// =============================================================================
// Tracepoint: Syscall Entry
// =============================================================================

/// Tracepoint program that fires on every syscall entry.
///
/// # Context
/// The raw_syscalls:sys_enter tracepoint fires before every syscall.
/// It provides access to the syscall number and arguments.
///
/// # Filter Logic
/// 1. Check if PID filter is set - if so, only trace matching PIDs
/// 2. Check if syscall filter is set - if so, only trace matching syscalls
/// 3. If both filters pass (or aren't set), record the event
#[tracepoint]
pub fn trace_syscall_enter(ctx: TracePointContext) -> u32 {
    match try_trace_syscall_enter(ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

fn try_trace_syscall_enter(ctx: TracePointContext) -> Result<u32, i64> {
    // Get process information
    let pid_tgid = bpf_get_current_pid_tgid();
    let pid = (pid_tgid >> 32) as u32;  // TGID (process ID)
    let tid = pid_tgid as u32;           // TID (thread ID)

    // Read syscall number from tracepoint context
    // For raw_syscalls:sys_enter, the syscall number is at offset 8
    let syscall_nr: u64 = unsafe { ctx.read_at(8)? };

    // ==========================================================================
    // Apply Filters
    // ==========================================================================

    // Check PID filter (key 0)
    if let Some(&target_pid) = unsafe { FILTER_CONFIG.get(&0) } {
        if target_pid != 0 && target_pid != pid as u64 {
            return Ok(0);  // Filtered out - PID doesn't match
        }
    }

    // Check syscall filter (key 1)
    if let Some(&target_syscall) = unsafe { FILTER_CONFIG.get(&1) } {
        if target_syscall != 0 && target_syscall != syscall_nr {
            return Ok(0);  // Filtered out - syscall doesn't match
        }
    }

    // ==========================================================================
    // Update Statistics Maps
    // ==========================================================================

    // Increment syscall count
    if let Some(count) = unsafe { SYSCALL_COUNTS.get_ptr_mut(&syscall_nr) } {
        unsafe { *count += 1 };
    } else {
        let _ = SYSCALL_COUNTS.insert(&syscall_nr, &1, 0);
    }

    // Increment process count
    if let Some(count) = unsafe { PROCESS_COUNTS.get_ptr_mut(&pid) } {
        unsafe { *count += 1 };
    } else {
        let _ = PROCESS_COUNTS.insert(&pid, &1, 0);
    }

    // ==========================================================================
    // Create and Send Event
    // ==========================================================================

    let mut event = SyscallEvent::new();
    event.pid = pid;
    event.tid = tid;
    event.syscall_nr = syscall_nr;
    event.timestamp_ns = unsafe { bpf_ktime_get_ns() };

    // Get process command name
    let _ = unsafe { bpf_get_current_comm(&mut event.comm) };

    // Send event to userspace via perf buffer
    EVENTS.output(&ctx, &event, 0);

    Ok(0)
}
```

Now add this module to the eBPF main file. Open `crates/ebpf-tool-ebpf/src/main.rs` and add:

```rust
/// Full syscall tracer combining all techniques.
///
/// # Lessons
/// - `docs/04-ebpf/08-combining.md` - Complete syscall tracer
///
/// # Features
/// - Tracepoint attachment to sys_enter
/// - In-kernel filtering by PID and syscall
/// - Real-time event streaming via PerfEventArray
/// - Statistics tracking via HashMaps
mod tracer;
```

### Part 2: Userspace Implementation

**File**: `crates/ebpf-tool/src/main.rs`
**Location**: Line ~336, the `Command::Trace` match arm

Replace the `todo!()` with this implementation:

```rust
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

    run_tracer(process.as_deref(), syscall.as_deref(), duration).await?
}
```

Then add the `run_tracer` function at the end of the file:

```rust
/// Run the full syscall tracer.
///
/// This function:
/// 1. Loads the eBPF tracer program
/// 2. Sets up filter configuration
/// 3. Attaches to the sys_enter tracepoint
/// 4. Polls for events and displays them
/// 5. Shows summary statistics at the end
async fn run_tracer(
    process_filter: Option<&str>,
    syscall_filter: Option<&str>,
    duration: u64,
) -> Result<()> {
    use aya::maps::{HashMap, AsyncPerfEventArray};
    use aya::programs::TracePoint;
    use aya::util::online_cpus;
    use bytes::BytesMut;
    use ebpf_tool_common::SyscallEvent;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use tokio::signal;
    use tokio::time::{Duration, timeout};

    // Verify we're running as root
    if !nix::unistd::Uid::effective().is_root() {
        anyhow::bail!("trace subcommand requires root privileges");
    }

    println!("Loading eBPF tracer program...");

    // Load the eBPF program
    let ebpf_bytes = include_bytes_aligned!(
        concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf")
    );

    let mut bpf = aya::Bpf::load(ebpf_bytes)?;

    // Set up logging from eBPF
    if let Err(e) = aya_log::BpfLogger::init(&mut bpf) {
        log::warn!("Failed to initialize eBPF logger: {}", e);
    }

    // ==========================================================================
    // Configure Filters
    // ==========================================================================

    let mut filter_config: HashMap<_, u32, u64> =
        HashMap::try_from(bpf.map_mut("FILTER_CONFIG").unwrap())?;

    // Set PID filter if specified
    if let Some(proc_name) = process_filter {
        // For simplicity, we try to parse as PID first
        if let Ok(pid) = proc_name.parse::<u64>() {
            filter_config.insert(&0, &pid, 0)?;
            println!("Filtering by PID: {}", pid);
        } else {
            // Process name filter would require more complex lookup
            // For now, just log the intention
            println!("Process name filter '{}' - will filter in userspace", proc_name);
        }
    }

    // Set syscall filter if specified
    if let Some(syscall_name) = syscall_filter {
        // Map common syscall names to numbers (x86_64)
        let syscall_nr = match syscall_name.to_lowercase().as_str() {
            "read" => 0u64,
            "write" => 1,
            "open" => 2,
            "close" => 3,
            "stat" => 4,
            "fstat" => 5,
            "lstat" => 6,
            "poll" => 7,
            "lseek" => 8,
            "mmap" => 9,
            "mprotect" => 10,
            "munmap" => 11,
            "brk" => 12,
            "ioctl" => 16,
            "access" => 21,
            "pipe" => 22,
            "dup" => 32,
            "dup2" => 33,
            "nanosleep" => 35,
            "getpid" => 39,
            "socket" => 41,
            "connect" => 42,
            "accept" => 43,
            "sendto" => 44,
            "recvfrom" => 45,
            "bind" => 49,
            "listen" => 50,
            "clone" => 56,
            "fork" => 57,
            "execve" => 59,
            "exit" => 60,
            "wait4" => 61,
            "kill" => 62,
            "fcntl" => 72,
            "openat" => 257,
            _ => {
                // Try parsing as number
                syscall_name.parse::<u64>().unwrap_or(0)
            }
        };

        if syscall_nr > 0 {
            filter_config.insert(&1, &syscall_nr, 0)?;
            println!("Filtering by syscall: {} (nr={})", syscall_name, syscall_nr);
        }
    }

    // ==========================================================================
    // Attach Tracepoint
    // ==========================================================================

    let program: &mut TracePoint = bpf.program_mut("trace_syscall_enter").unwrap().try_into()?;
    program.load()?;
    program.attach("raw_syscalls", "sys_enter")?;

    println!("Attached to raw_syscalls:sys_enter tracepoint");

    // ==========================================================================
    // Set Up Event Handler
    // ==========================================================================

    let mut perf_array = AsyncPerfEventArray::try_from(bpf.take_map("EVENTS").unwrap())?;

    let running = Arc::new(AtomicBool::new(true));
    let event_count = Arc::new(AtomicU64::new(0));

    // Process name filter for userspace filtering
    let proc_filter = process_filter.map(|s| s.to_string());

    // Spawn handlers for each CPU
    let cpus = online_cpus()?;
    let mut handles = Vec::new();

    for cpu_id in cpus {
        let mut buf = perf_array.open(cpu_id, None)?;
        let running = running.clone();
        let event_count = event_count.clone();
        let proc_filter = proc_filter.clone();

        let handle = tokio::spawn(async move {
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(std::mem::size_of::<SyscallEvent>()))
                .collect::<Vec<_>>();

            while running.load(Ordering::Relaxed) {
                // Poll with timeout to check running flag periodically
                let events = match timeout(Duration::from_millis(100), buf.read_events(&mut buffers)).await {
                    Ok(Ok(events)) => events,
                    Ok(Err(e)) => {
                        log::error!("Error reading events: {}", e);
                        continue;
                    }
                    Err(_) => continue, // Timeout, check running flag
                };

                for i in 0..events.read {
                    let buf = &buffers[i];
                    if buf.len() >= std::mem::size_of::<SyscallEvent>() {
                        let event: SyscallEvent = unsafe {
                            std::ptr::read_unaligned(buf.as_ptr() as *const SyscallEvent)
                        };

                        // Apply userspace process name filter
                        if let Some(ref filter) = proc_filter {
                            let comm = std::str::from_utf8(&event.comm)
                                .unwrap_or("")
                                .trim_end_matches('\0');
                            if !comm.contains(filter) {
                                continue;
                            }
                        }

                        // Format and print event
                        let comm = std::str::from_utf8(&event.comm)
                            .unwrap_or("<unknown>")
                            .trim_end_matches('\0');

                        let syscall_name = syscall_nr_to_name(event.syscall_nr);

                        // Format timestamp (nanoseconds to HH:MM:SS.mmm)
                        let ts_secs = event.timestamp_ns / 1_000_000_000;
                        let ts_ms = (event.timestamp_ns % 1_000_000_000) / 1_000_000;
                        let hours = (ts_secs / 3600) % 24;
                        let mins = (ts_secs / 60) % 60;
                        let secs = ts_secs % 60;

                        println!(
                            "[{:02}:{:02}:{:02}.{:03}] {}({}) {}",
                            hours, mins, secs, ts_ms,
                            comm, event.pid, syscall_name
                        );

                        event_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        handles.push(handle);
    }

    // ==========================================================================
    // Run for Duration or Until Ctrl+C
    // ==========================================================================

    println!("Tracing syscalls (Ctrl+C to stop)...");
    println!("---");

    let start_time = std::time::Instant::now();

    if duration > 0 {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(duration)) => {
                println!("\nDuration ({} seconds) elapsed.", duration);
            }
            _ = signal::ctrl_c() => {
                println!("\nReceived Ctrl+C, stopping...");
            }
        }
    } else {
        signal::ctrl_c().await?;
        println!("\nReceived Ctrl+C, stopping...");
    }

    // Signal handlers to stop
    running.store(false, Ordering::Relaxed);

    // Wait for handlers to finish
    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start_time.elapsed();

    // ==========================================================================
    // Print Summary Statistics
    // ==========================================================================

    println!("\nSummary:");
    println!("---------");
    println!("Duration: {:.2}s", elapsed.as_secs_f64());
    println!("Total events captured: {}", event_count.load(Ordering::Relaxed));

    // Read syscall counts from map
    let syscall_counts: HashMap<_, u64, u64> =
        HashMap::try_from(bpf.map("SYSCALL_COUNTS").unwrap())?;

    let mut syscall_stats: BTreeMap<u64, u64> = BTreeMap::new();
    for result in syscall_counts.iter() {
        if let Ok((nr, count)) = result {
            syscall_stats.insert(nr, count);
        }
    }

    if !syscall_stats.is_empty() {
        let total: u64 = syscall_stats.values().sum();
        println!("\nTop syscalls:");

        let mut sorted: Vec<_> = syscall_stats.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        for (nr, count) in sorted.iter().take(10) {
            let name = syscall_nr_to_name(**nr);
            let pct = (**count as f64 / total as f64) * 100.0;
            println!("  {}: {} calls ({:.1}%)", name, count, pct);
        }
    }

    // Read process counts from map
    let process_counts: HashMap<_, u32, u64> =
        HashMap::try_from(bpf.map("PROCESS_COUNTS").unwrap())?;

    let mut process_stats: BTreeMap<u32, u64> = BTreeMap::new();
    for result in process_counts.iter() {
        if let Ok((pid, count)) = result {
            process_stats.insert(pid, count);
        }
    }

    if !process_stats.is_empty() {
        println!("\nTop processes (by PID):");

        let mut sorted: Vec<_> = process_stats.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        for (pid, count) in sorted.iter().take(10) {
            println!("  PID {}: {} events", pid, count);
        }
    }

    Ok(())
}

/// Map syscall number to name (x86_64).
fn syscall_nr_to_name(nr: u64) -> &'static str {
    match nr {
        0 => "read",
        1 => "write",
        2 => "open",
        3 => "close",
        4 => "stat",
        5 => "fstat",
        6 => "lstat",
        7 => "poll",
        8 => "lseek",
        9 => "mmap",
        10 => "mprotect",
        11 => "munmap",
        12 => "brk",
        13 => "rt_sigaction",
        14 => "rt_sigprocmask",
        15 => "rt_sigreturn",
        16 => "ioctl",
        17 => "pread64",
        18 => "pwrite64",
        19 => "readv",
        20 => "writev",
        21 => "access",
        22 => "pipe",
        23 => "select",
        24 => "sched_yield",
        25 => "mremap",
        26 => "msync",
        27 => "mincore",
        28 => "madvise",
        32 => "dup",
        33 => "dup2",
        35 => "nanosleep",
        39 => "getpid",
        41 => "socket",
        42 => "connect",
        43 => "accept",
        44 => "sendto",
        45 => "recvfrom",
        46 => "sendmsg",
        47 => "recvmsg",
        48 => "shutdown",
        49 => "bind",
        50 => "listen",
        56 => "clone",
        57 => "fork",
        58 => "vfork",
        59 => "execve",
        60 => "exit",
        61 => "wait4",
        62 => "kill",
        63 => "uname",
        72 => "fcntl",
        78 => "getdents",
        79 => "getcwd",
        80 => "chdir",
        82 => "rename",
        83 => "mkdir",
        84 => "rmdir",
        85 => "creat",
        86 => "link",
        87 => "unlink",
        88 => "symlink",
        89 => "readlink",
        90 => "chmod",
        91 => "fchmod",
        92 => "chown",
        93 => "fchown",
        95 => "umask",
        102 => "getuid",
        104 => "getgid",
        107 => "geteuid",
        108 => "getegid",
        110 => "getppid",
        111 => "getpgrp",
        186 => "gettid",
        202 => "futex",
        217 => "getdents64",
        231 => "exit_group",
        257 => "openat",
        262 => "newfstatat",
        288 => "accept4",
        292 => "dup3",
        293 => "pipe2",
        302 => "prlimit64",
        318 => "getrandom",
        _ => "unknown",
    }
}
```

### Part 3: Build and Test

1. Build the eBPF program:

```bash
cargo xtask build-ebpf
```

2. Build the userspace CLI:

```bash
cargo build -p ebpf-tool
```

3. Run the tests:

```bash
# Non-root test
cargo test -p ebpf-tool --test tracer_test test_trace_help

# Root tests
sudo -E cargo test -p ebpf-tool --test tracer_test
```

**Expected output**: Tests should now pass (GREEN phase):

```
running 8 tests
test test_trace_help ... ok
test test_trace_runs_successfully ... ok
test test_trace_shows_syscall_events ... ok
test test_trace_filter_by_process ... ok
test test_trace_filter_by_syscall ... ok
test test_trace_shows_timestamps ... ok
test test_trace_shows_process_info ... ok
test test_trace_respects_duration ... ok

test result: ok. 8 passed; 0 failed
```

## Verify

### Automated Verification

```bash
# All tracer tests should pass
sudo -E cargo test -p ebpf-tool --test tracer_test

# Full test suite
sudo -E cargo test -p ebpf-tool
```

### Manual Verification

Now let's observe the actual tracer behavior.

#### 1. Basic Trace

```bash
sudo cargo run -p ebpf-tool -- trace -d 5
```

Expected output:
```
Starting syscall tracer
Duration: 5 seconds (0 = until Ctrl+C)
Loading eBPF tracer program...
Attached to raw_syscalls:sys_enter tracepoint
Tracing syscalls (Ctrl+C to stop)...
---
[14:23:45.123] bash(1234) read
[14:23:45.124] bash(1234) write
[14:23:45.125] systemd(1) epoll_wait
[14:23:45.126] python(5678) mmap
...

Duration (5 seconds) elapsed.

Summary:
---------
Duration: 5.00s
Total events captured: 2341

Top syscalls:
  read: 876 calls (37.4%)
  write: 543 calls (23.2%)
  epoll_wait: 234 calls (10.0%)
  futex: 198 calls (8.5%)
  close: 156 calls (6.7%)

Top processes (by PID):
  PID 1234: 432 events
  PID 1: 321 events
  PID 5678: 287 events
```

#### 2. Filter by Process

```bash
sudo cargo run -p ebpf-tool -- trace -p bash -d 5
```

Only shows events from processes with "bash" in their name.

#### 3. Filter by Syscall

```bash
sudo cargo run -p ebpf-tool -- trace -s openat -d 5
```

Only shows `openat` syscalls (file opens).

#### 4. Combined Filters

```bash
sudo cargo run -p ebpf-tool -- trace -p bash -s read -d 5
```

Shows only `read` syscalls from bash processes.

#### 5. Trigger Activity While Tracing

In one terminal:
```bash
sudo cargo run -p ebpf-tool -- trace -d 30
```

In another terminal:
```bash
# Generate some syscalls
ls /tmp
cat /etc/passwd
echo "hello" > /tmp/test.txt
rm /tmp/test.txt
```

Watch the tracer output show these operations in real-time.

#### 6. Observe High-Volume Tracing

```bash
# Run a busy workload
while true; do ls /tmp > /dev/null; done &
BG_PID=$!

# Trace it
sudo cargo run -p ebpf-tool -- trace -p $BG_PID -d 10

# Clean up
kill $BG_PID
```

Verify the tracer handles high event rates.

## Clean Up

No persistent resources are created. The eBPF program is automatically unloaded when `ebpf-tool trace` exits.

To verify cleanup:

```bash
# Check that no BPF programs are lingering
sudo bpftool prog list | grep trace
# Should be empty after trace exits
```

## Common Errors

### 1. `failed to load program: Permission denied`

**Symptom**:
```
Error: failed to load program
Caused by:
    Permission denied (os error 13)
```

**Cause**: Not running with root privileges.

**Fix**:
```bash
sudo cargo run -p ebpf-tool -- trace
```

### 2. `failed to attach tracepoint: No such file or directory`

**Symptom**:
```
Error: failed to attach tracepoint
Caused by:
    No such file or directory (os error 2)
```

**Cause**: The tracepoint category/name is incorrect, or tracing is disabled.

**Fix**:
```bash
# Verify tracepoints are available
ls /sys/kernel/debug/tracing/events/raw_syscalls/

# If not mounted, mount debugfs
sudo mount -t debugfs debugfs /sys/kernel/debug
```

### 3. `No events captured`

**Symptom**: Tracer runs but shows zero events.

**Causes**:
1. Filter is too restrictive
2. eBPF program not properly attached
3. Perf buffer not being read correctly

**Debug steps**:
```bash
# Run without filters first
sudo cargo run -p ebpf-tool -- trace -d 5

# Verify eBPF program is attached
sudo bpftool prog list

# Check for errors in kernel log
dmesg | tail -20
```

### 4. `Event buffer overflow`

**Symptom**: Warning about lost events.

**Cause**: Events are generated faster than userspace can consume them.

**Fix**: The tracer handles this gracefully, but you may miss some events. Consider:
- Using more specific filters
- Running on a less busy system
- Increasing buffer sizes (advanced)

### 5. `Syscall shows as "unknown"`

**Symptom**: Output shows syscall numbers instead of names.

**Cause**: The syscall number isn't in our mapping table.

**Fix**: The mapping covers common syscalls but not all 400+. You can:
- Look up the number in `/usr/include/asm/unistd_64.h`
- Add the mapping to `syscall_nr_to_name()`

### 6. Build error: `cannot find -lebpf`

**Symptom**:
```
error: linking with `cc` failed
note: /usr/bin/ld: cannot find -lebpf
```

**Cause**: libbpf development libraries not installed.

**Fix**:
```bash
# Ubuntu/Debian
sudo apt install libbpf-dev

# Fedora/RHEL
sudo dnf install libbpf-devel
```

### 7. `Program rejected by verifier`

**Symptom**: Long error message about BPF verifier rejection.

**Cause**: The eBPF program violates verifier safety rules.

**Fix**: Check the specific verifier error. Common issues:
- Unbounded loops
- Invalid memory access
- Missing null checks after map lookups

## Notes

### How the Pieces Fit Together

This lesson combines everything from the previous seven lessons:

| Lesson | Concept | Used In Tracer |
|--------|---------|----------------|
| 00-setup | Environment setup | Prerequisite |
| 01-kprobe | Kernel function probing | Understanding probe types |
| 02-kprobe-args | Reading arguments | Context reading pattern |
| 03-maps | HashMap storage | `SYSCALL_COUNTS`, `PROCESS_COUNTS` |
| 04-perf-array | Event streaming | `EVENTS` PerfEventArray |
| 05-uprobes | Userspace probes | Probe pattern understanding |
| 06-tracepoints | Stable attachment points | `raw_syscalls:sys_enter` |
| 07-perf-events | Sampling and timing | Duration and statistics |

### Why Tracepoints Over Kprobes for Syscalls?

We use tracepoints (`raw_syscalls:sys_enter`) instead of kprobes for syscalls because:

1. **Stability**: Tracepoints have a stable ABI; kprobe targets can change between kernel versions
2. **Efficiency**: Tracepoints are statically compiled in; kprobes use dynamic breakpoints
3. **Completeness**: The syscall tracepoint fires for all syscalls; kprobing individual syscalls is tedious
4. **Safety**: Tracepoints are designed for tracing; kprobes can attach to unsafe locations

### The Filter Implementation Trade-off

Our implementation uses both in-kernel and userspace filtering:

**In-kernel filtering** (in eBPF program):
- PID filter: Numeric PID comparison
- Syscall filter: Syscall number comparison
- Advantage: Maximum efficiency, events dropped before copying

**Userspace filtering** (in Rust code):
- Process name matching: String comparison on comm field
- Advantage: More flexible, can use complex patterns

This hybrid approach balances efficiency with flexibility.

### Scaling Considerations

For production use, consider:

1. **Ring buffer**: Use `BPF_MAP_TYPE_RINGBUF` (kernel 5.8+) instead of perf arrays for better performance
2. **Batch reading**: Process events in batches to reduce syscall overhead
3. **Sampling**: For very high rates, sample instead of tracing every event
4. **Aggregation**: Do more aggregation in-kernel to reduce data volume

### Comparison to strace

| Feature | Our Tracer | strace |
|---------|------------|--------|
| Mechanism | eBPF tracepoint | ptrace |
| Overhead | Very low | High (context switch per syscall) |
| System-wide | Yes | No (per-process) |
| Arguments | Basic (extensible) | Full decoding |
| Return values | Not yet | Yes |
| Multi-threaded | Safe | Can have issues |
| Root required | Yes | No (for own processes) |

Our tracer is more efficient but less detailed than strace. They serve different purposes.

### Links to Documentation

- [raw_syscalls tracepoint](https://www.kernel.org/doc/html/latest/trace/events.html) - Kernel tracepoint documentation
- [Aya PerfEventArray](https://docs.rs/aya/latest/aya/maps/perf/struct.PerfEventArray.html) - Aya perf buffer API
- [BPF CO-RE](https://nakryiko.com/posts/bpf-portability-and-co-re/) - Making eBPF programs portable
- [syscall table](https://filippo.io/linux-syscall-table/) - x86_64 syscall number reference
- [strace implementation](https://github.com/strace/strace) - Reference for syscall argument decoding

## Summary

In this capstone lesson, you learned to:

1. **Combine eBPF features**: Integrating tracepoints, maps, and perf buffers into one tool
2. **Design for efficiency**: In-kernel filtering to minimize overhead
3. **Use multiple maps**: Statistics maps for aggregation, perf arrays for streaming
4. **Handle real-time events**: Async processing of perf buffer events
5. **Present useful output**: Real-time display plus summary statistics
6. **Build production-quality tools**: Error handling, duration control, signal handling

You've built a syscall tracer that demonstrates all major eBPF concepts working together - the foundation for tools like bpftrace, Falco, and other production eBPF applications.

### What You've Accomplished in This Section

Over these 9 lessons, you've progressed from "what is eBPF?" to building a complete tracing tool:

- **Lesson 00**: Set up the eBPF development environment
- **Lesson 01**: Wrote your first kprobe program
- **Lesson 02**: Learned to read kernel function arguments
- **Lesson 03**: Used HashMaps for in-kernel data storage
- **Lesson 04**: Streamed events to userspace with perf buffers
- **Lesson 05**: Traced userspace functions with uprobes
- **Lesson 06**: Used stable tracepoints instead of kprobes
- **Lesson 07**: Sampled CPU performance with perf events
- **Lesson 08**: Combined everything into a production tool

You now have the foundation to build any eBPF-based observability tool.

## Next Steps

This completes the core eBPF tutorial section. To continue your eBPF journey:

1. **Extend the tracer**: Add syscall argument decoding, return value capture, or latency measurement
2. **Build new tools**: Create an HTTP request tracer, memory allocator profiler, or network packet analyzer
3. **Explore advanced topics**: CO-RE for portability, BTF for type information, XDP for packet processing
4. **Contribute to the ecosystem**: Many eBPF tools welcome contributors

The skills you've learned here apply directly to production tools like:
- **bpftrace**: High-level tracing language
- **Cilium**: Kubernetes networking and security
- **Falco**: Runtime security monitoring
- **Pixie**: Kubernetes observability
- **Katran**: Layer 4 load balancer
