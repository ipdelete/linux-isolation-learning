# 01 Cgroup v2 Basics

## Goal

Create and delete a cgroup v2 directory, and attach a process to it. You will implement three subcommands in `cgroup-tool`: `create`, `attach`, and `delete`. By the end of this lesson, you will understand the cgroup v2 unified hierarchy, how processes join cgroups, and the rules for cgroup lifecycle management.

**Estimated time**: 45-60 minutes

## Prereqs

- Completed `docs/00-foundations/` section (especially `02-cli-patterns.md` for Clap usage)
- `sudo` access (cgroup operations require root privileges)
- Linux kernel 4.5+ with cgroup v2 enabled (the default on modern distributions)
- Understanding of the Linux filesystem and directory operations

## Background: What are Control Groups (cgroups)?

Control groups (cgroups) are a Linux kernel feature that allows you to allocate resources (CPU, memory, I/O, network, etc.) among groups of processes. While namespaces provide isolation (what a process can see), cgroups provide resource limits (how much a process can use).

**Cgroup v2 vs v1:**
- **Cgroup v1** (legacy): Multiple independent hierarchies, one per controller (cpu, memory, etc.). Complex and prone to inconsistencies.
- **Cgroup v2** (unified): Single hierarchy with all controllers attached to the same tree. Simpler, more consistent, and the recommended approach since Linux 4.5.

This course uses cgroup v2 exclusively. If your system still uses v1 by default, see the troubleshooting section.

**Key concepts:**

1. **Unified hierarchy**: All cgroups live under `/sys/fs/cgroup` in a single tree structure
2. **Cgroup directory**: Each cgroup is a directory. Creating a directory creates a cgroup.
3. **Control files**: Each cgroup directory contains special files for configuration and monitoring
4. **Process membership**: A process belongs to exactly one cgroup per hierarchy (in v2, there is only one hierarchy)
5. **Controllers**: Resource controllers (cpu, memory, io, pids) limit and account for resource usage

**The cgroup v2 filesystem structure:**

```
/sys/fs/cgroup/                    # Root cgroup
├── cgroup.controllers             # Available controllers
├── cgroup.procs                   # PIDs in this cgroup
├── cgroup.subtree_control         # Controllers enabled for children
├── cpu.max                        # CPU limit (if cpu controller enabled)
├── memory.max                     # Memory limit (if memory controller enabled)
├── my-container/                  # A child cgroup
│   ├── cgroup.controllers
│   ├── cgroup.procs
│   ├── cgroup.subtree_control
│   └── nested-group/              # Nested cgroup
│       ├── cgroup.controllers
│       ├── cgroup.procs
│       └── ...
└── ...
```

**Important cgroup files:**

| File | Description |
|------|-------------|
| `cgroup.procs` | List of PIDs in this cgroup (write PID to move a process here) |
| `cgroup.controllers` | Controllers available in this cgroup |
| `cgroup.subtree_control` | Controllers enabled for child cgroups |
| `cgroup.events` | Cgroup lifecycle events (populated, frozen) |
| `cgroup.type` | Cgroup type (domain, threaded, domain threaded, domain invalid) |

## Write Tests (Red)

We will implement three subcommands, each with its own test file. Following TDD, we write tests first, then implement the code to make them pass.

### Part 1: Test cgroup creation

**Test file**: `crates/cgroup-tool/tests/create_test.rs`

What the tests should verify:
- Success case: Creating a cgroup creates a directory under `/sys/fs/cgroup`
- Success case: The created directory contains the standard `cgroup.procs` file
- Error case: Creating a duplicate cgroup fails with a clear error

Steps:

1. Open `crates/cgroup-tool/tests/create_test.rs`

