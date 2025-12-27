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
        // TODO: Implement cgroup creation
        // Lesson: docs/02-cgroups/01-create-attach.md (part 1)
        // Tests: tests/create_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/create_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Cgroup v2 root is typically at /sys/fs/cgroup
        // - Create cgroup by creating directory: /sys/fs/cgroup/{path}
        // - Use std::fs::create_dir or create_dir_all for nested paths
        // - Verify cgroup.procs file exists after creation
        Command::Create { path } => {
            todo!("Implement cgroup creation - write tests first! (path: {path})")
        }

        // TODO: Implement cgroup deletion
        // Lesson: docs/02-cgroups/01-create-attach.md (part 3)
        // Tests: tests/delete_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/delete_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Remove cgroup by removing directory: std::fs::remove_dir
        // - Cgroup must be empty (no processes, no child cgroups) to delete
        // - Returns EBUSY if not empty
        Command::Delete { path } => {
            todo!("Implement cgroup deletion - write tests first! (path: {path})")
        }

        // TODO: Implement process attachment
        // Lesson: docs/02-cgroups/01-create-attach.md (part 2)
        // Tests: tests/attach_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/attach_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Write PID to /sys/fs/cgroup/{path}/cgroup.procs
        // - Format: write PID as string (e.g., "12345\n")
        // - Verify by reading cgroup.procs after write
        // - Can also check /proc/{pid}/cgroup
        Command::Attach { path, pid } => {
            todo!("Implement process attachment - write tests first! (path: {path}, pid: {pid})")
        }

        // TODO: Implement memory limit setting
        // Lesson: docs/02-cgroups/02-memory.md
        // Tests: tests/memory_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/memory_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Write bytes to /sys/fs/cgroup/{path}/memory.max
        // - Format: write number as string (e.g., "104857600" for 100MB)
        // - Can write "max" to remove limit
        // - Verify by reading memory.max after write
        Command::MemoryMax { path, bytes } => {
            todo!("Implement memory limit - write tests first! (path: {path}, bytes: {bytes})")
        }

        // TODO: Implement CPU quota setting
        // Lesson: docs/02-cgroups/03-cpu.md
        // Tests: tests/cpu_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/cpu_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Write quota to /sys/fs/cgroup/{path}/cpu.max
        // - Format: "quota period" (both in microseconds)
        // - Example: "50000 100000" = 50% CPU
        // - Can write "max" to remove limit
        Command::CpuMax { path, quota } => {
            todo!("Implement CPU quota - write tests first! (path: {path}, quota: {quota})")
        }

        // TODO: Implement PIDs limit setting
        // Lesson: docs/02-cgroups/04-pids.md
        // Tests: tests/pids_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/pids_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Write max to /sys/fs/cgroup/{path}/pids.max
        // - Format: write number as string
        // - Can write "max" to remove limit
        // - Verify by reading pids.max after write
        Command::PidsMax { path, max } => {
            todo!("Implement PIDs limit - write tests first! (path: {path}, max: {max})")
        }
    }

    Ok(())
}
