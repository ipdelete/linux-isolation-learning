# Phase 3: Network Namespace - Python Edition

This guide covers the same network namespace concepts as Phase 3 in `01-namespaces.md`, but uses Python to interact with network namespaces programmatically. This builds on the approach used in `src/pid_namespace.py`.

**Time**: 4-5 hours | **Difficulty**: Intermediate

---

## Prerequisites

- Completed Phase 2 (PID Namespace) and understand `src/pid_namespace.py`
- Root access (network namespace operations require privileges)
- Python 3.9+ (for `os.unshare()` support)
- Basic networking knowledge (IP addresses, routing)

**Install required tools:**
```bash
sudo pacman -S iproute2 bridge-utils
```

---

## 3.1 Concept

Network namespace gives each namespace its own:

- **Network interfaces** (lo, eth0, etc.)
- **IP addresses** and subnets
- **Routing tables**
- **Firewall rules** (iptables/nftables)
- **Port numbers** (multiple containers can bind to port 80)

**Why this matters:**

- Each container can bind to the same port without conflicts
- Complete network isolation between containers
- Can create virtual networks between containers
- Foundation of container networking (Docker, Kubernetes)

**How runc uses this:** In `03-runc.md`, network isolation is controlled via a `"type": "network"` entry under `linux.namespaces` in `config.json`. To attach a container to an existing network namespace, runc uses `"path": "/var/run/netns/..."`. All patterns you learn here map directly to how runc manages container networking.

---

## 3.2 Basic Network Namespace Operations

### 3.2.1 Creating Network Namespaces with Python

Python doesn't provide high-level network namespace APIs, so we combine:
- `os.unshare()` or `ctypes.clone()` for namespace creation
- `subprocess` to call `ip` commands for network configuration

**Example: Create and inspect a network namespace**

Create `src/net_namespace_basic.py`:

```python
#!/usr/bin/env python3
"""
Basic network namespace creation and inspection.
Demonstrates the minimal state of a new network namespace.
"""

import os
import sys
import subprocess

def create_and_inspect_netns():
    """Create a network namespace and show its initial state"""

    # Create network namespace
    print(f"Creating network namespace 'test-ns'...")
    result = subprocess.run(
        ["sudo", "ip", "netns", "add", "test-ns"],
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"Error creating namespace: {result.stderr}", file=sys.stderr)
        return 1

    # List all namespaces
    print("\nAll network namespaces:")
    subprocess.run(["ip", "netns", "list"])

    # Inspect the new namespace
    print("\nInterfaces in test-ns:")
    subprocess.run([
        "sudo", "ip", "netns", "exec", "test-ns",
        "ip", "addr", "show"
    ])

    print("\nNotice: Only loopback (lo) exists, and it's DOWN!")

    # Cleanup
    print("\nCleaning up...")
    subprocess.run(["sudo", "ip", "netns", "del", "test-ns"])

    return 0

if __name__ == '__main__':
    sys.exit(create_and_inspect_netns())
```

**Run it:**
```bash
chmod +x src/net_namespace_basic.py
./src/net_namespace_basic.py
```

**Expected output:**
```
Creating network namespace 'test-ns'...

All network namespaces:
test-ns

Interfaces in test-ns:
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

Notice: Only loopback (lo) exists, and it's DOWN!

Cleaning up...
```

### 3.2.2 Using unshare() to Enter a Network Namespace

Here's how to create a network namespace and run code inside it using `os.unshare()`:

Create `src/net_namespace_unshare.py`:

```python
#!/usr/bin/env python3
"""
Create a network namespace using os.unshare() and run commands inside it.
"""

import os
import sys
import subprocess

def main():
    print(f"Parent PID: {os.getpid()}")
    print(f"Network namespace before unshare:")
    subprocess.run(["readlink", f"/proc/{os.getpid()}/ns/net"])

    # Create new network namespace
    print("\nCreating new network namespace with unshare()...")
    try:
        os.unshare(os.CLONE_NEWNET)
    except PermissionError:
        print("Error: This script requires root privileges", file=sys.stderr)
        return 1

    print(f"Network namespace after unshare:")
    subprocess.run(["readlink", f"/proc/{os.getpid()}/ns/net"])

    # Show network interfaces in new namespace
    print("\nNetwork interfaces in new namespace:")
    subprocess.run(["ip", "addr", "show"])

    print("\nNotice: Only loopback, and it's DOWN!")
    print("This is a completely isolated network stack.")

    return 0

if __name__ == '__main__':
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        sys.exit(1)

    sys.exit(main())
```

**Run it:**
```bash
sudo ./src/net_namespace_unshare.py
```

---

## 3.3 Virtual Ethernet Pairs (veth)

**Concept:** A veth pair is like a virtual ethernet cable with two ends. Put one end in the host namespace and one end in a container namespace to enable communication.

```
┌─────────────────┐           ┌──────────────────┐
│   Host Network  │           │  Container netns │
│                 │           │                  │
│  10.0.0.1/24    │◄─────────►│   10.0.0.2/24   │
│  veth-host      │  veth pair│   veth-container│
└─────────────────┘           └──────────────────┘
```

### 3.3.1 Complete Working Example

Create `src/net_veth_example.py`:

