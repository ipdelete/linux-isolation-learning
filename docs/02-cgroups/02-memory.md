# 02 Memory Controller

## Goal
Set memory limits for a cgroup using `memory.max`. You'll build a `memory-max` subcommand in `cgroup-tool` that writes a byte limit to the cgroup's memory controller, preventing processes in that cgroup from using more than the specified amount of memory.

## Prereqs
- Completed `docs/02-cgroups/01-cgv2-basics.md` (can create and delete cgroups)
- `sudo` access (writing to cgroup control files requires root)
- Basic understanding of cgroup v2 filesystem structure (`/sys/fs/cgroup`)

## Background: The Memory Controller

The cgroup v2 memory controller provides hard memory limits and usage tracking for processes. When you constrain a cgroup's memory, the kernel enforces those limits and takes action (OOM kill) when processes exceed them.

**Key control files:**

| File | Purpose |
|------|---------|
| `memory.max` | Hard memory limit in bytes. Processes are OOM-killed if they exceed this. |
| `memory.current` | Current memory usage in bytes (read-only). |
| `memory.high` | Soft limit - processes get throttled but not killed. |
| `memory.events` | Counters for memory events including OOM kills. |
| `memory.swap.max` | Maximum swap usage (if swap is available). |

**Important values for memory.max:**
- A numeric value sets the limit in bytes (e.g., `104857600` for 100 MB)
- The string `max` means unlimited (no constraint)
- Values are automatically rounded to page size (typically 4096 bytes)

**What happens when the limit is exceeded:**
1. The kernel tries to reclaim memory from the cgroup (flushing caches, swapping)
2. If reclamation fails and memory.max is still exceeded, the OOM killer activates
3. The kernel picks a process in the cgroup and sends it SIGKILL
4. The `memory.events` file increments its `oom_kill` counter

**Why this matters for containers:**
- Memory limits prevent runaway processes from consuming all host memory
- Containers can be guaranteed a predictable memory budget
- OOM events are isolated to the misbehaving container, not the whole system

## Write Tests (Red)

**Test file**: `crates/cgroup-tool/tests/memory_test.rs`

What the tests should verify:
- Success case: Writing a byte value to `memory.max` persists that value
- Success case: Reading back `memory.max` shows the written value
- Edge case: Large values (multi-GB) are handled correctly

Steps:

1. Open `crates/cgroup-tool/tests/memory_test.rs`

2. Find the `test_set_memory_limit` test function (line 12)

3. Replace the `todo!()` with a test implementation:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

// Helper to get the cgroup root path
fn cgroup_root() -> &'static str {
    "/sys/fs/cgroup"
}

// Helper to generate unique test cgroup names
fn test_cgroup_name(suffix: &str) -> String {
    format!("rust-test-memory-{}-{}", std::process::id(), suffix)
}

#[test]
fn test_set_memory_limit() {
    // Skip if not running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_set_memory_limit: requires root privileges");
        return;
    }

    let cgroup_name = test_cgroup_name("basic");
    let cgroup_path = format!("{}/{}", cgroup_root(), cgroup_name);
    let memory_max_path = format!("{}/memory.max", cgroup_path);

    // Step 1: Create the test cgroup directory
    fs::create_dir_all(&cgroup_path)
        .expect("Failed to create test cgroup - is cgroup v2 mounted?");

    // Step 2: Run our tool to set memory limit (100 MB = 104857600 bytes)
    let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    cmd.arg("memory-max")
        .arg(&cgroup_name)
        .arg("104857600")
        .assert()
        .success();

    // Step 3: Verify memory.max contains the expected value
    let memory_max = fs::read_to_string(&memory_max_path)
        .expect("Failed to read memory.max");

    // Note: The kernel may return the value with a newline
    let value: u64 = memory_max.trim().parse()
        .expect("memory.max should contain a numeric value");

    assert_eq!(value, 104857600,
        "memory.max should be set to 104857600 (100 MB)");

    // Step 4: Clean up - remove the cgroup
    // (cgroup must be empty of processes to delete)
    fs::remove_dir(&cgroup_path)
        .expect("Failed to clean up test cgroup");
}
```

4. Optionally, enable and implement the test for unlimited memory:

```rust
#[test]
fn test_set_memory_max_unlimited() {
    // Skip if not running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Skipping test_set_memory_max_unlimited: requires root privileges");
        return;
    }

    let cgroup_name = test_cgroup_name("unlimited");
    let cgroup_path = format!("{}/{}", cgroup_root(), cgroup_name);
    let memory_max_path = format!("{}/memory.max", cgroup_path);

    // Create the test cgroup
    fs::create_dir_all(&cgroup_path)
        .expect("Failed to create test cgroup");

    // First set a limit
    let mut cmd = Command::cargo_bin("cgroup-tool").unwrap();
    cmd.arg("memory-max")
        .arg(&cgroup_name)
        .arg("52428800")  // 50 MB
        .assert()
        .success();

    // Now remove the limit by setting to max
    // Note: This requires handling "max" as a special string in the CLI
    // For now, we test numeric values only - unlimited is a stretch goal

    // Verify the limit was set
    let memory_max = fs::read_to_string(&memory_max_path)
        .expect("Failed to read memory.max");

    let value: u64 = memory_max.trim().parse()
        .expect("memory.max should contain a numeric value");

    assert_eq!(value, 52428800);

    // Clean up
    fs::remove_dir(&cgroup_path)
        .expect("Failed to clean up test cgroup");
}
```

5. Add the necessary imports at the top of the test file (if not already present):

```rust
// Tests for the `memory-max` subcommand (memory limits)
// Lesson: docs/02-cgroups/02-memory.md

