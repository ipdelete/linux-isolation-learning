# Progress Log: Cgroup Delegation Investigation & Documentation

**Date**: 2025-12-28
**Session**: 023415
**Branch**: ft-valid
**Commit**: 598be2b

## Overview

Investigated why cgroup memory/CPU limits fail in the DevContainer environment and documented a Linux VM workaround. The root cause is Docker's cgroup namespace isolation combined with the cgroup v2 "no internal processes" rule.

## What We Built

### Environment Documentation
Added comprehensive documentation for two environment options:
1. **DevContainer** - Works for 8 of 10 lessons (namespaces, cgroup basics, OCI, eBPF)
2. **Linux VM** - Required for lessons 06-07 (memory/CPU limits)

### VM Setup Guide
Step-by-step instructions for Mac users with Apple Silicon to set up Ubuntu ARM64 in UTM with VS Code Remote-SSH.

## The Problem

### Cgroup v2 "No Internal Processes" Rule

```
/sys/fs/cgroup/                    <- Root cgroup
├── cgroup.controllers             -> "cpu memory pids io ..." (available)
├── cgroup.subtree_control         -> (empty, cannot be changed!)
├── cgroup.procs                   -> 24 processes (PID 1, VS Code, shell, etc.)
└── child/                         <- Child cgroups
    └── memory.max                 <- DOES NOT EXIST (no controllers delegated)
```

**The rule**: You cannot enable `cgroup.subtree_control` when processes exist directly in the cgroup. Since all 24 container processes live in the root cgroup (including PID 1), we cannot delegate controllers to child cgroups.

**What crashed the DevContainer earlier**: Writing to `/sys/fs/cgroup/memory.max` limits the root cgroup, which limits the entire container including VS Code server. Setting a restrictive limit caused OOM kills.

### Investigation Commands Used

```bash
# Check current process cgroup location
cat /proc/self/cgroup
# Output: 0::/  (root cgroup)

# Check available controllers
cat /sys/fs/cgroup/cgroup.controllers
# Output: cpuset cpu io memory hugetlb pids rdma

# Check delegated controllers (empty!)
cat /sys/fs/cgroup/cgroup.subtree_control
# Output: (nothing)

# Count processes in root cgroup
wc -l < /sys/fs/cgroup/cgroup.procs
# Output: 24

# Try to enable delegation (fails)
echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control
# Exit code: 1 (fails because processes exist in cgroup)
```

## Files Modified

### Created
None (documentation updates only)

### Modified

| File | Change |
|------|--------|
| `.devcontainer/validation.md:1-74` | Added Environment Options section with Linux VM setup instructions |
| `docs/fast-track/README.md:5-22` | Added environment options table and prerequisites |
| `docs/fast-track/README.md:40-53` | Added Env column to lessons table (DC vs VM) |
| `docs/fast-track/06-memory-limits.md:1-5` | Added VM requirement warning banner |
| `docs/fast-track/07-cpu-limits.md:1-5` | Added VM requirement warning banner |

### Moved (to completed)
| File | Status |
|------|--------|
| `backlog/bugs/completed/BUG-036-fast-track-memory-limits-missing-cgroup-delegation-steps.md` | RESOLVED |
| `backlog/bugs/completed/BUG-037-fast-track-cpu-limits-missing-cgroup-delegation-steps.md` | RESOLVED |

## Key Concepts

### Cgroup v2 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Host System                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Docker Container (DevContainer)           │  │
│  │                                                        │  │
│  │   /sys/fs/cgroup/  (container's root cgroup)          │  │
│  │   ├── cgroup.controllers: cpu memory pids...          │  │
│  │   ├── cgroup.subtree_control: (empty)                 │  │
│  │   ├── cgroup.procs: PID 1, VS Code, shell...          │  │
│  │   │                                                    │  │
│  │   │   Cannot enable subtree_control because           │  │
│  │   │   processes exist here (no internal processes     │  │
│  │   │   rule)                                            │  │
│  │   │                                                    │  │
│  │   └── child/                                           │  │
│  │       └── (no memory.max, cpu.max - not delegated)    │  │
│  │                                                        │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Why Linux VM Works

With systemd as PID 1 in a real Linux VM:

```
/sys/fs/cgroup/
├── user.slice/
│   └── user-1000.slice/
│       ├── cgroup.subtree_control: cpu memory pids io  <- Delegated!
│       └── test-cgroup/
│           └── memory.max  <- Writable!
└── system.slice/
    └── (system services)
```

Systemd creates proper slice hierarchy and delegates controllers to user sessions.

## How to Use

### DevContainer (Most Lessons)
```bash
# Just open in VS Code with DevContainers extension
# Works for lessons 01-05, 08-10
cargo build -p contain
cargo test -p contain
```

### Linux VM (Lessons 06-07)
```bash
# 1. Install UTM on Mac
# 2. Download Ubuntu 24.04 ARM64 server ISO
# 3. Create VM: 4GB RAM, 20GB disk
# 4. In VM:
sudo apt update && sudo apt install -y openssh-server
ip addr  # Get IP

# 5. On Mac, add to ~/.ssh/config:
#    Host linux-vm
#        HostName <vm-ip>
#        User <username>

# 6. VS Code: Remote-SSH -> Connect to linux-vm
# 7. Clone repo and verify:
cat /sys/fs/cgroup/user.slice/user-$(id -u).slice/cgroup.subtree_control
# Should show: cpu memory pids io ...
```

## Technical Notes

### Safe vs Dangerous Operations in DevContainer

| Operation | Safety |
|-----------|--------|
| Read `memory.current`, `memory.stat` | Safe |
| Read `cpu.stat`, `pids.current` | Safe |
| Create empty child cgroups | Safe (but no controllers) |
| Write `memory.max` on root | **CRASHES CONTAINER** |
| Write `cpu.max` on root | Throttles everything |

### OOM Kill Evidence from dmesg
```
Memory cgroup out of memory: Killed process 97417 (node)
Memory cgroup out of memory: Killed process 47075 (node)
```

## Next Steps (Not Implemented)

1. **Systemd-based DevContainer** - Could use a systemd image that handles cgroup delegation at startup, but adds complexity and startup time

2. **Docker Compose with cgroup_parent** - Might work on native Linux hosts but not Docker Desktop

3. **Alternative cgroup exercises** - Could add read-only cgroup exercises that work in DevContainer (reading stats, watching memory.current change)

4. **OrbStack testing** - OrbStack's Linux machines might have better cgroup support than Docker containers

## Repository Information

- **URL**: https://github.com/ipdelete/linux-isolation-learning.git
- **Branch**: ft-valid
- **Commit**: 598be2b
- **Message**:
  ```
  docs: add Linux VM setup for cgroup lessons (BUG-036, BUG-037)

  DevContainer cannot write to cgroup control files (memory.max, cpu.max)
  due to Docker's cgroup namespace isolation and the "no internal processes"
  rule preventing subtree_control enablement.
  ```

## Lessons Learned

1. **Cgroup v2 has stricter rules than v1** - The "no internal processes" rule is a key constraint
2. **Docker Desktop adds layers of isolation** - Even with `--privileged`, cgroup namespace limits what you can do
3. **Always test cgroup changes on separate processes** - Never limit the cgroup containing your shell/IDE
4. **Documentation over code changes** - Sometimes the right fix is better docs, not more code
