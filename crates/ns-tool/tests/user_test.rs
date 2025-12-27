// Tests for the `user` subcommand (user namespace for UID/GID mapping)
// Lesson: docs/01-namespaces/05-user-namespace.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: User namespaces are special - they can be created without root!
// These tests should work as a normal user.

#[test]
fn test_user_namespace_uid_mapping() {
    // TODO: Write a test that verifies UID mapping in a user namespace
    //
    // Hints:
    // - The `user` subcommand should unshare(CLONE_NEWUSER)
    // - Map the current user's UID to 0 (root) inside the namespace
    // - Verify getuid() returns 0 inside the namespace
    // - Verify the process appears as root inside but not outside
    //
    // Test approach:
    // 1. Run `ns-tool user` which should print UID inside namespace
    // 2. Verify output shows "UID: 0" (root)
    // 3. Verify current process still has original UID outside namespace

    todo!("Implement test for user namespace UID mapping")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_user_namespace_gid_mapping() {
    // TODO: Write a test that verifies GID mapping
    //
    // Hints:
    // - Similar to UID mapping, but for group IDs
    // - Map current GID to 0 inside the namespace

    todo!("Implement test for user namespace GID mapping")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_user_namespace_capabilities() {
    // TODO: Write a test that verifies capabilities inside user namespace
    //
    // Hints:
    // - Inside a user namespace, the process has full capabilities
    // - Can check /proc/self/status for CapEff (effective capabilities)
    // - Should show all capabilities set

    todo!("Implement test for capabilities inside user namespace")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_user_namespace_without_mapping_fails() {
    // TODO: Write a test that verifies namespace fails without proper UID/GID mapping
    //
    // Hints:
    // - If you don't set up uid_map and gid_map, many operations will fail
    // - Test that your implementation handles this correctly

    todo!("Implement test for error handling without UID/GID mapping")
}
