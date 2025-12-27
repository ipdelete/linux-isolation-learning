# Linux Isolation Learning

A hands-on Rust learning path for Linux container primitives: namespaces, cgroups, and OCI/runc.

## What You Will Build

Five Rust CLI tools that demonstrate container building blocks:
- **ns-tool** - namespace creation and management
- **netns-tool** - network namespace setup with veth pairs
- **cgroup-tool** - cgroup v2 resource limits
- **oci-tool** - OCI bundle helpers
- **ebpf-tool** - eBPF tracing with kprobes, uprobes, and tracepoints

## Getting Started

Start with [00-setup-rust.md](docs/00-foundations/00-setup-rust.md) and follow the numbered lessons in order.

## Table of Contents

### 00 - Foundations
- [00-setup-rust.md](docs/00-foundations/00-setup-rust.md)
- [01-rust-syscall-basics.md](docs/00-foundations/01-rust-syscall-basics.md)
- [02-cli-patterns.md](docs/00-foundations/02-cli-patterns.md)
- [03-procfs-intro.md](docs/00-foundations/03-procfs-intro.md)
- [04-permissions-and-sudo.md](docs/00-foundations/04-permissions-and-sudo.md)
- [05-error-handling.md](docs/00-foundations/05-error-handling.md)
- [06-unsafe-boundaries.md](docs/00-foundations/06-unsafe-boundaries.md)

### 01 - Namespaces
- [01-pid-namespace.md](docs/01-namespaces/01-pid-namespace.md)
- [02-unshare-vs-clone.md](docs/01-namespaces/02-unshare-vs-clone.md)
- [03-uts-ipc.md](docs/01-namespaces/03-uts-ipc.md)
- [04-mount-namespace.md](docs/01-namespaces/04-mount-namespace.md)
- [05-minimal-rootfs.md](docs/01-namespaces/05-minimal-rootfs.md)
- [06-netns-basics.md](docs/01-namespaces/06-netns-basics.md)
- [07-veth-bridge.md](docs/01-namespaces/07-veth-bridge.md)
- [08-netns-nat.md](docs/01-namespaces/08-netns-nat.md)
- [09-combine-ns.md](docs/01-namespaces/09-combine-ns.md)
- [10-join-existing.md](docs/01-namespaces/10-join-existing.md)

### 02 - Cgroups
- [01-cgv2-basics.md](docs/02-cgroups/01-cgv2-basics.md)
- [02-memory.md](docs/02-cgroups/02-memory.md)
- [03-cpu.md](docs/02-cgroups/03-cpu.md)
- [04-io.md](docs/02-cgroups/04-io.md)
- [05-pids.md](docs/02-cgroups/05-pids.md)
- [06-multi-resource.md](docs/02-cgroups/06-multi-resource.md)

### 03 - runc and OCI
- [01-oci-bundle.md](docs/03-runc/01-oci-bundle.md)
- [02-config-json.md](docs/03-runc/02-config-json.md)
- [03-run-basic.md](docs/03-runc/03-run-basic.md)
- [04-lifecycle.md](docs/03-runc/04-lifecycle.md)
- [05-seccomp.md](docs/03-runc/05-seccomp.md)
- [06-network-integration.md](docs/03-runc/06-network-integration.md)
- [07-cgroups-integration.md](docs/03-runc/07-cgroups-integration.md)

### 04 - eBPF
- [00-ebpf-setup.md](docs/04-ebpf/00-ebpf-setup.md)
- [01-hello-kprobe.md](docs/04-ebpf/01-hello-kprobe.md)
- [02-reading-data.md](docs/04-ebpf/02-reading-data.md)
- [03-maps.md](docs/04-ebpf/03-maps.md)
- [04-perf-events.md](docs/04-ebpf/04-perf-events.md)
- [05-uprobes.md](docs/04-ebpf/05-uprobes.md)
- [06-tracepoints.md](docs/04-ebpf/06-tracepoints.md)
- [07-perf-sampling.md](docs/04-ebpf/07-perf-sampling.md)
- [08-combining.md](docs/04-ebpf/08-combining.md)

### 90 - Appendix
- [01-rust-syscall-cheatsheet.md](docs/90-appendix/01-rust-syscall-cheatsheet.md)
- [02-troubleshooting.md](docs/90-appendix/02-troubleshooting.md)

## Safety Note

These exercises require root privileges and modify system state. Use a VM or container.
