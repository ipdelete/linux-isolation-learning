# 05 Error Handling for Systems Programming

## Goal
Learn idiomatic Rust error handling patterns for syscalls and systems programming. You will create a dedicated error module that converts low-level errors (from `nix` and `std::io`) into meaningful, user-friendly application errors using both `anyhow` (for rapid prototyping) and `thiserror` (for library-quality types).

**Deliverable**: An `error.rs` module with custom error types, proper error conversion, and contextual error messages that help users understand what went wrong and why.

**Estimated time**: 30-40 minutes

## Prereqs
- Completed `00-setup-rust.md` (Rust toolchain installed)
- Completed `04-permissions-and-sudo.md` (understand when syscalls fail due to permissions)
- Familiarity with Rust `Result` and `Option` types
- `cargo run -q -p ns-tool -- proc` works

## Concepts

### Why Error Handling Matters in Systems Programming

When working with Linux syscalls, errors are not exceptional - they are expected. A syscall might fail because:
- The process lacks permissions (EPERM, EACCES)
- A resource does not exist (ENOENT)
- The operation is not supported (ENOSYS)
- The system is out of resources (ENOMEM, EMFILE)

Raw syscall errors like "Operation not permitted" or "errno 1" are cryptic. Good error handling transforms these into actionable messages like:

```
Error: Failed to create PID namespace
Caused by: Operation not permitted (are you running as root?)
```

### The Error Handling Spectrum

Rust offers a spectrum of error handling approaches:

| Approach | Use Case | Crate |
|----------|----------|-------|
| `Result<T, Box<dyn Error>>` | Quick prototypes | std |
| `anyhow::Result<T>` | Applications, CLIs | anyhow |
| Custom types with `thiserror` | Libraries, precise matching | thiserror |

For this project, we use:
- **`anyhow`** in `main.rs` for convenient error propagation and context
- **`thiserror`** for defining error types that downstream code can match on

### Error Flow in ns-tool

```
 nix::Error (syscall)     std::io::Error (file ops)
         \                      /
          \                    /
           v                  v
         NsError (our custom type)
                  |
                  v
         anyhow::Result (in main)
                  |
                  v
         User-friendly message
```

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/error_test.rs`

We will write tests that verify our error handling behavior. The test file already exists with TODO stubs waiting to be filled in.

### Step 1: Review the test file structure

Open `crates/ns-tool/tests/error_test.rs` to see the test structure:

```rust
// Tests for error handling patterns
// Lesson: docs/00-foundations/05-error-handling.md
//
// These tests verify that:
// 1. Errors from syscalls are properly converted to our error types
// 2. Error messages include helpful context
// 3. The CLI displays user-friendly error messages
//
// Note: Tests marked with #[cfg(target_os = "linux")] only run on Linux
// because they depend on /proc/self/ns which is Linux-specific.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_invalid_subcommand_shows_help() {
    // Test that invalid subcommands produce helpful error messages
    // This test works on any platform since it tests clap's error handling
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
#[cfg(target_os = "linux")]
fn test_proc_command_succeeds() {
    // The proc subcommand should work without root
    // This verifies our success path works correctly
    // Linux-only: requires /proc/self/ns
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("proc")
        .assert()
        .success()
        .stdout(predicate::str::contains("pid"));
}

