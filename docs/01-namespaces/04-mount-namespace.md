# Mount Namespace

## Goal

Create a mount namespace to isolate filesystem mount points. You will implement the `mount` subcommand for `ns-tool` that creates a mount namespace, makes the root mount private to prevent mount propagation, creates isolated test mounts, and demonstrates that filesystem changes don't leak to the parent namespace.

**What you will build**: A `--mount` flag for `ns-tool` that creates a mount namespace, creates a tmpfs mount inside it, and proves the mount is isolated from the parent.

**Estimated time**: 45-50 minutes

## Prereqs

- Completed `03-uts-ipc.md` (understand UTS and IPC namespace creation)
- `sudo` access (mounting requires `CAP_SYS_ADMIN` capability)
- Basic understanding of Linux filesystem concepts (mount points, filesystems)
- A Linux system with tmpfs support (standard on all modern distributions)

## Concepts: Understanding Mount Namespaces

Before writing code, let's understand what mount namespaces are and why they're fundamental to container isolation.

### What is a Mount Namespace?

A **mount namespace** isolates the set of filesystem mount points visible to a process. When you create a new mount namespace:

- The new namespace starts with a copy of the parent's mount points
- Subsequent mount/unmount operations are isolated
- Changes in the child don't affect the parent (and vice versa, with proper configuration)
- Each namespace has its own `/proc/self/mounts` view

```
Parent Namespace          Child Namespace (after CLONE_NEWNS)
+------------------+      +------------------+
| /                |      | /                |  ← Initially identical
| /tmp             |      | /tmp             |
| /proc            |      | /proc            |
+------------------+      +------------------+
                                  |
                                  | mount tmpfs on /mnt/test
                                  ↓
                          +------------------+
                          | /                |
                          | /tmp             |
                          | /proc            |
                          | /mnt/test (new)  |  ← Only visible here
                          +------------------+
```

### Why Mount Namespaces Matter

Mount namespaces are critical for container isolation because they allow:

1. **Isolated root filesystems**: Each container can have its own `/` without affecting the host
2. **Custom mount points**: Containers can mount tmpfs, overlay filesystems, bind mounts without polluting the host
3. **Security boundaries**: Prevents containers from seeing sensitive host mounts
4. **Resource isolation**: Different containers can have different views of storage

Every container runtime (Docker, Podman, containerd) uses mount namespaces as a foundation.

### Mount Propagation: The Critical Detail

Here's where mount namespaces get tricky. By default, mount points are **shared** between parent and child namespaces. This means:

- If you mount in the child, it appears in the parent (not isolated!)
- If you mount in the parent, it appears in the child

This is usually **not** what you want for containers. To fix this, you must make the root mount **private** before creating other mounts:

```rust
// After creating mount namespace, make root mount private
mount(
    None::<&str>,                    // source (ignored for MS_PRIVATE)
    "/",                             // target (root)
    None::<&str>,                    // fstype (ignored for MS_PRIVATE)
    MsFlags::MS_PRIVATE | MsFlags::MS_REC,  // flags: private + recursive
    None::<&str>                     // data (ignored)
)?;
```

**Key flags**:
- `MS_PRIVATE`: Changes are not propagated to or from other namespaces
- `MS_REC`: Apply recursively to all mount points under `/`

Without this step, your mounts will leak to the parent namespace, defeating the purpose of isolation.

### The Four Mount Propagation Types

| Type | Propagation Behavior | Use Case |
|------|---------------------|----------|
| `MS_SHARED` | Bidirectional: parent ↔ child | Default (usually unwanted for containers) |
| `MS_PRIVATE` | No propagation | Container root filesystems |
| `MS_SLAVE` | One-way: parent → child | Receiving host updates but not sending changes |
| `MS_UNBINDABLE` | Can't be bind-mounted | Special security scenarios |

