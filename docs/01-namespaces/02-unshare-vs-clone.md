# 02 Unshare vs Clone

## Goal
Understand the difference between `unshare(2)` and `clone3(2)` for creating namespaces, and implement a `clone` subcommand that creates a PID namespace in a single syscall.

**Why this matters**: While `unshare()` is simpler (it affects the calling process), it requires a `fork()` for PID namespaces because the calling process already has a PID. The `clone3()` syscall is more powerful—it creates a new process with namespaces applied atomically. Understanding when to use each approach is essential for building container runtimes and sandboxing tools.

**Deliverable**: Extend `ns-tool` with a `clone` subcommand that demonstrates `clone3()`-based namespace creation, and compare its behavior with the `unshare()`-based `pid` subcommand from lesson 01.

## Prereqs
- Completed `01-pid-namespace.md` (you have a working `pid` subcommand using `unshare()`)
- Understanding of `fork()` and process creation in Unix
- `sudo` access for creating namespaces

## Core Concepts

### unshare(2): Modify Calling Process
```rust
// Pseudocode for unshare-based approach (lesson 01)
unshare(CLONE_NEWPID);  // Calling process enters new PID namespace
fork();                 // REQUIRED: creates first process in new namespace
// Child has PID 1, parent remains in original namespace
```

**Key characteristics**:
- Operates on the calling thread/process
- For PID namespaces: calling process keeps its original PID, so you must `fork()` to create PID 1
- Simpler API: just one flag parameter
- Cannot set all namespace options atomically

### clone3(2): Create Process with Namespaces
```rust
// Pseudocode for clone3-based approach (this lesson)
clone3(CloneArgs {
    flags: CLONE_NEWPID,
    // ... other fields
});
// Returns PID of new child process
// Child automatically has PID 1 in new namespace
```

**Key characteristics**:
- Creates a new process with namespaces in one atomic syscall
- More complex API: requires a `CloneArgs` struct with proper alignment
- More powerful: can set additional options (stack, file descriptors, PID file descriptors, etc.)
- Child immediately has PID 1 without needing a second fork

### When to Use Each

**Use `unshare()` when**:
- You want to modify the current process's namespaces (UTS, IPC, mount, network, etc.)
- You're creating a simple tool or script
- You don't need fine-grained control over the new process

