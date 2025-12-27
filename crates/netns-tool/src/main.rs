use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "netns-tool")]
#[command(about = "Network namespace tool (Rust-first rewrite)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Create { name: String },
    Delete { name: String },
    Veth { host: String, ns: String },
    Bridge { name: String },
    Nat { bridge: String, outbound: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // TODO: Implement network namespace creation
        // Lesson: docs/01-namespaces/05-network-namespace.md (part 1)
        // Tests: tests/create_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/create_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Create /run/netns directory if needed
        // - Use nix::sched::unshare(CloneFlags::CLONE_NEWNET)
        // - Bind-mount /proc/self/ns/net to /run/netns/{name}
        // - This makes the namespace persistent
        Command::Create { name } => {
            todo!("Implement network namespace creation - write tests first! (name: {name})")
        }

        // TODO: Implement network namespace deletion
        // Lesson: docs/01-namespaces/05-network-namespace.md (part 2)
        // Tests: tests/delete_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/delete_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Unmount /run/netns/{name}
        // - Remove the file
        // - Handle errors gracefully if namespace doesn't exist
        Command::Delete { name } => {
            todo!("Implement network namespace deletion - write tests first! (name: {name})")
        }

        // TODO: Implement veth pair creation
        // Lesson: docs/01-namespaces/05-network-namespace.md (part 3)
        // Tests: tests/veth_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/veth_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Create veth pair using rtnetlink crate or ip command
        // - Move one end to target namespace
        // - Assign IP addresses and bring interfaces UP
        // - For rtnetlink: see examples in rtnetlink crate docs
        Command::Veth { host, ns } => {
            todo!("Implement veth pair creation - write tests first! (host: {host}, ns: {ns})")
        }

        // TODO: Implement bridge creation
        // Lesson: docs/01-namespaces/05-network-namespace.md (part 4)
        // Tests: tests/bridge_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/bridge_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Use `ip link add {name} type bridge`
        // - Bring bridge UP
        // - Optionally assign IP address to bridge
        Command::Bridge { name } => {
            todo!("Implement bridge creation - write tests first! (name: {name})")
        }

        // TODO: Implement NAT setup for internet access
        // Lesson: docs/01-namespaces/05-network-namespace.md (part 5)
        // Tests: tests/nat_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/nat_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Enable IP forwarding: echo 1 > /proc/sys/net/ipv4/ip_forward
        // - Add iptables MASQUERADE rule
        // - Add forward accept rules for the bridge
        Command::Nat { bridge, outbound } => {
            todo!(
                "Implement NAT setup - write tests first! (bridge: {bridge}, outbound: {outbound})"
            )
        }
    }

    Ok(())
}
