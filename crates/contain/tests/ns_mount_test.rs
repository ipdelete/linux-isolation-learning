// Tests for the `ns mount` subcommand
// Lesson: docs/fast-track/02-mount-namespace.md
//
// TDD Workflow:
// 1. Write the test below FIRST (RED)
// 2. Implement code in src/ns.rs (GREEN)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_mount_namespace_isolation() {
    // TODO: Test that `contain ns mount` creates an isolated mount namespace
    // where mounts don't leak to the host.
    //
    // Steps:
    // 1. Read /proc/self/mounts before running command
    // 2. Run `contain ns mount`
    // 3. Assert success and output contains "/mnt/test_mount"
    // 4. Read /proc/self/mounts after and verify mount didn't leak
    //
    // Hints:
    // - Use fs::read_to_string("/proc/self/mounts")
    // - Use Command::cargo_bin("contain")
    // - Use predicate::str::contains for output matching
    // - Assert the mount point is NOT in host mounts after

    todo!("Implement test - see docs/fast-track/02-mount-namespace.md")
}
