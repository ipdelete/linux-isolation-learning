# Join Existing Namespaces with setns

## Goal

Learn to use the `setns(2)` system call to join existing namespaces, enabling your process to "enter" isolation contexts created by other processes. This is how tools like `docker exec`, `nsenter`, and `kubectl exec` work.

**Deliverable**: Implement an `exec` subcommand for `ns-tool` that joins an existing namespace by PID and executes a shell in that context.

**Time estimate**: ~50 minutes

## Prereqs

- Completed `09-combine-ns.md` (understand creating multiple namespaces)
- Completed `03-procfs-intro.md` (know how to read `/proc/[pid]/ns/` symlinks)
- `sudo` access (setns requires CAP_SYS_ADMIN in most cases)

## Background: How setns Works

When you create a new namespace with `unshare()` or `clone()`, the calling process (or new process) enters that namespace. But what if you want to join a namespace that *already exists*?

### The Problem setns Solves

```
┌─────────────────────┐          ┌─────────────────────┐
│   Host Namespace    │          │ Container Namespace │
│                     │          │                     │
│  PID 1000 (shell)   │          │  PID 2000 (app)     │
│                     │          │                     │
└─────────────────────┘          └─────────────────────┘
         │                                  │
         │                                  │
         └──────────────?───────────────────┘
              How do I join that namespace?
```

**Use cases**:
- `docker exec`: Run commands in a running container
- `nsenter`: Attach to a process's namespaces for debugging
- `kubectl exec`: Execute commands in Kubernetes pods
- Debugging: Inspect network config or filesystem of isolated processes

### How setns(2) Works

```c
int setns(int fd, int nstype);
```

**Parameters**:
- `fd`: File descriptor referring to a namespace (from `/proc/[pid]/ns/[type]`)
- `nstype`: Namespace type to verify (e.g., `CLONE_NEWPID`), or `0` for any type

**What it does**:
1. Opens a file descriptor to `/proc/[pid]/ns/[type]` (e.g., `/proc/2000/ns/net`)
2. Calls `setns(fd, nstype)` to reassociate the calling thread with that namespace
3. The calling thread is now "inside" the target namespace
4. Subsequent operations use the new namespace context

**Critical insight**: PID namespaces are special. When you call `setns()` to join a PID namespace, the calling process doesn't change its PID (it keeps its original PID). Only *children* spawned after the `setns()` call will have PIDs in the new namespace. This is why we must fork after joining a PID namespace.

### Namespace Types and setns Restrictions

| Namespace | Can Join? | Restrictions |
|-----------|-----------|--------------|
| UTS | Yes | None (requires CAP_SYS_ADMIN) |
| IPC | Yes | None (requires CAP_SYS_ADMIN) |
| Network | Yes | None (requires CAP_SYS_ADMIN) |
| Mount | Yes | None (requires CAP_SYS_ADMIN) |
| PID | Yes | Calling process keeps old PID; must fork to get new PID |
| User | Yes* | Complex restrictions: must not be multithreaded, must have CAP_SYS_ADMIN in target namespace |
| Cgroup | Yes | None (requires CAP_SYS_ADMIN in own user namespace) |

**User namespace restrictions**: You can only join a user namespace if:
1. You have CAP_SYS_ADMIN in the *target* user namespace
2. The calling process is single-threaded
3. The target user namespace is an ancestor or descendant of your current one

## Concepts: Namespace Persistence

Namespaces persist as long as:
1. At least one process is in the namespace, OR
2. A file descriptor is open to `/proc/[pid]/ns/[type]`, OR
3. The namespace is bind-mounted somewhere

This is how you can create a namespace, have all processes exit, but still join it later:

