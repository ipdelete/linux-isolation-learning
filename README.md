# Container Fundamentals Learning Path

A comprehensive, hands-on guide to understanding Linux containers from first principles.

## Overview

This learning path takes you from basic Linux isolation primitives to running production containers with runc. By the end, you'll understand exactly how Docker, Kubernetes, and other container technologies work under the hood.

**Total estimated time**: 40-50 hours of hands-on learning

---

## Prerequisites

- Linux system (Ubuntu 20.04+, Fedora 34+, or similar)
- Root/sudo access
- Basic Linux command-line knowledge
- Basic programming knowledge (C, Python, or Bash)
- Curiosity and patience!

**System requirements:**
- Kernel 4.5+ (for cgroups v2)
- 4GB+ RAM
- 20GB+ free disk space

---

## Learning Path

### Module 1: Linux Namespaces (01-namespaces.md)
**Time**: 15-20 hours | **Difficulty**: Intermediate

Learn the 7 namespace types that provide process isolation:

- **PID namespace**: Process ID isolation
- **Network namespace**: Network stack isolation
- **Mount namespace**: Filesystem view isolation
- **UTS namespace**: Hostname isolation
- **IPC namespace**: Inter-process communication isolation
- **User namespace**: UID/GID remapping
- **Cgroup namespace**: Cgroup hierarchy isolation

**What you'll build:**
- Create isolated processes with custom PIDs
- Set up virtual networks between isolated processes
- Build a minimal container-like environment
- Join and manage existing namespaces

**Key skills:**
- Understanding `unshare()`, `clone()`, `setns()` syscalls
- Creating veth pairs and bridges
- Setting up isolated filesystems with pivot_root
- Writing namespace-aware programs in C and Python

---

### Module 2: Cgroups (Control Groups) (02-cgroups.md)
**Time**: 12-15 hours | **Difficulty**: Intermediate

Learn to limit and monitor resource usage:

- **Memory controller**: Hard/soft limits, OOM killer
- **CPU controller**: Bandwidth limiting, proportional shares
- **I/O controller**: Disk I/O limits and priorities
- **PID controller**: Process count limits
- **Pressure Stall Information**: Detecting resource contention

**What you'll build:**
- Resource-limited execution environments
- Multi-tier cgroup hierarchies
- Real-time resource monitoring dashboard
- Container resource management system

**Key skills:**
- Understanding cgroups v2 unified hierarchy
- Setting memory, CPU, and I/O limits
- Monitoring resource usage and pressure
- Implementing proportional resource sharing

---

### Module 3: runc (OCI Runtime) (03-runc.md)
**Time**: 13-15 hours | **Difficulty**: Intermediate-Advanced

Learn the reference OCI runtime that powers Docker/Kubernetes:

- **OCI specification**: Runtime, image, and distribution specs
- **Bundle format**: config.json + rootfs structure
- **Lifecycle management**: create, start, kill, delete
- **Advanced features**: Seccomp, capabilities, hooks
- **Integration**: Combining namespaces + cgroups + security

**What you'll build:**
- Run your first OCI-compliant container
- Create custom container configurations
- Build a simple container runtime
- Set up networked containers
- Implement container monitoring tools

**Key skills:**
- Understanding OCI specification
- Creating and managing OCI bundles
- Configuring seccomp filters
- Managing container lifecycle
- Debugging containers at the runtime level

---

## Suggested Learning Schedule

### Intensive (2 weeks full-time)
- **Week 1**: Namespaces (Days 1-4), Cgroups (Days 5-7)
- **Week 2**: runc (Days 8-12), Projects (Days 13-14)

### Part-time (8 weeks, 5-7 hours/week)
- **Weeks 1-3**: Namespaces
- **Weeks 4-5**: Cgroups
- **Weeks 6-8**: runc

### Self-paced (flexible)
- Complete each module at your own pace
- Focus on understanding over speed
- Build all example projects

---

## How to Use This Guide

### 1. Follow the Order
The modules build on each other:
```
Namespaces → Cgroups → runc
```
Don't skip ahead - each module assumes knowledge from previous ones.

### 2. Hands-on Practice
**Don't just read - type every command!**

- Run all examples
- Modify configurations
- Break things and fix them
- Build all projects

### 3. Experiment
After each section:
- Try variations of examples
- Ask "what if I change X?"
- Combine techniques in new ways

### 4. Take Notes
Document:
- Key concepts
- Commands you use frequently
- Problems you solved
- "Aha!" moments

### 5. Build Projects
Each module ends with practical projects. **Complete them all!**

They solidify learning and give you portfolio pieces.

---

## Learning Objectives

By the end of this path, you will:

### Understand
- How containers provide isolation without VMs
- How Docker, Podman, and Kubernetes work internally
- Linux kernel features for resource management
- OCI specifications and container standards

### Be able to
- Create isolated execution environments
- Limit and monitor resource usage
- Run OCI-compliant containers
- Debug container runtime issues
- Build your own simple container runtime

### Have built
- Namespace management tools
- Cgroup resource managers
- Container monitoring systems
- A simple container runtime
- Network isolation systems

---

## Testing Your Knowledge

