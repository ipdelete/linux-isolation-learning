use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "oci-tool")]
#[command(about = "OCI bundle helper (Rust-first rewrite)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init { bundle: String },
    Show { bundle: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { bundle } => println!("todo: init OCI bundle at {bundle}"),
        Command::Show { bundle } => println!("todo: show config.json in {bundle}"),
    }

    Ok(())
}
