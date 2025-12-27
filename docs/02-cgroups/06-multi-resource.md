# 06 Multi-Resource Cgroup: Container-Like Resource Bundles

## Goal

Create a cgroup with combined memory, CPU, and PIDs limits -- a "container-like" resource bundle that protects the host system from runaway processes. You will learn how real containers apply multiple resource constraints together and how to monitor their combined effects.

This lesson synthesizes everything from lessons 01-05 into a practical, production-style workflow.

## Prereqs

- Completed `01-cgv2-basics.md` through `05-pids.md`
- Working implementations of: `create`, `delete`, `attach`, `memory-max`, `cpu-max`, `pids-max`
- `sudo` access for cgroup operations
- Understanding of cgroup v2 controller files

## Why Multi-Resource Limits?

Real containers never use just one resource limit. Consider what happens without combined limits:

| Single Limit Only | Risk |
|-------------------|------|
| Memory only | Process spawns unlimited threads, exhausting CPU |
| CPU only | Process allocates all RAM, triggering OOM |
| PIDs only | Each process allocates maximum memory |

**Production containers use all three together** to create a bounded resource "box" that protects the host regardless of what the contained process attempts.

Docker, Podman, and Kubernetes all configure these limits via the same cgroup v2 interface you have been building.

## Checking Controller Availability

Before applying limits, you need to verify which controllers are available and enabled.

### Available Controllers

The `cgroup.controllers` file lists which controllers the kernel supports at this level:

```bash
cat /sys/fs/cgroup/cgroup.controllers
```

Expected output (varies by kernel config):
```
cpuset cpu io memory hugetlb pids rdma misc
```

### Enabled Controllers for Children

The `cgroup.subtree_control` file controls which controllers are enabled for child cgroups:

```bash
cat /sys/fs/cgroup/cgroup.subtree_control
```

If a controller is not listed here, child cgroups cannot use it. Enable controllers by writing to this file:

```bash
# Enable memory, cpu, and pids controllers for children
echo "+memory +cpu +pids" | sudo tee /sys/fs/cgroup/cgroup.subtree_control
```

**Important**: You cannot enable a controller if processes are directly attached to the parent cgroup (the "no internal processes" rule in cgroup v2).

## Write Tests (Red)

**Test file**: `crates/cgroup-tool/tests/bundle_test.rs`

The test file already exists with test function stubs. Your task is to implement the test functions.

### Step 1: Open the Test File

Open `crates/cgroup-tool/tests/bundle_test.rs`. The file contains test stubs for multi-resource scenarios with `todo!()` placeholders and helpful comments.

### Step 2: Implement the Test Functions

Replace the `todo!()` calls with actual test implementations. Start with simpler tests:

- **test_apply_memory_cpu_pids_bundle**: Verify applying all three limits together (no `#[ignore]` attribute)
- **test_controllers_available**: Verify required controllers are available (no `#[ignore]` attribute)
- Other tests: Remove `#[ignore]` attribute when implementing

Example approach for `test_apply_memory_cpu_pids_bundle`:
1. Create test cgroup directory
2. Apply memory, CPU, and PIDs limits using the `cgroup-tool` subcommands
3. Read back each limit file to verify they were set correctly
4. Clean up the test cgroup

### Step 3: Run Tests (Expect Failure)

Run the tests and expect failure from `todo!()` macros:

```bash
sudo -E cargo test -p cgroup-tool --test bundle_test
```

Expected output: Tests fail because they contain `todo!()` macros (RED phase).

## Build (Green)

This lesson focuses on **using the subcommands you already built** rather than adding new code. The "build" phase here is about understanding how to chain your existing tools together.

### Option A: Shell Script Approach (Recommended First)

Create a shell script that combines your existing subcommands:

