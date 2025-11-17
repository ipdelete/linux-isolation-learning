# CLAUDE.md - linux-isolation-learning

## Project Overview

This is a comprehensive, hands-on educational project designed to teach container fundamentals from first principles. It progresses from raw Linux kernel features (namespaces, cgroups) through the OCI Runtime Specification (runc), enabling learners to understand exactly how Docker, Kubernetes, Podman, and other container technologies work under the hood.

**Project Goal**: Demystify containers by learning the underlying Linux primitives and how they're assembled into complete container runtimes.

**Estimated Duration**: 40-50 hours of hands-on learning

---

## Directory Structure

```
linux-isolation-learning/
├── README.md                    # Main entry point with learning path overview
├── 00-info.md                   # Quick info about uv scripts format
├── 01-namespaces.md             # Module 1: Linux Namespaces (15-20 hours)
├── 02-cgroups.md                # Module 2: Control Groups (12-15 hours)
├── 03-runc.md                   # Module 3: OCI Runtime (13-15 hours, partial)
├── scratch.md                   # Development notes on systemd-nspawn setup
│
├── src/                         # Python example scripts
│   ├── pid_namespace.py         # Clone-based PID namespace example
│   ├── pid_unshare.py           # Unshare-based PID namespace example (uv script)
│   └── pid_two.py               # Two isolated processes in separate namespaces (uv script)
│
├── scripts/
│   └── lab-env                  # Bash script for systemd-nspawn container management
│
└── lab_env/                     # Documentation for lab environment setup
    ├── 01-Setup.md              # Setting up systemd-nspawn Arch container
    ├── 02-Config.md             # Container configuration details
    ├── 03-Tear-Down-Recreate.md # Cleanup and recreation procedures
    └── 04-testing.md            # Testing guidelines
```

---

## Core Learning Modules

### Module 1: Linux Namespaces (01-namespaces.md)
**Time**: 15-20 hours | **Difficulty**: Intermediate

Teaches the 7 (+ time) namespace types that provide process isolation:

- **PID namespace**: Process ID isolation
- **Network namespace**: Network stack isolation
- **Mount namespace**: Filesystem view isolation
- **UTS namespace**: Hostname isolation
- **IPC namespace**: Inter-process communication isolation
- **User namespace**: UID/GID remapping
- **Cgroup namespace**: Cgroup hierarchy isolation
- **Time namespace**: Clock/time isolation

**Hands-on content**:
- Understanding namespace concepts via `/proc`
- Creating PID namespaces with `clone()` and `unshare()`
- Network namespace setup with `ip netns`, veth pairs, bridges
- Mount namespace and rootfs isolation with chroot/pivot_root
- UTS, IPC, User namespace exercises
- Practical projects: minimal container runtime, network setup scripts, monitoring tools

**Key files**:
- `/home/cip/wrk/linux-isolation-learning/01-namespaces.md` (1550+ lines of detailed content)
- `/home/cip/wrk/linux-isolation-learning/src/pid_namespace.py` - Clone-based example
- `/home/cip/wrk/linux-isolation-learning/src/pid_unshare.py` - Unshare-based example (uv script)
- `/home/cip/wrk/linux-isolation-learning/src/pid_two.py` - Dual namespace example (uv script)

### Module 2: Cgroups / Control Groups (02-cgroups.md)
**Time**: 12-15 hours | **Difficulty**: Intermediate

Teaches resource limiting and monitoring using cgroups v2 (modern standard):

- **Memory controller**: Hard/soft limits, OOM killer
- **CPU controller**: Bandwidth limiting (cpu.max), proportional shares (cpu.weight)
- **I/O controller**: Disk I/O limits and priorities
- **PID controller**: Process count limits (prevent fork bombs)
- **Pressure Stall Information (PSI)**: Resource contention detection

**Hands-on content**:
- Cgroups v2 vs v1 comparison and verification
- Creating cgroup hierarchies
- Memory limits and OOM behavior with stress testing
- CPU bandwidth limiting and proportional weight distribution
- I/O throttling with block device major:minor notation
- Process count limiting
- Combining multiple controllers for realistic containers
- Nested cgroup hierarchies
- Advanced topics: delegation, freezer, notifications

**Key files**:
- `/home/cip/wrk/linux-isolation-learning/02-cgroups.md` (1250+ lines)

### Module 3: runc (OCI Runtime) (03-runc.md)
**Time**: 13-15 hours | **Difficulty**: Intermediate-Advanced

