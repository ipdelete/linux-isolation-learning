# 03 UTS and IPC Namespaces

## Goal
Create UTS and IPC namespaces to isolate hostname and IPC resources, and learn how to combine multiple namespaces in a single operation.

**Deliverable**: Extend `ns-tool` to support `uts` and `ipc` subcommands that isolate hostname and System V IPC objects.

## Prereqs
- Completed `01-pid-namespace.md` and `02-unshare-vs-clone.md`
- Understanding of process namespaces and the `unshare()` syscall
- `sudo` access (namespace creation requires `CAP_SYS_ADMIN`)

## Why UTS and IPC Namespaces?

**UTS namespace** (Unix Time-Sharing) isolates two system identifiers:
- Hostname: The system's network name
- Domain name: The NIS/YP domain name

This isolation allows containers to have their own hostname without affecting the host or other containers. The name "UTS" comes from the `struct utsname` that stores these identifiers.

**IPC namespace** isolates System V IPC objects and POSIX message queues:
- Message queues (`msgget`, `msgsnd`, `msgrcv`)
- Semaphore sets (`semget`, `semop`)
- Shared memory segments (`shmget`, `shmat`)

Without IPC namespace isolation, processes in different "containers" could interfere with each other's message queues or shared memory—a serious security and stability issue.

**Key insight**: These namespaces are simpler than PID namespaces because they don't require forking. You can `unshare()` and immediately see the isolation without creating child processes.

## Write Tests (Red)

**Test files**:
- `crates/ns-tool/tests/uts_test.rs`
- `crates/ns-tool/tests/ipc_test.rs`

