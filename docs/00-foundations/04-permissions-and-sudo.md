# 04 Permissions and sudo

## Goal
Understand why Linux isolation syscalls require elevated privileges, learn to detect and report missing permissions from Rust, and discover how user namespaces can grant capabilities without root.

**Deliverable**: A `check-caps` subcommand that inspects effective capabilities and reports whether the process can create namespaces.

## Prereqs
- Completed `03-procfs-intro.md` (familiarity with reading `/proc`)
- `cargo run -q -p ns-tool -- proc` works
- Basic understanding of Linux users and root

**Estimated time**: ~30-40 minutes

## Concepts

### Why Do Namespace Operations Require Privileges?

Creating a namespace fundamentally changes the kernel's view of what resources a process can access. Without privilege checks, any unprivileged process could:

1. **Escape resource limits**: Create a new PID namespace to become PID 1 and avoid being killed
2. **Hide from monitoring**: Mount over `/proc` to hide processes from system administrators
3. **Spoof identity**: Change hostname or network identity to impersonate other systems
4. **Bypass access controls**: Use mount namespaces to remount filesystems with different permissions

The kernel enforces these checks through **capabilities** rather than just checking if UID is 0.

### Linux Capabilities: Fine-Grained Privilege Control

Instead of an all-or-nothing root model, Linux divides root powers into ~40 distinct **capabilities**. Key ones for container operations:

| Capability | Purpose | Required For |
|------------|---------|--------------|
| `CAP_SYS_ADMIN` | Catch-all for admin operations | Most namespace types, mounting |
| `CAP_NET_ADMIN` | Network configuration | Network namespaces, veth pairs |
| `CAP_SETUID` | Change process UIDs | User namespace UID mapping |
| `CAP_SETGID` | Change process GIDs | User namespace GID mapping |
| `CAP_SYS_CHROOT` | Use chroot() | Pivot root operations |

### The User Namespace Exception

User namespaces are special: **unprivileged users can create them** (on most modern systems). Inside a user namespace:

- The creating process gains **all capabilities within that namespace**
- This allows creating nested namespaces (PID, mount, network, etc.)
- The kernel still enforces security at namespace boundaries

This is how rootless containers (Podman, rootless Docker) work.

### Reading Capabilities from /proc

The kernel exposes capability sets in `/proc/self/status`:

```
CapInh: 0000000000000000    # Inheritable: passed to child processes
CapPrm: 0000000000000000    # Permitted: max caps process can use
CapEff: 0000000000000000    # Effective: caps currently active
CapBnd: 000001ffffffffff    # Bounding: upper limit for process
CapAmb: 0000000000000000    # Ambient: preserved across execve
```

These are hex-encoded bitmasks. For example, `CAP_SYS_ADMIN` is bit 21.

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/caps_test.rs`

We will add a new test file for capability checking. Create the file with these tests:

### Step 1: Create the test file

Create `crates/ns-tool/tests/caps_test.rs`:

```rust
// Tests for the `check-caps` subcommand (capability inspection)
// Lesson: docs/00-foundations/04-permissions-and-sudo.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests run as the current user (not root).
// Some tests check behavior with/without privileges.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_check_caps_runs_successfully() {
    // The check-caps subcommand should always succeed (even without root)
    // It inspects capabilities and reports what the process can do
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("check-caps")
        .assert()
        .success();
}

#[test]
fn test_check_caps_shows_effective_capabilities() {
    // Output should include the effective capability hex string
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("check-caps")
        .assert()
        .success()
        .stdout(predicate::str::contains("Effective capabilities:"));
}

#[test]
fn test_check_caps_reports_cap_sys_admin_status() {
    // Output should report whether CAP_SYS_ADMIN is present
    // This determines if most namespace operations will work
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("check-caps")
        .assert()
        .success()
        .stdout(predicate::str::contains("CAP_SYS_ADMIN:"));
}