```python
#!/usr/bin/env python3
"""
Create a veth pair connecting host and a network namespace.
Demonstrates container-to-host networking.
"""

import subprocess
import sys
import time

def run_cmd(cmd, description=""):
    """Run a command and print output"""
    if description:
        print(f"\n{description}")
        print(f"Running: {' '.join(cmd)}")

    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"Error: {result.stderr}", file=sys.stderr)
        return False

    if result.stdout:
        print(result.stdout)

    return True

def setup_veth_network():
    """Set up a veth pair between host and namespace"""

    namespace = "blue"
    host_veth = "veth-host"
    ns_veth = "veth-blue"
    host_ip = "10.0.0.1/24"
    ns_ip = "10.0.0.2/24"

    # Create namespace
    run_cmd(
        ["ip", "netns", "add", namespace],
        "Step 1: Create network namespace 'blue'"
    )

    # Create veth pair
    run_cmd(
        ["ip", "link", "add", host_veth, "type", "veth", "peer", "name", ns_veth],
        "Step 2: Create veth pair (virtual ethernet cable)"
    )

    # Move one end into namespace
    run_cmd(
        ["ip", "link", "set", ns_veth, "netns", namespace],
        f"Step 3: Move {ns_veth} into {namespace} namespace"
    )

    # Configure host side
    run_cmd(
        ["ip", "addr", "add", host_ip, "dev", host_veth],
        f"Step 4: Assign IP {host_ip} to host side"
    )

    run_cmd(
        ["ip", "link", "set", host_veth, "up"],
        f"Step 5: Bring up {host_veth} on host"
    )

    # Configure namespace side
    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "addr", "add", ns_ip, "dev", ns_veth],
        f"Step 6: Assign IP {ns_ip} to namespace side"
    )

    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "link", "set", ns_veth, "up"],
        f"Step 7: Bring up {ns_veth} in namespace"
    )

    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "link", "set", "lo", "up"],
        "Step 8: Bring up loopback in namespace"
    )

    # Show configuration
    print("\n" + "="*60)
    print("CONFIGURATION COMPLETE")
    print("="*60)

    print("\nHost side:")
    run_cmd(["ip", "addr", "show", host_veth])

    print("\nNamespace side:")
    run_cmd(["ip", "netns", "exec", namespace, "ip", "addr", "show"])

    # Test connectivity
    print("\n" + "="*60)
    print("CONNECTIVITY TEST")
    print("="*60)

    print("\nPing from namespace to host:")
    run_cmd([
        "ip", "netns", "exec", namespace,
        "ping", "-c", "3", "10.0.0.1"
    ])

    print("\nPing from host to namespace:")
    run_cmd(["ping", "-c", "3", "10.0.0.2"])

    return namespace

def cleanup(namespace):
    """Clean up the network namespace"""
    print("\n" + "="*60)
    print("CLEANUP")
    print("="*60)

    run_cmd(
        ["ip", "netns", "del", namespace],
        f"Deleting namespace {namespace} (this also removes veth pair)"
    )

def main():
    try:
        namespace = setup_veth_network()

        print("\n" + "="*60)
        print("Press Ctrl+C to cleanup and exit")
        print("="*60)

        # Keep running so user can experiment
        while True:
            time.sleep(1)

    except KeyboardInterrupt:
        print("\n\nReceived Ctrl+C, cleaning up...")
        cleanup(namespace)

    return 0

if __name__ == '__main__':
    import os
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        sys.exit(1)

    sys.exit(main())
```

**Run it:**
```bash
sudo ./src/net_veth_example.py
```

**What you'll see:**
1. Step-by-step veth pair creation
2. IP address assignment
3. Successful pings in both directions
4. The script keeps running so you can open another terminal and experiment

**While it's running, try in another terminal:**
```bash
# List network namespaces
ip netns list

# Execute commands in the namespace
sudo ip netns exec blue ip addr show
sudo ip netns exec blue ip route show
sudo ip netns exec blue ping 10.0.0.1

# See the veth on the host
ip addr show veth-host
```

Press Ctrl+C to cleanup.

---

## 3.4 Internet Access via NAT

Once you have a veth pair, you can give the namespace internet access through the host using Network Address Translation (NAT).

### 3.4.1 Complete Example with NAT

Create `src/net_internet_access.py`:

```python
#!/usr/bin/env python3
"""
Create a network namespace with internet access via NAT.
Demonstrates how containers get internet connectivity.
"""

import subprocess
import sys
import time

def run_cmd(cmd, description="", check=True):
    """Run a command and print output"""
    if description:
        print(f"\n{description}")
        print(f"Running: {' '.join(cmd)}")

    result = subprocess.run(cmd, capture_output=True, text=True)

    if check and result.returncode != 0:
        print(f"Error: {result.stderr}", file=sys.stderr)
        return False

    if result.stdout:
        print(result.stdout.strip())

    return True

def setup_nat_network():
    """Set up namespace with internet access"""

    namespace = "inet-test"
    host_veth = "veth-host-inet"
    ns_veth = "veth-inet"
    host_ip = "10.0.1.1"
    ns_ip = "10.0.1.2"
    subnet = "10.0.1.0/24"

    # Create namespace and veth pair (same as before)
    run_cmd(["ip", "netns", "add", namespace], "Create namespace")
    run_cmd(
        ["ip", "link", "add", host_veth, "type", "veth", "peer", "name", ns_veth],
        "Create veth pair"
    )
    run_cmd(["ip", "link", "set", ns_veth, "netns", namespace], "Move veth to namespace")

    # Configure IPs
    run_cmd(["ip", "addr", "add", f"{host_ip}/24", "dev", host_veth], "Set host IP")
    run_cmd(["ip", "link", "set", host_veth, "up"], "Bring up host veth")

    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "addr", "add", f"{ns_ip}/24", "dev", ns_veth],
        "Set namespace IP"
    )
    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "link", "set", ns_veth, "up"],
        "Bring up namespace veth"
    )
    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "link", "set", "lo", "up"],
        "Bring up loopback"
    )

    # Add default route in namespace (traffic goes to host)
    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "route", "add", "default", "via", host_ip],
        f"Add default route via {host_ip}"
    )

    # Enable IP forwarding on host
    run_cmd(
        ["sysctl", "-w", "net.ipv4.ip_forward=1"],
        "Enable IP forwarding on host"
    )

    # Set up NAT (masquerading)
    run_cmd(
        ["iptables", "-t", "nat", "-A", "POSTROUTING", "-s", subnet, "-j", "MASQUERADE"],
        f"Set up NAT for {subnet}"
    )

    print("\n" + "="*60)
    print("TESTING INTERNET ACCESS")
    print("="*60)

    print("\n1. Ping Google DNS from namespace:")
    run_cmd(
        ["ip", "netns", "exec", namespace, "ping", "-c", "3", "8.8.8.8"],
        check=False
    )

    print("\n2. DNS resolution test:")
    run_cmd(
        ["ip", "netns", "exec", namespace, "ping", "-c", "3", "example.com"],
        check=False
    )

    return namespace, subnet

def cleanup(namespace, subnet):
    """Clean up namespace and iptables rules"""
    print("\n" + "="*60)
    print("CLEANUP")
    print("="*60)

    run_cmd(
        ["iptables", "-t", "nat", "-D", "POSTROUTING", "-s", subnet, "-j", "MASQUERADE"],
        "Remove NAT rule",
        check=False
    )

    run_cmd(["ip", "netns", "del", namespace], "Delete namespace")

def main():
    namespace = None
    subnet = None

    try:
        namespace, subnet = setup_nat_network()

        print("\n" + "="*60)
        print("Namespace has internet access!")
        print("Try: sudo ip netns exec inet-test curl https://example.com")
        print("Press Ctrl+C to cleanup")
        print("="*60)

        while True:
            time.sleep(1)

    except KeyboardInterrupt:
        print("\n\nCleaning up...")
        if namespace and subnet:
            cleanup(namespace, subnet)

    return 0

if __name__ == '__main__':
    import os
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        sys.exit(1)

    sys.exit(main())
```

**Run it:**
```bash
sudo ./src/net_internet_access.py
```

**Experiment while it's running:**
```bash
# Fetch a webpage
sudo ip netns exec inet-test curl -I https://example.com

# Check routing
sudo ip netns exec inet-test ip route show

# Check DNS
sudo ip netns exec inet-test nslookup example.com
```

---

## 3.5 Bridge Networks (Container-to-Container)

