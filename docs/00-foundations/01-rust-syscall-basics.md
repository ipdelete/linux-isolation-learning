# Rust Syscall Basics

## Goal

Learn how to make Linux system calls from Rust safely and idiomatically. You will implement tests for the `proc` subcommand (which lists namespace information) and understand the fundamental patterns for syscall programming that underpin the entire course.

**What you will build**: Working tests that verify the `ns-tool proc` command correctly reads and displays namespace information from `/proc/self/ns`.

**Estimated time**: 30-40 minutes

## Prereqs

- Completed `00-setup-rust.md` (Rust installed, workspace builds successfully)
- `cargo run -p ns-tool -- proc` works and displays namespace output
- Basic Rust knowledge (variables, functions, match expressions)
- A Linux system (native or VM) - macOS/Windows will not work for this course

## Concepts: How Syscalls Work in Rust

Before writing code, let's understand what system calls are and how Rust interfaces with the Linux kernel.

### What is a System Call?

A **system call** (syscall) is how user programs request services from the kernel. When your program needs to:

- Read a file
- Create a process
- Allocate memory
- Create a namespace

...it must ask the kernel to perform that operation. The kernel is the only code that can directly manipulate hardware, memory mappings, and process state.

```
+------------------+     syscall     +------------------+
|   Your Program   | --------------> |   Linux Kernel   |
|   (user space)   | <-------------- |  (kernel space)  |
+------------------+      result     +------------------+
```

In C, you might call `unshare(CLONE_NEWPID)` to create a new PID namespace. Under the hood, this translates to:

1. Load the syscall number for `unshare` into a CPU register
2. Load the flags argument into another register
3. Execute the `syscall` instruction
4. The kernel handles the request
5. The result (success or error) is returned

### Rust's Approach: Type Safety Over Raw Syscalls

Rust provides two main ways to make syscalls:

#### 1. The `nix` Crate (Preferred)

The `nix` crate provides safe, idiomatic Rust wrappers around POSIX and Linux-specific APIs:

```rust
use nix::sched::{unshare, CloneFlags};

// Type-safe: CloneFlags is an enum, not a magic number
// Returns Result<(), Errno> - proper error handling
unshare(CloneFlags::CLONE_NEWPID)?;
```

**Advantages**:
- Type-safe enums instead of integer constants
- Rust's `Result` type for error handling
- No manual `unsafe` blocks needed
- Prevents common mistakes (wrong flags, missing error checks)

#### 2. The `libc` Crate (When Needed)

Sometimes `nix` doesn't wrap a syscall you need, or you need lower-level control:

```rust
use libc;

// Raw syscall - requires unsafe block
// You must handle errors manually
unsafe {
    let result = libc::unshare(libc::CLONE_NEWPID);
    if result == -1 {
        // Handle error via errno
    }
}
```

**When to use `libc`**:
- The syscall isn't wrapped by `nix`
- You need exact control over arguments
- Performance-critical paths (rare)

### Our Approach in This Course

We follow a simple rule:

> **Use `nix` for everything possible. Fall back to `libc` only when necessary.**

This gives us:
- Maximum safety
- Idiomatic Rust code
- Minimal `unsafe` blocks
- Clear error handling

The `print_proc_ns()` function you saw in setup doesn't actually use syscalls directly - it uses Rust's standard library (`std::fs`) to read files. This is because `/proc` is a virtual filesystem: reading it doesn't require special syscalls, just file I/O.

