// OCI bundle subcommands for the contain CLI
// These implement OCI container format from fast-track lessons 08-09.

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum OciCommand {
    /// Initialize an OCI bundle directory
    /// Lesson: docs/fast-track/08-oci-bundle.md
    Init {
        /// Path to create the OCI bundle
        path: String,
    },

    /// Run a container from an OCI bundle (using runc)
    /// Lesson: docs/fast-track/09-runc-run.md
    Run {
        /// Path to the OCI bundle
        path: String,

        /// Container ID
        #[arg(long, default_value = "mycontainer")]
        id: String,
    },
}

impl OciCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            OciCommand::Init { path } => {
                // TODO: Initialize OCI bundle structure
                // Lesson: docs/fast-track/08-oci-bundle.md
                // Tests: tests/oci_test.rs
                //
                // Implementation hints:
                // - Create <path>/rootfs directory
                // - Generate config.json with OCI spec
                // - Minimal config: process, root, linux namespaces
                let _ = path; // Suppress unused warning
                todo!("Implement OCI bundle init - see docs/fast-track/08-oci-bundle.md")
            }
            OciCommand::Run { path, id } => {
                // TODO: Run container using runc
                // Lesson: docs/fast-track/09-runc-run.md
                // Tests: tests/oci_test.rs
                //
                // Implementation hints:
                // - Invoke `runc run` with bundle path
                // - Handle container lifecycle
                let _ = (path, id); // Suppress unused warning
                todo!("Implement OCI run - see docs/fast-track/09-runc-run.md")
            }
        }
    }
}
