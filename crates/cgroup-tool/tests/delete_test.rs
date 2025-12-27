// Tests for the `delete` subcommand (cgroup deletion)
// Lesson: docs/02-cgroups/01-cgv2-basics.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor as needed
//
// NOTE: These tests require cgroup v2 and appropriate permissions.
// Run with: sudo -E cargo test -p cgroup-tool

#[test]
fn test_delete_empty_cgroup() {
    // TODO: Write a test that verifies deleting an empty cgroup
    //
    // Hints:
    // - Create a test cgroup
    // - Delete it by removing the directory
    // - Cgroup must be empty (no processes) to be deleted
    // - Verify the directory no longer exists
    //
    // Test approach:
    // 1. Create test cgroup
    // 2. Run `cgroup-tool delete test-cgroup`
    // 3. Verify /sys/fs/cgroup/test-cgroup no longer exists

    todo!("Implement test for deleting empty cgroup")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_delete_cgroup_with_processes_fails() {
    // TODO: Write a test that verifies deletion fails when cgroup has processes
    //
    // Hints:
    // - Create cgroup and attach a process
    // - Try to delete it
    // - Should fail with EBUSY
    // - Should return clear error message

    todo!("Implement test for error handling when deleting non-empty cgroup")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_delete_nonexistent_cgroup_fails() {
    // TODO: Write a test that verifies error when deleting non-existent cgroup
    //
    // Hints:
    // - Try to delete a cgroup that doesn't exist
    // - Should return clear error

    todo!("Implement test for error handling with non-existent cgroup")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_delete_nested_cgroups() {
    // TODO: Write a test that verifies deleting nested cgroups
    //
    // Hints:
    // - Create parent/child/grandchild hierarchy
    // - Must delete from deepest to shallowest (leaves first)
    // - Cannot delete parent while children exist

    todo!("Implement test for deleting nested cgroup hierarchy")
}
