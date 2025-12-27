# Learning runc (OCI Runtime)

A comprehensive guide to understanding and using runc, the reference implementation of the OCI Runtime Specification.

## What is runc?

**runc** is a CLI tool for spawning and running containers according to the Open Container Initiative (OCI) specification. It's the low-level runtime that Docker, Podman, and Kubernetes use under the hood.

**Think of it as**: The "assembly language" of containers - it directly uses Linux kernel features (namespaces, cgroups, seccomp) without the complexity of Docker.

---

## Prerequisites

- Completed namespace learning (01-namespaces.md)
- Completed cgroup learning (02-cgroups.md)
- Linux system (Ubuntu 20.04+, Fedora, or similar)
- Root access (or sudo)
- Understanding of:
  - All 7 namespace types
  - Cgroups v2 controllers
  - JSON format
  - Filesystem concepts (mount, chroot, pivot_root)

---

## Phase 1: Understanding runc and OCI (2-3 hours)

### 1.1 The Container Stack

```
┌─────────────────────────────────┐
│   Docker / Podman / CRI-O       │  High-level runtime
│   (image management, networks)   │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│   containerd / cri-o            │  Mid-level runtime
│   (lifecycle, state management)  │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│          runc                    │  Low-level runtime (OCI)
│   (create/run containers)        │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│    Linux Kernel                  │
│   (namespaces, cgroups, seccomp) │
└─────────────────────────────────┘
```

**runc's role:**
- Creates namespaces
- Sets up cgroups
- Applies seccomp filters
- Manages container lifecycle
- Does NOT handle: images, networks, volumes

### 1.2 OCI Specifications

Three main OCI specs:

| Spec | Purpose | What it Defines |
|------|---------|-----------------|
| **Runtime Spec** | Container execution | config.json format, lifecycle operations |
| **Image Spec** | Container images | Layer format, manifest structure |
| **Distribution Spec** | Image distribution | Registry API, content addressing |

**runc implements**: Runtime Spec only

### 1.3 OCI Bundle

An OCI bundle is a directory containing:

```
my-container/
├── config.json      # OCI runtime configuration
└── rootfs/         # Root filesystem
    ├── bin/
    ├── etc/
    ├── lib/
    ├── usr/
    └── ...
```

**Key concept**: runc doesn't care about images - it only needs a bundle!

---

## Phase 2: Installing and Setting Up runc (1-2 hours)

### 2.1 Installation

**Option 1: Package manager (recommended)**

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install runc

# Fedora
sudo dnf install runc

# Arch Linux
sudo pacman -S runc
```

**Option 2: Build from source**

```bash
# Install Go
sudo apt-get install golang

# Clone runc
git clone https://github.com/opencontainers/runc
cd runc

# Build
make

# Install
sudo make install

# Verify
runc --version
```

### 2.2 Verify Installation

```bash
runc --version
# Output: runc version 1.1.x
#         spec: 1.0.2-dev

# Check available commands
runc --help
```

### 2.3 Key runc Commands

| Command | Purpose |
|---------|---------|
| `runc run` | Create and start container (one step) |
| `runc create` | Create container (don't start yet) |
| `runc start` | Start a created container |
| `runc kill` | Send signal to container |
| `runc delete` | Delete container |
| `runc state` | Get container state |
| `runc list` | List containers |
| `runc exec` | Execute command in running container |
| `runc spec` | Generate default config.json |

---

## Phase 3: Your First Container (Hands-on: 2-3 hours)

### 3.1 Create a Minimal Rootfs

**Option 1: Using Docker (easiest)**

```bash
# Create directory for bundle
mkdir -p ~/containers/hello-container
cd ~/containers/hello-container

# Export a minimal Docker image as rootfs
docker export $(docker create busybox) | tar -C rootfs -xf -

# Verify
ls rootfs/
# Should see: bin  dev  etc  home  lib  proc  root  sys  tmp  usr  var
```

**Option 2: Using debootstrap (Ubuntu)**

```bash
mkdir -p ~/containers/hello-container/rootfs
cd ~/containers/hello-container

