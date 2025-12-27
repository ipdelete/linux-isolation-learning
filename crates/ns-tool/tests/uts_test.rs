// Tests for the `uts` subcommand (UTS namespace for hostname isolation)
// Lesson: docs/01-namespaces/02-uts-namespace.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges.
// Run with: sudo -E cargo test -p ns-tool

#[test]
fn test_uts_namespace_hostname_isolation() {
    // TODO: Write a test that verifies hostname isolation in UTS namespace
    //
    // Hints:
    // - The `uts` subcommand should unshare(CLONE_NEWUTS)
    // - Set a custom hostname inside the namespace (e.g., "container-test")
    // - Verify the hostname is changed inside the namespace
    // - Verify the original hostname is unchanged outside the namespace
    //
    // Test approach:
    // 1. Get current hostname before running command
    // 2. Run `ns-tool uts` which should set a different hostname and print it
    // 3. Verify command output shows the new hostname
    // 4. Verify current system hostname is still the original

    todo!("Implement test for UTS namespace hostname isolation")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_uts_namespace_domainname() {
    // TODO: Write a test that verifies domain name isolation
    //
    // Hints:
    // - Similar to hostname, but using setdomainname() and getdomainname()
    // - UTS namespace isolates both hostname and domain name

    todo!("Implement test for UTS namespace domain name isolation")
}
