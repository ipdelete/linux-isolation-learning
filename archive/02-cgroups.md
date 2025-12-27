# Learning Linux Cgroups (Control Groups)

A comprehensive guide to understanding and using Linux cgroups for resource management and limitation.

## What Are Cgroups?

Cgroups (control groups) are a Linux kernel feature that limits, accounts for, and isolates the resource usage (CPU, memory, disk I/O, network, etc.) of a collection of processes.

**Think of it as**: Resource budgets for processes - "you can use this much memory and no more."

---

## Prerequisites

- Completed namespace learning (01-namespaces.md)
- Linux system with cgroup v2 support (kernel 4.5+, Ubuntu 20.04+)
- Root access (or sudo)
- Basic understanding of:
  - Linux processes
  - System resources (CPU, memory, I/O)

---

## Phase 1: Understanding Cgroups (2-3 hours)

### 1.1 Cgroups v1 vs v2

**Important**: There are two cgroup versions!

| Aspect | Cgroups v1 | Cgroups v2 |
|--------|-----------|-----------|
| **Released** | 2008 | 2016 |
| **Hierarchy** | Multiple per controller | Single unified hierarchy |
| **Interface** | Complex, inconsistent | Simplified, consistent |
| **Status** | Deprecated | Current standard |
| **Mount point** | `/sys/fs/cgroup/<controller>` | `/sys/fs/cgroup` |

**Check your system:**

```bash
# Check if cgroup v2 is available
mount | grep cgroup2

# If using v2:
# cgroup2 on /sys/fs/cgroup type cgroup2 (rw,nosuid,nodev,noexec,relatime)

# Check which controllers are available
cat /sys/fs/cgroup/cgroup.controllers
# Output: cpuset cpu io memory hugetlb pids rdma misc

# Check cgroup version
stat -fc %T /sys/fs/cgroup/
# Output: cgroup2fs (v2) or tmpfs (v1)
```

**This guide focuses on cgroups v2** (the modern standard).

### 1.2 Cgroup Controllers

Controllers manage specific resource types:

| Controller | Purpose | Key Resources |
|-----------|---------|---------------|
| **cpu** | CPU time distribution | cpu.max, cpu.weight |
| **memory** | Memory limits | memory.max, memory.high |
| **io** | Block I/O (disk) | io.max, io.weight |
| **pids** | Process count limits | pids.max |
| **cpuset** | CPU and memory node assignment | cpuset.cpus, cpuset.mems |
| **rdma** | RDMA resources | rdma.max |
| **hugetlb** | Huge pages | hugetlb.<size>.max |

### 1.3 Cgroup Hierarchy Concepts

Cgroups form a tree structure:

```
/sys/fs/cgroup (root cgroup)
├── system.slice              (system services)
│   ├── sshd.service
│   └── cron.service
├── user.slice               (user sessions)
│   └── user-1000.slice
│       └── session-1.scope
└── my-containers/           (custom cgroups)
    ├── container-1/
    │   ├── cgroup.procs     (PIDs in this cgroup)
    │   ├── memory.max       (memory limit)
    │   └── cpu.max          (CPU quota)
    └── container-2/
```

**Key principles:**
- Child cgroups inherit from parent
- Resources are distributed top-down
- Processes can only be in leaf cgroups (v2)

---

## Phase 2: Basic Cgroup Operations (3-4 hours)

### 2.1 Exploring the Cgroup Filesystem

```bash
# Navigate to cgroup root
cd /sys/fs/cgroup

# List available controllers
cat cgroup.controllers

# List processes in root cgroup
cat cgroup.procs

# View subtree control (which controllers are enabled for children)
cat cgroup.subtree_control
```

### 2.2 Creating Your First Cgroup

```bash
# Create a cgroup (requires root)
sudo mkdir /sys/fs/cgroup/my-test-cgroup

# Enable controllers for this cgroup's children
echo "+cpu +memory +pids" | sudo tee /sys/fs/cgroup/cgroup.subtree_control

# Verify it was created
ls /sys/fs/cgroup/my-test-cgroup/

# You'll see files like:
# - cgroup.procs         (processes in this cgroup)
# - cpu.max             (CPU limit)
# - memory.max          (memory limit)
# - pids.max            (process count limit)
```

### 2.3 Adding Processes to a Cgroup

