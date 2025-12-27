// Tests for the `delete` subcommand (network namespace deletion)
// Lesson: docs/01-namespaces/05-network-namespace.md (part 2)
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges.
// Run with: sudo -E cargo test -p netns-tool

#[test]
fn test_delete_network_namespace() {
    // TODO: Write a test that verifies deleting a network namespace
    //
    // Hints:
    // - First create a test namespace
    // - Use `delete` subcommand to remove it
    // - Verify the namespace file is gone
    // - The delete should:
    //   1. Unmount /run/netns/<name>
    //   2. Remove the file
    //
    // Test approach:
    // 1. Create a test namespace (setup)
    // 2. Run `netns-tool delete test-ns`
    // 3. Verify /run/netns/test-ns no longer exists
    // 4. Verify unmount was successful (check mount table)

    todo!("Implement test for deleting network namespace")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_delete_nonexistent_namespace_fails() {
    // TODO: Write a test that verifies deleting non-existent namespace fails gracefully
    //
    // Hints:
    // - Try to delete a namespace that doesn't exist
    // - Should return clear error message

    todo!("Implement test for error handling when deleting non-existent namespace")
}
