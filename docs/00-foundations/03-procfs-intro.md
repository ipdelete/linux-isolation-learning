# 03 Procfs Intro: Reading Namespace Data from /proc

## Goal
Learn to read and parse namespace information from the `/proc` filesystem using Rust. You will implement tests for the `proc` subcommand and understand how `/proc/[pid]/ns/` reveals which namespaces a process belongs to.

**Deliverable**: Working tests that verify the `proc` subcommand correctly reads namespace symlinks and displays inode numbers.

**Time estimate**: ~30-40 minutes

## Prereqs
- Completed `00-setup-rust.md` (Rust toolchain installed, workspace builds)
- Completed `02-cli-patterns.md` (understand subcommand structure)
- Basic familiarity with Linux filesystems

## Background: Why /proc Matters for Isolation

The `/proc` filesystem is a virtual filesystem that exposes kernel data structures as files. For container isolation, three directories are essential:

| Path | Purpose |
|------|---------|
| `/proc/[pid]/ns/` | Symlinks to namespace inodes - reveals which namespaces a process belongs to |
| `/proc/[pid]/status` | Process metadata including UID/GID mappings for user namespaces |
| `/proc/[pid]/cgroup` | Cgroup membership for resource limits |

**Key insight**: Two processes share a namespace if and only if the inode numbers in their `/proc/[pid]/ns/` symlinks match. This is how tools like `docker exec` determine which namespaces to join.

### Namespace Types You'll See

When you read `/proc/self/ns/`, you'll see symlinks for each namespace type:

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

The number in brackets (e.g., `4026531836`) is the inode number. These numbers uniquely identify each namespace instance on the system.

## Explore /proc Manually First

Before writing code, explore `/proc` from the command line to understand what we're parsing:

```bash
# List your current process's namespace symlinks
ls -la /proc/self/ns/

# Read a specific namespace symlink (shows the target)
readlink /proc/self/ns/pid

# Compare with another shell's namespaces (open a second terminal)
# In terminal 1:
echo $$                           # Note your PID, e.g., 12345
readlink /proc/$$/ns/pid          # e.g., pid:[4026531836]

# In terminal 2:
readlink /proc/$$/ns/pid          # Same inode = same namespace

# View process status (useful for user namespace UID mappings later)
cat /proc/self/status | head -20
```

**Try it**: Run these commands and note the inode numbers. They should match between terminals because both shells are in the same root namespaces.

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/proc_test.rs`

The test file already has TODO placeholders. You will implement two tests:

1. `test_proc_lists_namespaces`: Verify the command outputs namespace names
2. `test_proc_shows_inode_numbers`: Verify inode numbers appear in the correct format

### Step 1: Open the Test File

Open `crates/ns-tool/tests/proc_test.rs` and examine the existing structure with its TODO comments.

### Step 2: Implement the First Test

Replace the first `todo!()` with a real test. The test should verify that `ns-tool proc` outputs known namespace names like `pid`, `net`, `mnt`, etc.

```rust
// crates/ns-tool/tests/proc_test.rs

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_proc_lists_namespaces() {
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("proc")
        .assert()
        .success()
        // Verify core namespace types appear in output
        .stdout(predicate::str::contains("pid"))
        .stdout(predicate::str::contains("net"))
        .stdout(predicate::str::contains("mnt"))
        .stdout(predicate::str::contains("uts"))
        .stdout(predicate::str::contains("ipc"))
        .stdout(predicate::str::contains("user"))
        .stdout(predicate::str::contains("cgroup"));
}
```

### Step 3: Implement the Second Test

The second test verifies that inode numbers appear in the expected format: `namespace:[inode_number]`.

```rust
#[test]
fn test_proc_shows_inode_numbers() {
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    let output = cmd.arg("proc").output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Each line should contain the pattern: name -> type:[inode]
    // Example: "pid -> pid:[4026531836]"
    for line in stdout.lines() {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Verify the arrow separator exists
        assert!(
            line.contains(" -> "),
            "Line missing ' -> ' separator: {}",
            line
        );

        // Verify the inode format [number] exists
        assert!(
            line.contains(":[") && line.contains("]"),
            "Line missing inode format :[number]: {}",
            line
        );
    }
}
```

### Step 4: Run Tests (Expect Success)

Since the `proc` subcommand is already implemented as a reference example, your tests should pass immediately:

```bash
cargo test -p ns-tool --test proc_test
```

Expected output:
```
running 2 tests
test test_proc_lists_namespaces ... ok
test test_proc_shows_inode_numbers ... ok

test result: ok. 2 passed; 0 finished in 0.15s
```

**Note**: This lesson is slightly different from others because the implementation already exists. The focus here is understanding how to test `/proc` access and learning the data format. In subsequent lessons, you'll write tests for unimplemented features (true Red phase).

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`
**Function**: `print_proc_ns()` (lines ~105-114)

The implementation is already complete. Study how it works:

```rust
fn print_proc_ns() -> Result<()> {
    // Read the directory listing of /proc/self/ns
    let entries = std::fs::read_dir("/proc/self/ns")?;

    for entry in entries {
        let entry = entry?;

        // Get the filename (e.g., "pid", "net")
        let name = entry.file_name();

        // Read the symlink target (e.g., "pid:[4026531836]")
        let target = std::fs::read_link(entry.path())?;

        // Print in format: "pid -> pid:[4026531836]"
        println!("{} -> {}", name.to_string_lossy(), target.display());
    }
    Ok(())
}
```

### Key Rust Patterns Used