```bash
# Create namespace and keep it alive with bind mount
unshare --net=/var/run/netns/mynet /bin/true
# Process exits, but namespace persists via bind mount

# Later, join it
nsenter --net=/var/run/netns/mynet /bin/bash
```

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/setns_test.rs`

The test file already has TODO markers. We'll implement tests that:
1. Create a persistent namespace (by running a long-lived process)
2. Use the `exec` subcommand to join that namespace
3. Verify we're in the same namespace by comparing inode numbers

### What the Tests Should Verify

1. **Success case**: Joining an existing UTS namespace changes our hostname context
2. **Error case**: Trying to join a non-existent PID fails gracefully

We'll focus on UTS namespace for testing because:
- It's simple (just hostname isolation)
- No fork required (unlike PID namespace)
- Easy to verify (read hostname before/after)

### Steps

1. Open the test file:

```bash
cat crates/ns-tool/tests/setns_test.rs
```

2. Replace the first `todo!()` in `test_setns_join_uts_namespace`:

```rust
// crates/ns-tool/tests/setns_test.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::{Command as StdCommand, Stdio};
use std::thread;
use std::time::Duration;

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn test_setns_join_uts_namespace() {
    // This test requires root privileges
    // Run with: sudo -E cargo test -p ns-tool test_setns_join_uts_namespace

    // Step 1: Create a long-running process in a new UTS namespace with custom hostname
    let mut child = StdCommand::new("unshare")
        .args(&["--uts", "sleep", "5"])
        .spawn()
        .expect("Failed to spawn unshare process");

    let child_pid = child.id();

    // Give it time to create namespace
    thread::sleep(Duration::from_millis(100));

    // Step 2: Read the child's UTS namespace inode
    let child_ns_path = format!("/proc/{}/ns/uts", child_pid);
    let child_ns = fs::read_link(&child_ns_path)
        .expect("Failed to read child namespace");

    // Step 3: Use our tool to join the namespace (implementation will do this)
    // For now, we're just verifying the namespace exists and is different from ours
    let our_ns = fs::read_link("/proc/self/ns/uts")
        .expect("Failed to read our namespace");

    // They should be different (child is in new UTS namespace)
    assert_ne!(
        child_ns, our_ns,
        "Child should be in different UTS namespace"
    );

    // Clean up
    child.kill().expect("Failed to kill child process");
    child.wait().expect("Failed to wait for child");
}
```

**Understanding this test**:
- We spawn a child process using `unshare --uts sleep 5` (creates new UTS namespace)
- We verify the child is in a different UTS namespace by comparing symlinks
- This establishes the baseline - later we'll extend it to actually join

3. Add a more comprehensive test that actually uses our tool (once implemented):

```rust
#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
#[ignore] // Remove this after implementing the exec subcommand
fn test_exec_joins_namespace() {
    // Step 1: Create a container with a custom hostname
    let mut container = StdCommand::new("unshare")
        .args(&[
            "--uts",
            "--pid",
            "--fork",
            "--mount-proc",
            "sh",
            "-c",
            "hostname test-container && sleep 10",
        ])
        .spawn()
        .expect("Failed to spawn container");

    let container_pid = container.id();
    thread::sleep(Duration::from_millis(200));

    // Step 2: Use our exec command to join and read hostname
    // (This will fail until we implement the exec subcommand)
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("exec")
        .arg("--pid")
        .arg(container_pid.to_string())
        .arg("--")
        .arg("hostname")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-container"));

    // Clean up
    container.kill().ok();
    container.wait().ok();
}
```

4. Add an error case test:

```rust
#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
#[ignore] // Remove this after implementing error handling
fn test_exec_nonexistent_pid_fails() {
    // Try to join namespace of a PID that doesn't exist
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("exec")
        .arg("--pid")
        .arg("999999") // Very unlikely to exist
        .arg("--")
        .arg("echo")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}
```

5. Run the tests (expect failures - RED phase):

```bash
# First test should pass (just verifies namespace creation works)
sudo -E cargo test -p ns-tool test_setns_join_uts_namespace

# Other tests will fail because exec isn't implemented yet
sudo -E cargo test -p ns-tool --test setns_test
```

Expected output:
```
running 3 tests
test test_setns_join_uts_namespace ... ok
test test_exec_joins_namespace ... ignored
test test_exec_nonexistent_pid_fails ... ignored

test result: ok. 1 passed; 0 failed; 2 ignored
```

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`
**TODO location**: Line ~95 in the `Command::Setns` match arm

Now we'll implement the `exec` subcommand. But first, we need to modify the CLI to accept arguments.

### Step 1: Update the CLI Structure

