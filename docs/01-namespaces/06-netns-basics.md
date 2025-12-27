# 06 Network Namespace Basics: Isolated Network Stack

## Goal
Create and configure a network namespace with a functional loopback interface. You will learn how network namespaces isolate the entire network stack (interfaces, routing tables, firewall rules) and understand why new network namespaces start with only a loopback interface in the DOWN state.

**Deliverable**: A `netns-tool create` subcommand that creates a persistent network namespace, brings up the loopback interface, and verifies connectivity with ping.

**Time estimate**: ~45-50 minutes

## Prereqs
- Completed `05-minimal-rootfs.md` (understand mount and namespace isolation)
- `sudo` access (network namespace operations require `CAP_NET_ADMIN`)
- Linux kernel 3.8+ (network namespaces are stable since then)

## Background: What Network Namespaces Isolate

Network namespaces provide complete isolation of the network stack. Each network namespace has its own:

| Resource | Description |
|----------|-------------|
| Network interfaces | Separate set of `eth0`, `lo`, etc. Physical interfaces can only be in one namespace at a time |
| IP addresses | Each interface can have different IPs in different namespaces |
| Routing tables | Routes are namespace-specific (`ip route` shows different results) |
| Firewall rules | `iptables`/`nftables` rules are isolated per namespace |
| Network statistics | `/proc/net/dev`, `/proc/net/tcp`, etc. show namespace-specific data |
| Sockets | Listening on port 80 in two namespaces doesn't conflict |

**Key insight**: When you create a new network namespace, it starts completely empty except for a loopback interface (`lo`) that is in the DOWN state. You must manually bring it up before you can use localhost connectivity.

### Why Loopback Matters

The loopback interface (`lo`) with IP `127.0.0.1` is critical for:
- Processes communicating with themselves (e.g., database on `localhost`)
- Unix domain sockets that rely on network stack initialization
- Testing network code without external interfaces

In the root namespace, loopback is always up. In a new namespace, it starts down - this is a common source of confusion for beginners.

## Explore Network Namespaces Manually First

Before writing code, create a network namespace using the `ip` command to understand the behavior:

```bash
# Create persistent network namespace (requires sudo)
sudo ip netns add test-manual

# List all network namespaces
ip netns list

# Execute a command inside the namespace
sudo ip netns exec test-manual ip link

# You should see ONLY the loopback interface, and it's DOWN:
# 1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN mode DEFAULT group default qlen 1000
#     link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

# Try to ping localhost (will fail - loopback is down)
sudo ip netns exec test-manual ping -c 1 127.0.0.1
# ping: connect: Network is unreachable

# Bring up the loopback interface
sudo ip netns exec test-manual ip link set lo up

# Verify loopback is now UP
sudo ip netns exec test-manual ip link show lo
# 1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000

# Now ping works
sudo ip netns exec test-manual ping -c 1 127.0.0.1
# PING 127.0.0.1 (127.0.0.1) 56(84) bytes of data.
# 64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.028 ms

# Check /proc/net inside the namespace
sudo ip netns exec test-manual cat /proc/net/dev
# Should show only lo interface

# Clean up
sudo ip netns delete test-manual
```

**Try it**: Run these commands and observe the difference between loopback DOWN and UP. This hands-on exploration helps you understand what your code will need to do.

### Where Are Network Namespaces Stored?

The `ip netns` command makes namespaces persistent by bind-mounting them to `/run/netns/`:

```bash
# Create a namespace
sudo ip netns add demo

# Check where it's stored
ls -la /run/netns/
# -r--r--r-- 1 root root 0 Dec 26 10:00 demo

# This is a bind-mount of the namespace file descriptor
stat /run/netns/demo
# Shows it's a regular file (but actually a bind mount)

# The namespace persists even if no processes are in it
# because the bind mount holds a reference
```

This is exactly what your `netns-tool create` command will implement.

## Write Tests (Red)

**Test file**: `crates/netns-tool/tests/create_test.rs`

The test file has TODO placeholders for three tests. You will implement tests that verify:
1. Creating a persistent network namespace
2. Loopback interface exists in the new namespace
3. Ping to 127.0.0.1 works after bringing up loopback

