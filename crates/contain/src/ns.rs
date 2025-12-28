// Namespace subcommands for the contain CLI
// These implement the core namespace isolation concepts from fast-track lessons.

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum NsCommand {
    /// Create a PID namespace and run a shell
    /// Lesson: docs/fast-track/01-pid-namespace.md
    Pid,

    /// Create a mount namespace with isolated /tmp
    /// Lesson: docs/fast-track/02-mount-namespace.md
    Mount,

    /// Create a mini-container with combined namespaces (PID + mount + UTS)
    /// Lesson: docs/fast-track/04-combine.md
    Container,
}

impl NsCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            NsCommand::Pid => {
                // TODO: Implement PID namespace isolation
                // Lesson: docs/fast-track/01-pid-namespace.md
                // Tests: tests/ns_test.rs
                //
                // Implementation hints:
                // - Use nix::sched::unshare with CloneFlags::CLONE_NEWPID
                // - Fork a child process
                // - Child sees itself as PID 1
                todo!("Implement PID namespace - see docs/fast-track/01-pid-namespace.md")
            }
            NsCommand::Mount => {
                // TODO: Implement mount namespace isolation
                // Lesson: docs/fast-track/02-mount-namespace.md
                // Tests: tests/ns_test.rs
                //
                // Implementation hints:
                // - Use nix::sched::unshare with CloneFlags::CLONE_NEWNS
                // - Create isolated /tmp with tmpfs
                // - Files created inside are invisible to host
                todo!("Implement mount namespace - see docs/fast-track/02-mount-namespace.md")
            }
            NsCommand::Container => {
                // TODO: Implement combined namespace container
                // Lesson: docs/fast-track/04-combine.md
                // Tests: tests/ns_test.rs
                //
                // Implementation hints:
                // - Combine CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS
                // - Set hostname inside container
                // - Mount private /proc
                todo!("Implement mini-container - see docs/fast-track/04-combine.md")
            }
        }
    }
}
