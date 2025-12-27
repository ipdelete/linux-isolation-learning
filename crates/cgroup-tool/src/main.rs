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
    Create {
        path: String,
    },
    Delete {
        path: String,
    },
    Attach {
        path: String,
        pid: u32,
    },
    MemoryMax {
        path: String,
        bytes: u64,
    },
    CpuMax {
        path: String,
        quota: String,
    },
    PidsMax {
        path: String,
        max: u64,
    },
    /// Set I/O bandwidth/IOPS limits for a device
    IoMax {
        path: String,
        /// Device major:minor (e.g., "8:0" for /dev/sda)
        device: String,
        /// I/O limit specification (e.g., "rbps=1048576 wbps=1048576")
        limit: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // TODO: Implement cgroup creation
        // Lesson: docs/02-cgroups/01-cgv2-basics.md
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
        // Lesson: docs/02-cgroups/01-cgv2-basics.md
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
        // Lesson: docs/02-cgroups/01-cgv2-basics.md
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
        // Lesson: docs/02-cgroups/05-pids.md
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

        // TODO: Implement I/O limit setting
        // Lesson: docs/02-cgroups/04-io.md
        // Tests: tests/io_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/io_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Write to /sys/fs/cgroup/{path}/io.max
        // - Format: "MAJ:MIN rbps=X wbps=X riops=X wiops=X"
        // - Example: "8:0 rbps=1048576 wbps=1048576"
        // - Can use "max" for unlimited
        // - Verify io controller is enabled in subtree_control
        Command::IoMax {
            path,
            device,
            limit,
        } => {
            todo!("Implement I/O limit - write tests first! (path: {path}, device: {device}, limit: {limit})")
        }
    }

    Ok(())
}
