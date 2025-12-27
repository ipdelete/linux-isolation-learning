# 01 PID Namespace

## Goal
Create a PID namespace using `unshare(2)` and observe process isolation. You'll build a `pid` subcommand in `ns-tool` that creates a new PID namespace, spawns a process that becomes PID 1 inside that namespace, and demonstrates how the process ID space is isolated from the parent namespace.

## Prereqs
- Completed `docs/00-foundations/` section (especially `01-rust-syscall-basics.md` and `05-error-handling.md`)
- `sudo` access (creating namespaces requires `CAP_SYS_ADMIN` capability)
- Understanding of process IDs and the role of PID 1 (init process)

## Background: What is a PID Namespace?

PID namespaces isolate the process ID number space. This means processes in different PID namespaces can have the same process ID. PID namespaces are hierarchical: when you create a new PID namespace, it becomes a child of the current namespace.

**Key properties:**
- The first process created in a new PID namespace becomes PID 1 inside that namespace
- Processes inside the namespace cannot see processes in the parent namespace
- The parent namespace can still see all child processes (but with their original PIDs)
- PID 1 has special responsibilities (reaping zombie processes)
- If PID 1 dies, all processes in the namespace are killed

**Why this matters for containers:**
- Containers appear to have their own init system (PID 1)
- Process isolation prevents containers from seeing host processes
- Each container can run its own process tree without PID conflicts

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/pid_test.rs`

What the tests should verify:
- Success case: The `pid` subcommand creates a new PID namespace and the child process reports PID 1
- Success case: The command outputs expected information about the namespace
- Error case: Running without root privileges fails with a clear error message

Steps:
1. Open `crates/ns-tool/tests/pid_test.rs`
2. Find the `test_pid_namespace_creation` test function (line 13)
3. Replace the `todo!()` with a test implementation:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_pid_namespace_creation() {
    // This test verifies that the pid subcommand successfully creates
    // a new PID namespace and the child process becomes PID 1

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    // The command should succeed when run with root privileges
    // Note: This test will be skipped in CI if not running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_pid_namespace_creation: requires root privileges");
        return;
    }

    cmd.arg("pid")
        .assert()
        .success()
        .stdout(predicate::str::contains("PID inside namespace: 1"))
        .stdout(predicate::str::contains("Parent PID inside namespace:"));
}
```

4. For the error handling test (line 45), implement `test_pid_namespace_without_root_fails`:

```rust
#[test]
fn test_pid_namespace_without_root_fails() {
    // Skip this test if we're actually running as root
    if nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_pid_namespace_without_root_fails: running as root");
        return;
    }

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("pid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Permission denied")
                .or(predicate::str::contains("EPERM"))
                .or(predicate::str::contains("requires root")));
}
```

5. You can leave `test_pid_namespace_isolation` with `#[ignore]` for now - it's more advanced.

6. Run the tests (expect failure because implementation is missing):
```bash
cargo test -p ns-tool --test pid_test
```

Expected output:
```
running 3 tests
test test_pid_namespace_creation ... FAILED
test test_pid_namespace_isolation ... ignored
test test_pid_namespace_without_root_fails ... ok

failures:

---- test_pid_namespace_creation stdout ----
thread 'test_pid_namespace_creation' panicked at 'not yet implemented: Implement PID namespace - write tests first!', crates/ns-tool/src/main.rs:47:26
```

This is the **RED** phase - your tests are written but the implementation doesn't exist yet.

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`
**TODO location**: Line 47 in the `Command::Pid` match arm

Now implement the PID namespace functionality to make your tests pass.

Steps:

1. Open `crates/ns-tool/src/main.rs`

2. Find the `Command::Pid => todo!(...)` match arm (around line 47)

3. Replace the `todo!()` with the implementation:

```rust
Command::Pid => {
    use nix::sched::{unshare, CloneFlags};
    use nix::unistd::{fork, ForkResult, getpid, getppid};
    use nix::sys::wait::{waitpid, WaitStatus};

    println!("Creating new PID namespace...");

    // Step 1: Create a new PID namespace
    // unshare(CLONE_NEWPID) affects only future children, not the current process
    unshare(CloneFlags::CLONE_NEWPID)
        .context("Failed to create PID namespace (are you running with sudo?)")?;

    println!("PID namespace created. Forking child process...");

    // Step 2: Fork a child process
    // The child will be the first process in the new PID namespace (PID 1)
    match unsafe { fork() }.context("Failed to fork child process")? {
        ForkResult::Parent { child } => {
            // Parent process - wait for child to complete
            println!("Parent: spawned child with PID {} (in parent namespace)", child);

            match waitpid(child, None)? {
                WaitStatus::Exited(pid, status) => {
                    println!("Parent: child {} exited with status {}", pid, status);
                }
                _ => {
                    println!("Parent: child exited (status unknown)");
                }
            }
        }
        ForkResult::Child => {
            // Child process - we are now PID 1 in the new namespace
            let my_pid = getpid();
            let my_ppid = getppid();

            println!("\n=== Inside PID Namespace ===");
            println!("PID inside namespace: {}", my_pid);
            println!("Parent PID inside namespace: {}", my_ppid);

            // Verify we're actually PID 1
            if my_pid.as_raw() == 1 {
                println!("Success! We are PID 1 in the new namespace");
            } else {
                eprintln!("Warning: Expected PID 1, got PID {}", my_pid);
            }

            println!("\nYou can verify isolation by checking /proc:");
            println!("  - /proc/self/status | grep NSpid");
            println!("  - ls /proc (shows only processes in this namespace)");
            println!("===========================\n");

            // Exit the child process
            std::process::exit(0);
        }
    }

    Ok(())
}
```

4. Run the tests (expect success):
```bash
sudo -E cargo test -p ns-tool --test pid_test
```

Expected output:
```
running 3 tests
test test_pid_namespace_creation ... ok
test test_pid_namespace_isolation ... ignored
test test_pid_namespace_without_root_fails ... ok

