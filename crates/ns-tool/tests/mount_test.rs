// Tests for the `mount` subcommand (mount namespace for filesystem isolation)
// Lesson: docs/01-namespaces/04-mount-namespace.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges.
// Run with: sudo -E cargo test -p ns-tool

#[test]
fn test_mount_namespace_mount_isolation() {
    // TODO: Write a test that verifies mount isolation
    //
    // Hints:
    // - The `mount` subcommand should unshare(CLONE_NEWNS)
    // - Create a temporary mount inside the namespace
    // - Verify the mount exists inside the namespace
    // - Verify the mount does NOT exist in the parent namespace
    //
    // Test approach:
    // 1. Run `ns-tool mount` which should create a mount and list /proc/self/mounts
    // 2. Verify test mount appears in command output
    // 3. Verify test mount does NOT appear in current /proc/self/mounts

    todo!("Implement test for mount namespace isolation")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_mount_namespace_tmpfs() {
    // TODO: Write a test that creates a tmpfs mount in an isolated namespace
    //
    // Hints:
    // - Create a tmpfs mount at a test directory
    // - Write a file to the tmpfs
    // - Verify the file exists inside the namespace
    // - Verify the file does NOT exist outside the namespace

    todo!("Implement test for tmpfs mount in isolated namespace")
}
