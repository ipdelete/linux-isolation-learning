// Tests for error handling patterns
// Lesson: docs/00-foundations/05-error-handling.md
//
// TDD Workflow:
// 1. These tests verify the error handling behavior of ns-tool
// 2. The error module in src/error.rs contains unit tests for error types
// 3. Run with: cargo test -p ns-tool --test error_test
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
        .stderr(predicate::str::contains(
            "failed to read namespace directory",
        ));
}

// Unit tests for error module are in src/error.rs
// We test the CLI behavior here, and unit test the error types in src/error.rs