For containers, `MS_PRIVATE | MS_REC` on `/` is the standard first step.

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/mount_test.rs`

Now let's write tests that verify mount isolation. These tests will fail until you implement the mount namespace functionality.

### What the Tests Should Verify

1. **Success case**: Running `ns-tool mount` should:
   - Create a mount namespace
   - Create a test mount (tmpfs) inside the namespace
   - Show the mount in `/proc/self/mounts` inside the namespace
   - **Not** leak the mount to the parent namespace

2. **Error case**: Not applicable for basic testing (we'll verify manually)

### Steps

1. Open the test file to see the current TODOs:

```bash
cat crates/ns-tool/tests/mount_test.rs
```

2. Open `crates/ns-tool/tests/mount_test.rs` in your editor.

3. Replace the first `todo!()` in `test_mount_namespace_mount_isolation`:

```rust
// Tests for the `mount` subcommand (mount namespace for filesystem isolation)
// Lesson: docs/01-namespaces/04-mount-namespace.md

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_mount_namespace_mount_isolation() {
    // This test verifies that:
    // 1. We can create a mount namespace
    // 2. We can create a tmpfs mount inside it
    // 3. The mount appears in /proc/self/mounts inside the namespace
    // 4. The mount does NOT appear in the parent's /proc/self/mounts

    // Read parent's mounts BEFORE running ns-tool
    let parent_mounts_before = fs::read_to_string("/proc/self/mounts")
        .expect("Failed to read /proc/self/mounts");

    // Run ns-tool mount (should create isolated tmpfs mount)
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("mount")
        .assert()
        .success()
        // Should show that a tmpfs was mounted at /mnt/test_mount
        .stdout(predicate::str::contains("tmpfs"))
        .stdout(predicate::str::contains("/mnt/test_mount"));

    // Read parent's mounts AFTER running ns-tool
    let parent_mounts_after = fs::read_to_string("/proc/self/mounts")
        .expect("Failed to read /proc/self/mounts");

    // Verify the test mount did NOT leak to parent namespace
    // (parent's mounts should be unchanged)
    assert_eq!(
        parent_mounts_before, parent_mounts_after,
        "Mount leaked to parent namespace! Mount propagation not properly isolated."
    );

    // Also verify the test mount path doesn't appear in parent
    assert!(
        !parent_mounts_after.contains("/mnt/test_mount"),
        "Test mount path found in parent namespace"
    );
}
```

**Understanding the test**:
- We read `/proc/self/mounts` before and after running `ns-tool mount`
- Inside `ns-tool mount`, we'll create a tmpfs mount at `/mnt/test_mount`
- The test verifies this mount shows up in the command's output (proving it was created)
- But critically, the parent's `/proc/self/mounts` should be **unchanged** (proving isolation)

4. Now replace the second `todo!()` in `test_mount_namespace_tmpfs`:

```rust
#[test]
fn test_mount_namespace_tmpfs() {
    // This test verifies we can create a tmpfs mount and write to it
    // The tmpfs should only exist inside the namespace

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("mount")
        .assert()
        .success()
        // Verify we mounted tmpfs (the filesystem type)
        .stdout(predicate::str::contains("tmpfs"))
        // Verify we mounted at the expected location
        .stdout(predicate::str::contains("/mnt/test_mount"))
        // Verify the mount appears in /proc/self/mounts inside namespace
        .stdout(predicate::str::contains("type tmpfs"));
}
```

5. Remove the `#[ignore]` attribute from `test_mount_namespace_tmpfs` (it's currently there to skip the test).

6. Run the tests (expect failure - RED phase):

```bash
sudo -E cargo test -p ns-tool --test mount_test
```

**Note**: We need `sudo` because creating mount namespaces and mounting filesystems requires root privileges. The `-E` flag preserves your environment variables (important for cargo to find the right binary).

### Expected Output (RED Phase)

```
running 2 tests
test test_mount_namespace_mount_isolation ... FAILED
test test_mount_namespace_tmpfs ... FAILED

failures:

---- test_mount_namespace_mount_isolation stdout ----
thread 'test_mount_namespace_mount_isolation' panicked at 'not yet implemented: Implement mount namespace - write tests first!'
```

