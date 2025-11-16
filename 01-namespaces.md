# Learning Linux Namespaces

A practical guide to understanding and using Linux namespaces for process isolation, with pointers to the kernel docs and classic deep‑dive articles.

## What Are Namespaces?

Namespaces are a Linux kernel feature that **wrap** global resources (like PIDs, network interfaces, mount points, user IDs) so that processes inside a namespace see what looks like their own private instance of those resources.

The formal definition from the `namespaces(7)` man page:

> A namespace wraps a global system resource in an abstraction that makes it appear to the processes within the namespace that they have their own isolated instance of the global resource. Changes to the global resource are visible to other processes that are members of the namespace, but are invisible to other processes.

**Think of it as**: Creating parallel universes where processes in each universe see different views of the system.

Namespaces are one of the two main primitives behind Linux containers:

- **Namespaces** → *what* a process can see (PID table, network stack, filesystem, hostname, etc.).
- **cgroups** → *how much* of a resource it can use (CPU, memory, I/O).

For the authoritative overview, see:

- [`namespaces(7)`](https://man7.org/linux/man-pages/man7/namespaces.7.html)
- LWN series "Namespaces in operation": https://lwn.net/Articles/531114/

### Namespaces in the runc Learning Path

This file (`01-namespaces.md`) focuses on the raw Linux primitives: what each
namespace type does and how to work with them directly via `unshare(1)`,
`ip netns`, and low-level code.

In `03-runc.md` you’ll see the same ideas applied by a real OCI runtime:

- `runc` reads a container’s `config.json` (defined by the OCI Runtime Spec).
- The `linux.namespaces` array in that file tells `runc` which namespaces to
    create or join for the container’s process (PID, mount, network, UTS, IPC,
    user, cgroup).
- The `linux.resources` section maps to cgroup configuration; `runc` programs
    the appropriate cgroup v2 controllers before starting the container.
- During `runc run …`, it calls the same kernel APIs shown here
    (`clone(2)`, `unshare(2)`, `setns(2)`) to:
    - enter or create namespaces,
    - set up mounts and the root filesystem,
    - move the container process into the configured cgroups.

As you progress:

- When you see **PID namespaces** here, look at how `03-runc.md` uses
    `"type": "pid"` under `linux.namespaces` and treats the container process
    as PID 1 in that namespace.
- When you see **network namespaces**, compare the `ip netns` examples here
    with the section in `03-runc.md` that joins an existing netns via
    `"path": "/var/run/netns/..."`.
- When you see **mount namespaces and rootfs**, compare with the `root` and
    `mounts` sections in `config.json`, which tell `runc` how to construct the
    container’s filesystem view.

By the time you finish this document, `03-runc.md` should read like
"how a production runtime wires together all the primitives you just
practiced."

---

## Prerequisites

- Linux system (Ubuntu 20.04+, Fedora, or similar)
- Root access (or sudo)
- Basic understanding of:
    - Linux processes (ps, fork, exec)
    - File descriptors
    - C programming (helpful but not required)

---

## Phase 1: Understanding Namespace Concepts (2–3 hours)

### 1.1 The Main Namespace Types

Historically there were "seven" namespaces; modern kernels also have **time** namespaces. The canonical reference table is in `namespaces(7)` (linked above).

Learn what each namespace isolates:

| Namespace | Man page | Constant | Isolates | Typical container use |
|-----------|----------|----------|----------|------------------------|
| **PID** | `pid_namespaces(7)` | `CLONE_NEWPID` | Process ID number space | Separate PID 1, isolated process tree |
| **Network** | `network_namespaces(7)` | `CLONE_NEWNET` | Network stack (interfaces, addresses, routes, iptables) | Each container has its own network stack |
| **Mount** | `mount_namespaces(7)` | `CLONE_NEWNS` | Mount table (what is mounted where) | Private root filesystem, bind mounts |
| **UTS** | `uts_namespaces(7)` | `CLONE_NEWUTS` | Hostname and NIS domain | Per‑container hostname |
| **IPC** | `ipc_namespaces(7)` | `CLONE_NEWIPC` | System V IPC, POSIX message queues | Contain shared memory segments, semaphores |
| **User** | `user_namespaces(7)` | `CLONE_NEWUSER` | User and group IDs, capabilities scope | Unprivileged user outside, root inside |
| **Cgroup** | `cgroup_namespaces(7)` | `CLONE_NEWCGROUP` | Cgroup root directory view | Isolate how cgroup hierarchies appear inside |
| **Time** | `time_namespaces(7)` | `CLONE_NEWTIME` | Boot and monotonic clocks | Time shifting for tests, per‑container time |

When you read kernel docs or `man` pages, you’ll see these `CLONE_NEW*` flags used in `clone(2)`, `unshare(2)`, and `setns(2)`.
| **Cgroup** | CLONE_NEWCGROUP | Cgroup hierarchy | Isolated cgroup view |

### 1.2 Key System Calls

Three core syscalls underpin all namespace operations (see `clone(2)`, `unshare(2)`, `setns(2)`):

```c
// Create new namespaces for current process
int unshare(int flags);

// Create child process with new namespaces
int clone(int (*fn)(void *), void *stack, int flags, void *arg);

// Join existing namespace
int setns(int fd, int nstype);
```

### 1.3 Namespace Discovery via `/proc`

**Hands-on Exercise:**

```bash
# View your current namespaces
ls -la /proc/$$/ns/

# Example output:
# lrwxrwxrwx 1 user user 0 Jan 1 12:00 ipc -> ipc:[4026531839]
# lrwxrwxrwx 1 user user 0 Jan 1 12:00 mnt -> mnt:[4026531840]
# lrwxrwxrwx 1 user user 0 Jan 1 12:00 net -> net:[4026531841]
# ...

# View namespaces for any process
ls -la /proc/1/ns/

# Compare namespaces between processes
readlink /proc/$$/ns/pid
readlink /proc/1/ns/pid
```

**Understanding the output:**

- Each namespace instance has a unique inode number (shown inside `[...]`).
- Processes in the same namespace share the same `(device, inode)` for a given `/proc/<pid>/ns/<type>` symlink.
- You can open these symlinks (or a bind‑mount of them) and pass the resulting file descriptor to `setns(2)` to join that namespace.

More details: `namespaces(7)` (section **The /proc/pid/ns/ directory**).

---

## Phase 2: PID Namespace (Hands-on: 3-4 hours)

### 2.1 Concept

PID namespace makes processes think they're PID 1 (init process).

**Why this matters:**

- Containers need their own PID 1
- Isolates process visibility
- Process signals are contained

### 2.2 Basic Example: Using unshare Command

```bash
# Create a new PID namespace (requires root)
sudo unshare --pid --fork --mount-proc bash

# Inside the new namespace:
ps aux
# You'll see ONLY processes in this namespace!

# Check your PID
echo $$  # Will show a low number, likely not 1 (yet)

# Exit the namespace
exit
```

**What happened:**

- `--pid`: New PID namespace
- `--fork`: Fork before exec (required for PID namespace)
- `--mount-proc`: Mount new /proc (so ps shows correct PIDs)

**How runc uses this:** In `03-runc.md`, the PID namespace is enabled by
adding an entry with `"type": "pid"` in the `linux.namespaces` array of
`config.json`. When you run `sudo runc run ...`, runc ensures the container
process becomes PID 1 inside that PID namespace and mounts `/proc` according
to the `mounts` section, so tools like `ps` see the correct, isolated PID view.

### 2.3 Write Your First Namespace Program (Python)

Here is a minimal Python example that uses `ctypes` to call `libc.clone()`
and run `ps aux` inside the new PID namespace.

```python
#!/usr/bin/env python3
import ctypes
import os
import sys
import signal

# Constants
CLONE_NEWPID = 0x20000000
STACK_SIZE = 1024 * 1024

# Load libc
libc = ctypes.CDLL('libc.so.6', use_errno=True)

def child_fn(arg):
    """Function to run in the new PID namespace"""
    pid = os.getpid()
    ppid = os.getppid()
    print(f"Child PID: {pid}")
    print(f"Child PPID: {ppid}")

    # Execute ps aux
    os.execlp("ps", "ps", "aux")
    # If exec fails, we'll get here
    return 1

def main():
    print(f"Parent PID: {os.getpid()}")

    # Create a callback type that matches the clone signature
    CHILD_FUNC = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p)
    child_callback = CHILD_FUNC(child_fn)

    # Allocate stack for child
    stack = ctypes.create_string_buffer(STACK_SIZE)
    stack_top = ctypes.c_void_p(ctypes.addressof(stack) + STACK_SIZE)

    # Clone with CLONE_NEWPID flag
    flags = CLONE_NEWPID | signal.SIGCHLD
    child_pid = libc.clone(
        child_callback,
        stack_top,
        flags,
        None
    )

    if child_pid == -1:
        errno = ctypes.get_errno()
        print(f"clone failed: {os.strerror(errno)}", file=sys.stderr)
        return 1

    print(f"Created child with PID: {child_pid}")

    # Wait for child to finish
    os.waitpid(child_pid, 0)
    return 0

if __name__ == '__main__':
    sys.exit(main())
```

**Run:**

```bash
chmod +x pid_namespace.py
sudo ./pid_namespace.py
```

**Expected output:**

```text
Parent PID: 12345
Created child with PID: 12346
Child PID: 1        # <-- Child thinks it's PID 1!
Child PPID: 0       # <-- Parent is PID 0 from child's view
```

### 2.4 Understanding the Double Fork (Python)

To truly become PID 1 inside the new namespace, the child created by `clone`
can fork again so the grandchild inherits PID 1. Here is the core of that
logic written in Python:

```python
import os
import time

def double_fork_demo():
    """Double fork to become PID 1 in the namespace"""
    print(f"First child PID: {os.getpid()}")

    pid = os.fork()

    if pid == 0:
        # Second child - will be PID 1 in new PID namespace
        print(f"Second child PID: {os.getpid()}")  # will be 1 in new PID ns
        time.sleep(100)
        os._exit(0)
    else:
        # First child waits for second child
        os.waitpid(pid, 0)
```

You would call `double_fork_demo()` from inside the `child_fn()` of the previous
example after entering the new PID namespace.

### 2.5 Exercises

1. **Exercise 1**: Modify the program to create a PID namespace and run `bash` inside it. Verify you're PID 1.

2. **Exercise 2**: Create a Python wrapper using ctypes:

    ```python
   import ctypes
   import os

   CLONE_NEWPID = 0x20000000

   libc = ctypes.CDLL('libc.so.6', use_errno=True)
   result = libc.unshare(CLONE_NEWPID)

   if result == 0:
       print("Created PID namespace!")
       # Fork to enter namespace
       if os.fork() == 0:
           print(f"Child PID: {os.getpid()}")
   ```

3. **Exercise 3**: Create two separate PID namespaces and run `sleep 100` in each. Verify they can't see each other's processes.

---

## Phase 3: Network Namespace (Hands-on: 4-5 hours)

### 3.1 Concept

Network namespace gives each namespace its own:

- Network interfaces
- IP addresses
- Routing tables
- Firewall rules
- Port numbers

**Why this matters:**

- Each container can bind to port 80 without conflicts
- Complete network isolation
- Can create virtual networks between containers

### 3.2 Basic Network Namespace

```bash
# Create network namespace
sudo ip netns add test-ns

# List namespaces
ip netns list

# Execute command in namespace
sudo ip netns exec test-ns ip addr show
# Output: Only loopback (lo), no other interfaces!

# Clean up
sudo ip netns del test-ns
```

### 3.3 Create Virtual Network Between Host and Namespace

This is the foundation of container networking!

```bash
# Create namespace
sudo ip netns add blue

# Create veth pair (virtual ethernet cable)
sudo ip link add veth-host type veth peer name veth-blue

# Move one end into namespace
sudo ip link set veth-blue netns blue

# Configure host side
sudo ip addr add 10.0.0.1/24 dev veth-host
sudo ip link set veth-host up

# Configure namespace side
sudo ip netns exec blue ip addr add 10.0.0.2/24 dev veth-blue
sudo ip netns exec blue ip link set veth-blue up
sudo ip netns exec blue ip link set lo up

# Test connectivity
sudo ip netns exec blue ping 10.0.0.1  # Should work!

# From host
ping 10.0.0.2  # Should work!
```

**Visualization:**

```text
┌─────────────────┐           ┌──────────────────┐
│   Host Network  │           │  blue namespace  │
│                 │           │                  │
│  10.0.0.1/24    │◄─────────►│   10.0.0.2/24   │
│  veth-host      │  veth pair│   veth-blue     │
└─────────────────┘           └──────────────────┘
```

### 3.4 Add Internet Access via NAT

```bash
# Enable IP forwarding on host
sudo sysctl -w net.ipv4.ip_forward=1

# Add default route in namespace
sudo ip netns exec blue ip route add default via 10.0.0.1

# Set up NAT (masquerading) on host
sudo iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -j MASQUERADE

# Test internet access from namespace
sudo ip netns exec blue ping 8.8.8.8  # Should work!
sudo ip netns exec blue curl https://example.com  # Should work!
```

### 3.5 Exercise: Two Containers Communicating

Create two namespaces that can talk to each other via a bridge:

```bash
# Create bridge (like a virtual switch)
sudo ip link add br0 type bridge
sudo ip addr add 10.0.0.1/24 dev br0
sudo ip link set br0 up

# Create namespace 1
sudo ip netns add ns1
sudo ip link add veth-ns1 type veth peer name veth-ns1-br
sudo ip link set veth-ns1 netns ns1
sudo ip link set veth-ns1-br master br0
sudo ip link set veth-ns1-br up
sudo ip netns exec ns1 ip addr add 10.0.0.2/24 dev veth-ns1
sudo ip netns exec ns1 ip link set veth-ns1 up
sudo ip netns exec ns1 ip link set lo up

# Create namespace 2
sudo ip netns add ns2
sudo ip link add veth-ns2 type veth peer name veth-ns2-br
sudo ip link set veth-ns2 netns ns2
sudo ip link set veth-ns2-br master br0
sudo ip link set veth-ns2-br up
sudo ip netns exec ns2 ip addr add 10.0.0.3/24 dev veth-ns2
sudo ip netns exec ns2 ip link set veth-ns2 up
sudo ip netns exec ns2 ip link set lo up

# Test: ns1 → ns2
sudo ip netns exec ns1 ping 10.0.0.3  # Should work!

# Test: ns2 → ns1
sudo ip netns exec ns2 ping 10.0.0.2  # Should work!
```

**Visualization:**

```text
       ┌────────────┐
       │   Bridge   │
       │   br0      │
       │ 10.0.0.1   │
       └─────┬──────┘
             │
       ┌─────┴──────┐
       │            │
   veth-ns1-br  veth-ns2-br
       │            │
       │            │
   veth-ns1     veth-ns2
       │            │
 ┌─────┴─────┐ ┌────┴──────┐
 │    ns1    │ │    ns2    │
 │ 10.0.0.2  │ │ 10.0.0.3  │
 └───────────┘ └───────────┘
```

### 3.6 Exercises

1. **Exercise 1**: Create a network namespace, add internet access, and run a simple HTTP server (Python: `python3 -m http.server 8000`). Access it from the host.

2. **Exercise 2**: Create 3 namespaces (A, B, C) where:
   - A can talk to B
   - B can talk to C
   - A cannot talk to C
   (Hint: Use separate bridges)

3. **Exercise 3**: Implement bandwidth limiting using `tc` (traffic control) on the veth interface.

**How runc uses this:** In `03-runc.md`, network isolation is controlled via a
`"type": "network"` entry under `linux.namespaces` in `config.json`. To
attach a container to an existing `ip netns` namespace, runc uses a
`"path": "/var/run/netns/..."` field for that namespace entry. All of the
`ip netns` patterns you practice here map directly to how runc can join or
create network namespaces when starting a container.

---

## Phase 4: Mount Namespace (Hands-on: 3-4 hours)

### 4.1 Concept

Mount namespace isolates filesystem mount points. Each namespace can have different filesystems mounted at the same path.

**Why this matters:**

- Each container has its own root filesystem
- Bind mounts don't affect host
- `/proc`, `/sys` can be remounted

### 4.2 Basic Mount Namespace

```bash
# Create mount namespace
sudo unshare --mount bash

# Inside namespace, mount something
mount -t tmpfs tmpfs /tmp/test

# Exit and check from host
ls /tmp/test  # Empty! Mount was isolated

# Clean up
umount /tmp/test
```

### 4.3 Private vs Shared Mount Propagation

Understanding mount propagation is critical!

```bash
# Make all mounts private (don't propagate to host)
sudo unshare --mount bash
mount --make-rprivate /

# Now mounts in this namespace won't affect host
mkdir -p /tmp/test-mount
mount -t tmpfs tmpfs /tmp/test-mount
echo "hello from namespace" > /tmp/test-mount/file.txt

# From another terminal (host):
ls /tmp/test-mount  # Empty!
```

### 4.4 Creating a Minimal Root Filesystem

This is the foundation of containers:

```bash
# Create directory structure
mkdir -p /tmp/mini-rootfs/{bin,lib,lib64,usr,proc,sys,dev,tmp}

# Copy essential binaries
cp /bin/bash /tmp/mini-rootfs/bin/
cp /bin/ls /tmp/mini-rootfs/bin/
cp /bin/ps /tmp/mini-rootfs/bin/

# Copy required libraries (use ldd to find them)
ldd /bin/bash
# Copy each library to mini-rootfs/lib or mini-rootfs/lib64

# Example for bash libraries:
cp /lib/x86_64-linux-gnu/libc.so.6 /tmp/mini-rootfs/lib/
cp /lib64/ld-linux-x86-64.so.2 /tmp/mini-rootfs/lib64/
# ... copy all dependencies

# Create mount + PID namespace
sudo unshare --mount --pid --fork bash

# Change root to our minimal filesystem
mount --make-rprivate /
mount --bind /tmp/mini-rootfs /tmp/mini-rootfs
cd /tmp/mini-rootfs
mount -t proc proc proc
pivot_root . .
cd /

# Now you're in a minimal container!
ls /  # See your mini filesystem
ps aux  # Will fail without /proc mounted properly
```

### 4.5 Using chroot vs pivot_root

**chroot** (old way):

```bash
sudo chroot /tmp/mini-rootfs /bin/bash
```

**pivot_root** (modern, more secure):

```bash
# pivot_root requires:
# 1. New root is a mount point
# 2. Old root is moved to a directory under new root

mkdir /tmp/mini-rootfs/oldroot
mount --bind /tmp/mini-rootfs /tmp/mini-rootfs
cd /tmp/mini-rootfs
pivot_root . oldroot
cd /
umount -l /oldroot
rmdir /oldroot
```

### 4.6 Exercises

1. **Exercise 1**: Create a mount namespace and mount a tmpfs at `/data`. Write a file to it. Verify the file doesn't exist on the host.

2. **Exercise 2**: Build a more complete rootfs using debootstrap:

    ```bash
   sudo debootstrap --variant=minbase focal /tmp/my-rootfs http://archive.ubuntu.com/ubuntu/
   sudo chroot /tmp/my-rootfs bash
   ```

3. **Exercise 3**: Create a program that combines PID + Mount + Network namespaces to create a fully isolated environment.

**How runc uses this:** In `03-runc.md`, the `root` and `mounts` sections of
`config.json` describe exactly the mount namespace layout that runc will
construct. The `"type": "mount"` entry in `linux.namespaces` tells runc to
create a private mount namespace, then it bind-mounts the `root.path` (rootfs)
and mounts `/proc`, `/dev`, `/sys`, and any bind mounts you configure.

---

## Phase 5: Other Namespaces (2-3 hours)

### 5.1 UTS Namespace (Hostname Isolation)

```bash
# Create UTS namespace
sudo unshare --uts bash

# Change hostname (only affects this namespace)
hostname my-container
hostname  # Shows: my-container

# Exit and check host
hostname  # Shows: original hostname
```

### 5.2 IPC Namespace (Inter-Process Communication)

```bash
# Create shared memory on host
ipcmk -M 1024

# List IPC resources
ipcs

# Create IPC namespace
sudo unshare --ipc bash

# Inside namespace
ipcs  # Empty! No shared memory visible

# Create new shared memory in namespace
ipcmk -M 1024

# Exit and check host
ipcs  # Original shared memory still there
```

### 5.3 User Namespace (UID/GID Remapping)

This is the most complex but powerful namespace!

```bash
# Create user namespace (doesn't require root!)
unshare --user bash

# Inside namespace
whoami  # Shows: nobody
id     # Shows: uid=65534(nobody) gid=65534(nogroup)

# But we can become "root" inside namespace
echo "0 $(id -u) 1" > /proc/self/uid_map
echo "0 $(id -g) 1" > /proc/self/gid_map

# Now we're root inside!
whoami  # root
```

**User namespace mapping:**

```text
Host UID 1000 ──maps to──> Container UID 0 (root)
```

This lets unprivileged users run "root" inside containers safely!

**How runc uses this:** runc can enable user namespaces via a `"type": "user"`
entry under `linux.namespaces` in `config.json` and by setting
`linux.uidMappings` and `linux.gidMappings`. That JSON describes the same
UID/GID remapping you experiment with here (host UID → container UID 0), so
the container sees "root" while the host still treats the process as an
unprivileged user.

### 5.4 Combining All Namespaces

```bash
sudo unshare \
    --pid \
    --net \
    --mount \
    --uts \
    --ipc \
    --fork \
    bash

# You're now in a fully isolated environment!
# This is essentially a container
```

---

## Phase 6: Advanced Topics (4-5 hours)

### 6.1 Namespace Persistence

Namespaces usually die when last process exits. To persist:

```bash
# Create namespace
sudo unshare --net=/var/run/netns/persistent bash

# From another terminal
sudo ip netns exec persistent ip addr show

# Namespace persists even after bash exits
```

### 6.2 Joining Existing Namespaces (setns)

Create `join_namespace.c`:

```c
#define _GNU_SOURCE
#include <fcntl.h>
#include <sched.h>
#include <stdio.h>
#include <unistd.h>

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: %s <namespace-path>\n", argv[0]);
        return 1;
    }

    // Open namespace file descriptor
    int fd = open(argv[1], O_RDONLY);
    if (fd == -1) {
        perror("open");
        return 1;
    }

    // Join the namespace
    if (setns(fd, 0) == -1) {
        perror("setns");
        return 1;
    }

    // Execute shell in the namespace
    execlp("bash", "bash", NULL);

    return 0;
}
```

**Usage:**
```bash
gcc -o join_namespace join_namespace.c

# Join PID namespace of process 1234
sudo ./join_namespace /proc/1234/ns/pid
```

### 6.3 Namespace Hierarchies

PID and user namespaces are hierarchical:

```
Init PID namespace (PID 1)
  ├─ Container 1 (PID 1 in its namespace)
  │   └─ Nested container (PID 1 in its namespace)
  └─ Container 2 (PID 1 in its namespace)
```

Create nested namespaces:
```bash
sudo unshare --pid --fork bash
# Inside first namespace
unshare --pid --fork bash
# Inside second namespace - doubly nested!
```

---

## Phase 7: Practical Projects (8-10 hours)

### Project 1: Minimal Container Runtime

Build a simple container runtime in Python:

```python
#!/usr/bin/env python3
import ctypes
import os
import sys

# Namespace flags
CLONE_NEWNS = 0x00020000    # Mount
CLONE_NEWPID = 0x20000000   # PID
CLONE_NEWNET = 0x40000000   # Network
CLONE_NEWUTS = 0x04000000   # UTS (hostname)
CLONE_NEWIPC = 0x08000000   # IPC

def create_container(rootfs, command):
    """Create and enter namespaces"""
    libc = ctypes.CDLL('libc.so.6', use_errno=True)

    # Create all namespaces
    flags = CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET | CLONE_NEWUTS | CLONE_NEWIPC
    result = libc.unshare(flags)

    if result != 0:
        print(f"unshare failed: {os.strerror(ctypes.get_errno())}")
        sys.exit(1)

    # Fork to enter PID namespace as PID 1
    pid = os.fork()

    if pid == 0:
        # Child process
        setup_filesystem(rootfs)
        setup_hostname("my-container")
        os.execvp(command[0], command)
    else:
        # Parent process
        os.waitpid(pid, 0)

def setup_filesystem(rootfs):
    """Set up mount namespace"""
    # Mount rootfs
    os.chroot(rootfs)
    os.chdir('/')

    # Mount /proc
    if not os.path.exists('/proc'):
        os.makedirs('/proc')
    os.system('mount -t proc proc /proc')

def setup_hostname(name):
    """Set hostname in UTS namespace"""
    with open('/proc/sys/kernel/hostname', 'w') as f:
        f.write(name)

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: container.py <rootfs> <command>")
        sys.exit(1)

    rootfs = sys.argv[1]
    command = sys.argv[2:]

    create_container(rootfs, command)
```

### Project 2: Network Setup Script

Create automated network namespace setup:

```bash
#!/bin/bash
# container-network.sh

CONTAINER_NS=$1
CONTAINER_IP=$2

# Create namespace
ip netns add $CONTAINER_NS

# Create veth pair
ip link add veth-${CONTAINER_NS} type veth peer name veth-${CONTAINER_NS}-br

# Move one end to namespace
ip link set veth-${CONTAINER_NS} netns $CONTAINER_NS

# Configure namespace side
ip netns exec $CONTAINER_NS ip addr add ${CONTAINER_IP}/24 dev veth-${CONTAINER_NS}
ip netns exec $CONTAINER_NS ip link set veth-${CONTAINER_NS} up
ip netns exec $CONTAINER_NS ip link set lo up
ip netns exec $CONTAINER_NS ip route add default via ${CONTAINER_IP%.*}.1

# Configure host side (assuming bridge exists)
ip link set veth-${CONTAINER_NS}-br master br0
ip link set veth-${CONTAINER_NS}-br up

echo "Network configured for $CONTAINER_NS at $CONTAINER_IP"
```

### Project 3: Container Monitoring Tool

Monitor processes across namespaces:

```python
#!/usr/bin/env python3
import os
from pathlib import Path

def get_namespace_info(pid):
    """Get namespace inodes for a process"""
    ns_path = Path(f"/proc/{pid}/ns")
    if not ns_path.exists():
        return None

    namespaces = {}
    for ns_file in ns_path.iterdir():
        try:
            link = os.readlink(str(ns_file))
            ns_type = ns_file.name
            ns_inode = link.split('[')[1].rstrip(']')
            namespaces[ns_type] = ns_inode
        except:
            pass

    return namespaces

def group_processes_by_namespace():
    """Group processes by their namespaces"""
    namespace_groups = {}

    for pid_dir in Path("/proc").glob("[0-9]*"):
        pid = int(pid_dir.name)
        ns_info = get_namespace_info(pid)

        if ns_info and 'pid' in ns_info:
            pid_ns = ns_info['pid']

            if pid_ns not in namespace_groups:
                namespace_groups[pid_ns] = []

            namespace_groups[pid_ns].append(pid)

    return namespace_groups

if __name__ == '__main__':
    groups = group_processes_by_namespace()

    print(f"Found {len(groups)} PID namespaces:\n")

    for ns_inode, pids in groups.items():
        print(f"Namespace {ns_inode}:")
        print(f"  Processes: {pids}")
        print()
```

---

## Resources

### Books

- "Linux Kernel Development" by Robert Love
- "The Linux Programming Interface" by Michael Kerrisk

### Man Pages (Essential Reading)
```bash
man namespaces       # Overview of all namespaces
man pid_namespaces   # PID namespace details
man network_namespaces
man mount_namespaces
man user_namespaces
man unshare
man clone
man setns
```

### Online Resources

- Michael Kerrisk's man-pages project: https://man7.org/linux/man-pages/
- LWN "Namespaces in operation" series index: https://lwn.net/Articles/531114/
- Jérôme Petazzoni's "Linux Containers" talk: https://www.youtube.com/watch?v=sK5i-N34im8

### Tools to Explore

```bash
# Namespace utilities
sudo apt-get install util-linux  # Provides unshare, nsenter

# Network utilities
sudo apt-get install iproute2     # Provides ip command
sudo apt-get install bridge-utils # Provides brctl

# Container tools to learn from
sudo apt-get install docker.io
sudo apt-get install podman
```

---

## Next Steps

After mastering namespaces, move to:
1. **Cgroups** (resource limiting)
2. **runc** (putting it all together)
3. **seccomp-bpf** (syscall filtering)
4. **capabilities** (privilege management)

This foundation will enable you to understand how Docker, Kubernetes, and other container technologies work under the hood!
