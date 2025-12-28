// contain - Learn container internals hands-on
//
// A unified CLI tool for the fast-track tutorials.
// Each subcommand corresponds to a lesson in docs/fast-track/.
//
// Usage:
//   contain ns pid          - PID namespace isolation
//   contain ns mount        - Mount namespace isolation
//   contain ns container    - Combined namespaces (mini-container)
//   contain net create      - Create network namespace
//   contain net delete      - Delete network namespace
//   contain net veth        - Create veth pair
//   contain cgroup create   - Create cgroup
//   contain cgroup delete   - Delete cgroup
//   contain cgroup attach   - Attach process to cgroup
//   contain cgroup memory   - Set memory limit
//   contain cgroup cpu      - Set CPU limit
//   contain oci init        - Initialize OCI bundle
//   contain oci run         - Run container with runc
//   contain trace check     - Check eBPF support
//   contain trace syscalls  - Trace syscalls with eBPF
//   contain trace events    - Trace container events

use anyhow::Result;
use clap::{Parser, Subcommand};

mod cgroup;
mod net;
mod ns;
mod oci;
mod trace;

#[derive(Parser)]
#[command(name = "contain")]
#[command(version = "0.1.0")]
#[command(about = "Learn container internals hands-on")]
#[command(long_about = "A unified CLI for the fast-track container tutorials.\n\n\
    Each subcommand teaches a core container concept:\n\
    - ns: Linux namespaces (PID, mount, network)\n\
    - net: Network namespace management\n\
    - cgroup: Resource limits (memory, CPU)\n\
    - oci: OCI bundle format and runc\n\
    - trace: eBPF observability")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Namespace operations (PID, mount, combined)
    /// Lessons: 01-pid, 02-mount, 04-combine
    Ns {
        #[command(subcommand)]
        cmd: ns::NsCommand,
    },

    /// Network namespace operations
    /// Lesson: 03-network-namespace
    Net {
        #[command(subcommand)]
        cmd: net::NetCommand,
    },

    /// Cgroup resource limit operations
    /// Lessons: 05-cgroup-basics, 06-memory, 07-cpu
    Cgroup {
        #[command(subcommand)]
        cmd: cgroup::CgroupCommand,
    },

    /// OCI bundle operations
    /// Lessons: 08-oci-bundle, 09-runc-run
    Oci {
        #[command(subcommand)]
        cmd: oci::OciCommand,
    },

    /// eBPF tracing operations
    /// Lesson: 10-ebpf-tracing
    Trace {
        #[command(subcommand)]
        cmd: trace::TraceCommand,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Ns { cmd } => cmd.run(),
        Command::Net { cmd } => cmd.run(),
        Command::Cgroup { cmd } => cmd.run(),
        Command::Oci { cmd } => cmd.run(),
        Command::Trace { cmd } => cmd.run(),
    }
}
