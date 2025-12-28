// Cgroup subcommands for the contain CLI
// These implement resource limits from fast-track lessons 05-07.

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CgroupCommand {
    /// Create a new cgroup
    /// Lesson: docs/fast-track/05-cgroup-basics.md
    Create {
        /// Cgroup path (e.g., /sys/fs/cgroup/mygroup)
        path: String,
    },

    /// Delete a cgroup
    /// Lesson: docs/fast-track/05-cgroup-basics.md
    Delete {
        /// Cgroup path to delete
        path: String,
    },

    /// Attach a process to a cgroup
    /// Lesson: docs/fast-track/05-cgroup-basics.md
    Attach {
        /// Cgroup path
        path: String,

        /// Process ID to attach
        pid: u32,
    },

    /// Set memory limit for a cgroup
    /// Lesson: docs/fast-track/06-memory-limits.md
    Memory {
        /// Cgroup path
        path: String,

        /// Memory limit (e.g., "50M", "1G")
        limit: String,
    },

    /// Set CPU limit for a cgroup
    /// Lesson: docs/fast-track/07-cpu-limits.md
    Cpu {
        /// Cgroup path
        path: String,

        /// CPU quota in microseconds per period (e.g., "50000" for 50% of one CPU)
        quota: String,
    },
}

impl CgroupCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            CgroupCommand::Create { path } => {
                // TODO: Create a new cgroup directory
                // Lesson: docs/fast-track/05-cgroup-basics.md
                // Tests: tests/cgroup_test.rs
                //
                // Implementation hints:
                // - Create directory under /sys/fs/cgroup/<path>
                // - Use std::fs::create_dir_all
                let _ = path; // Suppress unused warning
                todo!("Implement cgroup creation - see docs/fast-track/05-cgroup-basics.md")
            }
            CgroupCommand::Delete { path } => {
                // TODO: Delete a cgroup directory
                // Lesson: docs/fast-track/05-cgroup-basics.md
                // Tests: tests/cgroup_test.rs
                //
                // Implementation hints:
                // - Remove directory under /sys/fs/cgroup/<path>
                // - Cgroup must be empty (no processes)
                let _ = path; // Suppress unused warning
                todo!("Implement cgroup deletion - see docs/fast-track/05-cgroup-basics.md")
            }
            CgroupCommand::Attach { path, pid } => {
                // TODO: Attach process to cgroup
                // Lesson: docs/fast-track/05-cgroup-basics.md
                // Tests: tests/cgroup_test.rs
                //
                // Implementation hints:
                // - Write PID to /sys/fs/cgroup/<path>/cgroup.procs
                let _ = (path, pid); // Suppress unused warning
                todo!("Implement cgroup attach - see docs/fast-track/05-cgroup-basics.md")
            }
            CgroupCommand::Memory { path, limit } => {
                // TODO: Set memory limit
                // Lesson: docs/fast-track/06-memory-limits.md
                // Tests: tests/cgroup_test.rs
                //
                // Implementation hints:
                // - Parse limit (e.g., "50M" -> 52428800 bytes)
                // - Write to /sys/fs/cgroup/<path>/memory.max
                let _ = (path, limit); // Suppress unused warning
                todo!("Implement memory limit - see docs/fast-track/06-memory-limits.md")
            }
            CgroupCommand::Cpu { path, quota } => {
                // TODO: Set CPU limit
                // Lesson: docs/fast-track/07-cpu-limits.md
                // Tests: tests/cgroup_test.rs
                //
                // Implementation hints:
                // - Write "quota period" to /sys/fs/cgroup/<path>/cpu.max
                // - e.g., "50000 100000" = 50% of one CPU
                let _ = (path, quota); // Suppress unused warning
                todo!("Implement CPU limit - see docs/fast-track/07-cpu-limits.md")
            }
        }
    }
}
