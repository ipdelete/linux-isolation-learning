# 08 Network Namespace NAT

## Goal
Configure Network Address Translation (NAT) to give a network namespace full internet connectivity, allowing processes inside the namespace to reach external servers.

**Deliverable**: A `nat` subcommand in `netns-tool` that enables IP forwarding, configures iptables MASQUERADE rules, and sets up routing for namespace internet access.

## Prereqs
- Completed `07-veth-bridge.md` (veth pairs and bridges working)
- Understanding of basic networking (IP addresses, routing, DNS)
- `sudo` access (required for iptables and IP forwarding)

**Estimated time**: ~50 minutes

## Concepts

### The Problem: Isolated Networks Can't Reach the Internet

After completing the previous lessons, your network namespace has:
- Its own network stack (from `06-netns-basics.md`)
- A virtual interface connected to a bridge (from `07-veth-bridge.md`)
- An IP address that can communicate with the host bridge

However, **the namespace still cannot reach the internet**. Try it:

```bash
# From inside a namespace (if you had shell access)
ping 8.8.8.8  # Google's DNS - will fail!
```

Why does this fail? Three missing pieces:

1. **No default route**: The namespace doesn't know where to send traffic for external IPs
2. **No IP forwarding**: The host kernel won't forward packets between interfaces
3. **No NAT**: External routers don't know how to route back to private namespace IPs

### The Solution: NAT and Routing

Network Address Translation (NAT) solves this by making all namespace traffic appear to come from the host's IP address. Here's how:

```
Namespace (10.0.0.2)  →  Bridge (10.0.0.1)  →  Host routing  →  Internet
                             ↑
                             NAT happens here:
                             10.0.0.2 → host's public IP
```

When a reply comes back, NAT reverses the translation:

```
Internet  →  Host routing  →  NAT  →  Bridge  →  Namespace
                            ↓
                     host IP → 10.0.0.2
```

### Three Required Components

#### 1. Default Route in Namespace

The namespace needs to know "for any IP I don't have a specific route for, send traffic to the bridge":

```bash
# Inside namespace
ip route add default via 10.0.0.1
```

This tells the namespace: "The bridge (10.0.0.1) is my gateway to the world."

#### 2. IP Forwarding in Host Kernel

By default, Linux doesn't forward packets between network interfaces (for security). Enable it:

```bash
# On host
echo 1 > /proc/sys/net/ipv4/ip_forward
```

Now packets arriving at one interface can be routed to another.

#### 3. iptables MASQUERADE Rule

This performs the actual NAT, rewriting source IPs for outbound traffic:

```bash
# On host
iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o eth0 -j MASQUERADE
```

Breaking this down:
- `-t nat`: Use the NAT table (not the default filter table)
- `-A POSTROUTING`: Add to POSTROUTING chain (after routing decision)
- `-s 10.0.0.0/24`: Match packets from our bridge subnet
- `-o eth0`: Going out the host's main interface (adjust to your interface name)
- `-j MASQUERADE`: Replace source IP with outbound interface's IP

MASQUERADE is a special form of SNAT (Source NAT) that automatically uses the interface's current IP, handling DHCP changes gracefully.

### Why POSTROUTING?

iptables has several chains where rules can be applied:

```
       PREROUTING → routing decision → FORWARD → POSTROUTING → out
                          ↓
                    local process
                          ↓
                        OUTPUT
```

We use POSTROUTING because:
- Routing has already decided the packet goes out (not to a local process)
- We want to change the source IP just before transmission
- The kernel needs to track connection state for return packets

### DNS Configuration

Even with NAT and routing, DNS lookups will fail without a resolver configuration. The namespace needs `/etc/resolv.conf`:

```bash
# Create minimal resolv.conf in namespace
echo "nameserver 8.8.8.8" > /etc/resolv.conf
```

In a real container with its own mount namespace, you'd bind-mount or copy the host's `/etc/resolv.conf`. For network namespaces alone, we can use the global one (shared across network namespaces).

### Security Note: Forwarding Rules

For production systems, you should add FORWARD rules to control what traffic can be forwarded:

```bash
# Allow forwarding for our bridge
iptables -A FORWARD -i br0 -j ACCEPT
iptables -A FORWARD -o br0 -j ACCEPT
```

This prevents the namespace from becoming an unintended route for other traffic.

