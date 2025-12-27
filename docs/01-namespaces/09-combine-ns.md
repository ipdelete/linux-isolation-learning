# Combining Multiple Namespaces

## Goal

Create a fully isolated container-like environment by combining multiple namespace types (PID, UTS, IPC, mount, and network) into a single process. You will implement the `container` subcommand that creates comprehensive isolation comparable to what container runtimes like Docker provide.

**What you will build**: A `ns-tool container` command that creates a process isolated in all major namespace types, with its own hostname, network stack, filesystem view, and process tree.

**Estimated time**: 45-50 minutes

## Prereqs

- Completed `08-netns-nat.md` (or at minimum: `01-pid-namespace.md`, `02-uts-namespace.md`, `04-mount-namespace.md`, and `05-network-namespace.md`)
- Understanding of each individual namespace type and what it isolates
- `sudo` access (required for namespace creation and network setup)
- Familiarity with `unshare()` and `fork()` patterns from previous lessons

## Concepts: Why Combine Namespaces?

Before writing code, let's understand what container-like isolation really means and how combining namespaces achieves it.

### What is a Container?

A **container** is an isolated execution environment that combines multiple Linux isolation primitives:

```
+--------------------------------------------------+
|  Container Process                                |
|                                                   |
|  PID namespace:    Own process tree (PID 1)      |
|  UTS namespace:    Custom hostname               |
|  IPC namespace:    Separate message queues       |
|  Mount namespace:  Private filesystem view       |
|  Network namespace: Isolated network stack       |
|  (+ cgroups for resource limits)                 |
+--------------------------------------------------+
```

Each namespace type contributes a different dimension of isolation:

| Namespace | What It Isolates | Why It Matters |
|-----------|------------------|----------------|
| **PID** | Process IDs and tree | Process inside cannot see/signal host processes |
| **UTS** | Hostname and domain | Container has its own identity |
| **IPC** | Shared memory, semaphores | Prevents IPC-based attacks between containers |
| **Mount** | Filesystem view | Container has its own `/proc`, `/sys`, etc. |
| **Network** | Network interfaces, routing | Container has isolated network stack |

Without combining these, you have partial isolation. For example:
- PID namespace alone: Isolated processes, but shares network and filesystem
- Network namespace alone: Isolated network, but can see all host processes

### The Power of Combining: Multiple Flags in One Call

Linux allows you to create multiple namespaces with a single `unshare()` syscall by OR'ing flags together:

```rust
use nix::sched::{unshare, CloneFlags};

// Create all five namespace types at once
unshare(
    CloneFlags::CLONE_NEWPID |    // Process isolation
    CloneFlags::CLONE_NEWUTS |    // Hostname isolation
    CloneFlags::CLONE_NEWIPC |    // IPC isolation
    CloneFlags::CLONE_NEWNS |     // Mount isolation
    CloneFlags::CLONE_NEWNET      // Network isolation
)?;
```

This is more efficient than calling `unshare()` five separate times, and ensures atomicity - either all namespaces are created or none are.

### Setup Order Matters

When combining namespaces, some operations must happen in a specific sequence:

1. **Call `unshare()` first**: Creates all namespace types
2. **Fork for PID namespace**: Child becomes PID 1 in new namespace
3. **Set up mount namespace**: Make mounts private, create new proc
4. **Configure network**: Set up loopback interface
5. **Set hostname**: Configure UTS namespace identity
6. **Exec shell**: Give user an interactive environment

**Why this order?**

- PID namespace only takes effect after `fork()` - the child is PID 1
- Mount operations need private propagation to avoid leaking to parent
- Network setup requires root in the namespace (before dropping privileges)
- Hostname is cosmetic, can be set anytime before exec

### What "Container-like" Means

After completing this lesson, you'll have a process that:

1. **Cannot see host processes** (check with `ps aux`)
2. **Has its own hostname** (check with `hostname`)
3. **Has isolated network** (check with `ip addr` - only loopback initially)
4. **Has private mount namespace** (check with `findmnt`)
5. **Has separate IPC resources** (check with `ipcs`)

This is the foundation of container isolation. Later lessons add:
- User namespaces (UID/GID mapping)
- Cgroups (resource limits)
- Seccomp (syscall filtering)
- Capabilities (privilege restriction)

