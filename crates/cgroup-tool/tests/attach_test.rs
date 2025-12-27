// Tests for the `attach` subcommand (attaching processes to cgroups)
// Lesson: docs/02-cgroups/01-create-attach.md (part 2)
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor as needed
//
// NOTE: These tests require cgroup v2 and appropriate permissions.
// Run with: sudo -E cargo test -p cgroup-tool

#[test]
fn test_attach_process_to_cgroup() {
    // TODO: Write a test that verifies attaching a process to a cgroup
    //
    // Hints:
    // - Create a test cgroup first
    // - Spawn a test process (e.g., sleep command)
    // - Write the PID to the cgroup's cgroup.procs file
    // - Verify the PID appears in cgroup.procs
    // - Verify /proc/<pid>/cgroup shows the correct cgroup
    //
    // Test approach:
    // 1. Create test cgroup
    // 2. Spawn long-running test process
    // 3. Run `cgroup-tool attach test-cgroup <pid>`
    // 4. Verify PID is in /sys/fs/cgroup/test-cgroup/cgroup.procs
    // 5. Clean up: kill process, remove cgroup

    todo!("Implement test for attaching process to cgroup")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_attach_self_to_cgroup() {
    // TODO: Write a test that verifies a process can attach itself to a cgroup
    //
    // Hints:
    // - Can write "self" or current PID to cgroup.procs
    // - After attaching, /proc/self/cgroup should show new location

    todo!("Implement test for process attaching itself to cgroup")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_attach_to_nonexistent_cgroup_fails() {
    // TODO: Write a test that verifies error when attaching to non-existent cgroup
    //
    // Hints:
    // - Try to attach to a cgroup that doesn't exist
    // - Should return clear error

    todo!("Implement test for error handling with non-existent cgroup")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_attach_nonexistent_pid_fails() {
    // TODO: Write a test that verifies error when attaching non-existent PID
    //
    // Hints:
    // - Try to attach a PID that doesn't exist
    // - Should return clear error (ESRCH - no such process)

    todo!("Implement test for error handling with non-existent PID")
}
