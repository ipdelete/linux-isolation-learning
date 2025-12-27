# 01 Hello Kprobe

## Goal

Create your first eBPF kprobe program that traces kernel function calls. You will implement the `kprobe` subcommand in `ebpf-tool` that attaches a probe to any kernel function and logs when that function is invoked. This lesson teaches the fundamental structure of eBPF programs using the Aya framework.

## Prereqs

- Completed `docs/04-ebpf/00-ebpf-setup.md` (environment validation)
- `sudo` access (loading eBPF programs requires `CAP_BPF` or `CAP_SYS_ADMIN`)
- Linux kernel 5.8+ with BTF (BPF Type Format) support
- Understanding of the Aya project structure (three crates: userspace, eBPF, common)

## Background: What are Kprobes?

Kprobes (Kernel Probes) are a powerful Linux kernel mechanism for dynamic tracing. They allow you to attach handlers that execute when specific kernel functions are called, without modifying the kernel source code or rebooting.

**How kprobes work:**

1. When you attach a kprobe to a kernel function, the kernel replaces the first instruction of that function with a breakpoint (INT3 on x86)
2. When the CPU hits this breakpoint, your eBPF handler executes
3. The kernel then executes the original instruction and resumes normal execution
4. There is zero overhead when the probe is not attached

**Why kprobes matter for observability:**

- **System call tracing**: Monitor which processes call which syscalls
- **File system monitoring**: Track file opens, reads, writes in real-time
- **Network debugging**: Trace packet processing through the network stack
- **Performance analysis**: Measure function call latencies
- **Security monitoring**: Detect suspicious kernel activity

**Common kernel functions to probe:**

| Function | Description |
|----------|-------------|
| `do_sys_openat2` | Called when files are opened (handles open/openat syscalls) |
| `vfs_read` | Virtual filesystem read operations |
| `vfs_write` | Virtual filesystem write operations |
| `tcp_connect` | Outgoing TCP connection attempts |
| `do_exit` | Process termination |

## Aya Architecture Overview

The `ebpf-tool` project follows Aya's recommended three-crate structure:

```
crates/
  ebpf-tool/           <- Userspace CLI (loads eBPF, displays output)
  ebpf-tool-ebpf/      <- eBPF programs (no_std, compiles to BPF bytecode)
  ebpf-tool-common/    <- Shared types (used by both userspace and eBPF)
```

**Data flow:**

```
                        Linux Kernel
                    +------------------+
                    |  BPF VM          |
                    |  +------------+  |
                    |  | hello_     |  |  <-- Your eBPF program runs here
                    |  | kprobe     |  |
                    |  +-----+------+  |
                    |        |         |
                    |   aya_log_ebpf   |  <-- Sends log messages
                    |        |         |
                    +--------+---------+
                             |
                      perf buffer
                             |
                    +--------+---------+
                    |    Userspace     |
                    |  +------------+  |
                    |  | ebpf-tool  |  |  <-- Your CLI displays messages
                    |  | (Aya)      |  |
                    |  +------------+  |
                    +------------------+
```

**Key concepts:**

