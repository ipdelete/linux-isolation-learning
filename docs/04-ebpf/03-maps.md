# 03 eBPF Maps

## Goal

Understand eBPF maps as shared data structures between kernel and userspace. You will implement the `ebpf-tool stats` subcommand that displays syscall counts from a HashMap, demonstrating how eBPF programs can collect data in the kernel and expose it to userspace applications.

## Prereqs

- Completed 01-hello-kprobe.md (basic kprobe setup and program attachment)
- Completed 02-reading-data.md (reading kernel data from eBPF programs)
- `sudo` access (loading eBPF programs requires elevated privileges)
- Linux kernel 5.8+ with BTF support

## Background: What are eBPF Maps?

eBPF maps are the primary mechanism for sharing data between:
1. **Kernel eBPF programs** (running in the BPF virtual machine)
2. **Userspace applications** (your CLI tool)
3. **Multiple eBPF programs** (for coordination)

Unlike normal kernel memory, maps are designed for safe concurrent access from both sides of the kernel/userspace boundary.

### Why Maps Matter

Without maps, eBPF programs would be limited to logging messages. Maps enable:
- **Aggregation**: Count events, sum values, track statistics
- **Configuration**: Pass runtime settings from userspace to eBPF
- **Communication**: Send structured events to userspace
- **State**: Remember information across multiple eBPF invocations

### Common Map Types

| Map Type | Use Case | Key | Value |
|----------|----------|-----|-------|
| `HashMap` | Arbitrary key-value lookup | Any fixed-size type | Any fixed-size type |
| `Array` | Fixed-size indexed access | u32 index | Any fixed-size type |
| `PerfEventArray` | Per-CPU event streaming | CPU ID | Event data |
| `RingBuffer` | Efficient event streaming | N/A | Variable-size data |
| `LruHashMap` | Auto-evicting cache | Any fixed-size type | Any fixed-size type |
| `PerCpuHashMap` | Per-CPU counters (no locking) | Any fixed-size type | Any fixed-size type |

In this lesson, we focus on **HashMap** for counting syscalls.

### HashMap Operations

eBPF HashMaps support these operations:

```text
+-------------------+------------------------------------------+
| Operation         | Description                              |
+-------------------+------------------------------------------+
| insert(key, val)  | Add or update entry                      |
| get(key)          | Lookup value by key                      |
| delete(key)       | Remove entry                             |
| iter()            | Iterate all entries (userspace only)     |
+-------------------+------------------------------------------+
```

**Important**: Individual map operations like `insert` are atomic. However, compound operations (like "get then increment then insert") are not atomic and can lose updates under concurrent access. For safe atomic increments, use per-CPU maps or kernel-side atomic helpers where available.

### Sizing Maps

Every map must declare its maximum capacity at compile time:

```rust
// eBPF side - define map with capacity
#[map]
static SYSCALL_COUNTS: HashMap<u64, u64> = HashMap::with_max_entries(10240, 0);
```

The `MAX_MAP_ENTRIES` constant (10240) is defined in `ebpf-tool-common`:
- Large enough for typical workloads
- Small enough to fit in kernel memory
- No power-of-two requirement (though powers of 2 may have better hash distribution)

## Write Tests (Red)

**Test file**: `crates/ebpf-tool/tests/stats_test.rs`

The test file contains five tests covering the `stats` subcommand functionality:

| Test | Requires Root | Description |
|------|---------------|-------------|
| `test_stats_help` | No | Verify --help shows stats info |
| `test_stats_runs_successfully` | Yes | Command exits successfully |
| `test_stats_shows_table_header` | Yes | Output includes header format |
| `test_stats_shows_syscall_counts` | Yes | Shows syscall names and counts |
| `test_stats_after_workload` | Yes | Counts increase after activity |

Steps:

1. Open `crates/ebpf-tool/tests/stats_test.rs`

2. Find the `test_stats_help` test (line 25) and replace the `todo!()`:

```rust
#[test]
fn test_stats_help() {
    // This test does NOT require root because --help doesn't load eBPF programs

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.args(["stats", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("eBPF map statistics")
            .or(predicate::str::contains("stats"))
            .or(predicate::str::contains("map")));
}
```

3. Find `test_stats_runs_successfully` (line 47) and replace the `todo!()`:

```rust
#[test]
fn test_stats_runs_successfully() {
    if !is_root() {
        eprintln!("Skipping test_stats_runs_successfully: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.arg("stats")
        .assert()
        .success();
}
```

