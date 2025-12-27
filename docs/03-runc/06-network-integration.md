# 06 Network Integration

## Goal

Learn how OCI-compliant container runtimes handle network namespaces by connecting a runc container to a pre-configured network namespace. You will understand the two approaches to container networking (runc creates vs. external tool creates) and implement the external tool pattern used by Kubernetes, Docker, and other orchestrators.

**Deliverable**: Run an OCI container attached to a network namespace you created using tools from the namespace lessons, with verified network connectivity between the container and host.

**Time estimate**: ~50 minutes

## Prereqs

- Completed `05-seccomp.md` (understand OCI runtime configuration)
- Completed `docs/01-namespaces/06-netns-basics.md` through `08-netns-nat.md` (network namespace fundamentals)
- `runc` installed and working (`runc --version`)
- An OCI bundle ready (from lessons 01-03)
- `sudo` access for network operations

## Background: How Container Runtimes Handle Networking

Container networking is one of the most complex aspects of containerization. Unlike other namespaces (PID, mount, UTS) which containers typically create fresh, network namespaces often require external coordination.

### The Networking Problem

When a container starts, it needs:
1. An isolated network stack (network namespace)
2. Network interfaces configured inside that namespace
3. Connectivity to the host and/or external networks
4. Optionally, DNS resolution and port forwarding

But here's the challenge: setting up veth pairs, bridges, IP addresses, and routing requires the namespace to exist *before* the container process starts. This creates a chicken-and-egg problem.

### Two Approaches to Container Networking

#### Approach 1: Runtime Creates Namespace (Simple)

```json
{
  "linux": {
    "namespaces": [
      { "type": "network" }
    ]
  }
}
```

When `path` is omitted, runc creates a fresh network namespace. The container starts with only a loopback interface (down). This approach:
- Is simple to configure
- Results in complete network isolation
- Requires hooks or external tools to set up connectivity
- Is rarely used alone in production

#### Approach 2: Join Existing Namespace (Production Pattern)

```json
{
  "linux": {
    "namespaces": [
      { "type": "network", "path": "/var/run/netns/container-net" }
    ]
  }
}
```

When `path` is specified, runc joins an existing network namespace. This approach:
- Allows network setup *before* container starts
- Enables CNI (Container Network Interface) plugins
- Is how Docker, Kubernetes, and Podman actually work
- Separates networking concerns from runtime concerns

### Why the External Pattern Matters

Real container orchestrators like Kubernetes use the external pattern because:

1. **CNI plugins**: Networking is handled by pluggable components (Calico, Flannel, Cilium)
2. **Pre-configuration**: Network must be ready before container starts
3. **Cleanup**: Network resources can be cleaned up independently
4. **Flexibility**: Different containers can share or isolate networks

The workflow looks like:

```
1. Orchestrator calls CNI plugin
2. CNI creates network namespace at /var/run/netns/XXX
3. CNI configures interfaces, IPs, routes
4. Orchestrator calls runc with namespace path
5. Container starts in pre-configured network
6. On exit, orchestrator calls CNI for cleanup
```

## The config.json Namespaces Section

The `linux.namespaces` array in config.json controls namespace behavior:

```json
{
  "linux": {
    "namespaces": [
      { "type": "pid" },
      { "type": "mount" },
      { "type": "ipc" },
      { "type": "uts" },
      { "type": "network", "path": "/var/run/netns/mynetns" }
    ]
  }
}
```

**Namespace object fields**:
- `type`: One of `pid`, `network`, `mount`, `ipc`, `uts`, `user`, `cgroup`
- `path` (optional): Path to existing namespace file to join

**Behavior**:
- If `path` is omitted: runc creates a new namespace of that type
- If `path` is specified: runc joins the existing namespace at that path

**Important**: The path must be a bind-mount of a namespace file (like `/proc/[pid]/ns/net` or `/var/run/netns/[name]`). This is exactly what `ip netns add` creates.

## Exercises

This lesson uses manual exercises rather than TDD because:
1. We're configuring runc, not writing Rust code
2. Networking requires step-by-step verification
3. The tools (`netns-tool`, `ip`) are already implemented

