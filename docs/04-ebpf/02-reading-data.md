# Lesson 02: Reading Kernel Data

## Goal

Learn to extract meaningful data from kprobe context and send structured events from eBPF to userspace. You will extend the basic kprobe from Lesson 01 to capture process information (PID, process name) and timestamps, then display them in the userspace CLI.

## Prereqs

- Completed [Lesson 01: Hello Kprobe](./01-hello-kprobe.md)
- Understanding of kprobe basics and the Aya framework
- `sudo` access for loading eBPF programs
- Rust toolchain with `bpf-linker` installed (build.rs compiles eBPF)

## Concepts

### Why Extract Kernel Data?

A kprobe that only logs "kprobe triggered!" is useful for debugging but not for real observability. Production tracing tools need to answer questions like:

- **Who** triggered this function? (PID, process name)
- **When** did it happen? (high-resolution timestamp)
- **What** arguments were passed? (syscall parameters, file paths)

eBPF provides BPF helpers to access this information safely from kernel context.

### Key BPF Helpers

| Helper | Purpose | Return Value |
|--------|---------|--------------|
| `bpf_get_current_pid_tgid()` | Get process/thread ID | `(PID << 32) \| TID` as `u64` |
| `bpf_get_current_comm()` | Get process name | Fills `[u8; 16]` buffer |
| `bpf_ktime_get_ns()` | Monotonic timestamp | Nanoseconds since boot as `u64` |
| `ctx.arg::<T>(n)` | Read nth function argument | The argument value |

### Understanding PID vs TID

Linux uses "tgid" (thread group ID) for what userspace calls PID, and "pid" for thread ID:

```
bpf_get_current_pid_tgid() returns:
  High 32 bits: TGID (process ID - what ps shows)
  Low 32 bits:  PID  (thread ID - unique per thread)
```

For single-threaded processes, PID == TID.

### Event Structures

To send structured data from eBPF to userspace, you need a shared data structure. The `SyscallEvent` struct in `ebpf-tool-common` provides this:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallEvent {
    pub pid: u32,           // Process ID
    pub tid: u32,           // Thread ID
    pub syscall_nr: u64,    // Syscall number (optional)
    pub timestamp_ns: u64,  // Nanoseconds since boot
    pub comm: [u8; 16],     // Process name (null-padded)
}
```

The `#[repr(C)]` attribute ensures consistent memory layout between eBPF and userspace.

### Perf Event Arrays

Perf event arrays are the standard mechanism for streaming events from eBPF to userspace:

1. **eBPF side**: Call `EVENTS.output(&ctx, &event, 0)` to send an event
2. **Userspace side**: Poll the perf buffer and read events asynchronously

This provides efficient, zero-copy communication with backpressure handling.

---

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/kprobe_test.rs`

The tests for Lesson 02 are already defined but marked with `#[ignore]`. You need to enable them and implement the test logic.

### Step 1: Enable the Tests

Open the test file and find these two tests:

```rust
#[test]
#[ignore] // Enable after completing Lesson 02
fn test_kprobe_reads_process_info() {
    // ...
    todo!("Implement test verifying process info is read from kprobe context")
}

#[test]
#[ignore] // Enable after completing Lesson 02
fn test_kprobe_reads_function_arguments() {
    // ...
    todo!("Implement test verifying function arguments can be read")
}
```

Remove the `#[ignore]` attribute from both tests.

### Step 2: Implement test_kprobe_reads_process_info

This test verifies that the kprobe output includes process information (PID and/or process name):

```rust
#[test]
fn test_kprobe_reads_process_info() {
    if !is_root() {
        eprintln!("Skipping test_kprobe_reads_process_info: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["kprobe", "do_sys_openat2", "-d", "2"])
        .assert()
        .success()
        // Look for PID or process name in output
        .stdout(predicate::str::contains("pid")
            .or(predicate::str::contains("PID"))
            .or(predicate::str::contains("]")));  // Format: [12345] bash
}
```

### Step 3: Implement test_kprobe_reads_function_arguments

This test verifies that function arguments can be accessed:

```rust
#[test]
fn test_kprobe_reads_function_arguments() {
    if !is_root() {
        eprintln!("Skipping test_kprobe_reads_function_arguments: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["kprobe", "do_sys_openat2", "-d", "2"])
        .assert()
        .success()
        // Look for the dfd (directory file descriptor) argument in output
        .stdout(predicate::str::contains("dfd="));
}
```

### Step 4: Run Tests (Expect Failure)

