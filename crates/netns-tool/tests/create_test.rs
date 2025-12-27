// Tests for the `create` subcommand (network namespace creation)
// Lesson: docs/01-namespaces/05-network-namespace.md (part 1)
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges.
// Run with: sudo -E cargo test -p netns-tool

#[test]
fn test_create_network_namespace() {
    // TODO: Write a test that verifies creating a named network namespace
    //
    // Hints:
    // - Network namespaces can be made persistent by bind-mounting to /run/netns/
    // - The `create` subcommand should:
    //   1. Create /run/netns/ directory if it doesn't exist
    //   2. Use unshare(CLONE_NEWNET) to create new network namespace
    //   3. Bind-mount /proc/self/ns/net to /run/netns/<name>
    // - Verify the namespace file exists at /run/netns/<name>
    //
    // Test approach:
    // 1. Run `netns-tool create test-ns`
    // 2. Verify /run/netns/test-ns exists
    // 3. Verify it's a valid namespace (can be opened)
    // 4. Clean up: remove the namespace file and unmount

    todo!("Implement test for creating persistent network namespace")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_create_duplicate_namespace_fails() {
    // TODO: Write a test that verifies creating duplicate namespace fails
    //
    // Hints:
    // - Try to create a namespace with a name that already exists
    // - Should return an error, not overwrite

    todo!("Implement test for error handling when namespace already exists")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_create_namespace_has_loopback() {
    // TODO: Write a test that verifies new network namespace has loopback interface
    //
    // Hints:
    // - New network namespaces have a loopback interface (lo) but it's DOWN
    // - Can verify by executing `ip link` inside the namespace
    // - Use `ip netns exec <name> ip link` to run commands in the namespace

    todo!("Implement test verifying loopback interface exists in new namespace")
}