2. Find the first `todo!()` in `test_create_cgroup`. Replace it with:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn test_create_cgroup() {
    // Use a unique name to avoid conflicts with other tests
    let cgroup_name = format!("test-create-{}", std::process::id());
    let cgroup_path = format!("/sys/fs/cgroup/{}", cgroup_name);

    // Ensure we start clean (in case a previous test failed)
    let _ = fs::remove_dir(&cgroup_path);

    // Skip if not running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_create_cgroup: requires root privileges");
        return;
    }

    // Run the create command
    let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    cmd.arg("create")
        .arg(&cgroup_name)
        .assert()
        .success();

    // Verify the cgroup directory was created
    assert!(
        Path::new(&cgroup_path).exists(),
        "Cgroup directory should exist at {}",
        cgroup_path
    );

    // Verify cgroup.procs file exists (standard cgroup control file)
    let procs_file = format!("{}/cgroup.procs", cgroup_path);
    assert!(
        Path::new(&procs_file).exists(),
        "cgroup.procs should exist at {}",
        procs_file
    );

    // Clean up: remove the test cgroup
    fs::remove_dir(&cgroup_path).expect("Failed to clean up test cgroup");
}
```

3. Update the second test `test_create_nested_cgroup`. Remove the `#[ignore]` attribute and replace the `todo!()`:

```rust
#[test]
fn test_create_nested_cgroup() {
    let parent_name = format!("test-parent-{}", std::process::id());
    let child_name = format!("{}/child", parent_name);
    let parent_path = format!("/sys/fs/cgroup/{}", parent_name);
    let child_path = format!("/sys/fs/cgroup/{}", child_name);

    // Clean up any leftovers
    let _ = fs::remove_dir(&child_path);
    let _ = fs::remove_dir(&parent_path);

    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_create_nested_cgroup: requires root privileges");
        return;
    }

    // Create parent cgroup first
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("create")
        .arg(&parent_name)
        .assert()
        .success();

    // Create nested cgroup
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("create")
        .arg(&child_name)
        .assert()
        .success();

    // Verify both exist
    assert!(Path::new(&parent_path).exists(), "Parent cgroup should exist");
    assert!(Path::new(&child_path).exists(), "Child cgroup should exist");

    // Clean up (must delete child before parent)
    fs::remove_dir(&child_path).expect("Failed to remove child cgroup");
    fs::remove_dir(&parent_path).expect("Failed to remove parent cgroup");
}
```

4. Update the third test `test_create_duplicate_cgroup_fails`. Remove `#[ignore]` and replace the `todo!()`:

```rust
#[test]
fn test_create_duplicate_cgroup_fails() {
    let cgroup_name = format!("test-dup-{}", std::process::id());
    let cgroup_path = format!("/sys/fs/cgroup/{}", cgroup_name);

    let _ = fs::remove_dir(&cgroup_path);

    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_create_duplicate_cgroup_fails: requires root privileges");
        return;
    }

    // Create cgroup first time - should succeed
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("create")
        .arg(&cgroup_name)
        .assert()
        .success();

    // Try to create same cgroup again - should fail
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("create")
        .arg(&cgroup_name)
        .assert()
        .failure()
        .stderr(predicate::str::contains("exists")
            .or(predicate::str::contains("EEXIST"))
            .or(predicate::str::contains("already")));

    // Clean up
    fs::remove_dir(&cgroup_path).expect("Failed to clean up test cgroup");
}
```

5. Run the tests (expect failure because implementation is missing):

```bash
sudo -E cargo test -p cgroup-tool --test create_test
```

Expected output:
```
running 3 tests
test test_create_cgroup ... FAILED
test test_create_nested_cgroup ... FAILED
test test_create_duplicate_cgroup_fails ... FAILED

failures:

---- test_create_cgroup stdout ----
thread 'test_create_cgroup' panicked at crates/cgroup-tool/src/main.rs:41:13:
not yet implemented: Implement cgroup creation - write tests first! (path: ...)
```

This is the **RED** phase for `create`.

### Part 2: Test process attachment

**Test file**: `crates/cgroup-tool/tests/attach_test.rs`

What the tests should verify:
- Success case: A process can be attached to a cgroup by writing its PID to `cgroup.procs`
- Success case: The PID appears in `cgroup.procs` after attachment
- Error case: Attaching to a non-existent cgroup fails

Steps:

1. Open `crates/cgroup-tool/tests/attach_test.rs`

2. Replace the first `todo!()` in `test_attach_process_to_cgroup`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command as StdCommand, Stdio};