# Install debootstrap
sudo apt-get install debootstrap

# Create minimal Ubuntu rootfs
sudo debootstrap --variant=minbase focal rootfs http://archive.ubuntu.com/ubuntu/

# Fix permissions
sudo chown -R $USER:$USER rootfs
```

**Option 3: Manual minimal rootfs**

```bash
mkdir -p ~/containers/hello-container/rootfs/{bin,lib,lib64}
cd ~/containers/hello-container

# Copy busybox (provides many utilities)
cp /bin/busybox rootfs/bin/

# Create symlinks
for cmd in sh ls cat echo; do
    ln -s busybox rootfs/bin/$cmd
done

# Copy required libraries
cp /lib/x86_64-linux-gnu/libc.so.6 rootfs/lib/
cp /lib64/ld-linux-x86-64.so.2 rootfs/lib64/
```

### 3.2 Generate config.json

```bash
cd ~/containers/hello-container

# Generate default config
runc spec

# This creates config.json
ls -l config.json
```

### 3.3 Understand config.json Structure

```json
{
  "ociVersion": "1.0.2-dev",
  "process": {
    "terminal": true,
    "user": {"uid": 0, "gid": 0},
    "args": ["sh"],
    "env": [
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
      "TERM=xterm"
    ],
    "cwd": "/",
    "capabilities": {...},
    "rlimits": [...]
  },
  "root": {
    "path": "rootfs",
    "readonly": true
  },
  "hostname": "runc",
  "mounts": [...],
  "linux": {
    "namespaces": [...],
    "resources": {...}
  }
}
```

**Key sections:**
- `process`: Command to run, user, env vars
- `root`: Path to rootfs
- `mounts`: Filesystems to mount
- `linux.namespaces`: Which namespaces to create
- `linux.resources`: Cgroup limits

### 3.4 Run Your First Container

```bash
cd ~/containers/hello-container

# Run container (requires root)
sudo runc run my-first-container

# You're now inside the container!
# Try some commands:
hostname  # Should show 'runc'
ps aux    # Only see container processes
ls /      # See container's rootfs
exit      # Exit container
```

**What just happened?**
1. runc created 7 namespaces (PID, mount, network, IPC, UTS, user, cgroup)
2. Set up cgroups with default limits
3. Executed `sh` as PID 1 inside container
4. Cleaned up when you exited

### 3.5 Run Container in Background

```bash
# Modify config.json: set terminal to false and change command
cat > config.json << 'EOF'
{
  "ociVersion": "1.0.2-dev",
  "process": {
    "terminal": false,
    "user": {"uid": 0, "gid": 0},
    "args": ["sleep", "300"],
    "env": ["PATH=/usr/bin:/bin"],
    "cwd": "/"
  },
  "root": {
    "path": "rootfs",
    "readonly": true
  },
  "hostname": "background-container",
  "linux": {
    "namespaces": [
      {"type": "pid"},
      {"type": "network"},
      {"type": "ipc"},
      {"type": "uts"},
      {"type": "mount"}
    ]
  }
}
EOF

# Run in background
sudo runc run -d background-container

# List running containers
sudo runc list
# Output:
# ID                    PID         STATUS      BUNDLE                    CREATED
# background-container  12345       running     /home/.../hello-container 2024-01-01T12:00:00Z

# Check container state
sudo runc state background-container

# Kill container
sudo runc kill background-container SIGKILL