Covers the reference OCI runtime implementation used by Docker/Kubernetes:

- **OCI specification structure**: Runtime, Image, Distribution specs
- **Bundle format**: config.json + rootfs directory
- **Container lifecycle**: create, start, kill, delete operations
- **Advanced features**: Seccomp, capabilities, hooks
- **Integration**: Combining namespaces + cgroups + security

**Note**: This module is partially included in the repo (first 100 lines read)

**Key files**:
- `/home/cip/wrk/linux-isolation-learning/03-runc.md` (partial, ~900 lines total)

---

## Project Structure & Learning Progression

### Prerequisites
- Linux system (Ubuntu 20.04+, Fedora 34+, Arch)
- Root/sudo access
- Basic Linux command-line knowledge
- Basic programming (C, Python, or Bash)
- Kernel 4.5+ with cgroups v2 support
- 4GB+ RAM and 20GB+ free disk space

### Suggested Learning Paths

**Intensive (2 weeks full-time)**:
- Week 1: Namespaces (Days 1-4), Cgroups (Days 5-7)
- Week 2: runc (Days 8-12), Projects (Days 13-14)

**Part-time (8 weeks, 5-7 hours/week)**:
- Weeks 1-3: Namespaces
- Weeks 4-5: Cgroups
- Weeks 6-8: runc

**Self-paced**: Complete each module at your own pace

### Learning Strategy
1. **Follow the Order**: Modules build on each other (Namespaces → Cgroups → runc)
2. **Hands-on Practice**: Type every command, run all examples, modify configurations
3. **Experiment**: Try variations, ask "what if I change X?"
4. **Take Notes**: Document concepts, commands, and "aha!" moments
5. **Build Projects**: Complete all practical projects for consolidation

---

## Key Scripts and Tools

### Example Scripts (src/)

**pid_namespace.py**
- Uses `ctypes.clone()` with `CLONE_NEWPID` flag
- Creates a child process in a new PID namespace
- Child becomes PID 1 in the new namespace
- Requires root: `sudo ./pid_namespace.py`

**pid_unshare.py** (uv script)
- Uses `ctypes.unshare()` with `CLONE_NEWPID` flag
- Different approach: unshare applies to current process
- Parent becomes PID 1 when unshare is called before fork
- Runnable with: `./pid_unshare.py` or `uv run pid_unshare.py`
- Requires Python 3.12+

**pid_two.py** (uv script)
- Creates TWO separate PID namespaces with `clone()` calls
- Each runs `sleep 100` in its own isolated namespace
- Demonstrates namespace isolation: processes can't see each other
- Useful for verifying that different inode numbers = different namespaces

### Lab Environment (scripts/lab-env)
- Bash script for managing systemd-nspawn Arch container
- Provides shell access to a containerized Arch environment
- Enables testing namespace/cgroup code in an isolated system
- Commands: `exec`, `shell`, other container management operations

---

## Important Notes for Claude Code

### System Environment
- Platform: Linux (Arch-based system, "Omarchy")
- Git Repository: Yes (main branch, recent commits on namespaces and systemd-nspawn)
- Working Directory: `/home/cip/wrk/linux-isolation-learning`

### File Format Notes
- **00-info.md**: Documents PEP 723 uv script format used in some examples
- **Scripts (pid_unshare.py, pid_two.py)**: Use uv inline dependency format with shebang
  ```python
  #!/usr/bin/env -S uv run --script
  # /// script
  # requires-python = ">=3.12"
  # dependencies = []
  # ///
  ```