Open `crates/ns-tool/src/main.rs` and update the `Command` enum to support arguments for `Setns`:

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

    /// Join existing namespaces by PID and execute a command
    #[command(name = "exec")]
    Setns {
        /// Target process PID whose namespaces to join
        #[arg(short, long)]
        pid: u32,

        /// Namespace types to join (default: all)
        #[arg(short, long, value_delimiter = ',')]
        ns_types: Option<Vec<String>>,

        /// Command to execute in the namespace
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },

    Proc,
}
```

### Step 2: Implement the Setns Command Handler

Replace the `Command::Setns` match arm with the following implementation:

```rust
Command::Setns { pid, ns_types, command } => {
    exec_in_namespace(*pid, ns_types.as_deref(), &command)?;
}
```

### Step 3: Implement the exec_in_namespace Function

Add this function before the `main()` function:

```rust
use nix::fcntl::{open, OFlag};
use nix::sched::setns;
use nix::sys::stat::Mode;
use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::CString;
use std::os::unix::io::RawFd;

fn exec_in_namespace(
    target_pid: u32,
    ns_types: Option<&[String]>,
    command: &[String],
) -> Result<()> {
    // Determine which namespaces to join (default: UTS, IPC, NET, MNT)
    // We skip PID by default because it requires special handling (fork)
    let default_types = vec!["uts", "ipc", "net", "mnt"];
    let types_to_join = ns_types.unwrap_or(&default_types.iter().map(|s| s.to_string()).collect());

    let mut fds: Vec<(String, RawFd)> = Vec::new();

    // Step 1: Open file descriptors to target namespaces
    for ns_type in types_to_join {
        let ns_path = format!("/proc/{}/ns/{}", target_pid, ns_type);

        match open(
            ns_path.as_str(),
            OFlag::O_RDONLY,
            Mode::empty(),
        ) {
            Ok(fd) => {
                fds.push((ns_type.clone(), fd));
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to open namespace {}: {}",
                    ns_path,
                    e
                ));
            }
        }
    }

    // Step 2: Join each namespace
    for (ns_type, fd) in &fds {
        // setns() with CloneFlags corresponding to namespace type
        let clone_flag = match ns_type.as_str() {
            "uts" => nix::sched::CloneFlags::CLONE_NEWUTS,
            "ipc" => nix::sched::CloneFlags::CLONE_NEWIPC,
            "net" => nix::sched::CloneFlags::CLONE_NEWNET,
            "mnt" => nix::sched::CloneFlags::CLONE_NEWNS,
            "pid" => nix::sched::CloneFlags::CLONE_NEWPID,
            "user" => nix::sched::CloneFlags::CLONE_NEWUSER,
            "cgroup" => nix::sched::CloneFlags::CLONE_NEWCGROUP,
            _ => {
                return Err(anyhow::anyhow!("Unknown namespace type: {}", ns_type));
            }
        };

        setns(*fd, clone_flag)
            .with_context(|| format!("Failed to join {} namespace", ns_type))?;
    }

    // Step 3: If joining PID namespace, we must fork
    // The child will have a PID in the new namespace
    let needs_fork = types_to_join.iter().any(|t| t == "pid");

    if needs_fork {
        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                // Wait for child to complete
                use nix::sys::wait::waitpid;
                waitpid(child, None)?;
                return Ok(());
            }
            ForkResult::Child => {
                // Continue to exec in child process
            }
        }
    }

    // Step 4: Execute the command
    if command.is_empty() {
        return Err(anyhow::anyhow!("No command specified"));
    }

    let program = CString::new(command[0].as_str())?;
    let args: Result<Vec<CString>> = command
        .iter()
        .map(|s| CString::new(s.as_str()).map_err(|e| anyhow::anyhow!(e)))
        .collect();
    let args = args?;

    // execvp replaces the current process image
    // If it returns, an error occurred
    execvp(&program, &args)?;

    // This line should never be reached
    Ok(())
}
```

### Understanding the Implementation

**Key steps**:

1. **Open namespace file descriptors**: We open `/proc/[pid]/ns/[type]` for each namespace we want to join. These file descriptors act as references to the namespaces.

2. **Call setns for each namespace**: The `setns()` syscall reassociates the calling thread with the target namespace. We pass the file descriptor and the namespace type flag.

3. **Fork if joining PID namespace**: Because the calling process can't change its own PID, we fork a child. The child will have a PID in the new namespace.

4. **Execute the command**: We use `execvp()` to replace the process image with the target command. This is the same mechanism shells use to run programs.

**Why use CloneFlags**: Even though we're *joining* namespaces (not creating them), `setns()` requires namespace type flags to verify we're joining the correct type. This prevents mistakes like opening a UTS namespace file but trying to join it as a PID namespace.

**Why execvp**: We want to *become* the target command, not just run it as a subprocess. This is how `nsenter` and `docker exec` work - they replace themselves with your shell or command.

### Step 4: Add Required Imports

At the top of `src/main.rs`, ensure you have:

```rust
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use nix::fcntl::{open, OFlag};
use nix::sched::setns;
use nix::sys::stat::Mode;
use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::CString;
use std::os::unix::io::RawFd;
```

### Step 5: Run Tests (Expect Success - GREEN Phase)

Now remove the `#[ignore]` attributes from the tests and run them:

```bash
# Run all setns tests
sudo -E cargo test -p ns-tool --test setns_test

# Or just the specific tests
sudo -E cargo test -p ns-tool test_exec_joins_namespace
sudo -E cargo test -p ns-tool test_exec_nonexistent_pid_fails
```

Expected output:
```
running 3 tests
test test_setns_join_uts_namespace ... ok
test test_exec_joins_namespace ... ok
test test_exec_nonexistent_pid_fails ... ok

test result: ok. 3 passed; 0 failed
```

## Verify

### Automated Verification

```bash
# All setns tests should pass
sudo -E cargo test -p ns-tool --test setns_test
```

### Manual Verification

Now let's test the `exec` subcommand manually to see it in action.

**Scenario 1: Join a UTS namespace and see hostname change**

```bash
# Terminal 1: Create a container with custom hostname
sudo unshare --uts --pid --fork --mount-proc bash
# Inside the namespace:
hostname mycontainer
hostname  # Verify it changed
echo $$   # Note the PID (e.g., 1 inside namespace)

# Find the actual PID (from host perspective)
# Terminal 2 (on host):
ps aux | grep bash | grep -v grep
# Look for a bash process, note its PID (e.g., 12345)

# Join that namespace
sudo cargo run -p ns-tool -- exec --pid 12345 -- hostname
# Output: mycontainer

# Compare with host hostname
hostname
# Output: your-host-name (different!)
```

**Scenario 2: Join network namespace and inspect interfaces**

```bash
# Terminal 1: Create isolated network namespace
sudo unshare --net bash
# Inside:
ip link show
# Should see only 'lo' (loopback), no eth0 or other interfaces

# Terminal 2: Get the PID and join
ps aux | grep "unshare --net" | grep -v grep
# Note PID (e.g., 23456)

sudo cargo run -p ns-tool -- exec --pid 23456 -- ip link show
# Output: Only loopback interface (same as terminal 1)

# Compare with host
ip link show
# Output: Multiple interfaces (eth0, wlan0, docker0, etc.)
```

**Scenario 3: Inspect namespace inodes before and after**

```bash
# Create a namespace
sudo unshare --uts --net sleep 60 &
NS_PID=$!
echo "Namespace process PID: $NS_PID"

# Check namespace inodes
ls -la /proc/$NS_PID/ns/
# Note the inode numbers for uts and net

# Join and verify we're in the same namespace
sudo cargo run -p ns-tool -- exec --pid $NS_PID -- cat /proc/self/ns/uts
sudo cargo run -p ns-tool -- exec --pid $NS_PID -- cat /proc/self/ns/net

# Compare with host (should be different inodes)
readlink /proc/self/ns/uts
readlink /proc/self/ns/net

# Clean up
kill $NS_PID
```

**Scenario 4: Use with real containers**

If you have Docker installed:

```bash
# Start a container
docker run -d --name test-container alpine sleep 300

# Get the container's PID
CONTAINER_PID=$(docker inspect -f '{{.State.Pid}}' test-container)
echo "Container PID: $CONTAINER_PID"

# Exec into it using our tool
sudo cargo run -p ns-tool -- exec --pid $CONTAINER_PID -- sh -c 'hostname && ip addr'

# Compare with docker exec (should show same output)
docker exec test-container sh -c 'hostname && ip addr'

# Clean up
docker rm -f test-container
```

### Using lsns to Verify Namespace Sharing

The `lsns` command shows all namespaces and which processes are in them:

```bash
# List all namespaces
lsns

# List namespaces for a specific PID
lsns -p <PID>

# Show which processes share a specific namespace type
sudo lsns -t net
sudo lsns -t uts
```

## Clean Up

After manual testing, make sure to clean up any lingering processes:

```bash
# Kill any unshare processes
pkill -f "unshare.*sleep"

# Kill any test containers
docker rm -f test-container 2>/dev/null || true

# Verify no orphaned namespaces
lsns | grep -v "^4026531" | wc -l
# Should be 0 or close to it (4026531* are root namespace inodes)
```

## Common Errors

### 1. `Permission denied` when calling setns

**Symptom**:
```
Error: Failed to join uts namespace: EPERM: Operation not permitted
```

**Cause**: The `setns()` syscall requires CAP_SYS_ADMIN capability. You need root privileges or specific capabilities.

**Fix**:
```bash
# Run with sudo
sudo cargo run -p ns-tool -- exec --pid 1234 -- bash

# Or grant capabilities to the binary (not recommended for development)
sudo setcap cap_sys_admin+ep target/debug/ns-tool
```

### 2. `No such file or directory` when opening /proc/[pid]/ns/

**Symptom**:
```
Error: Failed to open namespace /proc/12345/ns/uts: ENOENT: No such file or directory
```