### Step 1: Open the Test File

Open `crates/netns-tool/tests/create_test.rs` and examine the structure.

### Step 2: Implement Test for Namespace Creation

Replace the first `todo!()` with a test that verifies basic namespace creation:

```rust
// crates/netns-tool/tests/create_test.rs

use assert_cmd::Command;
use std::path::Path;

#[test]
fn test_create_network_namespace() {
    let ns_name = "test-create-ns";
    let ns_path = format!("/run/netns/{}", ns_name);

    // Clean up any leftover namespace from previous runs
    let _ = std::fs::remove_file(&ns_path);

    // Create the namespace using our tool
    let mut cmd = Command::cargo_bin("netns-tool").unwrap();
    cmd.arg("create")
        .arg(ns_name)
        .assert()
        .success()
        .stdout(predicates::str::contains("Created network namespace"));

    // Verify the namespace file exists
    assert!(
        Path::new(&ns_path).exists(),
        "Namespace file should exist at {}",
        ns_path
    );

    // Clean up
    let _ = std::fs::remove_file(&ns_path);
}
```

### Step 3: Implement Test for Loopback Interface

The second test verifies that the new namespace has a loopback interface. We'll use the `ip netns exec` command to inspect:

```rust
#[test]
#[ignore] // Remove this after implementing the create command
fn test_create_namespace_has_loopback() {
    let ns_name = "test-loopback-ns";
    let ns_path = format!("/run/netns/{}", ns_name);

    // Clean up
    let _ = std::fs::remove_file(&ns_path);

    // Create namespace
    let mut cmd = Command::cargo_bin("netns-tool").unwrap();
    cmd.arg("create").arg(ns_name).assert().success();

    // Execute `ip link` inside the namespace to list interfaces
    let output = std::process::Command::new("ip")
        .args(&["netns", "exec", ns_name, "ip", "link", "show"])
        .output()
        .expect("Failed to execute ip netns exec");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify loopback interface exists
    assert!(
        stdout.contains("lo:"),
        "Loopback interface should exist in new namespace"
    );

    // Verify it's in UP state (our tool should bring it up)
    assert!(
        stdout.contains("state UP") || stdout.contains("UP,LOWER_UP"),
        "Loopback interface should be UP, got: {}",
        stdout
    );

    // Clean up
    let _ = std::fs::remove_file(&ns_path);
}
```

### Step 4: Implement Test for Localhost Connectivity

The third test verifies that ping to localhost works, which proves the loopback interface is functional:

```rust
#[test]
#[ignore] // Remove this after implementing the create command
fn test_ping_localhost_works() {
    let ns_name = "test-ping-ns";
    let ns_path = format!("/run/netns/{}", ns_name);

    // Clean up
    let _ = std::fs::remove_file(&ns_path);

    // Create namespace (should bring up loopback)
    let mut cmd = Command::cargo_bin("netns-tool").unwrap();
    cmd.arg("create").arg(ns_name).assert().success();

    // Ping localhost inside the namespace
    let output = std::process::Command::new("ip")
        .args(&[
            "netns", "exec", ns_name, "ping", "-c", "1", "-W", "1", "127.0.0.1",
        ])
        .output()
        .expect("Failed to execute ping");

    // Verify ping succeeded
    assert!(
        output.status.success(),
        "Ping to localhost should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1 packets transmitted, 1 received"),
        "Ping should report success: {}",
        stdout
    );

    // Clean up
    let _ = std::fs::remove_file(&ns_path);
}
```

### Step 5: Run Tests (Expect Failure)

Run the tests to enter the RED phase:

```bash
sudo -E cargo test -p netns-tool --test create_test

# Expected output:
# test test_create_network_namespace ... FAILED
# thread 'test_create_network_namespace' panicked at crates/netns-tool/src/main.rs:40:13:
# not yet implemented: Implement network namespace creation - write tests first!
```

The test fails because the `create` command is still a `todo!()`. This is the RED phase - tests are written but implementation is missing.

## Build (Green)

**Implementation file**: `crates/netns-tool/src/main.rs`
**TODO location**: Line ~39-41 in the `Command::Create` match arm