1. **eBPF programs are `#![no_std]`**: They run in the kernel, so no standard library
2. **eBPF programs are `#![no_main]`**: No traditional entry point; functions are invoked by events
3. **Communication via maps**: eBPF and userspace share data through BPF maps (perf buffers, hash maps, etc.)
4. **Logging with aya_log**: Messages from eBPF are sent to userspace via a perf buffer

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/kprobe_test.rs`

This lesson covers six tests. The first two do not require root privileges; the remaining four require running as root.

### Test 1: Verify --help Output

Open `crates/ebpf-tool/tests/kprobe_test.rs` and find `test_kprobe_help` (line ~34).

Replace the `todo!()` with:

```rust
#[test]
fn test_kprobe_help() {
    // Verify that `ebpf-tool kprobe --help` shows usage information
    // This test does NOT require root privileges.

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["kprobe", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("FUNCTION"))
        .stdout(predicate::str::contains("duration"));
}
```

### Test 2: Verify Missing Argument Fails

Find `test_kprobe_requires_function_arg` (line ~64).

Replace the `todo!()` with:

```rust
#[test]
fn test_kprobe_requires_function_arg() {
    // Verify that `ebpf-tool kprobe` without a function argument fails
    // This test does NOT require root privileges.

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.arg("kprobe")
        .assert()
        .failure()
        .stderr(predicate::str::contains("FUNCTION")
            .or(predicate::str::contains("required")));
}
```

### Test 3: Verify Kprobe Attachment

Find `test_kprobe_attaches_to_kernel_function` (line ~93).

Replace the `todo!()` with:

```rust
#[test]
fn test_kprobe_attaches_to_kernel_function() {
    // Verify that kprobe successfully attaches to a valid kernel function
    // This test REQUIRES root privileges.

    if !is_root() {
        eprintln!("Skipping test_kprobe_attaches_to_kernel_function: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["kprobe", "do_sys_openat2", "-d", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Attaching")
            .or(predicate::str::contains("attached"))
            .or(predicate::str::contains("kprobe")));
}
```

### Test 4: Verify Event Logging

Find `test_kprobe_shows_events` (line ~127).

Replace the `todo!()` with:

```rust
#[test]
fn test_kprobe_shows_events() {
    // Verify that kprobe logs events when the probed function is called
    // This test REQUIRES root privileges.

    if !is_root() {
        eprintln!("Skipping test_kprobe_shows_events: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["kprobe", "do_sys_openat2", "-d", "2"])
        .assert()
        .success()
        // The kprobe should fire during the 2-second run (system activity)
        .stdout(predicate::str::contains("triggered")
            .or(predicate::str::contains("event"))
            .or(predicate::str::contains("kprobe")));
}
```

### Test 5: Verify Duration Flag

Find `test_kprobe_respects_duration` (line ~164).

Replace the `todo!()` with:

```rust
#[test]
fn test_kprobe_respects_duration() {
    // Verify that the -d/--duration flag controls how long the kprobe runs
    // This test REQUIRES root privileges.

    if !is_root() {
        eprintln!("Skipping test_kprobe_respects_duration: requires root");
        return;
    }

    use std::time::Instant;

    let start = Instant::now();
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["kprobe", "do_sys_openat2", "-d", "1"])
        .assert()
        .success();
    let elapsed = start.elapsed();

    // Should complete within 1-3 seconds (1 sec duration + some overhead)
    assert!(elapsed.as_secs() >= 1, "Completed too quickly: {:?}", elapsed);
    assert!(elapsed.as_secs() <= 4, "Took too long: {:?}", elapsed);
}
```

### Test 6: Verify Invalid Function Handling

Find `test_kprobe_invalid_function` (line ~203).

Replace the `todo!()` with:

```rust
#[test]
fn test_kprobe_invalid_function() {
    // Verify that kprobe fails gracefully with an invalid function name
    // This test REQUIRES root privileges.

    if !is_root() {
        eprintln!("Skipping test_kprobe_invalid_function: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["kprobe", "nonexistent_function_xyz123", "-d", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent_function_xyz123")
            .or(predicate::str::contains("not found"))
            .or(predicate::str::contains("failed"))
            .or(predicate::str::contains("error")));
}
```

### Run Tests (Expect Failure)

```bash
# Run without root (some tests will skip, others will fail with todo!())
cargo test -p ebpf-tool --test kprobe_test

# Run with root to see all tests fail
sudo -E cargo test -p ebpf-tool --test kprobe_test
```

Expected output shows the **RED** phase - tests fail because implementation is missing:

```
running 6 tests
test test_kprobe_help ... FAILED
test test_kprobe_requires_function_arg ... FAILED
test test_kprobe_attaches_to_kernel_function ... FAILED
test test_kprobe_shows_events ... FAILED
test test_kprobe_respects_duration ... FAILED
test test_kprobe_invalid_function ... FAILED

failures:
    test_kprobe_help
    ...
```

## Build (Green)

Now implement the eBPF program and userspace CLI to make tests pass.

### Step 1: Implement the eBPF Program

**File**: `crates/ebpf-tool-ebpf/src/kprobe.rs`

First, uncomment the required import at the top of the file (around line 75):

```rust
use aya_log_ebpf::info;
```

Then find `hello_kprobe` (line ~147) and replace the `todo!()`:

```rust
#[kprobe]
pub fn hello_kprobe(ctx: ProbeContext) -> u32 {
    match try_hello_kprobe(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret as u32,
    }
}
```

Next, find `try_hello_kprobe` (line ~186) and replace the `todo!()`:

```rust
fn try_hello_kprobe(ctx: ProbeContext) -> Result<u32, i64> {
    // Log that the kprobe was triggered
    // This message is sent to userspace via aya_log's perf buffer
    info!(&ctx, "kprobe triggered!");

    // Return 0 to indicate success
    Ok(0)
}
```

### Step 2: Build the eBPF Program

The eBPF program must be compiled to BPF bytecode before the userspace CLI can use it.

The project's `build.rs` script automatically compiles the eBPF programs when you build the userspace tool:

```bash
# Build the userspace tool (automatically compiles eBPF via build.rs)
cargo build -p ebpf-tool
```

This invokes the build script which:
1. Compiles `ebpf-tool-ebpf` to BPF bytecode
2. Places the compiled program in `OUT_DIR`
3. Makes it available for embedding via `include_bytes_aligned!`

### Step 3: Implement the Userspace CLI

**File**: `crates/ebpf-tool/src/main.rs`

Find the `Command::Kprobe` match arm (line ~184) and replace the `todo!()`:

```rust
Command::Kprobe { function, duration } => {
    use aya::programs::KProbe;
    use aya::Ebpf;
    use std::time::Duration;
    use tokio::signal;
    use tokio::time::timeout;

    log::info!("Attaching kprobe to function: {}", function);
    log::info!("Duration: {} seconds (0 = until Ctrl+C)", duration);

    // Step 1: Load the eBPF bytecode
    // The include_bytes_aligned! macro embeds the compiled eBPF object file
    // The build.rs script places the compiled eBPF program in OUT_DIR
    let ebpf_bytes = include_bytes_aligned!(
        concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf")
    );

    let mut bpf = Ebpf::load(ebpf_bytes)
        .context("Failed to load eBPF program")?;

    // Step 2: Initialize aya_log to receive log messages from eBPF
    // This sets up a perf buffer to receive info!/warn!/error! messages
    if let Err(e) = aya_log::BpfLogger::init(&mut bpf) {
        // Non-fatal: logging might not be available on all systems
        log::warn!("Failed to initialize eBPF logger: {}", e);
    }

    // Step 3: Get the kprobe program by name
    // The name "hello_kprobe" matches the function name in kprobe.rs
    let program: &mut KProbe = bpf
        .program_mut("hello_kprobe")
        .context("Failed to find 'hello_kprobe' program")?
        .try_into()
        .context("Program is not a kprobe")?;

    // Step 4: Load the program into the kernel
    program.load()
        .context("Failed to load kprobe into kernel")?;

    // Step 5: Attach to the specified kernel function
    // The second argument (0) is the offset within the function
    program.attach(&function, 0)
        .with_context(|| format!(
            "Failed to attach kprobe to '{}'. Is this a valid kernel function?",
            function
        ))?;

    println!("Attached kprobe to: {}", function);
    println!("Listening for events... (Ctrl+C to stop)");

    // Step 6: Run for the specified duration or until Ctrl+C
    let duration_secs = if duration == 0 {
        // 0 means run until Ctrl+C
        Duration::from_secs(u64::MAX)
    } else {
        Duration::from_secs(duration)
    };

    // Use tokio for async signal handling
    // Note: We're already in an async context from #[tokio::main],
    // so we can use async/await directly without creating a nested runtime
    let ctrl_c = signal::ctrl_c();
    let sleep = tokio::time::sleep(duration_secs);

    tokio::select! {
        _ = ctrl_c => {
            println!("\nReceived Ctrl+C, detaching kprobe...");
        }
        _ = sleep => {
            println!("\nDuration elapsed, detaching kprobe...");
        }
    }

    // Program is automatically detached when `bpf` is dropped
    println!("Kprobe detached. Exiting.");

    Ok(())
}
```

### Step 4: Add Required Dependencies

Ensure `Cargo.toml` for `ebpf-tool` includes:

```toml
[dependencies]
aya = { version = "0.12", features = ["async_tokio"] }
aya-log = "0.2"
tokio = { version = "1", features = ["macros", "rt", "signal", "time"] }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
env_logger = "0.10"
log = "0.4"
nix = { version = "0.27", features = ["user"] }
```

### Step 5: Run Tests (Expect Success)

```bash
# Build eBPF and userspace (build.rs automatically compiles eBPF programs)
cargo build -p ebpf-tool

# Run tests with root
sudo -E cargo test -p ebpf-tool --test kprobe_test
```

Expected output shows the **GREEN** phase:

```
running 6 tests
test test_kprobe_help ... ok
test test_kprobe_requires_function_arg ... ok
test test_kprobe_attaches_to_kernel_function ... ok
test test_kprobe_shows_events ... ok
test test_kprobe_respects_duration ... ok
test test_kprobe_invalid_function ... ok

test result: ok. 6 passed; 0 failed; 0 ignored
```

## Verify

**Automated verification:**

```bash
# Run all ebpf-tool tests
sudo -E cargo test -p ebpf-tool

# Run just kprobe tests
sudo -E cargo test -p ebpf-tool --test kprobe_test
```

**Manual verification:**

1. Run the kprobe command and observe output:

```bash
sudo cargo run -p ebpf-tool -- kprobe do_sys_openat2 -d 5
```

Expected output:

```
Attaching kprobe to function: do_sys_openat2
Duration: 5 seconds (0 = until Ctrl+C)
Attached kprobe to: do_sys_openat2
Listening for events... (Ctrl+C to stop)
kprobe triggered!
kprobe triggered!
kprobe triggered!
... (many more events as files are opened system-wide)

Duration elapsed, detaching kprobe...
Kprobe detached. Exiting.
```

2. Try different kernel functions:

```bash
# Trace file writes
sudo cargo run -p ebpf-tool -- kprobe vfs_write -d 3

# Trace process exits
sudo cargo run -p ebpf-tool -- kprobe do_exit -d 10

# Generate activity in another terminal
echo "test" > /tmp/test.txt  # Triggers vfs_write
ls /tmp                       # Triggers do_sys_openat2
```

3. Use verbose mode for debugging:

```bash
sudo cargo run -p ebpf-tool -- -v kprobe do_sys_openat2 -d 2
```

4. List available kernel functions to probe:

```bash
# View all available kernel functions (many thousands!)
sudo cat /proc/kallsyms | head -50

# Search for specific functions
sudo cat /proc/kallsyms | grep -E "^[0-9a-f]+ [tT] (do_sys|vfs_|tcp_)"
```

## Common Errors

### 1. `Failed to load eBPF program: ... BPF_PROG_LOAD failed`

**Cause**: The eBPF program failed kernel verification. Common reasons:
- The eBPF bytecode was not compiled
- Kernel is too old (requires 5.8+ for full BTF support)
- Missing BTF data

**Fix**:
```bash
# Rebuild eBPF programs (via build.rs)
cargo build -p ebpf-tool

# Check kernel version
uname -r  # Should be 5.8+

# Verify BTF is available
ls -la /sys/kernel/btf/vmlinux
```

### 2. `Failed to attach kprobe to 'xyz': Function not found`

**Cause**: The specified function does not exist in the kernel, or its symbol is not exported.

**Fix**:
```bash
# Check if the function exists
sudo cat /proc/kallsyms | grep -w "your_function_name"

# Use a known-good function for testing
sudo cargo run -p ebpf-tool -- kprobe do_sys_openat2 -d 1
```

### 3. `Operation not permitted` when loading eBPF

**Cause**: Insufficient privileges to load eBPF programs.

**Fix**:
```bash
# Run with sudo
sudo cargo run -p ebpf-tool -- kprobe do_sys_openat2 -d 1

# Or grant CAP_BPF capability (less common)
sudo setcap cap_bpf+ep ./target/debug/ebpf-tool
```

### 4. `Failed to find 'hello_kprobe' program`

**Cause**: The eBPF program name in userspace does not match the function name in the eBPF code.

**Fix**: Ensure the function name in `kprobe.rs` matches exactly:
```rust
#[kprobe]
pub fn hello_kprobe(ctx: ProbeContext) -> u32 {  // <-- "hello_kprobe"
    ...
}
```

And in `main.rs`:
```rust
bpf.program_mut("hello_kprobe")  // <-- Must match exactly
```

### 5. `error[E0463]: can't find crate for 'core'` when building eBPF

**Cause**: The Rust nightly toolchain or `rust-src` component is missing.

**Fix**:
```bash
# Install nightly and required components
rustup install nightly
rustup component add rust-src --toolchain nightly
```

### 6. No output even though kprobe is attached

**Cause**: `aya_log` initialization failed silently, or the log level filters out messages.

**Fix**:
```bash
# Enable debug logging
RUST_LOG=debug sudo cargo run -p ebpf-tool -- kprobe do_sys_openat2 -d 2

# Check if aya_log initialization succeeded (look for warnings)
```

## Notes

**eBPF verifier**: Before any eBPF program runs, the kernel's BPF verifier analyzes it to ensure safety:
- Bounded loops (no infinite loops)
- Valid memory accesses
- Proper stack usage
- No division by zero possible

If verification fails, the program will not load and you will see detailed error messages.

**Kprobe overhead**: While kprobes are designed to be lightweight, attaching to frequently-called functions (like `do_sys_openat2`) can add measurable overhead. For production use, consider:
- Filtering by process ID or other criteria
- Using tracepoints instead (more stable, lower overhead)
- Sampling instead of tracing every call

**Function stability**: Kernel function names and signatures can change between kernel versions. For stable tracing:
- Prefer tracepoints (stable ABI) over kprobes
- Use BTF (BPF Type Format) for CO-RE (Compile Once, Run Everywhere)
- Test on your target kernel version

**Finding kernel functions**:
```bash
# All symbols
sudo cat /proc/kallsyms

# Just functions (t/T means text/code section)
sudo cat /proc/kallsyms | grep -E "^[0-9a-f]+ [tT]"

# Syscall entry points
sudo cat /proc/kallsyms | grep -E "^[0-9a-f]+ [tT] .*sys_"
```

**Understanding aya_log**:
- Messages from `info!(&ctx, "...")` are sent via a perf buffer
- Userspace must poll this buffer (handled by `BpfLogger::init`)
- If the buffer fills up, messages are dropped
- For high-volume tracing, consider using maps instead of logging

**Manual pages and references**:
- Aya Book: https://aya-rs.dev/book/
- Linux kprobes: `man 7 kprobes` or https://www.kernel.org/doc/html/latest/trace/kprobes.html
- BPF documentation: https://docs.kernel.org/bpf/
- `/sys/kernel/debug/tracing/README` (on-system tracing documentation)

## Next

`02-reading-data.md` - Extract process information and function arguments from kprobe context