But the namespace combination you build here is the core isolation mechanism.

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/container_test.rs`

This test file doesn't exist yet - you'll create it from scratch. This test is more complex than previous ones because we need to verify multiple namespace types are active.

### What the Tests Should Verify

1. **Success case**: `ns-tool container` should:
   - Create a new process in all namespace types
   - Report different namespace inodes than the parent
   - Successfully configure basic environment (hostname, network)
   - Exit cleanly

2. **Error case**:
   - Fail with a clear error if not run as root
   - Handle errors during setup gracefully

### Steps

1. Create the new test file:

```bash
touch crates/ns-tool/tests/container_test.rs
```

2. Open `crates/ns-tool/tests/container_test.rs` in your editor and add the test structure:

```rust
// Tests for the `container` subcommand (combined namespaces)
// Lesson: docs/01-namespaces/09-combine-ns.md
//
// This test verifies that combining multiple namespace types creates
// full container-like isolation.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
#[ignore]  // Requires root - run with: sudo -E cargo test -- --ignored
fn test_container_creates_multiple_namespaces() {
    // This test verifies that the container subcommand creates a process
    // with different namespace inodes than the parent for all namespace types.
    //
    // We'll run a command inside the container that prints namespace info,
    // then compare to the parent's namespace inodes.

    // Get parent's PID namespace inode for comparison
    let parent_pid_ns = fs::read_link("/proc/self/ns/pid")
        .expect("failed to read parent PID namespace");
    let parent_uts_ns = fs::read_link("/proc/self/ns/uts")
        .expect("failed to read parent UTS namespace");
    let parent_net_ns = fs::read_link("/proc/self/ns/net")
        .expect("failed to read parent network namespace");

    // Run container command with a script that prints namespace info
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("container")
        .arg("--")
        .arg("/bin/sh")
        .arg("-c")
        // Inside container: print namespace inodes
        .arg("readlink /proc/self/ns/pid && readlink /proc/self/ns/uts && readlink /proc/self/ns/net")
        .assert()
        .success();

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify that namespace inodes are different from parent
    // (This proves we're in new namespaces)
    assert!(
        !stdout.contains(&parent_pid_ns.to_string_lossy().to_string()),
        "PID namespace should be different from parent"
    );
    assert!(
        !stdout.contains(&parent_uts_ns.to_string_lossy().to_string()),
        "UTS namespace should be different from parent"
    );
    assert!(
        !stdout.contains(&parent_net_ns.to_string_lossy().to_string()),
        "Network namespace should be different from parent"
    );
}

#[test]
#[ignore]  // Requires root - run with: sudo -E cargo test -- --ignored
fn test_container_has_isolated_hostname() {
    // Verify that the container can set its own hostname without affecting parent

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("container")
        .arg("--hostname")
        .arg("isolated-container")
        .arg("--")
        .arg("/bin/sh")
        .arg("-c")
        .arg("hostname")
        .assert()
        .success()
        .stdout(predicate::str::contains("isolated-container"));
}

#[test]
#[ignore]  // Requires root - run with: sudo -E cargo test -- --ignored
fn test_container_has_pid_1() {
    // Verify that the container's init process has PID 1

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("container")
        .arg("--")
        .arg("/bin/sh")
        .arg("-c")
        .arg("echo $$")  // Print shell's PID
        .assert()
        .success()
        .stdout(predicate::str::contains("1\n"));
}

#[test]
#[ignore]  // Requires root - run with: sudo -E cargo test -- --ignored
fn test_container_has_isolated_network() {
    // Verify that the container has only loopback interface (no host interfaces)

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("container")
        .arg("--")
        .arg("/bin/sh")
        .arg("-c")
        .arg("ip link show")
        .assert()
        .success()
        .stdout(predicate::str::contains("lo"))  // Has loopback
        .stdout(predicate::str::contains("UP"));  // Loopback is up

    // Should NOT see typical host interfaces like eth0, ens33, etc.
    // (This is less reliable to test, so we just verify loopback exists)
}