```bash
# Run only the Lesson 02 tests
# Note: We removed #[ignore] from these tests, so we run them normally
sudo -E cargo test -p ebpf-tool test_kprobe_reads

# You should see failures like:
# thread 'test_kprobe_reads_process_info' panicked at 'not yet implemented'
```

The tests fail because the implementation still contains `todo!()` stubs. This is the RED phase of TDD.

---

## Build (Green)

Now implement the eBPF program and userspace handler to make the tests pass.

### Part 1: eBPF Program (crates/ebpf-tool-ebpf/src/kprobe.rs)

#### Step 1: Uncomment Required Imports (Lines 70-82)

Find the commented imports near the top of the file and uncomment them:

```rust
use aya_ebpf::{
    macros::kprobe,
    programs::ProbeContext,
    // Uncomment these for Lesson 02:
    helpers::{bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_ktime_get_ns},
};

// Uncomment for sending events to userspace:
use aya_ebpf::{
    macros::map,
    maps::PerfEventArray,
};
use ebpf_tool_common::SyscallEvent;
```

#### Step 2: Uncomment the EVENTS Map (Lines 91-94)

```rust
#[map]
static EVENTS: PerfEventArray<SyscallEvent> = PerfEventArray::new(0);
```

This creates a perf event array that can hold `SyscallEvent` structures.

#### Step 3: Implement syscall_kprobe (Line 262)

Replace the `todo!()` with the actual implementation:

```rust
#[kprobe]
pub fn syscall_kprobe(ctx: ProbeContext) -> u32 {
    match try_syscall_kprobe(ctx) {
        Ok(ret) => ret,
        Err(_) => 0,  // Silently ignore errors in kprobe
    }
}
```

#### Step 4: Implement try_syscall_kprobe (Line 296)

This is where the actual data extraction happens:

```rust
fn try_syscall_kprobe(ctx: ProbeContext) -> Result<u32, i64> {
    // 1. Get PID and TID from combined value
    let pid_tgid = unsafe { bpf_get_current_pid_tgid() };
    let pid = (pid_tgid >> 32) as u32;  // High 32 bits = process ID
    let tid = pid_tgid as u32;          // Low 32 bits = thread ID

    // 2. Get process name (up to 16 characters)
    let mut comm = [0u8; 16];
    unsafe { bpf_get_current_comm(&mut comm) }
        .map_err(|e| e as i64)?;

    // 3. Get high-resolution timestamp
    let timestamp_ns = unsafe { bpf_ktime_get_ns() };

    // 4. Read syscall argument (optional, depends on probe target)
    let syscall_nr = unsafe { try_read_syscall_args(&ctx).unwrap_or(0) };

    // 5. Build the event structure
    let event = SyscallEvent {
        pid,
        tid,
        syscall_nr,
        timestamp_ns,
        comm,
    };

    // 6. Send event to userspace via perf buffer
    EVENTS.output(&ctx, &event, 0);

    Ok(0)
}
```

#### Step 5: Implement try_read_syscall_args (Line 363)

**Note on syscall_nr field**: In this lesson, we're using the `syscall_nr` field
to store a function argument value to demonstrate how to read kprobe arguments.
When probing `do_sys_openat2`, arg(0) is the directory file descriptor (dfd),
not the syscall number. A real tracing tool would use a separate field for arguments.

```rust
unsafe fn try_read_syscall_args(ctx: &ProbeContext) -> Result<u64, i64> {
    // Read the first argument of the probed function
    // For do_sys_openat2, arg(0) is the directory file descriptor (dfd)
    // Common values: -100 (AT_FDCWD), or small positive integers for real fds
    let arg0: u64 = ctx.arg(0).ok_or(-1i64)?;
    Ok(arg0)
}
```

**Note**: The meaning of arguments depends on which function you are probing. For `do_sys_openat2`:
- `arg(0)`: directory fd (AT_FDCWD for current directory)
- `arg(1)`: pointer to filename string
- `arg(2)`: pointer to open_how struct

### Part 2: Userspace CLI (crates/ebpf-tool/src/main.rs)

#### Step 1: Add Required Imports

Add these imports near the top of the file:

```rust
use aya::{
    include_bytes_aligned,
    programs::KProbe,
    maps::perf::AsyncPerfEventArray,
    util::online_cpus,
    Ebpf,
};
use bytes::BytesMut;
use ebpf_tool_common::SyscallEvent;
use tokio::signal;
use tokio::time::{timeout, Duration};
```

