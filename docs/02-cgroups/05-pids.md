# 05 PIDs Controller

## Goal

Set process count limits for a cgroup using `pids.max` to protect against fork bombs. You will build a `pids-max` subcommand in `cgroup-tool` that limits how many processes (including threads) can run inside a cgroup hierarchy.

## Prereqs

- Completed `04-io.md` (or at minimum `01-cgv2-basics.md` for cgroup creation fundamentals)
- `sudo` access (writing to cgroup controllers requires root privileges)
- Understanding of cgroup v2 directory structure (`/sys/fs/cgroup/`)

## Background: The Fork Bomb Problem

A **fork bomb** is a denial-of-service attack that exploits the `fork()` system call to rapidly create new processes. Each new process immediately forks again, leading to exponential growth that can exhaust system resources in seconds:

```bash
# The classic fork bomb (DO NOT RUN THIS!)
:(){ :|:& };:
```

This creates an avalanche of processes that can:
- Exhaust the system's process table
- Consume all available memory
- Make the system unresponsive or crash
- Require a hard reboot to recover

**Why cgroups are essential:**
Traditional process limits (`ulimit -u`) are per-user, meaning a malicious container or script can still exhaust limits. The PIDs controller provides per-cgroup limits that cannot be bypassed by code running inside the cgroup.

**Key properties of the PIDs controller:**

| File | Purpose | Example Value |
|------|---------|---------------|
| `pids.max` | Maximum number of processes allowed | `100` or `max` (unlimited) |
| `pids.current` | Current number of processes | `5` (read-only) |
| `pids.peak` | Peak number of processes ever seen | `23` (read-only, kernel 5.7+) |
| `pids.events` | Count of times limit was hit | `max 3` (read-only) |

**How PIDs counting works:**
- Counts both processes AND threads (any task with a TID)
- Applies hierarchically (child cgroups count against parent limits)
- When limit is reached, `fork()` returns `-EAGAIN`
- Existing processes are not killed, but no new ones can spawn

## Write Tests (Red)

**Test file**: `crates/cgroup-tool/tests/pids_test.rs`

What the tests should verify:
- Success case: Setting a PIDs limit writes the correct value to `pids.max`
- Success case: The limit value can be read back correctly
- Error case: Invalid cgroup paths fail gracefully

Steps:

1. Open `crates/cgroup-tool/tests/pids_test.rs`

2. Find the `test_set_pids_limit` test function

3. Replace the `todo!()` with a test implementation:

```rust
use assert_cmd::Command;
use std::fs;
use std::path::Path;

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// Helper to create a test cgroup
fn create_test_cgroup(name: &str) -> String {
    let path = format!("{}/{}", CGROUP_ROOT, name);
    // Clean up first if it exists
    let _ = fs::remove_dir(&path);
    fs::create_dir(&path).expect("Failed to create test cgroup");
    path
}

/// Helper to clean up a test cgroup
fn cleanup_test_cgroup(name: &str) {
    let path = format!("{}/{}", CGROUP_ROOT, name);
    let _ = fs::remove_dir(&path);
}

#[test]
fn test_set_pids_limit() {
    // Skip if not running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_set_pids_limit: requires root privileges");
        return;
    }

    let cgroup_name = "test-pids-limit";
    let cgroup_path = create_test_cgroup(cgroup_name);

    // Run pids-max command to set limit of 10
    let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    cmd.arg("pids-max")
        .arg(cgroup_name)
        .arg("10")
        .assert()
        .success();

    // Verify pids.max was written correctly
    let pids_max_path = format!("{}/pids.max", cgroup_path);
    let content = fs::read_to_string(&pids_max_path)
        .expect("Failed to read pids.max");
    assert_eq!(content.trim(), "10", "pids.max should contain '10'");

    // Clean up
    cleanup_test_cgroup(cgroup_name);
}
```

4. Implement the second test for verifying current process count tracking:

```rust
#[test]
fn test_pids_current_tracking() {
    // Skip if not running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_pids_current_tracking: requires root privileges");
        return;
    }

    let cgroup_name = "test-pids-current";
    let cgroup_path = create_test_cgroup(cgroup_name);

    // Check pids.current starts at 0
    let pids_current_path = format!("{}/pids.current", cgroup_path);
    let content = fs::read_to_string(&pids_current_path)
        .expect("Failed to read pids.current");
    assert_eq!(content.trim(), "0", "pids.current should start at 0");

    // Clean up
    cleanup_test_cgroup(cgroup_name);
}
```

5. Remove the `#[ignore]` from the tests you want to run, and add the imports at the top of the file.

6. Run the tests (expect failure because implementation is missing):

```bash
sudo -E cargo test -p cgroup-tool --test pids_test
```

Expected output:
```
running 4 tests
test test_pids_current_tracking ... FAILED
test test_pids_limit_enforcement ... ignored
test test_set_pids_limit ... FAILED
test test_set_pids_max_unlimited ... ignored

failures:

---- test_set_pids_limit stdout ----
thread 'test_set_pids_limit' panicked at 'not yet implemented: Implement PIDs limit'
```

