# 03 CPU Controller

## Goal
Set CPU quota for a cgroup using `cpu.max` to limit how much CPU time processes can consume. You will build a `cpu-max` subcommand in `cgroup-tool` that writes quota and period values to control CPU bandwidth.

## Prereqs
- Completed `01-cgv2-basics.md` (you can create and delete cgroups)
- Completed `02-memory.md` (you understand the pattern for setting controller limits)
- `sudo` access (writing to cgroup files requires root)
- Basic understanding of CPU time and scheduling concepts

## Background: How CPU Bandwidth Control Works

The cgroup v2 CPU controller uses **bandwidth control** to limit how much CPU time a cgroup can consume. Unlike memory limits (which are hard caps), CPU limits are enforced over rolling time windows.

### The cpu.max File Format

The `cpu.max` file controls CPU bandwidth using two values:

```
QUOTA PERIOD
```

Both values are in **microseconds**:
- **QUOTA**: Maximum CPU time the cgroup can use per period
- **PERIOD**: The length of the scheduling period (default: 100000 = 100ms)

**Examples:**

| cpu.max Value | Meaning |
|---------------|---------|
| `50000 100000` | 50ms out of every 100ms = 50% of one CPU |
| `100000 100000` | 100ms out of every 100ms = 100% of one CPU |
| `200000 100000` | 200ms out of every 100ms = 200% = 2 full CPUs |
| `25000 100000` | 25ms out of every 100ms = 25% of one CPU |
| `max 100000` | Unlimited CPU (no quota enforced) |

**Key insight**: The quota is **per CPU core**, so on a 4-core system:
- `100000 100000` allows using one full CPU core
- `400000 100000` allows using all four CPU cores at 100%
- `50000 100000` restricts to 50% of one core (12.5% of a 4-core system)

### How the Kernel Enforces Limits

The kernel tracks CPU usage across each period window:

1. Processes in the cgroup run normally until they consume their quota
2. Once quota is exhausted, processes are **throttled** (put to sleep) until the next period
3. At the start of each new period, the quota resets
4. Throttling events are recorded in `cpu.stat`

This creates a "burst then wait" pattern: a process might use 50ms of CPU time very quickly, then be forced to wait until the period resets.

### Monitoring with cpu.stat

The `cpu.stat` file shows how the controller is working:

```
usage_usec 123456789     # Total CPU time used (microseconds)
user_usec 100000000      # Time in user mode
system_usec 23456789     # Time in kernel mode
nr_periods 1500          # Number of elapsed periods
nr_throttled 42          # Number of periods where throttling occurred
throttled_usec 2100000   # Total time processes were throttled
```

If `nr_throttled` is high relative to `nr_periods`, the cgroup is consistently hitting its CPU limit.

## Write Tests (Red)

**Test file**: `crates/cgroup-tool/tests/cpu_test.rs`

What the tests should verify:
- Success case: Writing a quota/period pair to `cpu.max` succeeds
- Success case: The value can be read back correctly
- Edge case: Writing "max" for unlimited CPU works
- Error case: Invalid formats are handled gracefully

Steps:

1. Open `crates/cgroup-tool/tests/cpu_test.rs`

2. Find the `test_set_cpu_quota` function (line 12) and replace the `todo!()`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// Helper to create a test cgroup
fn create_test_cgroup(name: &str) -> std::io::Result<()> {
    let path = format!("{}/{}", CGROUP_ROOT, name);
    if !Path::new(&path).exists() {
        fs::create_dir(&path)?;
    }
    Ok(())
}

/// Helper to delete a test cgroup
fn delete_test_cgroup(name: &str) -> std::io::Result<()> {
    let path = format!("{}/{}", CGROUP_ROOT, name);
    if Path::new(&path).exists() {
        fs::remove_dir(&path)?;
    }
    Ok(())
}

