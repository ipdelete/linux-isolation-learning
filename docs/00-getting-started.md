# Getting Started with Linux Isolation Learning

Welcome to the Linux Isolation Learning project! This guide will help you set up your environment and verify that you're ready to dive into learning about Linux namespaces, cgroups, and OCI containers through hands-on Rust programming.

## What You'll Learn

This course teaches Linux isolation primitives using a **Rust-first, Test-Driven Development (TDD)** approach. You'll build real, working tools while learning:

- **Namespaces**: Process, network, filesystem, and user isolation
- **Cgroups v2**: Resource limits and monitoring (CPU, memory, I/O, PIDs)
- **OCI/runc**: Container runtime standards and implementation

The unique aspect of this course: you write both tests and implementation code yourself, learning Rust syscall patterns and kernel behaviors simultaneously.

## Why Root Access Is Required

Linux namespace and cgroup operations require elevated privileges because they fundamentally alter process execution context and system resource allocation. Specifically:

- **Namespaces** isolate processes from each other, which could be used maliciously if unrestricted
- **Cgroups** control resource distribution across the system
- **Network operations** (veth pairs, bridges, iptables) modify system-wide networking state

Most lessons require `sudo` or root access. For safety, **always use a VM or disposable environment** - never run these exercises on a production system or your primary workstation.

### Important: DevContainer vs. Native Linux

**In the DevContainer**: You are running as `root` (UID 0), so commands in the lessons do **not** need `sudo` prefix. When a lesson shows:
```bash
sudo cargo run -p ns-tool -- pid /bin/true
```
In the DevContainer, you can simply run:
```bash
cargo run -p ns-tool -- pid /bin/true
```

**On a native Linux system**: You run as a regular user, so you **do** need `sudo` for privileged operations. The `sudo` prefixes in lessons are correct for native Linux environments.

The lessons are written for native Linux users (where `sudo` is needed), but they work identically in the DevContainer (just omit the `sudo`).

## Running on macOS or Windows?

