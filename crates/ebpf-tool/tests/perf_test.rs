// Tests for the `perf` subcommand
// Lessons: docs/04-ebpf/04-perf-events.md, docs/04-ebpf/07-perf-sampling.md
//
// TDD Workflow:
// 1. Write tests below FIRST (RED)
// 2. Implement code in src/main.rs and ebpf-tool-ebpf/src/perf.rs (GREEN)
//
// NOTE: Most tests require root privileges for eBPF operations.
// Run with: sudo -E cargo test -p ebpf-tool
//
// The `perf` subcommand provides CPU performance sampling using eBPF perf events.
// Usage: ebpf-tool perf [-f frequency] [-d duration]

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to check if we have root privileges.
/// Tests that require root should call this and skip if not root.
fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

#[test]
fn test_perf_help() {
    // TODO: Verify that `ebpf-tool perf --help` shows usage information
    //
    // This test does NOT require root - it only checks help text.
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool") to get the binary
    // - Add args ["perf", "--help"]
    // - Assert success and check for expected help text
    // - Help should mention: frequency, duration, Hz, sampling
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["perf", "--help"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("frequency"))
    //    .stdout(predicate::str::contains("duration"));

    todo!("Implement test for perf help text")
}

#[test]
fn test_perf_default_frequency() {
    // TODO: Verify that the default sampling frequency is 99 Hz
    //
    // This test does NOT require root - it only checks help/arg parsing.
    // The default of 99 Hz is chosen to avoid lockstep with timer interrupts
    // (100 Hz is common), which would bias samples toward timer code.
    //
    // Hints:
    // - Check the help output for "default_value = \"99\""
    // - Or check that --help shows [default: 99] for frequency
    //
    // Implementation:
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["perf", "--help"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("99"));

    todo!("Implement test for default 99 Hz frequency")
}

#[test]
fn test_perf_runs_successfully() {
    // TODO: Verify that perf subcommand runs and exits cleanly
    //
    // REQUIRES ROOT: eBPF perf event attachment needs CAP_BPF or CAP_SYS_ADMIN
    //
    // Hints:
    // - Skip test if not running as root: if !is_root() { return; }
    // - Run with a short duration: -d 1 (1 second)
    // - Assert command exits successfully
    // - The command should attach to perf events, sample briefly, then exit
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_perf_runs_successfully: requires root");
    //     return;
    // }
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["perf", "-d", "1"])
    //    .assert()
    //    .success();

    todo!("Implement test for perf running successfully")
}

#[test]
fn test_perf_custom_frequency() {
    // TODO: Verify that custom sampling frequency is accepted
    //
    // REQUIRES ROOT: eBPF perf event attachment needs CAP_BPF or CAP_SYS_ADMIN
    //
    // Hints:
    // - Skip test if not running as root
    // - Use -f flag to set custom frequency (e.g., -f 49)
    // - Lower frequencies reduce overhead but capture fewer samples
    // - Run with short duration: -d 1
    // - Assert command succeeds
    // - Optionally check output mentions the frequency
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_perf_custom_frequency: requires root");
    //     return;
    // }
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["perf", "-f", "49", "-d", "1"])
    //    .assert()
    //    .success();

    todo!("Implement test for custom sampling frequency")
}

#[test]
fn test_perf_shows_samples() {
    // TODO: Verify that perf output includes sample data
    //
    // REQUIRES ROOT: eBPF perf event attachment needs CAP_BPF or CAP_SYS_ADMIN
    //
    // Hints:
    // - Skip test if not running as root
    // - Run for at least 1-2 seconds to collect samples
    // - Check stdout for sample-related output (e.g., "samples", "stack", "count")
    // - The exact format depends on implementation, but should show
    //   some indication that samples were collected
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_perf_shows_samples: requires root");
    //     return;
    // }
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["perf", "-d", "2"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("sample").or(
    //        predicate::str::contains("Sample").or(
    //            predicate::str::contains("collected")
    //        )
    //    ));

    todo!("Implement test for sample output")
}

#[test]
fn test_perf_respects_duration() {
    // TODO: Verify that perf respects the duration flag
    //
    // REQUIRES ROOT: eBPF perf event attachment needs CAP_BPF or CAP_SYS_ADMIN
    //
    // Hints:
    // - Skip test if not running as root
    // - Run with a known duration (e.g., 2 seconds)
    // - Measure elapsed time and verify it's approximately correct
    // - Allow some tolerance (e.g., +/- 1 second)
    // - Use std::time::Instant to measure
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_perf_respects_duration: requires root");
    //     return;
    // }
    // let start = std::time::Instant::now();
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["perf", "-d", "2"])
    //    .assert()
    //    .success();
    // let elapsed = start.elapsed();
    // assert!(elapsed.as_secs() >= 1, "Duration too short");
    // assert!(elapsed.as_secs() <= 4, "Duration too long");

    todo!("Implement test for duration flag")
}

#[test]
fn test_perf_samples_all_cpus() {
    // TODO: Verify that perf samples from all available CPUs
    //
    // REQUIRES ROOT: eBPF perf event attachment needs CAP_BPF or CAP_SYS_ADMIN
    //
    // Hints:
    // - Skip test if not running as root
    // - Perf events should be attached to all online CPUs
    // - Check output indicates multi-CPU sampling or CPU IDs
    // - Get CPU count: num_cpus::get() or read /proc/cpuinfo
    // - The output might show "CPU 0", "CPU 1", etc. or aggregate stats
    //
    // Implementation:
    // if !is_root() {
    //     eprintln!("Skipping test_perf_samples_all_cpus: requires root");
    //     return;
    // }
    // let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    // cmd.args(["perf", "-d", "2"])
    //    .assert()
    //    .success()
    //    .stdout(predicate::str::contains("CPU").or(
    //        predicate::str::contains("cpu")
    //    ));

    todo!("Implement test for multi-CPU sampling")
}
