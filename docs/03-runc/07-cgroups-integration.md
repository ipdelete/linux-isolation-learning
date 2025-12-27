# 07 Cgroups Integration

## Goal

Attach an OCI container to resource limits using cgroups v2. You will learn how runc handles cgroups via `config.json`, explore two integration approaches (runc-managed vs pre-made cgroups), and connect the cgroup tools you built in Section 02 with the OCI runtime workflow. This final lesson synthesizes everything you have learned about namespaces, cgroups, and OCI specifications into a complete container runtime picture.

**Estimated time**: 60-90 minutes

## Prereqs

- Completed `06-network-integration.md`
- Completed cgroups lessons `docs/02-cgroups/01-cgv2-basics.md` through `06-multi-resource.md`
- Working `cgroup-tool` with `create`, `attach`, `delete`, `memory-max`, `cpu-max`, and `pids-max` subcommands
- `runc` installed on the system
- `sudo` access for cgroup and container operations
- An OCI bundle with rootfs (from earlier lessons)

## Background: How Container Runtimes Use Cgroups

Real container runtimes like Docker, Podman, and containerd use cgroups to enforce resource limits on containers. The OCI runtime specification defines a standard way to express these limits in `config.json`, and runc (the reference OCI runtime) translates these settings into cgroup v2 operations.

**The two approaches to cgroup management:**

| Approach | How It Works | When to Use |
|----------|--------------|-------------|
| **runc-managed** | runc creates and configures the cgroup based on `linux.resources` in config.json | Simple deployments, single containers |
| **Pre-made cgroup** | You create and configure the cgroup first, runc uses `cgroupsPath` to attach | Advanced orchestration, custom hierarchies, Kubernetes-style pod cgroups |

Both approaches result in the same outcome: a container process running inside a cgroup with enforced limits. The difference is who creates and configures the cgroup.

## Understanding the OCI Cgroup Configuration

The OCI runtime specification defines cgroup settings in the `linux` section of `config.json`. Here are the key fields:

### Resource Limits (linux.resources)

```json
{
  "linux": {
    "resources": {
      "memory": {
        "limit": 52428800,
        "reservation": 41943040
      },
      "cpu": {
        "quota": 50000,
        "period": 100000,
        "shares": 1024
      },
      "pids": {
        "limit": 20
      }
    }
  }
}
```

### OCI to Cgroup v2 Mapping

| OCI Field | Cgroup v2 File | Example Value | Meaning |
|-----------|---------------|---------------|---------|
| `memory.limit` | `memory.max` | `52428800` | Hard memory limit (bytes) |
| `memory.reservation` | `memory.low` | `41943040` | Memory reservation (soft limit) |
| `cpu.quota` / `cpu.period` | `cpu.max` | `50000 100000` | CPU bandwidth limit |
| `cpu.shares` | `cpu.weight` | `1024` -> `39` | Relative CPU weight (converted) |
| `pids.limit` | `pids.max` | `20` | Maximum number of processes |

**Note on cpu.shares conversion:** OCI uses cgroup v1 terminology (`shares`, range 2-262144, default 1024). Runc converts this to cgroup v2 `cpu.weight` (range 1-10000, default 100) using the formula: `weight = 1 + ((shares - 2) * 9999) / 262142`.

### Custom Cgroup Path (cgroupsPath)

```json
{
  "linux": {
    "cgroupsPath": "/my-containers/container1"
  }
}
```

When `cgroupsPath` is specified:
- Runc does NOT create the cgroup automatically
- The cgroup must already exist with appropriate permissions
- Runc attaches the container process to this existing cgroup
- Resource limits should already be configured (or can be combined with `linux.resources`)

## Exercises

This lesson uses an exercise-based approach since you are integrating existing tools rather than building new ones.

### Exercise 1: Inspect runc Cgroup Behavior

First, let us understand what runc does with cgroups by default.

**Step 1: Generate a default OCI spec**

```bash
cd /tmp
mkdir -p runc-cgroup-test/rootfs

# Generate default config.json
cd runc-cgroup-test
runc spec

# View the cgroup-related settings
grep -A 50 '"linux"' config.json | grep -A 30 '"resources"'
```

You should see default resource settings (or empty sections depending on runc version).

**Step 2: Check where runc creates cgroups**

