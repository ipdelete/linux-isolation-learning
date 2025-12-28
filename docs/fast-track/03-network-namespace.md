# Network Namespace (10 min)

## What you'll build

An isolated network stack with its own interfaces, connected to the host via veth pair.

## The test

**File**: `crates/contain/tests/net_test.rs`

```rust
#[test]
fn test_veth_pair_created() {
    if !nix::unistd::Uid::effective().is_root() { return; }

    // Create namespace and veth pair
    Command::cargo_bin("contain").unwrap()
        .args(["net", "create", "test-ns"])
        .assert().success();

    Command::cargo_bin("contain").unwrap()
        .args(["net", "veth", "--host", "veth-host", "--ns", "test-ns"])
        .assert().success();

    // Cleanup
    Command::cargo_bin("contain").unwrap()
        .args(["net", "delete", "test-ns"])
        .assert().success();
}
```

Run it: `sudo -E cargo test -p contain --test net_test`

## The implementation

**File**: `crates/contain/src/net.rs`

Create namespace:
```rust
NetCommand::Create { name } => {
    use std::process::Command as Cmd;

    // Create named network namespace
    Cmd::new("ip")
        .args(["netns", "add", &name])
        .status()?;

    println!("Created network namespace: {}", name);
    Ok(())
}
```

Create veth pair:
```rust
NetCommand::Veth { host, ns } => {
    use std::process::Command as Cmd;

    // Create veth pair
    Cmd::new("ip")
        .args(["link", "add", &host, "type", "veth", "peer", "name", "veth0"])
        .status()?;

    // Move one end into namespace
    Cmd::new("ip")
        .args(["link", "set", "veth0", "netns", &ns])
        .status()?;

    // Bring up host side
    Cmd::new("ip")
        .args(["link", "set", &host, "up"])
        .status()?;

    println!("Created veth pair: {} <-> veth0 (in {})", host, ns);
    Ok(())
}
```

## Run it

```bash
# Create namespace
sudo cargo run -p contain -- net create mynet

# Create veth pair
sudo cargo run -p contain -- net veth --host veth-host --ns mynet

# Check host side
ip link show veth-host

# Check inside namespace
sudo ip netns exec mynet ip link show

# Cleanup
sudo cargo run -p contain -- net delete mynet
```

## What just happened

Network namespaces isolate the entire network stack: interfaces, routing tables, firewall rules. A veth pair is a virtual ethernet cable—one end stays on the host, the other moves into the namespace. This is how containers get network connectivity.

## Next

[04-combine.md](04-combine.md) — Combine namespaces into a container
