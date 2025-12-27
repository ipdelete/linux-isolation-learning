# 05 Uprobes

## Goal

Attach a uprobe (userspace probe) to a function in a userspace binary and observe function calls as they happen. You will implement the `uprobe` subcommand in `ebpf-tool` that attaches an eBPF program to any exported function in an ELF binary or shared library, logging each time the function is called.

## Prereqs

- Completed prior lessons in `docs/04-ebpf/` (especially `01-hello-kprobe.md` for eBPF basics)
- `sudo` access (loading eBPF programs requires `CAP_BPF` or `CAP_SYS_ADMIN`)
- Basic understanding of ELF binaries and shared libraries
- Familiarity with the concept of function symbols

## Background: What are Uprobes?

Uprobes (userspace probes) allow you to dynamically trace function calls in userspace applications without recompiling or modifying the target binary. They are the userspace counterpart to kprobes, which trace kernel functions.

### Kprobes vs Uprobes

| Aspect | Kprobes | Uprobes |
|--------|---------|---------|
| **Target** | Kernel functions | Userspace functions |
| **Location** | Kernel address space | Process address space |
| **Symbols** | `/proc/kallsyms` | ELF symbol tables |
| **Scope** | System-wide (one probe) | Per-binary (all process instances) |
| **Overhead** | Lower | Higher (user/kernel context switches) |
| **Examples** | `do_sys_openat2`, `tcp_connect` | `malloc`, `SSL_read`, `readline` |

### How Uprobes Work

When you attach a uprobe:

1. **Symbol Resolution**: The kernel finds the function's offset in the ELF binary using the symbol table
2. **Breakpoint Installation**: The kernel replaces the first instruction(s) at that offset with an interrupt instruction
3. **Trap Handling**: When any process executes that binary and reaches the function, it traps into the kernel
4. **eBPF Execution**: Your eBPF program runs with access to function arguments and process context
5. **Instruction Emulation**: The kernel emulates the replaced instruction and returns control to the process

### ELF Symbols Explained

ELF (Executable and Linkable Format) binaries contain symbol tables that map function names to memory offsets. You can view these with:

```bash
# List symbols in a binary
nm /bin/ls | head -20

# For shared libraries, use -D for dynamic symbols
nm -D /lib/x86_64-linux-gnu/libc.so.6 | grep -E "^[0-9a-f]+ T " | head -20

# Find malloc in libc
nm -D /lib/x86_64-linux-gnu/libc.so.6 | grep " T malloc"
```

### Common Uprobe Targets

| Binary | Function | Purpose |
|--------|----------|---------|
| `libc.so.6` | `malloc` | Track memory allocations |
| `libc.so.6` | `free` | Track memory frees |
| `libc.so.6` | `open` / `openat` | Track file opens |
| `libc.so.6` | `write` | Track write operations |
| `libssl.so` | `SSL_read` | Monitor SSL/TLS traffic |
| `libssl.so` | `SSL_write` | Monitor SSL/TLS traffic |
| `/usr/bin/bash` | `readline` | Capture shell commands |

### Finding the libc Path

The libc path varies by distribution and architecture. Find it with:

```bash
# Method 1: Use ldd
ldd /bin/ls | grep libc
# Output: libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x...)

# Method 2: Use ldconfig
ldconfig -p | grep libc.so.6

# Method 3: Direct check (common paths)
ls -la /lib/x86_64-linux-gnu/libc.so.6    # Debian/Ubuntu
ls -la /lib64/libc.so.6                    # RHEL/CentOS
```