```bash
# List existing container cgroups (if any)
ls /sys/fs/cgroup/ | grep -E 'user|system|container' || echo "No container cgroups yet"

# Check the systemd cgroup hierarchy (common on systemd systems)
ls /sys/fs/cgroup/system.slice/ 2>/dev/null | head -5 || echo "No system.slice"
```

**Key insight:** On systemd-based systems, runc typically creates cgroups under `/sys/fs/cgroup/system.slice/` or uses a path specified in config.json.

### Exercise 2: Run Container with runc-Managed Cgroups

**Step 1: Prepare a minimal rootfs**

```bash
cd /tmp/runc-cgroup-test

# Create a minimal rootfs using busybox
mkdir -p rootfs/bin rootfs/proc rootfs/sys

# Copy busybox (adjust path as needed for your system)
if [ -f /bin/busybox ]; then
    cp /bin/busybox rootfs/bin/
    # Create minimal symlinks
    cd rootfs/bin
    for cmd in sh ls cat sleep ps; do
        ln -sf busybox $cmd
    done
    cd /tmp/runc-cgroup-test
else
    echo "busybox not found - install it or use an alternative rootfs"
fi
```

**Step 2: Configure resource limits**

Edit `config.json` and update the `linux.resources` section. Find the existing `"linux": {` section and ensure it contains:

```json
{
  "linux": {
    "resources": {
      "memory": {
        "limit": 52428800
      },
      "cpu": {
        "quota": 50000,
        "period": 100000
      },
      "pids": {
        "limit": 20
      }
    },
    "namespaces": [
      {"type": "pid"},
      {"type": "mount"}
    ]
  }
}
```

This sets:
- 50 MB memory limit
- 50% CPU (50000 microseconds out of every 100000)
- Maximum 20 processes

**Step 3: Update the process command**

Find the `"process": {` section and update `"args"` to run a shell:

```json
{
  "process": {
    "terminal": true,
    "args": ["/bin/sh"],
    "cwd": "/"
  }
}
```

**Step 4: Run the container**

```bash
cd /tmp/runc-cgroup-test
sudo runc run cgroup-test-container
```

If successful, you will get a shell inside the container. Keep this running.

**Step 5: Verify cgroup limits (from another terminal)**

```bash
# Find where runc created the cgroup
sudo find /sys/fs/cgroup -name "*cgroup-test*" -type d 2>/dev/null

# Or check the container process's cgroup
CONTAINER_PID=$(sudo runc ps cgroup-test-container | tail -1 | awk '{print $2}')
cat /proc/$CONTAINER_PID/cgroup

# Read the applied limits
CGROUP_PATH=$(cat /proc/$CONTAINER_PID/cgroup | cut -d: -f3)
echo "Memory limit: $(cat /sys/fs/cgroup$CGROUP_PATH/memory.max)"
echo "CPU max: $(cat /sys/fs/cgroup$CGROUP_PATH/cpu.max)"
echo "PIDs max: $(cat /sys/fs/cgroup$CGROUP_PATH/pids.max)"
```

**Step 6: Test the memory limit (inside container)**

```bash
# Inside the container shell:
# Try to allocate more than 50MB
# This should trigger OOM
dd if=/dev/zero of=/dev/null bs=60M count=1
```

Watch the container - it should be killed by the OOM killer.

**Step 7: Clean up**

```bash
# If the container is still running:
sudo runc kill cgroup-test-container SIGKILL
sudo runc delete cgroup-test-container
```

### Exercise 3: Use Pre-Made Cgroups with cgroup-tool

This approach mirrors what Kubernetes does - the orchestrator creates and configures cgroups, then the container runtime attaches containers to them.

**Step 1: Create the cgroup hierarchy with cgroup-tool**

```bash
# Create parent cgroup for all our containers
sudo cargo run -q -p cgroup-tool -- create my-containers

# Create specific container cgroup
sudo cargo run -q -p cgroup-tool -- create my-containers/container1

# Enable controllers for the parent (required for children to use them)
echo "+memory +cpu +pids" | sudo tee /sys/fs/cgroup/my-containers/cgroup.subtree_control

# Apply resource limits using your cgroup-tool
sudo cargo run -q -p cgroup-tool -- memory-max my-containers/container1 52428800
sudo cargo run -q -p cgroup-tool -- cpu-max my-containers/container1 "50000 100000"
sudo cargo run -q -p cgroup-tool -- pids-max my-containers/container1 20
```

