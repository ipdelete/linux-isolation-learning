# 02 CLI Patterns with Clap

## Goal
Learn how to build well-structured command-line tools using Rust's `clap` crate with derive macros. You will understand subcommand patterns, argument types, help text generation, and error handling. Understand the CLI architecture used throughout this course without reimplementing tests.

**Estimated time**: 30-40 minutes

## Prereqs
- Completed `00-setup-rust.md` (workspace builds successfully)
- Completed `01-rust-syscall-basics.md` (understand how `print_proc_ns()` works)
- Familiarity with Rust structs and enums

## Why CLI Patterns Matter

Every tool you build in this course follows the same pattern: a single binary with multiple subcommands. Think of tools like `git` (`git clone`, `git commit`, `git push`) or `cargo` (`cargo build`, `cargo test`, `cargo run`). This pattern:

1. **Keeps related functionality together** - All namespace operations live in `ns-tool`
2. **Maps directly to lessons** - Each lesson adds one subcommand
3. **Makes testing straightforward** - Test each subcommand in isolation
4. **Scales cleanly** - Add new subcommands without touching existing code

The `clap` crate with derive macros lets you define this structure declaratively, turning Rust types into CLI parsers automatically.

## Understanding the Clap Derive Pattern

Open `crates/ns-tool/src/main.rs` and examine the structure:

```rust
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
```

**Key concepts**:

1. **`#[derive(Parser)]`** - Generates argument parsing code for the struct
2. **`#[command(...)]`** - Configures the command (name, description, version)
3. **`#[derive(Subcommand)]`** - Makes the enum represent subcommands
4. **Enum variants become subcommands** - `Command::Proc` becomes `ns-tool proc`