**Use `clone3()` when**:
- Creating PID namespaces efficiently (no double fork required)
- Building container runtimes (precise control over process creation)
- You need to set `CLONE_PIDFD` (for monitoring) or other advanced options
- Creating multiple namespaces atomically for a new process

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/clone_test.rs`

What the tests should verify:
- **Success case**: The `clone` subcommand creates a new process in a PID namespace where the child has PID 1
- **Comparison case**: Both `pid` (unshare-based) and `clone` (clone3-based) produce similar isolation results
- **Error case**: Fails gracefully without root privileges

### Steps

1. **Create the test file**:
   ```bash
   touch crates/ns-tool/tests/clone_test.rs
   ```

2. **Open the file and add the basic test structure**:

   ```rust
   // Tests for the `clone` subcommand (clone3-based PID namespace creation)
   // Lesson: docs/01-namespaces/02-unshare-vs-clone.md
   //
   // TDD Workflow:
   // 1. Write the test(s) below FIRST (RED - they will fail)
   // 2. Add the Clone variant to Command enum in src/main.rs
   // 3. Implement the code in src/main.rs to make tests pass (GREEN)
   // 4. Refactor if needed

   use assert_cmd::Command;
   use predicates::prelude::*;

   #[test]
   fn test_clone3_creates_pid_namespace() {
       // TODO: Implement this test
       //
       // This test should verify that the `clone` subcommand:
       // 1. Successfully creates a PID namespace using clone3
       // 2. The child process has PID 1 inside the namespace
       // 3. The command exits successfully
       //
       // Hints:
       // - Use assert_cmd::Command to run the binary
       // - Pass the `clone` subcommand
       // - Execute a command like `ps -o pid,comm` to verify PID 1 exists
       // - Check stdout contains expected output
       //
       // Example pattern:
       // let mut cmd = Command::cargo_bin("ns-tool").unwrap();
       // cmd.arg("clone")
       //    .arg("--")
       //    .arg("/bin/sh")
       //    .arg("-c")
       //    .arg("echo PID=$$; ps -o pid,comm")
       //    .assert()
       //    .success()
       //    .stdout(predicate::str::contains("PID=1"));

       todo!("Implement test for clone3-based PID namespace creation")
   }

   #[test]
   #[ignore] // Remove after implementing
   fn test_clone3_isolation_from_parent() {
       // TODO: Implement this test
       //
       // Verify that the child process in the new namespace:
       // 1. Cannot see parent processes
       // 2. Has a different /proc/self/ns/pid than the parent
       //
       // Hints:
       // - Run `ps aux` in the child - should only see processes in new namespace
       // - Output should not contain the parent's processes

       todo!("Implement test verifying isolation from parent namespace")
   }

   #[test]
   #[ignore] // Remove after implementing
   fn test_clone3_requires_root() {
       // TODO: Implement this test
       //
       // Verify that running without root fails with a clear error message
       //
       // Hints:
       // - This test is tricky because cargo test usually runs as the same user
       // - You might skip this test and verify manually instead
       // - Or use a wrapper script that drops privileges

       todo!("Implement test for permission error handling")
   }
   ```

3. **Run the test to verify it fails (RED phase)**:
   ```bash
   cargo test -p ns-tool --test clone_test
   ```

   **Expected output**:
   ```
   error: no bin target named `ns-tool`
   ```
   This is expected! The test can't find the `clone` subcommand because we haven't added it to the CLI yet.

4. **Add the `Clone` variant to the CLI enum**:

   Open `crates/ns-tool/src/main.rs` and find the `Command` enum (around line 15). Add a new variant:

   ```rust
   #[derive(Subcommand)]
   enum Command {
       Pid,
       Clone,  // Add this line
       Uts,
       // ... rest of variants
   }
   ```

5. **Run the test again**:
   ```bash
   cargo test -p ns-tool --test clone_test
   ```

   **Expected output**:
   ```
   thread 'test_clone3_creates_pid_namespace' panicked at tests/clone_test.rs:XX:XX:
   not yet implemented: Implement test for clone3-based PID namespace creation
   ```

   Perfect! Now the test infrastructure is in place. Next, implement the actual test.

6. **Implement the test** by replacing the first `todo!()`:

   ```rust
   #[test]
   fn test_clone3_creates_pid_namespace() {
       let mut cmd = Command::cargo_bin("ns-tool").unwrap();
       cmd.arg("clone")
          .arg("--")
          .arg("/bin/sh")
          .arg("-c")
          .arg("echo PID=$$; ps -o pid,comm");

       // Note: This test requires sudo to create namespaces
       // Run with: sudo -E cargo test -p ns-tool --test clone_test
       cmd.assert()
          .success()
          .stdout(predicate::str::contains("PID=1"));
   }
   ```

7. **Run the test again (still RED)**:
   ```bash
   sudo -E cargo test -p ns-tool --test clone_test -- --nocapture
   ```

   **Expected output**:
   ```
   thread 'test_clone3_creates_pid_namespace' panicked at src/main.rs:XX:XX:
   not yet implemented: Implement Clone subcommand - write implementation!
   ```

   Excellent! The test is written and failing because the implementation doesn't exist yet. You're now in the RED phase of TDD.

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`

**TODO location**: In the `match cli.command` block (around line 32), you'll add a new `Command::Clone` arm.

### Implementation Strategy

The `clone3()` syscall is more complex than `unshare()`. Here's what we need to do:

1. Create a `CloneArgs` struct that matches the kernel's expectations
2. Allocate a stack for the child process
3. Call `clone3()` with appropriate flags
4. Handle parent and child execution paths separately
5. In the child, execute the requested command

### Steps