#[test]
fn test_check_caps_shows_namespace_creation_ability() {
    // Output should summarize what namespace operations are possible
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("check-caps")
        .assert()
        .success()
        .stdout(predicate::str::contains("Namespace creation:"));
}

#[test]
fn test_check_caps_always_shows_user_ns_as_available() {
    // User namespaces can be created without privileges (on most systems)
    // The output should reflect this
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("check-caps")
        .assert()
        .success()
        .stdout(predicate::str::contains("user").and(predicate::str::contains("available")));
}
```

### Step 2: Run the tests (expect failure)

```bash
cargo test -p ns-tool --test caps_test
```

Expected output: Tests fail because the `check-caps` subcommand is not yet implemented (RED phase).

The tests will try to run the `check-caps` subcommand but encounter a panic from the `todo!()` stub:

```
thread 'test_check_caps_runs_successfully' panicked at 'Implement check-caps - write tests first!'
```

This is expected and desired - it shows the tests are correctly exercising the CLI.

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`

### Step 1: Add the subcommand to the CLI

Find the `Command` enum (around line 13) and add a new variant:

```rust
#[derive(Subcommand)]
enum Command {
    Pid,
    Uts,
    Ipc,
    Mount,
    Net,
    User,
    Cgroup,
    Time,
    Setns,
    Proc,
    /// Check effective capabilities and report namespace creation ability
    CheckCaps,
}
```

### Step 2: Add the match arm

Find the `match cli.command` block (around line 29) and add a new arm before the closing brace:

```rust
        Command::CheckCaps => check_capabilities()?,
```

### Step 3: Implement the capability checking function

Add this function after `print_proc_ns()`:

```rust
/// Check current process capabilities and report namespace creation ability
fn check_capabilities() -> Result<()> {
    // Read capability information from /proc/self/status
    let status = std::fs::read_to_string("/proc/self/status")?;

    // Parse the CapEff (effective capabilities) line
    let cap_eff = status
        .lines()
        .find(|line| line.starts_with("CapEff:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| anyhow::anyhow!("Could not find CapEff in /proc/self/status"))?;

    // Parse the hex value
    let cap_value = u64::from_str_radix(cap_eff, 16)
        .map_err(|e| anyhow::anyhow!("Failed to parse capability value: {}", e))?;

    println!("Effective capabilities: 0x{:016x}", cap_value);
    println!();

    // Check specific capabilities
    // CAP_SYS_ADMIN is bit 21 (0x200000 = 1 << 21)
    const CAP_SYS_ADMIN: u64 = 1 << 21;
    const CAP_NET_ADMIN: u64 = 1 << 12;
    const CAP_SETUID: u64 = 1 << 7;
    const CAP_SETGID: u64 = 1 << 6;

    let has_sys_admin = (cap_value & CAP_SYS_ADMIN) != 0;
    let has_net_admin = (cap_value & CAP_NET_ADMIN) != 0;
    let has_setuid = (cap_value & CAP_SETUID) != 0;
    let has_setgid = (cap_value & CAP_SETGID) != 0;

    println!("Key capabilities:");
    println!("  CAP_SYS_ADMIN: {}", if has_sys_admin { "YES" } else { "NO" });
    println!("  CAP_NET_ADMIN: {}", if has_net_admin { "YES" } else { "NO" });
    println!("  CAP_SETUID:    {}", if has_setuid { "YES" } else { "NO" });
    println!("  CAP_SETGID:    {}", if has_setgid { "YES" } else { "NO" });
    println!();

    // Report namespace creation ability
    println!("Namespace creation:");

    // User namespaces can be created without privileges (check kernel config)
    let user_ns_available = check_user_ns_available();
    println!("  user:    {} (available without privileges)",
             if user_ns_available { "available" } else { "restricted" });

    // Other namespaces require CAP_SYS_ADMIN or being inside a user namespace
    let privileged = has_sys_admin;
    println!("  pid:     {}", if privileged { "available" } else { "requires sudo or user namespace" });
    println!("  mount:   {}", if privileged { "available" } else { "requires sudo or user namespace" });
    println!("  uts:     {}", if privileged { "available" } else { "requires sudo or user namespace" });
    println!("  ipc:     {}", if privileged { "available" } else { "requires sudo or user namespace" });
    println!("  net:     {}", if privileged && has_net_admin { "available" } else { "requires sudo or user namespace" });
    println!("  cgroup:  {}", if privileged { "available" } else { "requires sudo or user namespace" });
    println!();

    // Provide actionable guidance
    if !privileged {
        println!("Tip: Run with sudo for full namespace capabilities, or use");
        println!("     user namespaces to gain capabilities without root.");
    } else {
        println!("You have full privileges for namespace operations.");
    }

    Ok(())
}

/// Check if unprivileged user namespaces are allowed on this system
fn check_user_ns_available() -> bool {
    // Check the kernel sysctl that controls unprivileged user namespaces
    // This may be in different locations depending on the distro
    let paths = [
        "/proc/sys/kernel/unprivileged_userns_clone",
        "/proc/sys/user/max_user_namespaces",
    ];

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(value) = content.trim().parse::<u64>() {
                // If max_user_namespaces > 0 or unprivileged_userns_clone == 1
                if value > 0 {
                    return true;
                }
            }
        }
    }

    // If we can't check, assume it's available (most modern systems allow it)
    // A real implementation would try to create a user namespace as a test
    true
}
```

