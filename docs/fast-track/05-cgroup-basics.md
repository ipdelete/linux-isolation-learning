# Cgroup Basics (10 min)

## What you'll build

Create a cgroup and attach a process to it for resource control.

## The test

**File**: `crates/contain/tests/cgroup_test.rs`

```rust
#[test]
fn test_cgroup_create_and_attach() {
    if !nix::unistd::Uid::effective().is_root() { return; }

    let cgroup = "/sys/fs/cgroup/test-cg";

    Command::cargo_bin("contain").unwrap()
        .args(["cgroup", "create", cgroup])
        .assert().success();

    // Verify cgroup exists
    assert!(std::path::Path::new(cgroup).exists());

    // Cleanup
    Command::cargo_bin("contain").unwrap()
        .args(["cgroup", "delete", cgroup])
        .assert().success();
}
```

Run it: `sudo -E cargo test -p contain --test cgroup_test`

## The implementation

**File**: `crates/contain/src/cgroup.rs`

Create cgroup:
```rust
CgroupCommand::Create { path } => {
    std::fs::create_dir_all(&path)?;
    println!("Created cgroup: {}", path);
    Ok(())
}
```

Attach process:
```rust
CgroupCommand::Attach { path, pid } => {
    let procs_file = format!("{}/cgroup.procs", path);
    std::fs::write(&procs_file, pid.to_string())?;
    println!("Attached PID {} to {}", pid, path);
    Ok(())
}
```

Delete cgroup:
```rust
CgroupCommand::Delete { path } => {
    std::fs::remove_dir(&path)?;
    println!("Deleted cgroup: {}", path);
    Ok(())
}
```

## Run it

```bash
# Create a cgroup
sudo cargo run -p contain -- cgroup create /sys/fs/cgroup/mygroup

# Start a process and attach it
sleep 1000 &
PID=$!
sudo cargo run -p contain -- cgroup attach /sys/fs/cgroup/mygroup $PID

# Verify
cat /sys/fs/cgroup/mygroup/cgroup.procs

# Cleanup
kill $PID
sudo cargo run -p contain -- cgroup delete /sys/fs/cgroup/mygroup
```

## What just happened

Cgroups v2 uses a filesystem interface at `/sys/fs/cgroup`. Creating a directory creates a cgroup. Writing a PID to `cgroup.procs` moves that process into the group. Once processes are in a cgroup, you can set resource limits.

## Next

[06-memory-limits.md](06-memory-limits.md) — Limit memory usage