## Write Tests (Red)

**Test file**: `crates/netns-tool/tests/nat_test.rs`

The test file already exists with TODO placeholders. We'll implement the tests following TDD principles.

### Step 1: Understand the test structure

Open `crates/netns-tool/tests/nat_test.rs` and review the three test placeholders:
1. `test_setup_nat` - Verifies NAT configuration (IP forwarding and iptables)
2. `test_namespace_internet_access` - Integration test for actual internet connectivity
3. `test_nat_cleanup` - Verifies NAT rules can be removed

### Step 2: Implement the basic NAT setup test

Replace the `test_setup_nat` function:

```rust
#[test]
fn test_setup_nat() {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::process::Command as StdCommand;

    // This test requires root and modifies iptables
    // It verifies that the nat subcommand enables IP forwarding and sets up MASQUERADE

    // First, get the default network interface name
    let output = StdCommand::new("ip")
        .args(["route", "show", "default"])
        .output()
        .expect("Failed to get default route");
    let route_output = String::from_utf8_lossy(&output.stdout);

    // Extract interface name from "default via X.X.X.X dev INTERFACE"
    let interface = route_output
        .split_whitespace()
        .skip_while(|&s| s != "dev")
        .nth(1)
        .unwrap_or("eth0");

    // Create a test bridge first (needed for NAT setup)
    let bridge_name = "br_nat_test";
    let _ = StdCommand::new("ip")
        .args(["link", "del", bridge_name])
        .output(); // Clean up if exists

    StdCommand::new("ip")
        .args(["link", "add", bridge_name, "type", "bridge"])
        .output()
        .expect("Failed to create test bridge");

    StdCommand::new("ip")
        .args(["addr", "add", "10.0.99.1/24", "dev", bridge_name])
        .output()
        .expect("Failed to add IP to bridge");

    StdCommand::new("ip")
        .args(["link", "set", bridge_name, "up"])
        .output()
        .expect("Failed to bring up bridge");

    // Now test the nat subcommand
    let mut cmd = Command::cargo_bin("netns-tool").unwrap();
    cmd.arg("nat")
        .arg(bridge_name)
        .arg(interface)
        .assert()
        .success()
        .stdout(predicate::str::contains("NAT configured"));

    // Verify IP forwarding is enabled
    let forwarding = std::fs::read_to_string("/proc/sys/net/ipv4/ip_forward")
        .expect("Failed to read ip_forward");
    assert_eq!(forwarding.trim(), "1", "IP forwarding should be enabled");

    // Verify iptables MASQUERADE rule exists
    let iptables_output = StdCommand::new("iptables")
        .args(["-t", "nat", "-L", "POSTROUTING", "-v", "-n"])
        .output()
        .expect("Failed to list iptables rules");
    let rules = String::from_utf8_lossy(&iptables_output.stdout);

    assert!(
        rules.contains("MASQUERADE") && rules.contains("10.0.99.0/24"),
        "iptables MASQUERADE rule should exist for bridge subnet"
    );

    // Cleanup
    let _ = StdCommand::new("iptables")
        .args(["-t", "nat", "-D", "POSTROUTING", "-s", "10.0.99.0/24", "-o", interface, "-j", "MASQUERADE"])
        .output();

    let _ = StdCommand::new("ip")
        .args(["link", "del", bridge_name])
        .output();
}
```

### Step 3: Implement the cleanup test

This test is simpler - it verifies rules can be removed. Replace `test_nat_cleanup`:

```rust
#[test]
#[ignore] // Remove this after implementing cleanup functionality
fn test_nat_cleanup() {
    use assert_cmd::Command;
    use std::process::Command as StdCommand;

    let bridge_name = "br_cleanup_test";
    let interface = "eth0"; // Adjust if needed

    // Setup: Create bridge and NAT rules
    let _ = StdCommand::new("ip")
        .args(["link", "add", bridge_name, "type", "bridge"])
        .output();

    StdCommand::new("ip")
        .args(["addr", "add", "10.0.98.1/24", "dev", bridge_name])
        .output()
        .expect("Failed to add IP");

    // Add NAT rule manually (simulating what nat subcommand does)
    StdCommand::new("iptables")
        .args(["-t", "nat", "-A", "POSTROUTING", "-s", "10.0.98.0/24", "-o", interface, "-j", "MASQUERADE"])
        .output()
        .expect("Failed to add NAT rule");

    // Test cleanup (future enhancement - nat --delete or similar)
    // For now, just verify we can delete manually
    let cleanup = StdCommand::new("iptables")
        .args(["-t", "nat", "-D", "POSTROUTING", "-s", "10.0.98.0/24", "-o", interface, "-j", "MASQUERADE"])
        .output()
        .expect("Failed to delete rule");

    assert!(cleanup.status.success(), "Should be able to delete NAT rule");

    // Verify rule is gone
    let list_output = StdCommand::new("iptables")
        .args(["-t", "nat", "-L", "POSTROUTING", "-v", "-n"])
        .output()
        .expect("Failed to list rules");
    let rules = String::from_utf8_lossy(&list_output.stdout);

    assert!(
        !rules.contains("10.0.98.0/24") || !rules.contains("MASQUERADE"),
        "NAT rule should be deleted"
    );

    // Cleanup bridge
    let _ = StdCommand::new("ip")
        .args(["link", "del", bridge_name])
        .output();
}
```

### Step 4: Leave integration test for later

The `test_namespace_internet_access` test remains `#[ignore]` for now - it requires a complete end-to-end setup and is better suited for manual verification in this lesson.

### Step 5: Run the tests (expect failure)

```bash
cargo test -p netns-tool --test nat_test -- test_setup_nat
```

Expected output: Test fails because the `nat` subcommand is not implemented yet (RED phase).

You'll see errors like:
```
Error: "nat" is not a valid subcommand
```

Or the assertion failures will show missing IP forwarding and iptables rules.

## Build (Green)

**Implementation file**: `crates/netns-tool/src/main.rs`
**TODO location**: Line ~108 in the `Command::Nat` match arm

### Step 1: Add dependencies

First, check `Cargo.toml` has these dependencies (should already be present):

```toml
[dependencies]
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
nix = { version = "0.27", features = ["sched", "mount"] }
```

For iptables, we'll use `std::process::Command` to shell out - this is simpler and more reliable than iptables crates for this educational context.

### Step 2: Implement the NAT setup function

Add this function to `main.rs`, before the `main()` function:

```rust
/// Configure NAT for a bridge to enable internet access from namespaces
///
/// This enables IP forwarding and sets up iptables MASQUERADE for the bridge subnet
fn setup_nat(bridge: &str, outbound: &str) -> Result<()> {
    use std::process::Command;

    println!("Setting up NAT for bridge {} via {}", bridge, outbound);

    // Step 1: Enable IP forwarding
    println!("  Enabling IP forwarding...");
    std::fs::write("/proc/sys/net/ipv4/ip_forward", "1")
        .context("Failed to enable IP forwarding - are you running as root?")?;

    // Step 2: Get bridge subnet from current IP configuration
    // We need to know the subnet to create the iptables rule
    let output = Command::new("ip")
        .args(["-o", "-f", "inet", "addr", "show", bridge])
        .output()
        .context("Failed to get bridge IP address")?;

    if !output.status.success() {
        anyhow::bail!("Bridge {} not found or has no IP address", bridge);
    }

    let ip_output = String::from_utf8_lossy(&output.stdout);

    // Parse output like: "4: br0    inet 10.0.0.1/24 ..."
    let subnet = ip_output
        .split_whitespace()
        .find(|s| s.contains('/'))
        .ok_or_else(|| anyhow::anyhow!("Could not find IP/subnet on bridge {}", bridge))?;

    // Extract just the network part (e.g., "10.0.0.0/24" from "10.0.0.1/24")
    let parts: Vec<&str> = subnet.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid subnet format: {}", subnet);
    }
    let prefix_len = parts[1];

    // Convert IP to network address (simple approach: assume .0 network)
    // For production code, use proper CIDR calculation
    let ip_parts: Vec<&str> = parts[0].split('.').collect();
    if ip_parts.len() != 4 {
        anyhow::bail!("Invalid IP format: {}", parts[0]);
    }
    let network = format!("{}.{}.{}.0/{}", ip_parts[0], ip_parts[1], ip_parts[2], prefix_len);

    println!("  Bridge subnet detected: {}", network);

    // Step 3: Add iptables MASQUERADE rule
    println!("  Configuring iptables MASQUERADE rule...");

    // First check if rule already exists to avoid duplicates
    let check = Command::new("iptables")
        .args(["-t", "nat", "-C", "POSTROUTING", "-s", &network, "-o", outbound, "-j", "MASQUERADE"])
        .output()
        .context("Failed to check iptables rules")?;

    if !check.status.success() {
        // Rule doesn't exist, add it
        let add = Command::new("iptables")
            .args(["-t", "nat", "-A", "POSTROUTING", "-s", &network, "-o", outbound, "-j", "MASQUERADE"])
            .output()
            .context("Failed to add iptables MASQUERADE rule")?;

        if !add.status.success() {
            let stderr = String::from_utf8_lossy(&add.stderr);
            anyhow::bail!("iptables command failed: {}", stderr);
        }
    } else {
        println!("  (MASQUERADE rule already exists)");
    }

    // Step 4: Add FORWARD rules to allow traffic through the bridge
    println!("  Configuring FORWARD rules...");

    // Allow incoming to bridge
    let _ = Command::new("iptables")
        .args(["-C", "FORWARD", "-o", bridge, "-j", "ACCEPT"])
        .output();

    let forward_in = Command::new("iptables")
        .args(["-A", "FORWARD", "-o", bridge, "-j", "ACCEPT"])
        .output();

    // Allow outgoing from bridge
    let _ = Command::new("iptables")
        .args(["-C", "FORWARD", "-i", bridge, "-j", "ACCEPT"])
        .output();

    let forward_out = Command::new("iptables")
        .args(["-A", "FORWARD", "-i", bridge, "-j", "ACCEPT"])
        .output();

    // Don't fail if these rules already exist
    if forward_in.is_err() || forward_out.is_err() {
        println!("  (FORWARD rules may already exist or failed to add - continuing)");
    }

    println!();
    println!("NAT configured successfully!");
    println!();
    println!("Summary:");
    println!("  IP forwarding: enabled");
    println!("  NAT rule: {} -> {} (MASQUERADE)", network, outbound);
    println!("  Forward rules: {} <-> external networks", bridge);
    println!();
    println!("Namespaces on {} can now access the internet", bridge);
    println!();
    println!("Note: To allow namespace DNS resolution, ensure /etc/resolv.conf");
    println!("      is accessible or configure nameservers manually");

    Ok(())
}
```

### Step 3: Update the Command::Nat match arm

Find the `Command::Nat` match arm (around line 108) and replace the `todo!()`:

```rust
        Command::Nat { bridge, outbound } => {
            setup_nat(&bridge, &outbound)?;
        }
```

### Step 4: Add the use statement for context

Near the top of the file, ensure you have:

```rust
use anyhow::{Context, Result};
```

### Step 5: Run the tests (expect success)

```bash
# Run as root since we need to modify iptables
sudo -E cargo test -p netns-tool --test nat_test -- test_setup_nat
```

Expected output: Test passes (GREEN phase).

You should see:
```
test test_setup_nat ... ok
```

## Verify

**Automated verification**:
```bash
sudo -E cargo test -p netns-tool --test nat_test -- test_setup_nat
sudo -E cargo test -p netns-tool  # All netns-tool tests
```

**Manual verification** (observe actual internet connectivity):

### Step 1: Set up a complete test environment

```bash
# Create a network namespace
sudo ip netns add testnet

# Create a bridge
sudo ip link add br0 type bridge
sudo ip addr add 10.0.0.1/24 dev br0
sudo ip link set br0 up

# Create veth pair
sudo ip link add veth0 type veth peer name veth1

# Move one end to namespace
sudo ip link set veth1 netns testnet

# Attach host end to bridge
sudo ip link set veth0 master br0
sudo ip link set veth0 up

# Configure namespace end
sudo ip netns exec testnet ip addr add 10.0.0.2/24 dev veth1
sudo ip netns exec testnet ip link set veth1 up
sudo ip netns exec testnet ip link set lo up
```

### Step 2: Configure NAT

```bash
# Get your main network interface (usually eth0, wlan0, or similar)
ip route show default
# Look for "dev INTERFACE" in the output

# Run our NAT setup (adjust interface name)
sudo cargo run -q -p netns-tool -- nat br0 eth0
```

