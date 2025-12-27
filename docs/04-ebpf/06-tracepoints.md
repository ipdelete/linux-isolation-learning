# 06 Tracepoints

## Goal

Attach eBPF programs to kernel tracepoints - stable instrumentation points with well-defined ABIs. You will build a `tracepoint` subcommand in `ebpf-tool` that attaches to kernel tracepoints like `sched/sched_switch` or `syscalls/sys_enter_openat`, and observe events as they fire.

## Prereqs

- Completed `docs/04-ebpf/00-ebpf-setup.md` (eBPF environment validation)
- Completed `docs/04-ebpf/01-hello-kprobe.md` (basic eBPF program loading and attachment)
- `sudo` access (attaching to tracepoints requires `CAP_BPF` or `CAP_SYS_ADMIN`)
- Kernel 4.7+ (tracepoint eBPF support)
- Access to debugfs: `/sys/kernel/debug/tracing/events/`

## Background: Kprobes vs Tracepoints

In Lesson 01, you learned about kprobes - dynamic instrumentation that can attach to any kernel function. Tracepoints are a fundamentally different approach:

**Kprobes (Dynamic Instrumentation):**
- Can attach to any kernel function
- Function signatures may change between kernel versions
- Accessing function arguments requires manual offset calculations
- Probed function may be inlined or renamed in different kernel builds
- Zero overhead when not in use (probes are inserted at runtime)

**Tracepoints (Static Instrumentation):**
- Placed intentionally by kernel developers at semantically meaningful points
- Provide a stable ABI - the interface is documented and maintained
- Arguments are well-defined with documented formats
- Less likely to break across kernel upgrades
- Minimal overhead when disabled (just a branch prediction hint)

**When to use each:**

