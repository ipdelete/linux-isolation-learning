# Unsafe Boundaries: Safe Wrappers for Linux Syscalls

## Goal

Learn when and how to use `unsafe` code for Linux isolation syscalls that are not fully wrapped by the `nix` crate. You will create a minimal, well-documented safe wrapper around a raw `libc` syscall (`pivot_root`), demonstrating the pattern of pushing unsafe to the smallest possible helper function.

**Deliverable**: A tested safe wrapper function for `pivot_root` in `crates/ns-tool/src/syscall.rs`.

**Estimated time**: 30-40 minutes

## Prereqs

- Completed `05-error-handling.md`
- Familiarity with how `nix` wraps syscalls (see `01-rust-syscall-basics.md`)
- Basic understanding of Rust's `unsafe` keyword
- `sudo` access for running tests

## Concepts: When Do We Need Unsafe?

Before writing code, let's understand when `unsafe` becomes necessary.

### The Landscape of Syscall Access in Rust

```
Most Safe ─────────────────────────────────────────────► Least Safe

std library    nix crate      libc + unsafe     raw syscall asm
(File, Process)  (unshare,      (pivot_root,      (rarely needed)
                  setns)        older syscalls)
```

**Prefer this order:**
1. **Standard library** - Use when available (file I/O, basic process control)
2. **nix crate** - Safe wrappers for most Linux syscalls
3. **libc + unsafe** - When nix does not provide what you need
4. **Raw syscall** - Almost never needed; libc handles this

### Why Some Syscalls Need Unsafe

The `nix` crate provides safe wrappers for many Linux syscalls, but not all. You need `unsafe` when:

1. **nix does not wrap the syscall** - Examples: `pivot_root`, some namespace operations in older nix versions
2. **You need precise control** - Raw pointers, specific memory layouts
3. **The syscall is new or obscure** - Kernel features may outpace crate updates

### The pivot_root Example

`pivot_root` is a perfect teaching example because:
- It is essential for container filesystem isolation
- It was not in `nix` for a long time (added in nix 0.27+)
- It has specific safety requirements we must document
- It demonstrates the "safe wrapper" pattern clearly

```
pivot_root(new_root, put_old)
    |
    ├── Changes the root mount for the calling process
    ├── Moves old root to put_old directory
    └── Critical for container filesystem isolation
```

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/unsafe_wrapper_test.rs`

First, create the test file to drive our implementation.

### Step 1: Create the Test File

Create a new file at `crates/ns-tool/tests/unsafe_wrapper_test.rs`:

```rust
//! Tests for safe syscall wrappers
//!
//! These tests verify our safe wrappers around unsafe libc calls.
//! Run with: sudo -E cargo test -p ns-tool --test unsafe_wrapper_test

use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

// We'll import our wrapper once we create it
// use ns_tool::syscall::pivot_root;

/// Test that pivot_root wrapper validates paths exist
#[test]
fn test_pivot_root_rejects_nonexistent_path() {
    // TODO: Implement test
    //
    // This test should verify that our safe wrapper returns an error
    // when given paths that don't exist, rather than passing invalid
    // pointers to the kernel.
    //
    // Expected behavior:
    // - pivot_root("/nonexistent", "/also_nonexistent") returns Err
    // - Error message should indicate which path was invalid

    todo!("Implement: test that pivot_root rejects nonexistent paths")
}

/// Test that pivot_root wrapper handles path conversion correctly
#[test]
fn test_pivot_root_path_to_cstring() {
    // TODO: Implement test
    //
    // Verify that paths are correctly converted to CStrings.
    // A path containing a null byte should be rejected.
    //
    // Expected behavior:
    // - Normal paths convert successfully
    // - Paths with embedded nulls return an error

    todo!("Implement: test CString conversion for paths")
}

