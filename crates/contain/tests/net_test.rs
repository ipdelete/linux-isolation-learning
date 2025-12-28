// Tests for the `net` subcommands (create, delete, veth)
// Lesson: docs/fast-track/03-network-namespace.md
//
// TDD Workflow:
// 1. Write the tests below FIRST (RED)
// 2. Implement code in src/net.rs (GREEN)

use assert_cmd::Command;

#[test]
fn test_veth_pair_created() {
    // TODO: Test that `contain net` commands create namespace and veth pair.
    //
    // Steps:
    // 1. Skip if not root (requires CAP_NET_ADMIN)
    // 2. Run `contain net create test-ns`
    // 3. Run `contain net veth --host veth-host --ns test-ns`
    // 4. Assert both succeed
    // 5. Cleanup with `contain net delete test-ns`
    //
    // Hints:
    // - Check root: nix::unistd::Uid::effective().is_root()
    // - Use Command::cargo_bin("contain")
    // - Network namespaces require root privileges

    todo!("Implement test - see docs/fast-track/03-network-namespace.md")
}