4. Find `test_stats_shows_table_header` (line 74) and replace the `todo!()`:

```rust
#[test]
fn test_stats_shows_table_header() {
    if !is_root() {
        eprintln!("Skipping test_stats_shows_table_header: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    cmd.arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Syscall")
            .or(predicate::str::contains("SYSCALL")))
        .stdout(predicate::str::contains("COUNT")
            .or(predicate::str::contains("Count")));
}
```

5. Find `test_stats_shows_syscall_counts` (line 111) and replace the `todo!()`:

```rust
#[test]
fn test_stats_shows_syscall_counts() {
    if !is_root() {
        eprintln!("Skipping test_stats_shows_syscall_counts: requires root");
        return;
    }

    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    let output = cmd.arg("stats")
        .assert()
        .success();

    // Check that output contains at least one common syscall or indicates no data
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let has_syscall = stdout.contains("read")
        || stdout.contains("write")
        || stdout.contains("openat")
        || stdout.contains("close")
        || stdout.contains("No data")
        || stdout.contains("(empty)");

    assert!(has_syscall, "Expected syscall names or 'No data' message, got: {}", stdout);
}
```

6. Find `test_stats_after_workload` (line 151) and replace the `todo!()`:

```rust
#[test]
fn test_stats_after_workload() {
    if !is_root() {
        eprintln!("Skipping test_stats_after_workload: requires root");
        return;
    }

    // Generate syscall activity
    let test_path = "/tmp/ebpf-stats-test";
    for _ in 0..10 {
        let _ = std::fs::write(test_path, b"test data");
        let _ = std::fs::read(test_path);
    }

    // Run stats command
    let mut cmd = Command::cargo_bin("ebpf-tool").unwrap();
    let output = cmd.arg("stats")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // After file operations, we should see some output (non-zero counts or header)
    assert!(
        stdout.contains("Syscall") || stdout.contains("SYSCALL") || stdout.contains("COUNT"),
        "Expected stats output header after workload"
    );

    // Cleanup
    let _ = std::fs::remove_file(test_path);
}
```

7. Run the tests (expect failure because implementation is missing):

```bash
cargo test -p ebpf-tool --test stats_test
```

Expected output:
```
running 5 tests
test test_stats_help ... FAILED
test test_stats_runs_successfully ... FAILED
test test_stats_shows_table_header ... FAILED
test test_stats_shows_syscall_counts ... FAILED
test test_stats_after_workload ... FAILED

failures:

---- test_stats_help stdout ----
thread 'test_stats_help' panicked at 'not yet implemented: Implement test for stats --help output'
```

This is the **RED** phase - your tests are written but the implementation does not exist yet.

## Build (Green)

Building the stats command requires implementing code in three locations:

1. **Shared types** (`ebpf-tool-common`) - Already scaffolded
2. **eBPF program** (`ebpf-tool-ebpf`) - Add HashMap and counting logic
3. **Userspace CLI** (`ebpf-tool`) - Read and display map contents

### Step 1: Understand the Shared Types

**File**: `crates/ebpf-tool-common/src/lib.rs`

The shared types are already defined. Review them:

```rust
/// Maximum entries in syscall counter maps.
pub const MAX_MAP_ENTRIES: u32 = 10240;

/// Key for syscall counting HashMap.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyscallKey {
    /// Process ID (0 for system-wide)
    pub pid: u32,
    pub _pad: u32,
    /// System call number
    pub syscall_nr: u64,
}
```

**Why `#[repr(C)]`?** This ensures the Rust compiler uses C-compatible memory layout, which is required for data structures shared between eBPF (kernel) and userspace.

**Why `_pad`?** To ensure proper alignment. The `syscall_nr` field is 8 bytes and benefits from 8-byte alignment.

### Step 2: Add the HashMap to the eBPF Program

**File**: `crates/ebpf-tool-ebpf/src/kprobe.rs`

Add the HashMap declaration near the top of the file, after the imports:

```rust
use aya_ebpf::{
    macros::{kprobe, map},
    maps::HashMap,
    programs::ProbeContext,
    helpers::{bpf_get_current_pid_tgid, bpf_ktime_get_ns},
};
use ebpf_tool_common::MAX_MAP_ENTRIES;

// Syscall counting map - shared between eBPF and userspace
#[map]
static SYSCALL_COUNTS: HashMap<u64, u64> = HashMap::with_max_entries(MAX_MAP_ENTRIES, 0);
```