/// Test the actual pivot_root syscall in a mount namespace
#[test]
#[ignore] // Requires root and mount namespace setup
fn test_pivot_root_in_mount_namespace() {
    // TODO: Implement test
    //
    // This is an integration test that actually calls pivot_root.
    // It requires:
    // 1. Root privileges
    // 2. A new mount namespace (unshare CLONE_NEWNS)
    // 3. A valid new_root that is a mount point
    // 4. A put_old directory inside new_root
    //
    // Test approach:
    // 1. Create temp directory structure
    // 2. Enter new mount namespace
    // 3. Bind mount new_root to itself (makes it a mount point)
    // 4. Call pivot_root
    // 5. Verify "/" changed

    todo!("Implement: integration test for pivot_root")
}

/// Test that our wrapper returns proper error types
#[test]
fn test_pivot_root_error_types() {
    // TODO: Implement test
    //
    // Verify that errors from the syscall are properly converted
    // to our error type with useful context.
    //
    // Expected: Err variant contains errno information

    todo!("Implement: test error type conversion")
}
```

### Step 2: Run the Tests (Expect Failure)

```bash
cargo test -p ns-tool --test unsafe_wrapper_test 2>&1
```

**Expected output**: Compilation error because the test file references a module that does not exist yet. This is the RED phase - we have failing tests that will drive our implementation.

```
error[E0433]: failed to resolve: could not find `syscall` in `ns_tool`
```

## Build (Green)

Now we implement the safe wrapper to make our tests pass.

**Implementation file**: `crates/ns-tool/src/syscall.rs`

### Step 1: Create the Syscall Module

Create `crates/ns-tool/src/syscall.rs`:

```rust
//! Safe wrappers around Linux syscalls not provided by nix.
//!
//! # Design Philosophy
//!
//! This module follows the principle of **minimal unsafe surface area**:
//! - Unsafe blocks are as small as possible
//! - All safety invariants are documented
//! - Safe public functions validate inputs before calling unsafe code
//! - Errors are converted to idiomatic Rust Result types
//!
//! # Pattern: Safe Wrapper
//!
//! ```text
//! Public safe function (validates inputs, converts types)
//!     │
//!     └── Private unsafe helper (minimal, documented invariants)
//!             │
//!             └── libc syscall
//! ```

use std::ffi::CString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use anyhow::{Context, Result};

/// Changes the root mount and puts the old root at a new location.
///
/// This is a safe wrapper around the `pivot_root(2)` syscall.
///
/// # Arguments
///
/// * `new_root` - Path to the new root filesystem. Must be a mount point.
/// * `put_old` - Path where the old root will be moved. Must be under `new_root`.
///
/// # Errors
///
/// Returns an error if:
/// - Either path does not exist
/// - Either path contains a null byte
/// - The syscall fails (see `pivot_root(2)` man page for errno values)
///
/// # Example
///
/// ```no_run
/// use ns_tool::syscall::pivot_root;
/// use std::path::Path;
///
/// // After setting up mount namespace and bind mounts:
/// pivot_root(
///     Path::new("/tmp/newroot"),
///     Path::new("/tmp/newroot/oldroot")
/// ).expect("pivot_root failed");
/// ```
///
/// # Safety Considerations (internal)
///
/// The actual unsafe call is isolated in `pivot_root_raw`. This function
/// ensures all preconditions are met before calling the unsafe code.
pub fn pivot_root(new_root: &Path, put_old: &Path) -> Result<()> {
    // Validate paths exist before we convert to CString
    // This provides better error messages than letting the syscall fail
    if !new_root.exists() {
        anyhow::bail!(
            "pivot_root: new_root path does not exist: {}",
            new_root.display()
        );
    }
    if !put_old.exists() {
        anyhow::bail!(
            "pivot_root: put_old path does not exist: {}",
            put_old.display()
        );
    }

    // Convert paths to CStrings for the syscall
    // CString::new will fail if the path contains interior null bytes
    let new_root_cstr = path_to_cstring(new_root)
        .with_context(|| format!("invalid new_root path: {}", new_root.display()))?;
    let put_old_cstr = path_to_cstring(put_old)
        .with_context(|| format!("invalid put_old path: {}", put_old.display()))?;

    // SAFETY: We have validated:
    // 1. Both paths exist on the filesystem
    // 2. Both CStrings are valid (no interior nulls, null-terminated)
    // 3. The pointers will remain valid for the duration of the syscall
    let ret = unsafe { pivot_root_raw(new_root_cstr.as_ptr(), put_old_cstr.as_ptr()) };

    if ret == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
            .with_context(|| {
                format!(
                    "pivot_root({}, {}) failed",
                    new_root.display(),
                    put_old.display()
                )
            })
    }
}