Perfect! The tests fail because we haven't implemented the mount namespace functionality yet. This is the RED phase of TDD.

## Build (Green)

**Implementation file**: `crates/ns-tool/src/main.rs`
**TODO location**: Line ~72 in the `Command::Mount` match arm

Now let's implement the mount namespace functionality to make our tests pass.

### Steps

1. Open `crates/ns-tool/src/main.rs` and find the `Command::Mount` match arm (around line 72).

2. Replace the `todo!()` with this implementation:

```rust
Command::Mount => {
    use nix::mount::{mount, umount, MsFlags};
    use nix::sched::{unshare, CloneFlags};

    // Step 1: Create a new mount namespace
    // This gives us an isolated view of mount points
    unshare(CloneFlags::CLONE_NEWNS)
        .context("Failed to create mount namespace (CLONE_NEWNS)")?;

    // Step 2: Make the root mount private (prevent mount propagation)
    // This is CRITICAL - without this, our mounts leak to the parent!
    // MS_PRIVATE = changes don't propagate to other namespaces
    // MS_REC = apply recursively to all mounts under /
    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )
    .context("Failed to make root mount private")?;

    // Step 3: Create a test mount point directory
    let mount_point = "/mnt/test_mount";
    std::fs::create_dir_all(mount_point)
        .context(format!("Failed to create mount point: {}", mount_point))?;

    // Step 4: Mount a tmpfs filesystem at the test mount point
    // tmpfs is a memory-backed filesystem (like /tmp on many systems)
    // It's perfect for testing because it doesn't require a block device
    mount(
        Some("tmpfs"),                      // source: filesystem type
        mount_point,                        // target: where to mount
        Some("tmpfs"),                      // fstype: filesystem type
        MsFlags::MS_NODEV | MsFlags::MS_NOSUID,  // flags: security
        None::<&str>,                       // data: mount options
    )
    .context(format!("Failed to mount tmpfs at {}", mount_point))?;

    println!("Mount namespace created successfully!");
    println!("Root mount made private (MS_PRIVATE | MS_REC)");
    println!("tmpfs mounted at: {}", mount_point);
    println!();

    // Step 5: Show the mount table to prove isolation
    println!("Mount table inside namespace (/proc/self/mounts):");
    println!("------------------------------------------------------");

    let mounts = std::fs::read_to_string("/proc/self/mounts")
        .context("Failed to read /proc/self/mounts")?;

    // Filter to show only our test mount and a few context mounts
    for line in mounts.lines() {
        if line.contains(mount_point) || line.contains("type tmpfs") {
            println!("{}", line);
        }
    }

    println!("------------------------------------------------------");
    println!("Note: This mount is NOT visible in the parent namespace!");

    // Step 6: Clean up the mount before exiting
    // (prevents orphaned mount points)
    umount(mount_point)
        .context(format!("Failed to unmount {}", mount_point))?;

    std::fs::remove_dir(mount_point)
        .context(format!("Failed to remove mount point directory: {}", mount_point))?;

    println!("Cleanup: unmounted and removed {}", mount_point);

    Ok(())
}
```

### Understanding the Implementation

**Step 1: Create the mount namespace**
```rust
unshare(CloneFlags::CLONE_NEWNS)?;
```
- `CLONE_NEWNS` creates a new mount namespace
- The process now has an isolated view of mount points
- Initially, it's a copy of the parent's mount table

**Step 2: Make root mount private (THE CRITICAL STEP)**
```rust
mount(None::<&str>, "/", None::<&str>,
      MsFlags::MS_PRIVATE | MsFlags::MS_REC, None::<&str>)?;
```
- This is **essential** for isolation
- Without this, mounts propagate to/from parent (shared by default)
- `MS_PRIVATE`: No bidirectional propagation
- `MS_REC`: Apply to all mount points recursively

