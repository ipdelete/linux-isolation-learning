# Minimal Root Filesystem

## Goal

Create a minimal root filesystem and use `pivot_root(2)` to completely isolate the filesystem view. You will build the `chroot` subcommand for `ns-tool` that sets up and enters a minimal container root.

**What you will build**: A `ns-tool chroot` command that creates a mount namespace, sets up a minimal rootfs with busybox, pivots into it, and executes a shell.

**Why this matters**: `pivot_root` is how real containers (Docker, Podman, containerd) change their root filesystem. Unlike the older `chroot(2)` syscall, `pivot_root` completely replaces the root mount, making it impossible for the contained process to escape back to the original filesystem.

**Estimated time**: 50 minutes

## Prereqs

- Completed `04-mount-namespace.md` (understand mount namespaces and mount propagation)
- `sudo` access (required for mount operations and namespace creation)
- Basic understanding of Linux filesystem hierarchy (`/proc`, `/dev`, `/sys`)
- Approximately 10MB free disk space for the rootfs

## Concepts: Why pivot_root Instead of chroot?

Before writing code, let's understand the difference between `chroot` and `pivot_root`, and why containers use the latter.

### The Problem with chroot

The `chroot(2)` syscall changes the root directory (`/`) for the current process:

```c
chroot("/new/root");  // Changes root to /new/root
chdir("/");           // Change working directory to new root
```

**Security issue**: A process can escape chroot if it:
1. Has root privileges
2. Creates a new directory inside the chroot
3. Opens a file descriptor to that directory
4. Calls `chroot(".")` on that directory
5. Walks up the directory tree with `chdir("..")`

This is a well-known escape technique. `chroot` was never designed as a security boundary.

### How pivot_root Fixes This

`pivot_root(2)` is designed specifically for containers:

```c
pivot_root(new_root, put_old);  // Atomically swap entire root mount
```

**Key differences**:
1. **Operates on mount points**, not just directory paths
2. **Requires a mount namespace** - can't affect the host system
3. **Atomically swaps** the root mount and moves the old root elsewhere
4. **No escape possible** - the old root is completely unmounted after pivot

```
Before pivot_root:
    / (host root)
    └── /tmp/rootfs (new root, bind mounted)

After pivot_root("/tmp/rootfs", "/tmp/rootfs/oldroot"):
    / (new root - was /tmp/rootfs)
    └── /oldroot (old root, to be unmounted)

After unmounting /oldroot:
    / (new root - completely isolated)
```

### What Makes a Minimal Root Filesystem?

A bootable Linux system needs at least:

1. **Executable files**: A shell or init process (`/bin/sh`)
2. **Shared libraries**: Dependencies for executables (`/lib`, `/lib64`)
3. **Kernel interfaces**:
   - `/proc` - process information
   - `/dev` - device files
   - `/sys` - kernel/device information
4. **Basic directory structure**: `/tmp`, `/etc`, `/root`

For this lesson, we'll use **BusyBox**, a single binary that provides:
- Shell (`sh`)
- Core utilities (`ls`, `cat`, `ps`, `mount`, etc.)
- All in ~1-2MB, with no external dependencies

### pivot_root Requirements

The `pivot_root(2)` syscall has specific requirements:

1. **new_root must be a mount point** - use bind mount
2. **new_root and put_old must be different filesystems** - can't be the same mount
3. **Current working directory must be inside new_root** - change before pivoting
4. **Must be in a mount namespace** - can't pivot the host system

We'll handle all of these in our implementation.

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/chroot_test.rs`

This file doesn't exist yet. Let's create it with tests that verify our `chroot` subcommand works correctly.

### What the Tests Should Verify

1. **Success case**: Running `ns-tool chroot` should:
   - Create and set up a minimal rootfs
   - Successfully pivot into the new root
   - Execute a command in the isolated environment
   - Show only the new rootfs contents (not the host filesystem)

2. **Error case**: Running without sudo should fail with a permission error

### Steps

1. Create the test file:

```bash
touch crates/ns-tool/tests/chroot_test.rs
```

2. Open `crates/ns-tool/tests/chroot_test.rs` in your editor and add the following:

```rust
// Tests for the `chroot` subcommand (minimal rootfs + pivot_root)
// Lesson: docs/01-namespaces/05-minimal-rootfs.md

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_chroot_requires_root() {
    // This test runs without sudo - should fail
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("chroot")
        .assert()
        .failure();  // Should fail without root privileges
}

