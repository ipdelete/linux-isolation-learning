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
        // TODO: Implement OCI bundle initialization
        // Lesson: docs/03-runc/01-oci-bundle.md
        // Tests: tests/init_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/init_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Create bundle directory structure:
        //   {bundle}/
        //   ├── config.json
        //   └── rootfs/
        // - Generate minimal valid config.json following OCI runtime spec
        // - Required fields:
        //   - ociVersion: "1.0.0" (or latest)
        //   - root.path: "rootfs"
        //   - process.terminal, process.cwd, process.args
        // - Use serde_json to create the JSON structure
        // - See https://github.com/opencontainers/runtime-spec for full spec
        Command::Init { bundle } => {
            todo!("Implement OCI bundle initialization - write tests first! (bundle: {bundle})")
        }

        // TODO: Implement config.json display
        // Lesson: docs/03-runc/01-oci-bundle.md
        // Tests: tests/show_test.rs
        //
        // TDD Steps:
        // 1. Write tests in tests/show_test.rs (RED)
        // 2. Implement this function (GREEN)
        // 3. Refactor as needed
        //
        // Implementation hints:
        // - Read {bundle}/config.json
        // - Parse as JSON to validate
        // - Pretty-print to stdout using serde_json::to_string_pretty()
        // - Handle errors gracefully (bundle missing, config.json missing, invalid JSON)
        Command::Show { bundle } => {
            todo!("Implement config.json display - write tests first! (bundle: {bundle})")
        }
    }

    Ok(())
}