1. **`std::fs::read_dir()`**: Returns an iterator over directory entries. Each entry is wrapped in `Result` because reading might fail.

2. **`std::fs::read_link()`**: Reads a symlink's target without following it. This is how we get the namespace type and inode.

3. **Error propagation with `?`**: The function returns `Result<()>`, so any I/O error automatically propagates up.

4. **`OsString` handling**: `file_name()` returns `OsString` (not `String`) because filenames can contain non-UTF8 bytes. We use `to_string_lossy()` which replaces invalid UTF8 with the replacement character.

### Alternative: Using the procfs Crate

For more complex `/proc` parsing, consider the `procfs` crate. Here's how you could read namespace info with it:

```rust
// Add to Cargo.toml: procfs = "0.16"
use procfs::process::Process;

fn read_namespaces_with_procfs() -> Result<()> {
    let me = Process::myself()?;
    let ns = me.namespaces()?;

    for (ns_type, ns_info) in ns {
        println!("{}: inode={}", ns_type, ns_info.identifier);
    }
    Ok(())
}
```

For this course, we use `std::fs` directly to understand the underlying mechanics. The `procfs` crate is valuable for production code that needs to parse more complex `/proc` files like `status`, `stat`, or `cgroup`.

## Verify

**Automated verification**:
```bash
cargo test -p ns-tool --test proc_test
```

Both tests should pass.

**Manual verification**:

```bash
# Run the tool
cargo run -q -p ns-tool -- proc

# Expected output (inode numbers will differ on your system):
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

# Compare with manual inspection
ls /proc/self/ns/
readlink /proc/self/ns/pid

# Verify the output matches
```

**Check another process's namespaces** (useful skill for later lessons):

```bash
# Find a long-running process like your shell
echo "My shell PID: $$"

# Read its namespaces (use your actual PID)
ls -la /proc/$$/ns/

# Compare with PID 1 (init/systemd) - usually in root namespaces
sudo ls -la /proc/1/ns/
```

## Clean Up

No cleanup needed for this lesson. Reading `/proc` is a read-only operation that doesn't create any resources.

## Common Errors

1. **`No such file or directory: /proc/self/ns/`**
   - Cause: Not running on Linux (macOS, Windows, or WSL1)
   - Fix: Use a Linux VM or WSL2. The `/proc` filesystem is Linux-specific.

2. **`Permission denied` when reading `/proc/[pid]/ns/`**
   - Cause: Trying to read another user's process namespaces without permission
   - Fix: Use `sudo` or read only processes you own. `/proc/self/ns/` is always readable by the current process.

3. **Tests fail with `cargo_bin not found`**
   - Cause: The binary hasn't been built yet
   - Fix: Run `cargo build -p ns-tool` before running tests, or let `cargo test` build it automatically

4. **Inode numbers don't match between two terminals**
   - Cause: Likely not an error - one process may be in a different namespace (e.g., inside a container)
   - Fix: Verify both terminals are in the host system, not inside Docker/Podman

5. **Output shows `time` namespace as missing**
   - Cause: Kernel older than 5.6 (time namespaces added in 5.6)
   - Fix: Upgrade kernel or ignore - time namespace is rarely needed

## Understanding the Output

Let's decode what you're seeing:

```
pid -> pid:[4026531836]
│      │    └─────────── Inode number (unique namespace ID)
│      └──────────────── Namespace type
└─────────────────────── Symlink name in /proc/self/ns/
```

**Special entries**:
- `pid_for_children`: The PID namespace new children will be created in (may differ from current `pid` after `unshare`)
- `time_for_children`: Same concept for time namespace

**Comparing processes**: Two processes share the PID namespace if and only if:
```bash
readlink /proc/PID1/ns/pid == readlink /proc/PID2/ns/pid
```

This is exactly how container runtimes determine which namespaces to join when attaching to a running container.

## Notes

- **Symlinks are special**: The symlinks in `/proc/[pid]/ns/` can be opened as file descriptors and passed to `setns(2)` to join that namespace. We'll use this in later lessons.

- **Inode persistence**: As long as at least one process is in a namespace OR a file descriptor is open to it OR it's bind-mounted, the namespace persists. When all references are gone, the namespace is destroyed.

- **Kernel version differences**:
  - `pid_for_children` and `time_for_children` added in kernel 4.12
  - `time` namespace added in kernel 5.6
  - `cgroup` namespace added in kernel 4.6

- **Further reading**:
  - `man 7 namespaces` - Overview of all namespace types
  - `man 5 proc` - Documentation for `/proc` filesystem
  - `man 2 setns` - System call to join existing namespaces

## Exercises (Optional)

If you finish early, try these challenges:

### Exercise 1: Compare Processes
Write a shell script or Rust program that takes two PIDs and reports which namespaces they share:

```bash
# Example output:
$ ./compare_ns 1234 5678
pid: DIFFERENT (4026531836 vs 4026532100)
net: SAME (4026531840)
mnt: DIFFERENT (4026531841 vs 4026532098)
...
```

### Exercise 2: Find Container Processes
If you have Docker installed, find which processes are in non-root namespaces:

```bash
docker run -d --name test alpine sleep 1000
docker inspect test --format '{{.State.Pid}}'
# Use that PID to compare with PID 1
```

### Exercise 3: Watch Namespace Creation
In one terminal, watch `/proc/self/ns/pid`. In another, run `unshare --pid --fork /bin/bash`. Do the inode numbers change? Why or why not?

## Next
`04-permissions-and-sudo.md` - Understand when root access is required and how to handle permission errors gracefully
