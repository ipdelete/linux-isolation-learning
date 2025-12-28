# Combine Namespaces (15 min)

## What you'll build

A mini-container: isolated PID, hostname, network, and filesystem in one process.

## The test

**File**: `crates/contain/tests/ns_container_test.rs`

```rust
#[test]
fn test_container_isolation() {
    if !nix::unistd::Uid::effective().is_root() { return; }

    let parent_pid_ns = fs::read_link("/proc/self/ns/pid").unwrap();

    Command::cargo_bin("contain").unwrap()
        .args(["ns", "container", "--", "/bin/sh", "-c", "echo PID:$$ && hostname"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PID:1"))
        .stdout(predicate::str::contains("container"));

    // Verify we're in different namespace
    let still_same = fs::read_link("/proc/self/ns/pid").unwrap();
    assert_eq!(parent_pid_ns, still_same); // Parent unchanged
}
```

Run it (expect failure): `sudo -E cargo test -p contain --test ns_container_test`

## The implementation

**File**: `crates/contain/src/ns.rs`

```rust
NsCommand::Container { hostname, command } => {
    use nix::sched::{unshare, CloneFlags};
    use nix::unistd::{fork, ForkResult, sethostname, execvp, getpid};
    use nix::mount::{mount, MsFlags};
    use nix::sys::wait::waitpid;
    use std::ffi::CString;

    // Create all namespaces at once
    unshare(
        CloneFlags::CLONE_NEWPID |
        CloneFlags::CLONE_NEWUTS |
        CloneFlags::CLONE_NEWNS |
        CloneFlags::CLONE_NEWNET
    )?;

    match unsafe { fork()? } {
        ForkResult::Parent { child } => { waitpid(child, None)?; }
        ForkResult::Child => {
            // Make mounts private, remount /proc
            mount(None::<&str>, "/", None::<&str>,
                  MsFlags::MS_PRIVATE | MsFlags::MS_REC, None::<&str>)?;
            mount(Some("proc"), "/proc", Some("proc"),
                  MsFlags::empty(), None::<&str>)?;

            // Set hostname
            sethostname(&hostname)?;

            // Bring up loopback
            std::process::Command::new("ip")
                .args(["link", "set", "lo", "up"])
                .status()?;

            // Exec command (or shell)
            let cmd = if command.is_empty() {
                vec!["/bin/sh".to_string()]
            } else { command.clone() };

            let cstrs: Vec<CString> = cmd.iter()
                .map(|s| CString::new(s.as_str()).unwrap())
                .collect();
            execvp(&cstrs[0], &cstrs)?;
        }
    }
    Ok(())
}
```

Add to the `NsCommand` enum:
```rust
Container {
    #[arg(long, default_value = "container")]
    hostname: String,
    #[arg(last = true)]
    command: Vec<String>,
},
```

Run tests: `sudo -E cargo test -p contain --test ns_container_test`

## Run it

```bash
sudo cargo run -p contain -- ns container
```

You're now in an isolated shell:
```bash
hostname        # -> container
echo $$         # -> 1
ps aux          # -> only your shell process
ip addr         # -> only loopback
exit
```

## What just happened

One `unshare()` call with OR'd flags creates all namespaces atomically. After `fork()`:
- Child is PID 1 (new PID namespace)
- Has its own hostname (UTS namespace)
- Has private mounts (mount namespace)
- Has isolated network (network namespace)

This is the core of container isolation. Docker/containerd do the same thing, plus cgroups for resource limits.

## Next

[05-cgroup-basics.md](05-cgroup-basics.md) — Add resource limits with cgroups

*Want more depth? See [full combined namespaces tutorial](../01-namespaces/09-combine-ns.md)*