#[test]
fn test_container_without_root_fails() {
    // Run without sudo - should fail with permission error
    // This test does NOT need root, so it's not marked #[ignore]

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    // Note: If actually run as root, this test may pass unexpectedly
    // In practice, CI systems and dev environments typically run tests as non-root
    if nix::unistd::Uid::effective().is_root() {
        // Skip this test if running as root
        return;
    }

    cmd.arg("container")
        .arg("--")
        .arg("/bin/sh")
        .arg("-c")
        .arg("echo test")
        .assert()
        .failure();
}
```

3. Run the tests (expect them to fail because implementation doesn't exist yet):

```bash
# Non-root test (should fail because container subcommand doesn't exist)
cargo test -p ns-tool --test container_test

# Root tests (should also fail for same reason)
sudo -E cargo test -p ns-tool --test container_test -- --ignored
```

**Expected output**:

```
error: could not compile `ns-tool` due to previous error
```

Or if it compiles:

```
test test_container_without_root_fails ... FAILED
test test_container_creates_multiple_namespaces ... FAILED (ignored)
test test_container_has_isolated_hostname ... FAILED (ignored)
test test_container_has_pid_1 ... FAILED (ignored)
test test_container_has_isolated_network ... FAILED (ignored)
```

This is the **RED** phase - tests exist but fail because implementation is missing.

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`
**New function**: We'll add a `Container` variant to the `Command` enum and implement `run_container()`

### Implementation Strategy

We need to:

1. Add `Container` variant to the `Command` enum
2. Handle additional arguments (hostname, command to run)
3. Implement the container creation logic:
   - Call `unshare()` with all namespace flags
   - Fork a child process
   - In the child (which becomes PID 1):
     - Set up mount namespace (remount proc)
     - Set up network (bring up loopback)
     - Set hostname
     - Exec the specified command

### Steps

1. Open `crates/ns-tool/src/main.rs` and find the `Command` enum (around line 15):

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
    // ADD THIS:
    /// Create a container-like environment with multiple namespaces
    Container {
        /// Hostname for the container
        #[arg(long, default_value = "container")]
        hostname: String,

        /// Command to run in container (default: /bin/sh)
        #[arg(last = true)]
        command: Vec<String>,
    },
}
```

2. In the `main()` function's match statement, add the `Container` arm before `Command::Proc`:

```rust
        Command::Container { hostname, command } => {
            run_container(&hostname, &command)?;
        }
```

3. Add the `run_container()` function at the end of the file (after `print_proc_ns()`):

```rust
fn run_container(hostname: &str, command: &[String]) -> Result<()> {
    use nix::sched::{unshare, CloneFlags};
    use nix::unistd::{fork, ForkResult, sethostname, execvp, getpid};
    use nix::mount::{mount, MsFlags};
    use nix::sys::wait::{waitpid, WaitStatus};
    use std::ffi::CString;

    // Verify we're running as root
    if !nix::unistd::Uid::effective().is_root() {
        anyhow::bail!("container subcommand requires root privileges (run with sudo)");
    }

    println!("Creating container with combined namespaces...");

    // Step 1: Create all namespaces at once
    unshare(
        CloneFlags::CLONE_NEWPID |    // Process isolation
        CloneFlags::CLONE_NEWUTS |    // Hostname isolation
        CloneFlags::CLONE_NEWIPC |    // IPC isolation
        CloneFlags::CLONE_NEWNS |     // Mount isolation
        CloneFlags::CLONE_NEWNET      // Network isolation
    ).context("failed to create namespaces")?;

    println!("Namespaces created (PID, UTS, IPC, Mount, Network)");

    // Step 2: Fork - child becomes PID 1 in new PID namespace
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            // Parent: wait for child to complete
            println!("Parent: spawned container process (PID {})", child);

            match waitpid(child, None)? {
                WaitStatus::Exited(_, code) => {
                    println!("Container exited with code {}", code);
                    if code != 0 {
                        std::process::exit(code);
                    }
                }
                WaitStatus::Signaled(_, sig, _) => {
                    println!("Container killed by signal {:?}", sig);
                    std::process::exit(1);
                }
                _ => {}
            }
        }
        ForkResult::Child => {
            // We are now PID 1 in the new PID namespace!
            println!("Child: I am PID {} (should be 1)", getpid());

            // Step 3: Set up mount namespace
            // Make all mounts private to prevent propagation to parent
            mount(
                None::<&str>,
                "/",
                None::<&str>,
                MsFlags::MS_PRIVATE | MsFlags::MS_REC,
                None::<&str>,
            ).context("failed to make mounts private")?;

            // Mount a new /proc for this namespace
            mount(
                Some("proc"),
                "/proc",
                Some("proc"),
                MsFlags::empty(),
                None::<&str>,
            ).context("failed to mount /proc")?;

            println!("Mount namespace configured (private mounts, new /proc)");

            // Step 4: Set up network namespace
            // Bring up the loopback interface
            setup_loopback().context("failed to setup loopback interface")?;
            println!("Network namespace configured (loopback up)");

            // Step 5: Set hostname in UTS namespace
            sethostname(hostname).context("failed to set hostname")?;
            println!("Hostname set to: {}", hostname);

            // Step 6: Prepare command to execute
            let default_cmd = vec!["/bin/sh".to_string()];
            let cmd_to_run = if command.is_empty() {
                &default_cmd
            } else {
                command
            };

            println!("Executing: {:?}", cmd_to_run);
            println!("---");

            // Step 7: Exec into the command
            // Convert to CStrings for execvp
            let cmd_cstrings: Vec<CString> = cmd_to_run
                .iter()
                .map(|s| CString::new(s.as_str()).unwrap())
                .collect();

            // execvp replaces this process - if it returns, something went wrong
            execvp(&cmd_cstrings[0], &cmd_cstrings)
                .context("failed to exec command")?;

            // Unreachable if exec succeeds
            unreachable!("exec should not return");
        }
    }

    Ok(())
}

