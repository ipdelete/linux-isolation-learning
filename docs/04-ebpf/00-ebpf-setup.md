# 00 eBPF Setup

## Goal

Validate your eBPF development environment and understand the prerequisites for running eBPF programs. You will build a `check` subcommand in `ebpf-tool` that verifies kernel version, BTF availability, and permissions required for eBPF development.

## Prereqs

- Completed `docs/00-foundations/` section (especially `01-rust-syscall-basics.md`)
- Linux kernel 5.8+ (for CAP_BPF and modern eBPF features)
- `sudo` access (loading eBPF programs requires elevated privileges)
- Development container or system with LLVM and bpf-linker installed

## Background: What is eBPF?

eBPF (extended Berkeley Packet Filter) is a revolutionary technology that allows you to run sandboxed programs in the Linux kernel without changing kernel source code or loading kernel modules. Originally designed for network packet filtering, eBPF has evolved into a general-purpose in-kernel virtual machine.

**Key properties:**

- eBPF programs run in a safe sandbox within the kernel
- The eBPF verifier ensures programs cannot crash the kernel
- Programs are JIT-compiled for near-native performance
- Use cases include networking, observability, security, and tracing

**Why eBPF for container observability:**

- Trace syscalls made by containerized processes
- Monitor network traffic without modifying application code
- Collect performance metrics with minimal overhead
- Enforce security policies at the kernel level

### What is BTF?

BTF (BPF Type Format) is a metadata format that describes the types used in eBPF programs and the kernel. BTF enables CO-RE (Compile Once, Run Everywhere), which allows eBPF programs to run on different kernel versions without recompilation.

**Why BTF matters:**

- Kernel data structures change between versions
- Without BTF, you need to recompile eBPF programs for each kernel
- BTF stores type information so the loader can relocate field accesses
- Modern kernels ship with BTF at `/sys/kernel/btf/vmlinux`

### What is Aya?

Aya is a pure Rust eBPF framework. Unlike traditional approaches that require C and complex toolchains (clang, LLVM, libbpf), Aya lets you write both the eBPF programs and userspace loaders in Rust.

**Aya advantages:**

- Write eBPF programs in Rust (`#![no_std]`)
- No C toolchain dependencies for the eBPF code itself
- Type-safe communication between kernel and userspace
- Excellent error messages and developer experience
- Uses Rust's `aya-bpf` crate for the kernel side and `aya` crate for userspace

### Required Capabilities

Loading eBPF programs requires specific Linux capabilities:

- **CAP_BPF** (kernel 5.8+): Allows loading BPF programs and creating maps
- **CAP_PERFMON** (kernel 5.8+): Required for performance monitoring programs
- **CAP_SYS_ADMIN** (legacy): On older kernels, this single capability grants all BPF permissions

