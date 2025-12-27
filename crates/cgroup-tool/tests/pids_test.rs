// Tests for the `pids-max` subcommand (process number limits)
// Lesson: docs/02-cgroups/05-pids.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor as needed
//
// NOTE: These tests require cgroup v2 and appropriate permissions.
// Run with: sudo -E cargo test -p cgroup-tool

#[test]
fn test_set_pids_limit() {
    // TODO: Write a test that verifies setting PIDs limit
    //
    // Hints:
    // - pids.max limits the number of processes in the cgroup
    // - Write the limit as a number to pids.max
    // - Verify pids.max contains the correct value
    //
    // Test approach:
    // 1. Create test cgroup
    // 2. Run `cgroup-tool pids-max test-cgroup 10`
    // 3. Verify /sys/fs/cgroup/test-cgroup/pids.max contains "10"
    // 4. Clean up

    todo!("Implement test for setting PIDs limit")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_pids_limit_enforcement() {
    // TODO: Write a test that verifies PIDs limit is enforced
    //
    // Hints:
    // - Create cgroup with small PIDs limit (e.g., 5)
    // - Try to spawn more processes than the limit
    // - fork() should fail with EAGAIN when limit is reached
    // - Can verify by checking pids.events for "max" counter
    //
    // This is an integration test

    todo!("Implement integration test for PIDs limit enforcement")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_pids_current_tracking() {
    // TODO: Write a test that verifies pids.current tracks process count
    //
    // Hints:
    // - Attach processes to cgroup
    // - pids.current should increase with each process
    // - Should decrease when processes exit

    todo!("Implement test for PIDs usage tracking")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_set_pids_max_unlimited() {
    // TODO: Write a test that verifies removing PIDs limit
    //
    // Hints:
    // - Writing "max" to pids.max removes the limit

    todo!("Implement test for removing PIDs limit")
}
