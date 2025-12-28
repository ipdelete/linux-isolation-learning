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

**File**: `crates/ebpf-tool/tests/kprobe_test.rs`

```rust
#[test]
fn test_ebpf_builds() {
    // Just verify the eBPF program compiles
    Command::cargo_bin("ebpf-tool").unwrap()
        .args(["check"])
        .assert()
        .success();
}
```

Run it: `cargo test -p ebpf-tool --test kprobe_test`

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

**File**: `crates/ebpf-tool/src/main.rs`

```rust
Command::Trace => {
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
sudo cargo run -p ebpf-tool -- trace
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

The full `ebpf-tool` has more:
- `tracepoint` - Trace syscalls via tracepoints
- `uprobe` - Trace userspace functions
- `perf` - Performance sampling

See `docs/04-ebpf/` for detailed lessons.

## Congratulations!

You've completed the fast track. You now understand:
- **Namespaces** - Process, mount, network isolation
- **Cgroups** - Resource limits (memory, CPU)
- **OCI/runc** - Container bundle format and runtime
- **eBPF** - Kernel observability

For deeper dives, see the full tutorials in `docs/01-namespaces/`, `docs/02-cgroups/`, etc.