| Use Case | Recommendation |
|----------|----------------|
| Debugging specific kernel code | Kprobes (attach anywhere) |
| Production monitoring | Tracepoints (stable ABI) |
| System call tracing | Tracepoints (syscalls/*) |
| Scheduler analysis | Tracepoints (sched/*) |
| Network debugging | Either (depends on what you need) |
| Unknown kernel function internals | Kprobes (more flexible) |

Think of tracepoints as "officially supported" instrumentation points. The kernel developers have decided that these events are important enough to expose with a stable interface.

## Exploring Available Tracepoints

Before writing code, explore what tracepoints are available on your system:

```bash
# Mount debugfs if not already mounted
sudo mount -t debugfs debugfs /sys/kernel/debug 2>/dev/null || true

# List all tracepoint categories
ls /sys/kernel/debug/tracing/events/
```

Common categories you will see:

| Category | Description |
|----------|-------------|
| `syscalls` | System call entry/exit (most useful for tracing) |
| `sched` | Scheduler events (context switches, forks) |
| `net` | Networking events (packet receive/transmit) |
| `block` | Block I/O events (disk reads/writes) |
| `irq` | Interrupt handling |
| `signal` | Signal delivery |
| `raw_syscalls` | Raw syscall entry/exit (all syscalls, fewer args) |
| `kmem` | Kernel memory allocation |
| `skb` | Socket buffer events |

```bash
# List tracepoints in a specific category
ls /sys/kernel/debug/tracing/events/syscalls/ | head -20

# View a tracepoint's format (argument layout)
cat /sys/kernel/debug/tracing/events/syscalls/sys_enter_openat/format
```

The format file is crucial - it tells you exactly what data is available at each offset:

```
name: sys_enter_openat
ID: 614
format:
    field:unsigned short common_type;       offset:0;  size:2; signed:0;
    field:unsigned char common_flags;       offset:2;  size:1; signed:0;
    field:unsigned char common_preempt_cnt; offset:3;  size:1; signed:0;
    field:int common_pid;                   offset:4;  size:4; signed:1;

    field:int __syscall_nr;                 offset:8;  size:4; signed:1;
    field:int dfd;                          offset:16; size:8; signed:0;
    field:const char * filename;            offset:24; size:8; signed:0;
    field:int flags;                        offset:32; size:8; signed:0;
    field:umode_t mode;                     offset:40; size:8; signed:0;
```

Key observations:
- **Common fields** (offset 0-7): Present in all tracepoints - type, flags, preempt count, PID
- **Tracepoint-specific fields** (offset 8+): Arguments specific to this event
- **Pointers**: Fields like `filename` contain pointers, not the actual data - you must use `bpf_probe_read_user_str()` to read the string

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/tracepoint_test.rs`

The test file contains seven tests organized into two groups:

**Non-root tests** (can run without privileges):
1. `test_tracepoint_help` - Verify `--help` shows usage
2. `test_tracepoint_requires_category_arg` - Missing category fails
3. `test_tracepoint_requires_name_arg` - Missing name fails

**Root-required tests** (require `CAP_BPF`):
4. `test_tracepoint_attaches_successfully` - Attach to `sched/sched_switch`
5. `test_tracepoint_syscalls_openat` - Attach to `syscalls/sys_enter_openat`
6. `test_tracepoint_shows_events` - Events are captured and output
7. `test_tracepoint_invalid_category` - Invalid category produces error

### Step 1: Implement the Help Test

Open `crates/ebpf-tool/tests/tracepoint_test.rs` and find `test_tracepoint_help` (line 32).

Replace the `todo!()` with:

```rust
#[test]
fn test_tracepoint_help() {
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();

    cmd.args(["tracepoint", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("category"))
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("duration"));
}
```

### Step 2: Implement Argument Validation Tests

Find `test_tracepoint_requires_category_arg` (line 51) and replace:

```rust
#[test]
fn test_tracepoint_requires_category_arg() {
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();

    // Running with just "tracepoint" should fail - missing both category and name
    cmd.arg("tracepoint")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required")
            .or(predicate::str::contains("CATEGORY"))
            .or(predicate::str::contains("argument")));
}
```

Find `test_tracepoint_requires_name_arg` (line 67) and replace:

```rust
#[test]
fn test_tracepoint_requires_name_arg() {
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();

    // Running with just category but no name should fail
    cmd.args(["tracepoint", "syscalls"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required")
            .or(predicate::str::contains("NAME"))
            .or(predicate::str::contains("argument")));
}
```

### Step 3: Implement Attachment Test

Find `test_tracepoint_attaches_successfully` (line 87) and replace:

```rust
#[test]
fn test_tracepoint_attaches_successfully() {
    if !is_root() {
        eprintln!("Skipping test_tracepoint_attaches_successfully: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();

    // sched/sched_switch fires on every context switch - very frequent
    cmd.args(["tracepoint", "sched", "sched_switch", "-d", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sched")
            .or(predicate::str::contains("Attaching"))
            .or(predicate::str::contains("tracepoint")));
}
```

### Step 4: Implement Syscall Tracepoint Test

Find `test_tracepoint_syscalls_openat` (line 114) and replace:

```rust
#[test]
fn test_tracepoint_syscalls_openat() {
    if !is_root() {
        eprintln!("Skipping test_tracepoint_syscalls_openat: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();

    // syscalls/sys_enter_openat fires when files are opened
    cmd.args(["tracepoint", "syscalls", "sys_enter_openat", "-d", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("syscalls")
            .or(predicate::str::contains("openat"))
            .or(predicate::str::contains("Attaching")));
}
```

### Step 5: Implement Event Output Test

Find `test_tracepoint_shows_events` (line 141) and replace:

```rust
#[test]
fn test_tracepoint_shows_events() {
    if !is_root() {
        eprintln!("Skipping test_tracepoint_shows_events: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();

    // sched_switch is extremely frequent, should capture many events
    cmd.args(["tracepoint", "sched", "sched_switch", "-d", "2"])
        .assert()
        .success()
        // Look for evidence of captured events (varies by implementation)
        .stdout(predicate::str::contains("tracepoint")
            .or(predicate::str::contains("event"))
            .or(predicate::str::contains("triggered"))
            .or(predicate::str::is_match(r"\d+").unwrap())); // Contains numbers (PIDs, counts)
}
```

### Step 6: Implement Error Handling Test

Find `test_tracepoint_invalid_category` (line 169) and replace:

```rust
#[test]
fn test_tracepoint_invalid_category() {
    if !is_root() {
        eprintln!("Skipping test_tracepoint_invalid_category: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();

    // Use a clearly invalid category/name combination
    cmd.args(["tracepoint", "nonexistent_category_xyz", "fake_tracepoint", "-d", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error")
            .or(predicate::str::contains("Error"))
            .or(predicate::str::contains("failed"))
            .or(predicate::str::contains("not found"))
            .or(predicate::str::contains("invalid")));
}
```

### Step 7: Run the Tests (Expect Failure)

```bash
cargo test -p ebpf-tool --test tracepoint_test
```

Expected output:

```
running 7 tests
test test_tracepoint_help ... FAILED
test test_tracepoint_requires_category_arg ... ok
test test_tracepoint_requires_name_arg ... ok
test test_tracepoint_attaches_successfully ... FAILED (or skipped)
test test_tracepoint_syscalls_openat ... FAILED (or skipped)
test test_tracepoint_shows_events ... FAILED (or skipped)
test test_tracepoint_invalid_category ... FAILED (or skipped)
```

The CLI argument tests may pass because clap handles argument validation. The actual tracepoint tests will fail with `todo!()` panic. This is the **RED** phase.

## Build (Green)

Now implement the tracepoint functionality in both the eBPF program and the userspace CLI.

### Part 1: eBPF Program (Kernel Side)

**File**: `crates/ebpf-tool-ebpf/src/tracepoint.rs`
**TODO location**: Line 118 in `sys_enter_tracepoint` function

The eBPF tracepoint program uses `TracePointContext` instead of `ProbeContext`. Key differences:

| Kprobe | Tracepoint |
|--------|------------|
| `ProbeContext` | `TracePointContext` |
| `ctx.arg::<T>(n)` | `ctx.read_at::<T>(offset)` |
| `#[kprobe]` macro | `#[tracepoint]` macro |
| Dynamic offsets | Fixed offsets from format file |

Open `crates/ebpf-tool-ebpf/src/tracepoint.rs` and find the `sys_enter_tracepoint` function (line 118).

Replace the `todo!()` with:

```rust
#[tracepoint]
pub fn sys_enter_tracepoint(ctx: TracePointContext) -> u32 {
    match try_sys_enter_tracepoint(&ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

fn try_sys_enter_tracepoint(ctx: &TracePointContext) -> Result<u32, i64> {
    // Log that the tracepoint fired
    info!(ctx, "tracepoint triggered!");

    // Read syscall number from offset 8 (see format file)
    // let syscall_nr: i32 = unsafe { ctx.read_at(8)? };

    // Read directory file descriptor from offset 16
    // let dfd: i64 = unsafe { ctx.read_at(16)? };

    // info!(ctx, "openat: dfd={}", dfd);

    Ok(0)
}
```

For the `sched_tracepoint` function (line 186), you can also implement it:

```rust
#[tracepoint]
pub fn sched_tracepoint(ctx: TracePointContext) -> u32 {
    match try_sched_tracepoint(&ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

fn try_sched_tracepoint(ctx: &TracePointContext) -> Result<u32, i64> {
    // Read prev_pid (process being switched away from) at offset 24
    let prev_pid: i32 = unsafe { ctx.read_at(24)? };

    // Read next_pid (process being switched to) at offset 56
    let next_pid: i32 = unsafe { ctx.read_at(56)? };

    info!(ctx, "context switch: {} -> {}", prev_pid, next_pid);

    Ok(0)
}
```

Note the helper function pattern: the main tracepoint function calls a helper that returns `Result`, allowing for clean error handling in eBPF's constrained environment.

### Part 2: Userspace CLI

**File**: `crates/ebpf-tool/src/main.rs`
**TODO location**: Line 272 in the `Command::Tracepoint` match arm

Open `crates/ebpf-tool/src/main.rs` and find the tracepoint match arm (around line 272).

Replace the `todo!()` with:

```rust
Command::Tracepoint {
    category,
    name,
    duration,
} => {
    use aya::programs::TracePoint;
    use aya::Ebpf;
    use std::time::Duration;
    use tokio::signal;
    use tokio::time::sleep;

    log::info!("Attaching to tracepoint: {}/{}", category, name);
    log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);

    // Load the eBPF bytecode
    // The include_bytes_aligned! macro ensures proper 8-byte alignment
    let ebpf_bytes = include_bytes_aligned!(
        "../../ebpf-tool-ebpf/target/bpfel-unknown-none/release/tracepoint"
    );

    let mut bpf = Ebpf::load(ebpf_bytes)
        .context("Failed to load eBPF bytecode")?;

    // Initialize eBPF logging
    if let Err(e) = aya_log::BpfLogger::init(&mut bpf) {
        log::warn!("Failed to initialize eBPF logger: {}", e);
    }

    // Get the tracepoint program
    // The program name must match the function name in the eBPF code
    let program: &mut TracePoint = bpf
        .program_mut("sys_enter_tracepoint")
        .context("Failed to find tracepoint program")?
        .try_into()
        .context("Program is not a tracepoint")?;

    // Load the program into the kernel
    program.load()
        .context("Failed to load tracepoint program")?;

    // Attach to the specified tracepoint
    program.attach(&category, &name)
        .context(format!(
            "Failed to attach to tracepoint {}/{}. \
             Check if it exists: ls /sys/kernel/debug/tracing/events/{}/{}",
            category, name, category, name
        ))?;

    println!("Attached to tracepoint: {}/{}", category, name);
    println!("Listening for events...");

    // Run for specified duration or until Ctrl+C
    if duration > 0 {
        tokio::select! {
            _ = sleep(Duration::from_secs(duration)) => {
                println!("\nDuration elapsed. Detaching...");
            }
            _ = signal::ctrl_c() => {
                println!("\nReceived Ctrl+C. Detaching...");
            }
        }
    } else {
        // Run until Ctrl+C
        signal::ctrl_c().await?;
        println!("\nReceived Ctrl+C. Detaching...");
    }

    println!("Tracepoint detached successfully.");
    Ok(())
}
```

### Part 3: Build the eBPF Program

Before running tests, you must compile the eBPF program:

```bash
# Navigate to the eBPF crate
cd /workspaces/linux-isolation-learning/crates/ebpf-tool-ebpf

# Build for the BPF target
cargo build --target bpfel-unknown-none --release

# Or use cargo-xtask if available
cargo xtask build-ebpf --release
```

### Part 4: Run the Tests (Expect Success)

```bash
# Build the userspace tool
cargo build -p ebpf-tool

# Run non-root tests
cargo test -p ebpf-tool --test tracepoint_test

# Run all tests with root privileges
sudo -E cargo test -p ebpf-tool --test tracepoint_test
```

Expected output:

```
running 7 tests
test test_tracepoint_help ... ok
test test_tracepoint_requires_category_arg ... ok
test test_tracepoint_requires_name_arg ... ok
test test_tracepoint_attaches_successfully ... ok
test test_tracepoint_syscalls_openat ... ok
test test_tracepoint_shows_events ... ok
test test_tracepoint_invalid_category ... ok

test result: ok. 7 passed; 0 failed; 0 ignored
```

This is the **GREEN** phase.

## Verify

**Automated verification:**

```bash
# Run all ebpf-tool tests
sudo -E cargo test -p ebpf-tool
```

**Manual verification:**

1. **Attach to scheduler context switches:**

```bash
sudo cargo run -p ebpf-tool -- tracepoint sched sched_switch -d 5
```

Expected output:

```
Attaching to tracepoint: sched/sched_switch
Duration: 5 seconds (0 = until Ctrl+C)
Attached to tracepoint: sched/sched_switch
Listening for events...
context switch: 1234 -> 5678
context switch: 5678 -> 1234
context switch: 1234 -> 9012
...
Duration elapsed. Detaching...
Tracepoint detached successfully.
```

2. **Attach to file open syscalls:**

```bash
sudo cargo run -p ebpf-tool -- tracepoint syscalls sys_enter_openat -d 5
```

In another terminal, generate some file opens:

```bash
ls /etc
cat /etc/passwd
```

You should see events for each file open.

3. **List available tracepoints and explore:**

```bash
# List syscall tracepoints
ls /sys/kernel/debug/tracing/events/syscalls/

# View the format for a specific tracepoint
cat /sys/kernel/debug/tracing/events/sched/sched_switch/format
```

## Common Errors

1. **`Failed to attach to tracepoint <category>/<name>`**
   - Cause: The tracepoint does not exist on your kernel
   - Fix: Verify with `ls /sys/kernel/debug/tracing/events/<category>/<name>/`
   - Some tracepoints are kernel-config dependent or module-specific

2. **`Permission denied` or `Operation not permitted`**
   - Cause: Missing `CAP_BPF` or `CAP_SYS_ADMIN` capability
   - Fix: Run with `sudo` or grant the capability: `sudo setcap cap_bpf+ep ./ebpf-tool`

3. **`Failed to load eBPF bytecode` with BTF errors**
   - Cause: Kernel BTF not available or incompatible
   - Fix: Check `ls /sys/kernel/btf/vmlinux`. If missing, your kernel may not support BTF-based eBPF programs
   - Alternative: Use older eBPF without CO-RE (requires kernel-specific compilation)

4. **`No such file or directory` for `/sys/kernel/debug/tracing/events/`**
   - Cause: debugfs not mounted
   - Fix: `sudo mount -t debugfs debugfs /sys/kernel/debug`

5. **Events not appearing in output**
   - Cause: eBPF logging not initialized or events too infrequent
   - Fix: Ensure `aya_log::BpfLogger::init()` is called. Use verbose mode: `cargo run -p ebpf-tool -- -v tracepoint ...`
   - Try a high-frequency tracepoint like `sched/sched_switch`

6. **`Invalid argument` when reading tracepoint data**
   - Cause: Incorrect offset in `ctx.read_at()` call
   - Fix: Check the format file for correct offsets. Remember offsets may differ between 32-bit and 64-bit systems

7. **Pointer fields contain garbage instead of strings**
   - Cause: Reading the pointer value instead of the string it points to
   - Fix: Fields like `filename` are pointers. Use `bpf_probe_read_user_str()` to read the actual string content

## Notes

**Tracepoint naming convention:**
- Format: `<category>/<name>`
- Examples: `syscalls/sys_enter_openat`, `sched/sched_switch`, `net/netif_rx`
- Entry tracepoints: `sys_enter_*` (when syscall starts)
- Exit tracepoints: `sys_exit_*` (when syscall returns, includes return value)

**Reading tracepoint arguments:**
- Use `ctx.read_at::<T>(offset)` where offset comes from the format file
- Always wrap in `unsafe` - you are reading kernel memory
- Use the correct type size (check `size:` in format)
- For strings (pointers), read the pointer first, then use helper to read string

**Performance considerations:**
- Tracepoints have minimal overhead when disabled (~1 nop)
- When enabled, overhead depends on your eBPF program complexity
- Avoid heavy computation in hot tracepoints like `sched_switch`
- Use maps to aggregate data instead of logging every event

**Tracepoints vs raw_syscalls:**
- `syscalls/*` provides typed arguments for specific syscalls
- `raw_syscalls/sys_enter` fires for ALL syscalls but only provides syscall number
- Use `syscalls/*` when you need specific argument access
- Use `raw_syscalls/*` for broad syscall counting/filtering

**Documentation references:**
- Kernel tracepoints: `Documentation/trace/tracepoints.rst` in kernel source
- Aya TracePoint docs: https://docs.rs/aya/latest/aya/programs/tracepoint/
- Format files: `/sys/kernel/debug/tracing/events/<category>/<name>/format`

## Advanced Exercises (Optional)

After completing the basic implementation, try these extensions:

1. **Extract and log the filename from sys_enter_openat:**
   - Read the filename pointer at offset 24
   - Use `bpf_probe_read_user_str()` to read the actual path
   - Log the path being opened

2. **Track per-process context switch counts:**
   - Create a HashMap keyed by PID
   - Increment count each time a process is switched in
   - Display top-N processes by context switch count

3. **Measure syscall latency:**
   - Attach to both `sys_enter_*` and `sys_exit_*` for a syscall
   - Store entry timestamp in a HashMap keyed by (pid, tid)
   - Calculate and log duration on exit

4. **Implement the `net/netif_rx` tracepoint:**
   - Monitor incoming network packets
   - Track packets per interface
   - Generate traffic with `ping` or `curl` to test

## Next

`07-perf-sampling.md` - Use perf events to sample CPU activity for performance profiling