#[test]
fn test_attach_process_to_cgroup() {
    let cgroup_name = format!("test-attach-{}", std::process::id());
    let cgroup_path = format!("/sys/fs/cgroup/{}", cgroup_name);

    let _ = fs::remove_dir(&cgroup_path);

    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_attach_process_to_cgroup: requires root privileges");
        return;
    }

    // Create the test cgroup first
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("create")
        .arg(&cgroup_name)
        .assert()
        .success();

    // Spawn a long-running process to attach
    let mut sleep_proc = StdCommand::new("sleep")
        .arg("60")
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to spawn sleep process");

    let sleep_pid = sleep_proc.id();

    // Attach the process to the cgroup
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("attach")
        .arg(&cgroup_name)
        .arg(sleep_pid.to_string())
        .assert()
        .success();

    // Verify the PID is in cgroup.procs
    let procs_path = format!("{}/cgroup.procs", cgroup_path);
    let procs_content = fs::read_to_string(&procs_path)
        .expect("Failed to read cgroup.procs");

    assert!(
        procs_content.contains(&sleep_pid.to_string()),
        "PID {} should be in cgroup.procs, got: {}",
        sleep_pid,
        procs_content
    );

    // Clean up: kill the sleep process first
    sleep_proc.kill().expect("Failed to kill sleep process");
    sleep_proc.wait().expect("Failed to wait for sleep process");

    // Now we can remove the cgroup
    fs::remove_dir(&cgroup_path).expect("Failed to clean up test cgroup");
}
```

3. Update `test_attach_to_nonexistent_cgroup_fails`. Remove `#[ignore]` and replace:

```rust
#[test]
fn test_attach_to_nonexistent_cgroup_fails() {
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test: requires root privileges");
        return;
    }

    // Try to attach to a cgroup that does not exist
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("attach")
        .arg("nonexistent-cgroup-xyz")
        .arg("1")  // PID 1 always exists
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file")
            .or(predicate::str::contains("ENOENT"))
            .or(predicate::str::contains("not found"))
            .or(predicate::str::contains("does not exist")));
}
```

4. Run the tests:

```bash
sudo -E cargo test -p cgroup-tool --test attach_test
```

Expected: Tests fail because `attach` is not implemented yet (RED phase).

### Part 3: Test cgroup deletion

**Test file**: `crates/cgroup-tool/tests/delete_test.rs`

What the tests should verify:
- Success case: An empty cgroup can be deleted
- Error case: A cgroup with processes cannot be deleted (EBUSY)
- Error case: Deleting a non-existent cgroup fails

Steps:

1. Open `crates/cgroup-tool/tests/delete_test.rs`

2. Replace the `todo!()` in `test_delete_empty_cgroup`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::{Command as StdCommand, Stdio};

#[test]
fn test_delete_empty_cgroup() {
    let cgroup_name = format!("test-delete-{}", std::process::id());
    let cgroup_path = format!("/sys/fs/cgroup/{}", cgroup_name);

    let _ = fs::remove_dir(&cgroup_path);

    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_delete_empty_cgroup: requires root privileges");
        return;
    }

    // Create a cgroup
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("create")
        .arg(&cgroup_name)
        .assert()
        .success();

    assert!(Path::new(&cgroup_path).exists(), "Cgroup should exist before deletion");

    // Delete the cgroup
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("delete")
        .arg(&cgroup_name)
        .assert()
        .success();

    // Verify it no longer exists
    assert!(
        !Path::new(&cgroup_path).exists(),
        "Cgroup should not exist after deletion"
    );
}
```

3. Update `test_delete_cgroup_with_processes_fails`. Remove `#[ignore]` and replace:

```rust
#[test]
fn test_delete_cgroup_with_processes_fails() {
    let cgroup_name = format!("test-delete-busy-{}", std::process::id());
    let cgroup_path = format!("/sys/fs/cgroup/{}", cgroup_name);

    let _ = fs::remove_dir(&cgroup_path);

    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test: requires root privileges");
        return;
    }

    // Create cgroup
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("create")
        .arg(&cgroup_name)
        .assert()
        .success();

    // Spawn and attach a process
    let mut sleep_proc = StdCommand::new("sleep")
        .arg("60")
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to spawn sleep process");

    let sleep_pid = sleep_proc.id();

    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("attach")
        .arg(&cgroup_name)
        .arg(sleep_pid.to_string())
        .assert()
        .success();

    // Try to delete - should fail with EBUSY
    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("delete")
        .arg(&cgroup_name)
        .assert()
        .failure()
        .stderr(predicate::str::contains("busy")
            .or(predicate::str::contains("EBUSY"))
            .or(predicate::str::contains("not empty"))
            .or(predicate::str::contains("Device or resource busy")));

    // Clean up: kill process, then delete cgroup
    sleep_proc.kill().expect("Failed to kill sleep");
    sleep_proc.wait().expect("Failed to wait");
    fs::remove_dir(&cgroup_path).expect("Failed to clean up");
}
```