### Step 4: Run the tests (expect success)

```bash
cargo test -p ns-tool --test caps_test
```

Expected output: All tests pass (GREEN phase).

## Verify

**Automated verification**:
```bash
cargo test -p ns-tool --test caps_test  # All 5 tests pass
cargo test -p ns-tool                    # All ns-tool tests pass
```

**Manual verification** (observe the actual behavior):

### Without root privileges:
```bash
cargo run -q -p ns-tool -- check-caps
```

Expected output (as unprivileged user):
```
Effective capabilities: 0x0000000000000000

Key capabilities:
  CAP_SYS_ADMIN: NO
  CAP_NET_ADMIN: NO
  CAP_SETUID:    NO
  CAP_SETGID:    NO

Namespace creation:
  user:    available (available without privileges)
  pid:     requires sudo or user namespace
  mount:   requires sudo or user namespace
  uts:     requires sudo or user namespace
  ipc:     requires sudo or user namespace
  net:     requires sudo or user namespace
  cgroup:  requires sudo or user namespace

Tip: Run with sudo for full namespace capabilities, or use
     user namespaces to gain capabilities without root.
```

### With root privileges:
```bash
sudo cargo run -q -p ns-tool -- check-caps
```

Expected output:
```
Effective capabilities: 0x000001ffffffffff

Key capabilities:
  CAP_SYS_ADMIN: YES
  CAP_NET_ADMIN: YES
  CAP_SETUID:    YES
  CAP_SETGID:    YES

Namespace creation:
  user:    available (available without privileges)
  pid:     available
  mount:   available
  uts:     available
  ipc:     available
  net:     available
  cgroup:  available

You have full privileges for namespace operations.
```

### Cross-verify with system tools:
```bash
# Compare with getpcaps (if available)
getpcaps $$

# Or decode capabilities manually
grep Cap /proc/self/status
```

## Clean Up

No cleanup required for this lesson. The `check-caps` subcommand only reads from `/proc` and does not modify system state.

## Common Errors

### 1. "Operation not permitted" when creating namespaces

**Cause**: The process lacks `CAP_SYS_ADMIN` and is not inside a user namespace.

**Fix**: Either:
- Run with `sudo`
- Create a user namespace first (gives capabilities inside that namespace)

```bash
# This will fail without privileges:
cargo run -q -p ns-tool -- pid
# Error: Operation not permitted (os error 1)

# This works:
sudo cargo run -q -p ns-tool -- pid
```

### 2. User namespaces disabled on the system

