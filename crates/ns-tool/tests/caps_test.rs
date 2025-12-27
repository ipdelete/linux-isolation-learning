// Tests for the `check-caps` subcommand (capability inspection)
// Lesson: docs/00-foundations/04-permissions-and-sudo.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests run as the current user (not root).
// Some tests check behavior with/without privileges.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_check_caps_runs_successfully() {
    // TODO: The check-caps subcommand should always succeed (even without root)
    // It inspects capabilities and reports what the process can do
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    // cmd.arg("check-caps")
    //    .assert()
    //    .success();

    todo!("Implement test for check-caps subcommand - verify it runs successfully")
}

#[test]
fn test_check_caps_shows_effective_capabilities() {
    // TODO: Output should include the effective capability hex string
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    // cmd.arg("check-caps")
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("Effective capabilities:"));

    todo!("Implement test that verifies effective capabilities are displayed")
}

#[test]
fn test_check_caps_reports_cap_sys_admin_status() {
    // TODO: Output should report whether CAP_SYS_ADMIN is present
    // This determines if most namespace operations will work
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    // cmd.arg("check-caps")
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("CAP_SYS_ADMIN:"));

    todo!("Implement test that verifies CAP_SYS_ADMIN status is reported")
}

#[test]
fn test_check_caps_shows_namespace_creation_ability() {
    // TODO: Output should summarize what namespace operations are possible
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    // cmd.arg("check-caps")
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("Namespace creation:"));

    todo!("Implement test that verifies namespace creation ability is shown")
}

#[test]
fn test_check_caps_always_shows_user_ns_as_available() {
    // TODO: User namespaces can be created without privileges (on most systems)
    // The output should reflect this
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    // cmd.arg("check-caps")
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("user").and(predicate::str::contains("available")));

    todo!("Implement test that verifies user namespaces are shown as available")
}