use assert_cmd::Command;
use std::fs;

fn cgroup_root() -> &'static str {
    "/sys/fs/cgroup"
}

fn test_cgroup_name(suffix: &str) -> String {
    format!("rust-test-memory-{}-{}", std::process::id(), suffix)
}
```

6. Run the tests (expect failure because implementation is missing):

```bash
sudo -E cargo test -p cgroup-tool --test memory_test
```

Expected output:
```
running 4 tests
test test_memory_current_tracking ... ignored
test test_memory_limit_enforcement ... ignored
test test_set_memory_max_unlimited ... ignored
test test_set_memory_limit ... FAILED

failures:

---- test_set_memory_limit stdout ----
thread 'test_set_memory_limit' panicked at 'not yet implemented: Implement memory limit - write tests first!', crates/cgroup-tool/src/main.rs:94:13
```

This is the **RED** phase - your tests are written but the implementation returns `todo!()`.

## Build (Green)

**Implementation file**: `crates/cgroup-tool/src/main.rs`
**TODO location**: Line ~93 in the `Command::MemoryMax` match arm

Now implement the memory limit functionality to make your tests pass.

Steps:

1. Open `crates/cgroup-tool/src/main.rs`

2. Find the `Command::MemoryMax { path, bytes } => todo!(...)` match arm (around line 93)

3. Replace the `todo!()` with the implementation:

```rust
Command::MemoryMax { path, bytes } => {
    use std::fs;
    use std::path::Path;

    // Construct the full path to memory.max
    // path is relative to cgroup root (e.g., "my-cgroup" or "parent/child")
    let cgroup_root = Path::new("/sys/fs/cgroup");
    let memory_max_path = cgroup_root.join(&path).join("memory.max");

    // Verify the cgroup exists before attempting to write
    if !memory_max_path.exists() {
        anyhow::bail!(
            "Cgroup '{}' does not exist or has no memory controller. \
             Path checked: {:?}",
            path,
            memory_max_path
        );
    }

    // Write the byte value to memory.max
    // The kernel expects the value as a decimal string
    fs::write(&memory_max_path, bytes.to_string())
        .with_context(|| format!(
            "Failed to write {} to {:?}. \
             Do you have permission? Is the cgroup still valid?",
            bytes, memory_max_path
        ))?;

    // Verify the write by reading back
    let readback = fs::read_to_string(&memory_max_path)
        .with_context(|| format!("Failed to read back {:?}", memory_max_path))?;

    println!("Set memory.max to {} bytes for cgroup '{}'", bytes, path);
    println!("Verified: memory.max = {}", readback.trim());

    Ok(())
}
```

4. Make sure you have the necessary imports at the top of main.rs:

```rust
use anyhow::{Context, Result};
```

5. Run the tests (expect success):

```bash
sudo -E cargo test -p cgroup-tool --test memory_test
```

Expected output:
```
running 4 tests
test test_memory_current_tracking ... ignored
test test_memory_limit_enforcement ... ignored
test test_set_memory_max_unlimited ... ignored
test test_set_memory_limit ... ok

test result: ok. 1 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out
```

This is the **GREEN** phase - your test passes!

## Verify

**Automated verification**:
```bash
# Run all cgroup-tool tests (requires sudo for cgroup operations)
sudo -E cargo test -p cgroup-tool

# Run just the memory tests
sudo -E cargo test -p cgroup-tool --test memory_test
```

**Manual verification** (observe the actual behavior):

1. Create a test cgroup:
```bash
sudo mkdir -p /sys/fs/cgroup/manual-test
```

2. Check the default memory.max (should be "max" = unlimited):
```bash
cat /sys/fs/cgroup/manual-test/memory.max
```

Expected output:
```
max
```

3. Set a 100 MB memory limit using your tool:
```bash
sudo cargo run -p cgroup-tool -- memory-max manual-test 104857600
```

Expected output:
```
Set memory.max to 104857600 bytes for cgroup 'manual-test'
Verified: memory.max = 104857600
```

4. Verify the limit was set:
```bash
cat /sys/fs/cgroup/manual-test/memory.max
```

Expected output:
```
104857600
```

5. Check current memory usage (should be 0 or very small):
```bash
cat /sys/fs/cgroup/manual-test/memory.current
```

Expected output:
```
0
```

6. View memory events (OOM kill counter should be 0):
```bash
cat /sys/fs/cgroup/manual-test/memory.events
```

Expected output:
```
low 0
high 0
max 0
oom 0
oom_kill 0
oom_group_kill 0
```

7. (Optional) Test OOM kill behavior - **WARNING: This will kill a process!**
```bash
# Start a shell in the cgroup
echo $$ | sudo tee /sys/fs/cgroup/manual-test/cgroup.procs