1. **Add the `Clone` match arm** in `src/main.rs` (find the `match cli.command` block):

   ```rust
   // TODO: Implement Clone subcommand using clone3(2)
   // Lesson: docs/01-namespaces/02-unshare-vs-clone.md
   // Tests: tests/clone_test.rs
   //
   // TDD Steps:
   // 1. Tests are already written in tests/clone_test.rs (RED)
   // 2. Implement this function to make tests pass (GREEN)
   // 3. Refactor as needed
   //
   // Implementation hints:
   // - Use clone3(2) to create a new process with CLONE_NEWPID
   // - Child automatically gets PID 1 in the new namespace
   // - Need to allocate a stack for the child process
   // - Handle parent and child code paths separately
   Command::Clone => todo!("Implement Clone subcommand - write implementation!"),
   ```

2. **Understand the challenge**: Unlike `fork()`, `clone()` and `clone3()` require you to provide a stack for the child process. The `nix` crate provides `clone()` but not `clone3()` (as of version 0.27), so we'll use `libc::syscall()` directly.

3. **Add required imports** at the top of `src/main.rs`:

   ```rust
   use std::os::unix::process::CommandExt;
   use nix::sys::wait::{waitpid, WaitStatus};
   use nix::unistd::Pid;
   ```

4. **Implement the clone3 helper function**. Add this before the `main()` function:

   ```rust
   #[cfg(target_os = "linux")]
   fn create_namespace_with_clone3() -> Result<()> {
       use std::ptr;

       // clone3 CloneArgs structure (must match kernel's struct clone_args)
       #[repr(C)]
       struct CloneArgs {
           flags: u64,           // CLONE_* flags
           pidfd: u64,           // pointer to pid_t (for CLONE_PIDFD)
           child_tid: u64,       // pointer to pid_t (for CLONE_CHILD_SETTID)
           parent_tid: u64,      // pointer to pid_t (for CLONE_PARENT_SETTID)
           exit_signal: u64,     // signal to deliver on exit
           stack: u64,           // pointer to stack (0 = shared stack like fork)
           stack_size: u64,      // stack size
           tls: u64,             // TLS descriptor (for CLONE_SETTLS)
           set_tid: u64,         // pointer to pid_t array
           set_tid_size: u64,    // size of set_tid array
           cgroup: u64,          // cgroup file descriptor (for CLONE_INTO_CGROUP)
       }

       // Initialize CloneArgs with zeros (safe default)
       let mut args = CloneArgs {
           flags: libc::CLONE_NEWPID as u64,
           exit_signal: libc::SIGCHLD as u64,
           pidfd: 0,
           child_tid: 0,
           parent_tid: 0,
           stack: 0,        // 0 means share stack with parent (like fork)
           stack_size: 0,
           tls: 0,
           set_tid: 0,
           set_tid_size: 0,
           cgroup: 0,
       };

       // Call clone3 syscall
       // Safety: We're calling a raw syscall with a properly initialized struct
       let pid = unsafe {
           libc::syscall(
               libc::SYS_clone3,
               &mut args as *mut CloneArgs,
               std::mem::size_of::<CloneArgs>(),
           )
       };

       if pid == -1 {
           return Err(anyhow::anyhow!(
               "clone3 failed: {}",
               std::io::Error::last_os_error()
           ));
       }

       if pid == 0 {
           // Child process: we're now in the new PID namespace with PID 1
           let child_pid = nix::unistd::getpid();
           println!("Child PID inside namespace: {}", child_pid);

           // Execute the command passed via args (or default to /bin/sh)
           // For now, let's just run a simple shell command
           let args = vec!["/bin/sh", "-c", "echo PID=$$; ps -o pid,comm"];

           // Use std::process::Command::exec to replace this process
           let error = std::process::Command::new(args[0])
               .args(&args[1..])
               .exec();

           // If exec returns, it failed
           return Err(anyhow::anyhow!("exec failed: {}", error));
       } else {
           // Parent process: wait for child
           println!("Parent: created child with PID {}", pid);

           let child_pid = Pid::from_raw(pid as i32);
           match waitpid(child_pid, None) {
               Ok(WaitStatus::Exited(_, code)) => {
                   println!("Child exited with code {}", code);
                   if code != 0 {
                       return Err(anyhow::anyhow!("Child failed with exit code {}", code));
                   }
               }
               Ok(status) => {
                   return Err(anyhow::anyhow!("Child exited with unexpected status: {:?}", status));
               }
               Err(e) => {
                   return Err(anyhow::anyhow!("waitpid failed: {}", e));
               }
           }
       }

       Ok(())
   }
   ```