```bash
# Start a process
sleep 300 &
PID=$!

echo "Started process $PID"

# Add process to cgroup
echo $PID | sudo tee /sys/fs/cgroup/my-test-cgroup/cgroup.procs

# Verify it's in the cgroup
cat /sys/fs/cgroup/my-test-cgroup/cgroup.procs

# Check which cgroup a process is in
cat /proc/$PID/cgroup
# Output: 0::/my-test-cgroup
```

### 2.4 Cleaning Up

```bash
# Kill the process
kill $PID

# Remove the cgroup (must be empty)
sudo rmdir /sys/fs/cgroup/my-test-cgroup
```

**Important**: You can only remove empty cgroups!

---

## Phase 3: Memory Controller (Hands-on: 3-4 hours)

### 3.1 Memory Controller Files

```bash
cd /sys/fs/cgroup/my-cgroup/

# Key memory files:
# memory.current        - Current memory usage
# memory.max           - Hard limit (OOM kill if exceeded)
# memory.high          - Soft limit (throttling)
# memory.min           - Protected memory (won't be reclaimed)
# memory.low           - Best-effort protection
# memory.events        - OOM events, etc.
# memory.stat          - Detailed statistics
```

### 3.2 Setting Memory Limits

**Example: Limit to 50MB**

```bash
# Create cgroup
sudo mkdir /sys/fs/cgroup/memory-test
echo "+memory" | sudo tee /sys/fs/cgroup/cgroup.subtree_control

# Set 50MB hard limit
echo "52428800" | sudo tee /sys/fs/cgroup/memory-test/memory.max
# 52428800 bytes = 50 MB

# Or use shorthand:
echo "50M" | sudo tee /sys/fs/cgroup/memory-test/memory.max
```

### 3.3 Test Memory Limits

Create `memory_hog.c`:

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main() {
    size_t chunk_size = 10 * 1024 * 1024;  // 10 MB chunks
    int chunks = 0;

    while (1) {
        char *mem = malloc(chunk_size);
        if (!mem) {
            printf("malloc failed after %d MB\n", chunks * 10);
            return 1;
        }

        // Actually write to the memory (force allocation)
        memset(mem, 1, chunk_size);

        chunks++;
        printf("Allocated %d MB\n", chunks * 10);

        sleep(1);
    }

    return 0;
}
```

**Compile and test:**

```bash
gcc -o memory_hog memory_hog.c

# Run without limits
./memory_hog
# Will allocate until system runs out of memory!
# Press Ctrl+C to stop

# Run with cgroup limit
./memory_hog &
PID=$!

# Add to cgroup
echo $PID | sudo tee /sys/fs/cgroup/memory-test/cgroup.procs

# Watch memory usage
watch -n 1 'cat /sys/fs/cgroup/memory-test/memory.current'

# Process will be killed by OOM (Out of Memory) killer when hitting 50MB!
```

### 3.4 Understanding OOM Killer

When memory.max is exceeded:

1. Kernel tries to reclaim memory (swap, drop caches)
2. If that fails, OOM killer activates
3. Kills processes in the cgroup

**View OOM events:**

```bash
cat /sys/fs/cgroup/memory-test/memory.events
# Output:
# low 0
# high 0
# max 0
# oom 1        <-- OOM kill happened!
# oom_kill 1   <-- Number of processes killed
```

### 3.5 Memory Pressure and Soft Limits

**memory.high** causes throttling instead of killing:

```bash
# Set soft limit at 30MB, hard limit at 50MB
echo "30M" | sudo tee /sys/fs/cgroup/memory-test/memory.high
echo "50M" | sudo tee /sys/fs/cgroup/memory-test/memory.max

# Process will be throttled (slowed down) at 30MB
# Process will be killed at 50MB
```

### 3.6 Exercises

1. **Exercise 1**: Create a Python memory hog and limit it to 100MB:
   ```python
   data = []
   while True:
       data.append('x' * 1024 * 1024)  # 1MB strings
       print(f"Allocated {len(data)} MB")
   ```

2. **Exercise 2**: Create two cgroups with different memory limits and run processes in each. Monitor which gets killed first.

3. **Exercise 3**: Use `memory.stat` to understand memory breakdown:
   ```bash
   cat /sys/fs/cgroup/memory-test/memory.stat | grep anon
   # Shows anonymous memory (heap, stack)
   ```

---

## Phase 4: CPU Controller (Hands-on: 3-4 hours)

### 4.1 CPU Controller Files

```bash
# cpu.max          - CPU quota (bandwidth limiting)
# cpu.weight       - CPU shares (proportional distribution)
# cpu.stat         - CPU usage statistics
# cpu.pressure     - CPU pressure information
```

### 4.2 CPU Bandwidth Limiting (cpu.max)

**Format**: `<quota> <period>`
- Quota: microseconds of CPU time per period
- Period: usually 100000 (100ms)

**Examples:**

```bash
# Allow 1 full CPU core
echo "100000 100000" | sudo tee /sys/fs/cgroup/cpu-test/cpu.max