A **bridge** acts like a virtual network switch, allowing multiple namespaces to communicate with each other.

```
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
   veth-ns1     veth-ns2
       │            │
 ┌─────┴─────┐ ┌────┴──────┐
 │    ns1    │ │    ns2    │
 │ 10.0.0.2  │ │ 10.0.0.3  │
 └───────────┘ └───────────┘
```

### 3.5.1 Complete Bridge Example

Create `src/net_bridge_example.py`:

```python
#!/usr/bin/env python3
"""
Create a bridge connecting two network namespaces.
Demonstrates container-to-container networking.
"""

import subprocess
import sys
import time

def run_cmd(cmd, description=""):
    """Run a command and print output"""
    if description:
        print(f"\n{description}")

    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"Error: {result.stderr}", file=sys.stderr)
        return False

    if result.stdout:
        print(result.stdout.strip())

    return True

def setup_bridge_network():
    """Set up bridge with two connected namespaces"""

    bridge = "br0"
    bridge_ip = "10.0.0.1/24"

    # Create bridge
    run_cmd(
        ["ip", "link", "add", bridge, "type", "bridge"],
        f"Step 1: Create bridge {bridge}"
    )
    run_cmd(
        ["ip", "addr", "add", bridge_ip, "dev", bridge],
        f"Step 2: Assign IP {bridge_ip} to bridge"
    )
    run_cmd(
        ["ip", "link", "set", bridge, "up"],
        "Step 3: Bring up bridge"
    )

    # Setup namespace 1
    setup_namespace_on_bridge(
        ns_name="ns1",
        bridge=bridge,
        veth_ns="veth-ns1",
        veth_br="veth-ns1-br",
        ip="10.0.0.2/24"
    )

    # Setup namespace 2
    setup_namespace_on_bridge(
        ns_name="ns2",
        bridge=bridge,
        veth_ns="veth-ns2",
        veth_br="veth-ns2-br",
        ip="10.0.0.3/24"
    )

    print("\n" + "="*60)
    print("CONNECTIVITY TESTS")
    print("="*60)

    print("\nTest 1: ns1 → ns2")
    run_cmd(["ip", "netns", "exec", "ns1", "ping", "-c", "3", "10.0.0.3"])

    print("\nTest 2: ns2 → ns1")
    run_cmd(["ip", "netns", "exec", "ns2", "ping", "-c", "3", "10.0.0.2"])

    print("\nTest 3: ns1 → bridge (host)")
    run_cmd(["ip", "netns", "exec", "ns1", "ping", "-c", "3", "10.0.0.1"])

    return bridge

def setup_namespace_on_bridge(ns_name, bridge, veth_ns, veth_br, ip):
    """Set up a single namespace connected to a bridge"""

    print(f"\n{'='*60}")
    print(f"Setting up {ns_name}")
    print(f"{'='*60}")

    # Create namespace
    run_cmd(["ip", "netns", "add", ns_name], f"Create namespace {ns_name}")

    # Create veth pair
    run_cmd(
        ["ip", "link", "add", veth_ns, "type", "veth", "peer", "name", veth_br],
        f"Create veth pair: {veth_ns} <-> {veth_br}"
    )

    # Move one end to namespace
    run_cmd(
        ["ip", "link", "set", veth_ns, "netns", ns_name],
        f"Move {veth_ns} to {ns_name}"
    )

    # Connect bridge side to bridge
    run_cmd(
        ["ip", "link", "set", veth_br, "master", bridge],
        f"Connect {veth_br} to bridge {bridge}"
    )
    run_cmd(["ip", "link", "set", veth_br, "up"], f"Bring up {veth_br}")

    # Configure namespace side
    run_cmd(
        ["ip", "netns", "exec", ns_name, "ip", "addr", "add", ip, "dev", veth_ns],
        f"Assign {ip} to {veth_ns}"
    )
    run_cmd(
        ["ip", "netns", "exec", ns_name, "ip", "link", "set", veth_ns, "up"],
        f"Bring up {veth_ns}"
    )
    run_cmd(
        ["ip", "netns", "exec", ns_name, "ip", "link", "set", "lo", "up"],
        "Bring up loopback"
    )

def cleanup(bridge):
    """Clean up bridge and namespaces"""
    print("\n" + "="*60)
    print("CLEANUP")
    print("="*60)

    run_cmd(["ip", "netns", "del", "ns1"], "Delete ns1")
    run_cmd(["ip", "netns", "del", "ns2"], "Delete ns2")
    run_cmd(["ip", "link", "del", bridge], "Delete bridge")

def main():
    bridge = None

    try:
        bridge = setup_bridge_network()

        print("\n" + "="*60)
        print("Two namespaces connected via bridge!")
        print("Try: sudo ip netns exec ns1 ping 10.0.0.3")
        print("Press Ctrl+C to cleanup")
        print("="*60)

        while True:
            time.sleep(1)

    except KeyboardInterrupt:
        print("\n\nCleaning up...")
        if bridge:
            cleanup(bridge)

    return 0

if __name__ == '__main__':
    import os
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        sys.exit(1)

    sys.exit(main())
```

**Run it:**
```bash
sudo ./src/net_bridge_example.py
```

---

## 3.6 Exercises

### Exercise 1: HTTP Server in Network Namespace

**Goal:** Create a network namespace, add internet access, and run a simple HTTP server. Access it from the host.

**Pseudocode structure:**

```python
#!/usr/bin/env python3
"""
Exercise 1: HTTP server in isolated network namespace
"""

def setup_namespace_with_http():
    """
    1. Create network namespace "web-ns"
    2. Create veth pair: veth-host <-> veth-web
    3. Assign IPs:
       - Host side: 10.0.2.1/24
       - Namespace side: 10.0.2.2/24
    4. Enable IP forwarding
    5. Set up NAT for internet access
    6. Start HTTP server in namespace:
       ip netns exec web-ns python3 -m http.server 8000 --bind 10.0.2.2
    """

    # TODO: Implement namespace creation

    # TODO: Implement veth pair setup

    # TODO: Implement NAT setup

    # TODO: Start HTTP server in namespace
    #       Hint: Use subprocess.Popen() to run in background

    pass

def test_connectivity():
    """
    Test that you can access the server:
    1. From namespace: curl http://10.0.2.2:8000
    2. From host: curl http://10.0.2.2:8000
    3. Bonus: Make it accessible from host on localhost:8000
       Hint: Use iptables DNAT (Destination NAT)
    """

    # TODO: Implement tests

    pass

# Expected result:
# - HTTP server runs in isolated namespace
# - Accessible from host at 10.0.2.2:8000
# - Bonus: Accessible from host at localhost:8000
```

**Hints:**
- Build on `net_internet_access.py`
- Use `subprocess.Popen()` to run the HTTP server in background
- For localhost access, research iptables DNAT rules

