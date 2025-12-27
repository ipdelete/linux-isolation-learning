# 07 Perf Sampling

## Goal

Attach an eBPF program to hardware performance counters for CPU profiling. You'll implement the `perf` subcommand in `ebpf-tool` that samples CPU execution at a configurable frequency (default 99 Hz), captures where the CPU is spending time, and reports aggregated statistics - the foundation for building flame graphs and identifying performance bottlenecks.

## Prereqs

- Completed `docs/04-ebpf/04-perf-events.md` (PerfEventArray basics)
- Completed `docs/04-ebpf/01-hello-kprobe.md` (eBPF program loading)
- `sudo` access (perf events require `CAP_BPF` or `CAP_SYS_ADMIN`)
- Understanding of CPU scheduling and sampling concepts

## Background: What are Perf Events?

Linux's **perf events** subsystem provides unified access to performance monitoring facilities. Unlike kprobes (which fire on specific kernel functions), perf events fire based on hardware or software counters - letting you sample what the CPU is doing at regular intervals.

### Two Worlds of Perf Events

eBPF interacts with perf events in two distinct ways:

1. **PerfEventArray** (Lesson 04): A map type for streaming structured events from eBPF to userspace. The "perf" in the name refers to using the perf ring buffer infrastructure, not CPU sampling.

2. **Perf Sampling** (This Lesson): Attaching eBPF programs directly to CPU performance counters. When the counter overflows (e.g., every N CPU cycles), your eBPF program runs and can capture what the CPU was doing.

```text
Lesson 04: PerfEventArray (User-Kernel Communication)
+-------------+                      +-------------+
| eBPF probe  | --EVENTS.output()--> | Userspace   |
| (kprobe,    |    perf ring buffer  | reader      |
|  uprobe)    |                      |             |
+-------------+                      +-------------+

Lesson 07: Perf Sampling (CPU Profiling)
+-------------+    timer/cycle    +-------------+      +-------------+
| CPU running | ----overflow----> | eBPF perf   | ---> | Aggregation |
| application |   triggers        | event prog  |      | Flame graph |
+-------------+                   +-------------+      +-------------+
```

### Sampling vs Counting

Performance counters can operate in two modes:

| Mode | Description | Use Case |
|------|-------------|----------|
| **Counting** | Track total events (e.g., 1,234,567 cycles) | Precise benchmarking |
| **Sampling** | Record snapshots at intervals | Profiling, flame graphs |

For CPU profiling, we use **sampling mode**. Every N events (or at N Hz frequency), the kernel interrupts the CPU and gives us a snapshot of where it was executing.

### Common Perf Event Types

| Type | Constant | What It Measures |
|------|----------|------------------|
| Software | `PERF_COUNT_SW_CPU_CLOCK` | CPU time (based on kernel scheduler) |
| Software | `PERF_COUNT_SW_TASK_CLOCK` | CPU time for specific task |
| Hardware | `PERF_COUNT_HW_CPU_CYCLES` | Actual CPU cycles (requires PMU) |
| Hardware | `PERF_COUNT_HW_CACHE_MISSES` | L1/L2/LLC cache misses |
| Hardware | `PERF_COUNT_HW_BRANCH_MISSES` | Branch mispredictions |

We'll use `PERF_COUNT_SW_CPU_CLOCK` because it works in all environments (including VMs and containers where hardware PMUs may not be available).

### Why 99 Hz Instead of 100 Hz?

Most Linux systems run a timer interrupt at 100 Hz (or 250 Hz on desktop kernels). If we sample at exactly 100 Hz, we risk "lockstep" - our samples would consistently land on timer interrupt handling code, biasing our results.

Using 99 Hz (or 97 Hz, or any non-divisor frequency) ensures samples drift across different points in the timer cycle, giving a more accurate representation of where CPU time is actually spent.

### Per-CPU Attachment

Perf events are fundamentally per-CPU. To profile the entire system, you must:

1. Enumerate all online CPUs (`/sys/devices/system/cpu/online`)
2. Create a perf event file descriptor for each CPU
3. Attach your eBPF program to each file descriptor

```text
CPU 0 ──> perf_event_open() ──> fd0 ──┐
CPU 1 ──> perf_event_open() ──> fd1 ──┼──> eBPF program
CPU 2 ──> perf_event_open() ──> fd2 ──┤
CPU 3 ──> perf_event_open() ──> fd3 ──┘
```

