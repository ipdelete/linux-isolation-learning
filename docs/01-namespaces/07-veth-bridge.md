# Virtual Ethernet Pairs and Bridges

## Goal

Learn how to connect network namespaces to the host using virtual ethernet (veth) devices and Linux bridges. You will implement the `veth` subcommand in `netns-tool` that creates a veth pair, moves one end into a network namespace, and sets up a bridge on the host side for connecting multiple containers.

**What you will build**: Working tests and implementation for `netns-tool veth` that creates connected veth pairs across namespaces and sets up bridge networking.

**Estimated time**: 45-50 minutes

## Prereqs

- Completed `06-netns-basics.md` (network namespace creation and loopback setup)
- `sudo` access for network operations
- Understanding of IP addressing basics (what 10.0.0.1/24 means)

## Concepts: Virtual Ethernet and Bridges

Before writing code, let's understand how container networking actually works under the hood.

### What is a veth Pair?

A **veth (virtual ethernet)** device is like a virtual network cable with two ends. Anything that goes into one end comes out the other. They always come in pairs and are the fundamental building block of container networking.

Think of it as a patch cable connecting two network interfaces:

```
+------------------+                    +------------------+
|   Host Network   |                    | Network Namespace|
|                  |                    |                  |
|   veth0 <--------+-------- veth -----+--------> veth1   |
|   10.0.0.1       |      (pair)        |       10.0.0.2   |
+------------------+                    +------------------+
```

When you create a veth pair:
1. Two network interfaces are created: `veth0` and `veth1` (names are configurable)
2. Initially both are in the same network namespace
3. You move one end into a different namespace
4. Now the two namespaces can communicate through this virtual cable

### What is a Linux Bridge?

A **bridge** acts like a virtual network switch. It connects multiple network interfaces together so they can all communicate as if they were on the same physical network.

```
+------------------+
|   Host Network   |
|                  |
|    br0 (bridge)  |  <-- Acts like a virtual switch
|    10.0.0.1      |
|       |          |
|   +---+---+      |
+---|-------|------+
    |       |
    |       +----------------------+
    |                              |
veth-ns1                      veth-ns2
    |                              |
+---+---+                      +---+---+
|  ns1  |                      |  ns2  |
| veth1 |                      | veth1 |
|10.0.0.2|                     |10.0.0.3|
+-------+                      +-------+
```

Why use a bridge?
- Connect multiple containers to the same virtual network
- Allow containers to talk to each other
- Provide a single point to manage routing and firewalling
- Bridge can have its own IP address for host-to-container communication

### The veth + Bridge Pattern

This is the standard container networking pattern:

1. Create a bridge on the host (e.g., `br0`)
2. For each container:
   - Create a veth pair (e.g., `veth-ns1` and `veth1`)
   - Keep host end (`veth-ns1`) in the host namespace
   - Move container end (`veth1`) into the container's network namespace
   - Attach host end to the bridge
   - Assign IP addresses
   - Bring all interfaces up

This is exactly what Docker, Podman, and Kubernetes do (though they add additional layers for security and routing).

### Why This Matters

Understanding veth pairs and bridges is crucial because:
- They're the foundation of all container networking
- You'll debug networking issues by inspecting these devices
- Advanced features (NAT, port forwarding, service meshes) build on this pattern
- It's the difference between a container that can't reach anything and one that works perfectly

## Write Tests (Red)

**Test file**: `crates/netns-tool/tests/veth_test.rs`

The test file already has scaffolding with TODOs. You'll implement three tests to verify veth pair functionality.

### What the Tests Should Verify

1. **Success case - Basic veth creation**:
   - Create a network namespace
   - Create a veth pair with one end in each namespace
   - Verify both interfaces exist in their respective namespaces
   - Verify they have the expected names

2. **Success case - Connectivity**:
   - Create veth pair with IP addresses assigned
   - Bring both interfaces up
   - Ping from host to namespace IP
   - Should succeed if configuration is correct

