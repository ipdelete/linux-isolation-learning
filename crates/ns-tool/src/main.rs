use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod error;
pub use error::{NamespaceKind, NsError, NsResult};

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
    CheckCaps,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // TODO: Implement PID namespace subcommand
        // Lesson: docs/01-namespaces/01-pid-namespace.md
        // Tests: tests/pid_test.rs
        //
        // TDD Steps:
        // 1. First, write tests in tests/pid_test.rs (RED)
        // 2. Then implement this function to make tests pass (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Use nix::sched::unshare(CloneFlags::CLONE_NEWPID)
        // - Fork a child process with nix::unistd::fork()
        // - In child: getpid() should return 1
        // - Print "PID inside namespace: {pid}"
        Command::Pid => todo!("Implement PID namespace - write tests first!"),

        // TODO: Implement UTS namespace subcommand
        // Lesson: docs/01-namespaces/02-uts-namespace.md
        // Tests: tests/uts_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/uts_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Use nix::sched::unshare(CloneFlags::CLONE_NEWUTS)
        // - Use nix::unistd::sethostname() to set custom hostname
        // - Print old and new hostnames to verify isolation
        Command::Uts => todo!("Implement UTS namespace - write tests first!"),

        // TODO: Implement IPC namespace subcommand
        // Lesson: docs/01-namespaces/03-ipc-namespace.md
        // Tests: tests/ipc_test.rs
        Command::Ipc => todo!("Implement IPC namespace - write tests first!"),

        // TODO: Implement mount namespace subcommand
        // Lesson: docs/01-namespaces/04-mount-namespace.md
        // Tests: tests/mount_test.rs
        Command::Mount => todo!("Implement mount namespace - write tests first!"),

        // TODO: Implement network namespace subcommand
        // Lesson: docs/01-namespaces/05-network-namespace.md
        // Tests: (network tests are in netns-tool crate)
        // Note: For basic network namespace creation, see netns-tool
        Command::Net => todo!("Implement network namespace - write tests first!"),

        // TODO: Implement user namespace subcommand
        // Lesson: docs/01-namespaces/06-user-namespace.md
        // Tests: tests/user_test.rs
        Command::User => todo!("Implement user namespace - write tests first!"),

        // TODO: Implement cgroup namespace subcommand
        // Lesson: docs/01-namespaces/07-cgroup-namespace.md
        // Tests: (cgroup tests are in cgroup-tool crate)
        Command::Cgroup => todo!("Implement cgroup namespace - write tests first!"),

        // TODO: Implement time namespace subcommand
        // Lesson: docs/01-namespaces/08-time-namespace.md
        // Tests: (add tests/time_test.rs when implementing)
        Command::Time => todo!("Implement time namespace - write tests first!"),

        // TODO: Implement setns subcommand (joining existing namespaces)
        // Lesson: docs/01-namespaces/09-setns.md
        // Tests: tests/setns_test.rs
        Command::Setns => todo!("Implement setns - write tests first!"),

        // This is already implemented as a reference example
        // Study this before implementing other subcommands
        Command::Proc => print_proc_ns()?,

        // TODO: Implement check-caps subcommand (capability inspection)
        // Lesson: docs/00-foundations/04-permissions-and-sudo.md
        // Tests: tests/caps_test.rs
        //
        // TDD Steps:
        // 1. First, write tests in tests/caps_test.rs (RED)
        // 2. Then implement this function to make tests pass (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Read /proc/self/status to get CapEff (effective capabilities)
        // - Parse the hex value to check for CAP_SYS_ADMIN (bit 21)
        // - Report which namespaces can be created with current privileges
        Command::CheckCaps => todo!("Implement check-caps - write tests first!"),
    }

    Ok(())
}

fn print_proc_ns() -> Result<()> {
    let ns_path = "/proc/self/ns";

    // Using anyhow's Context trait to add context to errors
    let entries = std::fs::read_dir(ns_path)
        .with_context(|| format!("failed to read namespace directory: {}", ns_path))?;

    for entry in entries {
        let entry = entry.with_context(|| "failed to read directory entry")?;
        let name = entry.file_name();
        let target = std::fs::read_link(entry.path())
            .with_context(|| format!("failed to read symlink: {}", entry.path().display()))?;
        println!("{} -> {}", name.to_string_lossy(), target.display());
    }
    Ok(())
}