5. **Wire up the Clone command** in the match block:

   ```rust
   Command::Clone => create_namespace_with_clone3()?,
   ```

6. **Run the tests (GREEN phase)**:
   ```bash
   sudo -E cargo test -p ns-tool --test clone_test -- --nocapture
   ```

   **Expected output**:
   ```
   running 1 test
   test test_clone3_creates_pid_namespace ... ok

   test result: ok. 1 passed; 0 failed; 2 ignored; 0 finished in 0.05s
   ```

   Success! Your test is now passing. You've completed the GREEN phase.

### Refactor Phase (Optional)

At this point, you might want to:
- Extract command-line argument parsing to accept custom commands (instead of hardcoding `/bin/sh`)
- Add better error messages
- Share code between `pid` and `clone` subcommands

We'll keep the refactoring simple for now. The key learning is understanding the difference between `unshare()` and `clone3()`.

## Verify

### Automated Verification
```bash
# Run all clone tests
sudo -E cargo test -p ns-tool --test clone_test

# Compare with unshare-based tests
sudo -E cargo test -p ns-tool --test pid_test
```

### Manual Verification

Let's compare the behavior of both approaches side by side:

```bash
# Test 1: unshare-based approach (from lesson 01)
echo "=== Using unshare (pid subcommand) ==="
sudo cargo run -p ns-tool -- pid -- /bin/sh -c 'echo "PID=$$"; ps -o pid,comm'

# What you should see:
# - First line: "PID=1" (child process has PID 1)
# - ps output shows only processes in the new namespace
# - PID 1 is the sh process

# Test 2: clone3-based approach (this lesson)
echo "=== Using clone3 (clone subcommand) ==="
sudo cargo run -p ns-tool -- clone

# What you should see:
# - Parent reports creating child with some PID (e.g., 12345)
# - Child reports its PID inside namespace (should be 1)
# - ps output shows PID 1 and ps itself

# Test 3: Verify namespace isolation
echo "=== Verify namespace isolation ==="
# In one terminal, start a long-running process in a new namespace
sudo cargo run -p ns-tool -- clone -- /bin/sh -c 'echo "In namespace, PID=$$"; sleep 30' &

# In another terminal, check that you can't see it from the parent namespace
ps aux | grep sleep
# You should see the sleep process, but from the parent's perspective

# Inside the namespace, only the namespace's processes are visible
```

### Key Observations

1. **Process Creation**:
   - `unshare()`: Requires two process creations (unshare, then fork)
   - `clone3()`: Single syscall creates the child in the namespace

2. **Stack Handling**:
   - With `stack: 0`, `clone3()` shares the parent's stack (fork-like behavior)
   - For more control, you could allocate a custom stack (more complex)

3. **PID Namespace**:
   - Both approaches create equivalent isolation
   - Child always has PID 1 inside the namespace
   - Parent cannot see child's namespace processes

4. **Performance**:
   - `clone3()` is slightly more efficient (one syscall vs. two)
   - For most use cases, the difference is negligible

### Inspect Namespace IDs

```bash
# Check parent's PID namespace
ls -l /proc/self/ns/pid

# Run a command in a new namespace and check its namespace ID
sudo cargo run -p ns-tool -- clone -- /bin/sh -c 'ls -l /proc/self/ns/pid; sleep 1'

# The namespace IDs should be different (e.g., pid:[4026531836] vs pid:[4026532198])
```

## Clean Up

PID namespaces are automatically cleaned up when all processes in the namespace exit. No manual cleanup is required.

If you started any background processes during manual verification:
```bash
# Find any running ns-tool processes
ps aux | grep ns-tool

# Kill them if needed
sudo pkill -f ns-tool
```

## Common Errors

### 1. `clone3 failed: Function not implemented (ENOSYS)`
**Cause**: The `clone3()` syscall was added in Linux 5.3 (2019). Older kernels don't support it.

