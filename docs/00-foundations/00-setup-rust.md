# 00 Setup Rust Environment

## Goal
Install Rust and verify you can build and test the workspace. This lesson also introduces the Test-Driven Development (TDD) workflow you'll use throughout the course.

## Prereqs
- Linux host, VM, or DevContainer (Ubuntu 20.04+, Fedora 35+, Debian Trixie, or similar recommended)
- Many later lessons require elevated privileges for namespace/cgroup operations
  - **In DevContainer**: Commands run as root, so no `sudo` needed
  - **On native Linux**: You'll need `sudo` or root access
- Basic CLI tools: `git`, `curl`
- This repository cloned locally

## About This Course's TDD Workflow
Starting with the next lesson, you'll follow a **Red → Green → Refactor** cycle:

1. **Red**: Write a failing test first (in `crates/*/tests/`)
2. **Green**: Implement just enough code to make the test pass (in `crates/*/src/`)
3. **Refactor**: Clean up if needed (optional)

Each lesson provides:
- Test files with TODO markers for you to implement
- Implementation files with TODO markers and hints
- Clear verification steps (automated tests + manual inspection)

This lesson is setup-only (no TDD yet), but you'll write your first test in the next lesson.

## Build
1) Install Rust with rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts. Choose default options unless you have specific needs.

2) Restart your shell or source the cargo environment:
```bash
source "$HOME/.cargo/env"
```

3) Verify the toolchain is installed:
```bash
rustc --version   # Should show 1.70+ (or current stable)
cargo --version   # Should match rustc version
```

4) Build the entire workspace (as your normal user, not root):
```bash
cargo build
```

This will:
- Download and compile all dependencies (~2-3 minutes on first run)
- Build all workspace crates: `ns-tool`, `netns-tool`, `cgroup-tool`, `oci-tool`, `ebpf-tool`, and `ebpf-tool-common`
- Create binaries in `target/debug/`

## Verify
**Test the build** by running the only implemented subcommand (`proc`):
```bash
cargo run -p ns-tool -- proc
```

Expected output (your inode numbers will differ):
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

This shows your process's current namespace memberships by reading `/proc/self/ns`. Each namespace type has a unique inode number (the number in brackets).

**Verify tests work** (they should panic with `todo!()` because you haven't written them yet):
```bash
cargo test -p ns-tool --test proc_test
```

Expected output:
```
running 2 tests
test test_proc_lists_namespaces ... FAILED
test test_proc_shows_inode_numbers ... FAILED
```

Both tests fail with `not yet implemented` - this is correct! In the next lesson, you'll implement these tests.

## Common Errors
1. **`linker 'cc' not found` during `cargo build`**
   - Cause: No C compiler/linker installed (Rust needs it for linking)
   - Fix (Ubuntu/Debian): `sudo apt-get install build-essential`
   - Fix (Fedora/RHEL): `sudo dnf install gcc`
   - Fix (Arch): `sudo pacman -S base-devel`

2. **`cargo: command not found` after installing rustup**
   - Cause: Cargo's bin directory not in PATH (shell not restarted)
   - Fix: Close and reopen your terminal, or run `source "$HOME/.cargo/env"`

3. **`permission denied` when running `cargo run -p ns-tool -- proc`**
   - Cause: This specific subcommand (`proc`) doesn't need sudo, but something else is wrong
   - Fix: Check that `/proc/self/ns` is readable: `ls -l /proc/self/ns`

4. **Compilation errors about missing dependencies**
   - Cause: Network issues or stale Cargo cache
   - Fix: Remove build artifacts and retry: `cargo clean && cargo build`

## Notes
**Build pattern for later lessons**:
- Always build as your normal user: `cargo build -p <crate>`
- Run with `sudo` only when needed (namespace/cgroup operations): `sudo ./target/debug/<crate> <subcommand>`
- Alternatively, use `cargo run` with sudo: `sudo $(which cargo) run -p <crate> -- <subcommand>` (but this is slower)

**Additional tools** (install when needed in later lessons):
- `iproute2`: Provides `ip` command for network namespaces (Lesson 06+)
- `iptables` or `nftables`: For NAT setup (Lesson 08)
- `runc`: For OCI container runtime (Lesson 03-runc)

Most distributions include `iproute2` by default. We'll note when you need other tools.

## Next
`01-rust-syscall-basics.md` - Learn to call Linux syscalls from Rust and write your first test