/// Convert a Path to a CString.
///
/// Returns an error if the path contains interior null bytes.
fn path_to_cstring(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|e| anyhow::anyhow!("path contains null byte: {}", e))
}

/// Raw pivot_root syscall wrapper.
///
/// # Safety
///
/// Caller must ensure:
/// - `new_root` is a valid pointer to a null-terminated C string
/// - `put_old` is a valid pointer to a null-terminated C string
/// - Both pointers remain valid for the duration of the call
/// - The calling process has appropriate privileges (CAP_SYS_ADMIN)
/// - `new_root` is a mount point
/// - `put_old` is at or under `new_root`
///
/// See `pivot_root(2)` man page for complete requirements.
#[inline]
unsafe fn pivot_root_raw(
    new_root: *const libc::c_char,
    put_old: *const libc::c_char,
) -> libc::c_int {
    // pivot_root is not directly exposed by libc crate on all platforms,
    // so we use syscall() directly
    libc::syscall(libc::SYS_pivot_root, new_root, put_old) as libc::c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_to_cstring_valid() {
        let path = Path::new("/tmp/test");
        let result = path_to_cstring(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), b"/tmp/test");
    }

    #[test]
    fn test_path_to_cstring_with_null() {
        // Create a path with an embedded null byte
        let bytes = b"/tmp/test\x00hidden";
        let path = PathBuf::from(std::ffi::OsStr::from_bytes(bytes));
        let result = path_to_cstring(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_pivot_root_nonexistent_new_root() {
        let result = pivot_root(
            Path::new("/nonexistent_path_12345"),
            Path::new("/tmp"),
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("new_root"));
        assert!(err_msg.contains("does not exist"));
    }

    #[test]
    fn test_pivot_root_nonexistent_put_old() {
        // /tmp exists, but put_old does not
        let result = pivot_root(
            Path::new("/tmp"),
            Path::new("/nonexistent_path_12345"),
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("put_old"));
        assert!(err_msg.contains("does not exist"));
    }
}
```

### Step 2: Export the Module

Update `crates/ns-tool/src/main.rs` to include the new module. Add this line near the top of the file, after the imports:

```rust
pub mod syscall;
```

The beginning of `main.rs` should look like:

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod syscall;

#[derive(Parser)]
// ... rest of file
```

### Step 3: Update the Test File

Now update `crates/ns-tool/tests/unsafe_wrapper_test.rs` to use the actual implementation:

```rust
//! Tests for safe syscall wrappers
//!
//! These tests verify our safe wrappers around unsafe libc calls.
//! Run with: sudo -E cargo test -p ns-tool --test unsafe_wrapper_test

use std::path::Path;

// Import our wrapper
use ns_tool::syscall::pivot_root;

/// Test that pivot_root wrapper validates paths exist
#[test]
fn test_pivot_root_rejects_nonexistent_new_root() {
    let result = pivot_root(
        Path::new("/this_path_definitely_does_not_exist_12345"),
        Path::new("/tmp"),
    );

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("does not exist"),
        "Expected 'does not exist' in error, got: {}",
        err_msg
    );
}

/// Test that pivot_root wrapper validates put_old exists
#[test]
fn test_pivot_root_rejects_nonexistent_put_old() {
    let result = pivot_root(
        Path::new("/tmp"),  // exists
        Path::new("/this_path_definitely_does_not_exist_12345"),
    );

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("does not exist"),
        "Expected 'does not exist' in error, got: {}",
        err_msg
    );
}

/// Test that error messages are helpful
#[test]
fn test_pivot_root_error_includes_path() {
    let bad_path = "/nonexistent_test_path_xyz";
    let result = pivot_root(Path::new(bad_path), Path::new("/tmp"));

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains(bad_path),
        "Expected path '{}' in error message, got: {}",
        bad_path,
        err_msg
    );
}

/// Test the actual pivot_root syscall in a mount namespace
///
/// This test is ignored by default because it requires:
/// - Root privileges
/// - A properly set up mount namespace
/// - Careful cleanup
#[test]
#[ignore]
fn test_pivot_root_in_mount_namespace() {
    // This test would be implemented in the mount namespace lesson
    // Here we document what a full integration test would look like:
    //
    // 1. unshare(CLONE_NEWNS) - create new mount namespace
    // 2. mount(MS_PRIVATE) on / - prevent mount propagation
    // 3. Create temp directory structure:
    //    /tmp/newroot/
    //    /tmp/newroot/oldroot/
    // 4. Bind mount /tmp/newroot to itself (makes it a mount point)
    // 5. pivot_root("/tmp/newroot", "/tmp/newroot/oldroot")
    // 6. chdir("/")
    // 7. umount("/oldroot", MNT_DETACH)
    // 8. Verify new root

    todo!("Full integration test - see mount namespace lesson")
}
```

### Step 4: Create the Library Crate Interface

For the test imports to work, we need to make ns-tool usable as a library. Create `crates/ns-tool/src/lib.rs`:

```rust
//! ns-tool library interface
//!
//! This module exposes the syscall wrappers for use in tests
//! and potentially by other crates in this workspace.

pub mod syscall;
```

Then update `crates/ns-tool/Cargo.toml` to build both the binary and library:

The file already has the right structure - Cargo will automatically detect both `src/main.rs` and `src/lib.rs`.

### Step 5: Fix the Module Structure

Remove the `pub mod syscall;` line from `main.rs` since it is now in `lib.rs`. Update `main.rs` to use the library:

Add at the top of `main.rs`:
```rust
use ns_tool::syscall;  // If you need to use syscall module from main
```

Or simply keep `main.rs` focused on the CLI, and the syscall module is accessed through the library.

### Step 6: Run Tests (Expect Success)

```bash
cargo test -p ns-tool --test unsafe_wrapper_test
```

**Expected output**: All tests pass (GREEN phase).

```
running 3 tests
test test_pivot_root_rejects_nonexistent_new_root ... ok
test test_pivot_root_rejects_nonexistent_put_old ... ok
test test_pivot_root_error_includes_path ... ok
```

Also run the unit tests in the syscall module:

```bash
cargo test -p ns-tool syscall
```

## Verify

**Automated verification**:
```bash
# All ns-tool tests pass
cargo test -p ns-tool

# Specific unsafe wrapper tests
cargo test -p ns-tool --test unsafe_wrapper_test

# Unit tests in the syscall module
cargo test -p ns-tool syscall
```

**Manual verification** - examine the code structure:

```bash
# Verify the module structure
ls -la crates/ns-tool/src/

# Should show:
# lib.rs      <- library interface
# main.rs     <- CLI binary
# syscall.rs  <- our new safe wrappers

# Check that unsafe is minimal
grep -n "unsafe" crates/ns-tool/src/syscall.rs
```

You should see `unsafe` appears only:
1. In the call site within the safe `pivot_root` function
2. In the function signature of `pivot_root_raw`

This demonstrates the pattern: unsafe is pushed to the smallest possible scope.

## Clean Up

No cleanup required for this lesson. The files we created are part of the project structure:
- `crates/ns-tool/src/syscall.rs` - keeps safe wrappers
- `crates/ns-tool/src/lib.rs` - library interface
- `crates/ns-tool/tests/unsafe_wrapper_test.rs` - integration tests

## Common Errors

1. **`error[E0433]: failed to resolve: use of undeclared crate or module`**
   - Cause: The `lib.rs` file is missing or does not export the `syscall` module
   - Fix: Ensure `crates/ns-tool/src/lib.rs` exists with `pub mod syscall;`

2. **`error: cannot find macro `todo` in this scope`**
   - Cause: Using older Rust edition
   - Fix: Ensure `edition = "2021"` in Cargo.toml (already set in this project)

3. **`EPERM` (Operation not permitted) when testing actual pivot_root**
   - Cause: Not running as root or not in a mount namespace
   - Fix: Use `sudo -E cargo test` and ensure you are in a new mount namespace

4. **`EINVAL` from pivot_root syscall**
   - Cause: `new_root` is not a mount point, or `put_old` is not under `new_root`
   - Fix: Bind mount `new_root` to itself first: `mount --bind /path /path`

5. **Path with embedded null byte**
   - Cause: Unusual filenames or encoding issues
   - Fix: Our wrapper correctly rejects these with a clear error message

## The Unsafe Pattern in Detail

### Pattern: Minimal Unsafe Surface

```
                    ┌─────────────────────────────────────┐
                    │     Public API (100% Safe)          │
                    │                                      │
                    │  pub fn pivot_root(new: &Path,      │
                    │                    old: &Path)      │
                    │      -> Result<()>                  │
                    │                                      │
                    │  - Validates paths exist            │
                    │  - Converts to CString              │
                    │  - Handles errors idiomatically     │
                    └─────────────────┬───────────────────┘
                                      │
                    ┌─────────────────▼───────────────────┐
                    │  Private Unsafe Helper (minimal)    │
                    │                                      │
                    │  unsafe fn pivot_root_raw(          │
                    │      new: *const c_char,            │
                    │      old: *const c_char             │
                    │  ) -> c_int                         │
                    │                                      │
                    │  - Documents safety requirements    │
                    │  - Single syscall invocation        │
                    │  - No validation (caller's job)     │
                    └─────────────────┬───────────────────┘
                                      │
                    ┌─────────────────▼───────────────────┐
                    │         libc::syscall()             │
                    │     (kernel interface)              │
                    └─────────────────────────────────────┘
```

### Checklist for Safe Wrappers

When you create a safe wrapper around unsafe code, verify:

- [ ] **Unsafe block is minimal** - only the actual foreign call
- [ ] **Safety invariants documented** - what the caller must ensure
- [ ] **Inputs validated** - before entering unsafe code
- [ ] **Errors converted** - to idiomatic Rust types with context
- [ ] **Public API is safe** - users cannot cause undefined behavior
- [ ] **Tests cover edge cases** - invalid inputs, error conditions

## Notes

- The `nix` crate added `pivot_root` in version 0.27. If your project uses a newer version, you could use `nix::unistd::pivot_root` instead. However, this lesson teaches the pattern you will need for other syscalls not in nix.

- The `libc::syscall()` function lets you call any Linux syscall by number. This is the escape hatch when neither nix nor libc provides a direct wrapper.

- Always check the man page for a syscall before wrapping it. For pivot_root: `man 2 pivot_root`

- The pattern shown here applies to any syscall wrapper:
  1. Validate inputs in safe code
  2. Convert to C-compatible types
  3. Call unsafe helper with documented invariants
  4. Convert result to Rust error type

## Further Reading

- [The Rustonomicon - Unsafe](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html)
- [pivot_root(2) man page](https://man7.org/linux/man-pages/man2/pivot_root.2.html)
- [nix crate documentation](https://docs.rs/nix/latest/nix/)
- [libc crate documentation](https://docs.rs/libc/latest/libc/)

## Next

Move to namespace lessons: `../01-namespaces/01-pid-namespace.md` - Create your first isolated PID namespace using the safe wrappers and patterns you learned here.