**Important**: Uprobes require an absolute path to the binary or library.

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/uprobe_test.rs`

The test file contains seven tests covering help text, argument validation, successful attachment, event logging, and error handling.

### Test Descriptions

| Test | Requires Root | Purpose |
|------|---------------|---------|
| `test_uprobe_help` | No | Verify `--help` shows usage |
| `test_uprobe_requires_binary_arg` | No | Missing binary argument fails |
| `test_uprobe_requires_function_arg` | No | Missing function argument fails |
| `test_uprobe_attaches_to_libc` | Yes | Attach to `malloc` in libc |
| `test_uprobe_shows_events` | Yes | Events logged when function called |
| `test_uprobe_invalid_binary` | Yes | Non-existent binary fails gracefully |
| `test_uprobe_invalid_function` | Yes | Invalid function name fails gracefully |

### Step 1: Implement Help Test

Open `crates/ebpf-tool/tests/uprobe_test.rs` and find `test_uprobe_help` (line 26).

Replace the `todo!()` with:

```rust
#[test]
fn test_uprobe_help() {
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["uprobe", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("binary"))
        .stdout(predicate::str::contains("function"))
        .stdout(predicate::str::contains("duration"));
}
```

### Step 2: Implement Argument Validation Tests

Find `test_uprobe_requires_binary_arg` (line 52) and replace with:

```rust
#[test]
fn test_uprobe_requires_binary_arg() {
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.arg("uprobe")
        .assert()
        .failure()
        .stderr(predicate::str::contains("binary")
            .or(predicate::str::contains("required"))
            .or(predicate::str::contains("BINARY")));
}
```

Find `test_uprobe_requires_function_arg` (line 76) and replace with:

```rust
#[test]
fn test_uprobe_requires_function_arg() {
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["uprobe", "/bin/ls"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("function")
            .or(predicate::str::contains("required"))
            .or(predicate::str::contains("FUNCTION")));
}
```

### Step 3: Implement Root-Required Tests

Find `test_uprobe_attaches_to_libc` (line 110) and replace with:

```rust
#[test]
fn test_uprobe_attaches_to_libc() {
    if !is_root() {
        eprintln!("Skipping test_uprobe_attaches_to_libc: requires root");
        return;
    }

    // Find libc path (common locations)
    let libc_paths = [
        "/lib/x86_64-linux-gnu/libc.so.6",
        "/lib64/libc.so.6",
        "/usr/lib/libc.so.6",
    ];

    let libc_path = libc_paths.iter().find(|p| std::path::Path::new(p).exists());
    let Some(libc_path) = libc_path else {
        eprintln!("Skipping: libc not found at expected paths");
        return;
    };

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["uprobe", libc_path, "malloc", "-d", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Attaching uprobe")
            .or(predicate::str::contains("uprobe")));
}
```

Find `test_uprobe_shows_events` (line 146) and replace with:

```rust
#[test]
fn test_uprobe_shows_events() {
    if !is_root() {
        eprintln!("Skipping test_uprobe_shows_events: requires root");
        return;
    }

    let libc_paths = [
        "/lib/x86_64-linux-gnu/libc.so.6",
        "/lib64/libc.so.6",
        "/usr/lib/libc.so.6",
    ];

    let libc_path = libc_paths.iter().find(|p| std::path::Path::new(p).exists());
    let Some(libc_path) = libc_path else {
        eprintln!("Skipping: libc not found at expected paths");
        return;
    };

    // malloc is called frequently by most processes, so we should see events
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["uprobe", libc_path, "malloc", "-d", "2"])
        .assert()
        .success();
    // Events should appear in output (format depends on implementation)
    // You may want to add: .stdout(predicate::str::contains("uprobe triggered"));
}
```

Find `test_uprobe_invalid_binary` (line 178) and replace with:

```rust
#[test]
fn test_uprobe_invalid_binary() {
    if !is_root() {
        eprintln!("Skipping test_uprobe_invalid_binary: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["uprobe", "/nonexistent/binary/path", "some_function", "-d", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found")
            .or(predicate::str::contains("No such file"))
            .or(predicate::str::contains("does not exist"))
            .or(predicate::str::contains("failed")));
}
```

Find `test_uprobe_invalid_function` (line 209) and replace with:

```rust
#[test]
fn test_uprobe_invalid_function() {
    if !is_root() {
        eprintln!("Skipping test_uprobe_invalid_function: requires root");
        return;
    }

    // Use a valid binary but an invalid function name
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["uprobe", "/bin/ls", "nonexistent_function_xyz_123", "-d", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("function")
            .or(predicate::str::contains("symbol"))
            .or(predicate::str::contains("not found"))
            .or(predicate::str::contains("failed")));
}
```

### Step 4: Run Tests (Expect Failure)

```bash
cargo test -p ebpf-tool --test uprobe_test
```

Expected output:

```
running 7 tests
test test_uprobe_help ... FAILED
test test_uprobe_requires_binary_arg ... FAILED
test test_uprobe_requires_function_arg ... FAILED
test test_uprobe_attaches_to_libc ... FAILED
test test_uprobe_shows_events ... FAILED
test test_uprobe_invalid_binary ... FAILED
test test_uprobe_invalid_function ... FAILED