The `main()` function then matches on the parsed command:

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Proc => print_proc_ns()?,
        Command::Pid => todo!("..."),
        // ...other subcommands
    }

    Ok(())
}
```

This is the pattern you will use for every tool in this course.

## Common Argument Types

While `ns-tool` currently uses simple subcommands without arguments, you will need these patterns in later lessons:

### Flags (boolean options)

```rust
#[derive(Parser)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}
```

Usage: `ns-tool --verbose` or `ns-tool -v`

### Options (key-value arguments)

```rust
#[derive(Parser)]
struct Cli {
    /// Hostname to set in the namespace
    #[arg(short, long)]
    hostname: Option<String>,
}
```

Usage: `ns-tool uts --hostname myhost` or `ns-tool uts -h myhost`

### Positional arguments

```rust
#[derive(Subcommand)]
enum Command {
    /// Join an existing namespace
    Setns {
        /// PID of the process whose namespace to join
        pid: u32,
    },
}
```

Usage: `ns-tool setns 1234`

### Required vs optional

```rust
#[derive(Subcommand)]
enum Command {
    Create {
        /// Name for the new namespace (required)
        name: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
}
```

## Explore Clap's Help and Error Handling

**No test implementation for this lesson.** Instead, we'll explore how clap generates help text and handles errors.

The `proc` subcommand is already implemented. Your task is to understand how clap's derive macros generate CLI behavior without writing additional tests.

### What You'll Explore

1. **Help text generation**: How doc comments become CLI help
2. **Error handling**: How clap validates arguments and reports errors
3. **Subcommand dispatch**: How the enum structure maps to commands

### Steps

1. Open `crates/ns-tool/src/main.rs` and examine the `Command` enum:

```rust
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
```

2. Notice that each variant can have doc comments. Add doc comments to understand how clap generates help:

```rust
#[derive(Subcommand)]
enum Command {
    /// Create a new PID namespace and fork a child process
    Pid,
    /// Create a new UTS namespace and set hostname
    Uts,
    // ... etc
}
```

3. Test clap's help generation:

```bash
cargo run -p ns-tool -- --help
```

You should see descriptions for each subcommand.

4. Test clap's error handling by passing invalid arguments:

```bash
cargo run -p ns-tool -- invalid-command
cargo run -p ns-tool -- pid extra-arg
```

Observe how clap provides helpful error messages.

## Build (Green)

Since `proc` is already implemented, let us examine how it works and understand the pattern for future subcommands.

**Implementation file**: `crates/ns-tool/src/main.rs`
**Function location**: `print_proc_ns()` at the end of the file

### Understanding the implementation

```rust
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
```

**Key patterns to notice**:

1. **Returns `Result<()>`** - All subcommand handlers should return `Result` for proper error handling
2. **Uses `?` for error propagation** - Clean error handling without explicit match statements
3. **Focused on one task** - Reads `/proc/self/ns` and prints namespace info
4. **No side effects** - Just reads and prints, does not modify system state

### The dispatch pattern

In `main()`, the match arm simply calls the handler:

```rust
Command::Proc => print_proc_ns()?,
```

The `?` propagates any error up to `main()`, which returns it to the caller.

## Verify Your Understanding

**Test clap's behavior**:

1. Run the help command:
```bash
cargo run -p ns-tool -- --help
```

2. Test error handling:
```bash
cargo run -p ns-tool -- invalid-command
cargo run -p ns-tool -- proc extra-arg
```

3. Examine the error messages - clap generates them automatically.

## Exercise: Add Subcommand Documentation

Enhance clap's help text by adding doc comments to the `Command` enum:

1. Open `crates/ns-tool/src/main.rs`

2. Find the `Command` enum and add doc comments:

```rust
#[derive(Subcommand)]
enum Command {
    /// Create a new PID namespace and fork a child process
    Pid,
    /// Create a new UTS namespace and set hostname
    Uts,
    /// Create a new IPC namespace
    Ipc,
    /// Create a new mount namespace
    Mount,
    /// Create a new network namespace
    Net,
    /// Create a new user namespace
    User,
    /// Create a new cgroup namespace
    Cgroup,
    /// Create a new time namespace
    Time,
    /// Join an existing namespace
    Setns,
    /// Display current namespace information from /proc
    Proc,
}
```

3. Rebuild and test:
```bash
cargo run -p ns-tool -- --help
```

Notice how clap now displays your doc comments as descriptions in the help output.

## Structuring for Testability

The current structure makes testing easy:

1. **Integration tests** use `assert_cmd` to run the binary as a subprocess
2. **Logic is in separate functions** (`print_proc_ns()`) not inline in `main()`
3. **Each subcommand has its own test file** (`tests/pid_test.rs`, `tests/proc_test.rs`)

As you add more complex subcommands, consider:

- **Extract reusable logic into library code** - Create `src/lib.rs` for shared functions
- **Unit test internal functions** - Use `#[cfg(test)]` modules for unit tests
- **Keep handlers thin** - Match arm calls a function, function contains logic

## Clean Up

This lesson does not create any persistent resources. No cleanup needed.

## Common Errors

1. **`error: found argument ... which wasn't expected`**
   - Cause: Clap detected an unexpected argument
   - This is clap's error handling working correctly - it validates arguments
   - Fix: Only pass valid subcommands and arguments

2. **Help text shows no descriptions for subcommands**
   - Cause: The `Command` enum doesn't have doc comments
   - Fix: Add `/// Description` comments above each variant

## Summary

You learned:

- **Clap derive pattern**: `#[derive(Parser)]` and `#[derive(Subcommand)]` generate CLI parsers from types
- **Subcommand structure**: Enum variants become subcommands, match arms dispatch to handlers
- **Argument types**: Flags (`bool`), options (`Option<T>`), and positional arguments
- **Help text generation**: Doc comments (`/// ...`) automatically become help descriptions
- **Error handling**: Clap validates arguments and provides helpful error messages automatically

## Looking Ahead

In the next foundation lesson (03-procfs-intro), you will:

1. Write tests for the existing `proc` subcommand
2. Learn how to test CLI tools with `assert_cmd` and `predicates`
3. Understand the `/proc` filesystem structure

Then, in later lessons, you'll add new subcommands (PID, UTS, IPC, etc.) and write tests first (red), then implement (green).

## Notes

- **Clap version**: This course uses clap 4.x with derive macros. The derive API is stable and recommended for new projects.
- **Error handling**: We use `anyhow::Result` for flexible error handling. Later lessons cover this in depth (`05-error-handling.md`).
- **Binary name**: The binary name comes from `[package] name` in `Cargo.toml`, not the source file name.

## Next
`03-procfs-intro.md` - Learn to read `/proc` filesystem for process and namespace information
