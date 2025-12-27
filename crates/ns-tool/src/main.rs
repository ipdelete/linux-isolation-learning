use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ns-tool")]
#[command(about = "Namespace learning tool (Rust-first rewrite)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Pid,
    Uts,
    Ipc,
    Mount,
    Net,
    User,
    Cgroup,
    Time,
    Setns,
    Proc,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Pid => println!("todo: implement PID namespace example"),
        Command::Uts => println!("todo: implement UTS namespace example"),
        Command::Ipc => println!("todo: implement IPC namespace example"),
        Command::Mount => println!("todo: implement mount namespace example"),
        Command::Net => println!("todo: implement network namespace example"),
        Command::User => println!("todo: implement user namespace example"),
        Command::Cgroup => println!("todo: implement cgroup namespace example"),
        Command::Time => println!("todo: implement time namespace example"),
        Command::Setns => println!("todo: implement setns example"),
        Command::Proc => print_proc_ns()?,
    }

    Ok(())
}

fn print_proc_ns() -> Result<()> {
    let entries = std::fs::read_dir("/proc/self/ns")?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let target = std::fs::read_link(entry.path())?;
        println!("{} -> {}", name.to_string_lossy(), target.display());
    }
    Ok(())
}