**Verification:**
```bash
# Check server is running
sudo ip netns exec web-ns netstat -tlnp | grep 8000

# Access from host
curl http://10.0.2.2:8000

# Check iptables rules
sudo iptables -t nat -L -n -v
```

---

### Exercise 2: Three Namespaces with Controlled Access

**Goal:** Create 3 namespaces (A, B, C) where:
- A can talk to B
- B can talk to C
- A **cannot** talk to C

**Topology:**

```
┌────────┐      ┌────────┐      ┌────────┐
│   A    │─────►│   B    │─────►│   C    │
│10.0.1.2│      │10.0.1.3│      │10.0.2.3│
└────────┘      │10.0.2.2│      └────────┘
                └────────┘
                (two IPs)
```

**Pseudocode structure:**

```python
#!/usr/bin/env python3
"""
Exercise 2: Three namespaces with access control
"""

def setup_three_namespaces():
    """
    Network design:

    Bridge br1 (10.0.1.0/24):
      - Connects: A (10.0.1.2) and B (10.0.1.3)

    Bridge br2 (10.0.2.0/24):
      - Connects: B (10.0.2.2) and C (10.0.2.3)

    Namespace B has TWO interfaces:
      - veth-b1 on br1 (can talk to A)
      - veth-b2 on br2 (can talk to C)

    A and C are on different subnets → cannot communicate directly
    """

    # TODO: Create two bridges: br1 and br2

    # TODO: Create namespace A on br1
    #       IP: 10.0.1.2/24

    # TODO: Create namespace B with TWO veths:
    #       veth-b1: 10.0.1.3/24 on br1
    #       veth-b2: 10.0.2.2/24 on br2

    # TODO: Create namespace C on br2
    #       IP: 10.0.2.3/24

    pass

def verify_connectivity():
    """
    Test all connections:

    ✓ A → B (10.0.1.2 → 10.0.1.3): Should work
    ✓ B → A (10.0.1.3 → 10.0.1.2): Should work
    ✓ B → C (10.0.2.2 → 10.0.2.3): Should work
    ✓ C → B (10.0.2.3 → 10.0.2.2): Should work
    ✗ A → C: Should FAIL (different subnets, no route)
    ✗ C → A: Should FAIL (different subnets, no route)
    """

    # TODO: Implement ping tests for all combinations

    pass

# Hints:
# - You need TWO bridges
# - Namespace B needs TWO veth pairs (one for each bridge)
# - A and C are on different subnets with no routing between them
# - Use ip netns exec for all namespace operations
```

**Verification commands:**
```bash
# Show B has two interfaces
sudo ip netns exec ns-b ip addr show

# Try pings
sudo ip netns exec ns-a ping -c 1 10.0.1.3  # A → B (should work)
sudo ip netns exec ns-a ping -c 1 10.0.2.3  # A → C (should fail)
sudo ip netns exec ns-b ping -c 1 10.0.1.2  # B → A (should work)
sudo ip netns exec ns-b ping -c 1 10.0.2.3  # B → C (should work)
```

---

### Exercise 3: Bandwidth Limiting with tc (Traffic Control)

**Goal:** Implement bandwidth limiting on a veth interface using `tc` (traffic control).

**Pseudocode structure:**

```python
#!/usr/bin/env python3
"""
Exercise 3: Bandwidth limiting with tc
"""

def setup_rate_limited_namespace():
    """
    1. Create namespace "slow-ns"
    2. Create veth pair: veth-host <-> veth-slow
    3. Assign IPs (10.0.3.1/24, 10.0.3.2/24)
    4. Apply bandwidth limit using tc:

       # Limit to 1 Mbit/s on the namespace side
       tc qdisc add dev veth-slow root tbf rate 1mbit burst 32kbit latency 400ms

    5. Test with iperf3 or by downloading a file
    """

    # TODO: Create namespace and veth pair

    # TODO: Apply tc bandwidth limiting
    #       Research: tc qdisc, tbf (Token Bucket Filter)
    #       Command: tc qdisc add dev <interface> root tbf rate <rate> burst <burst> latency <latency>

    pass

def test_bandwidth():
    """
    Test bandwidth limit:

    Method 1: Using iperf3
      - Host: iperf3 -s
      - Namespace: ip netns exec slow-ns iperf3 -c 10.0.3.1
      - Expected: ~1 Mbit/s throughput

    Method 2: Downloading a file
      - Set up HTTP server on host
      - Download from namespace, measure speed
    """

    # TODO: Implement bandwidth test

    pass

# Hints:
# - Install tc: sudo pacman -S iproute2 (should be installed)
# - Install iperf3: sudo pacman -S iperf3
# - Research tc qdisc types: tbf, htb, netem
# - You can apply tc to either the host or namespace side of veth
```

**Verification:**
```bash
# Check tc configuration
tc qdisc show dev veth-slow

# Run iperf3 test
# Terminal 1 (host):
iperf3 -s

# Terminal 2 (namespace):
sudo ip netns exec slow-ns iperf3 -c 10.0.3.1 -t 10

# Expected output: bandwidth around 1 Mbit/s
```

---

## 3.7 Combining Network and PID Namespaces

Now that you understand both PID and network namespaces, you can combine them to create more container-like environments.

**Example: Minimal container with isolated PID and network**

Create `src/net_pid_combined.py`:

```python
#!/usr/bin/env python3
"""
Combine PID and network namespaces to create a container-like environment.
"""

import os
import sys
import ctypes
import signal
import subprocess

STACK_SIZE = 1024 * 1024

libc = ctypes.CDLL('libc.so.6', use_errno=True)

def child_fn(arg):
    """Function to run in the new namespaces"""
    print(f"\n{'='*60}")
    print("Inside container namespaces:")
    print(f"{'='*60}")

    print(f"PID: {os.getpid()}")  # Should be 1
    print(f"PPID: {os.getppid()}")  # Should be 0

    # Show network interfaces
    print("\nNetwork interfaces:")
    subprocess.run(["ip", "addr", "show"])

    print("\nStarting bash in container...")
    print("Try: ps aux, ip addr, hostname")
    print(f"{'='*60}\n")

    os.execlp("bash", "bash")
    return 1

def main():
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        return 1

    print(f"Parent PID: {os.getpid()}")

    # Create callback
    CHILD_FUNC = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p)
    child_callback = CHILD_FUNC(child_fn)

    # Allocate stack
    stack = ctypes.create_string_buffer(STACK_SIZE)
    stack_top = ctypes.c_void_p(ctypes.addressof(stack) + STACK_SIZE)

    # Create both PID and network namespaces
    flags = os.CLONE_NEWPID | os.CLONE_NEWNET | signal.SIGCHLD

    print(f"Creating container with PID + Network namespaces...")

    child_pid = libc.clone(child_callback, stack_top, flags, None)

    if child_pid == -1:
        errno = ctypes.get_errno()
        print(f"clone failed: {os.strerror(errno)}", file=sys.stderr)
        return 1

    print(f"Created container with PID: {child_pid}")

    # Parent waits
    os.waitpid(child_pid, 0)
    return 0

if __name__ == '__main__':
    sys.exit(main())
```

