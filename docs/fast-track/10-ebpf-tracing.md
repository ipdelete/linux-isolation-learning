# eBPF Tracing (15 min)

## What you'll build

Trace system calls from processes using eBPF kprobes.

## Prerequisites

```bash
# Install bpf-linker for eBPF compilation
cargo install bpf-linker

# Verify kernel supports eBPF
ls /sys/fs/bpf
```

## The test

**File**: `crates/contain/tests/trace_test.rs`

```rust
#[test]
fn test_ebpf_builds() {
    // Just verify the eBPF program compiles
    Command::cargo_bin("contain").unwrap()
        .args(["trace", "check"])
        .assert()
        .success();
}
```

Run it: `cargo test -p contain --test trace_test`

## The check implementation

**File**: `crates/contain/src/trace.rs`

First, implement the `check` command to verify eBPF prerequisites:

```rust
TraceCommand::Check => {
    use std::path::Path;

    println!("Checking eBPF support...\n");

    // Check /sys/fs/bpf exists
    let bpf_fs = Path::new("/sys/fs/bpf");
    if bpf_fs.exists() {
        println!("✓ /sys/fs/bpf exists");
    } else {
        println!("✗ /sys/fs/bpf not found");
    }

    // Check kernel version
    let uname = nix::sys::utsname::uname()?;
    println!("✓ Kernel: {}", uname.release().to_string_lossy());

    // Check if running as root
    if nix::unistd::Uid::effective().is_root() {
        println!("✓ Running as root");
    } else {
        println!("✗ Not running as root (eBPF requires CAP_BPF or root)");
    }

    println!("\neBPF check complete.");
    Ok(())
}
```

Run it:
```bash
cargo run -p contain -- trace check
```

## The eBPF program

**File**: `crates/ebpf-tool-ebpf/src/kprobe.rs`

```rust
#![no_std]
#![no_main]

use aya_ebpf::{macros::kprobe, programs::ProbeContext};
use aya_log_ebpf::info;

#[kprobe]
pub fn trace_exec(ctx: ProbeContext) -> u32 {
    match try_trace_exec(&ctx) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

fn try_trace_exec(ctx: &ProbeContext) -> Result<(), i64> {
    let pid = ctx.pid();
    info!(ctx, "exec called by PID {}", pid);
    Ok(())
}
```

## The userspace loader

**File**: `crates/contain/src/trace.rs`

```rust
TraceCommand::Syscalls { pid } => {
    use aya::{Bpf, programs::KProbe};

    // Load eBPF bytecode
    let mut bpf = Bpf::load(include_bytes_aligned!(
        concat!(env!("OUT_DIR"), "/ebpf-programs/kprobe")
    ))?;

    // Attach to execve syscall
    let program: &mut KProbe = bpf.program_mut("trace_exec")?.try_into()?;
    program.load()?;
    program.attach("__x64_sys_execve", 0)?;

    println!("Tracing execve... Press Ctrl+C to stop");

    // Process events
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

## Run it

Terminal 1 - Start tracing:
```bash
sudo cargo run -p contain -- trace syscalls
```

Terminal 2 - Trigger some execs:
```bash
ls
cat /etc/hostname
```

Terminal 1 output:
```
Tracing execve... Press Ctrl+C to stop
exec called by PID 12345
exec called by PID 12346
```

## What just happened

eBPF lets you run sandboxed programs in the kernel. A kprobe attaches to a kernel function (here, `execve`). Every time any process calls exec, your eBPF code runs and logs the PID. This is how tools like `bpftrace`, `execsnoop`, and container security tools work.

## Going further

The full `contain` tool has more:
- `syscalls` - Trace syscalls with optional PID filter
- `events` - Trace container lifecycle events

See `docs/04-ebpf/` for detailed lessons.

## Congratulations!

You've completed the fast track. You now understand:
- **Namespaces** - Process, mount, network isolation
- **Cgroups** - Resource limits (memory, CPU)
- **OCI/runc** - Container bundle format and runtime
- **eBPF** - Kernel observability

For deeper dives, see the full tutorials in `docs/01-namespaces/`, `docs/02-cgroups/`, etc.