Later lessons will introduce actual namespace syscalls like `unshare()` and `setns()`.

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/proc_test.rs`

Now let's write tests for the `proc` subcommand. The implementation already exists (study `src/main.rs` to see `print_proc_ns()`), so this is a great first exercise: you'll write tests for working code, learning the testing patterns before you implement features yourself.

### What the Tests Should Verify

1. **Success case**: Running `ns-tool proc` should:
   - Exit with status code 0 (success)
   - Output namespace names (pid, net, mnt, uts, ipc, user, cgroup)
   - Show inode numbers in the format `namespace:[number]`

2. **Error case**: Not applicable for this subcommand (reading `/proc/self/ns` should always work on a functioning Linux system)

### Steps

1. Open the test file:

```bash
# View the current test file with TODOs
cat crates/ns-tool/tests/proc_test.rs
```

2. Open `crates/ns-tool/tests/proc_test.rs` in your editor. You'll see two test functions with `todo!()` markers.

3. Replace the first `todo!()` in `test_proc_lists_namespaces`:

```rust
// Tests for the `proc` subcommand (/proc/self/ns inspection)
// Lesson: docs/00-foundations/01-rust-syscall-basics.md

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_proc_lists_namespaces() {
    // Run the ns-tool binary with the "proc" subcommand
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("proc")
        .assert()
        .success()  // Exit code 0
        .stdout(predicate::str::contains("pid"))
        .stdout(predicate::str::contains("net"))
        .stdout(predicate::str::contains("mnt"))
        .stdout(predicate::str::contains("uts"))
        .stdout(predicate::str::contains("ipc"))
        .stdout(predicate::str::contains("user"))
        .stdout(predicate::str::contains("cgroup"));
}
```

**Understanding the test**:
- `Command::cargo_bin("ns-tool")` finds and runs the compiled binary
- `.arg("proc")` passes the subcommand argument
- `.assert()` captures the result for assertions
- `.success()` verifies exit code 0
- `.stdout(predicate::str::contains(...))` checks output content

4. Replace the second `todo!()` in `test_proc_shows_inode_numbers`:

```rust
#[test]
fn test_proc_shows_inode_numbers() {
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    // Verify the output format: "name -> type:[inode]"
    // Example: "pid -> pid:[4026531836]"
    cmd.arg("proc")
        .assert()
        .success()
        // Check for the arrow separator and bracket format
        .stdout(predicate::str::contains("->"))
        // Inode numbers are shown in brackets after the namespace type
        .stdout(predicate::str::is_match(r"\[[\d]+\]").unwrap());
}
```

**Understanding the regex**:
- `\[[\d]+\]` matches `[` followed by one or more digits, followed by `]`
- This verifies we're showing inode numbers correctly

5. Run the tests (they should now pass since the implementation exists):

```bash
cargo test -p ns-tool --test proc_test
```

**Wait - shouldn't tests fail first (RED)?**

Good observation! In this lesson, the implementation already exists as a reference. We're writing tests *after* implementation to learn the testing patterns. Starting in the next lesson (and all future lessons), you'll write tests first, watch them fail, then implement.

Think of this as "training wheels" for the TDD workflow.

### Expected Output

```
running 2 tests
test test_proc_lists_namespaces ... ok
test test_proc_shows_inode_numbers ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

