// Tests for the `memory-max` subcommand (memory limits)
// Lesson: docs/02-cgroups/02-memory.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor as needed
//
// NOTE: These tests require cgroup v2 and appropriate permissions.
// Run with: sudo -E cargo test -p cgroup-tool

#[test]
fn test_set_memory_limit() {
    // TODO: Write a test that verifies setting memory limit
    //
    // Hints:
    // - Create a test cgroup
    // - Write memory limit (in bytes) to memory.max
    // - Verify memory.max contains the correct value
    //
    // Test approach:
    // 1. Create test cgroup
    // 2. Run `cgroup-tool memory-max test-cgroup 104857600` (100MB)
    // 3. Verify /sys/fs/cgroup/test-cgroup/memory.max contains "104857600"
    // 4. Clean up

    todo!("Implement test for setting memory limit")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_memory_limit_enforcement() {
    // TODO: Write a test that verifies memory limit is enforced
    //
    // Hints:
    // - Create cgroup with small memory limit (e.g., 10MB)
    // - Spawn process that tries to allocate more memory
    // - Process should be OOM-killed when it exceeds the limit
    // - Can verify by checking memory.events for "oom_kill" counter
    //
    // This is an integration test that verifies the kernel enforces limits

    todo!("Implement integration test for memory limit enforcement")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_memory_current_tracking() {
    // TODO: Write a test that verifies memory.current tracks usage
    //
    // Hints:
    // - Attach a process to cgroup
    // - Process allocates memory
    // - memory.current should increase
    // - Can be verified by reading memory.current before and after allocation

    todo!("Implement test for memory usage tracking")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_set_memory_max_unlimited() {
    // TODO: Write a test that verifies setting memory to unlimited
    //
    // Hints:
    // - Writing "max" to memory.max removes the limit
    // - Verify by checking memory.max shows "max"

    todo!("Implement test for removing memory limit")
}
