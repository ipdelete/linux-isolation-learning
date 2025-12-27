// Tests for multi-resource cgroup bundles (combining memory + CPU + PIDs limits)
// Lesson: docs/02-cgroups/06-multi-resource.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor as needed
//
// NOTE: These tests require cgroup v2 and appropriate permissions.
// Run with: sudo -E cargo test -p cgroup-tool --test bundle_test

use std::fs;

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

// Helper to create a test cgroup directly (bypasses our tool for setup)
fn create_test_cgroup(name: &str) -> std::io::Result<()> {
    fs::create_dir_all(format!("{}/{}", CGROUP_ROOT, name))
}

// Helper to clean up test cgroup
fn cleanup_test_cgroup(name: &str) {
    let _ = fs::remove_dir(format!("{}/{}", CGROUP_ROOT, name));
}

#[test]
fn test_apply_memory_cpu_pids_bundle() {
    // TODO: Write a test that verifies applying all three limits to one cgroup
    //
    // Test approach:
    // 1. Create test cgroup "test-bundle"
    // 2. Apply memory limit: 100MB (104857600 bytes)
    // 3. Apply CPU limit: 50% (50000 100000)
    // 4. Apply PIDs limit: 20
    // 5. Verify all three limit files contain correct values
    // 6. Clean up
    //
    // Hints:
    // - Use assert_cmd::Command to run cgroup-tool subcommands
    // - Read each limit file with std::fs::read_to_string to verify
    // - All limits should be independent (order doesn't matter)
    //
    // Example structure:
    // ```
    // use assert_cmd::Command;
    //
    // let cgroup_name = "test-bundle";
    // create_test_cgroup(cgroup_name).expect("Failed to create test cgroup");
    //
    // // Apply memory limit
    // Command::cargo_bin("cgroup-tool")
    //     .unwrap()
    //     .args(["memory-max", cgroup_name, "104857600"])
    //     .assert()
    //     .success();
    //
    // // Apply CPU limit
    // Command::cargo_bin("cgroup-tool")
    //     .unwrap()
    //     .args(["cpu-max", cgroup_name, "50000 100000"])
    //     .assert()
    //     .success();
    //
    // // Apply PIDs limit
    // Command::cargo_bin("cgroup-tool")
    //     .unwrap()
    //     .args(["pids-max", cgroup_name, "20"])
    //     .assert()
    //     .success();
    //
    // // Verify all limits
    // let memory_max = fs::read_to_string(format!("{}/{}/memory.max", CGROUP_ROOT, cgroup_name))
    //     .expect("Failed to read memory.max");
    // assert_eq!(memory_max.trim(), "104857600");
    //
    // // ... verify cpu.max and pids.max similarly
    //
    // cleanup_test_cgroup(cgroup_name);
    // ```

    todo!("Implement test for applying memory + CPU + PIDs bundle")
}

#[test]
#[ignore] // Remove after implementing
fn test_bundle_limits_are_enforced_together() {
    // TODO: Write an integration test that verifies combined enforcement
    //
    // Test approach:
    // 1. Create cgroup with tight limits (50MB memory, 25% CPU, 10 PIDs)
    // 2. Attach a stress test process
    // 3. Verify process is constrained by ALL limits
    // 4. Check memory.current, cpu.stat, pids.current for enforcement evidence
    //
    // This is an advanced integration test that requires:
    // - Spawning child processes
    // - Attaching them to the cgroup
    // - Measuring resource usage over time
    //
    // Hints:
    // - Use std::process::Command to spawn stress processes
    // - Use cgroup-tool attach to move them to the cgroup
    // - Read monitoring files to verify enforcement

    todo!("Implement integration test for combined limit enforcement")
}

#[test]
#[ignore] // Remove after implementing
fn test_monitoring_multi_resource_cgroup() {
    // TODO: Write a test that verifies reading all monitoring files
    //
    // Test approach:
    // 1. Create cgroup with limits
    // 2. Attach process that consumes resources
    // 3. Read memory.current, cpu.stat, pids.current
    // 4. Verify all return valid data (parseable numbers)
    //
    // Hints:
    // - memory.current: current bytes used (single number)
    // - cpu.stat: key-value pairs (usage_usec, nr_throttled, etc.)
    // - pids.current: current process count (single number)
    //
    // Example verification:
    // ```
    // let memory_current = fs::read_to_string(format!("{}/{}/memory.current", CGROUP_ROOT, cgroup_name))
    //     .expect("Failed to read memory.current");
    // let bytes: u64 = memory_current.trim().parse()
    //     .expect("memory.current should be a number");
    // assert!(bytes > 0, "Should have some memory usage");
    // ```

    todo!("Implement test for monitoring multi-resource cgroup")
}

#[test]
fn test_controllers_available() {
    // TODO: Write a test that verifies required controllers are available
    //
    // Test approach:
    // 1. Read /sys/fs/cgroup/cgroup.controllers
    // 2. Verify "memory", "cpu", and "pids" are present
    //
    // This is a prerequisite check for the other tests
    //
    // Example implementation:
    // ```
    // let controllers = fs::read_to_string(format!("{}/cgroup.controllers", CGROUP_ROOT))
    //     .expect("Failed to read cgroup.controllers");
    //
    // assert!(controllers.contains("memory"), "memory controller required");
    // assert!(controllers.contains("cpu"), "cpu controller required");
    // assert!(controllers.contains("pids"), "pids controller required");
    // ```

    todo!("Implement test for controller availability")
}

#[test]
#[ignore] // Remove after implementing
fn test_bundle_subcommand() {
    // TODO: Write a test for the optional Bundle subcommand
    //
    // This test only applies if you implemented the Bundle subcommand
    // from Option B in the lesson.
    //
    // Test approach:
    // 1. Run: cgroup-tool bundle test-bundle --memory-max 104857600 --cpu-quota "50000 100000" --pids-max 20
    // 2. Verify cgroup was created
    // 3. Verify all three limit files contain correct values
    // 4. Clean up
    //
    // Example:
    // ```
    // use assert_cmd::Command;
    //
    // let cgroup_name = "test-bundle-cmd";
    //
    // Command::cargo_bin("cgroup-tool")
    //     .unwrap()
    //     .args([
    //         "bundle", cgroup_name,
    //         "--memory-max", "104857600",
    //         "--cpu-quota", "50000 100000",
    //         "--pids-max", "20"
    //     ])
    //     .assert()
    //     .success();
    //
    // // Verify cgroup exists and has correct limits
    // let memory_max = fs::read_to_string(format!("{}/{}/memory.max", CGROUP_ROOT, cgroup_name))
    //     .expect("Failed to read memory.max");
    // assert_eq!(memory_max.trim(), "104857600");
    //
    // cleanup_test_cgroup(cgroup_name);
    // ```

    todo!("Implement test for bundle subcommand (optional)")
}