# Allow 50% of 1 CPU core (0.5 cores)
echo "50000 100000" | sudo tee /sys/fs/cgroup/cpu-test/cpu.max

# Allow 2 full CPU cores
echo "200000 100000" | sudo tee /sys/fs/cgroup/cpu-test/cpu.max

# Unlimited CPU
echo "max 100000" | sudo tee /sys/fs/cgroup/cpu-test/cpu.max
```

### 4.3 Test CPU Limits

Create `cpu_hog.c`:

```c
#include <stdio.h>
#include <time.h>

int main() {
    printf("Starting CPU-intensive task...\n");

    long long iterations = 0;
    time_t start = time(NULL);

    while (1) {
        // Busy loop (consumes 100% CPU)
        for (int i = 0; i < 1000000; i++) {
            iterations++;
        }

        // Print status every second
        time_t now = time(NULL);
        if (now > start) {
            printf("Iterations per second: %lld\n", iterations / (now - start));
        }
    }

    return 0;
}
```

**Test without limits:**

```bash
gcc -o cpu_hog cpu_hog.c

# Run on unlimited CPU
./cpu_hog &
PID=$!

# Check CPU usage (should be ~100%)
top -p $PID

kill $PID
```

**Test with 20% CPU limit:**

```bash
# Create cgroup
sudo mkdir /sys/fs/cgroup/cpu-test
echo "+cpu" | sudo tee /sys/fs/cgroup/cgroup.subtree_control

# Limit to 20% of 1 core (20000 microseconds per 100ms period)
echo "20000 100000" | sudo tee /sys/fs/cgroup/cpu-test/cpu.max

# Run process in cgroup
./cpu_hog &
PID=$!
echo $PID | sudo tee /sys/fs/cgroup/cpu-test/cgroup.procs

# Check CPU usage (should be ~20%)
top -p $PID

# Compare iterations (should be ~5x slower than unlimited)
```

### 4.4 CPU Weight (Proportional Shares)

**cpu.weight** distributes CPU proportionally when contention exists.

```bash
# Create two cgroups
sudo mkdir /sys/fs/cgroup/cpu-high
sudo mkdir /sys/fs/cgroup/cpu-low

# High priority: weight 200 (2x default)
echo "200" | sudo tee /sys/fs/cgroup/cpu-high/cpu.weight

# Low priority: weight 50 (0.5x default)
echo "50" | sudo tee /sys/fs/cgroup/cpu-low/cpu.weight

# Run CPU hogs in each
./cpu_hog &
HIGH_PID=$!
./cpu_hog &
LOW_PID=$!

echo $HIGH_PID | sudo tee /sys/fs/cgroup/cpu-high/cgroup.procs
echo $LOW_PID | sudo tee /sys/fs/cgroup/cpu-low/cgroup.procs

# Monitor - high should get 4x more CPU than low
top -p $HIGH_PID,$LOW_PID
```

**Weight ratio**: 200:50 = 4:1

### 4.5 CPU Statistics

```bash
cat /sys/fs/cgroup/cpu-test/cpu.stat

# Output:
# usage_usec 1234567890      # Total CPU time used (microseconds)
# user_usec 1234567800        # User mode CPU time
# system_usec 90              # Kernel mode CPU time
# nr_periods 12345            # Number of periods elapsed
# nr_throttled 5000           # Number of times throttled
# throttled_usec 1000000000   # Total time throttled
```

### 4.6 Exercises

1. **Exercise 1**: Run 4 CPU-intensive processes in separate cgroups with different weights. Verify proportional distribution.

2. **Exercise 2**: Use `stress` to create CPU load and measure the effect of cpu.max:
   ```bash
   sudo apt-get install stress
   stress --cpu 4 --timeout 60
   ```

3. **Exercise 3**: Create a cgroup that limits a process to 10 seconds of CPU time total, then kills it.

---

## Phase 5: I/O Controller (Hands-on: 3-4 hours)

### 5.1 I/O Controller Files

```bash
# io.max           - I/O bandwidth limits
# io.weight        - I/O proportional weight
# io.stat          - I/O statistics
# io.pressure      - I/O pressure information
```

### 5.2 Finding Your Disk Device

```bash
# List block devices
lsblk