Now implement the `create` command to make the tests pass.

### Step 1: Understand What We Need to Build

The implementation must:
1. Create `/run/netns/` directory if it doesn't exist
2. Create a new network namespace using `unshare(CLONE_NEWNET)`
3. Bind-mount the namespace to `/run/netns/{name}` to make it persistent
4. Bring up the loopback interface using netlink
5. Verify loopback is up by checking the interface state

### Step 2: Add Dependencies

First, add the `rtnetlink` crate for network configuration. Open `crates/netns-tool/Cargo.toml` and add:

```toml
[dependencies]
anyhow = { workspace = true }
clap = { workspace = true }
libc = { workspace = true }
nix = { workspace = true }
rtnetlink = "0.14"
tokio = { version = "1", features = ["rt", "macros"] }
futures = "0.3"
```

We need `tokio` because `rtnetlink` uses async I/O for communicating with the kernel via netlink sockets.

### Step 3: Implement the Create Function

Replace the `Command::Create` match arm in `src/main.rs`:

```rust
// crates/netns-tool/src/main.rs

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use nix::sched::{unshare, CloneFlags};
use nix::mount::{mount, MsFlags};
use std::fs;
use std::path::Path;

// ... (Cli and Command enum stay the same) ...

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Create { name } => {
            create_network_namespace(&name)?;
            Ok(())
        }
        // ... (other match arms stay the same) ...
    }
}

fn create_network_namespace(name: &str) -> Result<()> {
    // Step 1: Create /run/netns directory if it doesn't exist
    let netns_dir = Path::new("/run/netns");
    if !netns_dir.exists() {
        fs::create_dir_all(netns_dir)
            .context("Failed to create /run/netns directory")?;
    }

    let ns_path = netns_dir.join(name);

    // Check if namespace already exists
    if ns_path.exists() {
        anyhow::bail!("Network namespace '{}' already exists", name);
    }

    // Step 2: Create an empty file to serve as the mount point
    fs::File::create(&ns_path)
        .with_context(|| format!("Failed to create namespace file at {:?}", ns_path))?;

    // Step 3: Unshare network namespace
    unshare(CloneFlags::CLONE_NEWNET)
        .context("Failed to unshare network namespace (need CAP_NET_ADMIN)")?;

    // Step 4: Bind-mount /proc/self/ns/net to the namespace file
    // This makes the namespace persistent even after this process exits
    let source = Path::new("/proc/self/ns/net");
    mount(
        Some(source),
        &ns_path,
        None::<&str>,
        MsFlags::MS_BIND,
        None::<&str>,
    )
    .with_context(|| {
        format!(
            "Failed to bind-mount namespace to {:?}",
            ns_path
        )
    })?;

    // Step 5: Bring up the loopback interface
    bring_up_loopback()
        .context("Failed to bring up loopback interface")?;

    println!("Created network namespace '{}' at {:?}", name, ns_path);
    println!("Loopback interface is UP - localhost is reachable");

    Ok(())
}

// Bring up the loopback interface using rtnetlink
fn bring_up_loopback() -> Result<()> {
    // rtnetlink is async, so we need a tokio runtime
    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create tokio runtime")?;

    rt.block_on(async {
        bring_up_loopback_async().await
    })
}

async fn bring_up_loopback_async() -> Result<()> {
    use futures::stream::TryStreamExt;

    // Connect to the rtnetlink socket
    let (connection, handle, _) = rtnetlink::new_connection()
        .context("Failed to create rtnetlink connection")?;

    // Spawn the connection in the background
    tokio::spawn(connection);

    // Get the loopback interface
    let mut links = handle.link().get().match_name("lo".to_string()).execute();

    let link = links
        .try_next()
        .await
        .context("Failed to query loopback interface")?
        .ok_or_else(|| anyhow::anyhow!("Loopback interface not found"))?;

    // Bring the interface UP
    handle
        .link()
        .set(link.header.index)
        .up()
        .execute()
        .await
        .context("Failed to set loopback interface UP")?;

    Ok(())
}
```

### Step 4: Build and Test

Now build the crate and run the tests:

```bash
# Build first to catch any compilation errors
cargo build -p netns-tool

# Run the tests
sudo -E cargo test -p netns-tool --test create_test
```

Expected output:
```
running 3 tests
test test_create_network_namespace ... ok
test test_create_namespace_has_loopback ... ok (ignored)
test test_ping_localhost_works ... ok (ignored)

test result: ok. 1 passed; 2 ignored; 0 failed
```

### Step 5: Remove `#[ignore]` Attributes

Go back to `tests/create_test.rs` and remove the `#[ignore]` attributes from the second and third tests. Run again:

```bash
sudo -E cargo test -p netns-tool --test create_test
```

Expected output:
```
running 3 tests
test test_create_network_namespace ... ok
test test_create_namespace_has_loopback ... ok
test test_ping_localhost_works ... ok

test result: ok. 3 passed; 0 failed
```

All tests pass - this is the GREEN phase!

## Verify

**Automated verification**:
```bash
# Run all tests for the netns-tool crate
sudo -E cargo test -p netns-tool
```

All tests should pass.

**Manual verification** (observe the actual behavior):

```bash
# Create a network namespace using our tool
sudo cargo run -p netns-tool -- create demo

# Expected output:
# Created network namespace 'demo' at "/run/netns/demo"
# Loopback interface is UP - localhost is reachable

# Verify the namespace file exists
ls -la /run/netns/
# Should show: -r--r--r-- 1 root root 0 Dec 26 10:30 demo

# List interfaces inside the namespace
sudo ip netns exec demo ip link

# Expected output:
# 1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
#     link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

# Check IP addresses (loopback should have 127.0.0.1)
sudo ip netns exec demo ip addr show lo

# Expected output:
# 1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
#     link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
#     inet 127.0.0.1/8 scope host lo
#        valid_lft forever preferred_lft forever
#     inet6 ::1/128 scope host
#        valid_lft forever preferred_lft forever

# Ping localhost to verify connectivity
sudo ip netns exec demo ping -c 3 127.0.0.1

# Expected output:
# PING 127.0.0.1 (127.0.0.1) 56(84) bytes of data.
# 64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.028 ms
# 64 bytes from 127.0.0.1: icmp_seq=2 ttl=64 time=0.041 ms
# 64 bytes from 127.0.0.1: icmp_seq=3 ttl=64 time=0.037 ms

# Check /proc/net inside the namespace
sudo ip netns exec demo cat /proc/net/dev

# Expected output shows only lo interface:
# Inter-|   Receive                                                |  Transmit
#  face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
#     lo:     252       3    0    0    0     0          0         0      252       3    0    0    0     0       0          0

# Run a shell inside the namespace
sudo ip netns exec demo /bin/bash

# Inside the namespace shell, try these commands:
ip link                  # Only lo visible
ip route                 # Empty routing table (except localhost)
ping 8.8.8.8             # Fails - no external connectivity yet
exit
```

## Clean Up

Remove the network namespace we created:

```bash
# Unmount the bind mount first
sudo umount /run/netns/demo

# Remove the file
sudo rm /run/netns/demo

# Or use ip netns delete (does both steps)
sudo ip netns delete demo

# Verify it's gone
ip netns list
```

Note: The `delete` subcommand is a TODO for a future lesson. For now, use `ip netns delete` or the manual commands above.

## Common Errors

1. **`Permission denied` when calling `unshare()`**
   - Cause: Network namespace creation requires `CAP_NET_ADMIN` capability
   - Fix: Run with `sudo`. Check with `sudo -E cargo run ...` to preserve environment variables

2. **`Failed to create rtnetlink connection: Operation not permitted`**
   - Cause: The process lacks `CAP_NET_ADMIN` in the current namespace context
   - Fix: Ensure you're running with root privileges. This can happen if you drop privileges between `unshare()` and `bring_up_loopback()`

3. **Loopback interface stays DOWN**
   - Cause: The `bring_up_loopback()` function wasn't called or failed silently
   - Fix: Check error handling. Ensure the rtnetlink connection succeeds and the `set().up()` call executes

