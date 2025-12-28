# CPU Limits (10 min)

## What you'll build

Limit a process's CPU usage with cgroups.

## The test

**File**: `crates/contain/tests/cgroup_cpu_test.rs`

```rust
#[test]
fn test_cpu_limit_set() {
    if !nix::unistd::Uid::effective().is_root() { return; }

    let cgroup = "/sys/fs/cgroup/test-cpu";

    Command::cargo_bin("contain").unwrap()
        .args(["cgroup", "create", cgroup])
        .assert().success();

    // Set 50% CPU (50000 out of 100000 period)
    Command::cargo_bin("contain").unwrap()
        .args(["cgroup", "cpu", cgroup, "50000"])
        .assert().success();

    // Verify
    let limit = std::fs::read_to_string(format!("{}/cpu.max", cgroup)).unwrap();
    assert!(limit.starts_with("50000"));

    Command::cargo_bin("contain").unwrap()
        .args(["cgroup", "delete", cgroup])
        .assert().success();
}
```

Run it: `sudo -E cargo test -p contain --test cgroup_cpu_test`

## The implementation

**File**: `crates/contain/src/cgroup.rs`

```rust
CgroupCommand::Cpu { path, quota } => {
    // Format: "quota period" or just "quota" (uses default 100000 period)
    let cpu_max = format!("{}/cpu.max", path);
    let value = format!("{} 100000", quota);

    std::fs::write(&cpu_max, &value)?;

    println!("Set cpu.max = {} for {}", value, path);
    Ok(())
}
```

## Run it

```bash
# Create cgroup with CPU limit (50%)
sudo cargo run -p contain -- cgroup create /sys/fs/cgroup/cpulimit
sudo cargo run -p contain -- cgroup cpu /sys/fs/cgroup/cpulimit 50000

# Verify
cat /sys/fs/cgroup/cpulimit/cpu.max
# Output: 50000 100000

# Test it - start a CPU-intensive process
stress --cpu 1 &
PID=$!

# Attach and observe CPU usage capped at ~50%
sudo cargo run -p contain -- cgroup attach /sys/fs/cgroup/cpulimit $PID
top -p $PID  # Watch CPU% stay around 50

# Cleanup
kill $PID
sudo cargo run -p contain -- cgroup delete /sys/fs/cgroup/cpulimit
```

## What just happened

`cpu.max` uses quota/period format. With "50000 100000", the process gets 50000 microseconds of CPU time per 100000us period = 50%. The kernel throttles the process when it hits quota. This is how containers get CPU limits.

## Next

[08-oci-bundle.md](08-oci-bundle.md) — Create an OCI container bundle