**Run it:**
```bash
sudo ./src/net_pid_combined.py
```

**Inside the container bash:**
```bash
# Check PID
echo $$  # Should be 1

# Check processes
ps aux  # Only shows processes in this namespace

# Check network
ip addr show  # Only loopback, no network connectivity
```

This is the foundation! In Phase 4 (Mount Namespace) and Phase 6 (runc), you'll add filesystem isolation and bring all these pieces together into a complete container.

---

## 3.8 Two Approaches to Network Namespaces: `ip netns` vs `clone()`

You might have noticed this guide uses two different approaches to create network namespaces. Let's clarify when and why to use each.

### Approach Comparison

| Aspect | `ip netns add` | `clone(CLONE_NEWNET)` |
|--------|---------------|----------------------|
| **Persistence** | Persists in `/var/run/netns/` | Dies when process exits (unless bind-mounted) |
| **Named access** | Yes (`ip netns exec <name>`) | No, must use PID or `/proc/<pid>/ns/net` |
| **First principles** | No (uses iproute2 utility) | Yes (direct syscall) |
| **Container runtime approach** | Docker/runc often join existing netns | runc can also create via clone |
| **Flexibility** | Good for debugging, manual setup | Full control, programmatic |
| **Best for learning** | Understanding networking concepts | Understanding kernel primitives |

### Why Both Matter

**In production container runtimes:**

1. **CNI (Container Network Interface)** plugins often create persistent namespaces with `ip netns add`
2. The runtime (runc, crun) then **joins** those namespaces using `setns()`
3. Alternatively, the runtime can create namespaces directly with `clone()` and let the CNI plugin configure them

**For learning:**

- **Use `ip netns`** when you want to focus on networking concepts (veth, bridges, NAT)
- **Use `clone()`** when you want to understand how namespaces are actually created at the kernel level

### The Key Insight: Network Configuration is Separate from Creation

Whether you create a network namespace with `ip netns add` or `clone(CLONE_NEWNET)`, the **network configuration process is the same**:

1. Create veth pair
2. Move one end into the namespace
3. Assign IP addresses
4. Bring interfaces up
5. Configure routing

The difference is **how you reference the namespace** when configuring it.

---

## 3.9 Using `clone()` for Network Namespaces (Advanced)

Let's build network namespace examples using the `libc.clone()` approach from `src/pid_namespace.py`. This is closer to how container runtimes work internally.

### 3.9.1 The Challenge: Configuring from the Parent

When you create a namespace with `clone()`, the child process runs immediately in the isolated namespace. But network configuration (creating veth pairs, assigning IPs) must be done from the **parent** process, which sees the host's network namespace.

**The solution:** Use `/proc/<pid>/ns/net` to reference the child's namespace.

### 3.9.2 Complete Example: Network Namespace with clone()

Create `src/net_clone_veth.py`:

```python
#!/usr/bin/env python3
"""
Create network namespace using clone() and configure veth pair from parent.
Demonstrates how container runtimes configure networking.
"""

import os
import sys
import ctypes
import signal
import subprocess
import time

STACK_SIZE = 1024 * 1024

libc = ctypes.CDLL('libc.so.6', use_errno=True)

def child_fn(arg):
    """Function to run in the new network namespace"""

    print(f"\n{'='*60}")
    print("CHILD PROCESS (inside network namespace)")
    print(f"{'='*60}")

    print(f"Child PID: {os.getpid()}")
    print(f"Network namespace: {os.readlink('/proc/self/ns/net')}")

    # Wait for parent to configure networking
    print("\nWaiting for parent to configure network...")
    time.sleep(2)

    # Bring up loopback
    print("\nBringing up loopback interface...")
    subprocess.run(["ip", "link", "set", "lo", "up"])

    # Show network configuration
    print("\nNetwork interfaces after configuration:")
    subprocess.run(["ip", "addr", "show"])

    print("\nRouting table:")
    subprocess.run(["ip", "route", "show"])

    # Test connectivity
    print("\n" + "="*60)
    print("Testing connectivity to host (10.0.4.1)...")
    print("="*60)
    result = subprocess.run(
        ["ping", "-c", "3", "10.0.4.1"],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        print("✓ Success! Can ping host.")
    else:
        print("✗ Failed to ping host")
        print(result.stderr)

    # Keep running so user can experiment
    print("\n" + "="*60)
    print("Child process running. Try from another terminal:")
    print(f"  sudo nsenter -t {os.getpid()} -n ip addr show")
    print("Press Ctrl+C in parent to exit")
    print("="*60 + "\n")

    # Sleep indefinitely
    while True:
        time.sleep(1)

    return 0

def configure_veth_pair(child_pid):
    """Configure veth pair from parent process"""

    print(f"\n{'='*60}")
    print("PARENT PROCESS (configuring networking)")
    print(f"{'='*60}")

    veth_host = "veth-host-clone"
    veth_child = "veth-child-clone"
    host_ip = "10.0.4.1/24"
    child_ip = "10.0.4.2/24"

    # Step 1: Create veth pair
    print(f"\n1. Creating veth pair: {veth_host} <-> {veth_child}")
    subprocess.run([
        "ip", "link", "add", veth_host,
        "type", "veth",
        "peer", "name", veth_child
    ])

    # Step 2: Move child end to child's network namespace
    print(f"2. Moving {veth_child} to child's network namespace (PID {child_pid})")
    subprocess.run([
        "ip", "link", "set", veth_child, "netns", str(child_pid)
    ])

    # Step 3: Configure host side
    print(f"3. Configuring host side ({veth_host})")
    subprocess.run(["ip", "addr", "add", host_ip, "dev", veth_host])
    subprocess.run(["ip", "link", "set", veth_host, "up"])

    # Step 4: Configure child side (using nsenter to run command in child's netns)
    print(f"4. Configuring child side ({veth_child})")
    subprocess.run([
        "nsenter", "-t", str(child_pid), "-n",
        "ip", "addr", "add", child_ip, "dev", veth_child
    ])
    subprocess.run([
        "nsenter", "-t", str(child_pid), "-n",
        "ip", "link", "set", veth_child, "up"
    ])

    print("\n✓ Network configuration complete!")
    print(f"  Host:  {host_ip} on {veth_host}")
    print(f"  Child: {child_ip} on {veth_child}")

def main():
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        return 1

    print(f"Parent PID: {os.getpid()}")
    print(f"Parent network namespace: {os.readlink('/proc/self/ns/net')}")

    # Create callback
    CHILD_FUNC = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p)
    child_callback = CHILD_FUNC(child_fn)

    # Allocate stack
    stack = ctypes.create_string_buffer(STACK_SIZE)
    stack_top = ctypes.c_void_p(ctypes.addressof(stack) + STACK_SIZE)

    # Create network namespace using clone
    print("\nCreating network namespace with clone(CLONE_NEWNET)...")
    flags = os.CLONE_NEWNET | signal.SIGCHLD

    child_pid = libc.clone(child_callback, stack_top, flags, None)

    if child_pid == -1:
        errno = ctypes.get_errno()
        print(f"clone failed: {os.strerror(errno)}", file=sys.stderr)
        return 1

    print(f"✓ Created child with PID: {child_pid}")

    # Give child a moment to start
    time.sleep(0.5)

    # Configure networking from parent
    try:
        configure_veth_pair(child_pid)
    except subprocess.CalledProcessError as e:
        print(f"Network configuration failed: {e}", file=sys.stderr)
        return 1

    print("\n" + "="*60)
    print("PARENT WAITING FOR CHILD")
    print("="*60)
    print("Parent process will wait for child to exit...")
    print("Press Ctrl+C to terminate both processes")

    try:
        # Wait for child
        os.waitpid(child_pid, 0)
    except KeyboardInterrupt:
        print("\n\nReceived Ctrl+C, cleaning up...")

        # Kill child
        os.kill(child_pid, signal.SIGTERM)
        os.waitpid(child_pid, 0)

        # Clean up veth pair (child side deleted automatically with namespace)
        subprocess.run(["ip", "link", "del", "veth-host-clone"],
                      stderr=subprocess.DEVNULL)

    return 0

if __name__ == '__main__':
    sys.exit(main())
```

