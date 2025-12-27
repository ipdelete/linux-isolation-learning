// Tests for the `io-max` subcommand (I/O bandwidth limits)
// Lesson: docs/02-cgroups/04-io.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor as needed
//
// NOTE: These tests require cgroup v2, a block device, and appropriate permissions.
// Run with: sudo -E cargo test -p cgroup-tool --test io_test

use std::fs;
use std::path::Path;

/// Helper to get a valid block device for testing.
/// Returns None if no suitable device is found.
///
/// This function searches for real block devices (not loop or ram devices)
/// that can be used for I/O limit testing.
#[allow(dead_code)]
fn find_test_block_device() -> Option<String> {
    // Try common block device paths first
    let candidates = ["/sys/block/sda", "/sys/block/vda", "/sys/block/nvme0n1"];

    for candidate in candidates {
        if Path::new(candidate).exists() {
            if let Ok(dev) = fs::read_to_string(format!("{}/dev", candidate)) {
                return Some(dev.trim().to_string());
            }
        }
    }

    // Try to find any block device
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            // Skip loop devices without backing files and ram devices
            if name_str.starts_with("loop") || name_str.starts_with("ram") {
                continue;
            }
            let dev_path = entry.path().join("dev");
            if let Ok(dev) = fs::read_to_string(&dev_path) {
                return Some(dev.trim().to_string());
            }
        }
    }

    None
}

#[test]
fn test_set_io_limit() {
    // TODO: Write a test that verifies setting I/O bandwidth limit
    //
    // Hints:
    // - io.max format is "MAJ:MIN rbps=X wbps=X riops=X wiops=X"
    // - Need a real block device's major:minor number
    // - Use find_test_block_device() helper to get a valid device
    // - Verify io.max contains the correct limit after setting
    //
    // Test approach:
    // 1. Find a block device using find_test_block_device()
    // 2. Create test cgroup
    // 3. Enable io controller (write "+io" to parent's cgroup.subtree_control)
    // 4. Run `cgroup-tool io-max test-cgroup "8:0" "rbps=1048576 wbps=1048576"`
    // 5. Verify /sys/fs/cgroup/test-cgroup/io.max contains the expected line
    // 6. Clean up
    //
    // Example using assert_cmd:
    // ```
    // use assert_cmd::Command;
    //
    // // Setup: create cgroup and enable io controller
    // fs::create_dir_all("/sys/fs/cgroup/io-test").unwrap();
    // fs::write("/sys/fs/cgroup/cgroup.subtree_control", "+io").ok();
    //
    // // Find device
    // let device = find_test_block_device().expect("No block device found");
    //
    // // Run command
    // let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    // cmd.arg("io-max")
    //    .arg("/io-test")
    //    .arg(&device)
    //    .arg("rbps=1048576 wbps=1048576")
    //    .assert()
    //    .success();
    //
    // // Verify
    // let io_max = fs::read_to_string("/sys/fs/cgroup/io-test/io.max").unwrap();
    // assert!(io_max.contains(&format!("{} rbps=1048576 wbps=1048576", device)));
    //
    // // Cleanup
    // fs::remove_dir("/sys/fs/cgroup/io-test").ok();
    // ```

    todo!("Implement test for setting I/O bandwidth limit")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_io_limit_with_iops() {
    // TODO: Write a test that sets both bandwidth and IOPS limits
    //
    // Hints:
    // - Can combine all limit types: rbps, wbps, riops, wiops
    // - Example: "rbps=1048576 wbps=1048576 riops=100 wiops=100"
    // - All unspecified limits remain at "max" (unlimited)
    //
    // Test approach:
    // 1. Find a block device
    // 2. Create test cgroup with io controller enabled
    // 3. Set combined limits: bandwidth AND IOPS
    // 4. Verify io.max contains all specified limits
    // 5. Clean up

    todo!("Implement test for combined bandwidth and IOPS limits")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_io_limit_enforcement() {
    // TODO: Write a test that verifies I/O limit is enforced
    //
    // Hints:
    // - Create cgroup with low write bandwidth (e.g., 1MB/s = 1048576 bytes/s)
    // - Use dd with direct I/O to write data
    // - Measure time taken - should be throttled
    // - Can check io.stat for bytes written
    //
    // Example dd command:
    // dd if=/dev/zero of=/tmp/testfile bs=1M count=5 oflag=direct
    //
    // With 1MB/s limit, writing 5MB should take ~5 seconds
    //
    // Test approach:
    // 1. Create cgroup with wbps=1048576 (1MB/s)
    // 2. Spawn a process attached to the cgroup
    // 3. Process writes data using direct I/O
    // 4. Measure elapsed time
    // 5. Assert time >= expected_bytes / limit_bytes_per_sec
    // 6. Clean up
    //
    // Note: This is an integration test that depends on actual I/O behavior

    todo!("Implement integration test for I/O limit enforcement")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_io_stat_tracking() {
    // TODO: Write a test that verifies io.stat tracks I/O usage
    //
    // Hints:
    // - io.stat format: "MAJ:MIN rbytes=N wbytes=N rios=N wios=N dbytes=N dios=N"
    // - Attach a process to cgroup
    // - Process performs I/O operations
    // - io.stat should show rbytes/wbytes increasing
    // - Read io.stat before and after I/O to compare
    //
    // Test approach:
    // 1. Create cgroup
    // 2. Read initial io.stat (may be empty or have zeros)
    // 3. Attach a process that performs I/O
    // 4. Read io.stat again
    // 5. Assert that wbytes or rbytes increased
    // 6. Clean up

    todo!("Implement test for I/O statistics tracking")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_io_max_remove_limit() {
    // TODO: Write a test that verifies removing I/O limit
    //
    // Hints:
    // - Setting all values to "max" removes limits for that device
    // - Example: "8:0 rbps=max wbps=max riops=max wiops=max"
    // - After removal, io.max should not contain the device line
    //   (or show all "max" values)
    //
    // Test approach:
    // 1. Create cgroup with io controller enabled
    // 2. Set an initial limit
    // 3. Verify limit is set
    // 4. Set all limits to "max"
    // 5. Verify limit is removed or shows unlimited
    // 6. Clean up

    todo!("Implement test for removing I/O limit")
}

#[test]
fn test_io_max_invalid_device() {
    // TODO: Write a test for invalid device error handling
    //
    // Hints:
    // - Using a non-existent device should fail gracefully
    // - Invalid format (not MAJ:MIN) should be rejected
    // - The kernel returns an error for invalid devices
    //
    // Test approach:
    // 1. Create test cgroup with io controller enabled
    // 2. Try to set limit with invalid device like "999:999"
    // 3. Assert command fails (non-zero exit or error message)
    // 4. Clean up
    //
    // Example:
    // ```
    // use assert_cmd::Command;
    //
    // let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    // cmd.arg("io-max")
    //    .arg("/io-test")
    //    .arg("999:999")  // Invalid device
    //    .arg("rbps=1048576")
    //    .assert()
    //    .failure();  // Should fail
    // ```

    todo!("Implement test for invalid device error handling")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_io_max_multiple_devices() {
    // TODO: Write a test for setting limits on multiple devices
    //
    // Hints:
    // - Each device gets its own line in io.max
    // - Can set different limits for each device
    // - Writing to io.max adds/updates that device's limits
    //
    // Test approach:
    // 1. Create cgroup with io controller
    // 2. Set limit for first device
    // 3. Set limit for second device
    // 4. Verify both limits appear in io.max
    // 5. Clean up

    todo!("Implement test for multiple device limits")
}
