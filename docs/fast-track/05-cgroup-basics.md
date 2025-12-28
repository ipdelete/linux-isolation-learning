# Cgroup Basics (10 min)

## What you'll build

Create a cgroup and attach a process to it for resource control.

## The test

**File**: `crates/cgroup-tool/tests/create_test.rs`

```rust
#[test]
fn test_cgroup_create_and_attach() {
    if !nix::unistd::Uid::effective().is_root() { return; }

    let cgroup = "/sys/fs/cgroup/test-cg";

    Command::cargo_bin("cgroup-tool").unwrap()
        .args(["create", cgroup])
        .assert().success();

    // Verify cgroup exists
    assert!(std::path::Path::new(cgroup).exists());

    // Cleanup
    Command::cargo_bin("cgroup-tool").unwrap()
        .args(["delete", cgroup])
        .assert().success();
}
```

Run it: `sudo -E cargo test -p cgroup-tool --test create_test`

## The implementation

**File**: `crates/cgroup-tool/src/main.rs`

Create cgroup:
```rust
Command::Create { path } => {
    std::fs::create_dir_all(&path)?;
    println!("Created cgroup: {}", path);
    Ok(())
}
```

Attach process:
```rust
Command::Attach { path, pid } => {
    let procs_file = format!("{}/cgroup.procs", path);
    std::fs::write(&procs_file, pid.to_string())?;
    println!("Attached PID {} to {}", pid, path);
    Ok(())
}
```

Delete cgroup:
```rust
Command::Delete { path } => {
    std::fs::remove_dir(&path)?;
    println!("Deleted cgroup: {}", path);
    Ok(())
}
```

## Run it

```bash
# Create a cgroup
sudo cargo run -p cgroup-tool -- create /sys/fs/cgroup/mygroup

# Start a process and attach it
sleep 1000 &
PID=$!
sudo cargo run -p cgroup-tool -- attach /sys/fs/cgroup/mygroup $PID

# Verify
cat /sys/fs/cgroup/mygroup/cgroup.procs

# Cleanup
kill $PID
sudo cargo run -p cgroup-tool -- delete /sys/fs/cgroup/mygroup
```

## What just happened

Cgroups v2 uses a filesystem interface at `/sys/fs/cgroup`. Creating a directory creates a cgroup. Writing a PID to `cgroup.procs` moves that process into the group. Once processes are in a cgroup, you can set resource limits.

## Next

[06-memory-limits.md](06-memory-limits.md) — Limit memory usage