# Find device major:minor numbers
ls -l /dev/sda
# Output: brw-rw---- 1 root disk 8, 0 Jan 1 12:00 /dev/sda
#                              ^  ^
#                           major minor

# Or use:
cat /proc/partitions
# Output:
# major minor  #blocks  name
#    8       0  488386584 sda
```

### 5.3 Setting I/O Limits

**Format**: `<major>:<minor> rbps=<bytes> wbps=<bytes>`

```bash
# Create cgroup
sudo mkdir /sys/fs/cgroup/io-test
echo "+io" | sudo tee /sys/fs/cgroup/cgroup.subtree_control

# Limit to 1 MB/s read, 1 MB/s write on /dev/sda (8:0)
echo "8:0 rbps=1048576 wbps=1048576" | sudo tee /sys/fs/cgroup/io-test/io.max

# Or limit IOPS (I/O operations per second)
echo "8:0 riops=100 wiops=100" | sudo tee /sys/fs/cgroup/io-test/io.max
```

### 5.4 Test I/O Limits

**Create test file:**

```bash
# Generate 1GB test file
dd if=/dev/zero of=/tmp/testfile bs=1M count=1024

# Test read speed without limits
dd if=/tmp/testfile of=/dev/null bs=1M
# Should be fast (100+ MB/s on typical hardware)
```

**Test with cgroup limits:**

```bash
# Run dd in cgroup
bash -c "echo \$\$ | sudo tee /sys/fs/cgroup/io-test/cgroup.procs && \
         dd if=/tmp/testfile of=/dev/null bs=1M"

# Output will show ~1 MB/s (limited by io.max)
# 1073741824 bytes (1.1 GB) copied, 1000 s, 1.0 MB/s
```

### 5.5 I/O Statistics

```bash
cat /sys/fs/cgroup/io-test/io.stat

# Output:
# 8:0 rbytes=1073741824 wbytes=0 rios=1024 wios=0 dbytes=0 dios=0
#     ^        ^           ^       ^      ^      ^
#     device   read bytes  written read   write  discarded
```

### 5.6 I/O Weight (Proportional)

Similar to CPU weight, distributes I/O proportionally:

```bash
# High priority I/O
echo "200" | sudo tee /sys/fs/cgroup/io-high/io.weight

# Low priority I/O
echo "50" | sudo tee /sys/fs/cgroup/io-low/io.weight

# When disk is contended, io-high gets 4x more I/O than io-low
```

### 5.7 Exercises

1. **Exercise 1**: Create an I/O-intensive script and limit it to 5 MB/s:
   ```bash
   while true; do
       dd if=/dev/zero of=/tmp/test bs=1M count=100
   done
   ```

2. **Exercise 2**: Monitor I/O pressure during heavy I/O:
   ```bash
   cat /sys/fs/cgroup/io-test/io.pressure
   ```

3. **Exercise 3**: Compare performance of two processes with different io.weight values writing to disk simultaneously.

---

## Phase 6: PID Controller (Hands-on: 2-3 hours)

### 6.1 PID Controller Purpose

Limits the number of processes/threads that can be created in a cgroup.

**Why this matters:**
- Prevent fork bombs
- Limit container process count
- Protect system resources

### 6.2 Setting PID Limits

```bash
# Create cgroup
sudo mkdir /sys/fs/cgroup/pid-test
echo "+pids" | sudo tee /sys/fs/cgroup/cgroup.subtree_control

# Limit to 10 processes
echo "10" | sudo tee /sys/fs/cgroup/pid-test/pids.max

# View current count
cat /sys/fs/cgroup/pid-test/pids.current
```

### 6.3 Test PID Limits

Create `fork_bomb.sh`:

```bash
#!/bin/bash
# Fork bomb - DO NOT RUN WITHOUT LIMITS!

function fork_bomb() {
    fork_bomb | fork_bomb &
}

# fork_bomb  # Don't actually call this!
```

**Safe test with limits:**

```bash
# Start a shell in the cgroup
sudo bash -c 'echo $$ > /sys/fs/cgroup/pid-test/cgroup.procs && bash'