#[test]
fn test_set_cpu_quota() {
    // Skip if not running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_set_cpu_quota: requires root privileges");
        return;
    }

    let cgroup_name = "test-cpu-quota";

    // Setup: create test cgroup
    create_test_cgroup(cgroup_name).expect("Failed to create test cgroup");

    // Run the cpu-max command with 50% CPU limit (50ms out of 100ms)
    let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    cmd.arg("cpu-max")
        .arg(cgroup_name)
        .arg("50000 100000")
        .assert()
        .success();

    // Verify: read cpu.max and check the value
    let cpu_max_path = format!("{}/{}/cpu.max", CGROUP_ROOT, cgroup_name);
    let content = fs::read_to_string(&cpu_max_path)
        .expect("Failed to read cpu.max");

    assert!(
        content.trim() == "50000 100000",
        "Expected cpu.max to contain '50000 100000', got '{}'",
        content.trim()
    );

    // Cleanup
    delete_test_cgroup(cgroup_name).expect("Failed to delete test cgroup");
}
```

3. Optionally, implement `test_set_cpu_max_unlimited` (line 49) by removing the `#[ignore]` attribute:

```rust
#[test]
fn test_set_cpu_max_unlimited() {
    // Skip if not running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_set_cpu_max_unlimited: requires root privileges");
        return;
    }

    let cgroup_name = "test-cpu-unlimited";

    // Setup: create test cgroup
    create_test_cgroup(cgroup_name).expect("Failed to create test cgroup");

    // First set a limit
    let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    cmd.arg("cpu-max")
        .arg(cgroup_name)
        .arg("50000 100000")
        .assert()
        .success();

    // Now remove the limit by setting "max"
    let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    cmd.arg("cpu-max")
        .arg(cgroup_name)
        .arg("max 100000")
        .assert()
        .success();

    // Verify: cpu.max should show "max 100000"
    let cpu_max_path = format!("{}/{}/cpu.max", CGROUP_ROOT, cgroup_name);
    let content = fs::read_to_string(&cpu_max_path)
        .expect("Failed to read cpu.max");

    assert!(
        content.trim() == "max 100000",
        "Expected cpu.max to contain 'max 100000', got '{}'",
        content.trim()
    );

    // Cleanup
    delete_test_cgroup(cgroup_name).expect("Failed to delete test cgroup");
}
```

4. Run the tests (expect failure because implementation is missing):

```bash
sudo -E cargo test -p cgroup-tool --test cpu_test
```

Expected output:
```
running 3 tests
test test_cpu_quota_enforcement ... ignored
test test_set_cpu_max_unlimited ... ignored (or FAILED if you implemented it)
test test_set_cpu_quota ... FAILED

failures:

---- test_set_cpu_quota stdout ----
thread 'test_set_cpu_quota' panicked at 'not yet implemented: Implement CPU quota - write tests first!', crates/cgroup-tool/src/main.rs:112:13
```

This is the **RED** phase - your tests are written but the implementation does not exist yet.

## Build (Green)

**Implementation file**: `crates/cgroup-tool/src/main.rs`
**TODO location**: Line ~111 in the `Command::CpuMax` match arm

Now implement the CPU quota functionality to make your tests pass.

Steps:

1. Open `crates/cgroup-tool/src/main.rs`

2. Find the `Command::CpuMax { path, quota } => todo!(...)` match arm (around line 111)

3. Replace the `todo!()` with the implementation:

```rust
Command::CpuMax { path, quota } => {
    use std::fs;
    use std::path::Path;

    // Construct the full path to cpu.max
    let cgroup_path = if path.starts_with('/') {
        format!("/sys/fs/cgroup{}/cpu.max", path)
    } else {
        format!("/sys/fs/cgroup/{}/cpu.max", path)
    };

    // Verify the cgroup exists
    let cgroup_dir = Path::new(&cgroup_path).parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid cgroup path"))?;

    if !cgroup_dir.exists() {
        anyhow::bail!(
            "Cgroup '{}' does not exist. Create it first with: cgroup-tool create {}",
            path, path
        );
    }

    // Write the quota to cpu.max
    // The format should be "QUOTA PERIOD" or just "QUOTA" (uses default period)
    // Examples: "50000 100000" for 50% CPU, "max 100000" for unlimited
    fs::write(&cgroup_path, &quota)
        .with_context(|| format!("Failed to write '{}' to {}", quota, cgroup_path))?;

    println!("Set CPU quota for '{}' to: {}", path, quota);

    // Read back and display the value for confirmation
    let current = fs::read_to_string(&cgroup_path)
        .with_context(|| format!("Failed to read {}", cgroup_path))?;
    println!("Current cpu.max: {}", current.trim());

    Ok(())
}
```

4. You will need to add the `with_context` import at the top of the file if not already present:

```rust
use anyhow::{Context, Result};
```

5. Run the tests (expect success):

```bash
sudo -E cargo test -p cgroup-tool --test cpu_test
```

Expected output:
```
running 3 tests
test test_cpu_quota_enforcement ... ignored
test test_set_cpu_max_unlimited ... ok (or ignored)
test test_set_cpu_quota ... ok

test result: ok. 1 passed (or 2 passed); 0 failed; 2 ignored (or 1 ignored)
```

This is the **GREEN** phase - your tests now pass!

## Verify

**Automated verification**:
```bash
# Run all cgroup-tool tests
sudo -E cargo test -p cgroup-tool

# Run just the CPU tests
sudo -E cargo test -p cgroup-tool --test cpu_test
```

All tests should pass.

**Manual verification** (observe the actual behavior):

1. Create a test cgroup and set a CPU limit:

```bash
# Create a cgroup
sudo mkdir -p /sys/fs/cgroup/cpu-test

# Set 50% CPU limit (50ms every 100ms)
sudo cargo run -p cgroup-tool -- cpu-max cpu-test "50000 100000"
```

Expected output:
```
Set CPU quota for 'cpu-test' to: 50000 100000
Current cpu.max: 50000 100000
```

2. Verify the limit was applied:

```bash
cat /sys/fs/cgroup/cpu-test/cpu.max
```

Expected output:
```
50000 100000
```

3. Test CPU throttling (optional but instructive):

```bash
# Start a CPU-intensive process in the cgroup
# First, attach the current shell to the cgroup
echo $$ | sudo tee /sys/fs/cgroup/cpu-test/cgroup.procs

# Run a CPU stress test (this will be throttled)
dd if=/dev/zero of=/dev/null bs=1M &
DD_PID=$!

# Watch throttling statistics (in another terminal)
watch -n 1 cat /sys/fs/cgroup/cpu-test/cpu.stat

# You should see nr_throttled increasing
# After observing, kill the process
kill $DD_PID
```

The `cpu.stat` output will show:
```
usage_usec 5000000
user_usec 4500000
system_usec 500000
nr_periods 150
nr_throttled 75        <-- About 50% of periods are throttled
throttled_usec 3750000 <-- Time spent waiting
```

4. Inspect throttling behavior:

```bash
# View cpu.stat for throttling information
cat /sys/fs/cgroup/cpu-test/cpu.stat
```

Key fields to observe:
- `nr_periods`: Total number of scheduling periods elapsed
- `nr_throttled`: Number of periods where the cgroup hit its limit
- `throttled_usec`: Total microseconds processes were forced to wait

5. Test removing the limit:

```bash
# Set unlimited CPU
sudo cargo run -p cgroup-tool -- cpu-max cpu-test "max 100000"

# Verify
cat /sys/fs/cgroup/cpu-test/cpu.max
```

Expected output:
```
max 100000
```

## Clean Up

