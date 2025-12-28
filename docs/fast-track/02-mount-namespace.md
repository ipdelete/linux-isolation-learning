# Mount Namespace (10 min)

## What you'll build

A process with isolated filesystem mounts that don't leak to the host.

## The test

**File**: `crates/ns-tool/tests/mount_test.rs`

```rust
#[test]
fn test_mount_namespace_isolation() {
    let mounts_before = fs::read_to_string("/proc/self/mounts").unwrap();

    Command::cargo_bin("ns-tool").unwrap()
        .arg("mount")
        .assert()
        .success()
        .stdout(predicate::str::contains("/mnt/test_mount"));

    let mounts_after = fs::read_to_string("/proc/self/mounts").unwrap();
    assert!(!mounts_after.contains("/mnt/test_mount"), "Mount leaked!");
}
```

Run it (expect failure): `sudo -E cargo test -p ns-tool --test mount_test`

## The implementation

**File**: `crates/ns-tool/src/main.rs` — find `Command::Mount`

```rust
Command::Mount => {
    use nix::mount::{mount, umount, MsFlags};
    use nix::sched::{unshare, CloneFlags};

    // Create mount namespace
    unshare(CloneFlags::CLONE_NEWNS)?;

    // CRITICAL: Make root private (prevents mount propagation)
    mount(None::<&str>, "/", None::<&str>,
          MsFlags::MS_PRIVATE | MsFlags::MS_REC, None::<&str>)?;

    // Create isolated tmpfs mount
    let mount_point = "/mnt/test_mount";
    std::fs::create_dir_all(mount_point)?;
    mount(Some("tmpfs"), mount_point, Some("tmpfs"),
          MsFlags::MS_NODEV | MsFlags::MS_NOSUID, None::<&str>)?;

    println!("Mounted tmpfs at {}", mount_point);

    // Show it exists inside namespace
    let mounts = std::fs::read_to_string("/proc/self/mounts")?;
    for line in mounts.lines().filter(|l| l.contains(mount_point)) {
        println!("{}", line);
    }

    // Cleanup
    umount(mount_point)?;
    std::fs::remove_dir(mount_point)?;

    Ok(())
}
```

Run tests: `sudo -E cargo test -p ns-tool --test mount_test`

## Run it

```bash
sudo cargo run -p ns-tool -- mount
```

Output:
```
Mounted tmpfs at /mnt/test_mount
tmpfs /mnt/test_mount tmpfs rw,nosuid,nodev 0 0
```

Verify isolation (in another terminal):
```bash
grep test_mount /proc/self/mounts  # Nothing! Mount is isolated
```

## What just happened

`unshare(CLONE_NEWNS)` creates a mount namespace, but mounts still propagate by default. The key line is `mount("/", MS_PRIVATE | MS_REC)` — this makes all mounts private. Now any mount we create stays inside our namespace.

## Next

[03-network-namespace.md](03-network-namespace.md) — Network isolation

*Want more depth? See [full mount namespace tutorial](../01-namespaces/04-mount-namespace.md)*