# Inside the limited shell, try to create many processes
for i in {1..20}; do
    sleep 100 &
done

# After 10 processes, you'll get:
# bash: fork: retry: Resource temporarily unavailable
```

### 6.4 PID Events

```bash
cat /sys/fs/cgroup/pid-test/pids.events

# Output:
# max 5   # Number of times limit was hit
```

### 6.5 Exercises

1. **Exercise 1**: Write a Python script that spawns processes until hitting the limit:
   ```python
   import subprocess
   children = []
   while True:
       children.append(subprocess.Popen(['sleep', '60']))
   ```

2. **Exercise 2**: Create a cgroup that allows only 1 process. Try to run a pipeline:
   ```bash
   ls | grep test | wc -l  # Should fail!
   ```

---

## Phase 7: Combining Controllers (Hands-on: 4-5 hours)

### 7.1 Multi-Resource Container

Create a realistic container-like cgroup:

```bash
# Create container cgroup
sudo mkdir /sys/fs/cgroup/container-1
echo "+cpu +memory +io +pids" | sudo tee /sys/fs/cgroup/cgroup.subtree_control

cd /sys/fs/cgroup/container-1

# CPU: 50% of 1 core
echo "50000 100000" | sudo tee cpu.max

# Memory: 256 MB
echo "256M" | sudo tee memory.max

# I/O: 10 MB/s
echo "8:0 rbps=10485760 wbps=10485760" | sudo tee io.max

# PIDs: 50 processes max
echo "50" | sudo tee pids.max
```

### 7.2 Monitoring Multi-Resource Cgroup

Create `monitor_cgroup.sh`:

```bash
#!/bin/bash
CGROUP=$1

watch -n 1 "
echo '=== CPU ==='
cat /sys/fs/cgroup/$CGROUP/cpu.stat

echo
echo '=== Memory ==='
echo \"Current: \$(cat /sys/fs/cgroup/$CGROUP/memory.current | numfmt --to=iec)\"
echo \"Max: \$(cat /sys/fs/cgroup/$CGROUP/memory.max)\"

echo
echo '=== I/O ==='
cat /sys/fs/cgroup/$CGROUP/io.stat

echo
echo '=== PIDs ==='
echo \"Current: \$(cat /sys/fs/cgroup/$CGROUP/pids.current)\"
echo \"Max: \$(cat /sys/fs/cgroup/$CGROUP/pids.max)\"

echo
echo '=== Processes ==='
cat /sys/fs/cgroup/$CGROUP/cgroup.procs
"
```

### 7.3 Stress Testing Multi-Resource Limits

```bash
# Install stress-ng (better than stress)
sudo apt-get install stress-ng

# Run comprehensive stress test in cgroup
sudo bash -c "
echo \$\$ > /sys/fs/cgroup/container-1/cgroup.procs
stress-ng --cpu 4 --vm 2 --vm-bytes 200M --io 2 --timeout 60s
"

# Monitor in another terminal
./monitor_cgroup.sh container-1
```

### 7.4 Nested Cgroups

Create parent-child resource limits:

```bash
# Parent: 1 full CPU, 512 MB
sudo mkdir /sys/fs/cgroup/parent
echo "+cpu +memory" | sudo tee /sys/fs/cgroup/cgroup.subtree_control
echo "100000 100000" | sudo tee /sys/fs/cgroup/parent/cpu.max
echo "512M" | sudo tee /sys/fs/cgroup/parent/memory.max

# Child 1: Up to 50% of parent's CPU, 256 MB
sudo mkdir /sys/fs/cgroup/parent/child1
echo "+cpu +memory" | sudo tee /sys/fs/cgroup/parent/cgroup.subtree_control
echo "50000 100000" | sudo tee /sys/fs/cgroup/parent/child1/cpu.max
echo "256M" | sudo tee /sys/fs/cgroup/parent/child1/memory.max

# Child 2: Up to 50% of parent's CPU, 256 MB
sudo mkdir /sys/fs/cgroup/parent/child2
echo "50000 100000" | sudo tee /sys/fs/cgroup/parent/child2/cpu.max
echo "256M" | sudo tee /sys/fs/cgroup/parent/child2/memory.max
```

**Resource distribution:**
```
parent (1 CPU, 512 MB)
  ├── child1 (0.5 CPU, 256 MB)
  └── child2 (0.5 CPU, 256 MB)