**Step 2: Verify the cgroup is ready**

```bash
echo "=== Pre-made Cgroup Configuration ==="
echo "memory.max: $(cat /sys/fs/cgroup/my-containers/container1/memory.max)"
echo "cpu.max: $(cat /sys/fs/cgroup/my-containers/container1/cpu.max)"
echo "pids.max: $(cat /sys/fs/cgroup/my-containers/container1/pids.max)"
```

**Step 3: Update config.json to use the pre-made cgroup**

Edit `/tmp/runc-cgroup-test/config.json`. In the `linux` section, add `cgroupsPath`:

```json
{
  "linux": {
    "cgroupsPath": "/my-containers/container1",
    "namespaces": [
      {"type": "pid"},
      {"type": "mount"}
    ]
  }
}
```

**Important:** When using `cgroupsPath`, you can optionally keep `linux.resources` (runc will try to apply them) or remove it (use only the pre-configured limits).

**Step 4: Run the container**

```bash
cd /tmp/runc-cgroup-test
sudo runc run premade-cgroup-test
```

**Step 5: Verify the container is in your cgroup**

From another terminal:

```bash
# Check which PIDs are in your pre-made cgroup
cat /sys/fs/cgroup/my-containers/container1/cgroup.procs

# Verify from the process perspective
CONTAINER_PID=$(sudo runc ps premade-cgroup-test 2>/dev/null | tail -1 | awk '{print $2}')
cat /proc/$CONTAINER_PID/cgroup
# Should show: 0::/my-containers/container1
```

**Step 6: Monitor resource usage**

```bash
echo "=== Container Resource Usage ==="
echo "memory.current: $(cat /sys/fs/cgroup/my-containers/container1/memory.current) bytes"
echo "pids.current: $(cat /sys/fs/cgroup/my-containers/container1/pids.current)"
cat /sys/fs/cgroup/my-containers/container1/cpu.stat | head -3
```

**Step 7: Clean up**

```bash
# Kill and delete the container
sudo runc kill premade-cgroup-test SIGKILL 2>/dev/null
sudo runc delete premade-cgroup-test 2>/dev/null

# Delete the cgroups (children before parents)
sudo cargo run -q -p cgroup-tool -- delete my-containers/container1
sudo cargo run -q -p cgroup-tool -- delete my-containers
```

### Exercise 4: Test PIDs Limit Enforcement

**Step 1: Create a cgroup with a tight PIDs limit**

```bash
sudo cargo run -q -p cgroup-tool -- create pids-test
echo "+pids" | sudo tee /sys/fs/cgroup/pids-test/cgroup.subtree_control 2>/dev/null || true
sudo cargo run -q -p cgroup-tool -- pids-max pids-test 5
```

**Step 2: Create a test config.json**

```bash
mkdir -p /tmp/pids-test-bundle/rootfs/bin
cp /bin/busybox /tmp/pids-test-bundle/rootfs/bin/ 2>/dev/null || echo "Copy busybox manually"
cd /tmp/pids-test-bundle/rootfs/bin && ln -sf busybox sh && cd /tmp/pids-test-bundle

cat > /tmp/pids-test-bundle/config.json << 'EOF'
{
  "ociVersion": "1.0.0",
  "process": {
    "terminal": true,
    "user": {"uid": 0, "gid": 0},
    "args": ["/bin/sh"],
    "cwd": "/"
  },
  "root": {"path": "rootfs", "readonly": false},
  "linux": {
    "cgroupsPath": "/pids-test",
    "namespaces": [
      {"type": "pid"},
      {"type": "mount"}
    ]
  }
}
EOF
```

**Step 3: Run and test**

```bash
cd /tmp/pids-test-bundle
sudo runc run pids-limited-container
```

Inside the container, try to spawn multiple processes:

```bash
# This should fail when you hit the limit
for i in 1 2 3 4 5 6 7; do
    sleep 100 &
    echo "Started background process $i"
done
```

You should see fork failures after reaching the PIDs limit.

**Step 4: Clean up**

```bash
sudo runc kill pids-limited-container SIGKILL 2>/dev/null
sudo runc delete pids-limited-container 2>/dev/null
sudo cargo run -q -p cgroup-tool -- delete pids-test
rm -rf /tmp/pids-test-bundle
```

## Verify

### Automated Verification Checklist

Run through these checks to verify your understanding:

```bash
#!/bin/bash
# verification-07-cgroups.sh

echo "=== Cgroup Integration Verification ==="

# Check 1: cgroup v2 is available
echo -n "1. Cgroup v2 mounted: "
if mount | grep -q "cgroup2 on /sys/fs/cgroup"; then
    echo "PASS"
else
    echo "FAIL - cgroup v2 not mounted"
fi

# Check 2: Required controllers available
echo -n "2. Controllers available: "
CONTROLLERS=$(cat /sys/fs/cgroup/cgroup.controllers)
if echo "$CONTROLLERS" | grep -q "memory" && \
   echo "$CONTROLLERS" | grep -q "cpu" && \
   echo "$CONTROLLERS" | grep -q "pids"; then
    echo "PASS ($CONTROLLERS)"
else
    echo "FAIL - missing controllers"
fi

# Check 3: cgroup-tool works
echo -n "3. cgroup-tool functional: "
if sudo cargo run -q -p cgroup-tool -- create verify-test 2>/dev/null && \
   sudo cargo run -q -p cgroup-tool -- delete verify-test 2>/dev/null; then
    echo "PASS"
else
    echo "FAIL"
fi

# Check 4: runc available
echo -n "4. runc installed: "
if command -v runc &>/dev/null; then
    echo "PASS ($(runc --version | head -1))"
else
    echo "FAIL - runc not found"
fi

echo ""
echo "=== Verification Complete ==="
```

### Manual Verification

**Verify OCI config parsing:**

```bash
# Generate default spec and check cgroup fields
runc spec --rootless 2>/dev/null || runc spec
grep -E "(cgroupsPath|resources|memory|cpu|pids)" config.json
```

**Verify limit enforcement:**

```bash
# Create a cgroup and verify limits are readable
sudo cargo run -q -p cgroup-tool -- create test-verify
sudo cargo run -q -p cgroup-tool -- memory-max test-verify 50000000
cat /sys/fs/cgroup/test-verify/memory.max
# Should output: 50000000
sudo cargo run -q -p cgroup-tool -- delete test-verify
```

## Clean Up

Remove all test artifacts:

```bash
# Remove any leftover containers
sudo runc list 2>/dev/null | tail -n +2 | awk '{print $1}' | while read id; do
    sudo runc kill "$id" SIGKILL 2>/dev/null
    sudo runc delete "$id" 2>/dev/null
done

# Remove test cgroups (handles nested cleanup)
for cg in /sys/fs/cgroup/my-containers/container1 \
          /sys/fs/cgroup/my-containers \
          /sys/fs/cgroup/pids-test \
          /sys/fs/cgroup/test-verify; do
    if [ -d "$cg" ]; then
        # Move any remaining processes to root cgroup
        cat "$cg/cgroup.procs" 2>/dev/null | while read pid; do
            echo "$pid" | sudo tee /sys/fs/cgroup/cgroup.procs >/dev/null 2>&1
        done
        sudo rmdir "$cg" 2>/dev/null && echo "Removed: $cg"
    fi
done

# Remove test bundle directories
rm -rf /tmp/runc-cgroup-test /tmp/pids-test-bundle
```

## Common Errors

1. **"cgroup path does not exist" when using cgroupsPath**
   - Cause: The pre-made cgroup was not created before running the container
   - Fix: Create the cgroup first using `cgroup-tool create <path>` or `mkdir`
   - Verify: `ls /sys/fs/cgroup/<your-cgroup-path>`

2. **"cannot write to cgroup" or "operation not permitted"**
   - Cause: Controller not enabled in parent's `cgroup.subtree_control`
   - Fix: Enable controllers in parent: `echo "+memory +cpu +pids" | sudo tee /sys/fs/cgroup/<parent>/cgroup.subtree_control`
   - Note: The "no internal processes" rule may require restructuring

3. **OOM kills without memory limit taking effect**
   - Cause: Container process is in the wrong cgroup or wrong hierarchy level
   - Fix: Verify process cgroup with `cat /proc/<pid>/cgroup`
   - Check: Ensure `memory.max` in the correct cgroup shows your limit

4. **"operation not permitted" with cpu.max or memory.max**
   - Cause: Cgroup v1 vs v2 confusion - file names differ
   - cgroup v1: `cpu.cfs_quota_us`, `memory.limit_in_bytes`
   - cgroup v2: `cpu.max`, `memory.max`
   - Fix: Verify you are using cgroup v2: `mount | grep cgroup2`

