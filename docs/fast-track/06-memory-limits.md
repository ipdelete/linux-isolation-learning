# Memory Limits (10 min)

## What you'll build

Limit a process's memory usage with cgroups.

## The test

**File**: `crates/contain/tests/cgroup_memory_test.rs`

```rust
#[test]
fn test_memory_limit_set() {
    if !nix::unistd::Uid::effective().is_root() { return; }

    let cgroup = "/sys/fs/cgroup/test-mem";

    // Create and set limit
    Command::cargo_bin("contain").unwrap()
        .args(["cgroup", "create", cgroup])
        .assert().success();

    Command::cargo_bin("contain").unwrap()
        .args(["cgroup", "memory", cgroup, "50M"])
        .assert().success();

    // Verify limit was set
    let limit = std::fs::read_to_string(format!("{}/memory.max", cgroup)).unwrap();
    assert!(limit.trim().parse::<u64>().unwrap() == 50 * 1024 * 1024);

    // Cleanup
    Command::cargo_bin("contain").unwrap()
        .args(["cgroup", "delete", cgroup])
        .assert().success();
}
```

Run it: `sudo -E cargo test -p contain --test cgroup_memory_test`

## The implementation

**File**: `crates/contain/src/cgroup.rs`

```rust
CgroupCommand::Memory { path, limit } => {
    // Parse human-readable sizes: 50M, 1G, etc.
    let bytes_value = parse_size(&limit)?;

    let memory_max = format!("{}/memory.max", path);
    std::fs::write(&memory_max, bytes_value.to_string())?;

    println!("Set memory.max = {} for {}", limit, path);
    Ok(())
}

fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim();
    let (num, mult) = if s.ends_with('G') {
        (&s[..s.len()-1], 1024 * 1024 * 1024)
    } else if s.ends_with('M') {
        (&s[..s.len()-1], 1024 * 1024)
    } else if s.ends_with('K') {
        (&s[..s.len()-1], 1024)
    } else {
        (s, 1)
    };
    Ok(num.parse::<u64>()? * mult)
}
```

## Run it

```bash
# Create cgroup with memory limit
sudo cargo run -p contain -- cgroup create /sys/fs/cgroup/limited
sudo cargo run -p contain -- cgroup memory /sys/fs/cgroup/limited 50M

# Verify
cat /sys/fs/cgroup/limited/memory.max

# Test it (this process will be killed if it exceeds limit)
sudo cgexec -g memory:/limited stress --vm 1 --vm-bytes 100M

# Cleanup
sudo cargo run -p contain -- cgroup delete /sys/fs/cgroup/limited
```

## What just happened

Writing to `memory.max` sets a hard limit. If a process in the cgroup tries to use more, the kernel kills it (OOM). The cgroup also tracks usage in `memory.current`. This is how containers enforce memory limits.

## Next

[07-cpu-limits.md](07-cpu-limits.md) — Limit CPU usage