```

---

## Phase 8: Advanced Topics (4-5 hours)

### 8.1 Cgroup Delegation

Allow non-root users to manage cgroups:

```bash
# Create user-owned cgroup
sudo mkdir /sys/fs/cgroup/user-$UID
sudo chown -R $USER:$USER /sys/fs/cgroup/user-$UID

# Enable controllers
echo "+cpu +memory" | sudo tee /sys/fs/cgroup/cgroup.subtree_control

# Now user can manage without sudo
echo "50000 100000" > /sys/fs/cgroup/user-$UID/cpu.max
```

### 8.2 Pressure Stall Information (PSI)

PSI shows when resources are contended:

```bash
cat /sys/fs/cgroup/container-1/cpu.pressure

# Output:
# some avg10=0.00 avg60=0.00 avg300=0.00 total=0
# full avg10=0.00 avg60=0.00 avg300=0.00 total=0

# Interpretation:
# some: Some processes waiting for CPU
# full: All processes waiting for CPU
# avg10/60/300: Average pressure over 10/60/300 seconds
# total: Total time in microseconds
```

**Monitor pressure during stress:**

```bash
watch -n 1 'cat /sys/fs/cgroup/container-1/*.pressure'
```

### 8.3 Freezer (Pausing Processes)

Freeze all processes in a cgroup:

```bash
# Freeze cgroup (pause all processes)
echo "1" | sudo tee /sys/fs/cgroup/container-1/cgroup.freeze

# Verify frozen
cat /sys/fs/cgroup/container-1/cgroup.events
# frozen 1

# Unfreeze
echo "0" | sudo tee /sys/fs/cgroup/container-1/cgroup.freeze
```

### 8.4 Notifications via cgroup.events

Monitor cgroup state changes:

```bash
# Watch for events
inotify-wait -m /sys/fs/cgroup/container-1/cgroup.events

# Trigger event (e.g., all processes exit)
# Will show:
# /sys/fs/cgroup/container-1/cgroup.events MODIFY
```

### 8.5 Cgroup-aware OOM Killer

When memory is exhausted, prefer killing entire cgroups:

```bash
# Enable cgroup-aware OOM
echo "1" | sudo tee /sys/fs/cgroup/container-1/memory.oom.group

# Now if OOM occurs, ALL processes in cgroup are killed
# (Prevents leaving orphaned processes)
```

---

## Phase 9: Practical Projects (8-10 hours)

### Project 1: Cgroup Management Library (Python)

```python
#!/usr/bin/env python3
"""Simple cgroup v2 management library"""

from pathlib import Path

class Cgroup:
    """Manage a cgroup"""

    CGROUP_ROOT = Path("/sys/fs/cgroup")

    def __init__(self, name):
        self.name = name
        self.path = self.CGROUP_ROOT / name

    def create(self):
        """Create cgroup"""
        self.path.mkdir(parents=True, exist_ok=True)

    def delete(self):
        """Delete cgroup (must be empty)"""
        if self.path.exists():
            self.path.rmdir()

    def add_process(self, pid):
        """Add process to cgroup"""
        procs_file = self.path / "cgroup.procs"
        procs_file.write_text(str(pid))

    def get_processes(self):
        """Get list of PIDs in cgroup"""
        procs_file = self.path / "cgroup.procs"
        if procs_file.exists():
            return [int(pid) for pid in procs_file.read_text().split()]
        return []

    def set_memory_limit(self, bytes):
        """Set memory limit"""
        mem_file = self.path / "memory.max"
        mem_file.write_text(str(bytes))

    def set_cpu_limit(self, quota, period=100000):
        """Set CPU limit (quota/period)"""
        cpu_file = self.path / "cpu.max"
        cpu_file.write_text(f"{quota} {period}")

    def get_memory_usage(self):
        """Get current memory usage"""
        mem_file = self.path / "memory.current"
        return int(mem_file.read_text())

    def get_stats(self):
        """Get comprehensive statistics"""
        stats = {}

        # Memory stats
        stats['memory_current'] = int((self.path / "memory.current").read_text())
        stats['memory_max'] = (self.path / "memory.max").read_text().strip()

        # CPU stats
        cpu_stat = (self.path / "cpu.stat").read_text()
        for line in cpu_stat.split('\n'):
            if line:
                key, value = line.split()
                stats[f'cpu_{key}'] = int(value)

        # PID stats
        stats['pids_current'] = int((self.path / "pids.current").read_text())
        stats['pids_max'] = (self.path / "pids.max").read_text().strip()

        return stats