fn setup_loopback() -> Result<()> {
    use nix::sys::socket::{socket, AddressFamily, SockType, SockFlag};
    use std::os::unix::io::AsRawFd;

    // We need to use ioctl to bring up the loopback interface
    // This is lower-level than nix provides, so we use libc

    let sock = socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    ).context("failed to create socket")?;

    unsafe {
        // Prepare ifreq structure for loopback
        let mut ifr: libc::ifreq = std::mem::zeroed();

        // Set interface name to "lo"
        let ifname = b"lo\0";
        ifr.ifr_name[..3].copy_from_slice(
            &std::mem::transmute::<[u8; 3], [libc::c_char; 3]>(*b"lo\0")
        );

        // Get current flags
        if libc::ioctl(sock.as_raw_fd(), libc::SIOCGIFFLAGS, &mut ifr) == -1 {
            return Err(anyhow::anyhow!("SIOCGIFFLAGS failed: {}",
                std::io::Error::last_os_error()));
        }

        // Set UP and RUNNING flags
        ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as libc::c_short;

        // Apply the flags
        if libc::ioctl(sock.as_raw_fd(), libc::SIOCSIFFLAGS, &ifr) == -1 {
            return Err(anyhow::anyhow!("SIOCSIFFLAGS failed: {}",
                std::io::Error::last_os_error()));
        }
    }

    Ok(())
}
```

4. Add required dependencies to `crates/ns-tool/Cargo.toml` if not already present:

```bash
# Check current dependencies
cat crates/ns-tool/Cargo.toml
```

Ensure these are in the `[dependencies]` section:
- `nix = { version = "0.27", features = ["process", "sched", "mount", "socket", "user"] }`
- `anyhow = "1.0"`
- `clap = { version = "4.4", features = ["derive"] }`

5. Build the code:

```bash
cargo build -p ns-tool
```

**Expected output**: Successful compilation. If you get errors, check:
- All imports are correct
- The `Command` enum variant is properly added
- The `setup_loopback()` function is defined
- All braces and parentheses match

6. Run the tests again:

```bash
# First, run non-root test
cargo test -p ns-tool --test container_test

# Then run root tests
sudo -E cargo test -p ns-tool --test container_test -- --ignored
```

**Expected output**: Tests should now pass (GREEN phase):

```
running 5 tests
test test_container_without_root_fails ... ok
test test_container_creates_multiple_namespaces ... ok (ignored, requires root)
test test_container_has_isolated_hostname ... ok (ignored, requires root)
test test_container_has_pid_1 ... ok (ignored, requires root)
test test_container_has_isolated_network ... ok (ignored, requires root)