# Try to allocate more memory than allowed (will be OOM killed)
# This command will be killed and your shell will exit!
# Only run this if you understand the consequences
```

## Clean Up

Remove the test cgroup:
```bash
# First, ensure no processes are attached
cat /sys/fs/cgroup/manual-test/cgroup.procs

# If the output is empty, you can remove it
sudo rmdir /sys/fs/cgroup/manual-test

# If processes are attached, move them back to root cgroup first
# (replace PID with actual process IDs from the cgroup.procs output)
# echo <PID> | sudo tee /sys/fs/cgroup/cgroup.procs
```

If the cgroup has child cgroups, remove them first (deepest children first):
```bash
# List any child cgroups
ls -la /sys/fs/cgroup/manual-test/

# Remove children before parent
sudo rmdir /sys/fs/cgroup/manual-test/child-cgroup
sudo rmdir /sys/fs/cgroup/manual-test
```

## Common Errors

1. **`No such file or directory` when writing to memory.max**
   - Cause: The cgroup doesn't exist yet, or you're using the wrong path
   - Fix: Create the cgroup first with `mkdir -p /sys/fs/cgroup/<path>`
   - Also check: Are you including the `/sys/fs/cgroup/` prefix in the path argument? The tool expects just the relative path (e.g., `my-cgroup`, not `/sys/fs/cgroup/my-cgroup`)

2. **`Permission denied` when writing to memory.max**
   - Cause: Not running as root
   - Fix: Run with `sudo`: `sudo cargo run -p cgroup-tool -- memory-max ...`
   - The `-E` flag preserves environment variables when running tests: `sudo -E cargo test`

3. **`Device or resource busy` when trying to delete cgroup**
   - Cause: Processes are still attached to the cgroup, or child cgroups exist
   - Fix: First move all processes to another cgroup, then delete child cgroups from deepest to shallowest
   - Check attached processes: `cat /sys/fs/cgroup/<path>/cgroup.procs`
   - Move process to root: `echo <PID> | sudo tee /sys/fs/cgroup/cgroup.procs`

4. **Memory limit appears slightly different than what was set**
   - Cause: The kernel rounds memory values to page size boundaries (usually 4096 bytes)
   - Example: Setting 104857600 might read back as 104857600 on most systems, but edge cases exist
   - This is normal behavior - not an error

## Notes

**Byte value reference:**

| Human Readable | Bytes |
|----------------|-------|
| 1 MB | 1048576 |
| 10 MB | 10485760 |
| 50 MB | 52428800 |
| 100 MB | 104857600 |
| 256 MB | 268435456 |
| 512 MB | 536870912 |
| 1 GB | 1073741824 |
| 2 GB | 2147483648 |

**Memory controller files explained:**

- **memory.max**: Hard limit. When exceeded, OOM killer activates. Set to `max` for unlimited.
- **memory.current**: Instantaneous memory usage. Includes RSS, page cache, and kernel memory charged to the cgroup.
- **memory.high**: Soft limit. When exceeded, processes are throttled (slowed down) but not killed. Useful for gradual back-pressure.
- **memory.min**: Memory protection. The kernel guarantees at least this much memory even under memory pressure.
- **memory.low**: Best-effort memory protection. Like memory.min but can be violated under extreme pressure.
- **memory.swap.max**: Maximum swap space the cgroup can use. Only relevant if system has swap.

**Page size considerations:**
- Linux memory is managed in pages (typically 4096 bytes on x86_64)
- Memory limits are internally rounded to page boundaries
- Check your system's page size: `getconf PAGE_SIZE`

**Hierarchy behavior:**
- Child cgroups cannot exceed parent's memory.max
- The effective limit is min(own limit, parent limit, grandparent limit, ...)
- Use nested cgroups to create tiered memory allocation

**Documentation links:**
- Kernel cgroup v2 memory controller: https://docs.kernel.org/admin-guide/cgroup-v2.html#memory
- Memory controller design: https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files

**Differences from cgroup v1:**
- v2 unified hierarchy has single `memory.max` vs v1's `memory.limit_in_bytes`
- v2 charges page cache consistently; v1 had complex accounting
- v2 `memory.high` (throttling) is new - v1 had no equivalent
- v2 is recommended for all new deployments

## Next
`03-cpu.md` - Set CPU quotas and periods using the cpu controller