test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

This is the **GREEN** phase - your tests now pass!

## Verify

**Automated verification**:
```bash
# Run all tests for ns-tool (requires sudo for namespace tests)
sudo -E cargo test -p ns-tool

# Run just the PID namespace tests
sudo -E cargo test -p ns-tool --test pid_test
```

All tests should pass.

**Manual verification** (observe the actual behavior):

1. Run the `pid` subcommand manually:
```bash
sudo cargo run -p ns-tool -- pid
```

Expected output:
```
Creating new PID namespace...
PID namespace created. Forking child process...
Parent: spawned child with PID 12345 (in parent namespace)

=== Inside PID Namespace ===
PID inside namespace: 1
Parent PID inside namespace: 0
Success! We are PID 1 in the new namespace

You can verify isolation by checking /proc:
  - /proc/self/status | grep NSpid
  - ls /proc (shows only processes in this namespace)
===========================

Parent: child 12345 exited with status 0
```

2. To explore the namespace interactively, modify the code temporarily to spawn a shell instead of exiting:

Create a temporary test file `test_pid_shell.sh`:
```bash
#!/bin/bash
# This demonstrates the PID namespace interactively

sudo unshare --pid --fork --mount-proc bash -c '
    echo "=== Inside PID Namespace ==="
    echo "My PID: $$"
    echo "Processes visible:"
    ps aux
    echo ""
    echo "Namespace info:"
    cat /proc/self/status | grep -E "^(Pid|NSpid|PPid|NSpgid):"
'
```

Make it executable and run:
```bash
chmod +x test_pid_shell.sh
./test_pid_shell.sh
```

You'll see that `ps aux` only shows processes inside the namespace, not the host processes.

3. Inspect namespace IDs from the parent perspective:

In one terminal:
```bash
# Start a long-running process in a new PID namespace
sudo unshare --pid --fork sleep 1000 &
PID=$!
echo "Started process: $PID"
```

In another terminal:
```bash
# View the namespace ID
ls -l /proc/$PID/ns/pid

# Compare with your own namespace
ls -l /proc/self/ns/pid

# The inode numbers should be different
```

## Clean Up

PID namespaces are automatically cleaned up when all processes in them exit. No manual cleanup needed for this lesson.

If you started background processes during verification:
```bash
# Find and kill any background sleep/unshare processes
pkill -f "unshare.*sleep"

# Or if you saved the PID
kill $PID
```

## Common Errors

1. **`Operation not permitted (os error 1)` when running tests or the command**
   - Cause: Namespace creation requires `CAP_SYS_ADMIN` capability, which typically means running as root
   - Fix: Run with `sudo`: `sudo -E cargo test -p ns-tool --test pid_test`
   - The `-E` flag preserves environment variables (needed for Cargo to find dependencies)

2. **Child process has PID other than 1 inside the namespace**
   - Cause: Calling `unshare()` makes the new namespace apply only to future children, not the current process
   - Fix: Always fork after calling `unshare(CLONE_NEWPID)` - the child will be PID 1
   - This is by design: the calling process cannot change its own PID

3. **Zombie processes appearing after running the command**
   - Cause: Parent process exited without calling `waitpid()` on the child
   - Fix: Always wait for child processes using `waitpid()` (already in the implementation above)
   - Check for zombies: `ps aux | grep defunct`

4. **`failed to fork: ENOMEM` errors**
   - Cause: System resource limits (ulimit) or insufficient memory
   - Fix: Check limits: `ulimit -u` (max user processes), increase if needed: `ulimit -u 4096`
   - Also check: `cat /proc/sys/kernel/pid_max` (system-wide PID limit)

## Notes

**Understanding `fork()` safety:**
- The `nix::unistd::fork()` function is marked `unsafe` because forking in multi-threaded programs is dangerous
- Our program is single-threaded at the point of forking, so this is safe
- In a multi-threaded program, the child process inherits only the calling thread, which can lead to deadlocks

**PID namespace hierarchy:**
- PID namespaces form a tree (unlike other namespace types which are flat)
- A process can have different PIDs in different levels of the hierarchy
- You can see all PIDs using: `cat /proc/self/status | grep NSpid`
  - Example output: `NSpid: 12345 567 1` means PID 12345 in root namespace, 567 in parent namespace, 1 in current namespace

**Why PPID is 0:**
- Inside the namespace, the child's parent PID is 0 because the parent exists in a different namespace
- From the parent namespace's perspective, the normal parent-child relationship exists
- This is intentional isolation - processes cannot see their "real" parent across namespace boundaries

**PID 1 responsibilities:**
- The first process (PID 1) in any PID namespace must handle signal delivery and zombie reaping
- If PID 1 dies, the kernel sends SIGKILL to all other processes in that namespace
- Production containers typically run a proper init system (like `tini` or `dumb-init`) as PID 1

**Manual pages to review:**
- `man 2 unshare` - Create new namespaces
- `man 7 pid_namespaces` - Overview of PID namespace behavior
- `man 2 clone` - Alternative to unshare for creating namespaces
- `man 5 proc` - The /proc filesystem structure

**Kernel version considerations:**
- PID namespaces: Available since Linux 2.6.24 (2008)
- `/proc/[pid]/ns/pid`: Available since Linux 3.8 (2013)
- PID namespace isolation should work on any modern Linux distribution

## Next
`02-unshare-vs-clone.md` - Compare unshare vs clone: two approaches to creating namespaces