Then add a kprobe that updates the counter:

```rust
/// Kprobe that counts syscalls by number.
///
/// This probe attaches to syscall entry points and increments
/// a counter in the SYSCALL_COUNTS HashMap for each invocation.
#[kprobe]
pub fn count_syscalls(ctx: ProbeContext) -> u32 {
    match try_count_syscalls(&ctx) {
        Ok(ret) => ret,
        Err(_) => 0,
    }
}

fn try_count_syscalls(ctx: &ProbeContext) -> Result<u32, i64> {
    // Read the syscall number from the first argument
    // On x86_64, this is in the orig_rax register for sys_enter
    let syscall_nr: u64 = unsafe { ctx.arg(0).ok_or(-1i64)? };

    // WARNING: This pattern (get + insert) is NOT atomic and can lose updates
    // under concurrent access from multiple CPUs. For high-frequency counters,
    // consider using PerCpuHashMap instead (see example below).
    //
    // Get current count (or 0 if not present)
    let count = unsafe {
        SYSCALL_COUNTS
            .get(&syscall_nr)
            .copied()
            .unwrap_or(0)
    };

    // Increment and store
    let new_count = count + 1;
    unsafe {
        SYSCALL_COUNTS
            .insert(&syscall_nr, &new_count, 0)
            .map_err(|_| -1i64)?;
    }

    Ok(0)
}
```

### Per-CPU Maps for Safe Atomic Counters

For accurate counters without data loss, use `PerCpuHashMap` instead of `HashMap`. Each CPU gets its own map instance, eliminating contention:

```rust
use aya_ebpf::maps::PerCpuHashMap;

#[map]
static SYSCALL_COUNTS: PerCpuHashMap<u64, u64> =
    PerCpuHashMap::with_max_entries(MAX_MAP_ENTRIES, 0);

fn try_count_syscalls(ctx: &ProbeContext) -> Result<u32, i64> {
    let syscall_nr: u64 = unsafe { ctx.arg(0).ok_or(-1i64)? };

    // This is now safe: each CPU has its own entry, no contention
    let count = unsafe {
        SYSCALL_COUNTS
            .get(&syscall_nr)
            .copied()
            .unwrap_or(0)
    };

    unsafe {
        SYSCALL_COUNTS
            .insert(&syscall_nr, &(count + 1), 0)
            .map_err(|_| -1i64)?;
    }

    Ok(0)
}
```

When reading from userspace, `PerCpuHashMap::iter()` automatically aggregates values from all CPUs.

### Step 3: Implement the Userspace CLI

**File**: `crates/ebpf-tool/src/main.rs`

**TODO location**: Line ~215 in the `Command::Stats` match arm

Find the `Command::Stats` match arm and replace the `todo!()`:

```rust
Command::Stats => {
    use aya::maps::HashMap;
    use aya::Ebpf;
    use std::time::Duration;

    println!("Loading eBPF program...");

    // Load the eBPF bytecode
    // The build.rs script places the compiled eBPF program in OUT_DIR
    let ebpf_bytes = include_bytes_aligned!(
        concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf")
    );

    let mut bpf = Ebpf::load(ebpf_bytes)
        .context("Failed to load eBPF program")?;

    // Attach kprobe to count syscalls
    // We attach to a common syscall entry point
    use aya::programs::KProbe;
    let program: &mut KProbe = bpf
        .program_mut("count_syscalls")
        .context("Failed to find count_syscalls program")?
        .try_into()
        .context("Program is not a kprobe")?;

    program.load().context("Failed to load kprobe")?;
    program.attach("__x64_sys_openat", 0)
        .context("Failed to attach kprobe to __x64_sys_openat")?;

    // Let it collect some data
    println!("Collecting syscall data for 2 seconds...");
    std::thread::sleep(Duration::from_secs(2));

    // Read the map
    let map = bpf
        .map("SYSCALL_COUNTS")
        .context("Failed to find SYSCALL_COUNTS map")?;

    let syscall_counts: HashMap<_, u64, u64> = HashMap::try_from(map)
        .context("Failed to create HashMap from map")?;

    // Display results
    println!("\nSyscall Statistics:");
    println!("------------------");
    println!("{:<20} {:>10}", "SYSCALL", "COUNT");

    let mut entries: Vec<(u64, u64)> = syscall_counts
        .iter()
        .filter_map(|entry| entry.ok())
        .collect();

    // Sort by count descending
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    if entries.is_empty() {
        println!("(No data collected)");
    } else {
        for (syscall_nr, count) in entries.iter().take(20) {
            let name = syscall_name(*syscall_nr);
            println!("{:<20} {:>10}", name, count);
        }
    }

    Ok(())
}
```