#### Step 2: Implement the Kprobe Command Handler (Line 184)

Replace the `todo!()` in the `Command::Kprobe` match arm:

```rust
Command::Kprobe { function, duration } => {
    log::info!("Attaching kprobe to function: {}", function);
    log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);

    // Load the eBPF program bytecode
    // The build.rs script places the compiled eBPF program in OUT_DIR
    let ebpf_bytes = include_bytes_aligned!(
        concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf")
    );

    let mut bpf = Ebpf::load(ebpf_bytes)?;

    // Initialize eBPF logging (for debug messages from eBPF program)
    if let Err(e) = aya_log::BpfLogger::init(&mut bpf) {
        log::warn!("Failed to initialize eBPF logger: {}", e);
    }

    // Get the kprobe program and attach it
    let program: &mut KProbe = bpf.program_mut("syscall_kprobe").unwrap().try_into()?;
    program.load()?;
    program.attach(&function, 0)?;

    log::info!("Kprobe attached to {}. Listening for events...", function);

    // Set up the perf event array for receiving events
    let mut perf_array = AsyncPerfEventArray::try_from(bpf.take_map("EVENTS").unwrap())?;

    // Spawn a task for each CPU to read events
    let cpus = online_cpus()?;
    for cpu_id in cpus {
        let mut buf = perf_array.open(cpu_id, None)?;

        tokio::spawn(async move {
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(1024))
                .collect::<Vec<_>>();

            loop {
                let events = buf.read_events(&mut buffers).await.unwrap();
                for i in 0..events.read {
                    let buf = &buffers[i];
                    // Parse the event from raw bytes
                    let event = unsafe { &*(buf.as_ptr() as *const SyscallEvent) };

                    // Convert comm to string (null-terminated)
                    let comm = std::str::from_utf8(&event.comm)
                        .unwrap_or("???")
                        .trim_end_matches('\0');

                    println!(
                        "[{}] {}: dfd={}, timestamp={}",
                        event.pid, comm, event.syscall_nr, event.timestamp_ns
                    );
                }
            }
        });
    }

    // Run for specified duration or until Ctrl+C
    if duration > 0 {
        let _ = timeout(Duration::from_secs(duration), signal::ctrl_c()).await;
    } else {
        signal::ctrl_c().await?;
    }

    log::info!("Detaching kprobe...");
    Ok(())
}
```

### Step 3: Build Everything

```bash
# Build the userspace tool (build.rs automatically compiles eBPF programs)
cargo build -p ebpf-tool --release
```

### Step 4: Run Tests (Expect Success)

```bash
sudo -E cargo test -p ebpf-tool test_kprobe_reads
```

All tests should now pass. This is the GREEN phase of TDD.

---

## Verify

### Automated Verification

```bash
# Run all kprobe tests (requires root)
sudo -E cargo test -p ebpf-tool --test kprobe_test

# Expected: All tests pass including Lesson 02 tests
```

### Manual Verification

Run the kprobe tool and observe the output:

```bash
# Attach to do_sys_openat2 for 5 seconds
sudo cargo run -p ebpf-tool -- kprobe do_sys_openat2 -d 5
```

Expected output:

```
[INFO] Attaching kprobe to function: do_sys_openat2
[INFO] Duration: 5 seconds (0 = until Ctrl+C)
[INFO] Kprobe attached to do_sys_openat2. Listening for events...
[12345] bash: dfd=18446744073709551516, timestamp=1234567890123
[12346] cat: dfd=18446744073709551516, timestamp=1234567890456
[12347] ls: dfd=3, timestamp=1234567890789
[INFO] Detaching kprobe...

**Note**: The `dfd` value is the directory file descriptor argument from do_sys_openat2:
- `18446744073709551516` is -100 (AT_FDCWD) as u64, meaning "current working directory"
- Small positive values like `3` are actual file descriptors for open directories
- We're storing this in the `syscall_nr` field to demonstrate reading function arguments
```

In a separate terminal, generate some file activity:

```bash
# These commands will trigger do_sys_openat2
cat /etc/passwd
ls /tmp
echo "test" > /tmp/test.txt
```

You should see events appear in the kprobe output with the corresponding process names and PIDs.

### Inspecting Events in Detail

To see more verbose output, enable debug logging:

```bash
RUST_LOG=debug sudo -E cargo run -p ebpf-tool -- kprobe do_sys_openat2 -d 5
```

---

## Common Errors

### 1. "failed to create map" or "ENOMEM"