```bash
# Remove any processes from the cgroup first (move them to root cgroup)
# If the cgroup is empty, just delete it:
sudo rmdir /sys/fs/cgroup/cpu-test

# If there are processes in it, move them first:
cat /sys/fs/cgroup/cpu-test/cgroup.procs | while read pid; do
    echo $pid | sudo tee /sys/fs/cgroup/cgroup.procs
done
sudo rmdir /sys/fs/cgroup/cpu-test
```

Or use cgroup-tool if you have implemented the delete command:

```bash
sudo cargo run -p cgroup-tool -- delete cpu-test
```

## Common Errors

1. **`No such file or directory` when writing to cpu.max**
   - Cause: The cgroup directory does not exist
   - Fix: Create the cgroup first: `sudo mkdir /sys/fs/cgroup/your-cgroup`
   - Or use: `sudo cargo run -p cgroup-tool -- create your-cgroup`

2. **`Invalid argument` when writing to cpu.max**
   - Cause: The quota/period format is invalid
   - Fix: Ensure the format is `QUOTA PERIOD` where both are numbers (or "max" for quota)
   - Valid examples: `"50000 100000"`, `"max 100000"`, `"100000"`
   - Invalid examples: `"50%"`, `"50000,100000"`, `"50000/100000"`

3. **`Permission denied` when writing to cpu.max**
   - Cause: Not running with root privileges
   - Fix: Use `sudo` when running the command
   - For tests: `sudo -E cargo test -p cgroup-tool --test cpu_test`

4. **CPU limit seems to not be working (process uses more CPU than expected)**
   - Cause: The limit is per-CPU, not total system CPU
   - Example: On a 4-core system, `50000 100000` limits to 50% of ONE core
   - To limit total CPU, multiply quota by number of cores you want to allow
   - Also check: the process must be attached to the cgroup (`cgroup.procs`)

5. **`nr_throttled` stays at 0 even with a low quota**
   - Cause: No CPU-intensive processes are in the cgroup, or the process is I/O bound
   - Fix: Ensure processes are attached and actively using CPU
   - I/O-bound processes spend most time waiting, not consuming CPU quota

## Notes

**Understanding the quota/period relationship:**
- Smaller periods (e.g., 10000 = 10ms) give smoother scheduling but more overhead
- Larger periods (e.g., 1000000 = 1s) allow longer bursts but can cause latency issues
- The default period (100000 = 100ms) is a good balance for most workloads
- You can write just the quota (e.g., `"50000"`) and the kernel uses the existing period

**CPU quota vs. CPU shares (cpu.weight):**
- `cpu.max` sets a **hard limit** - processes cannot exceed this regardless of available CPU
- `cpu.weight` (1-10000, default 100) sets **relative priority** - determines how CPU is shared when there is contention
- Use `cpu.max` when you need guaranteed limits (e.g., billing, isolation)
- Use `cpu.weight` when you want fair sharing under load (e.g., multi-tenant systems)

**Multi-core considerations:**
- Quota is the total CPU time across ALL cores combined
- A quota of `200000` with period `100000` allows 200% CPU = 2 full cores
- To limit to X% of the total system, calculate: `quota = (X / 100) * num_cores * period`

**Burst behavior:**
- A process can use its entire quota in a burst at the start of a period
- This means a 50% limit might show as 100% CPU for 50ms, then 0% for 50ms
- For latency-sensitive workloads, consider using smaller periods

**Relationship to scheduler:**
- The CPU controller uses the CFS (Completely Fair Scheduler) bandwidth feature
- Throttling happens at the scheduler level, not through signals or process suspension
- Throttled processes are simply not scheduled until the next period

**Kernel documentation:**
- `Documentation/admin-guide/cgroup-v2.rst` in the kernel source
- Section "CPU" covers cpu.max, cpu.weight, and cpu.stat

**Minimum values:**
- The kernel enforces minimum period and quota values
- Typically: minimum period is 1000 (1ms), minimum quota is 1000 (1ms)
- Values below minimums may be silently adjusted

## Next
`04-io.md` - Control I/O bandwidth with the io controller