**Step 3-4: Create and mount tmpfs**
```rust
std::fs::create_dir_all(mount_point)?;
mount(Some("tmpfs"), mount_point, Some("tmpfs"),
      MsFlags::MS_NODEV | MsFlags::MS_NOSUID, None::<&str>)?;
```
- Create the mount point directory (must exist before mounting)
- Mount a tmpfs (RAM-backed filesystem) there
- `MS_NODEV`: Don't allow device files (security)
- `MS_NOSUID`: Don't allow setuid binaries (security)

**Step 5: Verify the mount**
- Read `/proc/self/mounts` to show the mount table
- Filter to show only tmpfs mounts for clarity
- This proves the mount exists in our namespace

**Step 6: Clean up**
- Unmount the tmpfs
- Remove the mount point directory
- Good practice: always clean up test mounts

3. Add the required imports at the top of `main.rs` if not already present (they should be):

```rust
use anyhow::{Context, Result};
```

4. Run the tests (expect success - GREEN phase):

```bash
sudo -E cargo test -p ns-tool --test mount_test
```

### Expected Output (GREEN Phase)

```
running 2 tests
test test_mount_namespace_mount_isolation ... ok
test test_mount_namespace_tmpfs ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

Success! The tests pass because:
1. We create a mount namespace
2. We make the root mount private (prevents propagation)
3. We mount tmpfs at `/mnt/test_mount`
4. The mount appears in the command's output
5. But the parent's `/proc/self/mounts` remains unchanged (isolation works!)

## Verify

### Automated Verification

```bash
# Run all mount namespace tests
sudo -E cargo test -p ns-tool --test mount_test

# Run all ns-tool tests (should all pass)
sudo -E cargo test -p ns-tool
```

Expected: All tests pass.

### Manual Verification

Now let's observe mount namespace isolation interactively.

**1. Run the mount subcommand and see the output:**

```bash
sudo cargo run -p ns-tool -- mount
```

Expected output:
```
Mount namespace created successfully!
Root mount made private (MS_PRIVATE | MS_REC)
tmpfs mounted at: /mnt/test_mount

Mount table inside namespace (/proc/self/mounts):
------------------------------------------------------
tmpfs /mnt/test_mount tmpfs rw,nosuid,nodev,relatime 0 0
tmpfs /dev/shm tmpfs rw,nosuid,nodev 0 0
tmpfs /run tmpfs rw,nosuid,nodev,mode=755 0 0
------------------------------------------------------
Note: This mount is NOT visible in the parent namespace!
Cleanup: unmounted and removed /mnt/test_mount
```

**2. Verify the mount doesn't leak to the parent:**

```bash
# In your current shell (parent namespace)
grep test_mount /proc/self/mounts
```

Expected: **No output** (the mount is isolated)

**3. Use `findmnt` to visualize mount trees:**

```bash
# View your current mount tree
findmnt | head -20

# The test mount won't appear here because it was created
# in an isolated namespace and already cleaned up
```

**4. Create a persistent mount namespace to explore:**

Let's create a longer-lived example to experiment with:

```bash
# Create a shell in a new mount namespace
sudo unshare --mount bash

# Inside the new namespace:
# Make root mount private
mount --make-rprivate /

# Create a test mount
mkdir -p /mnt/isolated_test
mount -t tmpfs tmpfs /mnt/isolated_test

# Verify the mount exists
grep isolated_test /proc/self/mounts
# Output: tmpfs /mnt/isolated_test tmpfs rw,relatime 0 0

# Write a test file
echo "This is isolated" > /mnt/isolated_test/testfile.txt
cat /mnt/isolated_test/testfile.txt

# Exit the namespace
exit
```

**5. Back in the parent namespace, verify isolation:**

```bash
# The mount should NOT exist here
grep isolated_test /proc/self/mounts
# (No output)

# The directory still exists (we created it), but it's empty
ls /mnt/isolated_test/
# (No files - the tmpfs was unmounted when we exited)

# Clean up the empty directory
sudo umount /mnt/isolated_test 2>/dev/null || true
sudo rm -rf /mnt/isolated_test
```

This demonstrates that:
- Mounts created in the child namespace don't appear in the parent
- Files written to tmpfs in the child are isolated
- When the namespace exits, its mounts disappear

**6. Inspect mount propagation flags:**

```bash
# View mount propagation for root filesystem
findmnt -o TARGET,PROPAGATION /

