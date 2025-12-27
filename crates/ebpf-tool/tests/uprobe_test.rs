// Tests for the `uprobe` subcommand
// Lesson: docs/04-ebpf/05-uprobes.md
//
// TDD Workflow:
// 1. Write tests below FIRST (RED)
// 2. Implement code in src/main.rs and ebpf-tool-ebpf/src/uprobe.rs (GREEN)
//
// Uprobes allow tracing userspace functions in binaries and shared libraries.
// Unlike kprobes (kernel functions), uprobes attach to functions in ELF binaries.
//
// Usage: ebpf-tool uprobe <binary> <function> [-d duration]
//
// Example: ebpf-tool uprobe /lib/x86_64-linux-gnu/libc.so.6 malloc -d 5
//
// NOTE: Root-required tests check `Uid::effective().is_root()` and skip if not root.
// Run with: sudo -E cargo test -p ebpf-tool

use assert_cmd::Command;
use predicates::prelude::*;

// =============================================================================
// Help and Argument Validation Tests (no root required)
// =============================================================================

#[test]
fn test_uprobe_help() {
    // TODO: Verify the uprobe subcommand shows helpful usage information
    //
    // The help text should explain:
    // - What uprobes are (tracing userspace functions)
    // - Required arguments: <binary> and <function>
    // - Optional arguments: -d/--duration
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool").unwrap()
    // - Add args: ["uprobe", "--help"]
    // - Assert success and check stdout contains key information
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["uprobe", "--help"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("binary"))
    //    .stdout(predicate::str::contains("function"))
    //    .stdout(predicate::str::contains("duration"));

    todo!("Implement test for uprobe help text")
}

#[test]
fn test_uprobe_requires_binary_arg() {
    // TODO: Verify that the binary argument is required
    //
    // Running `ebpf-tool uprobe` without arguments should fail
    // with an error message indicating the missing <binary> argument.
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool").unwrap()
    // - Add arg: "uprobe" (no binary or function)
    // - Assert failure (non-zero exit code)
    // - Check stderr contains error about missing argument
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.arg("uprobe")
    //    .assert()
    //    .failure()
    //    .stderr(predicate::str::contains("binary")
    //        .or(predicate::str::contains("required")));

    todo!("Implement test for missing binary argument")
}

#[test]
fn test_uprobe_requires_function_arg() {
    // TODO: Verify that the function argument is required
    //
    // Running `ebpf-tool uprobe /bin/ls` (with binary but no function)
    // should fail with an error message about the missing <function> argument.
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool").unwrap()
    // - Add args: ["uprobe", "/bin/ls"] (binary but no function)
    // - Assert failure (non-zero exit code)
    // - Check stderr contains error about missing argument
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["uprobe", "/bin/ls"])
    //    .assert()
    //    .failure()
    //    .stderr(predicate::str::contains("function")
    //        .or(predicate::str::contains("required")));

    todo!("Implement test for missing function argument")
}

// =============================================================================
// Root-Required Tests (skip if not running as root)
// =============================================================================

/// Helper function to check if running as root.
/// Tests that require root should call this and return early if false.
fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

#[test]
fn test_uprobe_attaches_to_libc() {
    // TODO: Verify that uprobe can attach to a libc function
    //
    // This test attaches a uprobe to a common libc function like `malloc`
    // and verifies the attachment succeeds. This requires root privileges.
    //
    // Hints:
    // - Skip if not root: if !is_root() { return; }
    // - Find libc path: usually /lib/x86_64-linux-gnu/libc.so.6 or similar
    //   (or use `ldd /bin/ls | grep libc` to find it)
    // - Use a short duration (-d 1) for quick test
    // - Assert success or check for expected output
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_uprobe_attaches_to_libc: requires root");
    //     return;
    // }
    //
    // // Find libc path (common locations)
    // let libc_path = std::path::Path::new("/lib/x86_64-linux-gnu/libc.so.6");
    // if !libc_path.exists() {
    //     eprintln!("Skipping: libc not found at expected path");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["uprobe", libc_path.to_str().unwrap(), "malloc", "-d", "1"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("Attaching uprobe"));

    todo!("Implement test for uprobe attachment to libc")
}

#[test]
fn test_uprobe_shows_events() {
    // TODO: Verify that uprobe logs events when the traced function is called
    //
    // This test attaches to a libc function and triggers it, then verifies
    // that events are logged. This requires root privileges.
    //
    // Hints:
    // - Skip if not root: if !is_root() { return; }
    // - Attach to a frequently-called function like `malloc` or `write`
    // - Run for a short duration (1-2 seconds)
    // - In a real scenario, you might spawn a child process that calls
    //   the traced function to generate events
    // - Check output contains event information (PID, function name, etc.)
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_uprobe_shows_events: requires root");
    //     return;
    // }
    //
    // let libc_path = "/lib/x86_64-linux-gnu/libc.so.6";
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["uprobe", libc_path, "malloc", "-d", "2"])
    //    .assert()
    //    .success();
    // // Verify events are logged (format depends on implementation)
    // // .stdout(predicate::str::contains("uprobe event"));

    todo!("Implement test for uprobe event logging")
}

#[test]
fn test_uprobe_invalid_binary() {
    // TODO: Verify appropriate error when binary path does not exist
    //
    // Trying to attach a uprobe to a non-existent binary should fail
    // with a clear error message. This requires root to attempt the
    // eBPF operation (non-root fails earlier with permission error).
    //
    // Hints:
    // - Skip if not root: if !is_root() { return; }
    // - Use a path that definitely doesn't exist: "/nonexistent/binary"
    // - Assert failure (non-zero exit code)
    // - Check stderr contains helpful error (e.g., "not found", "no such file")
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_uprobe_invalid_binary: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["uprobe", "/nonexistent/binary/path", "some_function", "-d", "1"])
    //    .assert()
    //    .failure()
    //    .stderr(predicate::str::contains("not found")
    //        .or(predicate::str::contains("No such file"))
    //        .or(predicate::str::contains("does not exist")));

    todo!("Implement test for invalid binary path error")
}

#[test]
fn test_uprobe_invalid_function() {
    // TODO: Verify appropriate error when function does not exist in binary
    //
    // Trying to attach a uprobe to a function that doesn't exist in the
    // binary should fail with a clear error message.
    //
    // Hints:
    // - Skip if not root: if !is_root() { return; }
    // - Use a valid binary (e.g., /bin/ls) but invalid function name
    // - Use a function name that definitely doesn't exist: "nonexistent_fn_xyz"
    // - Assert failure (non-zero exit code)
    // - Check stderr contains helpful error about the function not being found
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_uprobe_invalid_function: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["uprobe", "/bin/ls", "nonexistent_function_xyz", "-d", "1"])
    //    .assert()
    //    .failure()
    //    .stderr(predicate::str::contains("function")
    //        .or(predicate::str::contains("symbol"))
    //        .or(predicate::str::contains("not found")));

    todo!("Implement test for invalid function name error")
}