#[test]
#[ignore]  // Only run with: cargo test -- --ignored --test-threads=1
fn test_chroot_creates_isolated_root() {
    // This test requires sudo and creates actual filesystem changes
    // Run it manually with: sudo -E cargo test --test chroot_test -- --ignored

    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    // Run the chroot subcommand with a simple test: list root directory
    // In the isolated root, we should NOT see host directories
    cmd.arg("chroot")
        .arg("--")
        .arg("ls")
        .arg("/")
        .assert()
        .success()
        // Should see our minimal rootfs directories
        .stdout(predicate::str::contains("bin"))
        .stdout(predicate::str::contains("proc"))
        // Should NOT see typical host-only directories
        .stdout(predicate::str::contains("home").not())
        .stdout(predicate::str::contains("usr").not());
}

#[test]
#[ignore]
fn test_chroot_proc_is_mounted() {
    // Verify /proc is mounted in the new root (needed for ps, top, etc.)
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("chroot")
        .arg("--")
        .arg("mount")  // busybox mount command
        .assert()
        .success()
        .stdout(predicate::str::contains("proc on /proc"));
}

#[test]
#[ignore]
fn test_chroot_busybox_works() {
    // Verify busybox utilities are functional
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();

    cmd.arg("chroot")
        .arg("--")
        .arg("busybox")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("BusyBox"));
}
```

**Understanding the test structure**:

- **`#[ignore]` attribute**: Most tests need root privileges and create filesystem changes. We mark them as ignored so `cargo test` doesn't run them by default.
- **Running ignored tests**: Use `sudo -E cargo test --test chroot_test -- --ignored --test-threads=1`
- **`--`  argument separator**: Separates ns-tool args from the command to run inside the chroot
- **Test isolation**: Each test should be independent and clean up after itself

3. Add a helper module for test setup (at the bottom of the file):

```rust
// Helper functions for test setup and cleanup
#[cfg(test)]
mod helpers {
    use std::path::PathBuf;

    /// Get the path to our test rootfs
    pub fn rootfs_path() -> PathBuf {
        PathBuf::from("/tmp/ns-tool-test-rootfs")
    }

    /// Clean up test rootfs (call in test teardown if needed)
    pub fn cleanup_rootfs() {
        let path = rootfs_path();
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}
```

4. Run the tests (expect failure - RED phase):

```bash
# Regular tests (should fail without implementation)
cargo test -p ns-tool --test chroot_test

# Ignored tests (require sudo, will fail without implementation)
sudo -E cargo test -p ns-tool --test chroot_test -- --ignored --test-threads=1
```

**Expected output**:

```
test test_chroot_requires_root ... FAILED
test test_chroot_creates_isolated_root ... ignored
test test_chroot_proc_is_mounted ... ignored
test test_chroot_busybox_works ... ignored

failures:

---- test_chroot_requires_root stdout ----
thread 'test_chroot_requires_root' panicked at 'called `Result::unwrap()` on an `Err` value: ...'
note: the `chroot` subcommand is not implemented yet (todo!())
```

Perfect! We're in the RED phase. Tests exist but fail because we haven't implemented the functionality yet.

## Build (Green)

Now we'll implement the `chroot` subcommand to make our tests pass. This involves several steps:

1. Set up the rootfs directory structure
2. Download/install busybox
3. Create the mount namespace
4. Bind mount the new root
5. Mount proc/dev/sys
6. Pivot to the new root
7. Unmount the old root
8. Execute the shell or command

**Implementation file**: `crates/ns-tool/src/main.rs`
**New file**: `crates/ns-tool/src/rootfs.rs` (rootfs setup logic)

### Step 1: Update the Command enum

First, we need to add a new `Chroot` command to the enum (keep the existing `Mount` command from lesson 04):

1. Open `crates/ns-tool/src/main.rs`

2. Update the Command enum (around line 15) by adding the Chroot variant after Mount:

```rust
#[derive(Subcommand)]
enum Command {
    Pid,
    Uts,
    Ipc,
    Mount,  // Keep this from lesson 04 - it demonstrates basic mount namespace

    /// Create a mount namespace with a minimal rootfs and pivot into it
    #[command(name = "chroot")]
    Chroot {
        /// Command to run inside the chroot (default: /bin/sh)
        #[arg(last = true)]
        command: Vec<String>,
    },

    Net,
    User,
    Cgroup,
    Time,
    Setns,
    Proc,
}
```

**Understanding the arguments**:
- `#[arg(last = true)]`: Captures all remaining arguments after `--`
- `Vec<String>`: Allows running custom commands like `ls /` or `busybox ps`
- Default to `/bin/sh` if no command specified

**Note**: We keep the `Mount` command from lesson 04 because it teaches a foundational concept - creating a mount namespace and demonstrating mount isolation. Lesson 05 builds on this by adding a new `Chroot` command that goes further, using `pivot_root` to create a complete isolated filesystem. Both are valuable learning experiences.

