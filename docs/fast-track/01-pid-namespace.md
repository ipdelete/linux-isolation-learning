# PID Namespace (10 min)

## What you'll build

A process that becomes PID 1 in its own isolated process tree.

## The test

**File**: `crates/contain/tests/ns_pid_test.rs`

```rust
#[test]
fn test_pid_namespace_creation() {
    if !nix::unistd::Uid::effective().is_root() { return; }

    Command::cargo_bin("contain").unwrap()
        .args(["ns", "pid"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PID inside namespace: 1"));
}
```

Run it (expect failure): `sudo -E cargo test -p contain --test ns_pid_test`

## The implementation

**File**: `crates/contain/src/ns.rs` — find `NsCommand::Pid`

```rust
NsCommand::Pid => {
    use nix::sched::{unshare, CloneFlags};
    use nix::unistd::{fork, ForkResult, getpid};
    use nix::sys::wait::waitpid;

    // Create PID namespace (only affects children)
    unshare(CloneFlags::CLONE_NEWPID)?;

    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            waitpid(child, None)?;
        }
        ForkResult::Child => {
            println!("PID inside namespace: {}", getpid());
            std::process::exit(0);
        }
    }
    Ok(())
}
```

Run tests: `sudo -E cargo test -p contain --test ns_pid_test`

## Run it

```bash
sudo cargo run -p contain -- ns pid
```

Output:
```
PID inside namespace: 1
```

## What just happened

`unshare(CLONE_NEWPID)` creates a new PID namespace, but only affects child processes. After `fork()`, the child becomes PID 1 in its own isolated process tree. It can't see host processes; the host sees it with a normal PID.

## Next

[02-mount-namespace.md](02-mount-namespace.md) — Isolate filesystem mounts

*Want more depth? See [full PID namespace tutorial](../01-namespaces/01-pid-namespace.md)*