#[test]
#[cfg(target_os = "linux")]
fn test_proc_command_shows_namespace_format() {
    // Verify output format: "name -> namespace:[inode]"
    // Linux-only: requires /proc/self/ns
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("proc")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\w+ -> \w+:\[\d+\]").unwrap());
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_proc_command_fails_on_non_linux() {
    // On non-Linux systems, the proc command should fail gracefully
    // with a helpful error message about the missing /proc filesystem
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("proc")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read namespace directory"));
}

// Unit tests for error module are in src/error.rs
// We test the CLI behavior here, and unit test the error types in src/error.rs
```

Note: The `#[cfg(target_os = "linux")]` attribute ensures Linux-specific tests only run on Linux. The non-Linux test verifies that the error handling works correctly even when the `/proc` filesystem is not available.

### Step 2: Run the tests (expect them to pass - we are testing existing behavior)

```bash
cargo test -p ns-tool --test error_test
```

These tests verify existing behavior. Now let us add tests for error scenarios.

### Step 3: Add tests for error conversion (these will be unit tests)

The error types themselves are best tested as unit tests within the module. We will write those in the Build phase.

## Build (Green)

**Implementation file**: `crates/ns-tool/src/error.rs` (already exists with stubs)
**Update file**: `crates/ns-tool/src/main.rs` (reference to error module)

### Step 1: Review error module dependencies

The dependencies are already added to `Cargo.toml`:
- `thiserror` is in `[workspace.dependencies]`
- The crate already imports `thiserror = { workspace = true }`

### Step 2: Review the error module structure

Open `crates/ns-tool/src/error.rs` to see the complete error type definitions:

```rust
//! Error types for ns-tool
//!
//! This module demonstrates idiomatic Rust error handling for systems programming:
//! - Custom error types using thiserror for precise error matching
//! - Automatic conversion from syscall errors (nix::Error, std::io::Error)
//! - Contextual information to help users understand what failed
//!
//! # Error Design Principles
//!
//! 1. **Be specific**: "Failed to create PID namespace" > "syscall failed"
//! 2. **Include context**: Which namespace? Which file path? What operation?
//! 3. **Preserve the cause**: Chain errors so users can see the root cause
//! 4. **Suggest fixes**: When possible, hint at solutions (e.g., "try running as root")

use std::path::PathBuf;
use thiserror::Error;

/// The namespace types we work with
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamespaceKind {
    Pid,
    Uts,
    Ipc,
    Mount,
    Net,
    User,
    Cgroup,
    Time,
}

impl std::fmt::Display for NamespaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceKind::Pid => write!(f, "PID"),
            NamespaceKind::Uts => write!(f, "UTS"),
            NamespaceKind::Ipc => write!(f, "IPC"),
            NamespaceKind::Mount => write!(f, "mount"),
            NamespaceKind::Net => write!(f, "network"),
            NamespaceKind::User => write!(f, "user"),
            NamespaceKind::Cgroup => write!(f, "cgroup"),
            NamespaceKind::Time => write!(f, "time"),
        }
    }
}

/// Errors that can occur when working with namespaces
#[derive(Debug, Error)]
pub enum NsError {
    /// Failed to create a new namespace with unshare(2)
    #[error("failed to create {kind} namespace")]
    CreateNamespace {
        kind: NamespaceKind,
        #[source]
        source: nix::Error,
    },

    /// Failed to join an existing namespace with setns(2)
    #[error("failed to join {kind} namespace from {path}")]
    JoinNamespace {
        kind: NamespaceKind,
        path: PathBuf,
        #[source]
        source: nix::Error,
    },

    /// Failed to fork a child process
    #[error("failed to fork child process")]
    Fork(#[source] nix::Error),

    /// Failed to read from /proc filesystem
    #[error("failed to read {path}")]
    ProcRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to set hostname (UTS namespace operation)
    #[error("failed to set hostname to '{hostname}'")]
    SetHostname {
        hostname: String,
        #[source]
        source: nix::Error,
    },

    /// Operation requires root privileges
    #[error("{operation} requires root privileges (try: sudo)")]
    PermissionDenied { operation: String },

    /// A namespace file does not exist
    #[error("namespace file not found: {path}")]
    NamespaceNotFound { path: PathBuf },
}

impl NsError {
    /// Create a CreateNamespace error with the given kind and source
    pub fn create_namespace(kind: NamespaceKind, source: nix::Error) -> Self {
        // Check if this is a permission error and provide a better message
        if source == nix::Error::EPERM || source == nix::Error::EACCES {
            return NsError::PermissionDenied {
                operation: format!("creating {} namespace", kind),
            };
        }
        NsError::CreateNamespace { kind, source }
    }

    /// Create a JoinNamespace error
    pub fn join_namespace(kind: NamespaceKind, path: PathBuf, source: nix::Error) -> Self {
        if source == nix::Error::EPERM || source == nix::Error::EACCES {
            return NsError::PermissionDenied {
                operation: format!("joining {} namespace", kind),
            };
        }
        if source == nix::Error::ENOENT {
            return NsError::NamespaceNotFound { path };
        }
        NsError::JoinNamespace { kind, path, source }
    }

    /// Create a ProcRead error
    pub fn proc_read(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        NsError::ProcRead {
            path: path.into(),
            source,
        }
    }
}

// Convenience type alias for functions that return our error type
pub type NsResult<T> = Result<T, NsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_kind_display() {
        assert_eq!(NamespaceKind::Pid.to_string(), "PID");
        assert_eq!(NamespaceKind::Net.to_string(), "network");
        assert_eq!(NamespaceKind::Mount.to_string(), "mount");
    }

    #[test]
    fn test_create_namespace_error_display() {
        let err = NsError::CreateNamespace {
            kind: NamespaceKind::Pid,
            source: nix::Error::EINVAL,
        };
        assert_eq!(err.to_string(), "failed to create PID namespace");
    }

    #[test]
    fn test_permission_error_suggests_sudo() {
        let err = NsError::PermissionDenied {
            operation: "creating PID namespace".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("root"));
        assert!(msg.contains("sudo"));
    }

    #[test]
    fn test_proc_read_error_includes_path() {
        let err = NsError::proc_read(
            "/proc/self/ns/pid",
            std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        );
        assert!(err.to_string().contains("/proc/self/ns/pid"));
    }

    #[test]
    fn test_eperm_becomes_permission_denied() {
        let err = NsError::create_namespace(NamespaceKind::Pid, nix::Error::EPERM);
        match err {
            NsError::PermissionDenied { operation } => {
                assert!(operation.contains("PID"));
            }
            _ => panic!("Expected PermissionDenied, got {:?}", err),
        }
    }

    #[test]
    fn test_join_namespace_error_with_path() {
        let err = NsError::JoinNamespace {
            kind: NamespaceKind::Net,
            path: PathBuf::from("/proc/1234/ns/net"),
            source: nix::Error::EINVAL,
        };
        let msg = err.to_string();
        assert!(msg.contains("network"));
        assert!(msg.contains("/proc/1234/ns/net"));
    }
}
```

### Step 3: Understand the main.rs integration

The `crates/ns-tool/src/main.rs` already declares the error module and demonstrates how to use it:

```rust
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod error;
pub use error::{NamespaceKind, NsError, NsResult};

#[derive(Parser)]
#[command(name = "ns-tool")]
#[command(about = "Namespace learning tool (Rust-first rewrite)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

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
    Setns,
    Proc,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // TODO: Implement PID namespace subcommand
        // ... (existing TODO comments remain the same)
        Command::Pid => todo!("Implement PID namespace - write tests first!"),

        // ... (other TODO commands remain the same)
        Command::Uts => todo!("Implement UTS namespace - write tests first!"),
        Command::Ipc => todo!("Implement IPC namespace - write tests first!"),
        Command::Mount => todo!("Implement mount namespace - write tests first!"),
        Command::Net => todo!("Implement network namespace - write tests first!"),
        Command::User => todo!("Implement user namespace - write tests first!"),
        Command::Cgroup => todo!("Implement cgroup namespace - write tests first!"),
        Command::Time => todo!("Implement time namespace - write tests first!"),
        Command::Setns => todo!("Implement setns - write tests first!"),

        // This is already implemented as a reference example
        Command::Proc => print_proc_ns()?,
    }

    Ok(())
}

fn print_proc_ns() -> Result<()> {
    let ns_path = "/proc/self/ns";

    // Using anyhow's Context trait to add context to errors
    let entries = std::fs::read_dir(ns_path)
        .with_context(|| format!("failed to read namespace directory: {}", ns_path))?;

    for entry in entries {
        let entry = entry.with_context(|| "failed to read directory entry")?;
        let name = entry.file_name();
        let target = std::fs::read_link(entry.path())
            .with_context(|| format!("failed to read symlink: {}", entry.path().display()))?;
        println!("{} -> {}", name.to_string_lossy(), target.display());
    }
    Ok(())
}
```

### Step 4: Run all tests

```bash
cargo test -p ns-tool
```

All tests should pass now.

## Understanding the Patterns

### Pattern 1: Using anyhow for Context

The `anyhow` crate provides the `Context` trait which lets you add context to any error:

```rust
use anyhow::{Context, Result};

fn read_namespace_info(pid: u32) -> Result<String> {
    let path = format!("/proc/{}/ns/pid", pid);

    let ns_path = std::fs::read_link(&path)
        .with_context(|| format!("failed to read namespace for PID {}", pid))?;

    ns_path
        .to_str()
        .context("namespace path is not valid UTF-8")
        .map(|s| s.to_string())
}
```

When this fails, the error message shows the full chain:
```
Error: failed to read namespace for PID 12345
Caused by: No such file or directory (os error 2)
```

### Pattern 2: Converting nix::Error to Custom Types

The `nix` crate returns `nix::Error` which wraps errno values. Our custom error types convert these:

```rust
use nix::sched::{unshare, CloneFlags};

fn create_pid_namespace() -> NsResult<()> {
    unshare(CloneFlags::CLONE_NEWPID)
        .map_err(|e| NsError::create_namespace(NamespaceKind::Pid, e))
}
```

The `create_namespace` constructor checks for `EPERM`/`EACCES` and returns a `PermissionDenied` error with a helpful message instead of the raw errno.

### Pattern 3: Error Matching in Tests

With `thiserror`, you can pattern match on specific error variants:

```rust
#[test]
fn test_permission_handling() {
    let result = create_pid_namespace();

    match result {
        Err(NsError::PermissionDenied { operation }) => {
            assert!(operation.contains("PID"));
        }
        Err(other) => panic!("Expected PermissionDenied, got: {}", other),
        Ok(_) => {} // Test running as root
    }
}
```

### Pattern 4: The Error Chain

When using `#[source]` in thiserror, errors form a chain:

```rust
#[derive(Debug, Error)]
pub enum NsError {
    #[error("failed to create {kind} namespace")]
    CreateNamespace {
        kind: NamespaceKind,
        #[source]  // This links to the underlying error
        source: nix::Error,
    },
}
```

Using `anyhow`, the full chain is printed:
```
Error: failed to create PID namespace

Caused by:
    Operation not permitted (os error 1)
```

## Verify

**Automated verification**:
```bash
# Run all ns-tool tests including the new error tests
cargo test -p ns-tool

# Run just the error module unit tests
cargo test -p ns-tool error::tests

# Run the integration tests
cargo test -p ns-tool --test error_test
```

**Manual verification**:
```bash
# Test the proc command (should work without root)
cargo run -q -p ns-tool -- proc

# Expected output format:
# cgroup -> cgroup:[4026531835]
# ipc -> ipc:[4026531839]
# mnt -> mnt:[4026531841]
# net -> net:[4026531840]
# pid -> pid:[4026531836]
# ...

# Test an invalid subcommand
cargo run -q -p ns-tool -- nonexistent

# Expected: error message from clap about invalid subcommand
```

## Clean Up

This lesson does not create any persistent resources. No cleanup is needed.

If you modified the workspace `Cargo.toml` to add `thiserror`, that change will persist (and is intended to).

## Common Errors

1. **"unresolved import `thiserror`"**
   - Cause: The `thiserror` dependency was not added to the Cargo.toml files
   - Fix: Add `thiserror = "1.0"` to `[workspace.dependencies]` and `thiserror = { workspace = true }` to the crate's dependencies

2. **"the trait `std::error::Error` is not implemented for `nix::Error`"**
   - Cause: Using an older version of `nix` that does not implement std::error::Error
   - Fix: Ensure you are using `nix = "0.29"` or later, which implements the Error trait

3. **"cannot find value `EPERM` in module `nix::Error`"**
   - Cause: nix error constants are accessed differently
   - Fix: Use `nix::Error::EPERM` (it is an associated constant on the Error type)

4. **Tests fail with "Operation not permitted"**
   - Cause: Some namespace tests require root privileges
   - Fix: This is expected for actual namespace creation. The error handling tests should work without root. If testing namespace creation, use `sudo cargo test`

5. **"module `error` is private"**
   - Cause: Forgot to add `pub use error::...` in main.rs or lib.rs
   - Fix: Add `pub use error::{NamespaceKind, NsError, NsResult};` after the `mod error;` declaration

## Notes

### When to Use anyhow vs thiserror

| Use `anyhow` when... | Use `thiserror` when... |
|---------------------|------------------------|
| Building an application/CLI | Building a library |
| Errors are displayed, not matched | Callers need to match on error variants |
| You want quick iteration | You want stable error types |
| Error context is more important than type | Error type is part of your API |

**In ns-tool, we use both**: `thiserror` defines our error types (so future library users can match on them), and `anyhow` in `main()` provides convenient context and display.

### Error Handling Philosophy

1. **Fail fast, fail clearly**: Prefer returning errors early with good context
2. **Do not panic in libraries**: Reserve `panic!` for truly unrecoverable states
3. **Avoid `.unwrap()` in production code**: Use `?` or explicit error handling
4. **Test error paths**: Error handling code needs tests too

### Relevant Documentation

- [The Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [anyhow crate documentation](https://docs.rs/anyhow)
- [thiserror crate documentation](https://docs.rs/thiserror)
- [nix crate error handling](https://docs.rs/nix/latest/nix/errno/index.html)
- [errno(3) man page](https://man7.org/linux/man-pages/man3/errno.3.html) - Unix error numbers

## Exercises

### Exercise 1: Add a New Error Variant (5 minutes)

Add a `MountError` variant to `NsError` for mount namespace operations:

```rust
#[error("failed to mount {source_path} at {target_path}")]
MountFailed {
    source_path: PathBuf,
    target_path: PathBuf,
    #[source]
    source: nix::Error,
}
```

Add a test that verifies the error message includes both paths.

### Exercise 2: Improve Error Context (10 minutes)

Update `print_proc_ns()` to use `NsError::proc_read()` instead of anyhow context. This demonstrates converting from anyhow-style to thiserror-style error handling.

Hint: You will need to change the return type to `NsResult<()>` and add a `.map_err()` call.

### Exercise 3: Test a Permission Error (10 minutes)

Write a test that verifies creating a namespace without root produces a `PermissionDenied` error. The test should:
1. Skip if running as root (use `nix::unistd::geteuid().is_root()`)
2. Attempt to call `unshare(CLONE_NEWPID)`
3. Verify the error is `PermissionDenied`

This is a real-world example of testing error paths in TDD style.

## Next
`06-unsafe-boundaries.md` - Learn when and how to use unsafe code safely, keeping unsafe blocks small and well-documented for syscall wrappers.