**Run it:**
```bash
chmod +x src/net_clone_veth.py
sudo ./src/net_clone_veth.py
```

**Expected output:**
```
Parent PID: 12345
Parent network namespace: net:[4026531840]

Creating network namespace with clone(CLONE_NEWNET)...
✓ Created child with PID: 12346

============================================================
PARENT PROCESS (configuring networking)
============================================================

1. Creating veth pair: veth-host-clone <-> veth-child-clone
2. Moving veth-child-clone to child's network namespace (PID 12346)
3. Configuring host side (veth-host-clone)
4. Configuring child side (veth-child-clone)

✓ Network configuration complete!
  Host:  10.0.4.1/24 on veth-host-clone
  Child: 10.0.4.2/24 on veth-child-clone

============================================================
CHILD PROCESS (inside network namespace)
============================================================
Child PID: 12346
Network namespace: net:[4026532500]

Waiting for parent to configure network...

Bringing up loopback interface...

Network interfaces after configuration:
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
5: veth-child-clone@if6: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP
    link/ether 02:42:ac:11:00:02 brd ff:ff:ff:ff:ff:ff
    inet 10.0.4.2/24 scope global veth-child-clone

Routing table:
10.0.4.0/24 dev veth-child-clone proto kernel scope link src 10.0.4.2

============================================================
Testing connectivity to host (10.0.4.1)...
============================================================
✓ Success! Can ping host.
```

### 3.9.3 Key Techniques Used

1. **`nsenter` command**: Runs commands in the child's namespace
   ```bash
   nsenter -t <pid> -n <command>
   ```
   - `-t <pid>`: Target process ID
   - `-n`: Enter network namespace

2. **Moving veth to namespace by PID**:
   ```bash
   ip link set <interface> netns <pid>
   ```
   Instead of using a named namespace, we reference it by the child's PID.

3. **Parent configures, child uses**: The parent does all the heavy lifting (creating veth pair, moving it, assigning IPs), then the child just uses the configured network.

### 3.9.4 Combining PID + Network with Full Configuration

Now let's combine everything: PID namespace, network namespace, and proper network configuration.

Create `src/net_clone_full.py`:

```python
#!/usr/bin/env python3
"""
Create PID + Network namespaces using clone() with full network configuration.
This is closest to how container runtimes work.
"""

import os
import sys
import ctypes
import signal
import subprocess
import time

STACK_SIZE = 1024 * 1024

libc = ctypes.CDLL('libc.so.6', use_errno=True)

def child_fn(arg):
    """Function to run in the new PID + Network namespaces"""

    print(f"\n{'='*60}")
    print("INSIDE CONTAINER")
    print(f"{'='*60}")

    print(f"PID: {os.getpid()}")  # Should be 1
    print(f"PPID: {os.getppid()}")  # Should be 0
    print(f"Network namespace: {os.readlink('/proc/self/ns/net')}")

    # Wait for parent to configure networking
    time.sleep(2)

    # Bring up interfaces
    subprocess.run(["ip", "link", "set", "lo", "up"],
                  stderr=subprocess.DEVNULL)

    print("\nNetwork configuration:")
    subprocess.run(["ip", "addr", "show"])

    print("\nTesting internet connectivity (via NAT):")
    result = subprocess.run(
        ["ping", "-c", "2", "8.8.8.8"],
        capture_output=True,
        text=True,
        timeout=5
    )

    if result.returncode == 0:
        print("✓ Internet access works!")
    else:
        print("✗ No internet access (expected if NAT not configured)")

    print("\n" + "="*60)
    print("Starting bash. Try:")
    print("  ps aux          # See PID isolation")
    print("  ip addr         # See network isolation")
    print("  ping 10.0.5.1   # Ping host")
    print("="*60 + "\n")

    os.execlp("bash", "bash")
    return 0

def configure_network_with_nat(child_pid):
    """Configure network with NAT for internet access"""

    print(f"\n{'='*60}")
    print("CONFIGURING NETWORK")
    print(f"{'='*60}")

    veth_host = "veth-cnt-host"
    veth_child = "veth-cnt-child"
    host_ip = "10.0.5.1"
    child_ip = "10.0.5.2"
    subnet = "10.0.5.0/24"

    # Create veth pair
    print(f"\n1. Creating veth pair")
    subprocess.run([
        "ip", "link", "add", veth_host,
        "type", "veth",
        "peer", "name", veth_child
    ])

    # Move to child namespace
    print(f"2. Moving {veth_child} to container (PID {child_pid})")
    subprocess.run(["ip", "link", "set", veth_child, "netns", str(child_pid)])

    # Configure host side
    print(f"3. Configuring host: {host_ip}/24")
    subprocess.run(["ip", "addr", "add", f"{host_ip}/24", "dev", veth_host])
    subprocess.run(["ip", "link", "set", veth_host, "up"])

    # Configure container side
    print(f"4. Configuring container: {child_ip}/24")
    subprocess.run([
        "nsenter", "-t", str(child_pid), "-n",
        "ip", "addr", "add", f"{child_ip}/24", "dev", veth_child
    ])
    subprocess.run([
        "nsenter", "-t", str(child_pid), "-n",
        "ip", "link", "set", veth_child, "up"
    ])

    # Add default route in container
    print(f"5. Adding default route in container")
    subprocess.run([
        "nsenter", "-t", str(child_pid), "-n",
        "ip", "route", "add", "default", "via", host_ip
    ])

    # Enable IP forwarding
    print(f"6. Enabling IP forwarding")
    subprocess.run(["sysctl", "-w", "net.ipv4.ip_forward=1"],
                  stdout=subprocess.DEVNULL)

    # Set up NAT
    print(f"7. Setting up NAT (masquerading)")
    subprocess.run([
        "iptables", "-t", "nat", "-A", "POSTROUTING",
        "-s", subnet, "-j", "MASQUERADE"
    ])

    print(f"\n✓ Network configuration complete with internet access!")

def cleanup_nat(subnet):
    """Clean up NAT rule"""
    subprocess.run([
        "iptables", "-t", "nat", "-D", "POSTROUTING",
        "-s", subnet, "-j", "MASQUERADE"
    ], stderr=subprocess.DEVNULL)

def main():
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        return 1

    print(f"Parent PID: {os.getpid()}")

    # Create callback
    CHILD_FUNC = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p)
    child_callback = CHILD_FUNC(child_fn)

    # Allocate stack
    stack = ctypes.create_string_buffer(STACK_SIZE)
    stack_top = ctypes.c_void_p(ctypes.addressof(stack) + STACK_SIZE)

    # Create PID + Network namespaces
    print("\nCreating container with PID + Network namespaces...")
    flags = os.CLONE_NEWPID | os.CLONE_NEWNET | signal.SIGCHLD

    child_pid = libc.clone(child_callback, stack_top, flags, None)

    if child_pid == -1:
        errno = ctypes.get_errno()
        print(f"clone failed: {os.strerror(errno)}", file=sys.stderr)
        return 1

    print(f"✓ Created container with PID: {child_pid}")

    # Give child time to start
    time.sleep(0.5)

    # Configure networking
    try:
        configure_network_with_nat(child_pid)
    except Exception as e:
        print(f"Network configuration failed: {e}", file=sys.stderr)
        cleanup_nat("10.0.5.0/24")
        return 1

    print("\n" + "="*60)
    print("Container running! Parent waiting...")
    print("="*60)

    try:
        os.waitpid(child_pid, 0)
    except KeyboardInterrupt:
        print("\n\nCleaning up...")
        os.kill(child_pid, signal.SIGTERM)
        time.sleep(0.5)
        os.waitpid(child_pid, 0)
    finally:
        cleanup_nat("10.0.5.0/24")
        subprocess.run(["ip", "link", "del", "veth-cnt-host"],
                      stderr=subprocess.DEVNULL)

    return 0

if __name__ == '__main__':
    sys.exit(main())
```