### Exercise 1: Set Up a Network Namespace with Connectivity

First, create and configure a network namespace using the tools from the namespace lessons.

**Step 1: Create the network namespace**

```bash
# Create a persistent network namespace
sudo ip netns add container-net

# Verify it exists
ip netns list
# Output: container-net

# Check the namespace file
ls -la /var/run/netns/
# Output: container-net
```

**Step 2: Create a veth pair**

```bash
# Create veth pair: veth-host (on host) <-> veth-container (for namespace)
sudo ip link add veth-host type veth peer name veth-container

# Verify both ends exist on host (initially)
ip link show veth-host
ip link show veth-container
```

**Step 3: Move one end into the namespace**

```bash
# Move veth-container into the network namespace
sudo ip link set veth-container netns container-net

# Verify it's gone from host view
ip link show veth-container
# Error: Cannot find device "veth-container"

# Verify it's in the namespace
sudo ip netns exec container-net ip link show veth-container
# Success: shows interface in DOWN state
```

**Step 4: Configure IP addresses**

```bash
# Host side: 10.0.0.1/24
sudo ip addr add 10.0.0.1/24 dev veth-host

# Namespace side: 10.0.0.2/24
sudo ip netns exec container-net ip addr add 10.0.0.2/24 dev veth-container
```

**Step 5: Bring interfaces up**

```bash
# Host side
sudo ip link set veth-host up

# Namespace side (both veth and loopback)
sudo ip netns exec container-net ip link set veth-container up
sudo ip netns exec container-net ip link set lo up
```

**Step 6: Verify connectivity**

```bash
# Ping from host to namespace
ping -c 2 10.0.0.2
# Expected: 2 packets transmitted, 2 received

# Ping from namespace to host
sudo ip netns exec container-net ping -c 2 10.0.0.1
# Expected: 2 packets transmitted, 2 received
```

You now have a working network namespace at `/var/run/netns/container-net` that can communicate with the host.

### Exercise 2: Configure runc to Use the Network Namespace

Now we'll modify an OCI bundle to use the pre-made network namespace.

**Step 1: Create or navigate to your OCI bundle**

```bash
# Create a fresh bundle directory
mkdir -p ~/oci-netns-test
cd ~/oci-netns-test

# Generate default config.json
runc spec

# Create minimal rootfs with networking tools
mkdir -p rootfs/{bin,proc,sys,dev/pts,dev/shm,dev/mqueue,etc,root,run,tmp}

# Copy busybox for testing (provides ip, ping, sh)
cp /bin/busybox rootfs/bin/
# Or if using Alpine: docker export $(docker create alpine) | tar -C rootfs -xf -
```

**Alternative: Use Docker to extract Alpine rootfs**

```bash
# If busybox isn't available, use Alpine
docker pull alpine:latest
docker create --name temp-alpine alpine:latest
docker export temp-alpine | tar -C rootfs -xf -
docker rm temp-alpine
```

**Step 2: Examine the default namespace configuration**

```bash
# Look at the namespaces section
cat config.json | jq '.linux.namespaces'
```

Default output:
```json
[
  { "type": "pid" },
  { "type": "network" },
  { "type": "ipc" },
  { "type": "uts" },
  { "type": "mount" }
]
```

Note: `network` has no `path` field, meaning runc would create a new namespace.

**Step 3: Modify config.json to join the existing namespace**

Edit `config.json` to add the path to your network namespace:

```bash
# Using jq to modify in place
cat config.json | jq '.linux.namespaces = [
  { "type": "pid" },
  { "type": "network", "path": "/var/run/netns/container-net" },
  { "type": "ipc" },
  { "type": "uts" },
  { "type": "mount" }
]' > config.json.tmp && mv config.json.tmp config.json
```

Or manually edit to add the path:

```json
{
  "linux": {
    "namespaces": [
      { "type": "pid" },
      { "type": "network", "path": "/var/run/netns/container-net" },
      { "type": "ipc" },
      { "type": "uts" },
      { "type": "mount" }
    ]
  }
}
```

