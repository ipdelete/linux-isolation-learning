// Tests for the `check` subcommand (eBPF environment validation)
// Lesson: docs/04-ebpf/00-ebpf-setup.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: Most tests require root privileges to validate eBPF capabilities.
// Tests that require root will skip gracefully when run as non-root user.
// Run with: sudo -E cargo test -p ebpf-tool

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to check if running as root.
/// Tests that require root should call this and skip if not root.
fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

#[test]
fn test_check_help() {
    // TODO: Test that --help shows check subcommand info
    //
    // The check subcommand should have helpful documentation explaining
    // that it validates the eBPF environment including BTF, kernel version,
    // and permissions.
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add args: ["check", "--help"]
    // - Assert success
    // - Check stdout contains "Validate" (from the subcommand description)
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["check", "--help"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("Validate"));

    todo!("Implement test for check --help")
}

#[test]
fn test_check_runs_as_root() {
    // TODO: Test that check subcommand runs successfully with root privileges
    //
    // When run as root, the check command should complete successfully
    // and output diagnostic information about the eBPF environment.
    //
    // Hints:
    // - Use is_root() helper to skip if not root
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add arg: "check"
    // - Assert success (exit code 0)
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_check_runs_as_root: requires root privileges");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.arg("check")
    //    .assert()
    //    .success();

    todo!("Implement test for check running as root")
}

#[test]
fn test_check_shows_kernel_version() {
    // TODO: Test that check output includes kernel version information
    //
    // The check command should display the current kernel version
    // and indicate whether it meets the minimum requirements for eBPF.
    // A good eBPF experience requires kernel 5.8+ (for CAP_BPF, ring buffers, etc.)
    //
    // Hints:
    // - Use is_root() helper to skip if not root
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add arg: "check"
    // - Assert stdout contains "Kernel" or "kernel"
    // - Optionally check for version pattern like "5." or "6."
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_check_shows_kernel_version: requires root privileges");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.arg("check")
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::is_match("[Kk]ernel").unwrap());

    todo!("Implement test for kernel version in check output")
}

#[test]
fn test_check_shows_btf_status() {
    // TODO: Test that check output includes BTF availability status
    //
    // BTF (BPF Type Format) is essential for CO-RE (Compile Once, Run Everywhere).
    // The check command should verify that /sys/kernel/btf/vmlinux exists
    // and report its status.
    //
    // Hints:
    // - Use is_root() helper to skip if not root
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add arg: "check"
    // - Assert stdout contains "BTF" (case-sensitive, it's an acronym)
    // - The output might show path "/sys/kernel/btf/vmlinux" or just status
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_check_shows_btf_status: requires root privileges");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.arg("check")
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("BTF"));

    todo!("Implement test for BTF status in check output")
}

#[test]
fn test_check_shows_permissions() {
    // TODO: Test that check output includes permission/capability information
    //
    // Loading eBPF programs requires specific capabilities:
    // - CAP_BPF (kernel 5.8+) for loading BPF programs
    // - CAP_PERFMON for perf events and tracing
    // - CAP_SYS_ADMIN (legacy, pre-5.8 kernels)
    //
    // The check command should report whether the current user has
    // sufficient permissions to load and run eBPF programs.
    //
    // Hints:
    // - Use is_root() helper to skip if not root
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add arg: "check"
    // - Assert stdout contains permission-related text
    // - Look for "CAP_BPF", "CAP_SYS_ADMIN", "permission", or "root"
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_check_shows_permissions: requires root privileges");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.arg("check")
    //    .assert()
    //    .success()
    //    .stdout(
    //        predicate::str::contains("CAP_BPF")
    //            .or(predicate::str::contains("CAP_SYS_ADMIN"))
    //            .or(predicate::str::contains("ermission"))
    //    );

    todo!("Implement test for permissions in check output")
}