**Cause**: The kernel ran out of memory for eBPF maps, or you hit map limits.

**Fix**:
- Increase `ulimit -l unlimited` (locked memory limit)
- Or run as root with: `sudo -E cargo run`

### 2. "program not found: syscall_kprobe"

**Cause**: The eBPF program name doesn't match what you're loading.

**Fix**:
- Check the `#[kprobe]` function is named `syscall_kprobe`
- Verify you rebuilt the userspace tool (build.rs compiles eBPF): `cargo build -p ebpf-tool`

### 3. "invalid argument" when attaching kprobe

**Cause**: The kernel function name is incorrect or doesn't exist.

**Fix**:
- Verify the function exists: `sudo cat /proc/kallsyms | grep do_sys_openat2`
- Try alternative names: `__x64_sys_openat` or `ksys_open`

### 4. "bpf_get_current_comm failed"

**Cause**: The BPF helper couldn't read the process name.

**Fix**:
- This is rare; ensure your buffer is exactly 16 bytes
- Check you're not in an interrupt context (kprobes on some functions)

### 5. No events appearing

**Cause**: The probed function isn't being called, or events are being dropped.

**Fix**:
- Verify the function is called: `sudo cat /sys/kernel/debug/tracing/trace_pipe`
- Generate activity: `cat /etc/passwd` triggers `do_sys_openat2`
- Check for perf buffer overflows in logs

### 6. "invalid mem access" from BPF verifier

**Cause**: Unsafe memory access in the eBPF program.

**Fix**:
- All pointer reads must use BPF helper functions
- Ensure bounds checking on array accesses
- The verifier error message indicates which instruction failed

### 7. Events show wrong PID (0 or -1)

**Cause**: Calling `bpf_get_current_pid_tgid()` in wrong context.

**Fix**:
- This helper only works in process context, not interrupt/softirq
- For `do_sys_openat2`, it should always work since it's a syscall handler

---

## Deep Dive: Understanding the Data Flow

```
+------------------+     +-----------------+     +------------------+
| Kernel Function  |     | eBPF Program    |     | Userspace CLI    |
| do_sys_openat2   |     | syscall_kprobe  |     | ebpf-tool kprobe |
+--------+---------+     +--------+--------+     +---------+--------+
         |                        |                        |
         | 1. Function called     |                        |
         +----------------------->|                        |
         |                        |                        |
         |    2. Extract data:    |                        |
         |    - bpf_get_pid_tgid  |                        |
         |    - bpf_get_comm      |                        |
         |    - bpf_ktime_get_ns  |                        |
         |                        |                        |
         |    3. Build SyscallEvent                        |
         |                        |                        |
         |    4. EVENTS.output()  |                        |
         |                        +----------------------->|
         |                        |    5. Perf buffer      |
         |                        |       polling          |
         |                        |                        |
         |                        |    6. Parse & display  |
         |                        |       [PID] name: ...  |
         |                        |                        |
```

### Performance Considerations

- **Per-CPU buffers**: Perf arrays have separate buffers per CPU to avoid locking
- **Zero-copy**: Events are read directly from kernel memory
- **Async reading**: Userspace polls buffers asynchronously using tokio
- **Backpressure**: If buffers fill up, oldest events are dropped

---

## Notes

- The `comm` field is limited to 16 characters (TASK_COMM_LEN in Linux)
- Timestamps are monotonic nanoseconds since boot (not wall clock time)
- Thread IDs and process IDs differ for multi-threaded applications
- Some kernel functions may not be probeable (inline, optimized out, etc.)

### Related Man Pages

- `man 7 bpf` - BPF system call and map types
- `man 2 perf_event_open` - Perf event interface

### Official Documentation

- [Aya Book: Maps](https://aya-rs.dev/book/maps/)
- [Linux BPF Helpers](https://man7.org/linux/man-pages/man7/bpf-helpers.7.html)
- [Linux Kprobes](https://www.kernel.org/doc/html/latest/trace/kprobes.html)

---

## Exercises

1. **Add timestamp formatting**: Convert `timestamp_ns` to human-readable time (seconds.microseconds since boot)

2. **Filter by PID**: Add a `--pid` flag to only show events from a specific process

3. **Try different probe targets**: Attach to `vfs_read` or `vfs_write` and observe file I/O patterns

4. **Add thread ID**: Modify the output to show both PID and TID for multi-threaded processes

---

## Next

[03-maps.md](./03-maps.md) - Learn to use eBPF HashMaps to aggregate and count events, building a syscall statistics tool.