failures:
    thread 'test_uprobe_help' panicked at 'not yet implemented: Implement test for uprobe help text'
```

This is the **RED** phase. Your tests are written but the implementation has `todo!()` stubs.

## Build (Green)

Now implement the uprobe functionality to make your tests pass. This requires changes in two places:

1. **eBPF program** (`crates/ebpf-tool-ebpf/src/uprobe.rs`) - runs in kernel space
2. **Userspace CLI** (`crates/ebpf-tool/src/main.rs`) - loads and attaches the eBPF program

### Step 1: Implement the eBPF Program

**File**: `crates/ebpf-tool-ebpf/src/uprobe.rs`
**TODO location**: Line 99, `hello_uprobe` function

Open the file and find the `hello_uprobe` function. Replace the `todo!()` with:

```rust
#[uprobe]
pub fn hello_uprobe(ctx: ProbeContext) -> u32 {
    match try_hello_uprobe(&ctx) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

fn try_hello_uprobe(ctx: &ProbeContext) -> Result<(), i64> {
    // Get the current process ID (upper 32 bits of pid_tgid)
    let pid = unsafe { aya_ebpf::helpers::bpf_get_current_pid_tgid() } >> 32;

    // Log that the uprobe was triggered
    info!(ctx, "uprobe triggered! pid={}", pid);

    Ok(())
}
```

You also need to add the import at the top of the file (if not already present):

```rust
use aya_ebpf::helpers::bpf_get_current_pid_tgid;
```

**Build the eBPF program**:

```bash
cd crates/ebpf-tool-ebpf
cargo build --release --target bpfel-unknown-none -Z build-std=core
```

Or using xtask if available:

```bash
cargo xtask build-ebpf --release
```

### Step 2: Implement the Userspace Loader

**File**: `crates/ebpf-tool/src/main.rs`
**TODO location**: Line 246, `Command::Uprobe` match arm

Open the file and find the `Command::Uprobe { binary, function, duration }` match arm. Replace the `todo!()` with:

```rust
Command::Uprobe {
    binary,
    function,
    duration,
} => {
    use aya::programs::UProbe;
    use std::path::Path;
    use std::time::Duration;
    use tokio::signal;
    use tokio::time;

    // Validate the binary path exists
    if !Path::new(&binary).exists() {
        anyhow::bail!("Binary does not exist: {}", binary);
    }

    println!("Attaching uprobe to {}:{}", binary, function);
    println!("Duration: {} seconds (0 = until Ctrl+C)", duration);

    // Load the eBPF bytecode
    // The path depends on your build setup - adjust as needed
    #[cfg(debug_assertions)]
    let bytes = include_bytes_aligned!(
        "../../../target/bpfel-unknown-none/debug/uprobe"
    );
    #[cfg(not(debug_assertions))]
    let bytes = include_bytes_aligned!(
        "../../../target/bpfel-unknown-none/release/uprobe"
    );

    let mut bpf = aya::Bpf::load(bytes)?;

    // Initialize aya-log to receive log messages from the eBPF program
    if let Err(e) = aya_log::BpfLogger::init(&mut bpf) {
        // Non-fatal: logging is optional
        log::warn!("Failed to initialize eBPF logging: {}", e);
    }

    // Get the uprobe program
    let program: &mut UProbe = bpf
        .program_mut("hello_uprobe")
        .ok_or_else(|| anyhow::anyhow!("Program 'hello_uprobe' not found in eBPF object"))?
        .try_into()?;

    // Load the program into the kernel
    program.load()?;

    // Attach to the specified function in the binary
    // attach(fn_name, offset, target, pid)
    // - fn_name: Some(&function) to use symbol name, or None if using raw offset
    // - offset: 0 to attach at function entry
    // - target: path to the binary or shared library
    // - pid: None to trace all processes, Some(pid) for specific process
    program.attach(Some(&function), 0, &binary, None)
        .map_err(|e| anyhow::anyhow!(
            "Failed to attach uprobe to {}:{} - {}. \
             Check that the function exists: nm -D {} | grep {}",
            binary, function, e, binary, function
        ))?;

    println!("Uprobe attached successfully. Waiting for events...");
    println!("(Functions called will trigger the probe)\n");

    // Run for the specified duration or until Ctrl+C
    if duration == 0 {
        // Run until Ctrl+C
        println!("Press Ctrl+C to stop...");
        signal::ctrl_c().await?;
    } else {
        // Run for specified duration
        tokio::select! {
            _ = time::sleep(Duration::from_secs(duration)) => {
                println!("\nDuration complete.");
            }
            _ = signal::ctrl_c() => {
                println!("\nReceived Ctrl+C, stopping...");
            }
        }
    }

    println!("Uprobe detached.");
    Ok(())
}
```

### Step 3: Update Dependencies

Ensure `crates/ebpf-tool/Cargo.toml` includes the necessary dependencies:

```toml
[dependencies]
aya = { version = "0.12", features = ["async_tokio"] }
aya-log = "0.2"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
clap = { version = "4", features = ["derive"] }
log = "0.4"
env_logger = "0.11"
nix = { version = "0.29", features = ["user"] }
```

### Step 4: Build and Run Tests

First, build the eBPF program:

```bash
# Build eBPF bytecode
cd /workspaces/linux-isolation-learning
cargo xtask build-ebpf --release
# Or manually:
# cd crates/ebpf-tool-ebpf && cargo build --release --target bpfel-unknown-none -Z build-std=core
```

Then build the userspace tool:

```bash
cargo build -p ebpf-tool
```

Run the tests:

```bash
# Non-root tests (help and argument validation)
cargo test -p ebpf-tool --test uprobe_test -- test_uprobe_help test_uprobe_requires
```

```bash
# Root tests (actual eBPF attachment)
sudo -E cargo test -p ebpf-tool --test uprobe_test
```

Expected output:

```
running 7 tests
test test_uprobe_help ... ok
test test_uprobe_requires_binary_arg ... ok
test test_uprobe_requires_function_arg ... ok
test test_uprobe_attaches_to_libc ... ok
test test_uprobe_shows_events ... ok
test test_uprobe_invalid_binary ... ok
test test_uprobe_invalid_function ... ok

test result: ok. 7 passed; 0 failed; 0 filtered out
```

This is the **GREEN** phase. Your tests now pass.

## Verify

### Automated Verification

```bash
# Run all uprobe tests
sudo -E cargo test -p ebpf-tool --test uprobe_test

# Run entire ebpf-tool test suite
sudo -E cargo test -p ebpf-tool
```

### Manual Verification

#### 1. Trace malloc in libc

Find your libc path first:

```bash
ldd /bin/ls | grep libc
# Example output: libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
```

Attach the uprobe:

```bash
sudo cargo run -p ebpf-tool -- uprobe /lib/x86_64-linux-gnu/libc.so.6 malloc -d 5
```

Expected output:

```
Attaching uprobe to /lib/x86_64-linux-gnu/libc.so.6:malloc
Duration: 5 seconds (0 = until Ctrl+C)
Uprobe attached successfully. Waiting for events...
(Functions called will trigger the probe)

[2024-01-15T10:30:01Z INFO  ebpf_tool] uprobe triggered! pid=1234
[2024-01-15T10:30:01Z INFO  ebpf_tool] uprobe triggered! pid=1234
[2024-01-15T10:30:01Z INFO  ebpf_tool] uprobe triggered! pid=5678
...

Duration complete.
Uprobe detached.
```

You should see many events because `malloc` is called frequently by most processes.

#### 2. Trace a Less Frequent Function

To see clearer output, trace a less common function like `getenv`:

```bash
sudo cargo run -p ebpf-tool -- uprobe /lib/x86_64-linux-gnu/libc.so.6 getenv -d 10
```

In another terminal, trigger the function:

```bash
# This calls getenv() internally
env | head
bash -c 'echo $HOME'
```

#### 3. Trace bash readline

If tracing interactive shell commands:

```bash
# Find bash path
which bash
# /usr/bin/bash

# Check if readline is available
nm -D /usr/bin/bash 2>/dev/null | grep readline || echo "readline is dynamically linked"

# Trace libreadline instead
ldd /usr/bin/bash | grep readline
# Output might show: libreadline.so.8 => /lib/x86_64-linux-gnu/libreadline.so.8

sudo cargo run -p ebpf-tool -- uprobe /lib/x86_64-linux-gnu/libreadline.so.8 readline -d 30
```

Then type commands in another terminal running bash to see events.

#### 4. Verify Symbol Resolution

To confirm a function exists before tracing:

```bash
# Check if malloc exists in libc
nm -D /lib/x86_64-linux-gnu/libc.so.6 | grep " T malloc"
# Should output something like: 00000000000a1234 T malloc

# Check function offset (optional)
readelf -s /lib/x86_64-linux-gnu/libc.so.6 | grep malloc
```

## Common Errors

### 1. `Binary does not exist: /path/to/binary`

**Cause**: The specified binary path is incorrect or the file does not exist.

**Fix**:
- Use an absolute path (not relative)
- Verify the path: `ls -la /path/to/binary`
- For shared libraries, check the actual path: `ldd /bin/ls | grep libc`

### 2. `Failed to attach uprobe... symbol not found`

**Cause**: The function name does not exist in the binary's symbol table.

**Fix**:
- Check available symbols: `nm -D /path/to/binary | grep function_name`
- For stripped binaries, symbols may not be available
- Ensure you are using the correct function name (C++ uses mangled names)

```bash
# For C++ binaries, use c++filt to demangle
nm -D /path/to/binary | c++filt | grep function_name
```

### 3. `Permission denied` or `Operation not permitted`

**Cause**: eBPF operations require elevated privileges.

**Fix**: Run with sudo:

```bash
sudo -E cargo run -p ebpf-tool -- uprobe ...
```

### 4. `Program 'hello_uprobe' not found in eBPF object`

**Cause**: The eBPF bytecode was not built or the program name does not match.

**Fix**:
- Rebuild the eBPF program: `cargo xtask build-ebpf --release`
- Verify the function is annotated with `#[uprobe]`
- Ensure the function name in `.program_mut("hello_uprobe")` matches the Rust function name

### 5. No Events Appearing

**Cause**: The traced function is not being called, or logging is not working.

**Fix**:
- Try a frequently-called function like `malloc` or `write`
- Verify aya-log initialization succeeded (check for warnings)
- Manually trigger the function in another terminal
- Increase the duration: `-d 30`

### 6. `Failed to load eBPF program: EACCES`

**Cause**: Kernel lockdown or secure boot may restrict eBPF loading.

**Fix**:
- Check kernel lockdown: `cat /sys/kernel/security/lockdown`
- If in "integrity" or "confidentiality" mode, eBPF loading is restricted
- This may require kernel configuration changes or disabling secure boot

### 7. Uprobes Not Working on Containers

**Cause**: Container namespaces may affect binary paths.

**Fix**:
- Use the host path to the binary, not the container path
- Ensure the binary is the same file (check inode): `stat /path/to/binary`

## Notes

### Understanding Uprobe Overhead

Uprobes have higher overhead than kprobes because:

1. The trap switches from userspace to kernel mode
2. The eBPF program executes in kernel context
3. The replaced instruction must be emulated
4. Control returns to userspace

For high-frequency functions like `malloc`, this overhead can be significant. Consider:
- Filtering by PID to reduce event volume
- Using sampling instead of tracing every call
- Tracing less frequent functions when possible

### Return Probes (Uretprobes)

The scaffolding includes `hello_uretprobe` for tracing function returns. Uretprobes:

- Fire when the traced function returns
- Can access the return value
- Enable latency measurement (entry time - return time)

To implement duration tracking:

1. Store entry timestamp in a HashMap keyed by PID/TID
2. In uretprobe, retrieve entry time and calculate duration
3. Report both the return value and elapsed time

### Architecture Considerations

Function arguments follow calling conventions:

**x86_64 (System V ABI)**:
- Args 1-6: `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`
- Return value: `rax`

**ARM64**:
- Args 1-8: `x0` - `x7`
- Return value: `x0`

Access arguments with:

```rust
// First argument (e.g., size parameter to malloc)
let arg0: u64 = ctx.arg(0).unwrap_or(0);
```

### Stripped Binaries

Some binaries are "stripped" to remove debug symbols and reduce size:

```bash
# Check if binary is stripped
file /bin/ls
# Output: "... stripped" or "... not stripped"
```

Stripped binaries retain the dynamic symbol table (needed for linking) but lose internal symbols. You can still trace:
- Exported functions (in `.dynsym`)
- PLT entries (for shared library calls)

### Kernel Version Requirements

Uprobes require:
- Linux 3.5+ for basic uprobe support
- Linux 4.1+ for uretprobes
- Linux 4.17+ for better eBPF uprobe integration

Check your kernel:

```bash
uname -r
```

### Security Implications

Uprobes can observe sensitive data:
- Function arguments may contain passwords, keys, or PII
- SSL/TLS plaintext can be observed via `SSL_read`/`SSL_write`
- File paths and network addresses are visible

Always:
- Limit uprobe usage to authorized debugging/monitoring
- Avoid logging sensitive argument values in production
- Consider compliance requirements (GDPR, HIPAA, etc.)

### Man Pages

- `man 2 perf_event_open` - Low-level probe interface
- Documentation in `/usr/src/linux/Documentation/trace/uprobetracer.txt` (if available)

### Further Reading

- **BCC uprobe examples**: github.com/iovisor/bcc/blob/master/tools/funccount.py
- **Aya uprobe documentation**: aya-rs.dev/book/programs/probes
- **Linux uprobe internals**: www.kernel.org/doc/html/latest/trace/uprobetracer.html

## Extension: Reading Function Arguments

Once basic tracing works, you can extend the eBPF program to read arguments.

Example: Read the `size` argument to `malloc`:

```rust
#[uprobe]
pub fn hello_uprobe(ctx: ProbeContext) -> u32 {
    let pid = unsafe { bpf_get_current_pid_tgid() } >> 32;

    // malloc(size_t size) - first argument
    let size: u64 = match ctx.arg(0) {
        Some(s) => s,
        None => 0,
    };

    info!(&ctx, "malloc called: pid={} size={}", pid, size);
    0
}
```

This enables powerful observability:
- Track allocation sizes across the system
- Find processes making unusually large allocations
- Correlate with memory pressure events

## Next

`06-tracepoints.md` - Attach to kernel tracepoints for stable, documented trace points that survive kernel upgrades