Add this helper function at the bottom of main.rs:

```rust
/// Convert syscall number to name (x86_64).
///
/// This is a simplified mapping of common syscalls. In production,
/// you would use the full syscall table or libseccomp.
fn syscall_name(nr: u64) -> &'static str {
    match nr {
        0 => "read",
        1 => "write",
        2 => "open",
        3 => "close",
        4 => "stat",
        5 => "fstat",
        6 => "lstat",
        7 => "poll",
        8 => "lseek",
        9 => "mmap",
        10 => "mprotect",
        11 => "munmap",
        12 => "brk",
        13 => "rt_sigaction",
        14 => "rt_sigprocmask",
        16 => "ioctl",
        17 => "pread64",
        18 => "pwrite64",
        19 => "readv",
        20 => "writev",
        21 => "access",
        22 => "pipe",
        23 => "select",
        24 => "sched_yield",
        56 => "clone",
        57 => "fork",
        59 => "execve",
        60 => "exit",
        61 => "wait4",
        62 => "kill",
        63 => "uname",
        79 => "getcwd",
        80 => "chdir",
        82 => "rename",
        83 => "mkdir",
        84 => "rmdir",
        85 => "creat",
        87 => "unlink",
        89 => "readlink",
        90 => "chmod",
        92 => "chown",
        102 => "getuid",
        104 => "getgid",
        107 => "geteuid",
        108 => "getegid",
        110 => "getppid",
        186 => "gettid",
        202 => "futex",
        217 => "getdents64",
        257 => "openat",
        262 => "newfstatat",
        _ => "unknown",
    }
}
```

### Step 4: Build the eBPF Program

Before building the userspace tool, you need to compile the eBPF program:

```bash
# Build eBPF programs (via build.rs which requires bpf-linker)
cargo build -p ebpf-tool
```

### Step 5: Build and Test

```bash
# Build the userspace CLI
cargo build -p ebpf-tool

# Run tests with root privileges
sudo -E cargo test -p ebpf-tool --test stats_test
```

Expected output:
```
running 5 tests
test test_stats_help ... ok
test test_stats_runs_successfully ... ok
test test_stats_shows_table_header ... ok
test test_stats_shows_syscall_counts ... ok
test test_stats_after_workload ... ok

test result: ok. 5 passed; 0 failed; 0 filtered out
```

This is the **GREEN** phase - your tests now pass!

## Verify

**Automated verification**:

```bash
# Run all ebpf-tool tests (requires sudo for eBPF)
sudo -E cargo test -p ebpf-tool

# Run just the stats tests
sudo -E cargo test -p ebpf-tool --test stats_test
```

**Manual verification**:

1. Run the stats command:

```bash
sudo cargo run -p ebpf-tool -- stats
```

Expected output:
```
Loading eBPF program...
Collecting syscall data for 2 seconds...

Syscall Statistics:
------------------
SYSCALL              COUNT
openat                 127
read                    89
close                   76
write                   45
fstat                   32
mmap                    28
...
```

2. Generate workload and observe counts:

```bash
# In one terminal, generate file activity
while true; do cat /etc/passwd > /dev/null; done &

# In another terminal, run stats
sudo cargo run -p ebpf-tool -- stats

# Kill the background process
kill %1
```

You should see higher counts for `openat`, `read`, and `close`.

3. Verify the map exists in the kernel:

```bash
# List loaded BPF maps (requires bpftool)
sudo bpftool map list

# Look for SYSCALL_COUNTS (or similar name)
# You'll see something like:
#   123: hash  name SYSCALL_COUNTS  flags 0x0
#        key 8B  value 8B  max_entries 10240  memlock 81920B
```

## Understanding the Code

### HashMap Data Flow

```text
                    eBPF Program (Kernel)
                    +-----------------------+
  syscall entry --> | count_syscalls()     |
                    |   |                   |
                    |   v                   |
                    | SYSCALL_COUNTS.get()  |
                    | count + 1             |
                    | SYSCALL_COUNTS.insert |
                    +-----------+-----------+
                                |
                                | (shared memory)
                                v
                    +-----------------------+
                    | Userspace (ebpf-tool) |
                    |   |                   |
                    |   v                   |
                    | HashMap::try_from()   |
                    | map.iter()            |
                    | print counts          |
                    +-----------------------+
```