```bash
#!/bin/bash
# container-cgroup.sh - Create a container-like cgroup with resource limits

set -e

CGROUP_NAME="${1:-my-container}"
MEMORY_MAX="${2:-104857600}"      # 100MB default
CPU_QUOTA="${3:-50000 100000}"    # 50% CPU default
PIDS_MAX="${4:-50}"               # 50 processes default

echo "Creating container-like cgroup: $CGROUP_NAME"
echo "  Memory limit: $MEMORY_MAX bytes"
echo "  CPU quota: $CPU_QUOTA"
echo "  PIDs limit: $PIDS_MAX"

# Step 1: Create the cgroup
sudo cargo run -q -p cgroup-tool -- create "$CGROUP_NAME"
echo "[OK] Created cgroup"

# Step 2: Apply memory limit
sudo cargo run -q -p cgroup-tool -- memory-max "$CGROUP_NAME" "$MEMORY_MAX"
echo "[OK] Set memory.max = $MEMORY_MAX"

# Step 3: Apply CPU limit
sudo cargo run -q -p cgroup-tool -- cpu-max "$CGROUP_NAME" "$CPU_QUOTA"
echo "[OK] Set cpu.max = $CPU_QUOTA"

# Step 4: Apply PIDs limit
sudo cargo run -q -p cgroup-tool -- pids-max "$CGROUP_NAME" "$PIDS_MAX"
echo "[OK] Set pids.max = $PIDS_MAX"

echo ""
echo "Cgroup '$CGROUP_NAME' ready. To attach a process:"
echo "  sudo cargo run -q -p cgroup-tool -- attach $CGROUP_NAME <PID>"
```

### Option B: Add a Bundle Subcommand (Optional Enhancement)

If you want to extend `cgroup-tool`, add this variant to the `Command` enum:

**Implementation file**: `crates/cgroup-tool/src/main.rs`

Add to the `Command` enum:

```rust
/// Create cgroup with bundled resource limits (container-like)
Bundle {
    /// Cgroup path (relative to /sys/fs/cgroup)
    path: String,
    /// Memory limit in bytes (optional)
    #[arg(long)]
    memory_max: Option<u64>,
    /// CPU quota as "quota period" in microseconds (optional)
    #[arg(long)]
    cpu_quota: Option<String>,
    /// Maximum number of processes (optional)
    #[arg(long)]
    pids_max: Option<u64>,
},
```

Add the match arm:

```rust
// TODO: Implement bundle creation
// Lesson: docs/02-cgroups/06-multi-resource.md
// Tests: tests/bundle_test.rs
//
// Implementation hints:
// - Create the cgroup directory first
// - Apply each limit that is Some(_)
// - Print summary of applied limits
// - Order of limit application doesn't matter
Command::Bundle { path, memory_max, cpu_quota, pids_max } => {
    todo!("Implement bundle creation - combines create + multiple limits")
}
```

Usage would then be:

```bash
sudo cargo run -p cgroup-tool -- bundle my-container \
    --memory-max 104857600 \
    --cpu-quota "50000 100000" \
    --pids-max 50
```

## Verify

### Automated Verification

```bash
# Run all cgroup-tool tests
sudo -E cargo test -p cgroup-tool

# Run specifically the bundle tests
sudo -E cargo test -p cgroup-tool --test bundle_test
```

### Manual Verification: Create and Inspect

**Step 1: Check controller availability**

```bash
# What controllers does this kernel support?
cat /sys/fs/cgroup/cgroup.controllers

# What controllers are enabled for child cgroups?
cat /sys/fs/cgroup/cgroup.subtree_control
```

**Step 2: Create a container-like cgroup**

```bash
# Create the cgroup
sudo cargo run -q -p cgroup-tool -- create my-container

# Apply memory limit: 100MB
sudo cargo run -q -p cgroup-tool -- memory-max my-container 104857600

# Apply CPU limit: 50%
sudo cargo run -q -p cgroup-tool -- cpu-max my-container "50000 100000"

# Apply PIDs limit: 50
sudo cargo run -q -p cgroup-tool -- pids-max my-container 50
```

**Step 3: Verify all limits are set**

```bash
# Check all limits in one view
echo "=== Resource Limits ==="
echo "memory.max: $(cat /sys/fs/cgroup/my-container/memory.max)"
echo "cpu.max:    $(cat /sys/fs/cgroup/my-container/cpu.max)"
echo "pids.max:   $(cat /sys/fs/cgroup/my-container/pids.max)"
```

Expected output:
```
=== Resource Limits ===
memory.max: 104857600
cpu.max:    50000 100000
pids.max:   50
```

**Step 4: Attach a process and monitor**

```bash
# Start a background process
sleep 300 &
SLEEP_PID=$!

# Attach it to our cgroup
sudo cargo run -q -p cgroup-tool -- attach my-container $SLEEP_PID

# Verify attachment
echo "Processes in cgroup:"
cat /sys/fs/cgroup/my-container/cgroup.procs

# Check process's view
cat /proc/$SLEEP_PID/cgroup
```

**Step 5: Monitor resource usage**

```bash
echo "=== Current Resource Usage ==="
echo "memory.current: $(cat /sys/fs/cgroup/my-container/memory.current) bytes"
echo "pids.current:   $(cat /sys/fs/cgroup/my-container/pids.current)"
echo ""
echo "=== CPU Statistics ==="
cat /sys/fs/cgroup/my-container/cpu.stat
```

