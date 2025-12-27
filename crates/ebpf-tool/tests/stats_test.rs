// Tests for the `stats` subcommand
// Lesson: docs/04-ebpf/03-maps.md
//
// TDD Workflow:
// 1. Write tests below FIRST (RED)
// 2. Implement code in src/main.rs (GREEN)
//
// The `stats` subcommand displays eBPF map statistics from a HashMap that
// tracks syscall counts. It reads the SYSCALL_COUNTS map and presents the
// data in a formatted table.
//
// NOTE: Most tests require root privileges to load eBPF programs.
// Run with: sudo -E cargo test -p ebpf-tool

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to check if running as root.
/// Tests that require eBPF capabilities should skip if not root.
fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

#[test]
fn test_stats_help() {
    // TODO: Verify that `ebpf-tool stats --help` shows usage information
    //
    // This test does NOT require root because --help doesn't load eBPF programs.
    //
    // Hints:
    // - Use assert_cmd::Command::cargo_bin("ebpf-tool")
    // - Pass args: ["stats", "--help"]
    // - Assert success and check stdout contains expected help text
    // - Look for: "stats", "map", "statistics", or similar keywords
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["stats", "--help"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("eBPF map statistics"));

    todo!("Implement test for stats --help output")
}

#[test]
fn test_stats_runs_successfully() {
    // TODO: Verify that `ebpf-tool stats` runs and exits successfully
    //
    // This test REQUIRES root to load eBPF programs and access maps.
    // Skip the test if not running as root.
    //
    // Hints:
    // - Check is_root() first and return early if false
    // - Use assert_cmd::Command::cargo_bin("ebpf-tool")
    // - Pass arg: "stats"
    // - Assert success (exit code 0)
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_stats_runs_successfully: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.arg("stats")
    //    .assert()
    //    .success();

    todo!("Implement test that stats subcommand runs successfully")
}

#[test]
fn test_stats_shows_table_header() {
    // TODO: Verify that the stats output includes a proper table header
    //
    // This test REQUIRES root to load eBPF programs and access maps.
    // Skip the test if not running as root.
    //
    // The expected output format is:
    //   Syscall Statistics:
    //   ------------------
    //   SYSCALL          COUNT
    //   openat           1234
    //   read             5678
    //
    // Hints:
    // - Check is_root() first and return early if false
    // - Look for header text like "Syscall" or "Statistics" or "COUNT"
    // - Use predicate::str::contains() for flexible matching
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_stats_shows_table_header: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.arg("stats")
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("Syscall")
    //        .or(predicate::str::contains("SYSCALL")))
    //    .stdout(predicate::str::contains("COUNT")
    //        .or(predicate::str::contains("Count")));

    todo!("Implement test that verifies table header is displayed")
}

#[test]
fn test_stats_shows_syscall_counts() {
    // TODO: Verify that syscall names and counts appear in the output
    //
    // This test REQUIRES root to load eBPF programs and access maps.
    // Skip the test if not running as root.
    //
    // After running the stats command, the output should show:
    // - At least one syscall name (e.g., "read", "write", "openat")
    // - Numeric counts (digits 0-9)
    //
    // Hints:
    // - Check is_root() first and return early if false
    // - Common syscalls that always occur: read, write, close, openat
    // - Use predicate::str::is_match(r"\d+") to verify numbers appear
    // - The map may be empty initially if no eBPF program has populated it yet
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_stats_shows_syscall_counts: requires root");
    //     return;
    // }
    //
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // let output = cmd.arg("stats")
    //    .assert()
    //    .success();
    //
    // // Check that output contains at least one common syscall or is empty
    // // (empty is valid if map hasn't been populated yet)
    // let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // let has_syscall = stdout.contains("read")
    //     || stdout.contains("write")
    //     || stdout.contains("openat")
    //     || stdout.contains("No data");
    // assert!(has_syscall, "Expected syscall names or 'No data' message");

    todo!("Implement test that verifies syscall counts are displayed")
}

#[test]
fn test_stats_after_workload() {
    // TODO: Verify that counts increase after generating syscall activity
    //
    // This test REQUIRES root to load eBPF programs and access maps.
    // Skip the test if not running as root.
    //
    // This is a more complex integration test that:
    // 1. Runs stats once and captures initial counts
    // 2. Generates some syscall activity (e.g., file operations)
    // 3. Runs stats again and verifies counts increased
    //
    // Hints:
    // - Check is_root() first and return early if false
    // - Generate syscalls by reading/writing temp files:
    //   std::fs::write("/tmp/ebpf-test", "hello")
    //   std::fs::read("/tmp/ebpf-test")
    // - Parse output to extract counts (or just verify output changed)
    // - The eBPF program must be loaded and attached during this test
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_stats_after_workload: requires root");
    //     return;
    // }
    //
    // // Step 1: Note that the stats command loads the eBPF program
    // // which starts counting syscalls from that point forward
    //
    // // Step 2: Generate syscall activity
    // let test_path = "/tmp/ebpf-stats-test";
    // for _ in 0..10 {
    //     std::fs::write(test_path, b"test data").unwrap();
    //     let _ = std::fs::read(test_path);
    // }
    //
    // // Step 3: Run stats and verify counts are non-zero
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // let output = cmd.arg("stats")
    //    .assert()
    //    .success();
    //
    // let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // // After file operations, we should see non-zero counts
    // assert!(
    //     stdout.contains(|c: char| c.is_ascii_digit() && c != '0'),
    //     "Expected non-zero counts after generating syscalls"
    // );
    //
    // // Cleanup
    // let _ = std::fs::remove_file(test_path);

    todo!("Implement test that verifies counts increase after syscall activity")
}