**If you're on macOS or Windows**, you'll need a Linux environment. The easiest option is to use the included **DevContainer** configuration with VS Code:

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop/)
2. Install [VS Code](https://code.visualstudio.com/) with the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
3. Open this project in VS Code
4. When prompted, click "Reopen in Container" (or press F1 → "Dev Containers: Reopen in Container")
5. Wait for the container to build (first time takes 2-5 minutes)
6. **Run the validation script**:
   ```bash
   bash scripts/validate-devcontainer.sh
   ```

The devcontainer is pre-configured with:
- Debian Trixie with Linux kernel 5.15+
- All required packages (iproute2, iptables, busybox, etc.)
- Rust toolchain
- Privileged mode and necessary capabilities for namespaces/cgroups

If all validation checks pass, skip to the "Install Rust" section below.

For detailed validation steps and troubleshooting, see `.devcontainer/validation.md`.

## Prerequisites

### 1. Linux System Requirements

**Operating System**:
- Ubuntu 20.04+ (recommended for beginners)
- Fedora 35+
- Debian 11+
- Arch Linux (current)
- Any recent Linux distribution with kernel 5.0+

**Kernel Version**:
- **Minimum**: Linux 5.0 (for cgroup v2 unified hierarchy)
- **Recommended**: Linux 5.10+ (LTS kernel with full cgroup v2 support)
- **Ideal**: Linux 5.15+ or 6.1+ (latest LTS releases)

Check your kernel version:
```bash
uname -r
```

If you see something like `5.15.0-89-generic` or `6.1.0`, you're good to go. The first two numbers (5.15 or 6.1) are what matter.

**Why these versions?**
- Linux 4.5+ introduced cgroup v2, but it was experimental
- Linux 5.0+ stabilized the unified cgroup v2 hierarchy
- Older kernels (4.x) may work for namespace lessons but will struggle with cgroups

**Important namespace support**:
- PID namespaces: Linux 2.6.24+ (ancient, you definitely have this)
- Network namespaces: Linux 2.6.29+
- User namespaces: Linux 3.8+
- Time namespaces: Linux 5.6+ (optional - most lessons don't use this)

### 2. System Access

You'll need:
- **sudo access** or ability to run commands as root
- Ability to create network interfaces (for network namespace lessons)
- Ability to mount filesystems (for mount namespace lessons)

Test your sudo access:
```bash
sudo -v
```

If this prompts for your password and succeeds, you're set.

### 3. Required System Packages

Install development tools and dependencies for your distribution:

**Ubuntu/Debian**:
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    git \
    curl \
    iproute2 \
    iptables
```

**Fedora/RHEL/CentOS**:
```bash
sudo dnf install -y \
    gcc \
    git \
    curl \
    iproute \
    iptables
```

**Arch Linux**:
```bash
sudo pacman -S --needed \
    base-devel \
    git \
    curl \
    iproute2 \
    iptables
```

**Package explanations**:
- `build-essential`/`gcc`/`base-devel`: C compiler and linker (Rust needs these for native code compilation)
- `git`: Clone this repository and track your progress
- `curl`: Download Rust installer
- `iproute2`/`iproute`: Provides `ip` command for network namespace management
- `iptables`: NAT and firewall rules for network isolation lessons

**Optional packages** (install later when needed):
- `runc`: OCI runtime (needed for lessons in `docs/03-runc/`)
- `jq`: JSON manipulation for OCI bundle config editing

### 4. Rust Toolchain

**Do NOT install Rust yet** - the next section covers this in detail. You'll use `rustup` to get the latest stable Rust.

### 5. Editor/IDE Setup

Choose an editor with Rust support. Here are recommendations from most beginner-friendly to most advanced:

**VS Code** (Recommended for beginners):
- Install [VS Code](https://code.visualstudio.com/)
- Install the [rust-analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- Benefits: Excellent autocomplete, inline error checking, integrated terminal

**Other solid options**:
- **RustRover** (JetBrains): Full-featured IDE, free for learning
- **Neovim/Vim**: With rust-analyzer LSP support (for experienced vim users)
- **Emacs**: With rust-mode and LSP (for experienced Emacs users)
- **Sublime Text**: With rust-enhanced package
- **Helix**: Modern terminal editor with built-in LSP support

**What you need from your editor**:
- Syntax highlighting for Rust
- Code completion (via rust-analyzer Language Server)
- Inline error display (shows compiler errors as you type)
- Integrated terminal or easy access to a terminal

### 6. Hardware Requirements

**Minimal**:
- 2 CPU cores
- 4 GB RAM
- 10 GB free disk space

**Comfortable**:
- 4+ CPU cores
- 8 GB RAM
- 20 GB free disk space

Rust compilation is CPU and memory intensive. More resources mean faster build times, but the minimal specs will work (builds will just be slower).

## Initial Setup

### Step 1: Clone the Repository

```bash
cd ~  # or wherever you keep projects
git clone https://github.com/YOUR-USERNAME/linux-isolation-learning.git
cd linux-isolation-learning
```

Replace `YOUR-USERNAME` with the actual repository location. If you're working from a local copy, just navigate to that directory.

### Step 2: Verify Repository Structure

Check that all expected directories exist:
```bash
ls -l
```

You should see:
```
drwxr-xr-x  crates/       # Four Rust crates you'll implement
drwxr-xr-x  docs/         # Tutorial lessons
-rw-r--r--  Cargo.toml    # Workspace definition
-rw-r--r--  README.md     # Quick overview
-rw-r--r--  plan.md       # Project structure and approach
-rw-r--r--  todo.md       # Lesson completion tracker
```

### Step 3: Install Rust

Run the official rustup installer:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When prompted:
- Choose **option 1** (default installation)
- This installs the stable Rust toolchain and Cargo (the Rust build tool)

After installation completes:
```bash
source "$HOME/.cargo/env"
```

Or close and reopen your terminal.

### Step 4: Verify Rust Installation

Check that Rust is installed correctly:
```bash
rustc --version
cargo --version
```

Expected output (versions will be current stable):
```
rustc 1.75.0 (82e1608df 2023-12-21)
cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

You need Rust 1.70 or newer. If you see an older version, update with:
```bash
rustup update stable
```

### Step 5: Build the Workspace

Build all four crates to verify everything compiles:
```bash
cargo build
```

**What happens**:
1. Cargo downloads all dependencies from crates.io (this takes 2-4 minutes the first time)
2. Compiles dependencies
3. Compiles all four project crates: `ns-tool`, `netns-tool`, `cgroup-tool`, `oci-tool`
4. Creates debug binaries in `target/debug/`

**Expected output** (abbreviated):
```
   Compiling libc v0.2.150
   Compiling nix v0.27.1
   Compiling clap v4.4.11
   ...
   Compiling ns-tool v0.1.0 (/path/to/crates/ns-tool)
   Compiling netns-tool v0.1.0 (/path/to/crates/netns-tool)
   Compiling cgroup-tool v0.1.0 (/path/to/crates/cgroup-tool)
   Compiling oci-tool v0.1.0 (/path/to/crates/oci-tool)
    Finished dev [unoptimized + debuginfo] target(s) in 2m 34s
```

If you see `Finished dev`, the build succeeded!

## Quick Verification

Let's verify your setup works by running the one pre-implemented feature: the `proc` subcommand that lists your process's current namespaces.

### Run the proc Command

```bash
cargo run -p ns-tool -- proc
```

**Expected output** (your inode numbers will be different):
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

**What this shows**:
- Each line is a namespace type your process belongs to
- The number in brackets is the namespace's inode number (unique identifier)
- This reads from `/proc/self/ns/`, which is how Linux exposes namespace membership

### Verify Tests Are Set Up

Run the test suite (tests will fail with `todo!()` - this is expected):
```bash
cargo test -p ns-tool --test proc_test
```

**Expected output**:
```
running 2 tests
test test_proc_lists_namespaces ... FAILED
test test_proc_shows_inode_numbers ... FAILED

failures:

---- test_proc_lists_namespaces stdout ----
thread 'test_proc_lists_namespaces' panicked at 'not yet implemented'
```

**This is correct!** The tests are scaffolded but not implemented. You'll write real test assertions in your first lesson (`docs/00-foundations/01-rust-syscall-basics.md`).

### Check Sudo Access for Namespace Operations

Try creating a simple PID namespace (this should fail because the code isn't implemented yet, but it verifies sudo works):
```bash
sudo cargo run -p ns-tool -- pid /bin/true
```

**Expected output**:
```
thread 'main' panicked at 'not yet implemented: PID namespace creation'
```

The fact that it panicked with `not yet implemented` (instead of a permission error) means:
- sudo works correctly
- The binary can be run as root
- You're ready to implement namespace features

## Troubleshooting

### Build fails with "linker 'cc' not found"

**Cause**: No C compiler installed. Rust needs a C toolchain for linking native code.

**Fix**: Install development tools (see "Required System Packages" above).

### "cargo: command not found" after installing Rust

**Cause**: Cargo's binary directory isn't in your shell's PATH.

**Fix**:
```bash
source "$HOME/.cargo/env"
```

Or close and reopen your terminal. If this persists, check that `~/.cargo/bin` is in your PATH:
```bash
echo $PATH | grep -o ".cargo/bin"
```

### Build fails with dependency errors

**Cause**: Network issues or corrupted Cargo cache.

**Fix**:
```bash
cargo clean
rm -rf ~/.cargo/registry
cargo build
```

This removes all build artifacts and re-downloads dependencies.

### "/proc/self/ns: Permission denied"

**Cause**: Unusual filesystem permissions or SELinux policy.

**Fix**: Check that `/proc` is mounted and readable:
```bash
ls -l /proc/self/ns
```

All entries should be readable (`lrwxrwxrwx`). If you're on a system with SELinux, you may need to adjust policies or run in permissive mode (consult your distribution's documentation).

### Kernel version too old

**Cause**: Running on a kernel older than 5.0.

**Fix**: Upgrade your kernel or use a newer distribution. For learning purposes, consider using:
- Ubuntu 22.04 LTS (kernel 5.15+)
- Fedora 38+ (kernel 6.x)
- A cloud VM with a recent kernel

### Running on macOS or Windows

**This course requires Linux**. macOS and Windows have different kernel architectures and don't support Linux namespaces or cgroups.

**Options**:
1. **VM**: Use VirtualBox, VMware, or Parallels to run Ubuntu or Fedora
2. **WSL2** (Windows only): Works for most lessons, but some network operations may behave differently
3. **Cloud VM**: Spin up a Linux instance on AWS EC2, GCP, Azure, or DigitalOcean

**Note on WSL2**: While WSL2 runs a real Linux kernel, its integration with Windows can cause subtle differences in network namespace behavior. If you encounter issues, try the same exercise on a native Linux system or VM.

## Understanding the TDD Workflow

Starting with your first lesson, you'll follow this cycle:

### Red Phase (Write Failing Tests)
1. Open the test file (e.g., `crates/ns-tool/tests/feature_test.rs`)
2. Find the TODO markers
3. Write test code that describes the expected behavior
4. Run tests: `cargo test -p ns-tool --test feature_test`
5. See tests fail (RED) because implementation doesn't exist yet

### Green Phase (Implement Code)
1. Open the implementation file (e.g., `crates/ns-tool/src/main.rs`)
2. Find the TODO marker for the feature
3. Write just enough code to make the tests pass
4. Run tests again: `cargo test -p ns-tool --test feature_test`
5. See tests pass (GREEN)

### Refactor Phase (Optional)
1. Clean up code if needed
2. Re-run tests to ensure nothing broke
3. Move to the next lesson

**Why TDD?**
- You understand requirements before writing code (the tests document what you're building)
- You know immediately when you're done (tests pass)
- You build confidence that your code works correctly
- You learn to write testable Rust code from the start

## Next Steps

You're now ready to start learning! Follow the lessons in order:

### Phase 1: Foundations (Required)
Start here: **`docs/00-foundations/00-setup-rust.md`**

This lesson expands on the Rust setup you just completed and introduces the TDD workflow with your first real implementation.

Then continue through:
1. `01-rust-syscall-basics.md` - Your first syscall and test
2. `02-cli-patterns.md` - Structuring Rust CLI tools
3. `03-procfs-intro.md` - Reading `/proc` filesystem
4. `04-permissions-and-sudo.md` - Understanding privilege requirements
5. `05-error-handling.md` - Idiomatic Rust error patterns
6. `06-unsafe-boundaries.md` - When and how to use `unsafe`

### Phase 2: Namespaces
After foundations, dive into isolation:
- `docs/01-namespaces/01-pid-namespace.md` - Process isolation
- `docs/01-namespaces/02-unshare-vs-clone.md` - Different namespace creation methods
- Continue through network namespaces, mount namespaces, and more

### Phase 3: Cgroups
Learn resource control:
- `docs/02-cgroups/01-cgv2-basics.md` - Cgroup v2 fundamentals
- Memory, CPU, I/O, and PID limits
- Combining multiple resource constraints

### Phase 4: OCI/runc
Build container runtime knowledge:
- `docs/03-runc/01-oci-bundle.md` - OCI bundle structure
- Container lifecycle management
- Integration with namespaces and cgroups

## Getting Help

If you get stuck:

1. **Check the lesson's "Common Errors" section** - Most issues are covered there
2. **Read the Rust compiler errors carefully** - Rust's error messages are exceptionally helpful
3. **Consult the appendix** - `docs/90-appendix/02-troubleshooting.md` has solutions to frequent problems
4. **Look at man pages** - Run `man unshare`, `man namespaces`, `man cgroups` for authoritative documentation
5. **Search kernel documentation** - https://www.kernel.org/doc/html/latest/

## Learning Tips

**Go slow**: Each lesson is designed to be completed in 10-50 minutes. Don't rush. Understanding the concepts deeply is more valuable than speeding through.

**Experiment**: After completing a lesson, try variations. What happens if you change parameters? Can you break it? How does the kernel respond?

**Read error messages**: Rust's compiler and the Linux kernel both provide detailed errors. Train yourself to read them carefully instead of immediately searching for solutions.

**Use the REPL**: For quick Rust experiments, try `cargo install evcxr_repl` to get an interactive Rust environment.

**Keep notes**: Document your discoveries, especially when something surprises you. These notes will be valuable when you work on real projects.

## Safety Reminders

- **Always use a VM or disposable environment** for these exercises
- **Never run these exercises on production systems** or systems with important data
- **Be careful with sudo** - you're modifying system state
- **Clean up after each lesson** - Remove namespaces, cgroups, and network interfaces you create
- **Understand before running** - Read the lesson thoroughly before executing commands

## About This Course

This course teaches you to build the primitives that container runtimes like Docker, containerd, and Podman use internally. By the end, you'll understand:

- How containers achieve process isolation (namespaces)
- How containers enforce resource limits (cgroups)
- How container images become running processes (OCI runtime spec)
- How to write systems code in Rust safely and idiomatically

This is not a "how to use Docker" course - it's a "how Docker works underneath" course. You'll build real tools that interact directly with the Linux kernel.

## Ready to Begin

If you've completed all the verification steps above and seen the expected outputs, you're ready to start!

Open `docs/00-foundations/00-setup-rust.md` and begin your journey into Linux isolation and Rust systems programming.

Happy learning!
