# 04 Perf Events: Real-Time Kernel-to-Userspace Communication

## Goal

Implement real-time event streaming from eBPF programs to userspace using `PerfEventArray`. You will extend the `ebpf-tool` CLI with a `perf` subcommand that attaches to kernel events and streams them to userspace using the perf ring buffer infrastructure. By the end of this lesson, you will understand how PerfEventArray differs from regular eBPF maps, how per-CPU ring buffers work, and how to handle high-frequency event streams asynchronously.

**Estimated time**: 45-60 minutes

## Prereqs

- Completed `docs/04-ebpf/00-ebpf-setup.md` (eBPF environment validated)
- Completed `docs/04-ebpf/01-hello-kprobe.md` (basic kprobe attachment)
- Completed `docs/04-ebpf/02-reading-data.md` (reading kernel data)
- Completed `docs/04-ebpf/03-maps.md` (eBPF HashMap basics)
- `sudo` access (eBPF operations require `CAP_BPF` or `CAP_SYS_ADMIN`)
- Familiarity with async Rust (tokio basics)

## Background: What is PerfEventArray?

PerfEventArray is a specialized eBPF map type designed for **real-time, high-frequency event streaming** from kernel space to userspace. Unlike regular HashMaps where userspace must poll for data, PerfEventArray uses the Linux perf ring buffer infrastructure for efficient, low-latency event delivery.

### PerfEventArray vs HashMap: When to Use Which

| Aspect | HashMap | PerfEventArray |
|--------|---------|----------------|
| **Data Model** | Key-value storage | Event stream (FIFO) |
| **Access Pattern** | Userspace polls for changes | Kernel pushes, userspace receives |
| **Data Lifetime** | Persists until overwritten | Consumed once, then discarded |
| **Use Case** | Aggregated statistics, counters | Individual events, traces |
| **Latency** | Milliseconds (polling interval) | Microseconds (async notification) |
| **Ordering** | No ordering guarantees | Per-CPU FIFO ordering |
| **Memory** | Fixed size per entry | Variable size events, ring buffer |

**Use HashMap when:**
- You need aggregated data (e.g., syscall counts per process)
- Userspace reads infrequently (seconds to minutes)
- You want to reduce overhead by summarizing in kernel

**Use PerfEventArray when:**
- You need every individual event (e.g., syscall traces)
- Low latency is critical (security monitoring, debugging)
- Events are high-frequency and streaming

### How PerfEventArray Works

```text
     eBPF Program                          Userspace Application
    +--------------+                       +-------------------+
    |  tracepoint  |                       |  AsyncPerfEvent   |
    |  or kprobe   |                       |  ArrayReader      |
    +------+-------+                       +--------+----------+
           |                                        |
           | EVENTS.output(&ctx, &event, 0)         | buf.read_events(&mut buffers)
           |                                        |
           v                                        v
    +----------------------------------------------+
    |            Per-CPU Ring Buffers              |
    | +--------+  +--------+  +--------+  +------+ |
    | | CPU 0  |  | CPU 1  |  | CPU 2  |  | CPU N| |
    | | buffer |  | buffer |  | buffer |  |buffer| |
    | +--------+  +--------+  +--------+  +------+ |
    +----------------------------------------------+
```

**Key characteristics:**

1. **Per-CPU buffers**: Each CPU has its own ring buffer, eliminating lock contention. When an eBPF program calls `output()`, the event goes to the current CPU's buffer.

2. **Ring buffer semantics**: Buffers are circular. If userspace does not read fast enough, old events are overwritten (lost). The kernel tracks lost event counts.