# Example usage
if __name__ == '__main__':
    import os
    import time

    # Create cgroup
    cg = Cgroup("test-cgroup")
    cg.create()

    # Set limits
    cg.set_memory_limit(100 * 1024 * 1024)  # 100 MB
    cg.set_cpu_limit(50000, 100000)         # 50% CPU

    # Add current process
    cg.add_process(os.getpid())

    # Monitor
    for i in range(10):
        stats = cg.get_stats()
        print(f"Memory: {stats['memory_current'] / 1024 / 1024:.2f} MB")
        print(f"PIDs: {stats['pids_current']}")
        time.sleep(1)

    # Cleanup
    cg.delete()
```

### Project 2: Container Resource Manager

```bash
#!/bin/bash
# container-resources.sh - Manage container resources

CGROUP_BASE="/sys/fs/cgroup/containers"

create_container() {
    local name=$1
    local cpu_percent=$2
    local memory_mb=$3
    local max_pids=$4

    local cgroup_path="$CGROUP_BASE/$name"

    # Create cgroup
    mkdir -p "$cgroup_path"

    # Set CPU limit (percent to quota)
    local cpu_quota=$((cpu_percent * 1000))
    echo "$cpu_quota 100000" > "$cgroup_path/cpu.max"

    # Set memory limit
    echo "${memory_mb}M" > "$cgroup_path/memory.max"

    # Set PID limit
    echo "$max_pids" > "$cgroup_path/pids.max"

    echo "Container '$name' created with:"
    echo "  CPU: ${cpu_percent}%"
    echo "  Memory: ${memory_mb}MB"
    echo "  Max PIDs: $max_pids"
}

run_in_container() {
    local name=$1
    shift
    local command=$@

    local cgroup_path="$CGROUP_BASE/$name"

    if [ ! -d "$cgroup_path" ]; then
        echo "Container '$name' does not exist"
        exit 1
    fi

    # Run command in cgroup
    bash -c "echo \$$ > $cgroup_path/cgroup.procs && exec $command"
}

stats_container() {
    local name=$1
    local cgroup_path="$CGROUP_BASE/$name"

    echo "=== Container: $name ==="
    echo "Memory: $(cat $cgroup_path/memory.current | numfmt --to=iec) / $(cat $cgroup_path/memory.max)"
    echo "PIDs: $(cat $cgroup_path/pids.current) / $(cat $cgroup_path/pids.max)"
    echo
    echo "CPU Stats:"
    cat "$cgroup_path/cpu.stat"
}

delete_container() {
    local name=$1
    local cgroup_path="$CGROUP_BASE/$name"

    # Kill all processes in cgroup
    for pid in $(cat "$cgroup_path/cgroup.procs"); do
        kill -9 $pid 2>/dev/null
    done

    # Remove cgroup
    rmdir "$cgroup_path"
    echo "Container '$name' deleted"
}

# Main
case "$1" in
    create)
        create_container "$2" "$3" "$4" "$5"
        ;;
    run)
        run_in_container "$2" "${@:3}"
        ;;
    stats)
        stats_container "$2"
        ;;
    delete)
        delete_container "$2"
        ;;
    *)
        echo "Usage: $0 {create|run|stats|delete}"
        echo "  create <name> <cpu%> <memory_mb> <max_pids>"
        echo "  run <name> <command>"
        echo "  stats <name>"
        echo "  delete <name>"
        exit 1
        ;;
esac
```

**Usage:**

```bash
# Create container with limits
sudo ./container-resources.sh create my-app 50 512 100

# Run process in container
sudo ./container-resources.sh run my-app stress-ng --cpu 4 --timeout 30s

# Check stats
sudo ./container-resources.sh stats my-app

# Clean up
sudo ./container-resources.sh delete my-app
```

### Project 3: Resource Monitor Dashboard

Create a real-time monitoring dashboard using Python + curses:

```python
#!/usr/bin/env python3
"""Real-time cgroup monitor"""

import curses
import time
from pathlib import Path