4. **`Network is unreachable` when pinging 127.0.0.1**
   - Cause: Loopback interface is down or not assigned the 127.0.0.1 address
   - Fix: Verify `ip addr show lo` inside the namespace shows `127.0.0.1/8`. The kernel assigns this automatically when `lo` is brought up

5. **`Device or resource busy` when bind-mounting**
   - Cause: The target file already exists and is already a mount point
   - Fix: Clean up old namespaces with `sudo ip netns delete <name>` before creating a new one with the same name

6. **`Loopback interface not found` error from rtnetlink**
   - Cause: Querying interfaces before the namespace is fully initialized
   - Fix: Ensure `unshare()` is called before `bring_up_loopback()`. The loopback interface is created by the kernel immediately after entering the new namespace

7. **Tests hang or timeout**
   - Cause: Async runtime issues or netlink socket blocking
   - Fix: Ensure tokio runtime is properly configured. Check that the rtnetlink connection is spawned with `tokio::spawn()`

## Understanding Netlink and rtnetlink

The `rtnetlink` crate provides a Rust interface to the kernel's netlink socket, which is the modern API for configuring network interfaces. Here's what's happening under the hood:

### What is Netlink?

Netlink is a socket-based IPC mechanism for communication between kernel and userspace. The `NETLINK_ROUTE` protocol family handles:
- Creating/deleting network interfaces
- Adding/removing IP addresses
- Modifying routing tables
- Setting interface states (UP/DOWN)

### Alternative: Using the `ip` Command

You could also bring up loopback by spawning the `ip` command:

```rust
fn bring_up_loopback_with_ip() -> Result<()> {
    let output = std::process::Command::new("ip")
        .args(&["link", "set", "lo", "up"])
        .output()
        .context("Failed to execute ip command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to bring up loopback: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}
```

This works and is simpler, but has downsides:
- Requires the `ip` binary to be installed
- Less portable (ip command syntax may vary)
- Harder to test in isolation
- More overhead (fork/exec)

The `rtnetlink` approach uses the kernel API directly, which is more reliable and doesn't depend on external tools.

## Notes

- **Namespace persistence**: The bind-mount to `/run/netns/` keeps the namespace alive even with no processes in it. Without this, the namespace would be destroyed when the creating process exits.

- **Why /run/netns?**: This is a convention established by the `iproute2` package. Tools like `ip netns` expect to find namespaces here. Using this path makes our tool compatible with standard utilities.

- **Loopback always exists**: The kernel automatically creates a loopback interface in every network namespace. You don't create it - you just need to bring it UP.

- **IPv6 loopback**: When you bring up `lo`, the kernel assigns both `127.0.0.1/8` (IPv4) and `::1/128` (IPv6) automatically.

- **Async vs sync**: We use `tokio::runtime::Runtime::new().block_on()` to run async code in a sync context. This is fine for small tools. For larger applications, you'd typically use `#[tokio::main]` and make `main()` async.

- **Kernel versions**: Network namespaces are stable since kernel 3.8. All modern Linux distributions support them.

- **Further reading**:
  - `man 7 network_namespaces` - Overview of network namespace isolation
  - `man 7 netlink` - Netlink socket documentation
  - `man 8 ip-netns` - The `ip netns` command documentation
  - rtnetlink crate docs: https://docs.rs/rtnetlink/

## Optional Exercises

If you finish early or want to deepen your understanding:

### Exercise 1: Add Error Recovery

Modify the `create_network_namespace()` function to clean up partial state if an error occurs. For example, if `unshare()` succeeds but `bring_up_loopback()` fails, remove the created file.

### Exercise 2: Implement the Delete Command

Implement the `Command::Delete` match arm to properly clean up namespaces:
- Unmount the bind mount
- Remove the file
- Handle errors gracefully (e.g., namespace doesn't exist)

### Exercise 3: Add IP Address Assignment

Extend the tool to optionally assign an IP address to loopback (other than 127.0.0.1). Hint: use `handle.address().add()` from rtnetlink.

### Exercise 4: Compare with Existing Namespace

Write a test that creates a namespace and verifies its loopback interface has a different netlink index than the host namespace's loopback.

## Next
`07-veth-bridge.md` - Create virtual ethernet pairs (veth) and bridge them to enable communication between namespaces and the host