### Step 2: Create the rootfs module

Create a new file `crates/ns-tool/src/rootfs.rs`:

```bash
touch crates/ns-tool/src/rootfs.rs
```

Add the following implementation:

```rust
//! Minimal root filesystem creation and management
//!
//! This module handles creating a minimal rootfs with busybox
//! and the necessary directory structure for a working container.

use anyhow::{Context, Result, bail};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Default rootfs location
const DEFAULT_ROOTFS: &str = "/tmp/ns-tool-rootfs";

/// URL to download busybox (statically linked, no dependencies)
const BUSYBOX_URL: &str = "https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox";

/// Set up a minimal rootfs with busybox and required directories
pub fn setup_minimal_rootfs() -> Result<PathBuf> {
    let rootfs = PathBuf::from(DEFAULT_ROOTFS);

    // Create rootfs directory if it doesn't exist
    if !rootfs.exists() {
        fs::create_dir_all(&rootfs)
            .context("failed to create rootfs directory")?;
        println!("Created rootfs at {}", rootfs.display());
    }

    // Create essential directory structure
    create_directory_structure(&rootfs)?;

    // Install busybox if not present
    install_busybox(&rootfs)?;

    // Create busybox symlinks for common utilities
    create_busybox_symlinks(&rootfs)?;

    Ok(rootfs)
}

/// Create the minimal directory structure needed for a working system
fn create_directory_structure(rootfs: &Path) -> Result<()> {
    let dirs = [
        "bin",     // Executables
        "sbin",    // System executables
        "lib",     // Shared libraries (if needed)
        "lib64",   // 64-bit libraries (if needed)
        "proc",    // Mount point for /proc
        "dev",     // Mount point for /dev
        "sys",     // Mount point for /sys
        "tmp",     // Temporary files
        "etc",     // Configuration files
        "root",    // Root user home directory
        "oldroot", // Temporary mount point for old root (used during pivot)
    ];

    for dir in &dirs {
        let path = rootfs.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path)
                .with_context(|| format!("failed to create directory: {}", dir))?;
        }
    }

    println!("Created directory structure");
    Ok(())
}

/// Download and install busybox binary
fn install_busybox(rootfs: &Path) -> Result<()> {
    let busybox_path = rootfs.join("bin/busybox");

    // Check if busybox already exists and is executable
    if busybox_path.exists() {
        let metadata = fs::metadata(&busybox_path)?;
        if metadata.permissions().mode() & 0o111 != 0 {
            println!("Busybox already installed");
            return Ok(());
        }
    }

    println!("Downloading busybox from {}", BUSYBOX_URL);

    // Download busybox using curl or wget
    let status = Command::new("curl")
        .arg("-L")  // Follow redirects
        .arg("-o")
        .arg(&busybox_path)
        .arg(BUSYBOX_URL)
        .status()
        .or_else(|_| {
            // Fallback to wget if curl not available
            Command::new("wget")
                .arg("-O")
                .arg(&busybox_path)
                .arg(BUSYBOX_URL)
                .status()
        })
        .context("failed to download busybox (need curl or wget)")?;

    if !status.success() {
        bail!("busybox download failed");
    }

    // Make executable
    let mut perms = fs::metadata(&busybox_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&busybox_path, perms)?;

    println!("Installed busybox at {}", busybox_path.display());
    Ok(())
}

/// Create symlinks for common busybox utilities
fn create_busybox_symlinks(rootfs: &Path) -> Result<()> {
    let bin_dir = rootfs.join("bin");
    let busybox = "busybox";

    // Common utilities we want as separate commands
    let utils = [
        "sh", "ls", "cat", "cp", "mv", "rm", "mkdir", "rmdir",
        "ps", "top", "grep", "find", "echo", "mount", "umount",
        "chmod", "chown", "df", "du", "pwd", "cd", "env",
    ];

    for util in &utils {
        let link_path = bin_dir.join(util);
        if !link_path.exists() {
            std::os::unix::fs::symlink(busybox, &link_path)
                .with_context(|| format!("failed to create symlink for {}", util))?;
        }
    }

    println!("Created busybox symlinks");
    Ok(())
}

/// Clean up the rootfs (useful for testing)
pub fn cleanup_rootfs() -> Result<()> {
    let rootfs = PathBuf::from(DEFAULT_ROOTFS);
    if rootfs.exists() {
        fs::remove_dir_all(&rootfs)
            .context("failed to remove rootfs")?;
        println!("Cleaned up rootfs");
    }
    Ok(())
}
```

**Key implementation details**:

- **Statically linked busybox**: No shared library dependencies needed
- **Symlink approach**: busybox uses symlinks to determine which utility to run
- **Download fallback**: Try curl first, then wget
- **Idempotent**: Safe to run multiple times, checks if already set up

### Step 3: Add pivot_root implementation

Create `crates/ns-tool/src/pivot.rs`:

```bash
touch crates/ns-tool/src/pivot.rs
```

Add the implementation:

```rust
//! pivot_root implementation for changing container root filesystem
//!
//! This module handles the complex dance of pivot_root:
//! 1. Ensure new_root is a mount point (bind mount if needed)
//! 2. Create a place to put the old root
//! 3. Change current directory into new_root
//! 4. Call pivot_root syscall
//! 5. Change to the new root
//! 6. Unmount and remove old root

use anyhow::{Context, Result, bail};
use nix::mount::{mount, umount2, MsFlags, MntFlags};
use nix::unistd::chdir;
use std::path::{Path, PathBuf};

/// Pivot into a new root filesystem
///
/// This function performs all the steps necessary to pivot_root safely:
/// - Bind mounts new_root to ensure it's a mount point
/// - Creates old_root mount point inside new_root
/// - Changes to new_root
/// - Calls pivot_root
/// - Unmounts old root
pub fn pivot_root(new_root: &Path) -> Result<()> {
    let new_root = new_root
        .canonicalize()
        .context("failed to canonicalize new root path")?;

    println!("Pivoting root to {}", new_root.display());

    // Step 1: Ensure new_root is a mount point (requirement for pivot_root)
    // We do this by bind-mounting it to itself
    bind_mount_self(&new_root)?;

    // Step 2: Create put_old directory inside new_root
    // This is where the old root will be temporarily mounted
    let put_old = new_root.join("oldroot");
    if !put_old.exists() {
        std::fs::create_dir(&put_old)
            .context("failed to create oldroot directory")?;
    }

    // Step 3: Change current directory to new_root
    // pivot_root requires we're inside the new root
    chdir(&new_root)
        .context("failed to chdir to new root")?;

    // Step 4: Call pivot_root
    // This is the actual syscall that swaps the root mount
    syscall_pivot_root(".", "oldroot")?;

    // Step 5: Change to the new root
    chdir("/")
        .context("failed to chdir to /")?;

    // Step 6: Unmount and remove old root
    cleanup_old_root()?;

    println!("Successfully pivoted to new root");
    Ok(())
}

/// Bind mount a path to itself to ensure it's a mount point
fn bind_mount_self(path: &Path) -> Result<()> {
    mount(
        Some(path),
        path,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )
    .context("failed to bind mount new root")?;

    // Make the mount private to prevent propagation
    mount(
        None::<&str>,
        path,
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )
    .context("failed to make mount private")?;

    Ok(())
}

/// Call the pivot_root syscall
///
/// Unfortunately, the nix crate doesn't provide a wrapper for pivot_root,
/// so we use libc directly. This is a rare case where we need unsafe.
fn syscall_pivot_root(new_root: &str, put_old: &str) -> Result<()> {
    use std::ffi::CString;

    let new_root_c = CString::new(new_root)
        .context("invalid new_root path")?;
    let put_old_c = CString::new(put_old)
        .context("invalid put_old path")?;

    // SAFETY: We've validated the paths are proper C strings
    // and we're in a mount namespace so this can't affect the host
    let result = unsafe {
        libc::syscall(
            libc::SYS_pivot_root,
            new_root_c.as_ptr(),
            put_old_c.as_ptr(),
        )
    };

    if result == -1 {
        let error = std::io::Error::last_os_error();
        bail!("pivot_root failed: {}", error);
    }

    Ok(())
}

/// Unmount and remove the old root
fn cleanup_old_root() -> Result<()> {
    // Unmount recursively to handle any submounts
    umount2("/oldroot", MntFlags::MNT_DETACH)
        .context("failed to unmount old root")?;

    // Remove the directory
    std::fs::remove_dir("/oldroot")
        .context("failed to remove oldroot directory")?;

    Ok(())
}
```

**Why we need unsafe here**:

The `nix` crate doesn't wrap `pivot_root` (as of version 0.27), so we must use `libc::syscall` directly. This is one of the rare legitimate uses of `unsafe` in this course:

1. We validate the paths are proper C strings
2. We're in a mount namespace, so can't affect the host
3. The unsafe block is minimal (just the syscall itself)

### Step 4: Mount essential filesystems

Create `crates/ns-tool/src/mounts.rs`:

```bash
touch crates/ns-tool/src/mounts.rs
```

Add the implementation:

