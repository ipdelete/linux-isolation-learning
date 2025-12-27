// Tests for the `setns` subcommand (joining existing namespaces)
// Lesson: docs/01-namespaces/06-setns.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges.
// Run with: sudo -E cargo test -p ns-tool

#[test]
fn test_setns_join_pid_namespace() {
    // TODO: Write a test that verifies joining an existing PID namespace
    //
    // Hints:
    // - First create a persistent namespace (can use `unshare` command or another process)
    // - Use setns() to join that namespace by opening /proc/<pid>/ns/pid
    // - Verify the process is now in the target namespace
    //
    // Test approach:
    // 1. Create a long-running process in a new PID namespace
    // 2. Run `ns-tool setns --pid <target-pid>` to join that namespace
    // 3. Verify the namespace inode matches
    // 4. Clean up the test process

    todo!("Implement test for joining existing PID namespace via setns")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_setns_join_network_namespace() {
    // TODO: Write a test that verifies joining an existing network namespace
    //
    // Hints:
    // - Create a network namespace with different network config
    // - Join it using setns() with CLONE_NEWNET
    // - Verify network interfaces changed

    todo!("Implement test for joining existing network namespace")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_setns_join_multiple_namespaces() {
    // TODO: Write a test that joins multiple namespaces at once
    //
    // Hints:
    // - Can call setns() multiple times for different namespace types
    // - Or can use setns() with multiple flags (if supported)

    todo!("Implement test for joining multiple namespaces simultaneously")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_setns_invalid_namespace_fails() {
    // TODO: Write a test that verifies error handling for invalid namespace
    //
    // Hints:
    // - Try to join a non-existent namespace
    // - Try to open an invalid /proc path
    // - Verify proper error messages

    todo!("Implement test for error handling with invalid namespaces")
}