**Fix**:
- Check your kernel version: `uname -r`
- If kernel < 5.3, you have two options:
  1. Upgrade your kernel
  2. Fall back to `clone()` instead (older API, requires more manual stack management)

**Alternative implementation using `clone()`**:
```rust
// For older kernels, use clone(2) instead of clone3(2)
// Change syscall number from SYS_clone3 to SYS_clone
// Note: clone() API is more complex and architecture-dependent
```

### 2. `Operation not permitted (EPERM)`
**Cause**: Creating PID namespaces requires `CAP_SYS_ADMIN` capability (usually root).

**Fix**: Run with `sudo`:
```bash
sudo -E cargo test -p ns-tool --test clone_test
sudo cargo run -p ns-tool -- clone
```

### 3. Child exits immediately without output
**Cause**: Incorrect `CloneArgs` structure or stack setup. If the struct doesn't match the kernel's expectations, the child may crash silently.

**Fix**:
- Verify `CloneArgs` struct matches your kernel version (field order and sizes matter!)
- Use `#[repr(C)]` to ensure C-compatible layout
- Initialize all fields (use zeros for unused fields)
- Check `dmesg` for kernel messages: `sudo dmesg | tail -20`

### 4. `exec failed: No such file or directory`
**Cause**: The command path is incorrect or the executable doesn't exist in the namespace.

**Fix**:
- Use absolute paths: `/bin/sh` instead of `sh`
- Verify the binary exists: `which sh`
- Remember: mount namespaces can affect which binaries are visible

### 5. Stack-related segfaults
**Cause**: If you try to allocate a custom stack (non-zero `stack` field), you must:
- Allocate sufficient memory (typically 2MB+)
- Set `stack` to point to the TOP of the stack (stacks grow downward)
- Ensure proper alignment

**Fix**: For learning purposes, use `stack: 0` (shared stack) as shown in this lesson. Custom stacks are an advanced topic.

## Notes

### Why clone3 over clone?

The older `clone(2)` syscall has several issues:
- Architecture-dependent parameter order (x86_64 vs ARM vs others)
- Limited to ~32 flags (32-bit flags parameter)
- Cannot atomically set certain options (e.g., CLONE_PIDFD)

The newer `clone3(2)` syscall (Linux 5.3+):
- Uses a struct instead of individual parameters (architecture-independent)
- Supports 64-bit flags and extensible options
- Safer and more future-proof

### Stack Semantics

When `stack: 0` and `stack_size: 0`:
- Child shares parent's stack (fork-like behavior)
- Kernel handles stack setup automatically
- Simpler and sufficient for most use cases

When providing a custom stack:
- You have full control over stack size and location
- Required for some embedded or security-critical applications
- More complex: must handle allocation, alignment, and deallocation

### Comparing Approaches

| Feature | unshare() + fork() | clone3() |
|---------|-------------------|----------|
| Syscalls | 2 (unshare, then fork) | 1 (clone3) |
| Complexity | Lower | Higher (struct setup) |
| Use Case | Modifying current process | Creating new process |
| PID Namespace | Requires fork | Atomic creation |
| Kernel Version | Ancient (2.6.16+) | Modern (5.3+) |
| Preferred For | Simple tools | Container runtimes |

### Man Pages and Documentation

- `man 2 clone3` - clone3 syscall documentation
- `man 2 clone` - older clone syscall
- `man 2 unshare` - unshare syscall (lesson 01)
- `man 7 pid_namespaces` - PID namespace overview
- Kernel source: `include/uapi/linux/sched.h` - CloneArgs structure definition

### Rust Ecosystem Notes

As of late 2024:
- The `nix` crate (v0.27) provides `clone()` but not `clone3()`
- For `clone3()`, you must use `libc::syscall()` directly
- Future versions of `nix` may add `clone3()` support

If you want to avoid unsafe code entirely, stick with `unshare()` from the `nix` crate. The `clone3()` approach shown here is for educational purposes and for understanding how container runtimes work under the hood.

## Next

`03-uts-ipc.md` - Learn about UTS and IPC namespaces for hostname and IPC isolation.