In practice, running as root (UID 0) grants all capabilities. For production systems, you can use fine-grained capabilities with tools like `setcap`.

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/check_test.rs`

What the tests should verify:

- Help output: The `check --help` command displays subcommand documentation
- Success case: The `check` command runs successfully with root privileges
- Kernel version: Output includes kernel version information
- BTF status: Output shows whether BTF is available
- Permissions: Output shows capability/permission information

### Steps

1. Open `crates/ebpf-tool/tests/check_test.rs`

2. Find the `test_check_help` test function (line 23)

3. Replace the `todo!()` with the test implementation:

```rust
#[test]
fn test_check_help() {
    // Test that --help shows check subcommand info
    //
    // The check subcommand should have helpful documentation explaining
    // that it validates the eBPF environment.

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"));
}
```

4. Find the `test_check_runs_as_root` test function (line 47)

5. Replace the `todo!()` with the test implementation:

```rust
#[test]
fn test_check_runs_as_root() {
    // Test that check subcommand runs successfully with root privileges
    //
    // When run as root, the check command should complete successfully
    // and output diagnostic information about the eBPF environment.

    if !is_root() {
        eprintln!("Skipping test_check_runs_as_root: requires root privileges");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.arg("check")
        .assert()
        .success();
}
```

6. Find the `test_check_shows_kernel_version` test function (line 74)

7. Replace the `todo!()` with the test implementation:

```rust
#[test]
fn test_check_shows_kernel_version() {
    // Test that check output includes kernel version information
    //
    // The check command should display the current kernel version
    // and indicate whether it meets the minimum requirements for eBPF.

    if !is_root() {
        eprintln!("Skipping test_check_shows_kernel_version: requires root privileges");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.arg("check")
        .assert()
        .success()
        .stdout(predicate::str::is_match("[Kk]ernel").unwrap());
}
```

8. Find the `test_check_shows_btf_status` test function (line 104)

9. Replace the `todo!()` with the test implementation:

```rust
#[test]
fn test_check_shows_btf_status() {
    // Test that check output includes BTF availability status
    //
    // BTF (BPF Type Format) is essential for CO-RE (Compile Once, Run Everywhere).
    // The check command should verify that /sys/kernel/btf/vmlinux exists.

    if !is_root() {
        eprintln!("Skipping test_check_shows_btf_status: requires root privileges");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("BTF"));
}
```

10. Find the `test_check_shows_permissions` test function (line 134)

11. Replace the `todo!()` with the test implementation:

```rust
#[test]
fn test_check_shows_permissions() {
    // Test that check output includes permission/capability information
    //
    // The check command should report whether the current user has
    // sufficient permissions to load and run eBPF programs.

    if !is_root() {
        eprintln!("Skipping test_check_shows_permissions: requires root privileges");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.arg("check")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("CAP_BPF")
                .or(predicate::str::contains("CAP_SYS_ADMIN"))
                .or(predicate::str::contains("ermission"))
        );
}
```

12. Run the tests (expect failure because implementation is missing):

```bash
cargo test -p ebpf-tool --test check_test
```

Expected output:

```
running 5 tests
test test_check_help ... FAILED
test test_check_runs_as_root ... FAILED
test test_check_shows_btf_status ... FAILED
test test_check_shows_kernel_version ... FAILED
test test_check_shows_permissions ... FAILED

failures:

---- test_check_help stdout ----
thread 'test_check_help' panicked at 'not yet implemented: Implement test for check --help'
```

This is the **RED** phase - your tests are written but the implementation does not exist yet.

## Build (Green)

**Implementation file**: `crates/ebpf-tool/src/main.rs`
**TODO location**: Line ~159 in the `Command::Check` match arm

Now implement the `check` subcommand and helper functions to make your tests pass.

### Step 1: Implement the Helper Functions

First, implement the three helper functions at the bottom of `main.rs` (starting around line 363):

#### 1.1 Implement `get_kernel_version()` (line ~383)

Replace the `todo!()` with:

```rust
/// Get the kernel version as a tuple (major, minor, patch).
#[allow(dead_code)]
fn get_kernel_version() -> Result<(u32, u32, u32)> {
    use nix::sys::utsname::uname;

    let info = uname()?;
    let release = info.release().to_string_lossy();

    // Parse version string like "5.15.0-generic" or "6.1.0"
    let parts: Vec<&str> = release.split('.').collect();

    let major = parts
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let minor = parts
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Patch may have suffix like "0-generic", extract just the number
    let patch = parts
        .get(2)
        .and_then(|s| {
            s.split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0);

    Ok((major, minor, patch))
}
```

#### 1.2 Implement `check_btf_available()` (line ~375)

Replace the `todo!()` with:

```rust
/// Check if BTF (BPF Type Format) is available on the system.
///
/// BTF enables CO-RE (Compile Once, Run Everywhere) which allows
/// eBPF programs to run on different kernel versions without recompilation.
#[allow(dead_code)]
fn check_btf_available() -> bool {
    std::path::Path::new("/sys/kernel/btf/vmlinux").exists()
}
```

#### 1.3 Implement `check_bpf_capability()` (line ~363)

Replace the `todo!()` with:

```rust
/// Check if the current process has CAP_BPF or CAP_SYS_ADMIN capability.
///
/// This is needed for loading eBPF programs. On modern kernels (5.8+),
/// CAP_BPF is sufficient. On older kernels, CAP_SYS_ADMIN is required.
#[allow(dead_code)]
fn check_bpf_capability() -> bool {
    // Simple check: running as root grants all capabilities
    nix::unistd::Uid::effective().is_root()
}
```

### Step 2: Implement the Check Subcommand

Find the `Command::Check` match arm (around line 159) and replace the `todo!()`:

```rust
Command::Check => {
    println!("=== eBPF Environment Check ===\n");

    // Check 1: Kernel version
    match get_kernel_version() {
        Ok((major, minor, patch)) => {
            let version_str = format!("{}.{}.{}", major, minor, patch);
            let status = if major > 5 || (major == 5 && minor >= 8) {
                "[OK]"
            } else {
                "[WARN - recommend 5.8+]"
            };
            println!("Kernel version: {} {}", version_str, status);
        }
        Err(e) => {
            println!("Kernel version: ERROR - {}", e);
        }
    }

    // Check 2: BTF availability
    let btf_path = "/sys/kernel/btf/vmlinux";
    if check_btf_available() {
        println!("BTF available: {} [OK]", btf_path);
    } else {
        println!("BTF available: NOT FOUND [ERROR]");
        println!("  -> BTF is required for CO-RE. Check kernel config or install linux-headers.");
    }

    // Check 3: Permissions (capabilities)
    if check_bpf_capability() {
        // Try to detect which capability we likely have
        let (major, minor, _) = get_kernel_version().unwrap_or((0, 0, 0));
        let cap_name = if major > 5 || (major == 5 && minor >= 8) {
            "CAP_BPF"
        } else {
            "CAP_SYS_ADMIN"
        };
        println!("Permissions: {} [OK]", cap_name);
    } else {
        println!("Permissions: INSUFFICIENT [ERROR]");
        println!("  -> Run with sudo or grant CAP_BPF capability");
    }

    // Check 4: eBPF syscall accessibility (basic test)
    // We just report based on capability check since actual bpf() would require
    // loading a program, which we test in later lessons
    if check_bpf_capability() && check_btf_available() {
        println!("eBPF syscall: accessible [OK]");
        println!("\n=== All checks passed! Ready for eBPF development. ===");
    } else {
        println!("eBPF syscall: may not be accessible [WARN]");
        println!("\n=== Some checks failed. Review errors above. ===");
    }

    Ok(())
}
```

### Step 3: Run the Tests

Run the tests to verify your implementation:

```bash
sudo -E cargo test -p ebpf-tool --test check_test
```

Expected output:

```
running 5 tests
test test_check_help ... ok
test test_check_runs_as_root ... ok
test test_check_shows_btf_status ... ok
test test_check_shows_kernel_version ... ok
test test_check_shows_permissions ... ok

test result: ok. 5 passed; 0 failed; 0 filtered out
```

This is the **GREEN** phase - your tests now pass.

## Verify

**Automated verification**:

```bash
# Run all tests for ebpf-tool (requires sudo for capability checks)
sudo -E cargo test -p ebpf-tool

# Run just the check tests
sudo -E cargo test -p ebpf-tool --test check_test
```

All tests should pass.

**Manual verification** (observe the actual behavior):

1. Run the `check` subcommand manually:

```bash
sudo cargo run -p ebpf-tool -- check
```

Expected output (your versions may differ):

```
=== eBPF Environment Check ===

Kernel version: 6.1.0 [OK]
BTF available: /sys/kernel/btf/vmlinux [OK]
Permissions: CAP_BPF [OK]
eBPF syscall: accessible [OK]

=== All checks passed! Ready for eBPF development. ===
```

2. Verify the `--help` output:

```bash
cargo run -p ebpf-tool -- check --help
```

Expected output:

```
Validate eBPF environment (BTF, kernel version, permissions)

Usage: ebpf-tool check

Options:
  -h, --help  Print help
```

3. Manually verify kernel version:

```bash
uname -r
```

Compare with the output from the check command.

4. Manually verify BTF availability:

```bash
ls -la /sys/kernel/btf/vmlinux
```

If the file exists, BTF is available.

5. Check available tracing features (for future lessons):

```bash
# List available kprobes
sudo cat /sys/kernel/debug/tracing/available_filter_functions | head -20

# List available tracepoints
ls /sys/kernel/debug/tracing/events/ | head -10
```

## Common Errors

1. **`Operation not permitted` when running the check command**
   - Cause: The check command needs to verify capabilities, which requires elevated privileges
   - Fix: Run with `sudo`: `sudo cargo run -p ebpf-tool -- check`
   - Note: The check command itself does not load eBPF programs, but later lessons will require root

2. **BTF shows as NOT FOUND**
   - Cause: Kernel was compiled without BTF support or BTF file is missing
   - Fix:
     - Check kernel config: `zcat /proc/config.gz | grep CONFIG_DEBUG_INFO_BTF` (should be `=y`)
     - Install kernel headers: `apt install linux-headers-$(uname -r)` on Debian/Ubuntu
     - Consider using a newer kernel (5.4+ with distro backports, or 5.8+ for full support)
   - Workaround: You can still run eBPF programs without CO-RE, but they may not be portable

3. **Kernel version shows as [WARN]**
   - Cause: Kernel is older than 5.8, which lacks some modern eBPF features
   - Impact:
     - No CAP_BPF (requires CAP_SYS_ADMIN instead)
     - No BPF ring buffer (uses perf buffer instead)
     - Some program types may be unavailable
   - Fix: Consider upgrading to a newer kernel, or adjust expectations for available features

4. **`error[E0433]: failed to resolve: use of undeclared crate or module`**
   - Cause: Missing import statements in your implementation
   - Fix: Ensure you have the required imports at the top of main.rs or in the match arm:
     ```rust
     use nix::sys::utsname::uname;
     ```

## Notes

**Understanding the eBPF ecosystem:**

The ebpf-tool crate is part of a three-crate architecture:

```
crates/
  ebpf-tool/           <- Userspace CLI (you are here)
  ebpf-tool-ebpf/      <- eBPF programs (no_std, BPF target)
  ebpf-tool-common/    <- Shared types between userspace and eBPF
```

This separation exists because eBPF programs run in the kernel and have different constraints than userspace code:

- `ebpf-tool-ebpf`: Compiled for the BPF target, uses `#![no_std]`, has limited library access
- `ebpf-tool`: Normal Rust binary that loads and interacts with eBPF programs
- `ebpf-tool-common`: Shared data structures (event types, map keys) used by both

**Kernel version compatibility:**

| Feature | Minimum Kernel |
|---------|---------------|
| eBPF basics | 3.18 |
| Kprobes | 4.1 |
| Tracepoints | 4.7 |
| BTF | 4.18 (limited), 5.2 (full) |
| CO-RE | 5.2 |
| CAP_BPF | 5.8 |
| Ring buffer | 5.8 |
| Uprobes | 4.1 |

**Development container requirements:**

The devcontainer for this project includes:

- LLVM and clang (for compiling eBPF programs)
- bpf-linker (Rust BPF linker)
- rust-src component (for building `#![no_std]` code)
- Mounted `/sys/kernel/debug` for tracing access

If running outside the devcontainer, ensure these are installed:

```bash
# Install LLVM
apt install llvm clang

# Install bpf-linker
cargo install bpf-linker

# Add rust-src
rustup component add rust-src
```

**Manual pages and documentation:**

- `man 2 bpf` - The bpf() system call
- [Aya Book](https://aya-rs.dev/book/) - Official Aya documentation
- [BPF and XDP Reference](https://docs.cilium.io/en/latest/bpf/) - Cilium's BPF documentation
- [Kernel BPF Documentation](https://www.kernel.org/doc/html/latest/bpf/) - Official kernel docs

## Next

`01-hello-kprobe.md` - Write your first eBPF program using kprobes to trace kernel function calls