```rust
//! Mount essential filesystems in the new root
//!
//! A working Linux system needs /proc, /dev, and /sys mounted

use anyhow::{Context, Result};
use nix::mount::{mount, MsFlags};
use std::path::Path;

/// Mount all essential filesystems in the new root
pub fn mount_essential_filesystems(rootfs: &Path) -> Result<()> {
    mount_proc(rootfs)?;
    mount_dev(rootfs)?;
    mount_sys(rootfs)?;
    Ok(())
}

/// Mount /proc (process information)
fn mount_proc(rootfs: &Path) -> Result<()> {
    let target = rootfs.join("proc");

    mount(
        Some("proc"),
        &target,
        Some("proc"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
        None::<&str>,
    )
    .context("failed to mount /proc")?;

    println!("Mounted /proc");
    Ok(())
}

/// Mount /dev (device files)
fn mount_dev(rootfs: &Path) -> Result<()> {
    let target = rootfs.join("dev");

    // Mount a tmpfs for /dev
    mount(
        Some("tmpfs"),
        &target,
        Some("tmpfs"),
        MsFlags::MS_NOSUID,
        Some("mode=0755"),
    )
    .context("failed to mount /dev")?;

    println!("Mounted /dev");

    // Create essential device files
    create_essential_devices(&target)?;

    Ok(())
}

/// Mount /sys (kernel/device information)
fn mount_sys(rootfs: &Path) -> Result<()> {
    let target = rootfs.join("sys");

    mount(
        Some("sysfs"),
        &target,
        Some("sysfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV | MsFlags::MS_RDONLY,
        None::<&str>,
    )
    .context("failed to mount /sys")?;

    println!("Mounted /sys");
    Ok(())
}

/// Create essential device files in /dev
fn create_essential_devices(dev: &Path) -> Result<()> {
    use nix::sys::stat::{mknod, Mode, SFlag};
    use nix::unistd::{Gid, Uid};
    use std::os::unix::fs::symlink;

    // Create /dev/null
    let null = dev.join("null");
    if !null.exists() {
        let _ = mknod(
            &null,
            SFlag::S_IFCHR,
            Mode::from_bits(0o666).unwrap(),
            nix::sys::stat::makedev(1, 3),  // Major 1, Minor 3 for /dev/null
        );
    }

    // Create /dev/zero
    let zero = dev.join("zero");
    if !zero.exists() {
        let _ = mknod(
            &zero,
            SFlag::S_IFCHR,
            Mode::from_bits(0o666).unwrap(),
            nix::sys::stat::makedev(1, 5),  // Major 1, Minor 5 for /dev/zero
        );
    }

    // Create /dev/random and /dev/urandom
    let random = dev.join("random");
    if !random.exists() {
        let _ = mknod(
            &random,
            SFlag::S_IFCHR,
            Mode::from_bits(0o666).unwrap(),
            nix::sys::stat::makedev(1, 8),
        );
    }

    let urandom = dev.join("urandom");
    if !urandom.exists() {
        let _ = mknod(
            &urandom,
            SFlag::S_IFCHR,
            Mode::from_bits(0o666).unwrap(),
            nix::sys::stat::makedev(1, 9),
        );
    }

    // Create /dev/console
    let console = dev.join("console");
    if !console.exists() {
        let _ = mknod(
            &console,
            SFlag::S_IFCHR,
            Mode::from_bits(0o600).unwrap(),
            nix::sys::stat::makedev(5, 1),
        );
    }

    // Symlinks for standard file descriptors
    let _ = symlink("/proc/self/fd", dev.join("fd"));
    let _ = symlink("/proc/self/fd/0", dev.join("stdin"));
    let _ = symlink("/proc/self/fd/1", dev.join("stdout"));
    let _ = symlink("/proc/self/fd/2", dev.join("stderr"));

    Ok(())
}
```

**Understanding device files**:

- Device files have **major** and **minor** numbers that identify kernel drivers
- `/dev/null` (1,3): Data sink - reads return EOF, writes are discarded
- `/dev/zero` (1,5): Source of null bytes
- `/dev/random`, `/dev/urandom` (1,8), (1,9): Random number generators
- We ignore errors from `mknod` since we might not have permissions in all contexts

### Step 5: Wire everything together in main.rs

Now update `crates/ns-tool/src/main.rs`:

1. Add module declarations at the top (after the `mod error;` line):

```rust
mod error;
pub use error::{NamespaceKind, NsError, NsResult};

mod rootfs;
mod pivot;
mod mounts;
```

2. Add a new match arm for `Command::Chroot` (keep the existing `Command::Mount` arm unchanged):

