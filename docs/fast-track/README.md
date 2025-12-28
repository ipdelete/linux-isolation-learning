# Fast Track: Container Internals in 2 Hours

Learn Linux container primitives hands-on. 10 lessons, ~10 minutes each.

## Prerequisites

- Rust installed (`rustup`)
- Linux environment (see options below)
- Basic command line familiarity

## Environment Options

| Environment | Lessons Supported | Setup Effort |
|-------------|-------------------|--------------|
| **DevContainer** | 01-05, 08-10 | Low (just open in VS Code) |
| **Linux VM** | All (01-10) | Medium (one-time VM setup) |
| **Native Linux** | All (01-10) | None if you have it |

**Why the difference?** Lessons 06-07 (Memory/CPU Limits) require writing to cgroup control files, which Docker restricts. The DevContainer works for everything else.

See [.devcontainer/validation.md](../../.devcontainer/validation.md) for VM setup instructions.

## The Tool

All lessons use a single CLI tool: `contain`

```bash
cargo build -p contain
```

Subcommands:
- `contain ns` — Namespace operations (pid, mount, container)
- `contain net` — Network namespace (create, delete, veth)
- `contain cgroup` — Resource limits (create, attach, memory, cpu)
- `contain oci` — OCI bundle helpers
- `contain trace` — eBPF tracing

## Lessons

| # | Topic | Time | What You Build | Env |
|---|-------|------|----------------|-----|
| 01 | [PID Namespace](01-pid-namespace.md) | 10 min | Process becomes PID 1 | DC ✓ |
| 02 | [Mount Namespace](02-mount-namespace.md) | 10 min | Isolated filesystem mounts | DC ✓ |
| 03 | [Network Namespace](03-network-namespace.md) | 10 min | Isolated network + veth | DC ✓ |
| 04 | [Combine Namespaces](04-combine.md) | 15 min | Mini-container | DC ✓ |
| 05 | [Cgroup Basics](05-cgroup-basics.md) | 10 min | Create/attach cgroups | DC ✓ |
| 06 | [Memory Limits](06-memory-limits.md) | 10 min | Limit memory usage | **VM** |
| 07 | [CPU Limits](07-cpu-limits.md) | 10 min | Limit CPU usage | **VM** |
| 08 | [OCI Bundle](08-oci-bundle.md) | 10 min | Container bundle structure | DC ✓ |
| 09 | [Run with runc](09-runc-run.md) | 10 min | Execute with runc | DC ✓ |
| 10 | [eBPF Tracing](10-ebpf-tracing.md) | 15 min | Trace system calls | DC ✓ |

*DC = DevContainer works, VM = Linux VM required*

## Format

Each lesson follows code-first TDD:

1. **What you'll build** - One sentence
2. **The test** - Write it first
3. **The implementation** - Make it pass
4. **Run it** - See it work
5. **What just happened** - Brief explanation

## Want More Depth?

Each lesson links to detailed tutorials in the main docs:
- `docs/01-namespaces/` - Full namespace coverage
- `docs/02-cgroups/` - All cgroup controllers
- `docs/03-runc/` - OCI and runc deep dive
- `docs/04-ebpf/` - Complete eBPF tutorial

Start here: [01-pid-namespace.md](01-pid-namespace.md)