4. Update `test_delete_nonexistent_cgroup_fails`. Remove `#[ignore]` and replace:

```rust
#[test]
fn test_delete_nonexistent_cgroup_fails() {
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test: requires root privileges");
        return;
    }

    Command::cargo_bin("cgroup-tool")
        .unwrap()
        .arg("delete")
        .arg("nonexistent-cgroup-for-delete-test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file")
            .or(predicate::str::contains("ENOENT"))
            .or(predicate::str::contains("not found"))
            .or(predicate::str::contains("does not exist")));
}
```

5. Run the tests:

```bash
sudo -E cargo test -p cgroup-tool --test delete_test
```

Expected: Tests fail (RED phase).

## Build (Green)

Now implement the three subcommands to make all tests pass.

**Implementation file**: `crates/cgroup-tool/src/main.rs`

### Step 1: Implement `create`

**TODO location**: Line ~40 in the `Command::Create { path }` match arm

Replace the `todo!()` with:

```rust
Command::Create { path } => {
    use std::fs;
    use std::path::Path;

    // Construct the full cgroup path
    let cgroup_path = format!("/sys/fs/cgroup/{}", path);
    let cgroup_dir = Path::new(&cgroup_path);

    // Create the cgroup directory
    // Use create_dir (not create_dir_all) to fail if parent doesn't exist
    // This matches cgroup semantics - parent must exist first
    fs::create_dir(cgroup_dir)
        .with_context(|| format!("Failed to create cgroup at {}", cgroup_path))?;

    println!("Created cgroup: {}", cgroup_path);

    // Verify the cgroup was created correctly by checking for cgroup.procs
    let procs_file = cgroup_dir.join("cgroup.procs");
    if procs_file.exists() {
        println!("Verified: cgroup.procs exists");
    } else {
        anyhow::bail!("Cgroup created but cgroup.procs not found - is cgroup v2 mounted?");
    }
}
```

**Note**: You will need to add the `with_context` import. Add this at the top of the file if not already present:

```rust
use anyhow::{Context, Result};
```

### Step 2: Implement `attach`

**TODO location**: Line ~75 in the `Command::Attach { path, pid }` match arm

Replace the `todo!()` with:

```rust
Command::Attach { path, pid } => {
    use std::fs;
    use std::io::Write;

    // Construct path to cgroup.procs
    let cgroup_path = format!("/sys/fs/cgroup/{}", path);
    let procs_path = format!("{}/cgroup.procs", cgroup_path);

    // Write the PID to cgroup.procs
    // Opening and writing to cgroup.procs atomically moves the process
    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(&procs_path)
        .with_context(|| format!("Failed to open {} - does the cgroup exist?", procs_path))?;

    // Write the PID (kernel expects it as a string, newline optional but conventional)
    writeln!(file, "{}", pid)
        .with_context(|| format!("Failed to write PID {} to {}", pid, procs_path))?;

    println!("Attached PID {} to cgroup {}", pid, cgroup_path);

    // Verify the attachment by reading cgroup.procs
    let procs_content = fs::read_to_string(&procs_path)
        .with_context(|| format!("Failed to read {}", procs_path))?;

    if procs_content.lines().any(|line| line.trim() == pid.to_string()) {
        println!("Verified: PID {} is now in {}", pid, path);
    } else {
        println!("Warning: PID {} may not have been attached correctly", pid);
    }
}
```

### Step 3: Implement `delete`

**TODO location**: Line ~57 in the `Command::Delete { path }` match arm

Replace the `todo!()` with:

