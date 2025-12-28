// eBPF tracing subcommands for the contain CLI
// These implement observability from fast-track lesson 10.

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TraceCommand {
    /// Check eBPF support and prerequisites
    /// Lesson: docs/fast-track/10-ebpf-tracing.md
    Check,

    /// Trace system calls in a container using eBPF
    /// Lesson: docs/fast-track/10-ebpf-tracing.md
    Syscalls {
        /// Process ID to trace (optional, traces all if not specified)
        #[arg(long)]
        pid: Option<u32>,
    },

    /// Trace container events (clone, execve, exit)
    /// Lesson: docs/fast-track/10-ebpf-tracing.md
    Events,
}

impl TraceCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            TraceCommand::Check => {
                // TODO: Check eBPF support and prerequisites
                // Lesson: docs/fast-track/10-ebpf-tracing.md
                // Tests: tests/trace_test.rs
                //
                // Implementation hints:
                // - Check /sys/fs/bpf exists
                // - Check kernel version supports eBPF
                // - Check CAP_BPF or root privileges
                todo!("Implement eBPF check - see docs/fast-track/10-ebpf-tracing.md")
            }
            TraceCommand::Syscalls { pid } => {
                // TODO: Attach eBPF program to trace syscalls
                // Lesson: docs/fast-track/10-ebpf-tracing.md
                // Tests: tests/trace_test.rs
                //
                // Implementation hints:
                // - Load eBPF program for syscall tracing
                // - Filter by PID if specified
                // - Print syscall name and arguments
                let _ = pid; // Suppress unused warning
                todo!("Implement syscall tracing - see docs/fast-track/10-ebpf-tracing.md")
            }
            TraceCommand::Events => {
                // TODO: Trace container lifecycle events
                // Lesson: docs/fast-track/10-ebpf-tracing.md
                // Tests: tests/trace_test.rs
                //
                // Implementation hints:
                // - Attach to clone, execve, exit tracepoints
                // - Show container process creation and termination
                todo!("Implement event tracing - see docs/fast-track/10-ebpf-tracing.md")
            }
        }
    }
}