This is the **RED** phase - your tests are written but the implementation does not exist yet.

## Build (Green)

**Implementation file**: `crates/cgroup-tool/src/main.rs`
**TODO location**: Line ~129 in the `Command::PidsMax` match arm

Now implement the PIDs limit functionality to make your tests pass.

Steps:

1. Open `crates/cgroup-tool/src/main.rs`

2. Find the `Command::PidsMax { path, max } => todo!(...)` match arm

3. Replace the `todo!()` with the implementation:

```rust
Command::PidsMax { path, max } => {
    use std::fs;
    use std::path::Path;

    // Construct the path to pids.max
    let cgroup_path = Path::new("/sys/fs/cgroup").join(&path);
    let pids_max_path = cgroup_path.join("pids.max");

    // Verify the cgroup exists
    if !cgroup_path.exists() {
        anyhow::bail!("Cgroup '{}' does not exist at {:?}", path, cgroup_path);
    }

    // Verify the pids controller is available
    if !pids_max_path.exists() {
        anyhow::bail!(
            "pids.max not found - PIDs controller may not be enabled for this cgroup"
        );
    }

    // Write the limit value
    // The kernel accepts either a number or "max" for unlimited
    let limit_str = max.to_string();
    fs::write(&pids_max_path, &limit_str)
        .with_context(|| format!("Failed to write to {:?}", pids_max_path))?;

    // Read back and display the current state
    let current_max = fs::read_to_string(&pids_max_path)
        .with_context(|| format!("Failed to read {:?}", pids_max_path))?;

    let pids_current_path = cgroup_path.join("pids.current");
    let current_count = if pids_current_path.exists() {
        fs::read_to_string(&pids_current_path)
            .unwrap_or_else(|_| "unknown".to_string())
    } else {
        "unknown".to_string()
    };

    println!("PIDs limit set for cgroup '{}':", path);
    println!("  pids.max:     {}", current_max.trim());
    println!("  pids.current: {}", current_count.trim());

    Ok(())
}
```

4. Add the necessary import at the top of the match block if not already present:

```rust
use anyhow::Context;
```

5. Run the tests (expect success):

```bash
sudo -E cargo test -p cgroup-tool --test pids_test
```

Expected output:
```
running 4 tests
test test_pids_current_tracking ... ok
test test_pids_limit_enforcement ... ignored
test test_set_pids_limit ... ok
test test_set_pids_max_unlimited ... ignored

test result: ok. 2 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

This is the **GREEN** phase - your tests now pass!

## Verify

**Automated verification**:
```bash
# Run all cgroup-tool tests
sudo -E cargo test -p cgroup-tool

# Run just the PIDs tests
sudo -E cargo test -p cgroup-tool --test pids_test
```

**Manual verification** (observe the actual behavior):

1. Create a test cgroup and set a PIDs limit:

```bash
# Create a test cgroup
sudo mkdir -p /sys/fs/cgroup/pids-test

# Verify the controller files exist
ls /sys/fs/cgroup/pids-test/pids.*
# Expected: pids.current  pids.events  pids.max

# Check current values
cat /sys/fs/cgroup/pids-test/pids.max
# Expected: max (unlimited)

cat /sys/fs/cgroup/pids-test/pids.current
# Expected: 0

# Set a PIDs limit using our tool
sudo cargo run -q -p cgroup-tool -- pids-max pids-test 10
```

Expected output:
```
PIDs limit set for cgroup 'pids-test':
  pids.max:     10
  pids.current: 0
```

2. Verify the limit was applied:

```bash
cat /sys/fs/cgroup/pids-test/pids.max
# Expected: 10
```

3. Observe pids.current increasing with processes:

```bash
# Move current shell into the cgroup
echo $$ | sudo tee /sys/fs/cgroup/pids-test/cgroup.procs

# Check current count (should be 1 for the shell)
cat /sys/fs/cgroup/pids-test/pids.current
# Expected: 1

# Spawn a subprocess
sleep 100 &

# Check count increased
cat /sys/fs/cgroup/pids-test/pids.current
# Expected: 2

# Kill the background process
kill %1

# Count decreases
cat /sys/fs/cgroup/pids-test/pids.current
# Expected: 1
```

4. (Optional) Test limit enforcement:

```bash
# Create a fresh cgroup with a low limit
sudo mkdir -p /sys/fs/cgroup/fork-test
echo 5 | sudo tee /sys/fs/cgroup/fork-test/pids.max

# Move shell into it
echo $$ | sudo tee /sys/fs/cgroup/fork-test/cgroup.procs

# Try to spawn many processes (will hit limit)
for i in {1..10}; do
    sleep 100 &
done
# Expected: Some commands fail with "Resource temporarily unavailable" (EAGAIN)

# Check events to see how many times limit was hit
cat /sys/fs/cgroup/fork-test/pids.events
# Expected: max N (where N is the number of failed forks)