**Step 4: Configure the process to run a network command**

Modify the `process` section to run a command that shows network info:

```bash
cat config.json | jq '.process.args = ["sh", "-c", "ip addr && ping -c 3 10.0.0.1"]' > config.json.tmp && mv config.json.tmp config.json
```

Or manually edit:

```json
{
  "process": {
    "terminal": true,
    "args": ["sh", "-c", "ip addr && ping -c 3 10.0.0.1"]
  }
}
```

### Exercise 3: Run the Container and Verify Networking

**Step 1: Run the container**

```bash
cd ~/oci-netns-test
sudo runc run netns-test
```

**Expected output**:
```
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
       valid_lft forever preferred_lft forever
2: veth-container@if5: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP qlen 1000
    link/ether XX:XX:XX:XX:XX:XX brd ff:ff:ff:ff:ff:ff link-netnsid 0
    inet 10.0.0.2/24 scope global veth-container
       valid_lft forever preferred_lft forever
PING 10.0.0.1 (10.0.0.1): 56 data bytes
64 bytes from 10.0.0.1: seq=0 ttl=64 time=0.XXX ms
64 bytes from 10.0.0.1: seq=1 ttl=64 time=0.XXX ms
64 bytes from 10.0.0.1: seq=2 ttl=64 time=0.XXX ms
```

**What this proves**:
1. The container sees `veth-container` interface (not `veth-host`)
2. The container has IP `10.0.0.2/24` configured
3. The container can ping the host at `10.0.0.1`
4. The loopback interface is present and up

**Step 2: Verify from outside the container**

While the container is running (use a second terminal or run in background):

```bash
# See what processes are in the namespace
sudo ip netns pids container-net

# Run ip inside the namespace directly
sudo ip netns exec container-net ip addr
# Should match what the container sees
```

### Exercise 4: Compare with New Namespace (No Path)

See what happens when runc creates its own network namespace.

**Step 1: Create a second bundle**

```bash
mkdir -p ~/oci-newnetns-test
cd ~/oci-newnetns-test
runc spec
mkdir -p rootfs/{bin,proc,sys,dev/pts,dev/shm,dev/mqueue,etc,root,run,tmp}
cp /bin/busybox rootfs/bin/
# Or copy Alpine rootfs as before
```

**Step 2: Keep default config (no path on network)**

The default config already has `{ "type": "network" }` without a path.

Modify args to show network state:

```bash
cat config.json | jq '.process.args = ["sh", "-c", "ip link && ip addr"]' > config.json.tmp && mv config.json.tmp config.json
```

**Step 3: Run and observe**

```bash
sudo runc run newnetns-test
```

**Expected output**:
```
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
```

**Key observations**:
- Only loopback interface exists
- Loopback is DOWN (not configured)
- No IP addresses assigned
- No external connectivity possible

This demonstrates why the external namespace pattern is preferred in production.

### Exercise 5: Using Pre-start Hooks (Advanced)

Runc supports hooks that run at various lifecycle stages. Pre-start hooks can set up networking when you can't pre-create the namespace.

**config.json with hooks**:

```json
{
  "hooks": {
    "prestart": [
      {
        "path": "/usr/local/bin/setup-network.sh",
        "args": ["setup-network.sh", "--container-id"],
        "env": ["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"]
      }
    ],
    "poststop": [
      {
        "path": "/usr/local/bin/cleanup-network.sh"
      }
    ]
  }
}
```

The hook receives container state on stdin (JSON with PID, bundle path, etc.) and can:
1. Read the container's PID
2. Access `/proc/[pid]/ns/net` to get the namespace
3. Create veth pairs and configure networking
4. Set up iptables rules

**Note**: Hooks are more complex than the external namespace pattern. CNI plugins use this approach, but for learning, the external pattern is clearer.

## Verify

**Automated verification**:

Create a simple verification script:

```bash
#!/bin/bash
# verify-network-integration.sh

set -e

echo "=== Network Integration Verification ==="

# Check namespace exists
if ! ip netns list | grep -q "container-net"; then
    echo "FAIL: container-net namespace not found"
    exit 1
fi
echo "PASS: Network namespace exists"

# Check veth pair
if ! ip link show veth-host &>/dev/null; then
    echo "FAIL: veth-host not found"
    exit 1
fi
echo "PASS: veth-host interface exists"

# Check connectivity
if ! ping -c 1 -W 1 10.0.0.2 &>/dev/null; then
    echo "FAIL: Cannot ping namespace (10.0.0.2)"
    exit 1
fi
echo "PASS: Host can reach namespace"

# Check namespace can reach host
if ! sudo ip netns exec container-net ping -c 1 -W 1 10.0.0.1 &>/dev/null; then
    echo "FAIL: Namespace cannot ping host (10.0.0.1)"
    exit 1
fi
echo "PASS: Namespace can reach host"

# Check config.json has namespace path
if ! cat ~/oci-netns-test/config.json | jq -e '.linux.namespaces[] | select(.type == "network") | .path' &>/dev/null; then
    echo "FAIL: config.json missing network namespace path"
    exit 1
fi
echo "PASS: config.json configured correctly"

echo ""
echo "=== All verifications passed! ==="
```

Run it:

```bash
chmod +x verify-network-integration.sh
./verify-network-integration.sh
```

**Manual verification checklist**:

1. Network namespace exists: `ip netns list | grep container-net`
2. Veth pair is configured: `ip link show veth-host`
3. Namespace interface visible: `sudo ip netns exec container-net ip addr`
4. Bidirectional connectivity: ping both directions
5. Container sees correct network: run container with `ip addr`

## Clean Up

Remove all resources created during this lesson:

```bash
# Delete any running containers
sudo runc delete -f netns-test 2>/dev/null || true
sudo runc delete -f newnetns-test 2>/dev/null || true

# Delete the veth pair (deletes both ends)
sudo ip link del veth-host 2>/dev/null || true

# Delete the network namespace
sudo ip netns del container-net 2>/dev/null || true

# Verify cleanup
ip netns list | grep container-net && echo "WARN: namespace still exists" || echo "OK: namespace deleted"
ip link show veth-host 2>/dev/null && echo "WARN: veth still exists" || echo "OK: veth deleted"

# Remove test bundles
rm -rf ~/oci-netns-test ~/oci-newnetns-test
```

## Common Errors

### 1. "namespace path does not exist"

**Symptom**:
```
container_linux.go:XXX: starting container process caused:
    namespace path "/var/run/netns/container-net" does not exist
```

**Cause**: The network namespace wasn't created, or the path is wrong.

**Fix**:
```bash
# Verify namespace exists
ls -la /var/run/netns/container-net

# If not, create it
sudo ip netns add container-net

# Note: typos are common - check the exact name
ip netns list
```

### 2. No connectivity despite correct setup

**Symptom**: Container starts but can't reach host.

**Causes**:
- Interfaces not brought up
- IP addresses not assigned
- Wrong IP addresses or subnets

**Fix**:
```bash
# Check interface states (should be UP)
ip link show veth-host
sudo ip netns exec container-net ip link show veth-container

# Bring up if DOWN
sudo ip link set veth-host up
sudo ip netns exec container-net ip link set veth-container up

# Verify IP addresses match
ip addr show veth-host
sudo ip netns exec container-net ip addr show veth-container

# Ensure they're on same subnet (10.0.0.1/24 and 10.0.0.2/24)
```

### 3. "permission denied" accessing namespace

**Symptom**:
```
open /var/run/netns/container-net: permission denied
```

**Cause**: Running runc without root privileges.

**Fix**:
```bash
# Always run runc with sudo
sudo runc run mycontainer

# The namespace file is owned by root
ls -la /var/run/netns/
```

### 4. Container can't reach external networks

**Symptom**: Container can ping host (10.0.0.1) but not external IPs (8.8.8.8).

**Cause**: Missing NAT/routing configuration on host.

**Fix**: Follow the NAT setup from `docs/01-namespaces/08-netns-nat.md`:

```bash
# Enable IP forwarding
echo 1 | sudo tee /proc/sys/net/ipv4/ip_forward

# Add NAT rule (adjust eth0 to your outbound interface)
sudo iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o eth0 -j MASQUERADE

# Add default route in namespace
sudo ip netns exec container-net ip route add default via 10.0.0.1
```

### 5. "veth-container: No such device" after moving to namespace

**Symptom**: Can't find the interface after `ip link set ... netns`.

**Cause**: This is expected - the interface is now only visible from inside the namespace.

**Fix**:
```bash
# Wrong (from host):
ip link show veth-container  # Fails

# Right (from inside namespace):
sudo ip netns exec container-net ip link show veth-container  # Works
```

### 6. Container shows wrong interfaces

**Symptom**: Container shows host interfaces instead of namespace interfaces.

**Cause**: Network namespace path is incorrect or runc didn't join it.

**Fix**:
```bash
# Verify config.json has correct path
cat config.json | jq '.linux.namespaces[] | select(.type == "network")'

# Should show:
# {
#   "type": "network",
#   "path": "/var/run/netns/container-net"
# }

# If path is missing, container creates new namespace
# If path is wrong, runc fails to start
```

## Notes

### How This Relates to Docker and Kubernetes

**Docker**:
```bash
# Docker creates network namespace automatically
docker run --network bridge alpine ip addr
# Uses veth pairs attached to docker0 bridge
```

Behind the scenes, Docker:
1. Creates network namespace before container starts
2. Creates veth pair
3. Attaches one end to docker0 bridge
4. Moves other end into namespace
5. Starts container with `--net=/proc/[pid]/ns/net`

**Kubernetes**:
```yaml
# Pod spec doesn't mention networking directly
apiVersion: v1
kind: Pod
metadata:
  name: my-pod
spec:
  containers:
  - name: app
    image: nginx
```

Kubernetes networking flow:
1. Kubelet calls CNI plugin (Calico, Flannel, etc.)
2. CNI creates and configures network namespace
3. CNI returns namespace path to kubelet
4. Kubelet tells containerd/CRI-O to use that namespace
5. runc starts with `"path": "/var/run/netns/[cni-generated-name]"`

### The CNI Standard

The Container Network Interface (CNI) is the standard for container networking:

```json
{
  "cniVersion": "1.0.0",
  "name": "mynet",
  "type": "bridge",
  "bridge": "cni0",
  "ipam": {
    "type": "host-local",
    "subnet": "10.22.0.0/16"
  }
}
```

CNI plugins create namespaces exactly like we did manually:
1. Create namespace at `/var/run/netns/[id]`
2. Create veth pair
3. Configure interfaces and routing
4. Return results to runtime

### Sharing Network Namespaces

Containers can share network namespaces (like Kubernetes pods):

```json
{
  "linux": {
    "namespaces": [
      { "type": "pid" },
      { "type": "network", "path": "/var/run/netns/shared-pod-net" },
      { "type": "mount" }
    ]
  }
}
```

Multiple containers with the same `network.path` share:
- Network interfaces
- IP addresses
- Port bindings
- localhost communication

This is how Kubernetes pods work - all containers in a pod share one network namespace.

### OCI Runtime Spec Reference

The namespace configuration is defined in the [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#namespaces):

```
namespaces (array of objects, OPTIONAL) - set of namespaces for the container

    type (string, REQUIRED) - namespace type
    path (string, OPTIONAL) - namespace file path
```

Valid types: `pid`, `network`, `mount`, `ipc`, `uts`, `user`, `cgroup`

### Performance Considerations

Creating network namespaces and veth pairs is fast (microseconds). The overhead comes from:
- IP routing decisions
- NAT translation
- Bridge forwarding

For high-performance networking:
- Consider SR-IOV (hardware passthrough)
- Use MACVLAN/IPVLAN for direct host network access
- Evaluate eBPF-based solutions (Cilium)

## Next

`07-cgroups-integration.md` - Apply resource limits to OCI containers using cgroups, connecting the isolation work from section 02 with runc.