**Symptom**: Even user namespace creation fails with "Operation not permitted".

**Cause**: Some distributions disable unprivileged user namespaces for security. Check:
```bash
cat /proc/sys/kernel/unprivileged_userns_clone 2>/dev/null || echo "not configured"
sysctl user.max_user_namespaces
```

**Fix**: Enable with sysctl (as root):
```bash
sudo sysctl -w kernel.unprivileged_userns_clone=1
# Or permanently in /etc/sysctl.d/
```

### 3. "Could not find CapEff" error

**Cause**: The `/proc/self/status` file has an unexpected format (rare).

**Fix**: Verify your system exposes capabilities:
```bash
grep -i cap /proc/self/status
```

If capabilities are not shown, you may be on a very old or unusual kernel.

### 4. Capability bitmask appears as all zeros with sudo

**Cause**: You may have used `sudo -E` which does not give full root capabilities, or sudo is configured with limited capabilities.

**Fix**: Use plain `sudo` without `-E`, or verify your sudoers configuration:
```bash
sudo -n grep Cap /proc/self/status
```

## Best Practices: Fail Fast with Helpful Errors

When writing namespace code, always check capabilities **before** attempting the operation. This provides a much better user experience than cryptic syscall errors.

### Pattern: Early Permission Check

```rust
use anyhow::{bail, Result};

fn require_cap_sys_admin() -> Result<()> {
    let status = std::fs::read_to_string("/proc/self/status")?;
    let cap_eff = status
        .lines()
        .find(|line| line.starts_with("CapEff:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| anyhow::anyhow!("Could not read capabilities"))?;

    let cap_value = u64::from_str_radix(cap_eff, 16)?;
    const CAP_SYS_ADMIN: u64 = 1 << 21;

    if (cap_value & CAP_SYS_ADMIN) == 0 {
        bail!(
            "This operation requires CAP_SYS_ADMIN.\n\
             Try running with: sudo cargo run -p ns-tool -- <command>\n\
             Or use a user namespace to gain capabilities."
        );
    }

    Ok(())
}
```

### Pattern: User Namespace as Privilege Escalation

```rust
use nix::sched::{unshare, CloneFlags};

fn with_user_namespace<F>(f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    // Create a user namespace first (works without privileges)
    unshare(CloneFlags::CLONE_NEWUSER)?;

    // Now we have all capabilities *within* this namespace
    // Write UID/GID mappings (required for most operations)
    std::fs::write("/proc/self/uid_map", "0 1000 1")?;
    std::fs::write("/proc/self/setgroups", "deny")?;
    std::fs::write("/proc/self/gid_map", "0 1000 1")?;

    // Now we can create other namespaces
    f()
}
```

## Notes

- **Capability bit positions**: The kernel defines these in `include/uapi/linux/capability.h`. Common ones: `CAP_SYS_ADMIN=21`, `CAP_NET_ADMIN=12`, `CAP_SETUID=7`, `CAP_SETGID=6`.

- **The `caps` crate**: For production code, consider the [`caps`](https://crates.io/crates/caps) crate which provides a type-safe API. We read `/proc` directly here for educational purposes.

- **Ambient capabilities**: A newer mechanism (Linux 4.3+) that allows non-root programs to retain capabilities across `execve`. Useful for container runtimes.

- **Secure computing (seccomp)**: Even with capabilities, syscalls can be further restricted. We cover this in the runc/OCI lessons.

- **Podman rootless mode**: A great example of using user namespaces for unprivileged container operation. Explore with `podman unshare` to see how it sets up mappings.

## Further Reading

- `man 7 capabilities` - Comprehensive capability documentation
- `man 7 user_namespaces` - User namespace details and security model
- `man 2 unshare` - The syscall for creating namespaces
- [Rootless Containers](https://rootlesscontaine.rs/) - Community resource on unprivileged containers

## Next
`05-error-handling.md` - Use `anyhow` to surface syscall errors clearly with context.