Expected output:
```
Setting up NAT for bridge br0 via eth0
  Enabling IP forwarding...
  Bridge subnet detected: 10.0.0.0/24
  Configuring iptables MASQUERADE rule...
  Configuring FORWARD rules...

NAT configured successfully!

Summary:
  IP forwarding: enabled
  NAT rule: 10.0.0.0/24 -> eth0 (MASQUERADE)
  Forward rules: br0 <-> external networks

Namespaces on br0 can now access the internet

Note: To allow namespace DNS resolution, ensure /etc/resolv.conf
      is accessible or configure nameservers manually
```

### Step 3: Add default route in namespace

```bash
# The namespace needs to know the bridge is its gateway
sudo ip netns exec testnet ip route add default via 10.0.0.1
```

### Step 4: Verify internet connectivity

```bash
# Test IP connectivity (should work!)
sudo ip netns exec testnet ping -c 3 8.8.8.8

# Expected output:
# PING 8.8.8.8 (8.8.8.8) 56(84) bytes of data.
# 64 bytes from 8.8.8.8: icmp_seq=1 ttl=... time=...
# ...
# 3 packets transmitted, 3 received, 0% packet loss
```

### Step 5: Verify routing and NAT configuration

```bash
# Check IP forwarding
cat /proc/sys/net/ipv4/ip_forward
# Should output: 1

# Check iptables NAT rules
sudo iptables -t nat -L POSTROUTING -v -n
# Should show MASQUERADE rule for 10.0.0.0/24

# Check FORWARD rules
sudo iptables -L FORWARD -v -n | grep br0
# Should show ACCEPT rules for br0

# Check route in namespace
sudo ip netns exec testnet ip route
# Should show:
# default via 10.0.0.1 dev veth1
# 10.0.0.0/24 dev veth1 proto kernel scope link src 10.0.0.2
```

### Step 6: Test DNS resolution (optional)

DNS should work automatically since `/etc/resolv.conf` is shared:

```bash
# Test DNS lookup
sudo ip netns exec testnet ping -c 2 google.com

# Expected: Name resolves and packets are sent
# If this fails, DNS is not configured - see Common Errors section
```

## Clean Up

After testing, remove the created resources:

```bash
# Delete network namespace
sudo ip netns del testnet

# Delete bridge (this also removes veth0)
sudo ip link del br0

# Remove iptables NAT rule (adjust interface name)
sudo iptables -t nat -D POSTROUTING -s 10.0.0.0/24 -o eth0 -j MASQUERADE

# Remove FORWARD rules
sudo iptables -D FORWARD -i br0 -j ACCEPT
sudo iptables -D FORWARD -o br0 -j ACCEPT

# Optionally disable IP forwarding (only if you're sure nothing else needs it)
# echo 0 | sudo tee /proc/sys/net/ipv4/ip_forward
```

**Important**: In production, you'd want a cleanup command or script to remove NAT rules automatically. For now, manual cleanup is sufficient for learning.

## Common Errors

### 1. "Network is unreachable" when pinging from namespace

**Symptom**:
```bash
sudo ip netns exec testnet ping 8.8.8.8
# ping: connect: Network is unreachable
```

**Cause**: No default route configured in the namespace.

**Fix**: Add the default route:
```bash
sudo ip netns exec testnet ip route add default via 10.0.0.1
```

Verify with:
```bash
sudo ip netns exec testnet ip route
# Should show: default via 10.0.0.1 dev veth1
```

### 2. Ping times out, no response from external IPs

**Symptom**: Ping starts but receives no replies.

**Causes**:
- IP forwarding not enabled
- Wrong outbound interface in iptables rule
- Firewall blocking forwarded traffic

**Fix**:
```bash
# Check IP forwarding
cat /proc/sys/net/ipv4/ip_forward
# Should be 1, if not: echo 1 | sudo tee /proc/sys/net/ipv4/ip_forward

# Verify correct interface in NAT rule
ip route show default
# Use the interface shown after "dev"

# Check iptables for MASQUERADE
sudo iptables -t nat -L POSTROUTING -v -n

# Check for blocking rules
sudo iptables -L FORWARD -v -n
# Should show ACCEPT for your bridge, not DROP
```

### 3. "Permission denied" when writing to /proc/sys/net/ipv4/ip_forward

**Symptom**:
```
Error: Failed to enable IP forwarding - are you running as root?
```

**Cause**: Not running with root privileges.

