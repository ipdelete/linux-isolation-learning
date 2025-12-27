# 02 CLI Patterns

## Goal
- Use subcommands to map tools to lessons.

## Prereqs
- You can run `cargo run -q -p ns-tool -- --help`.

## Build
1) Review subcommands in:
   - `crates/ns-tool/src/main.rs`
   - `crates/netns-tool/src/main.rs`
   - `crates/cgroup-tool/src/main.rs`
   - `crates/oci-tool/src/main.rs`
2) Add a new subcommand only when a lesson needs it.

Example pattern (shortened):
```rust
#[derive(Subcommand)]
enum Command {
    Proc,
}

match cli.command {
    Command::Proc => print_proc_ns()?,
}
```

## Verify
```bash
cargo run -q -p netns-tool -- --help
```

## Notes
- Each lesson adds a single subcommand or a single flag.