### Stress Testing (Optional)

To see limits in action, you can run stress tests inside the cgroup.

**Memory stress** (be careful -- this will trigger OOM):

```bash
# Start a process that allocates memory
bash -c 'x=""; while true; do x="$x$(head -c 1000000 /dev/zero)"; done' &
STRESS_PID=$!

# Attach to cgroup
sudo cargo run -q -p cgroup-tool -- attach my-container $STRESS_PID

# Watch memory.current increase until OOM
watch -n 0.5 'cat /sys/fs/cgroup/my-container/memory.current'

# Check OOM events
cat /sys/fs/cgroup/my-container/memory.events
```

**CPU stress**:

```bash
# Start CPU-intensive process
bash -c 'while true; do :; done' &
CPU_PID=$!

# Attach to cgroup
sudo cargo run -q -p cgroup-tool -- attach my-container $CPU_PID

# Watch CPU throttling
watch -n 1 'cat /sys/fs/cgroup/my-container/cpu.stat | grep throttled'

# Clean up
kill $CPU_PID
```

**PIDs stress** (fork bomb protection):

```bash
# This is a controlled fork bomb - the cgroup limit prevents system damage
bash -c 'while true; do sleep 100 & done' &
FORK_PID=$!

# Attach to cgroup (must be done quickly!)
sudo cargo run -q -p cgroup-tool -- attach my-container $FORK_PID

# Watch pids.current hit the limit
watch -n 0.5 'cat /sys/fs/cgroup/my-container/pids.current; cat /sys/fs/cgroup/my-container/pids.events'

# Clean up - kill the parent and all children
pkill -P $FORK_PID
kill $FORK_PID 2>/dev/null
```

## Monitoring Multi-Resource Cgroups

Real container runtimes continuously monitor these files. Here is a summary of what to watch:

### Memory Monitoring

| File | Contents | Example |
|------|----------|---------|
| `memory.current` | Current memory usage (bytes) | `52428800` |
| `memory.max` | Configured limit | `104857600` |
| `memory.high` | Throttling threshold | `max` (disabled) |
| `memory.events` | OOM and limit events | `oom_kill 1` |

### CPU Monitoring

| File | Contents | Example |
|------|----------|---------|
| `cpu.max` | Quota and period | `50000 100000` |
| `cpu.stat` | Usage statistics | `usage_usec 1234567` |
| `cpu.stat` | Throttle count | `nr_throttled 42` |
| `cpu.stat` | Throttled time | `throttled_usec 987654` |

### PIDs Monitoring

| File | Contents | Example |
|------|----------|---------|
| `pids.current` | Current process count | `3` |
| `pids.max` | Configured limit | `50` |
| `pids.events` | Max-reached events | `max 0` |

### Combined Monitoring Script

```bash
#!/bin/bash
# monitor-cgroup.sh - Watch a cgroup's resource usage

CGROUP="${1:-my-container}"
CGROUP_PATH="/sys/fs/cgroup/$CGROUP"

if [ ! -d "$CGROUP_PATH" ]; then
    echo "Error: Cgroup '$CGROUP' does not exist"
    exit 1
fi

while true; do
    clear
    echo "=== Cgroup: $CGROUP ==="
    echo "$(date)"
    echo ""

    echo "--- Memory ---"
    printf "  current/max: %s / %s bytes\n" \
        "$(cat $CGROUP_PATH/memory.current)" \
        "$(cat $CGROUP_PATH/memory.max)"

    echo ""
    echo "--- CPU ---"
    printf "  quota: %s\n" "$(cat $CGROUP_PATH/cpu.max)"
    grep -E "usage_usec|nr_throttled|throttled_usec" $CGROUP_PATH/cpu.stat | \
        sed 's/^/  /'

    echo ""
    echo "--- PIDs ---"
    printf "  current/max: %s / %s\n" \
        "$(cat $CGROUP_PATH/pids.current)" \
        "$(cat $CGROUP_PATH/pids.max)"

    echo ""
    echo "--- Processes ---"
    cat $CGROUP_PATH/cgroup.procs | head -5

    sleep 2
done
```

## Clean Up

```bash
# Kill any test processes in the cgroup first
for pid in $(cat /sys/fs/cgroup/my-container/cgroup.procs 2>/dev/null); do
    kill $pid 2>/dev/null
done

# Wait a moment for processes to exit
sleep 1

# Delete the cgroup
sudo cargo run -q -p cgroup-tool -- delete my-container

# Verify deletion
ls /sys/fs/cgroup/my-container 2>&1 || echo "Cgroup deleted successfully"
```