# Delete container
sudo runc delete background-container
```

---

## Phase 4: Container Configuration Deep Dive (4-5 hours)

### 4.1 Process Configuration

**Terminal vs Non-terminal:**

```json
{
  "process": {
    "terminal": true,   // Interactive shell
    "terminal": false,  // Background daemon
    ...
  }
}
```

**User and Groups:**

```json
{
  "process": {
    "user": {
      "uid": 1000,           // Run as user 1000
      "gid": 1000,           // Primary group 1000
      "additionalGids": [10, 20]  // Additional groups
    }
  }
}
```

**Environment Variables:**

```json
{
  "process": {
    "env": [
      "PATH=/usr/bin:/bin",
      "HOME=/root",
      "MY_VAR=value"
    ]
  }
}
```

**Working Directory:**

```json
{
  "process": {
    "cwd": "/app"  // Start in /app directory
  }
}
```

### 4.2 Root Filesystem

```json
{
  "root": {
    "path": "rootfs",        // Relative or absolute path
    "readonly": true         // Mount rootfs read-only
  }
}
```

**Exercise: Read-only rootfs with tmpfs /tmp**

```json
{
  "root": {
    "path": "rootfs",
    "readonly": true
  },
  "mounts": [
    {
      "destination": "/tmp",
      "type": "tmpfs",
      "source": "tmpfs",
      "options": ["nosuid", "nodev", "mode=1777"]
    }
  ]
}
```

### 4.3 Mounts

**Standard mounts:**

```json
{
  "mounts": [
    {
      "destination": "/proc",
      "type": "proc",
      "source": "proc"
    },
    {
      "destination": "/dev",
      "type": "tmpfs",
      "source": "tmpfs",
      "options": ["nosuid", "strictatime", "mode=755", "size=65536k"]
    },
    {
      "destination": "/sys",
      "type": "sysfs",
      "source": "sysfs",
      "options": ["nosuid", "noexec", "nodev", "ro"]
    }
  ]
}
```

**Bind mounts (share host directory):**

```json
{
  "mounts": [
    {
      "destination": "/data",
      "type": "bind",
      "source": "/host/data",
      "options": ["rbind", "rw"]
    }
  ]
}
```

### 4.4 Namespaces

**All namespace types:**

```json
{
  "linux": {
    "namespaces": [
      {"type": "pid"},       // PID isolation
      {"type": "network"},   // Network isolation
      {"type": "ipc"},       // IPC isolation
      {"type": "uts"},       // Hostname isolation
      {"type": "mount"},     // Mount isolation
      {"type": "cgroup"},    // Cgroup isolation
      {"type": "user"}       // User ID remapping
    ]
  }
}
```

**Join existing namespace:**

```json
{
  "linux": {
    "namespaces": [
      {
        "type": "network",
        "path": "/var/run/netns/my-network"
      }
    ]
  }
}
```

### 4.5 Cgroups (Resource Limits)

**Memory limits:**

```json
{
  "linux": {
    "resources": {
      "memory": {
        "limit": 134217728,      // 128 MB hard limit
        "reservation": 67108864,  // 64 MB soft limit
        "swap": 0                // No swap
      }
    }
  }
}
```

**CPU limits:**

```json
{
  "linux": {
    "resources": {
      "cpu": {
        "shares": 1024,        // Relative weight
        "quota": 50000,        // 50% of 1 core
        "period": 100000,      // 100ms period
        "cpus": "0-1"          // Allow CPUs 0 and 1
      }
    }
  }
}
```

**PID limits:**

```json
{
  "linux": {
    "resources": {
      "pids": {
        "limit": 100  // Max 100 processes
      }
    }
  }
}
```

**Block I/O limits:**

```json
{
  "linux": {
    "resources": {
      "blockIO": {
        "weight": 500,  // I/O weight (10-1000)
        "throttleReadBpsDevice": [
          {
            "major": 8,
            "minor": 0,
            "rate": 1048576  // 1 MB/s read limit on /dev/sda
          }
        ],
        "throttleWriteBpsDevice": [
          {
            "major": 8,
            "minor": 0,
            "rate": 1048576  // 1 MB/s write limit
          }
        ]
      }
    }
  }
}
```

### 4.6 Capabilities

Drop dangerous capabilities:

```json
{
  "process": {
    "capabilities": {
      "bounding": [
        "CAP_CHOWN",
        "CAP_DAC_OVERRIDE",
        "CAP_FOWNER",
        "CAP_KILL",
        "CAP_NET_BIND_SERVICE",
        "CAP_SETGID",
        "CAP_SETUID"
      ],
      "effective": ["...same list..."],
      "inheritable": ["...same list..."],
      "permitted": ["...same list..."]
    }
  }
}
```

**Drop ALL capabilities (maximum security):**

```json
{
  "process": {
    "capabilities": {
      "bounding": [],
      "effective": [],
      "inheritable": [],
      "permitted": []
    }
  }
}
```

---

## Phase 5: Advanced Features (4-5 hours)

### 5.1 Seccomp (Syscall Filtering)

Block dangerous syscalls:

```json
{
  "linux": {
    "seccomp": {
      "defaultAction": "SCMP_ACT_ALLOW",
      "architectures": ["SCMP_ARCH_X86_64"],
      "syscalls": [
        {
          "names": ["socket", "bind", "connect"],
          "action": "SCMP_ACT_ERRNO",
          "args": []
        },
        {
          "names": ["open", "openat", "creat"],
          "action": "SCMP_ACT_ERRNO",
          "args": []
        }
      ]
    }
  }
}
```

**Actions:**
- `SCMP_ACT_ALLOW`: Allow syscall
- `SCMP_ACT_ERRNO`: Return error
- `SCMP_ACT_KILL`: Kill process
- `SCMP_ACT_TRAP`: Generate SIGSYS signal

**Test seccomp:**

```bash
# Run container with socket() blocked
sudo runc run blocked-network

