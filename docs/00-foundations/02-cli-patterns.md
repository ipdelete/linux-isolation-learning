# 02 CLI Patterns with Clap

## Goal
Learn how to build well-structured command-line tools using Rust's `clap` crate with derive macros. You will understand subcommand patterns, argument types, and how to structure your CLI for testability. By the end, you will have implemented tests for the existing `proc` subcommand and understand the CLI architecture used throughout this course.

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

## Write Tests (Red)

**Test file**: `crates/ns-tool/tests/proc_test.rs`

The `proc` subcommand is already implemented. Your task is to write tests that verify its behavior. This teaches you the testing pattern you will use for every subcommand.

### What the tests should verify

- **Success case**: The command runs successfully and outputs namespace information
- **Format case**: Output includes namespace types with inode numbers in the expected format

### Steps

1. Open `crates/ns-tool/tests/proc_test.rs`

2. Find the first TODO in `test_proc_lists_namespaces`. Replace the `todo!()` with:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_proc_lists_namespaces() {
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("proc")
        .assert()
        .success()
        .stdout(predicate::str::contains("pid"))
        .stdout(predicate::str::contains("net"))
        .stdout(predicate::str::contains("mnt"))
        .stdout(predicate::str::contains("uts"))
        .stdout(predicate::str::contains("ipc"));
}
```

3. Find the second TODO in `test_proc_shows_inode_numbers`. Replace the `todo!()` with:

```rust
#[test]
fn test_proc_shows_inode_numbers() {
    let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    cmd.arg("proc")
        .assert()
        .success()
        // Verify the format: namespace -> namespace:[inode]
        // The inode is a number in brackets
        .stdout(predicate::str::is_match(r"pid.*\[\d+\]").unwrap());
}
```

4. Run the tests (expect success since `proc` is already implemented):

```bash
cargo test -p ns-tool --test proc_test
```

**Expected output**:
```
running 2 tests
test test_proc_lists_namespaces ... ok
test test_proc_shows_inode_numbers ... ok

test result: ok. 2 passed; 0 failed
```

Wait - the tests pass immediately! This is unusual for TDD, but intentional here: the `proc` subcommand is your **reference implementation**. You are learning the testing pattern by verifying working code. In later lessons, you will write tests first for unimplemented subcommands (true red-green cycle).

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

## Verify

**Automated verification**:

```bash
cargo test -p ns-tool --test proc_test
```

All tests should pass.

**Full test suite** (runs all ns-tool tests):

```bash
cargo test -p ns-tool
```

Some tests will fail with `todo!()` - this is expected. Only `proc_test` should pass.

**Manual verification**:

```bash
cargo run -p ns-tool -- proc
```

You should see output like:
```
cgroup -> cgroup:[4026531835]
ipc -> ipc:[4026531839]
mnt -> mnt:[4026531841]
net -> net:[4026531840]
pid -> pid:[4026531836]
pid_for_children -> pid:[4026531836]
time -> time:[4026532448]
time_for_children -> time:[4026532448]
user -> user:[4026531837]
uts -> uts:[4026531838]
```

**Verify help text**:

```bash
cargo run -p ns-tool -- --help
```

You should see all subcommands listed:
```
Namespace learning tool (Rust-first rewrite)

Usage: ns-tool <COMMAND>

Commands:
  pid
  uts
  ipc
  mount
  net
  user
  cgroup
  time
  setns
  proc
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Adding Subcommand Documentation

Notice the help output above shows empty descriptions for subcommands. Clap uses doc comments to generate help text. Update the enum to add descriptions:

**Exercise** (optional): Add doc comments to the `Command` enum in `main.rs`:

```rust
#[derive(Subcommand)]
enum Command {
    /// Create a new PID namespace and fork a child process
    Pid,
    /// Create a new UTS namespace and set hostname
    Uts,
    /// Create a new IPC namespace
    Ipc,
    // ...etc
}
```

After adding doc comments, `--help` will show:
```
Commands:
  pid     Create a new PID namespace and fork a child process
  uts     Create a new UTS namespace and set hostname
  ...
```

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

1. **`error[E0433]: failed to resolve: use of undeclared crate or module 'predicates'`**
   - Cause: Missing import in test file
   - Fix: Add `use predicates::prelude::*;` at the top of the test file

2. **`error: no bin target named 'ns-tool'`**
   - Cause: Running tests from wrong directory or typo in crate name
   - Fix: Run from workspace root, verify crate name matches `Cargo.toml`

3. **Test passes but you expected it to fail**
   - Cause: The `proc` subcommand is already implemented (this is the reference example)
   - This is intentional for this lesson. Future lessons will have true red-green cycles.

4. **`regex parse error` in `is_match()`**
   - Cause: Invalid regex syntax in the predicate
   - Fix: Escape special characters properly. Use raw strings `r"..."` for regex patterns.

## Summary

You learned:

- **Clap derive pattern**: `#[derive(Parser)]` and `#[derive(Subcommand)]` generate CLI parsers from types
- **Subcommand structure**: Enum variants become subcommands, match arms dispatch to handlers
- **Argument types**: Flags (`bool`), options (`Option<T>`), and positional arguments
- **Testing with assert_cmd**: Run the binary as a subprocess, check exit status and output
- **Predicates**: Use `predicate::str::contains()` and `predicate::str::is_match()` for output assertions

## Looking Ahead

In the remaining foundation lessons and namespace lessons, you will:

1. Add new subcommands by adding enum variants
2. Write tests first (red), then implement the handler (green)
3. Use the same `assert_cmd` pattern for integration tests
4. Add arguments to subcommands when operations need parameters

The CLI pattern stays consistent throughout the course. Master it here, and you will focus on the interesting parts (namespaces, cgroups, syscalls) in later lessons.

## Notes

- **Clap version**: This course uses clap 4.x with derive macros. The derive API is stable and recommended for new projects.
- **Error handling**: We use `anyhow::Result` for flexible error handling. Later lessons cover this in depth (`05-error-handling.md`).
- **Binary name**: The binary name comes from `[package] name` in `Cargo.toml`, not the source file name.

## Next
`03-procfs-intro.md` - Learn to read `/proc` filesystem for process and namespace information