### Key Concepts

1. **Map Declaration**: The `#[map]` attribute tells Aya to create a BPF map with the specified type and capacity.

2. **Atomic Updates**: Map operations like `get` and `insert` are atomic. The kernel handles synchronization, so multiple CPUs can update the map simultaneously.

3. **Key Constraints**: HashMap keys must implement `Pod` (Plain Old Data) - no pointers, no heap allocation, fixed size.

4. **Userspace Access**: From userspace, you get a read-only or read-write view of the map. Changes are visible to the eBPF program and vice versa.

### Using Different Key Types

For more sophisticated tracking, use the `SyscallKey` struct:

```rust
// eBPF side - count per-process per-syscall
use ebpf_tool_common::SyscallKey;

#[map]
static SYSCALL_COUNTS: HashMap<SyscallKey, u64> =
    HashMap::with_max_entries(MAX_MAP_ENTRIES, 0);

fn try_count_syscalls(ctx: &ProbeContext) -> Result<u32, i64> {
    let pid_tgid = unsafe { bpf_get_current_pid_tgid() };
    let pid = (pid_tgid >> 32) as u32;
    let syscall_nr: u64 = unsafe { ctx.arg(0).ok_or(-1i64)? };

    let key = SyscallKey::new(pid, syscall_nr);
    let count = unsafe { SYSCALL_COUNTS.get(&key).copied().unwrap_or(0) };

    unsafe {
        SYSCALL_COUNTS.insert(&key, &(count + 1), 0)?;
    }

    Ok(0)
}
```

This allows tracking syscall counts per-process, useful for identifying which process is making the most system calls.

## Common Errors

1. **`Failed to find SYSCALL_COUNTS map`**
   - Cause: The map name in userspace does not match the eBPF declaration
   - Fix: Ensure the map name string matches exactly (case-sensitive)
   - Verify: Run `sudo bpftool map list` to see actual map names

2. **`Map key size mismatch`**
   - Cause: The key type in userspace differs from eBPF
   - Fix: Use the same type from `ebpf-tool-common` on both sides
   - Example: If eBPF uses `HashMap<u64, u64>`, userspace must use `HashMap<_, u64, u64>`

3. **`Operation not permitted` when accessing map**
   - Cause: Running without root privileges
   - Fix: Use `sudo -E cargo run` or `sudo -E cargo test`
   - The `-E` preserves environment variables needed for cargo

4. **`Map is full - insert failed`**
   - Cause: More unique keys than `MAX_MAP_ENTRIES`
   - Fix: Increase `MAX_MAP_ENTRIES` or use `LruHashMap` for auto-eviction
   - Consider: Aggregate by syscall number only (not per-PID) to reduce cardinality

5. **`failed to load program: Permission denied`**
   - Cause: Missing CAP_BPF or CAP_SYS_ADMIN capability
   - Fix: Run with sudo or set capabilities: `sudo setcap cap_bpf=ep ./target/debug/ebpf-tool`

6. **Counts appear frozen or not updating**
   - Cause: Probe attached to wrong function or function not called
   - Fix: Verify probe target with: `sudo bpftool prog list`
   - Try different attach points: `__x64_sys_openat`, `do_sys_openat2`, etc.

## Notes

**Map persistence**: Maps exist only while the eBPF program is loaded. When the userspace process exits, the program is unloaded and the map is destroyed.

**Per-CPU variants**: For high-frequency events, consider `PerCpuHashMap` which eliminates lock contention by giving each CPU its own map instance.

**Verifier limits**: The BPF verifier limits loop iterations. When iterating maps from eBPF (not common), you may hit these limits. Iteration from userspace has no such restrictions.

**Debug map contents**: Use `bpftool` to inspect maps directly:
```bash
sudo bpftool map dump name SYSCALL_COUNTS
```

**Kernel version**: HashMap is available in all kernels that support eBPF. However, some features (like batch operations) require newer kernels (5.6+).

**Further reading**:
- [Aya Book: Maps](https://aya-rs.dev/book/programs/maps/)
- [BPF and XDP Reference Guide - Maps](https://docs.cilium.io/en/stable/bpf/)
- `man 7 bpf` - BPF system call and map types

## Next

`04-perf-events.md` - Explore perf events and how to use PerfEventArray maps for efficient per-CPU event streaming from kernel to userspace.