**Fix**: Use `sudo`:
```bash
sudo cargo run -p netns-tool -- nat br0 eth0
```

### 4. DNS lookups fail but ping 8.8.8.8 works

**Symptom**:
```bash
sudo ip netns exec testnet ping 8.8.8.8  # Works
sudo ip netns exec testnet ping google.com  # Fails with "unknown host"
```

**Cause**: `/etc/resolv.conf` is not accessible or empty.

**Fix**: The network namespace shares the mount namespace, so `/etc/resolv.conf` should work. If it doesn't:

```bash
# Check resolv.conf
cat /etc/resolv.conf

# If empty or missing, configure manually
echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf
echo "nameserver 1.1.1.1" | sudo tee -a /etc/resolv.conf

# Or use a public DNS in the ping command
sudo ip netns exec testnet nslookup google.com 8.8.8.8
```

### 5. iptables command not found

**Symptom**:
```
Failed to add iptables MASQUERADE rule
```

**Cause**: iptables not installed, or system uses nftables.

**Fix**:
```bash
# Check if iptables is available
which iptables

# On Debian/Ubuntu
sudo apt install iptables

# On systems using nftables, you can use iptables-nft wrapper
sudo apt install iptables-nft
```

### 6. Duplicate iptables rules accumulating

**Symptom**: Running the NAT setup multiple times creates duplicate rules.

**Cause**: The check command (iptables -C) may fail for various reasons, causing re-addition.

**Fix**: Our code checks for existing rules, but you can manually verify:
```bash
# List rules with line numbers
sudo iptables -t nat -L POSTROUTING --line-numbers -v -n

# Delete specific rule by line number if needed
sudo iptables -t nat -D POSTROUTING 3  # Replace 3 with actual line number
```

### 7. "Cannot find device br0" error

**Symptom**: iptables or ip commands fail saying bridge doesn't exist.

**Cause**: Bridge was not created or was deleted.

**Fix**: Create the bridge before running NAT setup:
```bash
sudo ip link add br0 type bridge
sudo ip addr add 10.0.0.1/24 dev br0
sudo ip link set br0 up
```

## Notes

- **iptables vs nftables**: Modern systems are moving to nftables. The `iptables` command may be a wrapper around nftables (`iptables-nft`). For educational purposes, iptables is more widely documented, but production systems should consider native nftables configuration.

- **Persistent configuration**: Changes made with `ip` and `iptables` commands are **not persistent** across reboots. For permanent configuration, use:
  - `netplan`, `NetworkManager`, or `/etc/network/interfaces` for network config
  - `iptables-persistent` package or `/etc/nftables.conf` for firewall rules
  - `sysctl -w` settings written to `/etc/sysctl.d/` for IP forwarding

- **SNAT vs MASQUERADE**: MASQUERADE is a special form of SNAT (Source NAT) that automatically uses the outbound interface's current IP. Use SNAT with `-j SNAT --to-source IP` for static IPs (slightly more efficient).

- **Connection tracking**: NAT relies on conntrack (connection tracking) in the kernel. View active connections:
  ```bash
  sudo conntrack -L | grep 10.0.0.2
  ```

- **Docker/Podman comparison**: Container engines set up NAT automatically using similar techniques. Explore with:
  ```bash
  sudo iptables -t nat -L -v -n  # After starting a container
  ```

- **Security considerations**:
  - NAT alone doesn't provide security - use firewall rules to restrict access
  - Consider adding specific FORWARD rules instead of blanket ACCEPT
  - Audit iptables rules regularly in production

- **IPv6**: This lesson focuses on IPv4. For IPv6, you'd use `ip6tables` and `net.ipv6.conf.all.forwarding`.

## Further Reading

- `man iptables` - Comprehensive iptables documentation
- `man iptables-extensions` - Details on match types and targets (including MASQUERADE)
- [Netfilter Documentation](https://www.netfilter.org/documentation/) - Official kernel packet filtering docs
- `man ip-route` - Routing table management
- [NAT Tutorial](https://www.karlrupp.net/en/computer/nat_tutorial) - Deep dive into how NAT works
- [Connection Tracking](https://wiki.nftables.org/wiki-nftables/index.php/Connection_Tracking_System) - Understanding conntrack

## Next
`09-combine-ns.md` - Combine multiple namespace types (network + mount + PID) for complete container-like isolation.
