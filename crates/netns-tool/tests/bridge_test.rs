// Tests for the `bridge` subcommand (network bridge creation)
// Lesson: docs/01-namespaces/05-network-namespace.md (part 4)
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges.
// Run with: sudo -E cargo test -p netns-tool

#[test]
fn test_create_bridge() {
    // TODO: Write a test that verifies creating a network bridge
    //
    // Hints:
    // - Use `ip link add <name> type bridge` to create bridge
    // - Bring the bridge UP
    // - Verify bridge exists and is UP
    //
    // Test approach:
    // 1. Run `netns-tool bridge br0`
    // 2. Verify bridge exists: `ip link show br0`
    // 3. Verify it's type bridge: check link type
    // 4. Clean up: delete bridge

    todo!("Implement test for creating network bridge")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_bridge_add_interface() {
    // TODO: Write a test that verifies adding interface to bridge
    //
    // Hints:
    // - Create a bridge
    // - Create a veth pair
    // - Add one end of veth to the bridge
    // - Verify the interface is in the bridge

    todo!("Implement test for adding interface to bridge")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_multiple_namespaces_via_bridge() {
    // TODO: Write a test that verifies multiple namespaces can communicate via bridge
    //
    // Hints:
    // - Create a bridge
    // - Create 2+ namespaces
    // - Connect each namespace to bridge via veth
    // - Verify namespaces can ping each other through the bridge

    todo!("Implement test for namespace-to-namespace communication via bridge")
}