```rust
Command::Delete { path } => {
    use std::fs;

    // Construct the full cgroup path
    let cgroup_path = format!("/sys/fs/cgroup/{}", path);

    // Remove the cgroup directory
    // Note: remove_dir only works on empty directories
    // A cgroup is "empty" when it has no processes AND no child cgroups
    fs::remove_dir(&cgroup_path)
        .with_context(|| format!(
            "Failed to delete cgroup {} - is it empty? (no processes and no child cgroups)",
            cgroup_path
        ))?;

    println!("Deleted cgroup: {}", cgroup_path);
}
```

### Step 4: Verify implementation compiles

```bash
cargo build -p cgroup-tool
```

### Step 5: Run all tests

```bash
sudo -E cargo test -p cgroup-tool
```

Expected output (all tests should pass):

```
running 3 tests
test test_create_cgroup ... ok
test test_create_duplicate_cgroup_fails ... ok
test test_create_nested_cgroup ... ok

test result: ok. 3 passed; 0 failed

running 2 tests
test test_attach_process_to_cgroup ... ok
test test_attach_to_nonexistent_cgroup_fails ... ok

test result: ok. 2 passed; 0 failed

running 3 tests
test test_delete_empty_cgroup ... ok
test test_delete_cgroup_with_processes_fails ... ok
test test_delete_nonexistent_cgroup_fails ... ok

test result: ok. 3 passed; 0 failed
```

This is the **GREEN** phase!

## Verify

**Automated verification**:

```bash
# Run all cgroup-tool tests
sudo -E cargo test -p cgroup-tool

# Run specific test files
sudo -E cargo test -p cgroup-tool --test create_test
sudo -E cargo test -p cgroup-tool --test attach_test
sudo -E cargo test -p cgroup-tool --test delete_test
```

**Manual verification** (observe the actual behavior):

1. Create a cgroup:

```bash
sudo cargo run -p cgroup-tool -- create my-test-cgroup
```

Expected output:
```
Created cgroup: /sys/fs/cgroup/my-test-cgroup
Verified: cgroup.procs exists
```

2. Inspect the created cgroup:

```bash
# List the cgroup directory
ls -la /sys/fs/cgroup/my-test-cgroup/

# View available controllers
cat /sys/fs/cgroup/my-test-cgroup/cgroup.controllers

# View processes in the cgroup (should be empty)
cat /sys/fs/cgroup/my-test-cgroup/cgroup.procs
```

3. Attach a process:

```bash
# Start a background sleep process
sleep 300 &
SLEEP_PID=$!
echo "Started sleep with PID: $SLEEP_PID"

# Attach it to our cgroup
sudo cargo run -p cgroup-tool -- attach my-test-cgroup $SLEEP_PID

# Verify the process is in the cgroup
cat /sys/fs/cgroup/my-test-cgroup/cgroup.procs

# Also check from the process's perspective
cat /proc/$SLEEP_PID/cgroup
```

Expected `/proc/$PID/cgroup` output:
```
0::/my-test-cgroup
```

The format is `hierarchy-id:controller-list:cgroup-path`. In cgroup v2:
- `0` is the hierarchy ID (always 0 for unified hierarchy)
- Empty controller list (all controllers are in unified hierarchy)
- `/my-test-cgroup` is the path relative to the cgroup root

4. Try to delete the cgroup while a process is attached:

```bash
sudo cargo run -p cgroup-tool -- delete my-test-cgroup
```

Expected: Error message about cgroup being busy

5. Kill the process and delete:

```bash
kill $SLEEP_PID
sudo cargo run -p cgroup-tool -- delete my-test-cgroup
```

Expected:
```
Deleted cgroup: /sys/fs/cgroup/my-test-cgroup
```

6. Verify deletion:

```bash
ls /sys/fs/cgroup/my-test-cgroup
# Should output: "No such file or directory"
```

## Clean Up

If you created cgroups during manual testing that were not cleaned up:

```bash
# List your test cgroups
ls /sys/fs/cgroup/ | grep -E '^(my-|test-)'

# For each cgroup, first check if it has processes
cat /sys/fs/cgroup/<cgroup-name>/cgroup.procs

# If it has processes, move them back to root cgroup
# (or kill them if they are test processes)
sudo sh -c 'echo <pid> > /sys/fs/cgroup/cgroup.procs'

# Then delete the cgroup
sudo rmdir /sys/fs/cgroup/<cgroup-name>
```

For nested cgroups, delete from deepest to shallowest:

```bash
sudo rmdir /sys/fs/cgroup/parent/child/grandchild
sudo rmdir /sys/fs/cgroup/parent/child
sudo rmdir /sys/fs/cgroup/parent
```

## Common Errors

1. **`Permission denied (os error 13)` when creating or deleting cgroups**
   - Cause: Cgroup operations require root privileges
   - Fix: Run with `sudo`: `sudo -E cargo run -p cgroup-tool -- create test`
   - The `-E` flag preserves environment variables for Cargo

2. **`File exists (os error 17)` or `EEXIST` when creating**
   - Cause: A cgroup with that name already exists
   - Fix: Choose a different name or delete the existing cgroup first
   - Check existing cgroups: `ls /sys/fs/cgroup/`

3. **`Device or resource busy (os error 16)` or `EBUSY` when deleting**
   - Cause: The cgroup still has processes or child cgroups
   - Fix: Move or kill all processes, delete child cgroups first
   - Check processes: `cat /sys/fs/cgroup/<name>/cgroup.procs`
   - Check children: `ls /sys/fs/cgroup/<name>/`

4. **`No such file or directory (os error 2)` or `ENOENT`**
   - Cause: Trying to attach to or delete a cgroup that does not exist
   - Fix: Create the cgroup first, or check the path is correct
   - Note: For nested cgroups, the parent must exist

5. **`cgroup.procs not found` after creating a cgroup**
   - Cause: Cgroup v2 is not mounted or the system uses cgroup v1
   - Fix: Check cgroup version with: `mount | grep cgroup`
   - Cgroup v2 shows: `cgroup2 on /sys/fs/cgroup type cgroup2`
   - If using v1, see Notes section for migration

6. **`No such process (os error 3)` or `ESRCH` when attaching**
   - Cause: The PID does not exist (process already exited)
   - Fix: Verify the process exists with `ps -p <pid>`

## Notes

**Cgroup v2 unified hierarchy:**
- In cgroup v2, there is exactly one hierarchy mounted at `/sys/fs/cgroup`
- All controllers (cpu, memory, io, pids, etc.) share this single tree
- A process can only be in one cgroup (no separate per-controller cgroups)
- Controllers are explicitly enabled on a per-subtree basis via `cgroup.subtree_control`

**The cgroup.procs file:**
- Contains one PID per line for all processes in this cgroup
- Writing a PID to this file atomically moves the process to this cgroup
- All threads of a process move together (thread-level control uses `cgroup.threads`)
- You can write your own PID to move yourself to a cgroup

**Controller availability:**
- Not all controllers may be available in all cgroups
- Check `cgroup.controllers` to see what is available
- Enable controllers for children via `cgroup.subtree_control`:
  ```bash
  echo "+cpu +memory" > /sys/fs/cgroup/my-cgroup/cgroup.subtree_control
  ```

**Checking if your system uses cgroup v2:**

```bash
# Check what is mounted
mount | grep cgroup

# Cgroup v2 (unified) output:
# cgroup2 on /sys/fs/cgroup type cgroup2 (rw,nosuid,nodev,noexec,relatime)

# Cgroup v1 (legacy) output:
# cgroup on /sys/fs/cgroup/memory type cgroup (rw,...,memory)
# cgroup on /sys/fs/cgroup/cpu type cgroup (rw,...,cpu)
# (multiple separate mounts)

# Hybrid mode (both v1 and v2):
# You will see both types of mounts
```

**Enabling cgroup v2 if needed:**
- Most modern distributions (Ubuntu 21.10+, Fedora 31+, Debian 11+) use v2 by default
- To force v2 on boot, add `systemd.unified_cgroup_hierarchy=1` to kernel cmdline
- Docker and containerd support cgroup v2 on recent versions

**The "no internal processes" rule:**
- In cgroup v2, a cgroup cannot have both processes AND child cgroups with controllers enabled
- This is called the "no internal processes" constraint
- If you need a process in a parent cgroup, create a "leaf" child for it

**Manual pages to review:**
- `man 7 cgroups` - Overview of cgroups (both v1 and v2)
- Kernel documentation: `/usr/share/doc/linux-doc/cgroup-v2.txt` or online at kernel.org

## Next

`02-memory.md` - Set memory limits on a cgroup and observe what happens when a process exceeds them