**Run it:**
```bash
chmod +x src/net_clone_full.py
sudo ./src/net_clone_full.py
```

**Inside the container:**
```bash
# Check you're PID 1
echo $$

# Check network
ip addr show

# Ping host
ping 10.0.5.1

# Test internet
ping 8.8.8.8
curl https://example.com
```

### 3.9.5 Why This Matters

This approach (`clone()` + parent configuration) is **exactly how container runtimes work**:

1. **runc** calls `clone()` with multiple namespace flags
2. The parent process configures the environment (network, mounts, cgroups)
3. The child process runs the container payload

**Key differences from `ip netns`:**

```python
# ip netns approach
subprocess.run(["ip", "netns", "add", "mycontainer"])
subprocess.run(["ip", "netns", "exec", "mycontainer", "bash"])

# clone() approach (what runc does)
flags = os.CLONE_NEWPID | os.CLONE_NEWNET | signal.SIGCHLD
child_pid = libc.clone(child_callback, stack_top, flags, None)
# Parent configures networking using child_pid
```

The `clone()` approach gives you:
- Multiple namespaces atomically
- The child is PID 1 (proper init process)
- Full programmatic control
- Direct understanding of kernel primitives

### 3.9.6 Using `unshare()` for Multiple Namespaces

An alternative to `clone()` is using `unshare()` to create multiple namespaces at once, similar to `src/pid_unshare.py`. This approach is simpler and doesn't require stack allocation.

**Key behavioral differences:**

| Namespace Type | Behavior after `unshare()` |
|---------------|---------------------------|
| **Network** | Calling process **immediately** enters new namespace |
| **PID** | Calling process stays in old namespace; only **children** enter new namespace |

This means when you combine them with `unshare(CLONE_NEWPID | CLONE_NEWNET)`:
- You immediately enter the new network namespace
- You must fork to have a child enter the new PID namespace

Create `src/net_pid_unshare.py`:

```python
#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

"""
Create PID + Network namespaces using unshare() - simpler than clone().
Mirrors src/pid_unshare.py but adds network namespace.
"""

import os
import ctypes
import sys
import subprocess

libc = ctypes.CDLL('libc.so.6', use_errno=True)

def main():
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        return 1

    print(f"Parent PID in original namespaces: {os.getpid()}")
    print(f"Network namespace before: {os.readlink('/proc/self/ns/net')}")

    # Create both PID and Network namespaces at once
    print("\nCalling unshare(CLONE_NEWPID | CLONE_NEWNET)...")
    result = libc.unshare(os.CLONE_NEWPID | os.CLONE_NEWNET)

    if result != 0:
        errno = ctypes.get_errno()
        print(f"unshare failed: {os.strerror(errno)}", file=sys.stderr)
        return 1

    print("✓ Created namespaces with unshare()")

    # After unshare:
    # - We're IMMEDIATELY in the new network namespace
    # - We're still in the old PID namespace (our PID hasn't changed)
    print(f"\nParent PID after unshare (still in old PID ns): {os.getpid()}")
    print(f"Network namespace after unshare (new): {os.readlink('/proc/self/ns/net')}")

    # Show network is isolated
    print("\nNetwork interfaces in new network namespace:")
    subprocess.run(["ip", "addr", "show"])

    # Fork to enter the new PID namespace
    print("\nForking to enter PID namespace...")
    pid = os.fork()

    if pid == 0:
        # Child process - now in BOTH new namespaces
        print("\n" + "="*60)
        print("CHILD PROCESS (in new PID + Network namespaces)")
        print("="*60)

        print(f"Child PID (in new PID namespace): {os.getpid()}")  # Should be 1
        print(f"Child PPID: {os.getppid()}")  # Should be 0
        print(f"Network namespace: {os.readlink('/proc/self/ns/net')}")

        # Bring up loopback
        subprocess.run(["ip", "link", "set", "lo", "up"],
                      stderr=subprocess.DEVNULL)

        print("\nNetwork interfaces:")
        subprocess.run(["ip", "addr", "show"])

        print("\n" + "="*60)
        print("Child in isolated container environment")
        print("PID isolation: ✓ (PID 1)")
        print("Network isolation: ✓ (only loopback)")
        print("="*60)

        return 0
    else:
        # Parent process - still in old PID namespace, new network namespace
        print(f"\nParent PID (still in old PID namespace): {os.getpid()}")
        print("Parent waiting for child...")

        os.waitpid(pid, 0)
        print("\nChild exited, parent exiting")

    return 0

if __name__ == '__main__':
    sys.exit(main())
```