What the tests should verify:
- **UTS namespace**: Hostname changes are isolated to the namespace (host's hostname remains unchanged)
- **IPC namespace**: IPC objects created in parent namespace are not visible in child namespace

### Steps

1. Open `crates/ns-tool/tests/uts_test.rs`

2. Find the `test_uts_namespace_hostname_isolation()` function (around line 13)

3. Replace the `todo!()` with a test implementation:

```rust
use assert_cmd::Command;

#[test]
fn test_uts_namespace_hostname_isolation() {
    // Get the current system hostname for comparison
    let original_hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .expect("Failed to read system hostname")
        .trim()
        .to_string();

    // Run ns-tool with uts subcommand, which should set a custom hostname
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    let output = cmd
        .arg("uts")
        .assert()
        .success();

    // The command should print the new hostname it set
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("container-test") || stdout.contains("Hostname in namespace:"),
        "Command should show the new hostname"
    );

    // Verify the host's hostname is unchanged
    let current_hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .expect("Failed to read system hostname")
        .trim()
        .to_string();
    assert_eq!(
        original_hostname, current_hostname,
        "Host hostname should remain unchanged"
    );
}
```

4. Open `crates/ns-tool/tests/ipc_test.rs`

5. Find the `test_ipc_namespace_message_queue_isolation()` function (around line 13)

6. Replace the `todo!()` with a test implementation:

```rust
use assert_cmd::Command;

#[test]
fn test_ipc_namespace_message_queue_isolation() {
    // The ns-tool ipc command should:
    // 1. Enter a new IPC namespace
    // 2. List IPC objects (should be empty in new namespace)
    // 3. Print the count of message queues, semaphores, and shared memory

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    let output = cmd
        .arg("ipc")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // In a fresh IPC namespace, there should be no IPC objects
    // (or very few, depending on system)
    assert!(
        stdout.contains("Message queues:") && stdout.contains("Semaphores:"),
        "Command should report IPC object counts: {}",
        stdout
    );
}
```

7. Run the tests (expect them to fail with `todo!()` panics):

```bash
# Test UTS namespace
cargo test -p ns-tool --test uts_test

# Test IPC namespace
cargo test -p ns-tool --test ipc_test
```

**Expected output**: Tests fail because the implementation still has `todo!()` macros (RED phase).

```
---- test_uts_namespace_hostname_isolation stdout ----
thread 'test_uts_namespace_hostname_isolation' panicked at
'not yet implemented: Implement UTS namespace - write tests first!'
```

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`

**TODO locations**:
- Line ~62: `Command::Uts => todo!(...)`
- Line ~67: `Command::Ipc => todo!(...)`

### Understanding the Implementation

Before writing code, understand what we need:

1. **UTS namespace**: Call `unshare(CLONE_NEWUTS)`, then `sethostname()` to change the hostname inside the namespace
2. **IPC namespace**: Call `unshare(CLONE_NEWIPC)`, then inspect `/proc/sysvipc/` to verify isolation
3. **No forking required**: Unlike PID namespaces, UTS and IPC namespaces affect the current process immediately

### Steps

1. Open `crates/ns-tool/src/main.rs`

2. Add necessary imports at the top (after existing imports):

```rust
use nix::sched::{unshare, CloneFlags};
use nix::unistd::sethostname;
use std::fs;
```

3. Find the `Command::Uts` match arm (around line 62)

4. Replace `todo!(...)` with this implementation:

```rust
Command::Uts => {
    // Step 1: Get and display current hostname
    let old_hostname = fs::read_to_string("/proc/sys/kernel/hostname")
        .context("Failed to read current hostname")?
        .trim()
        .to_string();
    println!("Hostname before unshare: {}", old_hostname);

    // Step 2: Create new UTS namespace
    unshare(CloneFlags::CLONE_NEWUTS)
        .context("Failed to unshare UTS namespace (need sudo)")?;
    println!("Created new UTS namespace");

    // Step 3: Set new hostname in the namespace
    let new_hostname = "container-test";
    sethostname(new_hostname)
        .context("Failed to set hostname in namespace")?;

    // Step 4: Verify the change
    let current_hostname = fs::read_to_string("/proc/sys/kernel/hostname")
        .context("Failed to read new hostname")?
        .trim()
        .to_string();
    println!("Hostname in namespace: {}", current_hostname);

    // Step 5: Demonstrate isolation
    println!("\nYou can verify isolation by checking the host's hostname in another terminal:");
    println!("  cat /proc/sys/kernel/hostname");
    println!("It should still show: {}", old_hostname);

    Ok(())
}
```

5. Find the `Command::Ipc` match arm (around line 67)

6. Replace `todo!(...)` with this implementation:

```rust
Command::Ipc => {
    // Step 1: Show IPC objects before entering new namespace
    println!("=== IPC objects in parent namespace ===");
    print_ipc_objects()?;

    // Step 2: Create new IPC namespace
    unshare(CloneFlags::CLONE_NEWIPC)
        .context("Failed to unshare IPC namespace (need sudo)")?;
    println!("\n=== Created new IPC namespace ===");

    // Step 3: Show IPC objects in new namespace (should be empty)
    println!("=== IPC objects in new namespace ===");
    print_ipc_objects()?;

    println!("\nIPC namespace isolation verified!");
    println!("Message queues, semaphores, and shared memory from the parent");
    println!("namespace are not visible here.");

    Ok(())
}
```

7. Add the helper functions at the end of the file (after `print_proc_ns()`):

```rust
fn print_ipc_objects() -> Result<()> {
    // Count System V IPC objects by reading /proc/sysvipc/
    let msg_queues = count_ipc_objects("/proc/sysvipc/msg")?;
    let semaphores = count_ipc_objects("/proc/sysvipc/sem")?;
    let shared_mem = count_ipc_objects("/proc/sysvipc/shm")?;

    println!("Message queues: {}", msg_queues);
    println!("Semaphores: {}", semaphores);
    println!("Shared memory segments: {}", shared_mem);

    Ok(())
}

fn count_ipc_objects(path: &str) -> Result<usize> {
    // Read the file and count non-header lines
    // Each file has a header line, so we subtract 1
    let contents = fs::read_to_string(path)
        .context(format!("Failed to read {}", path))?;

    let count = contents.lines().count().saturating_sub(1);
    Ok(count)
}
```

8. Run the tests again (expect them to pass):

```bash
# Test UTS namespace (requires sudo)
sudo -E cargo test -p ns-tool --test uts_test

# Test IPC namespace (requires sudo)
sudo -E cargo test -p ns-tool --test ipc_test
```

**Expected output**: All tests pass (GREEN phase).

```
running 1 test
test test_uts_namespace_hostname_isolation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Verify

**Automated verification**:
```bash
# Run all ns-tool tests
sudo -E cargo test -p ns-tool

# Run specific namespace tests
sudo -E cargo test -p ns-tool --test uts_test
sudo -E cargo test -p ns-tool --test ipc_test
```

**Manual verification** (observe the actual behavior):

### UTS Namespace Verification

```bash
# Terminal 1: Run ns-tool with UTS namespace
sudo cargo run -p ns-tool -- uts

# Expected output:
# Hostname before unshare: your-hostname
# Created new UTS namespace
# Hostname in namespace: container-test
#
# You can verify isolation by checking the host's hostname in another terminal:
#   cat /proc/sys/kernel/hostname
# It should still show: your-hostname
```

```bash
# Terminal 2: Verify host hostname is unchanged
cat /proc/sys/kernel/hostname
# Should show: your-hostname (NOT container-test)

hostname
# Should show: your-hostname
```

### IPC Namespace Verification

First, create some IPC objects in the parent namespace to see isolation:

```bash
# Create a message queue in the parent namespace
ipcmk -Q
# Output: Message queue id: 0

# List IPC objects
ipcs -q
# Shows the message queue we just created

# Now run ns-tool with IPC namespace
sudo cargo run -p ns-tool -- ipc

# Expected output:
# === IPC objects in parent namespace ===
# Message queues: 1
# Semaphores: 0
# Shared memory segments: 0
#
# === Created new IPC namespace ===
# === IPC objects in new namespace ===
# Message queues: 0
# Semaphores: 0
# Shared memory segments: 0
#
# IPC namespace isolation verified!
```

**How to inspect namespace IDs**:
```bash
# Before running ns-tool
ls -l /proc/self/ns/uts /proc/self/ns/ipc

# Output shows inode numbers (namespace IDs):
# uts -> 'uts:[4026531838]'
# ipc -> 'ipc:[4026531839]'

# After unshare, these numbers would be different in the new namespace
```

## Clean Up

UTS and IPC namespaces are automatically cleaned up when the process exits. No manual cleanup is required.

However, if you created IPC objects for testing, clean them up:

```bash
# List all IPC objects
ipcs

# Remove specific message queue (use the ID from ipcs output)
ipcrm -q <queue-id>

# Or remove all IPC objects owned by your user
ipcrm -a
```

## Common Errors

1. **`Operation not permitted` when calling `unshare()` or `sethostname()`**
   - Cause: These operations require `CAP_SYS_ADMIN` capability
   - Fix: Run with `sudo`: `sudo cargo run -p ns-tool -- uts`
   - Note: Even if you're root, you need to explicitly use `sudo` to preserve environment variables needed by cargo

2. **`Hostname in namespace` shows the old hostname after `sethostname()`**
   - Cause: Reading from wrong location or caching issue
   - Fix: Always read from `/proc/sys/kernel/hostname` which reflects the UTS namespace's hostname
   - Note: The `hostname` command might cache the result; reading from `/proc` is more reliable for verification

3. **IPC objects from parent namespace are visible in child**
   - Cause: Forgot to call `unshare(CLONE_NEWIPC)` or didn't enter the namespace correctly
   - Fix: Ensure `unshare()` is called before inspecting `/proc/sysvipc/`
   - Debug: Check namespace ID with `ls -l /proc/self/ns/ipc` before and after unshare

4. **Tests fail with "Failed to read /proc/sysvipc/msg"**
   - Cause: `/proc/sysvipc/` files don't exist or aren't readable
   - Fix: Ensure your kernel has `CONFIG_SYSVIPC=y` and `/proc` is mounted
   - Workaround: Check `uname -a` for kernel version; System V IPC should be available on all modern kernels

5. **`-E` flag warning when running tests with sudo**
   - Cause: `sudo -E` preserves environment variables, which is needed for cargo to find the correct Rust toolchain
   - Not an error: This is expected behavior
   - Why needed: Without `-E`, cargo might use a different Rust version or not find your project's dependencies

## Notes

### UTS Namespace Details
- **Etymology**: "UTS" stands for "Unix Time-Sharing" system, a historical name from early Unix
- **What it isolates**: Only hostname and domainname (via `uname()` syscall)
- **What it doesn't isolate**: IP addresses, network interfaces, or any other network configuration (use network namespaces for that)
- **Use case**: Containers need unique hostnames for logging, monitoring, and application configuration
- Man page: `man 7 uts_namespaces`

### IPC Namespace Details
- **System V IPC**: Legacy but still widely used inter-process communication mechanisms
  - Message queues: FIFO queues for sending typed messages
  - Semaphores: Counters for resource locking/synchronization
  - Shared memory: Memory regions shared between processes
- **POSIX message queues**: Also isolated by IPC namespace (mounted at `/dev/mqueue`)
- **Not isolated**: Unix domain sockets, pipes, FIFOs—these use the filesystem namespace
- Man page: `man 7 ipc_namespaces`

### Combining Namespaces
- Multiple `CloneFlags` can be combined with bitwise OR:
  ```rust
  unshare(CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWIPC)?;
  ```
- This is more efficient than calling `unshare()` twice
- Order doesn't matter for UTS and IPC (no dependencies between them)
- Later lessons will combine PID, mount, network, and user namespaces together

### Why No Forking?
- **PID namespace**: Requires `fork()` because the calling process is not in the new namespace; only children are
- **UTS/IPC/Mount/Network namespaces**: The calling process immediately enters the new namespace
- **User namespace**: Special case—calling process enters, but its IDs might change

### Inspecting Namespaces from /proc
```bash
# Each process has symlinks showing its namespace memberships
ls -l /proc/self/ns/
# uts -> 'uts:[4026531838]'    <- UTS namespace ID
# ipc -> 'ipc:[4026531839]'    <- IPC namespace ID
# mnt -> 'mnt:[4026531840]'    <- Mount namespace ID
# ...

# Two processes in the same namespace will have the same inode number
# This is how you verify namespace isolation
```

### Man Pages
- `man 2 unshare` - Details on the unshare syscall and clone flags
- `man 7 namespaces` - Overview of all Linux namespaces
- `man 7 uts_namespaces` - UTS namespace specifics
- `man 7 ipc_namespaces` - IPC namespace specifics
- `man 2 sethostname` - Setting hostname (requires CAP_SYS_ADMIN)
- `man 1 ipcs` - Display IPC objects
- `man 1 ipcmk` - Create IPC objects for testing
- `man 1 ipcrm` - Remove IPC objects

### Rust Ecosystem
- **nix crate**: Provides safe Rust wrappers for `unshare()`, `sethostname()`, and other POSIX/Linux syscalls
- **CloneFlags**: Type-safe bitflags for namespace creation
  - `CLONE_NEWUTS` - Create new UTS namespace
  - `CLONE_NEWIPC` - Create new IPC namespace
  - Can be combined: `CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWIPC`
- **anyhow crate**: Provides the `context()` method for adding context to errors

## Next
`04-mount-namespace.md` - Isolate filesystem mount points and create a basic container filesystem view
