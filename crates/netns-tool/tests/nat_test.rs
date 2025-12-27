// Tests for the `nat` subcommand (NAT/masquerading for internet access)
// Lesson: docs/01-namespaces/05-network-namespace.md (part 5)
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges and modify iptables/nftables.
// Run with: sudo -E cargo test -p netns-tool

#[test]
fn test_setup_nat() {
    // TODO: Write a test that verifies NAT setup for internet access
    //
    // Hints:
    // - Enable IP forwarding: echo 1 > /proc/sys/net/ipv4/ip_forward
    // - Add iptables MASQUERADE rule for the bridge subnet
    // - Verify rule exists in iptables
    //
    // Implementation should:
    // 1. Enable IP forwarding
    // 2. Add iptables rule: iptables -t nat -A POSTROUTING -s <bridge-subnet> -o <outbound> -j MASQUERADE
    // 3. Add forward rules for the bridge
    //
    // Test approach:
    // 1. Create a bridge with subnet (e.g., 10.0.0.1/24)
    // 2. Run `netns-tool nat --bridge br0 --outbound eth0`
    // 3. Verify IP forwarding is enabled
    // 4. Verify iptables MASQUERADE rule exists
    // 5. Clean up iptables rules

    todo!("Implement test for NAT setup")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_namespace_internet_access() {
    // TODO: Write a test that verifies namespace can access internet through NAT
    //
    // Hints:
    // - Set up complete network: namespace + veth + bridge + NAT
    // - Configure default route in namespace to point to bridge
    // - Test by pinging external IP (e.g., 8.8.8.8) from inside namespace
    // - This is an integration test combining all network features

    todo!("Implement integration test for internet access from namespace via NAT")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_nat_cleanup() {
    // TODO: Write a test that verifies NAT rules can be cleaned up
    //
    // Hints:
    // - Add NAT rules
    // - Delete them
    // - Verify they're gone from iptables

    todo!("Implement test for cleaning up NAT rules")
}
