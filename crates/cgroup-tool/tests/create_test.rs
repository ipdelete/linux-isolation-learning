// Tests for the `create` subcommand (cgroup creation)
// Lesson: docs/02-cgroups/01-cgv2-basics.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require cgroup v2 and appropriate permissions.
// Run with: sudo -E cargo test -p cgroup-tool

#[test]
fn test_create_cgroup() {
    // TODO: Write a test that verifies creating a cgroup
    //
    // Hints:
    // - Cgroup v2 hierarchy is typically mounted at /sys/fs/cgroup
    // - Create a cgroup by creating a directory under the hierarchy
    // - The path is relative to the cgroup root (e.g., "test-cgroup")
    // - Verify the directory exists
    // - Verify it has the standard cgroup.* control files
    //
    // Test approach:
    // 1. Run `cgroup-tool create test/my-cgroup`
    // 2. Verify /sys/fs/cgroup/test/my-cgroup exists
    // 3. Verify /sys/fs/cgroup/test/my-cgroup/cgroup.procs exists
    // 4. Clean up: remove the cgroup directory

    todo!("Implement test for creating cgroup")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_create_nested_cgroup() {
    // TODO: Write a test that verifies creating nested cgroups
    //
    // Hints:
    // - Can create multi-level hierarchy (e.g., "parent/child/grandchild")
    // - Parent must exist before creating child

    todo!("Implement test for creating nested cgroups")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_create_duplicate_cgroup_fails() {
    // TODO: Write a test that verifies error when cgroup already exists
    //
    // Hints:
    // - Try to create same cgroup twice
    // - Should return clear error

    todo!("Implement test for error handling when cgroup exists")
}
