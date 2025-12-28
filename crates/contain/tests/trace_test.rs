// Tests for the `trace` subcommands
// Lesson: docs/fast-track/10-ebpf-tracing.md
//
// TDD Workflow:
// 1. Write the test below FIRST (RED)
// 2. Implement code in src/trace.rs (GREEN)

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_ebpf_check() {
    // TODO: Test that `contain trace check` verifies eBPF support
    //
    // Steps:
    // 1. Run `contain trace check`
    // 2. Assert success if eBPF is supported
    // 3. Check output indicates eBPF status
    //
    // Hints:
    // - Use Command::cargo_bin("contain")
    // - Check for /sys/fs/bpf existence message
    // - This test may need root or CAP_BPF

    todo!("Implement test - see docs/fast-track/10-ebpf-tracing.md")
}

#[test]
fn test_trace_syscalls_requires_root() {
    // TODO: Test that `contain trace syscalls` requires elevated privileges
    //
    // Steps:
    // 1. Run `contain trace syscalls` without root
    // 2. Assert it fails with permission error (or skip if already root)
    //
    // Hints:
    // - Check nix::unistd::Uid::effective().is_root()
    // - If root, skip this test
    // - Otherwise expect failure

    todo!("Implement test for privilege check")
}