## Common Errors

1. **"Device or resource busy" when deleting cgroup**
   - Cause: Processes still attached or child cgroups exist
   - Fix: Kill all processes first (`cat cgroup.procs` to find them), remove child cgroups recursively

2. **Controller not available (e.g., "No such file: cpu.max")**
   - Cause: Controller not enabled in `cgroup.subtree_control` of parent
   - Fix: Enable with `echo "+cpu" | sudo tee /sys/fs/cgroup/cgroup.subtree_control`

3. **"Invalid argument" when writing to controller file**
   - Cause: Wrong format (e.g., missing space in cpu.max)
   - Fix: Check format -- `memory.max` wants bytes as integer, `cpu.max` wants "quota period"

4. **Process not constrained after attaching**
   - Cause: Attached to wrong cgroup level or controller not propagating
   - Fix: Verify `/proc/<pid>/cgroup` shows correct path; check `cgroup.subtree_control`

5. **OOM killer terminates wrong process**
   - Cause: Memory limit too low or memory.oom.group not set
   - Fix: Increase limit or set `memory.oom.group` to kill all processes in cgroup together

## How Limits Interact

Understanding how limits affect each other is crucial for production tuning:

### Memory Pressure + CPU Throttling

When memory is near the limit, the kernel reclaims pages, which consumes CPU cycles. A tight CPU limit can make memory reclaim slower, potentially leading to OOM before reclaim completes.

**Recommendation**: Set memory.high below memory.max to trigger early throttling:
```bash
# Soft limit at 80MB, hard limit at 100MB
echo 83886080 | sudo tee /sys/fs/cgroup/my-container/memory.high
echo 104857600 | sudo tee /sys/fs/cgroup/my-container/memory.max
```

### PIDs + Memory

Each process consumes kernel memory for its task_struct (several KB). A high PIDs limit with low memory can cause OOM from kernel memory alone.

**Recommendation**: Ensure memory limit can support your PIDs limit:
```bash
# Rule of thumb: at least 1MB per allowed process for safety
# 50 processes * ~1MB = 50MB minimum memory
```

### CPU + PIDs

Many processes competing for limited CPU means each gets a tiny slice. This can appear as "hanging" even though processes are running.

**Recommendation**: Balance PIDs with CPU quota:
```bash
# If allowing 50 processes but only 25% CPU, each process averages 0.5% CPU
# Consider if this is sufficient for your workload
```

## Notes

- **Cgroup v2 only**: This lesson uses the unified hierarchy; cgroup v1 works differently
- **Kernel version matters**: Some features (like `memory.high`) require kernel 4.5+
- **Container runtimes do more**: Docker/Podman also configure I/O limits, device access, and freeze/thaw capabilities
- **systemd integration**: On systemd systems, be aware of the default cgroup hierarchy managed by systemd

### Production Defaults (Reference)

Here are typical limits used by container runtimes:

| Resource | Docker Default | Kubernetes Default |
|----------|----------------|-------------------|
| Memory | Unlimited | Unlimited (requires request/limit) |
| CPU | Unlimited | Unlimited (requires request/limit) |
| PIDs | Unlimited | 4096 (configurable) |

### Further Reading

- `man 7 cgroups` - Linux cgroups manual
- [Kernel cgroup v2 documentation](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html)
- [Docker resource constraints](https://docs.docker.com/config/containers/resource_constraints/)

## What You Have Built

Congratulations! You have now implemented a complete cgroup v2 toolkit with:

- **Create/Delete**: Cgroup lifecycle management
- **Attach**: Process assignment to cgroups
- **Memory limits**: OOM protection via `memory.max`
- **CPU limits**: Fair scheduling via `cpu.max`
- **PIDs limits**: Fork bomb protection via `pids.max`
- **Combined limits**: Container-like resource bundles

This is the same foundation that Docker, Podman, containerd, and Kubernetes use to implement resource isolation.

## Next

Move to OCI/runc lessons to see how container runtimes use both namespaces and cgroups together:

**`../03-runc/01-oci-bundle.md`** - Create the standard OCI bundle format that runc uses to launch containers. You will learn how namespaces, cgroups, and filesystem isolation combine into a single `config.json` specification.

The OCI lessons will bring together everything from both the namespace and cgroup sections into a complete container runtime workflow.