```rust
Command::Chroot { command } => {
    use nix::sched::{unshare, CloneFlags};
    use nix::unistd::{execvp, chdir};
    use std::ffi::CString;

    // Step 1: Create a mount namespace
    unshare(CloneFlags::CLONE_NEWNS)
        .context("failed to create mount namespace")?;
    println!("Created mount namespace");

    // Step 2: Set up the minimal rootfs
    let rootfs = rootfs::setup_minimal_rootfs()
        .context("failed to set up rootfs")?;

    // Step 3: Mount essential filesystems (before pivot)
    mounts::mount_essential_filesystems(&rootfs)
        .context("failed to mount essential filesystems")?;

    // Step 4: Pivot into the new root
    pivot::pivot_root(&rootfs)
        .context("failed to pivot root")?;

    // Step 5: Execute the command (or default shell)
    let args: Vec<CString> = if command.is_empty() {
        // Default: run /bin/sh
        vec![CString::new("/bin/sh").unwrap()]
    } else {
        // Run the specified command
        command
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect()
    };

    // Change to root directory
    chdir("/")?;

    println!("Executing: {:?}", args);

    // Replace current process with the command
    execvp(&args[0], &args)
        .context("failed to exec command")?;

    // execvp never returns on success
    unreachable!();
}
```

**Understanding execvp**:

- `execvp` replaces the current process with a new program
- If successful, it **never returns** (the old process is gone)
- If it returns, an error occurred
- We use `unreachable!()` to tell the compiler this code path is impossible

### Step 6: Update dependencies

Add required dependencies to `crates/ns-tool/Cargo.toml`:

```toml
[dependencies]
nix = { version = "0.27", features = ["mount", "sched", "process", "fs"] }
libc = "0.2"
```

The `mount` and `fs` features might not be enabled yet.

3. Build the project (expect compilation):

```bash
cargo build -p ns-tool
```

Fix any compilation errors (likely missing imports or feature flags).

4. Run tests (expect success - GREEN phase):

```bash
# Basic test (no sudo needed)
cargo test -p ns-tool --test chroot_test test_chroot_requires_root

# Full test suite (requires sudo)
sudo -E cargo test -p ns-tool --test chroot_test -- --ignored --test-threads=1
```

**Expected output**:

```
running 4 tests
test test_chroot_requires_root ... ok
test test_chroot_creates_isolated_root ... ok
test test_chroot_proc_is_mounted ... ok
test test_chroot_busybox_works ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

Congratulations! You're now in the GREEN phase. Tests pass.

## Verify

### Automated Verification

```bash
# All chroot tests should pass
sudo -E cargo test -p ns-tool --test chroot_test -- --ignored --test-threads=1
```

### Manual Verification

Let's explore the isolated root filesystem:

#### 1. Enter the chroot and explore

```bash
# Start an interactive shell in the isolated root
sudo cargo run -p ns-tool -- chroot

# Now you're in the isolated root. Try these commands:
ls /
# Output: bin  dev  etc  lib  lib64  oldroot  proc  root  sbin  sys  tmp
# Notice: No /home, /usr, /var from the host!

# Verify /proc is mounted
mount | grep proc
# Output: proc on /proc type proc (rw,nosuid,nodev,noexec,relatime)

# Check process view
ps aux
# You should see a minimal process list

# Verify busybox
busybox --help
# Shows all available busybox applets

# Exit the chroot
exit
```

#### 2. Run commands without interactive shell

```bash
# List root directory
sudo cargo run -p ns-tool -- chroot -- ls -la /

# Check mount points
sudo cargo run -p ns-tool -- chroot -- mount

# Run ps (needs /proc mounted)
sudo cargo run -p ns-tool -- chroot -- ps aux

# Show kernel version (from /proc)
sudo cargo run -p ns-tool -- chroot -- cat /proc/version
```

#### 3. Verify isolation from host filesystem

```bash
# From host, create a test file
touch /tmp/host-file.txt

# Try to access it from the chroot
sudo cargo run -p ns-tool -- chroot -- ls /tmp/
# host-file.txt should NOT appear (different /tmp)

# Verify we can't see host directories
sudo cargo run -p ns-tool -- chroot -- ls /home
# Should fail: /home doesn't exist in our minimal rootfs
```

#### 4. Verify pivot_root worked (not just chroot)

```bash
# Inside the chroot, check mount table
sudo cargo run -p ns-tool -- chroot -- cat /proc/mounts

# Look for the root mount - should be our rootfs, not host root
# You should see something like:
# /dev/sda1 / ext4 rw,relatime 0 0 (or similar)
# NOT the host's original root mount
```

#### 5. Test namespace isolation

```bash
# Start a shell in the chroot
sudo cargo run -p ns-tool -- chroot

# Inside the chroot:
readlink /proc/self/ns/mnt
# Output: mnt:[4026532199] (some inode number)

