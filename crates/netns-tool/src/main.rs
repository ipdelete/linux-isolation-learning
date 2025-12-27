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
        Command::Create { name } => {
            println!("todo: create netns {name}");
        }
        Command::Delete { name } => {
            println!("todo: delete netns {name}");
        }
        Command::Veth { host, ns } => {
            println!("todo: create veth host={host} ns={ns}");
        }
        Command::Bridge { name } => {
            println!("todo: create bridge {name}");
        }
        Command::Nat { bridge, outbound } => {
            println!("todo: add NAT for bridge={bridge} outbound={outbound}");
        }
    }

    Ok(())
}