3. **Async I/O**: Userspace uses `epoll` (or tokio's async) to wait for events. No busy polling needed.

4. **Zero-copy (almost)**: Events are copied once from eBPF stack to ring buffer, then read by userspace. Efficient for high throughput.

### Event Loss and Buffer Sizing

When the eBPF program produces events faster than userspace can consume them, events are lost. The `read_events()` call reports how many events were lost since the last read.

**Factors affecting event loss:**
- **Buffer size**: Larger buffers can absorb bursts (set via `open(cpu, Some(page_count))`)
- **Event frequency**: High-frequency events (1000s/second) require faster consumption
- **Userspace processing time**: Complex processing delays reading
- **CPU count**: More CPUs = more parallel buffers to read

**Default buffer size**: One page (4KB on most systems). For high-frequency events, consider 16-64 pages per CPU.

```rust
// Default: 1 page per CPU
let buf = perf_array.open(cpu_id, None)?;

// Custom: 16 pages (64KB) per CPU for high-frequency events
let buf = perf_array.open(cpu_id, Some(16))?;
```

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/perf_test.rs`

For Lesson 04 (event streaming), we focus on three tests. The remaining tests in the file are for Lesson 07 (perf sampling).

### Part 1: Test help text (no root required)

This test verifies the CLI is properly configured.

1. Open `crates/ebpf-tool/tests/perf_test.rs`

2. Find `test_perf_help()` (line 24) and replace the `todo!()`:

```rust
#[test]
fn test_perf_help() {
    // Verify that `ebpf-tool perf --help` shows usage information
    // This test does NOT require root - it only checks help text.

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("frequency"))
        .stdout(predicate::str::contains("duration"))
        .stdout(predicate::str::contains("Hz"));
}
```

### Part 2: Test successful execution (requires root)

This test verifies the command runs and exits cleanly.

3. Find `test_perf_runs_successfully()` (line 69) and replace the `todo!()`:

```rust
#[test]
fn test_perf_runs_successfully() {
    // REQUIRES ROOT: eBPF perf event attachment needs CAP_BPF or CAP_SYS_ADMIN

    if !is_root() {
        eprintln!("Skipping test_perf_runs_successfully: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "-d", "1"])  // Run for 1 second
        .assert()
        .success();
}
```

### Part 3: Test event output (requires root)

This test verifies that events are actually received and displayed.

4. Find `test_perf_shows_samples()` (line 121) and replace the `todo!()`:

```rust
#[test]
fn test_perf_shows_samples() {
    // REQUIRES ROOT: eBPF perf event attachment needs CAP_BPF or CAP_SYS_ADMIN

    if !is_root() {
        eprintln!("Skipping test_perf_shows_samples: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "-d", "2"])  // Run for 2 seconds to collect samples
        .assert()
        .success()
        .stdout(
            predicate::str::contains("event").or(
                predicate::str::contains("sample").or(
                    predicate::str::contains("received").or(
                        predicate::str::contains("PID")
                    )
                )
            )
        );
}
```

5. Run the tests (expect failure because implementation is missing):

```bash
cargo test -p ebpf-tool --test perf_test test_perf_help
```

Expected output:
```
running 1 test
test test_perf_help ... FAILED

failures:

---- test_perf_help stdout ----
Error: ebpf-tool perf --help
```

The test panics because the `perf` subcommand hits `todo!()`.

This is the **RED** phase - your tests are written but the implementation is missing.

## Build (Green)

Implementation spans three files:
1. **eBPF program** (`perf.rs`): Sends events from kernel to userspace
2. **Shared types** (`ebpf-tool-common`): Event struct definition (already exists)
3. **Userspace CLI** (`main.rs`): Receives and displays events

### Step 1: Implement the eBPF helper function

**File**: `crates/ebpf-tool-ebpf/src/perf.rs`
**TODO location**: Line ~298 in `send_event()` function

The `send_event` helper wraps the `PerfEventArray::output()` call. Find the `todo!()` and replace:

```rust
fn send_event<C: EbpfContext>(ctx: &C, event: &SyscallEvent) -> Result<(), i64> {
    // Send the event to userspace via the per-CPU ring buffer
    //
    // The 0 flag means: use the current CPU's buffer (BPF_F_CURRENT_CPU)
    // This is the most common and efficient approach.
    EVENTS.output(ctx, event, 0)
}
```

**Understanding the flags:**
- `0` (or `BPF_F_CURRENT_CPU`): Send to current CPU's buffer (most common)
- `BPF_F_INDEX_MASK`: Send to a specific CPU's buffer (advanced use cases)

### Step 2: Extend an existing probe to use PerfEventArray

For this lesson, we will use the syscall kprobe from Lesson 02 to generate events. The `EVENTS` map is already defined in `perf.rs` and can be shared across modules.

However, for a complete standalone implementation, you can create a dedicated kprobe that sends events through PerfEventArray. Here is the pattern:

**File**: `crates/ebpf-tool-ebpf/src/kprobe.rs`

Uncomment the required imports and map definition at the top of the file:

```rust
use aya_ebpf::{
    macros::{kprobe, map},
    maps::PerfEventArray,
    programs::ProbeContext,
    helpers::{bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_ktime_get_ns},
};
use aya_log_ebpf::info;
use ebpf_tool_common::SyscallEvent;

#[map]
static EVENTS: PerfEventArray<SyscallEvent> = PerfEventArray::new(0);
```

Then implement the `try_syscall_kprobe` helper:

```rust
fn try_syscall_kprobe(ctx: ProbeContext) -> Result<u32, i64> {
    // 1. Get process identifiers
    let pid_tgid = unsafe { bpf_get_current_pid_tgid() };
    let pid = (pid_tgid >> 32) as u32;  // Process ID (tgid)
    let tid = pid_tgid as u32;          // Thread ID (pid in kernel terms)

    // 2. Get process command name
    let mut comm = [0u8; 16];
    unsafe { bpf_get_current_comm(&mut comm) }
        .map_err(|e| e as i64)?;

    // 3. Get timestamp
    let timestamp_ns = unsafe { bpf_ktime_get_ns() };

    // 4. Build the event
    let event = SyscallEvent {
        pid,
        tid,
        syscall_nr: 0,  // Will be populated based on probe target
        timestamp_ns,
        comm,
    };

    // 5. Send to userspace via perf event array
    EVENTS.output(&ctx, &event, 0);

    Ok(0)
}
```

### Step 3: Implement the userspace receiver

**File**: `crates/ebpf-tool/src/main.rs`
**TODO location**: Line ~309 in the `Command::Perf` match arm

Replace the `todo!()` with the async event receiver implementation:

```rust
Command::Perf {
    frequency,
    duration,
} => {
    use aya::maps::perf::AsyncPerfEventArray;
    use aya::util::online_cpus;
    use bytes::BytesMut;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    log::info!("Starting CPU sampling at {} Hz", frequency);
    log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);

    // Load the eBPF program
    // Note: For this lesson, we attach a kprobe that uses PerfEventArray
    // In a real implementation, you would load a specific perf program
    let elf_bytes = include_bytes_aligned!(
        "../../ebpf-tool-ebpf/target/bpfel-unknown-none/release/ebpf-tool-ebpf"
    );

    let mut bpf = aya::Ebpf::load(elf_bytes)
        .context("Failed to load eBPF program")?;

    // Get the perf event array map
    let perf_map = bpf.take_map("EVENTS")
        .ok_or_else(|| anyhow::anyhow!("EVENTS map not found"))?;

    let mut perf_array = AsyncPerfEventArray::try_from(perf_map)
        .context("Failed to create AsyncPerfEventArray")?;

    // Shared counter for received events
    let event_count = Arc::new(AtomicU64::new(0));
    let lost_count = Arc::new(AtomicU64::new(0));

    println!("Starting event stream...");

    // Open a buffer for each online CPU and spawn async readers
    let cpus = online_cpus().context("Failed to get online CPUs")?;
    let mut handles = Vec::new();

    for cpu_id in cpus {
        // Open the per-CPU buffer
        // None = default size (1 page), Some(n) = n pages
        let mut buf = perf_array
            .open(cpu_id, Some(16))  // 16 pages = 64KB buffer
            .context(format!("Failed to open perf buffer for CPU {}", cpu_id))?;

        let event_count = Arc::clone(&event_count);
        let lost_count = Arc::clone(&lost_count);

        // Spawn an async task to read events from this CPU's buffer
        let handle = tokio::spawn(async move {
            // Pre-allocate buffers for batch reading
            // Each buffer can hold one event; we read up to 10 at a time
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(1024))
                .collect::<Vec<_>>();

            loop {
                // Wait for and read events (async, non-blocking)
                match buf.read_events(&mut buffers).await {
                    Ok(events) => {
                        // Track lost events (buffer overflow)
                        if events.lost > 0 {
                            lost_count.fetch_add(events.lost as u64, Ordering::Relaxed);
                            eprintln!(
                                "Warning: {} events lost on CPU {}",
                                events.lost, cpu_id
                            );
                        }

                        // Process received events
                        for i in 0..events.read {
                            let buf = &buffers[i];

                            // Safely cast the buffer to our event type
                            // SAFETY: The eBPF program sends SyscallEvent structs
                            if buf.len() >= std::mem::size_of::<ebpf_tool_common::SyscallEvent>() {
                                let event = unsafe {
                                    (buf.as_ptr() as *const ebpf_tool_common::SyscallEvent)
                                        .read_unaligned()
                                };

                                // Display the event
                                let comm = std::str::from_utf8(&event.comm)
                                    .unwrap_or("<invalid>")
                                    .trim_end_matches('\0');

                                println!(
                                    "[{}] {}: syscall {} (PID: {}, TID: {})",
                                    event.timestamp_ns / 1_000_000,  // ms since boot
                                    comm,
                                    event.syscall_nr,
                                    event.pid,
                                    event.tid
                                );

                                event_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    Err(e) => {
                        // Check if we should stop (channel closed or timeout)
                        if e.to_string().contains("closed") {
                            break;
                        }
                        eprintln!("Error reading events on CPU {}: {}", cpu_id, e);
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Attach a kprobe to generate events
    // For demonstration, we attach to a common syscall entry point
    let program: &mut aya::programs::KProbe = bpf
        .program_mut("syscall_kprobe")
        .ok_or_else(|| anyhow::anyhow!("syscall_kprobe program not found"))?
        .try_into()
        .context("Program is not a kprobe")?;

    program.load().context("Failed to load kprobe")?;
    program
        .attach("do_sys_openat2", 0)
        .context("Failed to attach kprobe to do_sys_openat2")?;

    log::info!("Attached to do_sys_openat2, streaming events...");

    // Run for the specified duration
    if duration > 0 {
        sleep(Duration::from_secs(duration)).await;
    } else {
        // Wait forever (until Ctrl+C)
        tokio::signal::ctrl_c()
            .await
            .context("Failed to wait for Ctrl+C")?;
    }

    // Print summary
    let total_events = event_count.load(Ordering::Relaxed);
    let total_lost = lost_count.load(Ordering::Relaxed);

    println!("\n--- Event Stream Summary ---");
    println!("Received {} events in {} seconds", total_events, duration);
    if total_lost > 0 {
        println!("Lost {} events (buffer overflow)", total_lost);
    }
    println!("----------------------------");

    // Clean up: cancel all reader tasks
    for handle in handles {
        handle.abort();
    }

    Ok(())
}
```

### Step 4: Add required dependencies

Ensure the following dependencies are in `crates/ebpf-tool/Cargo.toml`:

```toml
[dependencies]
aya = { version = "0.12", features = ["async_tokio"] }
aya-log = "0.2"
bytes = "1"
tokio = { version = "1", features = ["full", "signal"] }
ebpf-tool-common = { path = "../ebpf-tool-common" }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
env_logger = "0.10"
log = "0.4"
```

### Step 5: Build and verify

```bash
# Build the eBPF program first
cd crates/ebpf-tool-ebpf
cargo xtask build-ebpf --release

# Build the userspace tool
cd ../..
cargo build -p ebpf-tool

# Run the tests
sudo -E cargo test -p ebpf-tool --test perf_test
```

Expected output:
```
running 6 tests
test test_perf_help ... ok
test test_perf_default_frequency ... ok
test test_perf_runs_successfully ... ok
test test_perf_shows_samples ... ok
test test_perf_custom_frequency ... ignored
test test_perf_respects_duration ... ignored

test result: ok. 4 passed; 0 failed; 2 ignored
```

This is the **GREEN** phase!

## Verify

**Automated verification**:

```bash
# Run all perf tests (requires sudo)
sudo -E cargo test -p ebpf-tool --test perf_test

# Run specific tests
sudo -E cargo test -p ebpf-tool --test perf_test test_perf_help
sudo -E cargo test -p ebpf-tool --test perf_test test_perf_runs_successfully
```

**Manual verification** (observe actual event streaming):

1. Run the perf subcommand:

```bash
sudo cargo run -p ebpf-tool -- perf -d 5
```

Expected output:
```
Starting event stream...
[1234567] bash: syscall 257 (PID: 12345, TID: 12345)
[1234568] cat: syscall 257 (PID: 12346, TID: 12346)
[1234570] ls: syscall 257 (PID: 12347, TID: 12347)
...

--- Event Stream Summary ---
Received 42 events in 5 seconds
----------------------------
```

2. Generate activity to trigger more events:

In another terminal, while the perf command is running:

```bash
# Generate file system activity
ls /etc
cat /etc/passwd
find /tmp -name "*.txt"
```

You should see corresponding events appear in the perf output.

3. Test with high-frequency events:

```bash
# Run a program that generates many syscalls
while true; do ls /dev/null; done &
PID=$!

# Start perf capture
sudo cargo run -p ebpf-tool -- perf -d 5

# Clean up
kill $PID
```

Watch for "events lost" warnings, which indicate the buffer is overflowing.

## Understanding the Event Flow

Let us trace an event from kernel to userspace:

```text
1. Application calls open("/etc/passwd", O_RDONLY)
         |
         v
2. Kernel invokes do_sys_openat2()
         |
         v
3. Kprobe fires, our eBPF handler runs:
   - bpf_get_current_pid_tgid() -> pid, tid
   - bpf_get_current_comm() -> "cat"
   - bpf_ktime_get_ns() -> timestamp
   - EVENTS.output(&ctx, &event, 0)
         |
         v
4. Event copied to CPU 2's ring buffer (if running on CPU 2)
         |
         v
5. Userspace tokio task for CPU 2 wakes up (epoll notification)
         |
         v
6. buf.read_events(&mut buffers) returns the event
         |
         v
7. We print: "[1234567] cat: syscall 257 (PID: 12346, TID: 12346)"
```

## Common Errors

1. **`EVENTS map not found` when loading eBPF program**
   - Cause: The eBPF program does not define an `EVENTS` PerfEventArray map
   - Fix: Ensure `perf.rs` has `#[map] static EVENTS: PerfEventArray<SyscallEvent>` and is compiled

2. **`Failed to open perf buffer for CPU X` with ENOMEM**
   - Cause: Requested buffer size too large, or system memory limits
   - Fix: Reduce page count: `perf_array.open(cpu, Some(4))` instead of 16
   - Check: `ulimit -l` shows locked memory limit; increase with `ulimit -l unlimited`

3. **Events lost (buffer overflow) warnings**
   - Cause: Events produced faster than consumed
   - Fixes:
     - Increase buffer size: `open(cpu, Some(64))`
     - Reduce event frequency (filter in eBPF program)
     - Process events faster (less work per event)
     - Use multiple reader threads per CPU

4. **`read_unaligned()` causes crashes or garbage data**
   - Cause: Mismatched struct definition between eBPF and userspace
   - Fix: Ensure `SyscallEvent` is identical in both `ebpf-tool-common` crates
   - Check: `#[repr(C)]` is required on the struct

5. **No events appearing despite the program running**
   - Cause: Kprobe not attached, or attached to wrong function
   - Fix: Check kernel function exists: `cat /proc/kallsyms | grep do_sys_openat2`
   - Try alternative: `do_sys_open` or `sys_openat` depending on kernel version

6. **`Permission denied` when opening perf buffer**
   - Cause: Not running as root, or perf_event_paranoid too restrictive
   - Fix: Run with `sudo`, or check: `cat /proc/sys/kernel/perf_event_paranoid`
   - Value meanings: -1 = no restrictions, 0-3 = increasing restrictions

7. **Tokio panics or async errors**
   - Cause: Missing tokio runtime or wrong features enabled
   - Fix: Ensure `#[tokio::main]` on main function and `features = ["full"]` in Cargo.toml

## Notes

**Event ordering guarantees:**
- Events from the same CPU are ordered (FIFO within each per-CPU buffer)
- Events from different CPUs have no global ordering
- Use timestamps (`bpf_ktime_get_ns()`) to sort events if needed

**Comparison with RingBuf (BPF ring buffer):**
- `RingBuf` (Linux 5.8+) is a newer alternative to PerfEventArray
- Single shared buffer across all CPUs (simpler API)
- Better for low-frequency events; PerfEventArray better for high-frequency
- Aya supports both; check `aya::maps::RingBuf`

**Sizing the buffers:**
- Default: 1 page (4KB) per CPU
- Rule of thumb: `event_size * events_per_second / 1000 * CPUs` bytes
- Example: 48-byte events at 10,000/sec on 4 CPUs = ~2MB total
- Pages per CPU: `2MB / 4 / 4KB = ~125 pages`

**Async vs sync reading:**
- `AsyncPerfEventArray` uses tokio (recommended for most use cases)
- `PerfEventArray` (sync) requires manual epoll management
- Async is easier but adds tokio dependency

**BPF_F_CURRENT_CPU flag:**
- The `0` in `EVENTS.output(&ctx, &event, 0)` means `BPF_F_CURRENT_CPU`
- This sends to the current CPU's buffer (most efficient)
- Rarely need to send to a specific CPU's buffer

**Debugging tips:**
- Add `log::debug!` statements in the userspace reader
- Check `bpftool prog list` to see loaded programs
- Check `bpftool map list` to see the EVENTS map
- Use `bpftool map dump name EVENTS` (though perf arrays are not dumpable)

**Manual pages and references:**
- `man 2 perf_event_open` - Low-level perf event interface
- [Aya Book: PerfEventArray](https://aya-rs.dev/book/maps/perf-event-array/)
- [BPF ring buffer vs perf buffer](https://nakryiko.com/posts/bpf-ringbuf/)
- Kernel source: `kernel/bpf/arraymap.c` (perf array implementation)

## Next

`05-uprobes.md` - Attach probes to userspace function calls using uprobes