### Related Documentation
- **scratch.md**: Contains practical notes on systemd-nspawn setup and Arch container management
- **lab_env/**: Setup guides for the containerized learning environment

### Recent Work
- Enhanced testing documentation
- Added systemd drop-in for container capabilities
- Improved namespace documentation with appendices on fork/clone comparison
- Added Exercise 3 appendix for verifying namespace isolation

---

## How to Help Users

### Common Tasks

1. **Understanding a specific namespace type**
   - Refer to the appropriate section in 01-namespaces.md
   - Point to relevant exercises and appendices

2. **Debugging cgroup issues**
   - Check /sys/fs/cgroup hierarchy
   - Verify cgroups v2 is mounted: `stat -fc %T /sys/fs/cgroup/`
   - Refer to troubleshooting section in 02-cgroups.md

3. **Running example scripts**
   - pid_namespace.py: `sudo python3 src/pid_namespace.py` or `sudo ./src/pid_namespace.py`
   - pid_unshare.py: `./src/pid_unshare.py` (requires uv and Python 3.12+)
   - pid_two.py: `./src/pid_two.py` (requires uv and Python 3.12+)

4. **Setting up lab environment**
   - Refer to lab_env/01-Setup.md for systemd-nspawn setup
   - Use scripts/lab-env for container management

5. **Providing exercises and verification steps**
   - Each module ends with hands-on exercises
   - Include verification commands (lsns, readlink /proc/*/ns/*, systemd-cgtop, etc.)
   - Encourage users to compare outputs before/after changes

### Debugging Workflows

**Namespace Issues**:
```bash
# View all namespaces
lsns

# Check namespace of process
ls -la /proc/<PID>/ns/

# Compare namespace inodes
readlink /proc/<PID1>/ns/pid
readlink /proc/<PID2>/ns/pid
```

**Cgroup Issues**:
```bash
# List cgroup hierarchy
systemd-cgls

# View specific cgroup
cat /sys/fs/cgroup/<cgroup-path>/cgroup.procs

# Monitor resources
systemd-cgtop
watch -n 1 'cat /sys/fs/cgroup/<cgroup-path>/*'
```

**Permission Issues**:
- Most operations require root: use `sudo`
- Verify cgroup v2 support: `stat -fc %T /sys/fs/cgroup/` should return "cgroup2fs"
- Check kernel version: `uname -r` (needs 4.5+)

---

## Testing Knowledge

Users can self-assess with the following checklists:

### After Namespaces
- [ ] Create a process with PID 1 in its own namespace?
- [ ] Set up networking between two isolated namespaces?
- [ ] Create a minimal container with mount namespace?
- [ ] Explain the difference between unshare() and clone()?

### After Cgroups
- [ ] Limit a process to 100MB of memory?
- [ ] Give two processes 2:1 CPU ratio?
- [ ] Prevent fork bombs with pids.max?
- [ ] Monitor resource pressure in real-time?

### After runc
- [ ] Run an OCI container from scratch?
- [ ] Write a valid config.json by hand?
- [ ] Debug why a container won't start?
- [ ] Integrate networking with runc containers?

---

## Additional Resources

### Documentation Referenced
- Linux man pages: https://man7.org/linux/man-pages/
- OCI Specifications: https://github.com/opencontainers/specs
- Kernel docs: https://www.kernel.org/doc/html/latest/
- LWN namespace series: https://lwn.net/Articles/531114/
- Cgroups v2 guide: https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html

### Essential Man Pages
```bash
man namespaces          # Overview
man pid_namespaces      # PID namespace details
man cgroups             # Cgroups overview
man systemd-nspawn      # Container management
man runc                # OCI runtime
```

### Recommended Tools to Install
```bash
sudo pacman -S util-linux          # unshare, nsenter, lsns
sudo pacman -S iproute2            # ip command
sudo pacman -S bridge-utils        # brctl
sudo pacman -S runc                # OCI runtime
sudo pacman -S stress-ng           # Resource stress testing
sudo pacman -S systemd-container   # systemd-nspawn
```

---

## Next Steps After Completion

Advanced topics users can explore after mastering this learning path:

- **Buildah/Podman**: Build and manage containers
- **Kubernetes**: Container orchestration
- **Docker internals**: How Docker uses runc
- **Seccomp-BPF**: Deep dive into syscall filtering
- **AppArmor/SELinux**: Mandatory access control
- **Container security**: Best practices and hardening
- **CNI/CSI**: Container networking and storage interfaces
- **WebAssembly runtimes**: Wasmtime, WasmEdge

---

## Git Information

- **Repository**: Git repository with version history
- **Main Branch**: Used for PRs and stable content
- **Recent Commits**:
  - Enhanced testing documentation and systemd drop-in
  - Added systemd-nspawn container setup and management
  - Initial commit with namespaces, cgroups, and runc guides

---

## Project Philosophy

This project emphasizes **learning by doing**:

1. Don't just read - execute commands and observe results
2. Break things intentionally to understand why they work
3. Modify examples and see what happens
4. Build practical projects, not just abstract concepts
5. Connect raw kernel features to real container technology

**Safety Reminder**: These exercises involve low-level system operations. Use a VM or test system, not production machines!

---

## File Accessibility

All documentation is written in Markdown for readability and is version-controlled in Git. Code examples are provided in Python (using ctypes and uv script format) and Bash for maximum compatibility with the learning environment.