# In another terminal on the host:
readlink /proc/self/ns/mnt
# Output: mnt:[4026531841] (different inode number)

# This proves we're in a different mount namespace
exit
```

### What You Should Observe

1. **Isolated filesystem**: Only see `/bin`, `/proc`, `/dev`, etc. - not host `/home`, `/usr`
2. **Working /proc**: Commands like `ps` and `mount` work correctly
3. **Functional busybox**: All standard utilities available via symlinks
4. **Different mount namespace**: Different `mnt:[inode]` than host
5. **No escape possible**: Can't access host files via any path

## Clean Up

The rootfs persists in `/tmp/ns-tool-rootfs` for reuse. To clean it up:

```bash
# Remove the rootfs directory
sudo rm -rf /tmp/ns-tool-rootfs

# Or use our cleanup function
sudo cargo run -p ns-tool -- cleanup-rootfs
# (if you add a Cleanup command variant)
```

**Note**: Mount namespaces are automatically cleaned up when the process exits. No manual cleanup needed for namespaces.

## Common Errors

### 1. `Operation not permitted` when creating mount namespace

**Symptom**:
```
Error: failed to create mount namespace
Caused by: Operation not permitted (os error 1)
```

**Cause**: Creating namespaces requires root privileges (or user namespaces).

**Fix**: Run with sudo:
```bash
sudo cargo run -p ns-tool -- chroot
```

### 2. `Device or resource busy` during pivot_root

**Symptom**:
```
Error: pivot_root failed: Device or resource busy
```

**Cause**: Current working directory is not inside new_root, or old_root has active mounts.

**Fix**: Our implementation handles this by:
1. Changing to new_root before pivot_root
2. Using `MNT_DETACH` to lazily unmount old root

If you still see this, check:
```bash
# See what's mounted
mount | grep ns-tool
```

### 3. `No such file or directory` when exec'ing command

**Symptom**:
```
Error: failed to exec command
Caused by: No such file or directory
```

**Cause**: The command doesn't exist in the minimal rootfs, or busybox symlinks weren't created.

**Fix**:
1. Check busybox is installed: `ls -la /tmp/ns-tool-rootfs/bin/busybox`
2. Verify symlinks exist: `ls -la /tmp/ns-tool-rootfs/bin/`
3. For commands not in busybox, use: `busybox <command>`

Example:
```bash
# This works (busybox has ls)
sudo cargo run -p ns-tool -- chroot -- ls /

# This might not work (busybox might not have this)
sudo cargo run -p ns-tool -- chroot -- vim
```

### 4. `curl: command not found` during setup

**Symptom**:
```
Error: failed to download busybox (need curl or wget)
```

**Cause**: Neither `curl` nor `wget` is installed on your system.

**Fix**: Install one:
```bash
# Ubuntu/Debian
sudo apt install curl

# Fedora/RHEL
sudo dnf install curl

# Arch
sudo pacman -S curl
```

Alternatively, manually download busybox:
```bash
mkdir -p /tmp/ns-tool-rootfs/bin
cd /tmp/ns-tool-rootfs/bin
wget https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox
chmod +x busybox
```

### 5. `Permission denied` when creating device files

**Symptom**:
```
Warning: Could not create /dev/null (permission denied)
```

**Cause**: Creating device files with `mknod` requires `CAP_MKNOD` capability, which might be restricted even with sudo in some environments.

**Fix**: This is often a non-fatal warning. Many operations will still work. If you need full `/dev` support, run in a less restricted environment or use:
```bash
# Run with full privileges
sudo -E cargo run -p ns-tool -- chroot
```

If still failing, the issue might be:
- SELinux policies blocking mknod
- Running in a container that restricts mknod
- AppArmor profile restrictions

### 6. `Invalid argument` from pivot_root

**Symptom**:
```
Error: pivot_root failed: Invalid argument (os error 22)
```

**Cause**: Several possible issues:
1. `new_root` is not a mount point
2. `new_root` and `put_old` are on the same filesystem
3. Not in a mount namespace

**Fix**: Our implementation handles #1 by bind-mounting new_root to itself. For #2 and #3:

```bash
# Verify you're in a mount namespace
readlink /proc/self/ns/mnt
# Should show different inode after unshare

# Verify new_root is a mount point
findmnt /tmp/ns-tool-rootfs
# Should show as a mount point after bind mount
```

### 7. Tests fail: `binary not found`

**Symptom**:
```
thread 'test_chroot_creates_isolated_root' panicked at 'called `Result::unwrap()` on an `Err` value: CargoError'
```

**Cause**: ns-tool binary not built yet.

**Fix**:
```bash
# Build first
cargo build -p ns-tool

