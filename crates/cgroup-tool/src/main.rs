use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cgroup-tool")]
#[command(about = "Cgroup v2 tool (Rust-first rewrite)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Create { path: String },
    Delete { path: String },
    Attach { path: String, pid: u32 },
    MemoryMax { path: String, bytes: u64 },
    CpuMax { path: String, quota: String },
    PidsMax { path: String, max: u64 },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Create { path } => println!("todo: create cgroup {path}"),
        Command::Delete { path } => println!("todo: delete cgroup {path}"),
        Command::Attach { path, pid } => println!("todo: attach pid {pid} to {path}"),
        Command::MemoryMax { path, bytes } => println!("todo: set memory.max {bytes} for {path}"),
        Command::CpuMax { path, quota } => println!("todo: set cpu.max {quota} for {path}"),
        Command::PidsMax { path, max } => println!("todo: set pids.max {max} for {path}"),
    }

    Ok(())
}
