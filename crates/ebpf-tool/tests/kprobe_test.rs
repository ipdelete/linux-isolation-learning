// Tests for the `kprobe` subcommand
// Lessons: docs/04-ebpf/01-hello-kprobe.md, docs/04-ebpf/02-reading-data.md
//
// TDD Workflow:
// 1. Write tests below FIRST (RED)
// 2. Implement code in src/main.rs and ebpf-tool-ebpf/src/kprobe.rs (GREEN)
//
// Kprobe Overview:
// - Kprobes allow dynamic tracing of kernel functions
// - Usage: `ebpf-tool kprobe <function> [-d duration]`
// - Common functions to probe: do_sys_openat2, vfs_read, vfs_write
//
// NOTE: Most kprobe tests require root privileges (CAP_BPF or CAP_SYS_ADMIN).
// Tests that require root will skip automatically when run as a normal user.
// Run with: sudo -E cargo test -p ebpf-tool

use assert_cmd::Command;
use predicates::prelude::*;

// =============================================================================
// Helper: Check if running as root
// =============================================================================

/// Returns true if the current process is running as root.
/// Used to skip tests that require elevated privileges.
fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

// =============================================================================
// Basic CLI Tests (no root required)
// =============================================================================

#[test]
fn test_kprobe_help() {
    // TODO: Verify that `ebpf-tool kprobe --help` shows usage information
    //
    // This test does NOT require root privileges.
    //
    // Expected behavior:
    // - Command should exit successfully
    // - Output should contain usage information about the kprobe subcommand
    // - Should mention the <FUNCTION> argument
    // - Should mention the -d/--duration option
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool") to get the binary
    // - Add args: ["kprobe", "--help"]
    // - Use .assert().success() to verify exit code
    // - Use .stdout(predicate::str::contains(...)) to check output
    //
    // Implementation skeleton:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["kprobe", "--help"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("FUNCTION"))
    //    .stdout(predicate::str::contains("duration"));

    todo!("Implement test for kprobe --help output")
}

#[test]
fn test_kprobe_requires_function_arg() {
    // TODO: Verify that `ebpf-tool kprobe` without a function argument fails
    //
    // This test does NOT require root privileges.
    //
    // Expected behavior:
    // - Command should fail (non-zero exit code)
    // - Error message should indicate that <FUNCTION> is required
    //
    // Hints:
    // - Run `ebpf-tool kprobe` with no additional arguments
    // - Use .assert().failure() to verify non-zero exit code
    // - Use .stderr(predicate::str::contains(...)) to check error message
    //
    // Implementation skeleton:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.arg("kprobe")
    //    .assert()
    //    .failure()
    //    .stderr(predicate::str::contains("FUNCTION"));

    todo!("Implement test verifying function argument is required")
}

// =============================================================================
// Kprobe Attachment Tests (require root)
// =============================================================================

#[test]
fn test_kprobe_attaches_to_kernel_function() {
    // TODO: Verify that kprobe successfully attaches to a valid kernel function
    //
    // This test REQUIRES root privileges.
    // Skip the test if not running as root.
    //
    // Expected behavior:
    // - Attach kprobe to a commonly available kernel function (e.g., "do_sys_openat2")
    // - Command should start successfully
    // - Output should indicate the probe was attached
    // - Use a short duration (-d 1) so test completes quickly
    //
    // Hints:
    // - First check is_root() and skip if not root
    // - Use "do_sys_openat2" as a reliable kernel function (handles open() syscall)
    // - Pass "-d 1" to run for only 1 second
    // - Look for output indicating successful attachment
    //
    // Implementation skeleton:
    // if !is_root() {
    //     eprintln!("Skipping test_kprobe_attaches_to_kernel_function: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["kprobe", "do_sys_openat2", "-d", "1"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("Attaching").or(predicate::str::contains("attached")));

    todo!("Implement test for kprobe attachment to kernel function")
}

#[test]
fn test_kprobe_shows_events() {
    // TODO: Verify that kprobe logs events when the probed function is called
    //
    // This test REQUIRES root privileges.
    // Skip the test if not running as root.
    //
    // Expected behavior:
    // - Attach kprobe to a function that will be triggered during the test
    // - Generate some activity that calls the probed function
    // - Verify that events are logged in the output
    //
    // Hints:
    // - Use "do_sys_openat2" which is called when files are opened
    // - The test itself will cause file opens (loading the binary, etc.)
    // - Alternatively, spawn a subprocess that opens a file
    // - Look for event output showing the function was called
    // - Consider using timeout or a very short duration
    //
    // Implementation skeleton:
    // if !is_root() {
    //     eprintln!("Skipping test_kprobe_shows_events: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["kprobe", "do_sys_openat2", "-d", "2"])
    //    .assert()
    //    .success()
    //    // Look for evidence of events being captured
    //    .stdout(predicate::str::contains("event")
    //        .or(predicate::str::contains("called"))
    //        .or(predicate::str::contains("pid")));

    todo!("Implement test verifying kprobe logs events")
}