# Typical output:
# TARGET PROPAGATION
# /      shared

# In a container, this would show 'private' instead
```

## Clean Up

The `mount` subcommand cleans up after itself (unmounts tmpfs and removes the directory). However, if you created additional mounts during manual verification:

```bash
# Remove any test mount points you created
sudo umount /mnt/isolated_test 2>/dev/null || true
sudo umount /mnt/test_mount 2>/dev/null || true
sudo rm -rf /mnt/isolated_test /mnt/test_mount

# Verify no orphaned mounts
grep -E "test_mount|isolated_test" /proc/self/mounts
# (Should return nothing)
```

## Common Errors

### 1. `Operation not permitted (os error 1)` when mounting

**Symptom**:
```
Error: Failed to create mount namespace (CLONE_NEWNS)
Caused by: Operation not permitted (os error 1)
```

**Cause**: Insufficient privileges. Creating mount namespaces and mounting filesystems requires `CAP_SYS_ADMIN` capability (typically root).

**Fix**: Always run with `sudo`:
```bash
sudo cargo run -p ns-tool -- mount
sudo -E cargo test -p ns-tool --test mount_test
```

### 2. Mount leaks to parent namespace

**Symptom**: Running the test shows the mount appears in both child and parent `/proc/self/mounts`.

**Cause**: Forgot to make the root mount private with `MS_PRIVATE | MS_REC`.

**Fix**: Ensure you have this code IMMEDIATELY after `unshare()`:
```rust
mount(None::<&str>, "/", None::<&str>,
      MsFlags::MS_PRIVATE | MsFlags::MS_REC, None::<&str>)?;
```

This is the single most critical step for mount namespace isolation. Without it, the default `MS_SHARED` propagation causes all mounts to leak.

### 3. `No such file or directory` when mounting

**Symptom**:
```
Error: Failed to mount tmpfs at /mnt/test_mount
Caused by: No such file or directory (os error 2)
```

**Cause**: The mount point directory doesn't exist.

**Fix**: Create the directory before mounting:
```rust
std::fs::create_dir_all(mount_point)?;
mount(Some("tmpfs"), mount_point, ...)?;
```

### 4. Orphaned mounts after crashes

**Symptom**: After killing the program or experiencing a panic, `grep test_mount /proc/self/mounts` shows the mount still exists.

**Cause**: The cleanup code (`umount`) didn't run because the process was interrupted.

**Fix**: Manually unmount:
```bash
sudo umount /mnt/test_mount
sudo rm -rf /mnt/test_mount
```

**Prevention**: In production code, use RAII (Resource Acquisition Is Initialization) patterns or signal handlers to ensure cleanup on abnormal exit. For learning, manual cleanup is fine.

### 5. `Device or resource busy` when unmounting

**Symptom**:
```
Error: Failed to unmount /mnt/test_mount
Caused by: Device or resource busy (os error 16)
```

**Cause**: Something is still using the mount (a process has it as its current directory, or a file is open).

**Fix**:
```bash
# Find what's using the mount
sudo lsof +D /mnt/test_mount