# Inside container, try:
python3 -c "import socket; socket.socket()"
# Error: OSError: [Errno 1] Operation not permitted
```

### 5.2 Hooks

Run code at specific lifecycle points:

```json
{
  "hooks": {
    "prestart": [
      {
        "path": "/usr/local/bin/setup-network.sh",
        "args": ["setup", "container-1"],
        "env": ["CONTAINER_ID=container-1"]
      }
    ],
    "poststart": [
      {
        "path": "/usr/local/bin/register-container.sh",
        "args": ["register"]
      }
    ],
    "poststop": [
      {
        "path": "/usr/local/bin/cleanup.sh",
        "args": ["cleanup"]
      }
    ]
  }
}
```

**Hook lifecycle:**

```
create container
    ↓
prestart hooks (container created, not started)
    ↓
start container
    ↓
createRuntime hooks (container started)
    ↓
createContainer hooks
    ↓
startContainer hooks
    ↓
poststart hooks (container running)
    ↓
... container runs ...
    ↓
poststop hooks (container stopped)
```

### 5.3 Masked and Readonly Paths

Hide or protect sensitive paths:

```json
{
  "linux": {
    "maskedPaths": [
      "/proc/acpi",
      "/proc/kcore",
      "/proc/keys",
      "/proc/latency_stats",
      "/sys/firmware"
    ],
    "readonlyPaths": [
      "/proc/asound",
      "/proc/bus",
      "/proc/fs",
      "/proc/irq",
      "/proc/sys",
      "/proc/sysrq-trigger"
    ]
  }
}
```

### 5.4 Devices

Allow access to specific devices:

```json
{
  "linux": {
    "devices": [
      {
        "path": "/dev/null",
        "type": "c",
        "major": 1,
        "minor": 3,
        "fileMode": 438,
        "uid": 0,
        "gid": 0
      },
      {
        "path": "/dev/random",
        "type": "c",
        "major": 1,
        "minor": 8
      }
    ]
  }
}
```

---

## Phase 6: Container Lifecycle Management (3-4 hours)

### 6.1 Create vs Run

**Two-step process:**

```bash
# Step 1: Create container (set up namespaces/cgroups, don't start)
sudo runc create my-container

# Check state
sudo runc state my-container
# Status: created

# Step 2: Start the container
sudo runc start my-container

# Check state again
sudo runc state my-container
# Status: running
```

**One-step process:**

```bash
# Create + start in one command
sudo runc run my-container
```

### 6.2 Container States

```
┌─────────┐
│ created │  (namespaces/cgroups set up, process not started)
└────┬────┘
     │ start
     ▼
┌─────────┐
│ running │  (process executing)
└────┬────┘
     │ kill/exit
     ▼
┌─────────┐
│ stopped │  (process exited, resources not cleaned)
└────┬────┘
     │ delete
     ▼
(removed)
```

### 6.3 Executing Commands in Running Container

```bash
# Start long-running container
sudo runc run -d my-container

