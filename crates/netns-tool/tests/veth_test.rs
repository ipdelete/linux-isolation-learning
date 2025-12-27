// Tests for the `veth` subcommand (virtual ethernet pair creation)
// Lesson: docs/01-namespaces/05-network-namespace.md (part 3)
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges.
// Run with: sudo -E cargo test -p netns-tool

#[test]
fn test_create_veth_pair() {
    // TODO: Write a test that verifies creating a veth pair
    //
    // Hints:
    // - Create a network namespace first
    // - Use `veth` subcommand to create veth pair
    // - One end stays in host, other end goes to namespace
    // - Verify both ends exist in their respective namespaces
    //
    // Implementation should:
    // 1. Create veth pair using rtnetlink or ip command
    // 2. Move one end into the target namespace
    // 3. Assign IP addresses to both ends
    //
    // Test approach:
    // 1. Create test namespace
    // 2. Run `netns-tool veth --host veth0 --ns veth1` (or similar)
    // 3. Verify veth0 exists on host (`ip link show veth0`)
    // 4. Verify veth1 exists in namespace (`ip netns exec test-ns ip link show veth1`)
    // 5. Clean up

    todo!("Implement test for creating veth pair across namespaces")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_veth_connectivity() {
    // TODO: Write a test that verifies connectivity through veth pair
    //
    // Hints:
    // - Create veth pair with IP addresses
    // - Bring both interfaces UP
    // - Ping from host to namespace IP
    // - Should succeed if veth pair is configured correctly

    todo!("Implement test for veth pair connectivity via ping")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_veth_to_nonexistent_namespace_fails() {
    // TODO: Write a test that verifies error when target namespace doesn't exist
    //
    // Hints:
    // - Try to create veth pair to non-existent namespace
    // - Should fail with clear error

    todo!("Implement test for error handling with non-existent namespace")
}