# Force unmount (use with caution)
sudo umount -l /mnt/test_mount  # lazy unmount
```

### 6. Tests pass but manual run shows "already exists"

**Symptom**: Tests pass, but running `sudo cargo run -p ns-tool -- mount` multiple times shows errors about `/mnt/test_mount` already existing.

**Cause**: Previous run crashed before cleanup, or you have leftover mounts.

**Fix**: Clean up manually:
```bash
sudo umount /mnt/test_mount 2>/dev/null || true
sudo rm -rf /mnt/test_mount
```

## Notes

### Why tmpfs for Testing?

We use `tmpfs` for testing because:

1. **No block device needed**: tmpfs is memory-backed, so we don't need to attach or partition a disk
2. **Fast**: RAM is much faster than disk for test operations
3. **Automatic cleanup**: tmpfs contents disappear when unmounted (no orphaned files)
4. **Standard**: Every Linux system supports tmpfs (it's how `/tmp` and `/dev/shm` often work)

### Mount Flags Explained

Common mount flags you'll see:

| Flag | Meaning | Use Case |
|------|---------|----------|
| `MS_NODEV` | Don't allow device files | Security (prevents privilege escalation via device files) |
| `MS_NOSUID` | Ignore setuid/setgid bits | Security (prevents setuid privilege escalation) |
| `MS_NOEXEC` | Don't allow execution | Security (prevents running binaries from this mount) |
| `MS_RDONLY` | Read-only mount | Immutable filesystems |
| `MS_PRIVATE` | Private mount propagation | Container isolation |
| `MS_SLAVE` | Receive but don't send propagation | Host updates without child changes |
| `MS_SHARED` | Bidirectional propagation | Default (usually unwanted for containers) |
| `MS_REC` | Apply flags recursively | Affect all submounts |

### Mount Propagation in Container Runtimes

Real container runtimes (Docker, Podman) do this more comprehensively:

```rust
// In a real container runtime, after creating mount namespace:
// 1. Make old root private
mount(None::<&str>, "/", None::<&str>, MS_PRIVATE | MS_REC, None::<&str>)?;

// 2. Pivot to new root filesystem
// (we'll cover this in 05-minimal-rootfs.md)

// 3. Unmount old root

// 4. Make new root private as well
mount(None::<&str>, "/", None::<&str>, MS_PRIVATE | MS_REC, None::<&str>)?;
```

This lesson is the foundation; later lessons build the complete container filesystem isolation.

### Comparing with `unshare` Command

The `unshare` command does similar work:

```bash
# What we implemented
sudo cargo run -p ns-tool -- mount

# Equivalent with unshare
sudo unshare --mount bash -c '
  mount --make-rprivate /
  mkdir -p /mnt/test_mount
  mount -t tmpfs tmpfs /mnt/test_mount
  grep test_mount /proc/self/mounts
  umount /mnt/test_mount
  rmdir /mnt/test_mount
'
```

Study both to understand the relationship between the syscalls and the command-line tools.

### Links to Official Documentation

- [mount_namespaces(7) man page](https://man7.org/linux/man-pages/man7/mount_namespaces.7.html) - Comprehensive mount namespace documentation
- [mount(2) man page](https://man7.org/linux/man-pages/man2/mount.2.html) - mount syscall details
- [mount(8) man page](https://man7.org/linux/man-pages/man8/mount.8.html) - mount command usage
- [shared subtrees documentation](https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt) - Mount propagation details
- [nix::mount module](https://docs.rs/nix/latest/nix/mount/index.html) - Rust nix crate mount APIs
- [tmpfs documentation](https://www.kernel.org/doc/html/latest/filesystems/tmpfs.html) - tmpfs filesystem details

## Summary

In this lesson, you learned:

1. **Mount namespaces**: Isolate filesystem mount points between processes
2. **Mount propagation**: The critical difference between shared, private, slave, and unbindable mounts
3. **MS_PRIVATE | MS_REC**: The essential step to prevent mount leaks (make root mount private)
4. **tmpfs mounts**: How to create memory-backed filesystems for testing
5. **Mount verification**: Reading `/proc/self/mounts` to inspect mount tables
6. **Cleanup**: Properly unmounting and removing test mounts

You implemented the `mount` subcommand, which creates an isolated mount namespace and demonstrates that filesystem changes don't leak to the parent. This is a critical foundation for container filesystems.

**Key insight**: The `MS_PRIVATE | MS_REC` step is **not optional** for container isolation. Without it, mount propagation defeats the purpose of mount namespaces. This is a common mistake - always make the root mount private after creating a mount namespace.

## Next

`05-minimal-rootfs.md` - Build a minimal root filesystem and use `pivot_root` to create a complete filesystem isolation environment (the foundation of container images).