# Execute command inside
sudo runc exec my-container ps aux

# Start interactive shell
sudo runc exec -t my-container sh
```

### 6.4 Pausing and Resuming

```bash
# Pause container (freeze all processes)
sudo runc pause my-container

# Check state
sudo runc state my-container
# Status: paused

# Resume
sudo runc resume my-container
```

### 6.5 Killing Containers

```bash
# Send SIGTERM (graceful shutdown)
sudo runc kill my-container TERM

# Send SIGKILL (immediate termination)
sudo runc kill my-container KILL

# Send custom signal
sudo runc kill my-container USR1
```

### 6.6 Container Cleanup

```bash
# Delete stopped container
sudo runc delete my-container

# Force delete (kills if running)
sudo runc delete --force my-container
```

---

## Phase 7: Practical Projects (8-10 hours)

### Project 1: Build a Simple Container Runner

```python
#!/usr/bin/env python3
"""Simple container runner using runc"""

import json
import subprocess
import sys
from pathlib import Path

class Container:
    """Manage runc container lifecycle"""

    def __init__(self, name, bundle_path):
        self.name = name
        self.bundle_path = Path(bundle_path)
        self.config_path = self.bundle_path / "config.json"

    def create_config(self, command, memory_mb=128, cpu_quota=50000):
        """Generate config.json"""
        config = {
            "ociVersion": "1.0.2-dev",
            "process": {
                "terminal": False,
                "user": {"uid": 0, "gid": 0},
                "args": command,
                "env": ["PATH=/usr/bin:/bin"],
                "cwd": "/",
                "capabilities": {
                    "bounding": [],
                    "effective": [],
                    "inheritable": [],
                    "permitted": []
                }
            },
            "root": {
                "path": "rootfs",
                "readonly": True
            },
            "hostname": self.name,
            "mounts": [
                {
                    "destination": "/proc",
                    "type": "proc",
                    "source": "proc"
                },
                {
                    "destination": "/tmp",
                    "type": "tmpfs",
                    "source": "tmpfs",
                    "options": ["nosuid", "nodev", "mode=1777"]
                }
            ],
            "linux": {
                "namespaces": [
                    {"type": "pid"},
                    {"type": "network"},
                    {"type": "ipc"},
                    {"type": "uts"},
                    {"type": "mount"}
                ],
                "resources": {
                    "memory": {
                        "limit": memory_mb * 1024 * 1024
                    },
                    "cpu": {
                        "quota": cpu_quota,
                        "period": 100000
                    }
                },
                "maskedPaths": [
                    "/proc/acpi",
                    "/proc/kcore",
                    "/proc/keys"
                ],
                "readonlyPaths": [
                    "/proc/bus",
                    "/proc/fs",
                    "/proc/irq",
                    "/proc/sys"
                ]
            }
        }

        with open(self.config_path, 'w') as f:
            json.dump(config, f, indent=2)

    def run(self):
        """Run container"""
        subprocess.run([
            "sudo", "runc", "run",
            "--bundle", str(self.bundle_path),
            self.name
        ], check=True)

    def run_detached(self):
        """Run container in background"""
        subprocess.run([
            "sudo", "runc", "run",
            "--bundle", str(self.bundle_path),
            "--detach",
            self.name
        ], check=True)

    def exec(self, command):
        """Execute command in running container"""
        subprocess.run([
            "sudo", "runc", "exec",
            self.name
        ] + command, check=True)

    def kill(self, signal="KILL"):
        """Kill container"""
        subprocess.run([
            "sudo", "runc", "kill",
            self.name,
            signal
        ], check=True)

    def delete(self):
        """Delete container"""
        subprocess.run([
            "sudo", "runc", "delete",
            "--force",
            self.name
        ], check=True)

    def state(self):
        """Get container state"""
        result = subprocess.run([
            "sudo", "runc", "state",
            self.name
        ], capture_output=True, text=True, check=True)
        return json.loads(result.stdout)