test result: ok. 5 passed; 0 failed; 0 ignored
```

## Verify

### Automated Verification

```bash
# All container tests should pass
sudo -E cargo test -p ns-tool --test container_test -- --ignored
```

Expected:
```
running 5 tests
test test_container_creates_multiple_namespaces ... ok
test test_container_has_isolated_hostname ... ok
test test_container_has_pid_1 ... ok
test test_container_has_isolated_network ... ok
test test_container_without_root_fails ... ok

test result: ok. 5 passed; 0 failed
```

### Manual Verification

Now let's observe the actual container-like isolation behavior.

#### 1. Basic Container Launch

```bash
sudo cargo run -p ns-tool -- container
```

You should see:
```
Creating container with combined namespaces...
Namespaces created (PID, UTS, IPC, Mount, Network)
Parent: spawned container process (PID 12345)
Child: I am PID 1 (should be 1)
Mount namespace configured (private mounts, new /proc)
Network namespace configured (loopback up)
Hostname set to: container
Executing: ["/bin/sh"]
---
# <-- You're now in a shell inside the container
```

#### 2. Verify PID Isolation

Inside the container shell:

```bash
# Check your PID (should be 1)
echo $$
# Output: 1

# List all processes (should only see processes in this namespace)
ps aux
# Output: Shows only the shell and ps command - NOT host processes!

# Compare to parent's view
# In another terminal on the host:
ps aux | wc -l
# Output: Many processes (all host processes)
```

#### 3. Verify Hostname Isolation

Inside the container:

```bash
# Check hostname
hostname
# Output: container

# Set a different hostname (only affects this namespace)
hostname my-isolated-container
hostname
# Output: my-isolated-container
```

On the host (in another terminal):
```bash
hostname
# Output: <your actual hostname> (unchanged!)
```

#### 4. Verify Network Isolation

Inside the container:

```bash
# List network interfaces
ip addr show
# Output: Only shows 'lo' (loopback) - no eth0, ens33, etc.

# Verify loopback works
ping -c 1 127.0.0.1
# Output: 1 packet transmitted, 1 received, 0% packet loss

# Try to ping external (will fail - no route to outside)
ping -c 1 8.8.8.8
# Output: Network unreachable (expected - isolated network)
```

On the host:
```bash
ip addr show
# Output: All your normal interfaces (eth0, wlan0, etc.)
```

#### 5. Verify Mount Isolation

Inside the container:

```bash
# Check /proc - should show only container processes
ls -la /proc
cat /proc/mounts | grep proc
# Output: Shows proc mounted at /proc

# Mount points are isolated
findmnt | head -20
```

#### 6. Verify Namespace Inodes

Inside the container:

```bash
# List all namespace inodes
ls -la /proc/self/ns/
# Output: All symlinks point to different inodes than parent
```

On the host:
```bash
ls -la /proc/self/ns/
# Output: Different inode numbers than container
```

#### 7. Custom Hostname Container

```bash
sudo cargo run -p ns-tool -- container --hostname "rust-container"
```

Inside:
```bash
hostname
# Output: rust-container
```

#### 8. Run Specific Command

```bash
sudo cargo run -p ns-tool -- container -- /bin/sh -c 'hostname && ip addr && echo "PID: $$"'
```

Expected output:
```
container
1: lo: <LOOPBACK,UP,LOWER_UP> ...
    inet 127.0.0.1/8 ...
PID: 1
```

#### 9. Inspect Namespace Differences

Compare namespace inodes between parent and container:

```bash
# On host: Get namespace inodes
readlink /proc/self/ns/{pid,uts,ipc,mnt,net}

# Inside container: Get namespace inodes
sudo cargo run -p ns-tool -- container -- /bin/sh -c 'readlink /proc/self/ns/{pid,uts,ipc,mnt,net}'

# All inodes should be different!
```

## Clean Up

Exit the container shell:

```bash
# Inside container:
exit
```

The container process terminates and all namespaces are automatically cleaned up by the kernel. No manual cleanup needed!

**Verify cleanup**:

```bash
# Check that no orphaned namespaces remain
# (If container exited cleanly, there should be none)
lsns | grep container
# Output: (empty if cleanup worked)
```

**Note**: Unlike previous lessons with network namespaces, combined namespaces clean up automatically when the last process exits. The kernel tracks namespace reference counts and removes them when they reach zero.

## Common Errors

### 1. `Operation not permitted (os error 1)` when creating namespaces

**Symptom**:
```
Error: failed to create namespaces