5. **"Device or resource busy" when deleting cgroup**
   - Cause: Container processes still attached or child cgroups exist
   - Fix: Kill container first (`runc kill <id> SIGKILL`), then delete cgroup
   - Check: `cat /sys/fs/cgroup/<path>/cgroup.procs` should be empty

6. **runc creates cgroup in unexpected location**
   - Cause: systemd cgroup driver vs cgroupfs driver
   - On systemd systems, runc may use `/sys/fs/cgroup/system.slice/runc-<id>.scope`
   - Fix: Use explicit `cgroupsPath` in config.json for predictable placement

7. **CPU quota not limiting as expected**
   - Cause: Quota/period values misunderstood
   - Format: `quota period` (e.g., `50000 100000` = 50% of one CPU)
   - For multi-CPU: `200000 100000` = 200% = 2 full CPUs
   - Fix: Verify with `cat /sys/fs/cgroup/<path>/cpu.max`

## Notes

### How runc Detects Cgroup Version

Runc automatically detects whether the system uses cgroup v1 or v2:

```bash
# Check what runc sees
sudo runc --debug run --bundle /tmp/test test 2>&1 | grep -i cgroup
```

On cgroup v2 systems:
- Single hierarchy at `/sys/fs/cgroup`
- All controllers share the same tree
- Resource settings use unified syntax (`memory.max`, `cpu.max`, `pids.max`)

### The Complete Picture: Namespaces + Cgroups + OCI

This lesson completes the container isolation story:

| Component | What It Provides | OCI Config Section |
|-----------|------------------|-------------------|
| **PID Namespace** | Process isolation | `linux.namespaces` with `type: pid` |
| **Mount Namespace** | Filesystem isolation | `linux.namespaces` with `type: mount` |
| **Network Namespace** | Network isolation | `linux.namespaces` with `type: network` |
| **User Namespace** | UID/GID isolation | `linux.namespaces` with `type: user` |
| **Cgroups** | Resource limits | `linux.resources` and `linux.cgroupsPath` |
| **Seccomp** | Syscall filtering | `linux.seccomp` |
| **Rootfs** | Container filesystem | `root.path` |

Real containers combine ALL of these. The OCI specification (`config.json`) is the declarative interface that ties them together, and runc is the runtime that interprets this specification and creates the isolated environment.

### Docker and Kubernetes Use the Same Mechanisms

When you run `docker run --memory=50m --cpus=0.5 myimage`:

1. Docker generates an OCI `config.json` with memory/cpu limits
2. Docker calls containerd, which calls runc
3. Runc creates the cgroup (or uses a pre-made one from the orchestrator)
4. Runc writes resource limits to cgroup files
5. Runc forks and attaches the container process to the cgroup

Kubernetes does similar work through the kubelet and container runtime interface (CRI).

### Production Considerations

| Aspect | Development | Production |
|--------|-------------|------------|
| Cgroup management | runc-managed is simpler | Pre-made for orchestrator control |
| Memory limits | Test OOM behavior | Add memory.high for graceful degradation |
| CPU limits | Simple quota/period | Consider cpu.weight for relative priority |
| PIDs limit | Low limit for testing | Higher limits (1000+) for real workloads |
| Monitoring | Manual inspection | Prometheus/cAdvisor integration |

## Summary: What You Have Built

Congratulations! You have completed the OCI/runc section and the core learning path. You now understand:

1. **OCI Bundle Structure** - The standard format for container images and configuration
2. **config.json** - The declarative specification for container properties
3. **Container Lifecycle** - How runc creates, runs, and destroys containers
4. **Seccomp Integration** - Syscall filtering for security
5. **Network Integration** - How containers connect to networks via namespaces
6. **Cgroups Integration** - Resource limits via cgroups v2

These are the same building blocks that power Docker, Podman, containerd, and Kubernetes.

## Next

Congratulations on completing the core learning path!

For reference material and troubleshooting help, see the appendix:

**`../90-appendix/01-rust-syscall-cheatsheet.md`** - Quick reference for system calls and their Rust bindings

**`../90-appendix/02-troubleshooting.md`** - Common issues and solutions

You are now equipped to:
- Read and understand container runtime source code
- Debug container isolation issues at the kernel level
- Build custom container tooling with Rust
- Contribute to container ecosystem projects