# Example usage
if __name__ == '__main__':
    if len(sys.argv) < 4:
        print("Usage: container.py <name> <bundle-path> <command...>")
        sys.exit(1)

    name = sys.argv[1]
    bundle_path = sys.argv[2]
    command = sys.argv[3:]

    container = Container(name, bundle_path)
    container.create_config(command)
    container.run()
```

### Project 2: Container with Networking

Create a container with veth network:

```bash
#!/bin/bash
# run-with-network.sh

CONTAINER_NAME=$1
CONTAINER_IP=$2
BUNDLE_PATH=$3

# Create network namespace
sudo ip netns add $CONTAINER_NAME

# Create veth pair
sudo ip link add veth-$CONTAINER_NAME type veth peer name veth-$CONTAINER_NAME-br

# Move one end to namespace
sudo ip link set veth-$CONTAINER_NAME netns $CONTAINER_NAME

# Configure namespace network
sudo ip netns exec $CONTAINER_NAME ip addr add $CONTAINER_IP/24 dev veth-$CONTAINER_NAME
sudo ip netns exec $CONTAINER_NAME ip link set veth-$CONTAINER_NAME up
sudo ip netns exec $CONTAINER_NAME ip link set lo up
sudo ip netns exec $CONTAINER_NAME ip route add default via ${CONTAINER_IP%.*}.1

# Configure host side
sudo ip link set veth-$CONTAINER_NAME-br master br0
sudo ip link set veth-$CONTAINER_NAME-br up

# Update config.json to use existing network namespace
cd $BUNDLE_PATH
cat config.json | jq '.linux.namespaces = [
  {"type": "pid"},
  {"type": "network", "path": "/var/run/netns/'$CONTAINER_NAME'"},
  {"type": "ipc"},
  {"type": "uts"},
  {"type": "mount"}
]' > config.json.tmp
mv config.json.tmp config.json

# Run container
sudo runc run --bundle $BUNDLE_PATH $CONTAINER_NAME

# Cleanup network
sudo ip netns del $CONTAINER_NAME
```

### Project 3: Container Monitoring Tool

```python
#!/usr/bin/env python3
"""Monitor all runc containers"""

import json
import subprocess
import time

def list_containers():
    """Get list of running containers"""
    result = subprocess.run(
        ["sudo", "runc", "list"],
        capture_output=True,
        text=True,
        check=True
    )

    lines = result.stdout.strip().split('\n')[1:]  # Skip header
    containers = []

    for line in lines:
        if line:
            parts = line.split()
            containers.append({
                'id': parts[0],
                'pid': int(parts[1]),
                'status': parts[2]
            })

    return containers

def get_container_stats(container_id):
    """Get resource usage statistics"""
    # Read from cgroup
    cgroup_base = f"/sys/fs/cgroup/system.slice/runc-{container_id}.scope"

    stats = {}

    try:
        # Memory usage
        with open(f"{cgroup_base}/memory.current") as f:
            stats['memory_bytes'] = int(f.read())

        # CPU usage
        with open(f"{cgroup_base}/cpu.stat") as f:
            for line in f:
                if line.startswith('usage_usec'):
                    stats['cpu_usec'] = int(line.split()[1])

        # PID count
        with open(f"{cgroup_base}/pids.current") as f:
            stats['pids'] = int(f.read())

    except FileNotFoundError:
        pass

    return stats

def monitor():
    """Monitor all containers"""
    print("Container Monitor (Ctrl+C to exit)\n")

    try:
        while True:
            containers = list_containers()

            print(f"\n{'ID':<20} {'PID':<10} {'Status':<10} {'Memory':<15} {'PIDs':<5}")
            print("=" * 70)

            for container in containers:
                stats = get_container_stats(container['id'])

                mem = stats.get('memory_bytes', 0) / 1024 / 1024
                pids = stats.get('pids', 0)

                print(f"{container['id']:<20} "
                      f"{container['pid']:<10} "
                      f"{container['status']:<10} "
                      f"{mem:>10.2f} MB   "
                      f"{pids:<5}")

            time.sleep(2)

    except KeyboardInterrupt:
        print("\n\nExiting...")

if __name__ == '__main__':
    monitor()
