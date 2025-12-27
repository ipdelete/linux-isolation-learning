# Progress Log: eBPF Tutorial Setup

**Date**: 2025-12-27
**Session**: 054855
**Branch**: `feature/04-ebpf-tutorials`
**Commit**: `a95a7d9`

## Overview

Completed the **Setup** phase for the eBPF tutorial series. This prepares the repository infrastructure for adding Aya-based eBPF tracing tutorials to the existing Linux isolation learning path.

---

## What We Built

### 1. Workspace Configuration for eBPF
Added eBPF crate support to the Cargo workspace with all required dependencies for the Aya framework.

### 2. Development Container Updates
Extended the devcontainer with LLVM toolchain, BPF linker, and kernel debug mounts required for eBPF development.

### 3. Documentation Structure
Updated README to reflect the new 5-tool structure and added the complete 04-eBPF section to the table of contents.

---

## Files Modified

| File | Changes |
|------|---------|
| `Cargo.toml:7-8` | Added `ebpf-tool` and `ebpf-tool-common` workspace members |
| `Cargo.toml:14-22` | Added aya, aya-log, tokio, log, env_logger dependencies |
| `.devcontainer/devcontainer.json:32` | Added llvm, clang, rust-src, bpf-linker to postCreateCommand |
| `.devcontainer/devcontainer.json:41-44` | Added kernel debug/BTF bind mounts |
| `README.md:7` | Changed "Four" to "Five" CLI tools |
| `README.md:12` | Added ebpf-tool description |
| `README.md:58-67` | Added 04-eBPF section with 9 lesson links |
| `backlog/todos/04_ebpf_todo.md:4-6` | Marked Setup items as complete |

---

## Key Configuration Changes

### Cargo.toml - New Dependencies
```toml
aya = "0.13"
aya-log = "0.2"
tokio = { version = "1", features = ["full"] }
log = "0.4"
env_logger = "0.11"
```

### Devcontainer - eBPF Toolchain
```json
"postCreateCommand": "... && apt-get install -y ... llvm clang && rustup component add rust-src && cargo install bpf-linker"
```

### Devcontainer - Kernel Mounts
```json
"mounts": [
    "source=/sys/kernel/debug,target=/sys/kernel/debug,type=bind",
    "source=/sys/kernel/btf,target=/sys/kernel/btf,type=bind,readonly"
]
```

---

## Architecture Notes

### Crate Structure (Planned)
```
crates/
├── ebpf-tool/           # Userspace CLI (workspace member)
├── ebpf-tool-common/    # Shared types (workspace member)
└── ebpf-tool-ebpf/      # BPF programs (NOT workspace member - different target)
```

The `ebpf-tool-ebpf` crate is intentionally excluded from the workspace because it compiles to BPF bytecode using a different compilation target (`bpfel-unknown-none`).

---

## Next Steps (Not Implemented)

Per `backlog/todos/04_ebpf_todo.md`, remaining work:

### Crate Scaffolding
- [ ] `crates/ebpf-tool-common/` - Shared types (Cargo.toml, lib.rs)
- [ ] `crates/ebpf-tool-ebpf/` - BPF programs (Cargo.toml, main.rs, probe modules)
- [ ] `crates/ebpf-tool/` - Userspace CLI (Cargo.toml, build.rs, main.rs, tests)

### Tutorial Documents
- [ ] `docs/04-ebpf/00-ebpf-setup.md` - Setup and check command
- [ ] `docs/04-ebpf/01-hello-kprobe.md` - First kprobe
- [ ] `docs/04-ebpf/02-reading-data.md` - Reading syscall arguments
- [ ] `docs/04-ebpf/03-maps.md` - eBPF maps and counters
- [ ] `docs/04-ebpf/04-perf-events.md` - User-kernel communication
- [ ] `docs/04-ebpf/05-uprobes.md` - Userspace probes
- [ ] `docs/04-ebpf/06-tracepoints.md` - Kernel tracepoints
- [ ] `docs/04-ebpf/07-perf-sampling.md` - CPU sampling
- [ ] `docs/04-ebpf/08-combining.md` - Full syscall tracer

---

## Repository Information

- **URL**: (local workspace)
- **Branch**: `feature/04-ebpf-tutorials`
- **Base Commit**: `a95a7d95f5fa09d5823eb76876f4f94f1d1c6560`
- **Status**: Setup phase complete, uncommitted changes ready for review