# Then test
sudo -E cargo test -p ns-tool --test chroot_test -- --ignored
```

## Notes

### Why This Matters for Containers

This lesson demonstrates the core of how container runtimes isolate the filesystem:

1. **Docker/Podman**: Use the same pivot_root technique to enter OCI bundle rootfs
2. **Container images**: Are just tarballs of rootfs contents (like our minimal setup)
3. **Image layers**: Are stacked using overlay filesystems (advanced topic)
4. **OCI runtime spec**: Defines how to set up the rootfs before calling pivot_root

### Security Implications

**pivot_root vs chroot security**:

- `chroot()`: Can be escaped with sufficient privileges (see concept section)
- `pivot_root()`: Cannot be escaped - the old root is completely unmounted
- Container escapes typically exploit kernel bugs, not pivot_root itself

**Our minimal rootfs is intentionally simple**:

- No package manager
- No system services (systemd, etc.)
- No user accounts (besides root)
- No network tools (besides busybox basics)

Real container images add these layers as needed.

### Busybox vs Full Distribution

**Busybox (~1-2MB)**:
- Single binary, statically linked
- Implements ~400 common Unix utilities
- Perfect for containers and embedded systems
- Used by Alpine Linux

**Full distribution rootfs (100-1000MB+)**:
- Separate binaries for each utility
- Shared library dependencies
- Package manager (apt, dnf, pacman)
- System services and daemons

For learning, busybox is perfect. For production, you'd typically use a real distribution.

### Advanced: Overlay Filesystems

Container images use overlay filesystems to layer changes:

```
┌─────────────────┐
│   Container     │  (read-write layer)
│   Changes       │
├─────────────────┤
│   App Layer     │  (read-only)
├─────────────────┤
│   Base Image    │  (read-only)
└─────────────────┘
```

We'll explore this in the OCI lessons.

### Links to Official Documentation

- [pivot_root(2) man page](https://man7.org/linux/man-pages/man2/pivot_root.2.html) - Detailed syscall documentation
- [mount_namespaces(7) man page](https://man7.org/linux/man-pages/man7/mount_namespaces.7.html) - Mount namespace behavior
- [BusyBox](https://busybox.net/) - The Swiss Army Knife of Embedded Linux
- [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec) - How real containers work
- [Linux device numbers](https://www.kernel.org/doc/Documentation/admin-guide/devices.txt) - Official device major/minor assignments

### Differences from Real Container Runtimes

Our implementation is simplified for learning. Real runtimes like `runc` add:

1. **More namespaces**: Combined PID, network, IPC, UTS namespaces
2. **Cgroups**: Resource limits (CPU, memory, I/O)
3. **Security**: Seccomp filters, AppArmor/SELinux profiles, capabilities
4. **Networking**: Virtual ethernet pairs, bridges, iptables rules
5. **Storage**: Overlay/AUFS for layered filesystems
6. **OCI compliance**: Full runtime specification implementation

We'll build toward these in later lessons.

## Summary

In this lesson, you learned:

1. **pivot_root vs chroot**: Why containers use pivot_root for true filesystem isolation
2. **Minimal rootfs structure**: Essential directories and files needed for a working system
3. **BusyBox**: How a single static binary can provide a complete Unix userland
4. **Essential mounts**: Why /proc, /dev, and /sys are necessary
5. **Safe pivot_root**: The multi-step process to safely change the root filesystem
6. **Testing rootfs**: How to verify filesystem isolation programmatically

You built a `chroot` subcommand that creates a complete isolated filesystem environment - the foundation of container filesystem isolation.

**Key takeaways**:

- `pivot_root` provides stronger isolation than `chroot`
- A minimal Linux system needs surprisingly little (1-2MB with busybox)
- Mount namespaces + pivot_root = filesystem container
- This is fundamentally how Docker/Podman isolate filesystems

## Relationship to Lesson 04

This lesson builds on lesson 04 (Mount Namespace):

- **Lesson 04** teaches how to create a mount namespace and demonstrates mount isolation with a simple tmpfs mount
- **Lesson 05** (this lesson) extends that knowledge by adding a complete isolated filesystem with `pivot_root`

Both commands coexist in the CLI:
- `ns-tool mount`: Simple mount namespace isolation (lesson 04)
- `ns-tool chroot`: Complete filesystem isolation with pivot_root (lesson 05)

This is intentional - the `mount` command remains a useful standalone example, while the `chroot` command demonstrates a production-like container filesystem. Keeping both allows learners to understand the progression from simple namespace isolation to complete filesystem isolation.

## Next

`06-netns-basics.md` - Create network namespaces and configure isolated network stacks (loopback interface, network isolation)