```

---

## Phase 8: Integration with Namespaces and Cgroups (4-5 hours)

### 8.1 Manual Namespace + runc

Create namespaces manually, then use runc:

```bash
# Create network namespace
sudo ip netns add test-net

# Create cgroup
sudo mkdir /sys/fs/cgroup/test-cgroup
echo "+cpu +memory" | sudo tee /sys/fs/cgroup/cgroup.subtree_control

# Modify config.json to use existing namespaces
cat > config.json << EOF
{
  "ociVersion": "1.0.2-dev",
  "process": {
    "args": ["sleep", "300"],
    ...
  },
  "linux": {
    "namespaces": [
      {"type": "network", "path": "/var/run/netns/test-net"}
    ],
    "cgroupsPath": "/test-cgroup"
  }
}
EOF

# Run container
sudo runc run my-container
```

### 8.2 Shared Namespaces Between Containers

Multiple containers sharing same network:

```bash
# Container 1 creates network namespace
sudo runc run --bundle /path/to/container1 container1

# Find network namespace
NETNS=$(sudo ip netns list | grep container1)

# Container 2 joins same network
# Update container2's config.json:
{
  "linux": {
    "namespaces": [
      {"type": "network", "path": "/var/run/netns/$NETNS"},
      {"type": "pid"},
      ...
    ]
  }
}

sudo runc run --bundle /path/to/container2 container2

# Both containers share network!
```

### 8.3 Custom Cgroup Hierarchy

```bash
# Create cgroup hierarchy
sudo mkdir -p /sys/fs/cgroup/production/web
echo "+cpu +memory" | sudo tee /sys/fs/cgroup/cgroup.subtree_control
echo "+cpu +memory" | sudo tee /sys/fs/cgroup/production/cgroup.subtree_control

# Set parent limits
echo "500M" | sudo tee /sys/fs/cgroup/production/memory.max
echo "200000 100000" | sudo tee /sys/fs/cgroup/production/cpu.max

# Configure container to use this cgroup
{
  "linux": {
    "cgroupsPath": "/production/web"
  }
}
```

---

## Resources

### Official Documentation
- OCI Runtime Spec: https://github.com/opencontainers/runtime-spec
- runc GitHub: https://github.com/opencontainers/runc
- Man pages: `man runc`, `man runc-run`, `man runc-create`

### Example Configs
```bash
# Official examples
git clone https://github.com/opencontainers/runtime-spec
cd runtime-spec/config-linux.md  # Detailed examples
```

### Related Tools
```bash
# Container runtimes
sudo apt-get install podman          # Docker alternative
sudo apt-get install buildah         # Build OCI images
sudo apt-get install skopeo          # Copy/inspect OCI images

# Low-level tools
sudo apt-get install umoci           # OCI image manipulation
sudo apt-get install crun            # Fast OCI runtime (C)
```

### Debugging
```bash
# Debug runc
sudo runc --debug run my-container

# Trace syscalls
sudo strace -f runc run my-container

# Check cgroup assignment
cat /proc/<PID>/cgroup
```

---

## Common Pitfalls

1. **Forgetting to use absolute paths**: Bundle paths must be absolute
2. **Rootfs not complete**: Missing libraries, devices, or /proc
3. **Permission issues**: Many operations require root
4. **Cgroup not initialized**: Must enable controllers in parent
5. **Namespace conflicts**: Can't delete namespace if processes exist
6. **Config validation**: Use `runc spec` as starting point

---

## Next Steps

You now understand:
- How Docker/Kubernetes/Podman run containers under the hood
- OCI specification and container standards
- Low-level Linux container primitives

**Advanced topics to explore:**
1. **Container images**: Learn OCI Image Spec, buildah, skopeo
2. **CNI networking**: Container Network Interface for networking
3. **Container orchestration**: Kubernetes architecture
4. **Alternative runtimes**: crun, youki, gVisor, Kata Containers
5. **Security hardening**: Seccomp profiles, AppArmor, SELinux

**Build your own container runtime!**

Now you have the knowledge to build a production-grade container system or contribute to existing projects like Docker, Podman, or Kubernetes.
