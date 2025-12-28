// Tests for the `ns pid` subcommand
// Lesson: docs/fast-track/01-pid-namespace.md
//
// TDD Workflow:
// 1. Write the test below FIRST (RED)
// 2. Implement code in src/ns.rs (GREEN)

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_pid_namespace_creation() {
    // TODO: Test that `contain ns pid` creates a PID namespace
    // where the child process sees itself as PID 1.
    //
    // Steps:
    // 1. Skip if not root (requires CAP_SYS_ADMIN)
    // 2. Run `contain ns pid`
    // 3. Assert success and output contains "PID inside namespace: 1"
    //
    // Hints:
    // - Check root: nix::unistd::Uid::effective().is_root()
    // - Use Command::cargo_bin("contain")
    // - Use predicate::str::contains for output matching

    todo!("Implement test - see docs/fast-track/01-pid-namespace.md")
}