Caused by:
    Operation not permitted (os error 1)
```

**Cause**: Not running with root privileges. Namespace creation requires `CAP_SYS_ADMIN`.

**Fix**: Run with `sudo`:
```bash
sudo cargo run -p ns-tool -- container
```

For tests:
```bash
sudo -E cargo test -p ns-tool --test container_test -- --ignored
```

### 2. `failed to mount /proc: Device or resource busy`

**Symptom**:
```
Error: failed to mount /proc

Caused by:
    Device or resource busy (os error 16)
```

**Cause**: The mount namespace wasn't properly isolated, or `/proc` is already mounted and we're trying to mount over it without proper flags.

**Fix**: Ensure the `MS_PRIVATE | MS_REC` flags are set before mounting `/proc`:
```rust
mount(
    None::<&str>,
    "/",
    None::<&str>,
    MsFlags::MS_PRIVATE | MsFlags::MS_REC,
    None::<&str>,
)?;
```

This makes all mounts private so changes don't propagate to parent namespace.

### 3. `PID is not 1 inside container`

**Symptom**: Tests fail because `echo $$` prints a PID other than 1.

**Cause**: The PID namespace only takes effect after `fork()`. If you're checking PID before forking, you'll see the parent's PID.

**Fix**: Ensure you're checking PID in the **child** process, after the `fork()` call:
```rust
ForkResult::Child => {
    println!("My PID: {}", getpid());  // This should be 1
    // ... rest of setup
}
```

### 4. `Loopback interface not UP`

**Symptom**: Inside container, `ip addr show lo` shows loopback as `DOWN`.

**Cause**: The `setup_loopback()` function failed silently, or the ioctl call didn't properly set flags.

**Fix**: Check that the ioctl calls succeed and properly set `IFF_UP | IFF_RUNNING`:
```rust
ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as libc::c_short;
```

Debug by adding `println!()` statements in `setup_loopback()` to see where it fails.

### 5. `Container exits immediately without showing shell`

**Symptom**: The container prints setup messages but immediately exits without giving you a shell prompt.

**Cause**: The parent process isn't waiting for the child, or `execvp()` is failing.

**Fix**:
1. Check that parent has `waitpid()` call
2. Verify that `/bin/sh` exists and is executable
3. Try specifying an absolute path: `/bin/sh` not just `sh`

### 6. `Tests pass but manual verification shows wrong behavior`

**Symptom**: Tests pass but when you run manually, you can still see host processes or network interfaces.

**Cause**: Test might be checking the wrong thing, or manual verification is running without `sudo`.

**Fix**:
- Ensure manual verification uses `sudo`
- Check that you're actually inside the container when testing (look at the prompt)
- Verify namespace inodes differ: `readlink /proc/self/ns/pid`

### 7. `Container cannot resolve hostnames or access network`

**Symptom**: This is **expected behavior**, not an error!

**Explanation**: The container has an isolated network namespace with only a loopback interface. It has no connection to the outside world. This is by design for this lesson.

**Future lessons**: Later lessons (`07-veth-bridge.md` and `08-netns-nat.md`) show how to connect container network namespaces to the host network using veth pairs, bridges, and NAT.

## Notes

### Why This Order of Operations?

The setup sequence in `run_container()` is carefully ordered:

1. **`unshare()` first**: Must create all namespaces before fork
2. **`fork()` second**: PID namespace only affects children
3. **Mount setup**: Must happen before other operations need new `/proc`
4. **Network setup**: Needs to happen in the child (PID 1) with proper privileges
5. **Hostname**: Can happen anytime but logical after network
6. **`exec()`**: Last operation, replaces process image

Changing this order can cause subtle failures. For example:
- Network setup before fork: Affects parent's network (bad!)
- exec before mount setup: New process can't see new `/proc`

### Understanding the Fork Dance

```rust
match unsafe { fork()? } {
    ForkResult::Parent { child } => {
        // Parent: runs in original PID namespace
        // Just waits for child to exit
    }
    ForkResult::Child => {
        // Child: runs as PID 1 in new PID namespace
        // Performs all setup and exec
    }
}
```

**Why is fork unsafe?**

In Rust, `fork()` is marked unsafe because:
- It creates a new process with shared memory (before exec)
- Multi-threaded programs can deadlock if other threads held locks
- File descriptors are inherited in complex ways

We can safely call it here because:
- Our program is single-threaded at fork time
- We immediately exec in the child (doesn't touch shared state)
- We properly wait in the parent

### Loopback Setup: Why Manual ioctl?

The `setup_loopback()` function uses raw ioctl calls because:

1. **nix doesn't wrap network interface configuration**: The `nix` crate focuses on portable POSIX APIs. Network interface configuration is very Linux-specific.

2. **The ioctl approach is standard**: This is how all network configuration tools (ip, ifconfig) work under the hood.

3. **It's safe within bounds**: The unsafe block is minimal - just the ioctl syscall. We've validated the interface name and flags.

If you wanted to avoid this unsafe code, you could:
- Shell out to `ip link set lo up` (but that's slower and requires ip command)
- Use the `neli` crate (netlink library) - more complex but more "Rust-y"
- Use `rtnetlink` crate for async netlink

For educational purposes, the ioctl approach is most direct.

### Comparing to Docker

What you've built is similar to Docker's core isolation, but simpler:

| Feature | Your Container | Docker Container |
|---------|----------------|------------------|
| PID namespace | Yes | Yes |
| UTS namespace | Yes | Yes |
| IPC namespace | Yes | Yes |
| Mount namespace | Yes | Yes (+ layered FS) |
| Network namespace | Yes (isolated) | Yes (+ veth to host) |
| User namespace | No (root in container) | Optional (UID mapping) |
| Cgroups | No | Yes (resource limits) |
| Seccomp | No | Yes (syscall filtering) |
| Capabilities | No (full root) | Yes (dropped caps) |
| Layered filesystem | No (shared /) | Yes (overlay/aufs) |

Future lessons add these missing pieces. But the namespace combination you've built here is the foundation.

### Manual Testing vs Automated Testing

You might notice the tests are less comprehensive than the manual verification. This is intentional:

**Automated tests verify**:
- Basic functionality (namespaces are created)
- Critical invariants (different inodes, PID is 1)
- Error handling (fails without root)

**Manual verification confirms**:
- User-facing behavior (shell prompt, commands work)
- System integration (process list, network interfaces)
- Practical usability (can actually use the container)

Good systems software tests combine both approaches. Automated tests catch regressions; manual verification proves the system actually works as intended.

### Links to Official Documentation

- [namespaces(7) man page](https://man7.org/linux/man-pages/man7/namespaces.7.html) - Overview of all namespace types
- [unshare(2) man page](https://man7.org/linux/man-pages/man2/unshare.2.html) - System call documentation
- [mount_namespaces(7)](https://man7.org/linux/man-pages/man7/mount_namespaces.7.html) - Mount namespace details
- [pid_namespaces(7)](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html) - PID namespace details
- [network_namespaces(7)](https://man7.org/linux/man-pages/man7/network_namespaces.7.html) - Network namespace details
- [nix::sched documentation](https://docs.rs/nix/latest/nix/sched/index.html) - Rust API reference

## Summary

In this lesson, you learned:

1. **Container fundamentals**: How combining namespaces creates container-like isolation
2. **Namespace combinations**: Using OR'ed flags to create multiple namespaces atomically
3. **Setup sequencing**: Why the order of operations matters (unshare → fork → setup → exec)
4. **Complete isolation**: How PID + UTS + IPC + Mount + Network namespaces work together
5. **Testing strategies**: Verifying multi-dimensional isolation with automated tests
6. **Practical containers**: Building a minimal but functional container environment

You've built a process that:
- Cannot see or signal host processes
- Has its own hostname and identity
- Uses isolated IPC resources
- Has a private filesystem view
- Operates in an isolated network

This is the core of containerization. Every container runtime (Docker, containerd, runc) builds on this foundation.

## Next

`10-join-existing.md` - Learn how to join existing namespaces using `setns()`, enabling multi-process containers and debugging running containers.
