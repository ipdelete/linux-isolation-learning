// Tests for the `tracepoint` subcommand
// Lesson: docs/04-ebpf/06-tracepoints.md
//
// TDD Workflow:
// 1. Write tests below FIRST (RED)
// 2. Implement code in src/main.rs and ebpf-tool-ebpf/src/tracepoint.rs (GREEN)
//
// Tracepoints are stable kernel instrumentation points defined in the kernel source.
// Unlike kprobes, tracepoints have a stable ABI and are less likely to break across
// kernel versions.
//
// Usage: ebpf-tool tracepoint <category> <name> [-d duration]
// Example: ebpf-tool tracepoint syscalls sys_enter_openat -d 5
//
// NOTE: Most tests require root privileges (CAP_BPF or CAP_SYS_ADMIN).
// Run with: sudo -E cargo test -p ebpf-tool

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to check if running as root.
/// Tests that require root will skip if this returns false.
fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

// =============================================================================
// Non-root tests (can run without privileges)
// =============================================================================

#[test]
fn test_tracepoint_help() {
    // TODO: Verify that `ebpf-tool tracepoint --help` shows usage information
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool") to get the binary
    // - Add args: ["tracepoint", "--help"]
    // - Assert success (exit code 0)
    // - Check stdout contains "tracepoint" and usage info
    // - Check for category and name arguments in help text
    //
    // Example assertions:
    // - stdout.contains("category") - the tracepoint category argument
    // - stdout.contains("name") - the tracepoint name argument
    // - stdout.contains("duration") - the optional duration flag

    todo!("Implement test for tracepoint help text")
}

#[test]
fn test_tracepoint_requires_category_arg() {
    // TODO: Verify that running `ebpf-tool tracepoint` without arguments fails
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add args: ["tracepoint"] (missing category and name)
    // - Assert failure (non-zero exit code)
    // - Check stderr contains error about missing argument
    //
    // The command should fail because <category> is a required positional argument.
    // clap will produce an error message indicating the missing argument.

    todo!("Implement test for missing category argument")
}

#[test]
fn test_tracepoint_requires_name_arg() {
    // TODO: Verify that running `ebpf-tool tracepoint syscalls` without name fails
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add args: ["tracepoint", "syscalls"] (missing name)
    // - Assert failure (non-zero exit code)
    // - Check stderr contains error about missing argument
    //
    // The command should fail because <name> is a required positional argument.
    // Even with a valid category, the name must also be provided.

    todo!("Implement test for missing name argument")
}

// =============================================================================
// Root-required tests (require CAP_BPF or root privileges)
// =============================================================================

#[test]
fn test_tracepoint_attaches_successfully() {
    // TODO: Verify that the tracepoint subcommand can attach to a valid tracepoint
    //
    // Skip this test if not running as root:
    // if !is_root() {
    //     eprintln!("Skipping test_tracepoint_attaches_successfully: requires root");
    //     return;
    // }
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add args: ["tracepoint", "sched", "sched_switch", "-d", "1"]
    // - sched/sched_switch is a common tracepoint available on all Linux systems
    // - Assert success (exit code 0)
    // - Check output indicates successful attachment
    //
    // The sched_switch tracepoint fires on every context switch, so it's
    // guaranteed to produce events during the 1-second duration.
    //
    // Expected output should contain:
    // - "Attaching to tracepoint: sched/sched_switch" or similar
    // - Indication of successful attachment

    todo!("Implement test for successful tracepoint attachment")
}

#[test]
fn test_tracepoint_syscalls_openat() {
    // TODO: Verify tracepoint works with syscalls/sys_enter_openat
    //
    // Skip this test if not running as root:
    // if !is_root() {
    //     eprintln!("Skipping test_tracepoint_syscalls_openat: requires root");
    //     return;
    // }
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add args: ["tracepoint", "syscalls", "sys_enter_openat", "-d", "2"]
    // - The syscalls category contains tracepoints for system call entry/exit
    // - sys_enter_openat fires when the openat() syscall is invoked
    // - Assert success (exit code 0)
    //
    // Note: sys_enter_openat is heavily used (file opens happen constantly),
    // so this tracepoint should capture events during the test.
    //
    // You might want to trigger some file opens during the test:
    // - Spawn a background process that opens files
    // - Or rely on system activity during the 2-second window

    todo!("Implement test for syscalls/sys_enter_openat tracepoint")
}

#[test]
fn test_tracepoint_shows_events() {
    // TODO: Verify that the tracepoint outputs captured events
    //
    // Skip this test if not running as root:
    // if !is_root() {
    //     eprintln!("Skipping test_tracepoint_shows_events: requires root");
    //     return;
    // }
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add args: ["tracepoint", "sched", "sched_switch", "-d", "2"]
    // - sched_switch is very frequent, so events should be captured
    // - Assert success (exit code 0)
    // - Check stdout contains event data (PID, timestamp, or event count)
    //
    // The eBPF program should log or output information about each event.
    // Verify that the output contains meaningful data like:
    // - Process IDs (PIDs)
    // - Timestamps
    // - Event counts or statistics
    //
    // Example check: stdout contains digits (PIDs, counts, etc.)

    todo!("Implement test for tracepoint event output")
}

#[test]
fn test_tracepoint_invalid_category() {
    // TODO: Verify that an invalid tracepoint category produces an error
    //
    // Skip this test if not running as root:
    // if !is_root() {
    //     eprintln!("Skipping test_tracepoint_invalid_category: requires root");
    //     return;
    // }
    //
    // Hints:
    // - Use Command::cargo_bin("ebpf-tool")
    // - Add args: ["tracepoint", "nonexistent_category", "fake_name", "-d", "1"]
    // - Assert failure (non-zero exit code)
    // - Check stderr contains error message about invalid tracepoint
    //
    // When attaching to a non-existent tracepoint, the aya framework
    // should return an error. The tool should handle this gracefully
    // and display a helpful error message.
    //
    // Expected error message should indicate:
    // - The tracepoint was not found
    // - Or the category/name combination is invalid
    // - Possibly suggest checking /sys/kernel/debug/tracing/events/

    todo!("Implement test for invalid tracepoint category error")
}

// =============================================================================
// Additional test ideas (optional, for learners who want more practice)
// =============================================================================

#[test]
#[ignore] // Remove this attribute when implementing
fn test_tracepoint_duration_zero_runs_until_interrupted() {
    // TODO: Verify that duration=0 runs until Ctrl+C (manual interrupt)
    //
    // This is tricky to test automatically. Consider:
    // - Start the command in a separate thread
    // - Wait a short time
    // - Send SIGINT to the process
    // - Verify it exits gracefully
    //
    // This test is marked #[ignore] because it requires signal handling.

    todo!("Implement test for duration=0 behavior")
}

#[test]
#[ignore] // Remove this attribute when implementing
fn test_tracepoint_net_netif_rx() {
    // TODO: Verify tracepoint works with net/netif_rx (network receive)
    //
    // Hints:
    // - The net category contains network-related tracepoints
    // - netif_rx fires when packets are received
    // - You might need to generate network traffic to trigger events
    //
    // This is an advanced test that requires network activity.

    todo!("Implement test for net/netif_rx tracepoint")
}