3. **Error case - Nonexistent namespace**:
   - Try to create veth pair targeting a namespace that doesn't exist
   - Should fail gracefully with an error message

### Steps

1. Review the existing test scaffolding:

```bash
cat crates/netns-tool/tests/veth_test.rs
```

You'll see three test functions with `todo!()` markers and detailed hints.

2. Open `crates/netns-tool/tests/veth_test.rs` in your editor.

3. Implement the first test (`test_create_veth_pair`):

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;

#[test]
fn test_create_veth_pair() {
    // Setup: Create a test namespace
    let ns_name = "test-veth-ns";

    // Clean up any leftover namespace from previous test runs
    let _ = StdCommand::new("ip")
        .args(["netns", "delete", ns_name])
        .output();

    // Create the namespace
    Command::cargo_bin("netns-tool")
        .unwrap()
        .args(["create", ns_name])
        .assert()
        .success();

    // Test: Create veth pair
    // host end: veth0, namespace end: veth1
    let mut cmd = Command::cargo_bin("netns-tool").unwrap();
    cmd.args(["veth", "veth0", ns_name, "veth1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("veth pair created"));

    // Verify: Host end exists in host namespace
    let output = StdCommand::new("ip")
        .args(["link", "show", "veth0"])
        .output()
        .expect("Failed to run ip link show");
    assert!(output.status.success(), "veth0 should exist on host");

    // Verify: Namespace end exists in target namespace
    let output = StdCommand::new("ip")
        .args(["netns", "exec", ns_name, "ip", "link", "show", "veth1"])
        .output()
        .expect("Failed to check veth1 in namespace");
    assert!(output.status.success(), "veth1 should exist in namespace");

    // Cleanup
    Command::cargo_bin("netns-tool")
        .unwrap()
        .args(["delete", ns_name])
        .assert()
        .success();

    // Clean up veth (deleting namespace should clean it up, but be explicit)
    let _ = StdCommand::new("ip")
        .args(["link", "delete", "veth0"])
        .output();
}
```

4. Implement the second test (`test_veth_connectivity`). Remove the `#[ignore]` attribute first:

```rust
#[test]
fn test_veth_connectivity() {
    // Setup: Create namespace
    let ns_name = "test-veth-ping";

    let _ = StdCommand::new("ip")
        .args(["netns", "delete", ns_name])
        .output();

    Command::cargo_bin("netns-tool")
        .unwrap()
        .args(["create", ns_name])
        .assert()
        .success();

    // Create veth pair
    Command::cargo_bin("netns-tool")
        .unwrap()
        .args(["veth", "veth-host", ns_name, "veth-ns"])
        .assert()
        .success();

    // Assign IP addresses
    // Host end: 10.200.1.1/24
    StdCommand::new("ip")
        .args(["addr", "add", "10.200.1.1/24", "dev", "veth-host"])
        .output()
        .expect("Failed to assign IP to host veth");

    // Namespace end: 10.200.1.2/24
    StdCommand::new("ip")
        .args([
            "netns", "exec", ns_name,
            "ip", "addr", "add", "10.200.1.2/24", "dev", "veth-ns"
        ])
        .output()
        .expect("Failed to assign IP to namespace veth");

    // Bring interfaces up
    StdCommand::new("ip")
        .args(["link", "set", "veth-host", "up"])
        .output()
        .expect("Failed to bring up host veth");

    StdCommand::new("ip")
        .args(["netns", "exec", ns_name, "ip", "link", "set", "veth-ns", "up"])
        .output()
        .expect("Failed to bring up namespace veth");

    // Test: Ping from host to namespace
    let output = StdCommand::new("ping")
        .args(["-c", "1", "-W", "1", "10.200.1.2"])
        .output()
        .expect("Failed to ping");

    assert!(
        output.status.success(),
        "Should be able to ping namespace IP through veth pair"
    );

    // Cleanup
    let _ = StdCommand::new("ip").args(["link", "delete", "veth-host"]).output();
    Command::cargo_bin("netns-tool")
        .unwrap()
        .args(["delete", ns_name])
        .assert()
        .success();
}
```

5. Implement the third test (`test_veth_to_nonexistent_namespace_fails`). Remove the `#[ignore]` attribute:

```rust
#[test]
fn test_veth_to_nonexistent_namespace_fails() {
    let ns_name = "nonexistent-namespace-12345";

    // Make sure it doesn't exist
    let _ = StdCommand::new("ip")
        .args(["netns", "delete", ns_name])
        .output();

    // Try to create veth to non-existent namespace
    let mut cmd = Command::cargo_bin("netns-tool").unwrap();
    cmd.args(["veth", "veth-test", ns_name, "veth1"])
        .assert()
        .failure()  // Should fail
        .stderr(predicate::str::contains("namespace").or(
            predicate::str::contains("not found")
        ));
}
```

6. Run the tests (expect failures):

```bash
cargo test -p netns-tool --test veth_test
```

**Expected output**: Tests panic with `todo!()` or fail because the `veth` subcommand implementation is missing (RED phase).

```
running 3 tests
test test_create_veth_pair ... FAILED
test test_veth_connectivity ... FAILED
test test_veth_to_nonexistent_namespace_fails ... FAILED

failures:
    test_create_veth_pair - explicit panic with 'not yet implemented: Implement veth pair creation'
```

This is expected! You've successfully entered the RED phase of TDD.

## Build (Green)

**Implementation file**: `crates/netns-tool/src/main.rs`
**TODO location**: Line ~74 in the `Command::Veth` match arm

Now let's implement the veth creation logic to make the tests pass.

### Understanding the Implementation Approach

We have two main approaches for creating veth pairs in Rust:

1. **Using `rtnetlink` crate**: Pure Rust, async API, type-safe
2. **Using `std::process::Command` to call `ip`**: Simpler, synchronous, relies on system tools

For this tutorial, we'll use the `ip` command approach because:
- It's simpler to understand and debug
- It matches how you'd do it manually
- The `rtnetlink` crate requires async runtime setup (Tokio)
- You can see exactly what commands are being executed

Later lessons or advanced challenges can refactor to use `rtnetlink` for a pure-Rust solution.

### Steps

1. Open `crates/netns-tool/src/main.rs`

2. First, update the `Command::Veth` variant definition to accept three arguments (around line 16):

```rust
#[derive(Subcommand)]
enum Command {
    Create { name: String },
    Delete { name: String },
    Veth {
        host_if: String,      // Host-side interface name
        ns_name: String,      // Target namespace name
        ns_if: String,        // Namespace-side interface name
    },
    Bridge { name: String },
    Nat { bridge: String, outbound: String },
}
```

3. Find the `Command::Veth` match arm (around line 74) and replace the `todo!()` with this implementation:

```rust
Command::Veth { host_if, ns_name, ns_if } => {
    println!("Creating veth pair: {} (host) <-> {} (namespace {})", host_if, ns_if, ns_name);

    // Step 1: Verify the target namespace exists
    let ns_path = format!("/run/netns/{}", ns_name);
    if !std::path::Path::new(&ns_path).exists() {
        anyhow::bail!(
            "Network namespace '{}' not found. Create it first with: netns-tool create {}",
            ns_name, ns_name
        );
    }

    // Step 2: Create the veth pair (both ends start in host namespace)
    let output = std::process::Command::new("ip")
        .args(["link", "add", &host_if, "type", "veth", "peer", "name", &ns_if])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create veth pair: {}", stderr);
    }

    println!("  Created veth pair: {} <-> {}", host_if, ns_if);

    // Step 3: Move the namespace end into the target network namespace
    let output = std::process::Command::new("ip")
        .args(["link", "set", &ns_if, "netns", &ns_name])
        .output()?;

    if !output.status.success() {
        // Cleanup: delete the veth pair we just created
        let _ = std::process::Command::new("ip")
            .args(["link", "delete", &host_if])
            .output();

        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to move {} to namespace: {}", ns_if, stderr);
    }

    println!("  Moved {} into namespace {}", ns_if, ns_name);
    println!("veth pair created successfully!");
    println!("\nNext steps:");
    println!("  1. Assign IP to host side:    sudo ip addr add <IP/CIDR> dev {}", host_if);
    println!("  2. Bring up host side:        sudo ip link set {} up", host_if);
    println!("  3. Assign IP in namespace:    sudo ip netns exec {} ip addr add <IP/CIDR> dev {}", ns_name, ns_if);
    println!("  4. Bring up in namespace:     sudo ip netns exec {} ip link set {} up", ns_name, ns_if);

    Ok(())
}
```

**Understanding the code:**

- **Step 1**: We check if the namespace exists by looking for `/run/netns/{name}`. This gives a clear error message if the user forgot to create the namespace first.

- **Step 2**: `ip link add <name> type veth peer name <peer-name>` creates both ends of the veth pair. Initially both are in the host network namespace.

- **Step 3**: `ip link set <interface> netns <namespace>` moves one end into the target namespace. If this fails, we clean up the veth pair we just created (avoid leaving orphaned interfaces).

- **Output**: We print helpful next steps for the user, showing exactly how to configure IP addresses and bring interfaces up.

4. Run the tests:

```bash
cargo test -p netns-tool --test veth_test
```

**Expected output**: All tests should pass (GREEN phase).

```
running 3 tests
test test_create_veth_pair ... ok
test test_veth_connectivity ... ok
test test_veth_to_nonexistent_namespace_fails ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Congratulations! You've successfully implemented veth pair creation using TDD.

## Verify

**Automated verification**:
```bash
# Run all netns-tool tests
cargo test -p netns-tool

# Run just the veth tests with output
cargo test -p netns-tool --test veth_test -- --nocapture
```

**Manual verification** (observe the actual behavior):

Let's manually create a veth pair and explore it step by step.

```bash
# 1. Create a test namespace
sudo cargo run -p netns-tool -- create demo-ns

# 2. Create a veth pair
sudo cargo run -p netns-tool -- veth veth-demo demo-ns veth-ns

# What you should see:
# Creating veth pair: veth-demo (host) <-> veth-ns (namespace demo-ns)
#   Created veth pair: veth-demo <-> veth-ns
#   Moved veth-ns into namespace demo-ns
# veth pair created successfully!
#
# Next steps:
#   1. Assign IP to host side:    sudo ip addr add <IP/CIDR> dev veth-demo
#   2. Bring up host side:        sudo ip link set veth-demo up
#   3. Assign IP in namespace:    sudo ip netns exec demo-ns ip addr add <IP/CIDR> dev veth-ns
#   4. Bring up in namespace:     sudo ip netns exec demo-ns ip link set veth-ns up
```

**Inspect the veth pair:**

```bash
# 3. Check the host-side interface
ip link show veth-demo

# You should see:
# XX: veth-demo@if[YY]: <BROADCAST,MULTICAST> mtu 1500 qdisc noop state DOWN mode DEFAULT
#     link/ether [MAC] brd ff:ff:ff:ff:ff:ff link-netns demo-ns

# Note:
# - State is DOWN (we haven't brought it up yet)
# - "link-netns demo-ns" shows it's paired with an interface in that namespace
# - The "@ifYY" shows the peer interface index

# 4. Check the namespace-side interface
sudo ip netns exec demo-ns ip link show veth-ns

# You should see:
# YY: veth-ns@if[XX]: <BROADCAST,MULTICAST> mtu 1500 qdisc noop state DOWN mode DEFAULT
#     link/ether [MAC] brd ff:ff:ff:ff:ff:ff link-netnsid 0

# Note:
# - Also DOWN initially
# - "@ifXX" references the peer (veth-demo) by its index
# - "link-netnsid 0" means it's paired with an interface in another namespace

# 5. Assign IP addresses and bring interfaces up
sudo ip addr add 10.100.1.1/24 dev veth-demo
sudo ip link set veth-demo up

sudo ip netns exec demo-ns ip addr add 10.100.1.2/24 dev veth-ns
sudo ip netns exec demo-ns ip link set veth-ns up

# 6. Verify connectivity
ping -c 3 10.100.1.2

# You should see:
# PING 10.100.1.2 (10.100.1.2) 56(84) bytes of data.
# 64 bytes from 10.100.1.2: icmp_seq=1 ttl=64 time=0.XXX ms
# 64 bytes from 10.100.1.2: icmp_seq=2 ttl=64 time=0.XXX ms
# 64 bytes from 10.100.1.2: icmp_seq=3 ttl=64 time=0.XXX ms

# Success! The host can reach the namespace through the veth pair.

# 7. Check from the namespace side
sudo ip netns exec demo-ns ping -c 3 10.100.1.1

# Should also succeed - connectivity is bidirectional

# 8. View routing table in namespace
sudo ip netns exec demo-ns ip route

# You should see:
# 10.100.1.0/24 dev veth-ns proto kernel scope link src 10.100.1.2
# This route was automatically added when we assigned the IP address
```

**Understanding what you're seeing:**

- The veth pair creates a point-to-point link between namespaces
- Each end has its own MAC address (like a real network card)
- The kernel automatically adds routes for the subnet
- Traffic sent to `10.100.1.2` from host goes through `veth-demo` and emerges from `veth-ns` in the namespace
- This is exactly how Docker creates per-container networking

## Clean Up

```bash
# Delete the veth pair (deletes both ends)
sudo ip link delete veth-demo

# Delete the namespace
sudo cargo run -p netns-tool -- delete demo-ns

# Verify cleanup
ip link show veth-demo  # Should fail: "does not exist"
sudo ip netns list | grep demo-ns  # Should return nothing
```

**Note**: Deleting either end of a veth pair automatically deletes the other end. Deleting the namespace also removes any interfaces inside it.

## Common Errors

1. **Error: "RTNETLINK answers: File exists" when creating veth**
   - Cause: An interface with that name already exists (leftover from previous run)
   - Fix: Delete the existing interface first:
     ```bash
     sudo ip link delete veth0
     ```
   - Prevention: Always clean up in tests and manual experiments

2. **Error: "Cannot find device 'veth-ns'" when trying to configure it from host**
   - Cause: You moved `veth-ns` into the namespace, so it's no longer visible from the host
   - Fix: Use `ip netns exec <namespace> ip ...` to run commands inside the namespace:
     ```bash
     sudo ip netns exec demo-ns ip link set veth-ns up
     ```
   - Understanding: Each namespace has its own network stack; interfaces in one namespace are invisible to another

3. **Ping fails with "Network is unreachable" even though veth exists**
   - Cause: One or both interfaces are still DOWN, or IP addresses weren't assigned correctly
   - Fix: Check status and bring interfaces up:
     ```bash
     ip link show veth-demo  # Should show "state UP"
     sudo ip netns exec demo-ns ip link show veth-ns  # Should also be UP
     ip addr show veth-demo  # Should show IP address
     sudo ip netns exec demo-ns ip addr show veth-ns  # Should show IP
     ```
   - If DOWN, bring them up:
     ```bash
     sudo ip link set veth-demo up
     sudo ip netns exec demo-ns ip link set veth-ns up
     ```

4. **Error: "RTNETLINK answers: Invalid argument" when moving interface to namespace**
   - Cause: Target namespace doesn't exist, or typo in namespace name
   - Fix: Verify namespace exists:
     ```bash
     sudo ip netns list
     ```
   - The namespace must be created before you can move interfaces into it

5. **Veth pair gets deleted when cleaning up namespace**
   - Cause: This is expected behavior - interfaces in a namespace are deleted when the namespace is deleted
   - Fix: If you want to keep the host-side interface, move it to another namespace before deleting:
     ```bash
     sudo ip netns exec demo-ns ip link set veth-ns netns 1  # Move to init (PID 1) namespace
     ```
   - Understanding: This automatic cleanup is actually helpful - no orphaned interfaces

6. **Test failure: "veth1 should exist in namespace" but interface doesn't appear**
   - Cause: The interface name arguments were passed in the wrong order
   - Fix: Remember the order: `veth <host-if> <namespace> <ns-if>`
     - First argument: host-side interface name
     - Second argument: namespace name
     - Third argument: namespace-side interface name
   - The implementation moves the third argument (ns-if) into the namespace, leaving the first argument (host-if) on the host

## Notes

### Why veth Pairs Matter

- **Foundation of container networking**: Every container runtime (Docker, containerd, Podman) uses veth pairs to connect containers to the host network
- **Performance**: veth is implemented entirely in software; data just moves between kernel memory structures (no hardware involved)
- **Flexibility**: You can attach veth ends to bridges, route between them, apply firewall rules, rate limit traffic, etc.
- **Debugging**: When container networking breaks, you'll inspect veth pairs to diagnose the issue

### Veth Naming Conventions

Common patterns you'll see in the wild:

- Docker: `veth<random-hash>` on host side (e.g., `veth7a3b1f2`)
- Kubernetes: `veth<pod-id>` patterns
- Custom: Often prefix with purpose: `veth-container1`, `vethbr0-ns1`

Choose descriptive names in production to make debugging easier.

### Performance Characteristics

- **Latency**: Very low (just memory copies within kernel)
- **Throughput**: Can handle 10+ Gbps depending on hardware
- **Overhead**: Minimal CPU usage for typical workloads
- **Limitations**: Each veth pair only connects two network namespaces (use bridges for more)

### Bridge Setup (Next Lesson Preview)

This lesson focused on veth pairs. The next lesson will cover creating and using bridges to connect multiple containers:

```bash
# Create a bridge
sudo ip link add br0 type bridge

# Attach veth host-end to bridge
sudo ip link set veth-demo master br0

# Bring bridge up
sudo ip link set br0 up

# Now multiple veth pairs can all attach to br0
```

This will be covered in detail in `08-netns-nat.md`, where we'll set up NAT for external connectivity.

### Alternative Approaches: Pure Rust Implementation

While we used `ip` commands for simplicity, production tools often use:

- **rtnetlink crate**: Pure Rust async API for netlink (requires Tokio or async-std)
  ```rust
  use rtnetlink::{new_connection, Handle};
  use futures::stream::TryStreamExt;

  // Example (async context required)
  let (connection, handle, _) = new_connection().unwrap();
  tokio::spawn(connection);

  handle.link()
      .add()
      .veth("veth0".into(), "veth1".into())
      .execute()
      .await?;
  ```

- **netlink-sys**: Lower-level netlink socket protocol
- **nix crate**: May add higher-level wrappers in future versions

For learning, the `ip` command approach is clearest. For production Rust projects, consider `rtnetlink` to avoid spawning processes.

**Challenge**: After completing this lesson, try refactoring the implementation to use `rtnetlink` instead of the `ip` command. You'll need to add Tokio to dependencies and convert the function to async.

### Related Man Pages

- `man 8 ip-link` - Network device configuration (essential reference)
- `man 8 ip-netns` - Network namespace management
- `man 7 veth` - Virtual ethernet device pairs (detailed kernel documentation)
- `man 8 bridge` - Bridge administration

### Kernel Support

- Veth devices: Available since Linux 2.6.23 (2007)
- Network namespaces: Linux 2.6.24 (2008)
- Both are extremely stable and widely used in production

All modern Linux distributions support these features out of the box.

## Next

`08-netns-nat.md` - Set up NAT and routing to give namespaces internet access via the host's network connection