This is why you'll see profilers report "CPU 0: 248 samples, CPU 1: 247 samples" - each CPU is sampled independently.

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/perf_test.rs`

The test file contains seven tests that progressively verify the `perf` subcommand. Some tests work without root (checking CLI behavior), while others require root to actually attach eBPF programs.

### Step 1: Implement Help Text Test

Open the test file and find `test_perf_help`:

```bash
# Open the test file
# File: crates/ebpf-tool/tests/perf_test.rs
# Line: ~24
```

Replace the `todo!()` with:

```rust
#[test]
fn test_perf_help() {
    // Verify that perf --help shows usage information
    // This test does NOT require root

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("frequency"))
        .stdout(predicate::str::contains("duration"))
        .stdout(predicate::str::contains("Hz").or(predicate::str::contains("sampling")));
}
```

### Step 2: Implement Default Frequency Test

Find `test_perf_default_frequency` (~line 47):

```rust
#[test]
fn test_perf_default_frequency() {
    // Verify the default sampling frequency is 99 Hz
    // This test does NOT require root

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("99"));
}
```

### Step 3: Implement Basic Run Test

Find `test_perf_runs_successfully` (~line 69):

```rust
#[test]
fn test_perf_runs_successfully() {
    // Verify perf subcommand runs and exits cleanly
    // REQUIRES ROOT

    if !is_root() {
        eprintln!("Skipping test_perf_runs_successfully: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "-d", "1"])
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success();
}
```

### Step 4: Implement Custom Frequency Test

Find `test_perf_custom_frequency` (~line 94):

```rust
#[test]
fn test_perf_custom_frequency() {
    // Verify custom sampling frequency is accepted
    // REQUIRES ROOT

    if !is_root() {
        eprintln!("Skipping test_perf_custom_frequency: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "-f", "49", "-d", "1"])
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success();
}
```

### Step 5: Implement Sample Output Test

Find `test_perf_shows_samples` (~line 121):

```rust
#[test]
fn test_perf_shows_samples() {
    // Verify perf output includes sample data
    // REQUIRES ROOT

    if !is_root() {
        eprintln!("Skipping test_perf_shows_samples: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "-d", "2"])
        .timeout(std::time::Duration::from_secs(15))
        .assert()
        .success()
        .stdout(
            predicate::str::contains("sample")
                .or(predicate::str::contains("Sample"))
                .or(predicate::str::contains("collected")),
        );
}
```

### Step 6: Implement Duration Test

Find `test_perf_respects_duration` (~line 152):

```rust
#[test]
fn test_perf_respects_duration() {
    // Verify perf respects the duration flag
    // REQUIRES ROOT

    if !is_root() {
        eprintln!("Skipping test_perf_respects_duration: requires root");
        return;
    }

    let start = std::time::Instant::now();
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "-d", "2"])
        .timeout(std::time::Duration::from_secs(15))
        .assert()
        .success();
    let elapsed = start.elapsed();

    // Should run for approximately 2 seconds (allow 1-4 second window)
    assert!(
        elapsed.as_secs() >= 1,
        "Duration too short: {:?}",
        elapsed
    );
    assert!(
        elapsed.as_secs() <= 4,
        "Duration too long: {:?}",
        elapsed
    );
}
```

### Step 7: Implement Multi-CPU Test

Find `test_perf_samples_all_cpus` (~line 181):

```rust
#[test]
fn test_perf_samples_all_cpus() {
    // Verify perf samples from all available CPUs
    // REQUIRES ROOT

    if !is_root() {
        eprintln!("Skipping test_perf_samples_all_cpus: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["perf", "-d", "2"])
        .timeout(std::time::Duration::from_secs(15))
        .assert()
        .success()
        .stdout(predicate::str::contains("CPU").or(predicate::str::contains("cpu")));
}
```

### Run Tests (Expect Failure)

```bash
# Run all perf tests (non-root tests only)
cargo test -p ebpf-tool --test perf_test

# Run with root for full test coverage
sudo -E cargo test -p ebpf-tool --test perf_test
```

Expected output:
```
running 7 tests
test test_perf_help ... FAILED
test test_perf_default_frequency ... FAILED
test test_perf_runs_successfully ... FAILED
...

failures:

---- test_perf_runs_successfully stdout ----
thread 'test_perf_runs_successfully' panicked at 'not yet implemented:
    Implement perf subcommand - write tests first!'
```

This is the **RED** phase - your tests are written but the implementation doesn't exist yet.

## Build (Green)

Now implement the `perf` subcommand to make your tests pass. This requires changes in two places:

1. **eBPF program** (`crates/ebpf-tool-ebpf/src/perf.rs`) - captures samples in kernel
2. **Userspace CLI** (`crates/ebpf-tool/src/main.rs`) - sets up perf events and displays results

### Part A: Define the Event Type

**File**: `crates/ebpf-tool-common/src/lib.rs`

First, add a new event type for perf samples. Find the existing `SyscallEvent` struct and add below it:

```rust
/// Event generated during CPU sampling.
///
/// Used for profiling and flame graph generation. The eBPF perf_event
/// program populates this on each sample and sends it to userspace.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PerfSampleEvent {
    /// Process ID (tgid in kernel terms)
    pub pid: u32,
    /// Thread ID (pid in kernel terms)
    pub tid: u32,
    /// CPU where the sample was taken
    pub cpu: u32,
    /// Padding for alignment
    pub _pad: u32,
    /// Timestamp in nanoseconds (from bpf_ktime_get_ns)
    pub timestamp_ns: u64,
    /// Kernel stack ID (from STACKS map, -1 if unavailable)
    pub kernel_stack_id: i64,
    /// User stack ID (from STACKS map, -1 if unavailable)
    pub user_stack_id: i64,
    /// Process command name (null-padded)
    pub comm: [u8; 16],
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for PerfSampleEvent {}
```

### Part B: Implement the eBPF Program

**File**: `crates/ebpf-tool-ebpf/src/perf.rs`
**Location**: The `perf_sample` function (~line 209)

Replace the `todo!()` with the implementation:

```rust
use aya_ebpf::{
    bindings::bpf_get_smp_processor_id,
    helpers::bpf_ktime_get_ns,
    macros::{map, perf_event},
    maps::{PerfEventArray, StackTraceMap},
    programs::PerfEventContext,
    EbpfContext,
};
use aya_log_ebpf::info;
use ebpf_tool_common::PerfSampleEvent;

/// Stack trace storage for CPU profiling.
#[map]
static STACKS: StackTraceMap = StackTraceMap::with_max_entries(10000, 0);

/// Perf event array for sending samples to userspace.
#[map]
static PERF_SAMPLES: PerfEventArray<PerfSampleEvent> = PerfEventArray::new(0);

#[perf_event]
pub fn perf_sample(ctx: PerfEventContext) -> u32 {
    match try_perf_sample(&ctx) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

fn try_perf_sample(ctx: &PerfEventContext) -> Result<(), i64> {
    // Get CPU ID
    let cpu = unsafe { bpf_get_smp_processor_id() };

    // Get process information from context
    let pid = ctx.tgid();
    let tid = ctx.pid();

    // Get timestamp
    let timestamp_ns = unsafe { bpf_ktime_get_ns() };

    // Get process name
    let mut comm = [0u8; 16];
    let _ = bpf_get_current_comm(&mut comm);

    // Try to capture kernel stack trace
    let kernel_stack_id = unsafe {
        aya_ebpf::helpers::bpf_get_stackid(
            ctx.as_ptr() as *mut _,
            &STACKS as *const _ as *mut _,
            0, // kernel stack
        )
    };

    // Try to capture user stack trace
    let user_stack_id = unsafe {
        aya_ebpf::helpers::bpf_get_stackid(
            ctx.as_ptr() as *mut _,
            &STACKS as *const _ as *mut _,
            aya_ebpf::bindings::BPF_F_USER_STACK as u64,
        )
    };

    // Build and send event
    let event = PerfSampleEvent {
        pid,
        tid,
        cpu,
        _pad: 0,
        timestamp_ns,
        kernel_stack_id,
        user_stack_id,
        comm,
    };

    PERF_SAMPLES.output(ctx, &event, 0)?;

    info!(ctx, "perf sample: pid={} cpu={}", pid, cpu);
    Ok(())
}

fn bpf_get_current_comm(comm: &mut [u8; 16]) -> Result<(), i64> {
    let ret = unsafe {
        aya_ebpf::helpers::bpf_get_current_comm(
            comm.as_mut_ptr() as *mut _,
            16,
        )
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}
```

### Part C: Implement the Userspace CLI

**File**: `crates/ebpf-tool/src/main.rs`
**Location**: `Command::Perf { frequency, duration }` match arm (~line 303)

Replace the `todo!()` with:

```rust
Command::Perf { frequency, duration } => {
    use aya::programs::PerfEvent;
    use aya::maps::AsyncPerfEventArray;
    use aya::util::online_cpus;
    use bytes::BytesMut;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::sync::Mutex;

    println!("Starting CPU sampling at {} Hz", frequency);
    println!("Duration: {} seconds", duration);

    // Load the eBPF bytecode
    // The build.rs script places the compiled eBPF program in OUT_DIR
    let ebpf_bytes = include_bytes_aligned!(
        concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf")
    );

    let mut bpf = aya::Ebpf::load(ebpf_bytes)
        .context("Failed to load eBPF program")?;

    // Initialize aya-log for eBPF logging (optional)
    if let Err(e) = aya_log::BpfLogger::init(&mut bpf) {
        log::warn!("Failed to init eBPF logger: {}", e);
    }

    // Get the perf event program
    let program: &mut PerfEvent = bpf
        .program_mut("perf_sample")
        .context("perf_sample program not found")?
        .try_into()
        .context("Program is not a perf event")?;

    program.load().context("Failed to load perf_sample program")?;

    // Shared sample counter
    let sample_count = Arc::new(AtomicU64::new(0));
    let cpu_counts: Arc<Mutex<HashMap<u32, u64>>> = Arc::new(Mutex::new(HashMap::new()));

    // Attach to perf events on each CPU
    let cpus = online_cpus().context("Failed to get online CPUs")?;
    println!("Attaching to {} CPUs...", cpus.len());

    for cpu in &cpus {
        // Create perf event using Aya's helper
        let link = program
            .attach(
                aya::programs::perf_event::PerfTypeId::Software,
                aya::programs::perf_event::perf_sw_ids::PERF_COUNT_SW_CPU_CLOCK as u64,
                aya::programs::perf_event::PerfEventScope::AllProcessesOneCpu { cpu: *cpu },
                aya::programs::perf_event::SamplePolicy::Frequency(frequency),
                true, // inherit
            )
            .with_context(|| format!("Failed to attach perf event to CPU {}", cpu))?;

        // Keep the link alive
        std::mem::forget(link);
    }

    println!("Sampling... (press Ctrl+C to stop early)\n");

    // Set up event reader
    let mut perf_array = AsyncPerfEventArray::try_from(
        bpf.take_map("PERF_SAMPLES").context("PERF_SAMPLES map not found")?
    )?;

    // Spawn readers for each CPU
    for cpu in &cpus {
        let mut buf = perf_array.open(*cpu, None)?;
        let sample_count = sample_count.clone();
        let cpu_counts = cpu_counts.clone();
        let cpu_id = *cpu;

        tokio::spawn(async move {
            let mut buffers = vec![BytesMut::with_capacity(1024); 10];

            loop {
                match buf.read_events(&mut buffers).await {
                    Ok(events) => {
                        if events.read > 0 {
                            sample_count.fetch_add(events.read as u64, Ordering::Relaxed);
                            let mut counts = cpu_counts.lock().await;
                            *counts.entry(cpu_id).or_insert(0) += events.read as u64;
                        }
                    }
                    Err(e) => {
                        log::debug!("Error reading perf events on CPU {}: {}", cpu_id, e);
                        break;
                    }
                }
            }
        });
    }

    // Run for the specified duration
    if duration > 0 {
        tokio::time::sleep(std::time::Duration::from_secs(duration)).await;
    } else {
        // Wait for Ctrl+C
        tokio::signal::ctrl_c().await?;
    }

    // Print results
    let total = sample_count.load(Ordering::Relaxed);
    println!("\n=== CPU Sampling Results ===");
    println!("Collected {} samples", total);

    let counts = cpu_counts.lock().await;
    for cpu in &cpus {
        if let Some(count) = counts.get(cpu) {
            println!("CPU {}: {} samples", cpu, count);
        }
    }

    if total > 0 {
        let expected = frequency * duration;
        let ratio = total as f64 / expected as f64;
        println!(
            "\nSample rate: {:.1}% of expected ({} Hz x {} sec = {} expected)",
            ratio * 100.0,
            frequency,
            duration,
            expected
        );
    }

    println!("===========================");

    Ok(())
}
```

### Add Required Dependencies

Ensure these are in your `Cargo.toml` (if not already):

```toml
[dependencies]
bytes = "1"
tokio = { version = "1", features = ["full"] }
```

### Build and Run Tests

```bash
# Build the userspace tool (build.rs automatically compiles eBPF programs)
cd /workspaces/linux-isolation-learning
cargo build -p ebpf-tool

# Run tests
sudo -E cargo test -p ebpf-tool --test perf_test
```

Expected output:
```
running 7 tests
test test_perf_help ... ok
test test_perf_default_frequency ... ok
test test_perf_runs_successfully ... ok
test test_perf_custom_frequency ... ok
test test_perf_shows_samples ... ok
test test_perf_respects_duration ... ok
test test_perf_samples_all_cpus ... ok

test result: ok. 7 passed; 0 failed; 0 ignored
```

This is the **GREEN** phase - your tests now pass!

## Verify

**Automated verification**:
```bash
# Run all tests for ebpf-tool (requires sudo for root tests)
sudo -E cargo test -p ebpf-tool

# Run just the perf tests
sudo -E cargo test -p ebpf-tool --test perf_test
```

**Manual verification** (observe the actual behavior):

1. Run the `perf` subcommand with default settings:
```bash
sudo cargo run -p ebpf-tool -- perf -d 5
```

Expected output:
```
Starting CPU sampling at 99 Hz
Duration: 5 seconds
Attaching to 4 CPUs...
Sampling... (press Ctrl+C to stop early)

=== CPU Sampling Results ===
Collected 495 samples
CPU 0: 124 samples
CPU 1: 123 samples
CPU 2: 125 samples
CPU 3: 123 samples

Sample rate: 100.0% of expected (99 Hz x 5 sec = 495 expected)
===========================
```

2. Try different frequencies:
```bash
# Lower frequency (less overhead, fewer samples)
sudo cargo run -p ebpf-tool -- perf -f 49 -d 3

# Higher frequency (more samples, higher overhead)
sudo cargo run -p ebpf-tool -- perf -f 199 -d 2
```

3. Generate CPU load while sampling:
```bash
# In one terminal, run the sampler
sudo cargo run -p ebpf-tool -- perf -f 99 -d 10

# In another terminal, generate load
stress-ng --cpu 2 --timeout 10s
# Or: dd if=/dev/zero of=/dev/null bs=1M count=10000
```

You should see samples distributed across the CPUs doing work.

4. Compare with the native `perf` tool:
```bash
# Record with Linux perf
sudo perf record -F 99 -a -g -- sleep 5

# Compare sample counts
sudo perf report --stdio | head -20
```

## Understanding the Output

### Sample Distribution

On an idle system, samples will be concentrated in:
- `do_idle` - the kernel idle loop
- `cpuidle_enter` - power-saving idle states
- `schedule` - context switching

When you add load, you'll see samples from:
- Your application code
- System call handlers
- Driver code (I/O operations)

### Sample Rate Analysis

The "sample rate" percentage tells you about system behavior:

| Rate | Meaning |
|------|---------|
| ~100% | Samples collected as expected |
| <90% | Some samples may be lost (high load) |
| >100% | Normal variation, timing differences |
| 0% | No events (check permissions, eBPF errors) |

### Stack Traces (Advanced)

The implementation captures stack IDs. To resolve them into function names:

1. Read the `STACKS` map to get frame addresses
2. Use `/proc/kallsyms` for kernel symbols
3. Use the ELF symbol table for userspace symbols

This is what tools like `perf` and flame graph generators do - but implementing full symbol resolution is beyond this lesson's scope.

## Clean Up

Perf events are automatically cleaned up when the program exits. No manual cleanup is required.

If the program terminates abnormally:
```bash
# Check for orphaned eBPF programs
sudo bpftool prog list | grep perf

# Remove if necessary (use program ID from list)
sudo bpftool prog delete id <ID>
```

## Common Errors

1. **`Operation not permitted (os error 1)` or `EPERM`**
   - Cause: Perf events require elevated privileges
   - Fix: Run with `sudo -E cargo run -p ebpf-tool -- perf`
   - For unprivileged perf, see `/proc/sys/kernel/perf_event_paranoid`

2. **`No such file or directory` for eBPF bytecode**
   - Cause: eBPF program not built before userspace
   - Fix: Build eBPF first:
     ```bash
     cd crates/ebpf-tool-ebpf
     cargo build --release --target bpfel-unknown-none
     ```

3. **`Failed to attach perf event to CPU X`**
   - Cause: CPU may be offline or perf subsystem not available
   - Fix: Check CPU status: `cat /sys/devices/system/cpu/online`
   - In containers, ensure perf events are allowed (may need `--privileged`)

4. **`0 samples collected` even with activity**
   - Cause: PerfEventArray reader not receiving events
   - Fix: Check eBPF log output (`RUST_LOG=debug`), verify map names match

5. **Inconsistent sample counts between runs**
   - Cause: Normal behavior - sampling is inherently statistical
   - Fix: Not an error! Run longer durations for more consistent results

6. **`STACKS map not found` or stack ID is always -1**
   - Cause: Stack trace capture may fail in some contexts
   - Fix: Stack traces are optional; the profiler still works without them
   - User stack traces require the binary to have frame pointers or DWARF info

## Notes

**How `perf_event_open()` Works:**

The `perf_event_open()` syscall creates a file descriptor representing a performance counter. Key parameters:

```c
struct perf_event_attr {
    __u32 type;          // PERF_TYPE_HARDWARE or PERF_TYPE_SOFTWARE
    __u64 config;        // Which specific event (e.g., CPU_CYCLES)
    __u64 sample_freq;   // Samples per second (when freq=1)
    // ... many more fields
};

int fd = perf_event_open(&attr,
    -1,    // pid: -1 = all processes
    cpu,   // cpu: specific CPU
    -1,    // group_fd: -1 = new group
    0      // flags
);
```

Aya abstracts this complexity with `PerfEvent::attach()`.

**Sampling Overhead:**

| Frequency | Overhead | Use Case |
|-----------|----------|----------|
| 49 Hz | Very low | Long-running production profiling |
| 99 Hz | Low | Standard profiling |
| 999 Hz | Moderate | Short targeted analysis |
| 9999 Hz | High | Microsecond resolution |

For production use, 49-99 Hz is typically sufficient.

**perf_event_paranoid Settings:**

The kernel limits unprivileged perf access via `/proc/sys/kernel/perf_event_paranoid`:

| Value | Meaning |
|-------|---------|
| -1 | Allow all users (least restrictive) |
| 0 | Allow process counters |
| 1 | Allow kernel profiling with CAP_PERFMON |
| 2 | Disallow kernel profiling (default) |
| 3 | Disallow all perf events |

```bash
# Check current setting
cat /proc/sys/kernel/perf_event_paranoid

# Allow all (temporarily, for development)
sudo sysctl kernel.perf_event_paranoid=-1
```

**Flame Graph Integration:**

To generate flame graphs from perf samples:

1. Collect stack traces with this tool
2. Export in a format like `folded stacks`
3. Use Brendan Gregg's `flamegraph.pl` or similar

Example workflow (conceptual):
```bash
# Collect samples with stack traces
sudo ebpf-tool perf -f 99 -d 30 --output stacks.txt

# Generate flame graph
./flamegraph.pl stacks.txt > profile.svg
```

**Manual pages to review:**
- `man 2 perf_event_open` - Perf event creation syscall
- `man 7 perf_event_open` - Detailed perf event documentation
- `man 1 perf` - Linux profiling with perf

**Kernel version considerations:**
- Perf events: Available since Linux 2.6.31 (2009)
- eBPF perf event programs: Linux 4.9+ (2016)
- `bpf_get_stackid()`: Linux 4.6+ (2016)
- Per-CPU perf buffers work reliably on Linux 5.8+

## Next

`08-combining.md` - Combine kprobes, maps, and perf events to build a comprehensive syscall tracer