**Cause**: Either:
- The PID doesn't exist (process exited)
- The PID is in a different PID namespace (you can't see it)
- You specified a wrong namespace type

**Fix**:
```bash
# Verify the process exists
ps -p 12345

# Check what namespaces it has
ls /proc/12345/ns/

# Use a long-running process
unshare --uts sleep 1000 &
NS_PID=$!
# Now use $NS_PID immediately
```

### 3. `Invalid argument` when joining user namespace

**Symptom**:
```
Error: Failed to join user namespace: EINVAL: Invalid argument
```

**Cause**: User namespaces have special restrictions:
- Can't join from a multithreaded process
- Must have appropriate privileges in the target namespace
- May have UID/GID mapping issues

**Fix**: For this lesson, avoid joining user namespaces. Stick to UTS, IPC, NET, and MNT which are more straightforward.

### 4. PID Namespace Join Doesn't Change Current PID

**Symptom**:
```bash
# After joining PID namespace
echo $$  # Still shows old PID, not 1
```

**Cause**: This is expected behavior. The calling process can't change its own PID. Only children spawned *after* joining the PID namespace get new PIDs.

**Fix**: This is why our implementation forks when joining PID namespaces. The child gets a PID in the new namespace.

### 5. `execvp` fails with `No such file or directory`

**Symptom**:
```
Error: ENOENT: No such file or directory
```

**Cause**: The command you're trying to execute doesn't exist in the target namespace's PATH or filesystem.

**Fix**:
```bash
# Use absolute paths
sudo cargo run -p ns-tool -- exec --pid 1234 -- /bin/bash

# Or verify the command exists in the namespace first
sudo nsenter -t 1234 -a which hostname
```

### 6. Tests Fail: "process didn't exit successfully: `sleep 10` (signal: 9, SIGKILL)"

**Symptom**: Test cleanup code kills child process, causing error messages.

**Cause**: This is normal - we use `.kill()` to forcefully terminate test processes.

**Fix**: This is expected. As long as the test still passes, the signal: 9 message is fine. You can suppress it with `.ok()`:
```rust
container.kill().ok();
container.wait().ok();
```

## Understanding setns vs unshare vs clone

Let's clarify the relationship between these three namespace syscalls:

| Syscall | Purpose | When to Use |
|---------|---------|-------------|
| `unshare()` | Create new namespaces and move calling process into them | Creating a new isolated context from scratch |
| `clone()` | Create new process with new namespaces | Spawning a child process in fresh isolation (like `fork` + `unshare`) |
| `setns()` | Join existing namespaces | Attaching to already-running isolated processes ("docker exec") |

**Visual comparison**:

```
unshare():   [Process A] ──┐
                           └──> [New NS] ← Process A now here

clone():     [Process A] ──> [New NS + Process B]
                                         └──> Process B starts here

setns():     [Process A] ──┐
             [Process B in Existing NS]
                           └──> Process A joins B's namespace
```

## Notes

### Why File Descriptors Matter

Opening `/proc/[pid]/ns/[type]` gives you a file descriptor that *references* the namespace. This keeps the namespace alive even if all processes exit:

```bash
# Process holds namespace alive
unshare --uts sleep 10 &
PID=$!

# Open fd to namespace (in a real program)
exec 3</proc/$PID/ns/uts  # bash syntax to keep fd open

# Kill the process
kill $PID

# Namespace still exists because fd 3 is open
# Can still join it via fd 3
```

This is how container runtimes create persistent namespaces.

### PID Namespace Special Case

PID namespaces are hierarchical. When you join a PID namespace with `setns()`:

1. Your current process keeps its old PID
2. Children you spawn get PIDs in the new namespace
3. You can't "see" processes in the new namespace with your old PID

This is why `nsenter` and `docker exec` always fork before executing your command.

### The setns Syscall is Not for Creating

**Common misconception**: You might think `setns()` creates namespaces. It doesn't!

- **Wrong**: `setns()` creates a new network namespace
- **Right**: `setns()` joins an existing network namespace; use `unshare()` to create

### Combining with unshare

You can create some namespaces with `unshare()` and join others with `setns()`:

```rust
// Create new UTS namespace
unshare(CloneFlags::CLONE_NEWUTS)?;
sethostname("my-container")?;

// But join existing network namespace from another container
let net_fd = open("/proc/1234/ns/net", OFlag::O_RDONLY, Mode::empty())?;
setns(net_fd, CloneFlags::CLONE_NEWNET)?;

// Now: isolated hostname, shared network
```

This is exactly what container networking plugins do when creating pod networks in Kubernetes.

### Security Implications

Joining namespaces has security implications:

- **Capability required**: CAP_SYS_ADMIN (root-equivalent)
- **Attack surface**: If an attacker can call `setns()`, they can break into your containers
- **User namespaces**: Extra restrictions prevent easy privilege escalation

Container runtimes carefully control who can call `setns()` (that's why `docker exec` requires root or specific user permissions).

## Links and References

- [setns(2) man page](https://man7.org/linux/man-pages/man2/setns.2.html) - Official setns documentation
- [namespaces(7) man page](https://man7.org/linux/man-pages/man7/namespaces.7.html) - Namespace overview
- [nsenter(1) man page](https://man7.org/linux/man-pages/man1/nsenter.1.html) - Tool that uses setns
- [PID namespaces](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html) - Explains PID namespace hierarchy
- [nix::sched::setns documentation](https://docs.rs/nix/latest/nix/sched/fn.setns.html)

## Summary

In this lesson, you learned:

1. **What setns does**: Allows a process to join existing namespaces created by other processes
2. **How to use setns**: Open `/proc/[pid]/ns/[type]` and call `setns(fd, clone_flag)`
3. **PID namespace special handling**: Must fork after joining because the calling process keeps its old PID
4. **Real-world applications**: This is how `docker exec`, `nsenter`, and `kubectl exec` work
5. **Testing namespace operations**: How to spawn isolated processes and verify namespace sharing

You implemented a working `exec` subcommand that can attach to any running namespace, giving you the foundational skill for understanding container debugging and orchestration tools.

## Next

This completes the namespaces section! You've now learned:
- Creating namespaces with `unshare()`
- Combining multiple namespaces for isolation
- Joining existing namespaces with `setns()`

The next section explores **cgroups v2** for resource limits:

`../02-cgroups/01-cgv2-basics.md` - Learn cgroup v2 fundamentals and how to create/manage control groups for CPU, memory, and I/O limits.
