// Network namespace subcommands for the contain CLI
// These implement network isolation from fast-track lesson 03.

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum NetCommand {
    /// Create a new network namespace
    /// Lesson: docs/fast-track/03-network-namespace.md
    Create {
        /// Name for the network namespace
        name: String,
    },

    /// Delete an existing network namespace
    /// Lesson: docs/fast-track/03-network-namespace.md
    Delete {
        /// Name of the network namespace to delete
        name: String,
    },

    /// Create a veth pair connecting host and network namespace
    /// Lesson: docs/fast-track/03-network-namespace.md
    Veth {
        /// Name for the host-side veth interface
        #[arg(long)]
        host: String,

        /// Name of the network namespace (and namespace-side interface)
        #[arg(long)]
        ns: String,
    },
}

impl NetCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            NetCommand::Create { name } => {
                // TODO: Create a new network namespace
                // Lesson: docs/fast-track/03-network-namespace.md
                // Tests: tests/net_test.rs
                //
                // Implementation hints:
                // - Use `ip netns add <name>` or nix syscalls
                // - Creates /var/run/netns/<name>
                let _ = name; // Suppress unused warning
                todo!("Implement network namespace creation - see docs/fast-track/03-network-namespace.md")
            }
            NetCommand::Delete { name } => {
                // TODO: Delete an existing network namespace
                // Lesson: docs/fast-track/03-network-namespace.md
                // Tests: tests/net_test.rs
                //
                // Implementation hints:
                // - Use `ip netns del <name>` or unlink /var/run/netns/<name>
                let _ = name; // Suppress unused warning
                todo!("Implement network namespace deletion - see docs/fast-track/03-network-namespace.md")
            }
            NetCommand::Veth { host, ns } => {
                // TODO: Create veth pair and move one end to namespace
                // Lesson: docs/fast-track/03-network-namespace.md
                // Tests: tests/net_test.rs
                //
                // Implementation hints:
                // - Create veth pair with `ip link add`
                // - Move one end to namespace with `ip link set netns`
                // - Assign IP addresses to both ends
                let _ = (host, ns); // Suppress unused warning
                todo!("Implement veth pair creation - see docs/fast-track/03-network-namespace.md")
            }
        }
    }
}