def get_cgroup_stats(cgroup_path):
    """Gather cgroup statistics"""
    stats = {}

    # Memory
    try:
        stats['mem_current'] = int((cgroup_path / "memory.current").read_text())
        stats['mem_max'] = (cgroup_path / "memory.max").read_text().strip()
    except:
        pass

    # CPU
    try:
        cpu_stat = (cgroup_path / "cpu.stat").read_text()
        for line in cpu_stat.split('\n'):
            if 'usage_usec' in line:
                stats['cpu_usage'] = int(line.split()[1])
    except:
        pass

    # PIDs
    try:
        stats['pids_current'] = int((cgroup_path / "pids.current").read_text())
    except:
        pass

    return stats

def draw_dashboard(stdscr, cgroup_path):
    """Draw monitoring dashboard"""
    curses.curs_set(0)
    stdscr.nodelay(1)
    stdscr.timeout(1000)

    prev_cpu = None

    while True:
        stdscr.clear()
        stats = get_cgroup_stats(cgroup_path)

        # Title
        stdscr.addstr(0, 0, f"Cgroup Monitor: {cgroup_path.name}", curses.A_BOLD)
        stdscr.addstr(1, 0, "=" * 60)

        row = 3

        # Memory
        if 'mem_current' in stats:
            mem_mb = stats['mem_current'] / 1024 / 1024
            stdscr.addstr(row, 0, "Memory:")
            stdscr.addstr(row, 15, f"{mem_mb:.2f} MB / {stats['mem_max']}")
            row += 1

        # CPU usage rate
        if 'cpu_usage' in stats:
            if prev_cpu:
                cpu_rate = (stats['cpu_usage'] - prev_cpu) / 1_000_000  # Convert to seconds
                cpu_percent = (cpu_rate / 1.0) * 100  # Assuming 1 second interval
                stdscr.addstr(row, 0, "CPU:")
                stdscr.addstr(row, 15, f"{cpu_percent:.1f}%")
                row += 1

            prev_cpu = stats['cpu_usage']

        # PIDs
        if 'pids_current' in stats:
            stdscr.addstr(row, 0, "Processes:")
            stdscr.addstr(row, 15, f"{stats['pids_current']}")
            row += 1

        stdscr.addstr(row + 1, 0, "Press 'q' to quit")
        stdscr.refresh()

        key = stdscr.getch()
        if key == ord('q'):
            break

        time.sleep(1)

if __name__ == '__main__':
    import sys

    if len(sys.argv) < 2:
        print("Usage: cgroup-monitor.py <cgroup-path>")
        sys.exit(1)

    cgroup_path = Path("/sys/fs/cgroup") / sys.argv[1]
    if not cgroup_path.exists():
        print(f"Cgroup not found: {cgroup_path}")
        sys.exit(1)

    curses.wrapper(lambda stdscr: draw_dashboard(stdscr, cgroup_path))
```

---

## Resources

### Official Documentation
```bash
# Kernel documentation (essential reading)
man cgroups
man systemd.resource-control

# Online
# https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html
```

### Books
- "Linux Kernel Development" by Robert Love
- "Understanding the Linux Kernel" by Bovet & Cesati

### Tools
```bash
# Cgroup management
sudo apt-get install cgroup-tools    # Legacy v1 tools
sudo apt-get install systemd         # systemd-cgls, systemd-cgtop

# System monitoring
sudo apt-get install sysstat         # sar, iostat, mpstat
sudo apt-get install stress-ng       # Resource stress testing
```

### Useful Commands
```bash
# List cgroup hierarchy
systemd-cgls

# Top-like cgroup monitor
systemd-cgtop

# Find which cgroup a process is in
cat /proc/<PID>/cgroup

# Show all processes in a cgroup
cat /sys/fs/cgroup/<path>/cgroup.procs | xargs ps -p
```

---

## Common Pitfalls

1. **Forgetting to enable controllers**: Use `cgroup.subtree_control`
2. **Trying to remove non-empty cgroups**: Kill all processes first
3. **Confusing v1 and v2**: Check which version you're using!
4. **Setting limits too low**: Process will be killed/throttled
5. **Not accounting for kernel memory**: Some memory isn't counted in memory.current

---

## Next Steps

After mastering cgroups, you're ready for:
1. **runc** (03-runc.md) - Putting namespaces + cgroups together
2. **seccomp-bpf** - Syscall filtering
3. **capabilities** - Fine-grained privilege management
4. **SELinux/AppArmor** - Mandatory access control

You now understand how Docker and Kubernetes limit container resources!
