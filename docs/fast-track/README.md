# Fast Track: Container Internals in 2 Hours

Learn Linux container primitives hands-on. 10 lessons, ~10 minutes each.

## Prerequisites

- Rust installed (`rustup`)
- Linux with root access (or DevContainer)
- Basic command line familiarity

## Lessons

| # | Topic | Time | What You Build |
|---|-------|------|----------------|
| 01 | [PID Namespace](01-pid-namespace.md) | 10 min | Process becomes PID 1 |
| 02 | [Mount Namespace](02-mount-namespace.md) | 10 min | Isolated filesystem mounts |
| 03 | [Network Namespace](03-network-namespace.md) | 10 min | Isolated network + veth |
| 04 | [Combine Namespaces](04-combine.md) | 15 min | Mini-container |
| 05 | [Cgroup Basics](05-cgroup-basics.md) | 10 min | Create/attach cgroups |
| 06 | [Memory Limits](06-memory-limits.md) | 10 min | Limit memory usage |
| 07 | [CPU Limits](07-cpu-limits.md) | 10 min | Limit CPU usage |
| 08 | [OCI Bundle](08-oci-bundle.md) | 10 min | Container bundle structure |
| 09 | [Run with runc](09-runc-run.md) | 10 min | Execute with runc |
| 10 | [eBPF Tracing](10-ebpf-tracing.md) | 15 min | Trace system calls |

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