#[test]
fn test_kprobe_respects_duration() {
    // TODO: Verify that the -d/--duration flag controls how long the kprobe runs
    //
    // This test REQUIRES root privileges.
    // Skip the test if not running as root.
    //
    // Expected behavior:
    // - With -d 1, command should complete in approximately 1 second
    // - With -d 2, command should complete in approximately 2 seconds
    // - The actual time might be slightly longer due to setup/teardown
    //
    // Hints:
    // - Use std::time::Instant to measure elapsed time
    // - Run with -d 1 and verify it takes roughly 1-3 seconds
    // - Don't be too strict on timing (allow some tolerance)
    //
    // Implementation skeleton:
    // if !is_root() {
    //     eprintln!("Skipping test_kprobe_respects_duration: requires root");
    //     return;
    // }
    //
    // use std::time::Instant;
    //
    // let start = Instant::now();
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["kprobe", "do_sys_openat2", "-d", "1"])
    //    .assert()
    //    .success();
    // let elapsed = start.elapsed();
    //
    // // Should complete within 1-3 seconds (1 sec duration + some overhead)
    // assert!(elapsed.as_secs() >= 1, "Completed too quickly: {:?}", elapsed);
    // assert!(elapsed.as_secs() <= 3, "Took too long: {:?}", elapsed);

    todo!("Implement test verifying duration flag is respected")
}

#[test]
fn test_kprobe_invalid_function() {
    // TODO: Verify that kprobe fails gracefully with an invalid function name
    //
    // This test REQUIRES root privileges.
    // Skip the test if not running as root.
    //
    // Expected behavior:
    // - Attempting to attach to a non-existent kernel function should fail
    // - Error message should be informative (mention the function name)
    // - Command should exit with non-zero status
    //
    // Hints:
    // - Use a clearly invalid function name like "nonexistent_function_xyz123"
    // - The error may come from the kernel or from Aya's validation
    // - Check stderr for error message
    //
    // Implementation skeleton:
    // if !is_root() {
    //     eprintln!("Skipping test_kprobe_invalid_function: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["kprobe", "nonexistent_function_xyz123", "-d", "1"])
    //    .assert()
    //    .failure()
    //    .stderr(predicate::str::contains("nonexistent_function_xyz123")
    //        .or(predicate::str::contains("not found"))
    //        .or(predicate::str::contains("failed"))
    //        .or(predicate::str::contains("error")));

    todo!("Implement test for invalid function name handling")
}

// =============================================================================
// Lesson 02: Reading Data from Kprobe Context
// =============================================================================

#[test]
#[ignore] // Enable after completing Lesson 02
fn test_kprobe_reads_process_info() {
    // TODO: Verify that kprobe can read process information from the probe context
    //
    // This test REQUIRES root privileges.
    // This is part of Lesson 02: Reading Data.
    //
    // Expected behavior:
    // - When a kprobe fires, the eBPF program should read process info
    // - Output should include PID (process ID)
    // - Output may include process name (comm)
    //
    // Hints:
    // - Use bpf_get_current_pid_tgid() in the eBPF program
    // - Use bpf_get_current_comm() to get process name
    // - Events should show which process triggered the probe
    //
    // Implementation skeleton:
    // if !is_root() {
    //     eprintln!("Skipping test_kprobe_reads_process_info: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["kprobe", "do_sys_openat2", "-d", "2"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("pid")
    //        .or(predicate::str::contains("PID")));

    todo!("Implement test verifying process info is read from kprobe context")
}

#[test]
#[ignore] // Enable after completing Lesson 02
fn test_kprobe_reads_function_arguments() {
    // TODO: Verify that kprobe can read function arguments
    //
    // This test REQUIRES root privileges.
    // This is part of Lesson 02: Reading Data.
    //
    // Expected behavior:
    // - Kprobe should be able to access the arguments of the probed function
    // - For do_sys_openat2, this includes the file path being opened
    //
    // Hints:
    // - Access function arguments via ProbeContext
    // - For do_sys_openat2: ctx.arg(0) is dfd, ctx.arg(1) is filename pointer
    // - Reading strings from userspace requires bpf_probe_read_user_str()
    // - Be careful with pointer validation in eBPF
    //
    // Implementation skeleton:
    // if !is_root() {
    //     eprintln!("Skipping test_kprobe_reads_function_arguments: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["kprobe", "do_sys_openat2", "-d", "2"])
    //    .assert()
    //    .success()
    //    // Look for file path or argument data in output
    //    .stdout(predicate::str::contains("/")
    //        .or(predicate::str::contains("path"))
    //        .or(predicate::str::contains("arg")));

    todo!("Implement test verifying function arguments can be read")
}
