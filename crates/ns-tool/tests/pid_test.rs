// Tests for the `pid` subcommand (PID namespace creation)
// Lesson: docs/01-namespaces/01-pid-namespace.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: Some of these tests require root privileges to create namespaces.
// Run with: sudo -E cargo test -p ns-tool

#[test]
fn test_pid_namespace_creation() {
    // TODO: Write a test that verifies creating a new PID namespace
    //
    // Hints:
    // - The `pid` subcommand should use unshare(CLONE_NEWPID) to create a new PID namespace
    // - After unsharing, fork a child process
    // - The child's PID should be 1 inside the new namespace (verify via getpid())
    // - The test should verify that the child reports PID 1
    //
    // This is an integration test - you may want to verify behavior by:
    // 1. Running the command
    // 2. Checking its output includes "PID inside namespace: 1"
    // 3. Ensuring it exits successfully

    todo!("Implement test for PID namespace creation - verify child process has PID 1")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_pid_namespace_isolation() {
    // TODO: Write a test that verifies process isolation
    //
    // Hints:
    // - The parent and child should have different PID namespaces
    // - You can verify by comparing /proc/self/ns/pid symlinks
    // - The child cannot see parent processes

    todo!("Implement test that verifies PID namespace isolation from parent")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_pid_namespace_without_root_fails() {
    // TODO: Write a test that verifies the command fails gracefully without root
    //
    // Hints:
    // - If not run as root, unshare should fail with EPERM
    // - The command should return a clear error message
    // - Test this by checking the command's stderr or exit code

    todo!("Implement test that verifies proper error handling when run without root")
}