If tests fail, check:
- Did you add the `use` statements at the top of the file?
- Is the regex syntax correct (use raw strings `r"..."`)?
- Does `cargo run -p ns-tool -- proc` work manually?

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`
**Function**: `print_proc_ns()` at line ~105

Since the implementation already exists, this section explains how it works rather than having you write it. Study this code - it demonstrates patterns you'll use throughout the course.

### The Implementation Explained

```rust
fn print_proc_ns() -> Result<()> {
    // Read directory entries from /proc/self/ns
    let entries = std::fs::read_dir("/proc/self/ns")?;

    for entry in entries {
        // Handle potential I/O errors on each entry
        let entry = entry?;

        // Get the filename (e.g., "pid", "net", "mnt")
        let name = entry.file_name();

        // Each entry is a symlink; read where it points
        // Example: pid -> pid:[4026531836]
        let target = std::fs::read_link(entry.path())?;

        // Print in a readable format
        println!("{} -> {}", name.to_string_lossy(), target.display());
    }
    Ok(())
}
```

### Key Patterns

**1. Error Propagation with `?`**

```rust
let entries = std::fs::read_dir("/proc/self/ns")?;
```

The `?` operator:
- If the operation succeeds, unwraps the `Ok` value
- If it fails, returns early from the function with the error
- This is idiomatic Rust - no manual error checking needed

**2. Using `anyhow::Result`**

```rust
fn print_proc_ns() -> Result<()> {
```

The `Result<()>` here is `anyhow::Result<()>`, which can hold any error type. This simplifies error handling when you're calling functions that return different error types.

**3. Reading `/proc` as Files**

Even though `/proc` is a virtual filesystem (the kernel generates its contents on-the-fly), we read it like regular files. This is a common Linux pattern:

- `/proc/self/ns/*` - symlinks to current process's namespaces
- `/proc/self/cgroup` - current cgroup memberships
- `/proc/self/status` - process status information

### Why No `nix` or `libc` Here?

This function uses only `std::fs` because:

1. Reading `/proc` is just file I/O - no special syscalls needed
2. The kernel exposes namespace information through these virtual files
3. Rust's standard library handles the underlying syscalls (`open`, `read`, `readlink`)

In later lessons, when we *create* namespaces (not just read about them), we'll use `nix` for syscalls like `unshare()`.

## Deep Dive: When to Use `nix` vs `libc` vs `std`

Here's a decision tree for choosing the right approach:

```
Need to interact with Linux kernel?
    |
    +-- No: Use std (files, threads, networking)
    |
    +-- Yes: Is it in nix?
            |
            +-- Yes: Use nix (namespaces, signals, mount)
            |
            +-- No: Is it in libc?
                    |
                    +-- Yes: Use libc with unsafe {}
                    |
                    +-- No: Use raw syscall (extremely rare)
```

### Examples from This Course

| Operation | Crate | Example |
|-----------|-------|---------|
| Read `/proc/self/ns` | `std::fs` | `read_dir("/proc/self/ns")` |
| Create PID namespace | `nix` | `unshare(CloneFlags::CLONE_NEWPID)` |
| Set hostname | `nix` | `sethostname("container")` |
| Fork process | `nix` | `fork()` |
| Mount filesystem | `nix` | `mount(...)` |
| Raw clone3 syscall | `libc` | `libc::syscall(libc::SYS_clone3, ...)` |

## Practical Example: Previewing `unshare`

While we won't implement namespace creation until later lessons, let's see what the code will look like. This prepares you for what's coming.

```rust
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{fork, ForkResult, getpid};

fn create_pid_namespace() -> Result<()> {
    // Create a new PID namespace (requires root)
    unshare(CloneFlags::CLONE_NEWPID)?;

    // Fork - the child will be PID 1 in the new namespace
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            println!("Parent: spawned child with PID {}", child);
            // Wait for child...
        }
        ForkResult::Child => {
            // In the new PID namespace, our PID is 1!
            println!("Child: my PID is {}", getpid());
        }
    }
    Ok(())
}
```

**Notice**:
- `unshare()` from `nix` returns `Result` - proper error handling
- `CloneFlags::CLONE_NEWPID` is type-safe, not a magic number
- `fork()` is unsafe because it's fundamentally dangerous (creates a new process with shared memory)
- `getpid()` is safe - just returns an integer

You'll implement this in `docs/01-namespaces/01-pid-namespace.md`.

## Verify

### Automated Verification

```bash
# All proc_test tests should pass
cargo test -p ns-tool --test proc_test
```

Expected:
```
running 2 tests
test test_proc_lists_namespaces ... ok
test test_proc_shows_inode_numbers ... ok

test result: ok. 2 passed; 0 failed
```

### Manual Verification

Run the proc command and inspect the output:

```bash
cargo run -p ns-tool -- proc
```

You should see all namespace types with their inode numbers:

```
cgroup -> cgroup:[4026531835]
ipc -> ipc:[4026531839]
mnt -> mnt:[4026531841]
net -> net:[4026531840]
pid -> pid:[4026531836]
pid_for_children -> pid:[4026531836]
time -> time:[4026532448]
time_for_children -> time:[4026532448]
user -> user:[4026531837]
uts -> uts:[4026531838]
```

**Explore further**:

```bash
# Verify by reading /proc directly
ls -la /proc/self/ns/

# See that each entry is a symlink
readlink /proc/self/ns/pid

# Compare two shells - they share namespaces (same inode numbers)
# In terminal 1:
readlink /proc/self/ns/pid
# Output: pid:[4026531836]

# In terminal 2:
readlink /proc/self/ns/pid
# Output: pid:[4026531836]  (same inode = same namespace)
```

## Clean Up

No cleanup needed for this lesson. We only read files, didn't create any resources.

## Common Errors

### 1. `error[E0432]: unresolved import`

**Symptom**:
```
error[E0432]: unresolved import `assert_cmd`
  --> tests/proc_test.rs:1:5
   |
1  | use assert_cmd::Command;
   |     ^^^^^^^^^ use of undeclared crate
```

**Cause**: Missing `use` statements at the top of the test file.

**Fix**: Ensure your test file starts with:
```rust
use assert_cmd::Command;
use predicates::prelude::*;
```

### 2. `error: expected one of ... found keyword 'use'`

**Symptom**: Syntax errors when adding imports.

**Cause**: `use` statements placed inside a function instead of at the top of the file.

**Fix**: Move all `use` statements to the top of the file, before any `#[test]` functions.

### 3. Regex Compilation Errors

**Symptom**:
```
error: expected expression
  --> tests/proc_test.rs:15:42
   |
15 |     .stdout(predicate::str::is_match(r"[[\d]+]").unwrap());
   |                                          ^
```

**Cause**: Incorrect regex escaping.

**Fix**: Use `r"\[[\d]+\]"` - the raw string (`r"..."`) prevents Rust from interpreting backslashes, and we escape the literal brackets.

### 4. Tests Fail with "binary not found"

**Symptom**:
```
thread 'test_proc_lists_namespaces' panicked at 'called `Result::unwrap()` on an `Err` value: CargoError(...)'
```

**Cause**: The binary hasn't been built yet.

**Fix**: Build before testing:
```bash
cargo build -p ns-tool
cargo test -p ns-tool --test proc_test
```

### 5. Tests Pass Locally but Fail in CI

**Cause**: Different Linux distributions may have different namespace types (e.g., `time` namespace requires kernel 5.6+).

**Fix**: For this lesson, all assertions should work on any modern Linux. If you're on an older kernel, check your kernel version:
```bash
uname -r
```

## Notes

### Syscall Safety Philosophy

We keep `unsafe` blocks as small as possible. Compare:

**Bad** (too much in unsafe):
```rust
unsafe {
    let result = libc::unshare(libc::CLONE_NEWPID);
    if result == -1 {
        let errno = *libc::__errno_location();
        panic!("unshare failed with errno {}", errno);
    }
    // ... more code ...
}
```

**Good** (minimal unsafe):
```rust
let result = unsafe { libc::unshare(libc::CLONE_NEWPID) };
if result == -1 {
    return Err(std::io::Error::last_os_error().into());
}
```

Or even better, use `nix`:
```rust
nix::sched::unshare(CloneFlags::CLONE_NEWPID)?;  // No unsafe needed!
```

### Understanding `/proc/self/ns`

Each file in `/proc/self/ns/` is a symbolic link that points to a namespace identifier:

- **Format**: `type:[inode]`
- **inode**: A unique number identifying this specific namespace instance
- **Same inode** = **same namespace** (processes share isolation context)
- **Different inode** = **different namespace** (processes are isolated)

This becomes important when we create new namespaces: the inode will change, proving we're in a separate isolation context.

### Links to Official Documentation

- [nix crate documentation](https://docs.rs/nix/latest/nix/)
- [libc crate documentation](https://docs.rs/libc/latest/libc/)
- [proc(5) man page](https://man7.org/linux/man-pages/man5/proc.5.html)
- [namespaces(7) man page](https://man7.org/linux/man-pages/man7/namespaces.7.html)

## Summary

In this lesson, you learned:

1. **What syscalls are**: How user programs request kernel services
2. **Rust's syscall options**: `nix` (preferred, type-safe) vs `libc` (raw, requires unsafe)
3. **Testing patterns**: Using `assert_cmd` and `predicates` to test CLI tools
4. **Error handling**: Using `?` operator and `anyhow::Result` for clean error propagation
5. **Reading `/proc`**: How Linux exposes process information through virtual files

You wrote your first tests for `ns-tool`, establishing the patterns you'll use throughout this course.

## Next

`02-cli-patterns.md` - Learn how we structure Rust CLI tools with `clap`, building the command-line interface for namespace operations.
