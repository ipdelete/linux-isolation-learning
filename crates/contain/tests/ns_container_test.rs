// Tests for the `ns container` subcommand
// Lesson: docs/fast-track/04-combine.md
//
// TDD Workflow:
// 1. Write the test below FIRST (RED)
// 2. Implement code in src/ns.rs (GREEN)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_container_isolation() {
    // TODO: Test that `contain ns container` creates isolated namespaces
    // where the process is PID 1 with custom hostname.
    //
    // Steps:
    // 1. Skip if not root (requires CAP_SYS_ADMIN)
    // 2. Read /proc/self/ns/pid to get parent namespace
    // 3. Run `contain ns container -- /bin/sh -c 'echo PID:$$ && hostname'`
    // 4. Assert success and output contains "PID:1" and "container"
    // 5. Verify parent namespace unchanged
    //
    // Hints:
    // - Check root: nix::unistd::Uid::effective().is_root()
    // - Use fs::read_link("/proc/self/ns/pid")
    // - Use predicate::str::contains for output matching

    todo!("Implement test - see docs/fast-track/04-combine.md")
}