### After Namespaces
Can you:
- [ ] Create a process with PID 1 in its own namespace?
- [ ] Set up networking between two isolated namespaces?
- [ ] Create a minimal container with mount namespace?
- [ ] Explain the difference between unshare() and clone()?

### After Cgroups
Can you:
- [ ] Limit a process to 100MB of memory?
- [ ] Give two processes 2:1 CPU ratio?
- [ ] Prevent fork bombs with pids.max?
- [ ] Monitor resource pressure in real-time?

### After runc
Can you:
- [ ] Run an OCI container from scratch?
- [ ] Write a valid config.json by hand?
- [ ] Debug why a container won't start?
- [ ] Integrate networking with runc containers?

---

## Troubleshooting

### Common Issues

**"Operation not permitted"**
- Most operations require root: use `sudo`
- Check if you're in the right namespace
- Verify cgroup permissions

**"Device or resource busy"**
- Processes still running in namespace/cgroup
- Use `kill -9` to force kill
- Check with `lsns` or `cat cgroup.procs`

**"No such file or directory" in cgroups**
- Enable controllers: `echo "+cpu" > cgroup.subtree_control`
- Check cgroup v2 is mounted
- Verify you're using correct cgroup version

**Container networking doesn't work**
- Check IP forwarding: `sysctl net.ipv4.ip_forward`
- Verify iptables NAT rules
- Ensure veth pair is up on both ends

### Getting Help

1. **Read error messages carefully** - they're usually helpful!
2. **Check logs**: `dmesg`, `journalctl`, container output
3. **Verify step-by-step**: Don't skip verification commands
4. **Start simple**: If complex example fails, try basic version first
5. **Search docs**: Man pages are comprehensive (`man namespaces`, `man cgroups`)

### Useful Commands for Debugging

```bash
# List all namespaces
lsns

# See which namespace a process is in
ls -la /proc/<PID>/ns/

# List cgroup hierarchy
systemd-cgls

# Monitor cgroup resource usage
systemd-cgtop

# Trace syscalls
strace -f <command>

# Check kernel messages
dmesg -T

# Verify cgroup v2
stat -fc %T /sys/fs/cgroup/
```

---

## Additional Resources

### Books
- **"The Linux Programming Interface"** by Michael Kerrisk
  - Comprehensive Linux system programming
  - Chapters on namespaces, cgroups

- **"Linux Kernel Development"** by Robert Love
  - Understanding kernel internals

### Online Resources

**Official Documentation:**
- Linux man pages: https://man7.org/linux/man-pages/
- OCI Specifications: https://github.com/opencontainers/specs
- Kernel docs: https://www.kernel.org/doc/html/latest/

**Articles:**
- LWN.net namespace series: https://lwn.net/Articles/531114/
- Cgroups v2 guide: https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html

**Videos:**
- "Cgroups, namespaces, and beyond" - Jérôme Petazzoni
- "Building a container from scratch" - Liz Rice

### Tools to Install

```bash
# Namespace utilities
sudo apt-get install util-linux        # unshare, nsenter, lsns

# Network tools
sudo apt-get install iproute2          # ip command
sudo apt-get install bridge-utils      # brctl

# Container tools
sudo apt-get install runc              # OCI runtime
sudo apt-get install podman            # Docker alternative
sudo apt-get install buildah           # Build containers

# Debugging
sudo apt-get install strace            # Syscall tracing
sudo apt-get install htop              # Process monitoring

# Testing
sudo apt-get install stress-ng         # Resource stress testing
```

---

## What's Next?

After completing this learning path, you're ready for:

### Container Ecosystem
- **Buildah/Podman**: Build and manage containers
- **Kubernetes**: Container orchestration
- **Docker internals**: How Docker uses runc
- **Alternative runtimes**: crun, gVisor, Kata Containers

### Security
- **Seccomp-BPF**: Deep dive into syscall filtering
- **AppArmor/SELinux**: Mandatory access control
- **Container security**: Best practices and hardening
- **Rootless containers**: Running without privileges

### Advanced Topics
- **CNI (Container Network Interface)**: Standard networking
- **CSI (Container Storage Interface)**: Storage plugins
- **CRI (Container Runtime Interface)**: Kubernetes integration
- **WebAssembly runtimes**: Wasmtime, WasmEdge

### Build Your Own
- Container runtime
- Container orchestrator (simple Kubernetes)
- Container networking system
- Container image builder

---

## Contributing

Found an error? Have a suggestion? Want to add examples?

This is a living document. Feedback welcome!

---

## License

These learning materials are provided for educational purposes.

All code examples are provided as-is with no warranty.

**Safety reminder**: These exercises involve low-level system operations. Use a VM or test system, not production machines!

---

## Acknowledgments

This learning path is inspired by:
- The Linux kernel community
- Open Container Initiative (OCI)
- Container runtime maintainers (runc, crun, etc.)
- Countless blog posts, talks, and documentation

**Special thanks to**:
- Michael Kerrisk for comprehensive Linux man pages
- The runc maintainers for the reference implementation
- Everyone who's written about containers and shared knowledge

---

## Ready to Start?

Begin with **[01-namespaces.md](./01-namespaces.md)**

Good luck on your container learning journey! 🚀

Remember: The best way to learn is by doing. Get your hands dirty, break things, and understand why they broke. That's how you truly learn system programming.
