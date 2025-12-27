// Tests for the `cpu-max` subcommand (CPU limits)
// Lesson: docs/02-cgroups/03-cpu.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor as needed
//
// NOTE: These tests require cgroup v2 and appropriate permissions.
// Run with: sudo -E cargo test -p cgroup-tool

#[test]
fn test_set_cpu_quota() {
    // TODO: Write a test that verifies setting CPU quota
    //
    // Hints:
    // - cpu.max format is "quota period" (both in microseconds)
    // - Example: "50000 100000" means 50ms out of every 100ms (50% CPU)
    // - Default period is usually 100000 (100ms)
    // - Verify cpu.max contains the correct quota
    //
    // Test approach:
    // 1. Create test cgroup
    // 2. Run `cgroup-tool cpu-max test-cgroup "50000 100000"`
    // 3. Verify /sys/fs/cgroup/test-cgroup/cpu.max contains "50000 100000"
    // 4. Clean up

    todo!("Implement test for setting CPU quota")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_cpu_quota_enforcement() {
    // TODO: Write a test that verifies CPU quota is enforced
    //
    // Hints:
    // - Create cgroup with CPU limit (e.g., 50%)
    // - Spawn CPU-intensive process
    // - Measure CPU usage over time
    // - Should not exceed the quota
    // - Can check cpu.stat for throttling events
    //
    // This is an integration test

    todo!("Implement integration test for CPU quota enforcement")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_set_cpu_max_unlimited() {
    // TODO: Write a test that verifies removing CPU limit
    //
    // Hints:
    // - Writing "max" to cpu.max removes the limit
    // - Format: "max 100000" or just "max"

    todo!("Implement test for removing CPU limit")
}