# Clean up - move shell back to root cgroup first
echo $$ | sudo tee /sys/fs/cgroup/cgroup.procs
pkill -P $$  # Kill child processes
```

5. View the pids.peak value (kernel 5.7+):

```bash
# If available, shows the maximum processes ever seen in this cgroup
cat /sys/fs/cgroup/pids-test/pids.peak 2>/dev/null || echo "pids.peak not available (requires kernel 5.7+)"
```

## Clean Up

Move any attached processes back to the root cgroup, then remove the test cgroups:

```bash
# Ensure shell is in root cgroup
echo $$ | sudo tee /sys/fs/cgroup/cgroup.procs

# Kill any remaining background processes in test cgroups
sudo cat /sys/fs/cgroup/pids-test/cgroup.procs | xargs -r kill 2>/dev/null || true
sudo cat /sys/fs/cgroup/fork-test/cgroup.procs | xargs -r kill 2>/dev/null || true

# Remove test cgroups
sudo rmdir /sys/fs/cgroup/pids-test 2>/dev/null || true
sudo rmdir /sys/fs/cgroup/fork-test 2>/dev/null || true
```

**Verification that cleanup succeeded:**
```bash
ls -d /sys/fs/cgroup/pids-test 2>/dev/null && echo "Cleanup failed" || echo "Cleanup successful"
ls -d /sys/fs/cgroup/fork-test 2>/dev/null && echo "Cleanup failed" || echo "Cleanup successful"
```

## Common Errors

1. **`pids.max not found` or `No such file or directory`**
   - Cause: The PIDs controller is not enabled for this cgroup
   - Fix: Enable the controller in the parent cgroup:
     ```bash
     echo "+pids" | sudo tee /sys/fs/cgroup/cgroup.subtree_control
     ```
   - Note: Controllers must be enabled at each level of the hierarchy

2. **`Device or resource busy` when deleting cgroup**
   - Cause: Processes are still attached to the cgroup
   - Fix: Move all processes to another cgroup first:
     ```bash
     # List attached processes
     cat /sys/fs/cgroup/your-cgroup/cgroup.procs

     # Move each to root (or another cgroup)
     cat /sys/fs/cgroup/your-cgroup/cgroup.procs | while read pid; do
         echo $pid | sudo tee /sys/fs/cgroup/cgroup.procs
     done
     ```

3. **`fork: Resource temporarily unavailable` (EAGAIN)**
   - Cause: The PIDs limit has been reached
   - This is **expected behavior** when testing limit enforcement
   - Check `pids.events` to see how many times the limit was hit:
     ```bash
     cat /sys/fs/cgroup/your-cgroup/pids.events
     # Output: max 5  (limit was hit 5 times)
     ```

4. **Limit does not seem to apply**
   - Cause: You may be testing from outside the cgroup
   - Fix: Ensure your shell or test process is attached to the cgroup:
     ```bash
     cat /proc/$$/cgroup
     # Should show your-cgroup in the hierarchy
     ```
   - Also verify hierarchical counting: child cgroup limits cannot exceed parent limits

## Notes

**PIDs vs processes:**
- The PIDs controller counts all tasks with a Thread ID (TID), meaning both processes and kernel threads
- A multi-threaded application counts as multiple PIDs (one per thread)
- This prevents both fork bombs AND thread bombs

**Hierarchical limits:**
- PIDs limits apply hierarchically: a child cgroup cannot have more processes than its ancestors allow
- Example: If parent has `pids.max=100` and child has `pids.max=50`, the child is limited to 50
- If parent has `pids.max=100` and child has `pids.max=200`, the child is still limited to 100 (the parent's limit applies)

**Setting "max" for unlimited:**
- To remove a PIDs limit, write "max" to pids.max:
  ```bash
  echo max | sudo tee /sys/fs/cgroup/your-cgroup/pids.max
  ```
- Note: In our implementation, we accept a number only. Extending to accept "max" as a special string value is left as an exercise.

**Production recommendations:**
- Always set PIDs limits on container cgroups to prevent fork bombs
- Typical values: 100-1000 depending on workload
- Monitor `pids.events` for signs of limit being hit (might indicate attack or resource exhaustion)
- Consider using `pids.peak` (kernel 5.7+) for capacity planning

**Kernel documentation:**
- `Documentation/admin-guide/cgroup-v2.rst` - PIDs controller section
- Kernel source: `kernel/cgroup/pids.c`

**Comparison with ulimit:**
| Feature | ulimit -u | cgroup pids.max |
|---------|-----------|-----------------|
| Scope | Per-user | Per-cgroup |
| Bypassable | By other users | No (kernel enforced) |
| Hierarchical | No | Yes |
| Threads | Separate limit | Included in count |
| Container-friendly | No | Yes |

## Next

`06-multi-resource.md` - Combine multiple resource controllers (memory + CPU + PIDs) on a single cgroup for comprehensive resource isolation
