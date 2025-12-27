# Rust Syscall Basics

## Goal

Learn how to make Linux system calls from Rust safely and idiomatically. Understand the fundamental patterns for syscall programming that underpin the entire course: when to use `nix` vs `libc` vs `std`, and how Rust's type system prevents common syscall mistakes.

**What you will learn**: How to choose the right tool (`nix`, `libc`, or `std`) for different kernel interactions, and why Rust is better than C for syscalls.

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

## Deep Dive: Rust's Syscall Options (Instead of Red/Green)

Since this lesson focuses on understanding syscall patterns rather than implementing a specific feature, we'll study the existing code to learn.

### Study the Code Pattern

Open `crates/ns-tool/src/main.rs` and find the `print_proc_ns()` function. This function demonstrates a key pattern: **reading from `/proc` doesn't actually require syscalls** - it just uses `std::fs` for file I/O, which internally calls syscalls like `open`, `read`, and `readlink`.

This illustrates the decision tree you should follow when choosing how to interact with the kernel.

## Build (Green)

**No implementation needed for this lesson.** The code is already written - we're studying it, not writing it.

Open `crates/ns-tool/src/main.rs` and examine `print_proc_ns()`. Note:

### The Pattern Explained

```rust
fn print_proc_ns() -> Result<()> {
    let entries = std::fs::read_dir("/proc/self/ns")?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let target = std::fs::read_link(entry.path())?;
        println!("{} -> {}", name.to_string_lossy(), target.display());
    }
    Ok(())
}
```

**Why this is instructive**:

1. **Error propagation with `?`** - Clean, idiomatic Rust error handling
2. **`anyhow::Result<()>`** - Flexible error type that accepts any error
3. **No `unsafe` blocks** - Reading files doesn't require unsafe Rust, even though `open()` and `readlink()` are syscalls beneath the surface
4. **`std::fs` is enough** - We don't need `nix` or `libc` for this operation

### Why This Doesn't Use `nix` or `libc`

This function uses only `std::fs` because:

1. Reading `/proc` is just file I/O - no special syscalls needed
2. The kernel exposes namespace information through these virtual files
3. Rust's standard library handles the underlying syscalls internally

**When you WOULD need `nix` or `libc`**:
- Creating namespaces: `unshare(CloneFlags::CLONE_NEWPID)` from `nix`
- Joining namespaces: `setns()` from `nix`
- Raw syscalls not wrapped by `nix`: `libc::syscall(...)`

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

## Verify Your Understanding

**Study the working implementation**: This lesson has no tests or builds to verify - you're studying patterns instead.

Make sure you can answer these questions:

1. **When should you use `nix` vs `libc` vs `std`?** - Look at the decision tree above.
2. **Why doesn't `print_proc_ns()` use syscall wrappers?** - Because file I/O through `std::fs` is enough.
3. **What syscalls WILL you use later?** - `unshare()`, `setns()`, `fork()` - the namespace-creation operations.

You can optionally run the existing tool to see it work:

```bash
cargo run -p ns-tool -- proc
```

You should see namespace types with inode numbers. This working code becomes a reference for future lessons.

## Clean Up

No cleanup needed for this lesson. We only read files, didn't create any resources.

## Common Errors

This lesson is conceptual, so most errors will be from exploring `/proc` manually or examining the code:

### 1. `No such file or directory: /proc/self/ns/`

**Cause**: Not running on Linux (macOS, Windows, or WSL1).

**Fix**: Use a Linux VM, WSL2, or DevContainer. The `/proc` filesystem is Linux-specific.

### 2. `Permission denied` when reading `/proc/[pid]/ns/`

**Cause**: Trying to read another user's process namespaces without permission.

**Fix**: Use `sudo` or read only processes you own. `/proc/self/ns/` is always readable.

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
2. **Rust's syscall options**: When to use `nix` (preferred, type-safe) vs `libc` (raw, requires unsafe) vs `std` (safe, when applicable)
3. **Error handling**: Using `?` operator and `anyhow::Result` for clean error propagation
4. **Reading `/proc`**: How Linux exposes process information through virtual files
5. **Safety philosophy**: Why Rust's type system makes syscalls safer than C

You studied the `print_proc_ns()` implementation, which demonstrates these patterns. Next lesson, you'll learn how CLI tools are structured with `clap`.

## Next

`02-cli-patterns.md` - Learn how we structure Rust CLI tools with `clap`, building the command-line interface for namespace operations.