**Run it:**
```bash
chmod +x src/net_pid_unshare.py
sudo ./src/net_pid_unshare.py
```

**Expected output:**
```
Parent PID in original namespaces: 12345
Network namespace before: net:[4026531840]

Calling unshare(CLONE_NEWPID | CLONE_NEWNET)...
✓ Created namespaces with unshare()

Parent PID after unshare (still in old PID ns): 12345
Network namespace after unshare (new): net:[4026532500]

Network interfaces in new network namespace:
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

Forking to enter PID namespace...

Parent PID (still in old PID namespace): 12345
Parent waiting for child...

============================================================
CHILD PROCESS (in new PID + Network namespaces)
============================================================
Child PID (in new PID namespace): 1
Child PPID: 0
Network namespace: net:[4026532500]

Network interfaces:
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo

============================================================
Child in isolated container environment
PID isolation: ✓ (PID 1)
Network isolation: ✓ (only loopback)
============================================================

Child exited, parent exiting
```

**Key observations:**

1. **Parent enters network namespace immediately**
   - After `unshare()`, parent is already in new network namespace
   - Parent's PID doesn't change (still in old PID namespace)

2. **Child enters PID namespace via fork**
   - Child becomes PID 1 in new PID namespace
   - Child inherits the network namespace from parent
   - Both namespaces are now active for the child

3. **Simpler than `clone()`**
   - No stack allocation needed
   - No callback function required
   - Standard `os.fork()` pattern
   - Uses uv script format (easier to run)

**Comparison with `clone()` approach:**

| Aspect | `unshare()` + `fork()` | `clone()` |
|--------|------------------------|-----------|
| **Complexity** | Simpler, standard fork | More complex, needs stack |
| **Code style** | Familiar Python pattern | C-style callback |
| **Flexibility** | Good for most cases | Full control over child |
| **When to use** | Learning, prototyping | Production-like code |

**This is the approach used by:**
- `unshare(1)` command-line tool
- Many container runtime initialization sequences
- Quick namespace experiments

The `unshare()` approach is excellent for understanding namespace behavior and is often used in combination with `nsenter` for container management.

---

## 3.10 Practical Comparison: Both Approaches Side-by-Side

Let's see both approaches solving the same problem: creating a network namespace with internet access.

### Approach 1: Using `ip netns`

```python
# Create named namespace
subprocess.run(["ip", "netns", "add", "web"])

# Create and configure veth
subprocess.run(["ip", "link", "add", "veth0", "type", "veth", "peer", "name", "veth1"])
subprocess.run(["ip", "link", "set", "veth1", "netns", "web"])  # Use name
subprocess.run(["ip", "addr", "add", "10.0.0.1/24", "dev", "veth0"])
subprocess.run(["ip", "link", "set", "veth0", "up"])

# Configure inside namespace
subprocess.run(["ip", "netns", "exec", "web",  # Reference by name
               "ip", "addr", "add", "10.0.0.2/24", "dev", "veth1"])
subprocess.run(["ip", "netns", "exec", "web",
               "ip", "link", "set", "veth1", "up"])

# Run program in namespace
subprocess.run(["ip", "netns", "exec", "web", "python3", "-m", "http.server"])
```

**Pros:** Simple, persistent, easy to debug
**Cons:** Not programmatic, not how runtimes work

### Approach 2: Using `clone()`

```python
def child_fn(arg):
    # Already in network namespace
    # Just use the configured network
    os.execlp("python3", "python3", "-m", "http.server")

# Create namespace
flags = os.CLONE_NEWNET | signal.SIGCHLD
child_pid = libc.clone(child_callback, stack_top, flags, None)

# Configure from parent
subprocess.run(["ip", "link", "add", "veth0", "type", "veth", "peer", "name", "veth1"])
subprocess.run(["ip", "link", "set", "veth1", "netns", str(child_pid)])  # Use PID
subprocess.run(["ip", "addr", "add", "10.0.0.1/24", "dev", "veth0"])
subprocess.run(["ip", "link", "set", "veth0", "up"])

# Configure child's side using nsenter
subprocess.run(["nsenter", "-t", str(child_pid), "-n",
               "ip", "addr", "add", "10.0.0.2/24", "dev", "veth1"])
subprocess.run(["nsenter", "-t", str(child_pid), "-n",
               "ip", "link", "set", "veth1", "up"])

os.waitpid(child_pid, 0)
```

**Pros:** Direct syscalls, multiple namespaces, runtime approach
**Cons:** More complex, requires nsenter or /proc

### When to Use Each

**Use `ip netns` when:**
- Learning networking concepts
- Quick experiments and debugging
- Working with CNI plugins
- You want persistent namespaces

**Use `clone()` when:**
- Learning container runtime internals
- Building container runtimes
- Combining multiple namespace types
- Understanding kernel primitives

**In production:** Container runtimes use both! CNI plugins create `ip netns`, then runc joins them with `setns()`, or runc creates with `clone()` and CNI configures them.

---

## Resources

### Python Networking Libraries

- **subprocess**: For running `ip`, `tc`, `iptables` commands
- **socket**: For low-level socket programming
- **pyroute2**: High-level Python library for network configuration (optional)
  ```bash
  pip install pyroute2
  ```

### Man Pages

```bash
man ip-netns          # Network namespace management
man ip-link           # Link (interface) configuration
man tc                # Traffic control
man iptables          # Firewall and NAT
man veth              # Virtual ethernet devices
```

### Debugging Tools

```bash
# Show all network namespaces
ip netns list

# Show interfaces in a namespace
sudo ip netns exec <ns> ip addr show

# Show routes in a namespace
sudo ip netns exec <ns> ip route show

# Show iptables rules
sudo iptables -t nat -L -n -v

# Capture traffic on veth
sudo tcpdump -i veth-host -n

# Monitor bandwidth
sudo iftop -i veth-host
```

---

## Next Steps

After completing this phase:

1. **Phase 4: Mount Namespace** - Add filesystem isolation
2. **Combine all namespaces** - Create a minimal container runtime
3. **Phase 6: runc** - See how production runtimes implement these concepts

**Key takeaways:**

- Network namespaces provide complete network isolation
- veth pairs connect namespaces like virtual cables
- Bridges enable container-to-container communication
- NAT provides internet access through the host
- Traffic control (`tc`) manages bandwidth and latency
- Python + subprocess is a practical way to manage network namespaces

These are the same primitives Docker and Kubernetes use for container networking!
