# Troubleshooting

Comprehensive reference for diagnosing and fixing common errors in this learning path. Organized by category with error messages, causes, diagnostic commands, and fixes.

---

## Quick Diagnostics

Run these commands first to understand your environment:

```bash
# Check kernel version (need 5.x+ for full cgroup v2 support)
uname -r

# Verify cgroup v2 is mounted (unified hierarchy)
mount | grep cgroup2
# Expected: cgroup2 on /sys/fs/cgroup type cgroup2 (rw,...)

# Check available cgroup controllers
cat /sys/fs/cgroup/cgroup.controllers
# Expected: cpuset cpu io memory hugetlb pids rdma misc

# Verify namespace support in kernel
ls /proc/self/ns/
# Expected: cgroup ipc mnt net pid pid_for_children user uts

# Check if running as root
id
# For most operations, uid=0 is required

# Verify runc is installed
runc --version

# Check current capabilities
capsh --print
```

---

## DevContainer Gotchas

This learning path runs in a DevContainer. These are known limitations and workarounds:

### 1. Privileged Mode Required

**Issue**: Most namespace and cgroup operations fail with `EPERM`.

**Cause**: DevContainers run unprivileged by default.

**Fix**: The `.devcontainer/devcontainer.json` must include:
```json
{
  "runArgs": ["--privileged"],
  "capAdd": ["SYS_ADMIN", "NET_ADMIN"]
}
```

### 2. Cgroup v2 Delegation

**Issue**: Cannot create child cgroups or write to controller files.

**Cause**: The DevContainer's cgroup may not have controllers delegated.

**Diagnostic**:
```bash
cat /sys/fs/cgroup/cgroup.subtree_control
# Should show: cpu cpuset io memory pids
```

**Fix**: Enable controllers at the root level (requires host configuration):
```bash
echo "+cpu +memory +io +pids" | sudo tee /sys/fs/cgroup/cgroup.subtree_control
```

### 3. Nested Container Limitations

**Issue**: Running `runc` inside a DevContainer has restrictions.

**Cause**: Some isolation features require kernel-level access that Docker/Podman already uses.

**Workarounds**:
- Use `--rootless` mode for runc where possible
- Some seccomp filters may conflict; adjust OCI config accordingly
- Network namespace creation may require `--net=host` on the DevContainer

### 4. /proc and /sys Mounting

**Issue**: Cannot mount `/proc` or write to `/sys/fs/cgroup`.

**Cause**: These filesystems are bind-mounted from the host.

**Fix**: For `/proc` in a new PID namespace, mount a fresh instance:
```rust
// After unshare(CLONE_NEWPID) and fork()
mount(Some("proc"), "/proc", Some("proc"), MsFlags::empty(), None::<&str>)?;
```

### 5. Test Cleanup Failures

**Issue**: Tests leave behind cgroups or network namespaces.

**Cause**: Test panics or signals interrupt cleanup code.

**Diagnostic**:
```bash
# Find orphaned test cgroups
ls /sys/fs/cgroup/ | grep -E "^rust-test-"

# Find orphaned network namespaces
ip netns list | grep -E "^test-"
```

**Fix**: Manual cleanup:
```bash
# Remove orphaned cgroups (must be empty of processes)
sudo rmdir /sys/fs/cgroup/rust-test-*

# Remove orphaned network namespaces
sudo ip netns delete test-ns-name
```

---

## Environment Issues

### Kernel Version Too Old

**Error**: Various failures with namespace or cgroup operations.

**Cause**: Kernel < 5.x lacks full cgroup v2 support or namespace features.

**Diagnostic**:
```bash
uname -r
# Need 5.x or newer for full feature support
```

**Fix**: Update to a newer kernel or use a modern Linux distribution.

**Reference**: Prerequisites in `docs/00-foundations/00-setup-rust.md`

---

### Cgroup v1 Instead of v2 (Hybrid Mode)

**Error**:
```
cgroup.controllers: No such file or directory
```

**Cause**: System is using cgroup v1 or hybrid mode instead of unified v2.

**Diagnostic**:
```bash
# Check mount type
mount | grep cgroup

# v1 shows multiple mounts like:
# cgroup on /sys/fs/cgroup/memory type cgroup (rw,memory)
# cgroup on /sys/fs/cgroup/cpu type cgroup (rw,cpu)

# v2 shows single unified mount:
# cgroup2 on /sys/fs/cgroup type cgroup2 (rw,...)
```

**Fix**: Boot with `systemd.unified_cgroup_hierarchy=1` kernel parameter, or use a distribution that defaults to cgroup v2 (Ubuntu 22.04+, Fedora 31+).

**Reference**: `docs/02-cgroups/01-cgv2-basics.md`

---

### Container Runtime Conflict

**Error**: Operations succeed but behavior is unexpected, or existing cgroups interfere.

**Cause**: Docker, Podman, or containerd are managing cgroups and may conflict.

**Diagnostic**:
```bash
# Check for container runtime cgroups
ls /sys/fs/cgroup/ | grep -E "docker|podman|containerd|kubepods"

# Check what's using your test cgroup
cat /sys/fs/cgroup/your-test-cgroup/cgroup.procs
```

**Fix**: Use unique cgroup names (the test helpers do this with PID prefixes). Avoid modifying cgroups owned by container runtimes.

**Reference**: `docs/02-cgroups/01-cgv2-basics.md`

---

## Permission Errors

### EPERM - Operation Not Permitted (os error 1)

**Error**:
```
Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
```

**Cause**: Missing `CAP_SYS_ADMIN` capability. Most namespace and cgroup operations require root or specific capabilities.

**Diagnostic**:
```bash
# Check effective user
id

# Check capabilities
capsh --print | grep Current
```

**Fix**: Run with `sudo`:
```bash
sudo -E cargo test -p ns-tool
sudo -E cargo run -p cgroup-tool -- create my-cgroup
```

**Reference**: All namespace lessons in `docs/01-namespaces/`

---

### EACCES - Permission Denied (os error 13)

**Error**:
```
Os { code: 13, kind: PermissionDenied, message: "Permission denied" }
```

**Cause**: File-level permission denied (different from capability-based EPERM). Common when:
- Trying to write to cgroup files without root
- Accessing another user's namespace files
- Mount operations without privileges

**Diagnostic**:
```bash
# Check file permissions
ls -la /sys/fs/cgroup/your-cgroup/

# Check if you can read the file
cat /sys/fs/cgroup/your-cgroup/memory.max
```

**Fix**: Run as root with `sudo`.

**Reference**: `docs/02-cgroups/01-cgv2-basics.md`, `docs/02-cgroups/02-memory.md`

---

### Capability Not Effective After setns

**Error**: Operations fail after joining another namespace, even as root.

**Cause**: When joining a user namespace, capabilities are relative to that namespace. The target namespace may not grant the required capabilities.

**Diagnostic**:
```bash
# Check capabilities in current namespace
capsh --print
```

**Fix**: Ensure the user namespace grants the needed capabilities, or create your own user namespace where you have full capabilities.

**Reference**: `docs/01-namespaces/10-join-existing.md`

---

## Namespace Errors

### unshare() Fails with EPERM

**Error**:
```
unshare(CLONE_NEWUSER) failed: Operation not permitted
```

**Cause**:
- User namespaces may be disabled (`/proc/sys/kernel/unprivileged_userns_clone = 0`)
- Already in a restricted namespace that doesn't allow nesting
- Missing CAP_SYS_ADMIN for non-user namespaces

**Diagnostic**:
```bash
# Check if unprivileged user namespaces are allowed
cat /proc/sys/kernel/unprivileged_userns_clone
# 1 = allowed, 0 = disabled

# Check current namespace depth
ls -la /proc/self/ns/
```

**Fix**:
- For user namespaces: Enable with `echo 1 | sudo tee /proc/sys/kernel/unprivileged_userns_clone`
- For other namespaces: Run with `sudo`

**Reference**: `docs/01-namespaces/08-user-namespace.md`

---

### setns() Fails with ENOENT

**Error**:
```
Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

**Cause**: The namespace file doesn't exist because:
- The target process has exited
- Wrong PID specified
- The namespace type doesn't exist on this kernel

**Diagnostic**:
```bash
# Check if process exists
ps -p <PID>

# Check if namespace file exists
ls -la /proc/<PID>/ns/
```

**Fix**: Verify the target process is still running. Use a long-lived process for testing:
```bash
# Start a long-running process in a new namespace
sudo unshare --pid --fork sleep 3600 &
echo $!  # This is the PID to target
```

**Reference**: `docs/01-namespaces/10-join-existing.md`

---

### setns() for User Namespace Fails with EINVAL

**Error**:
```
Os { code: 22, kind: InvalidInput, message: "Invalid argument" }
```

**Cause**: Attempting to join a user namespace while multi-threaded. The kernel requires the process to be single-threaded.

**Diagnostic**:
```bash
# Check thread count
ls /proc/self/task/ | wc -l
# Should be 1 for setns to user namespace
```

**Fix**: Ensure your program is single-threaded when calling `setns()` for user namespaces. Rust's test harness is multi-threaded by default; use `--test-threads=1`:
```bash
cargo test -- --test-threads=1
```

**Reference**: `docs/01-namespaces/10-join-existing.md`

---

### PID Not 1 Inside New PID Namespace

**Error**: After `unshare(CLONE_NEWPID)`, `getpid()` still returns the original PID.

**Cause**: The calling process remains in the parent PID namespace. Only child processes created after `unshare()` get PID 1.

**Diagnostic**:
```rust
// This shows the problem:
unshare(CloneFlags::CLONE_NEWPID)?;
println!("My PID: {}", getpid());  // Still old PID!
```

**Fix**: Fork after unshare. The child process will be PID 1:
```rust
unshare(CloneFlags::CLONE_NEWPID)?;
match unsafe { fork()? } {
    ForkResult::Child => {
        // Now getpid() returns 1
    }
    ForkResult::Parent { child } => {
        waitpid(child, None)?;
    }
}
```

**Reference**: `docs/01-namespaces/01-pid-namespace.md`, `docs/01-namespaces/09-combine-ns.md`

---

### execvp Fails with ENOENT After Namespace Change

**Error**:
```
Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

**Cause**: After entering a new mount namespace, the expected binary doesn't exist:
- The PATH is different
- The rootfs has changed (pivot_root)
- The binary isn't installed in the target filesystem

**Diagnostic**:
```bash
# In the new namespace, check what's available
ls /bin/ /usr/bin/
echo $PATH
```

**Fix**: Use absolute paths to binaries that exist in the target filesystem, or ensure the binary is present:
```rust
// Use absolute path
execvp(&CString::new("/bin/sh")?, &[CString::new("sh")?])?;
```

**Reference**: `docs/01-namespaces/10-join-existing.md`

---

### Container Exits Immediately

**Error**: The container process starts but exits right away with no output.

**Cause**:
- Parent didn't wait for child process (`waitpid` missing)
- Child process crashed before producing output
- `execvp` failed silently

**Diagnostic**:
```rust
// Add explicit error handling
match unsafe { fork()? } {
    ForkResult::Child => {
        // Add debug output before exec
        eprintln!("Child starting, PID={}", getpid());

        if let Err(e) = execvp(...) {
            eprintln!("execvp failed: {}", e);
            std::process::exit(1);
        }
    }
    ForkResult::Parent { child } => {
        let status = waitpid(child, None)?;
        eprintln!("Child exited with: {:?}", status);
    }
}
```

**Fix**: Always `waitpid()` on child processes and add error handling around `execvp`.

**Reference**: `docs/01-namespaces/09-combine-ns.md`

---

## Mount Errors

### mount() Fails with EBUSY

**Error**:
```
Os { code: 16, kind: ResourceBusy, message: "Device or resource busy" }
```

**Cause**:
- The mount point is already mounted
- A process has the mount point as its current directory
- The filesystem is in use

**Diagnostic**:
```bash
# Check what's mounted at that location
mount | grep /your/mount/point

# Check if any processes use it
lsof /your/mount/point
fuser -v /your/mount/point
```

**Fix**:
- Unmount first if already mounted: `umount /your/mount/point`
- Change directory away from the mount point
- Kill processes using the mount point

**Reference**: `docs/01-namespaces/09-combine-ns.md`

---

### Cannot Mount /proc: Already Mounted

**Error**:
```
mount("proc", "/proc", "proc"): Device or resource busy
```

**Cause**: In the container's mount namespace, `/proc` is already bind-mounted from the host.

**Fix**: Make the mount private first, then mount fresh:
```rust
// Make mount points private to prevent propagation
mount(
    None::<&str>,
    "/",
    None::<&str>,
    MsFlags::MS_REC | MsFlags::MS_PRIVATE,
    None::<&str>,
)?;

// Now mount fresh procfs
mount(
    Some("proc"),
    "/proc",
    Some("proc"),
    MsFlags::empty(),
    None::<&str>,
)?;
```

**Reference**: `docs/01-namespaces/09-combine-ns.md`

---

### pivot_root Fails

**Error**:
```
pivot_root: Invalid argument
```

**Cause**:
- New root and put_old are on the same filesystem
- New root is not a mount point
- put_old is not under new root

**Fix**: Ensure new_root is a mount point (bind mount to itself if needed):
```rust
// Make new_root a mount point
mount(
    Some(new_root),
    new_root,
    None::<&str>,
    MsFlags::MS_BIND,
    None::<&str>,
)?;

// Now pivot_root will work
pivot_root(new_root, put_old)?;
```

**Reference**: `docs/01-namespaces/05-mount-namespace.md`

---

### Overlayfs Mount Fails

**Error**:
```
mount: wrong fs type, bad option, bad superblock
```

**Cause**: Overlayfs options are incorrect or the directories don't exist.

**Diagnostic**:
```bash
# Check if overlayfs is available
cat /proc/filesystems | grep overlay

# Verify directories exist
ls -la /path/to/lower /path/to/upper /path/to/work /path/to/merged
```

**Fix**: Ensure all directories exist and use correct options:
```rust
mount(
    Some("overlay"),
    "/merged",
    Some("overlay"),
    MsFlags::empty(),
    Some("lowerdir=/lower,upperdir=/upper,workdir=/work"),
)?;
```

**Reference**: `docs/01-namespaces/05-mount-namespace.md`

---

## Network Errors

### veth Pair Creation Fails

**Error**:
```
RTNETLINK answers: Operation not permitted
```

**Cause**: Missing `CAP_NET_ADMIN` capability.

**Diagnostic**:
```bash
capsh --print | grep net_admin
```

**Fix**: Run with `sudo` or add `CAP_NET_ADMIN`:
```bash
sudo ip link add veth0 type veth peer name veth1
```

**Reference**: `docs/01-namespaces/07-network-namespace.md`

---

### Loopback Interface Not UP

**Error**: Network operations fail inside namespace; `lo` shows `state DOWN`.

**Cause**: New network namespaces start with loopback down.

**Diagnostic**:
```bash
ip link show lo
# state DOWN means loopback is not active
```

**Fix**: Bring up loopback after entering the namespace:
```rust
// Using ioctl to bring up loopback
let sock = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), None)?;
// ... set IFF_UP flag via ioctl
```

Or via command:
```bash
ip link set lo up
```

**Reference**: `docs/01-namespaces/07-network-namespace.md`, `docs/01-namespaces/09-combine-ns.md`

---

### Container Cannot Access External Network

**Error**: `ping` or network requests from container fail.

**Cause**: This is expected behavior! A new network namespace is completely isolated with no connectivity.

**Diagnostic**:
```bash
# Inside container
ip addr  # Only sees lo
ip route  # No routes
```

**Fix**: Network connectivity requires configuration:
1. Create veth pair connecting container to host
2. Assign IP addresses to both ends
3. Set up routing and NAT (iptables)

This is typically handled by container runtimes, not manually.

**Reference**: `docs/01-namespaces/07-network-namespace.md`

---

### Bridge Interface Issues

**Error**:
```
RTNETLINK answers: File exists
```

**Cause**: Trying to create a bridge that already exists.

**Diagnostic**:
```bash
ip link show type bridge
```

**Fix**: Delete existing bridge first or use a unique name:
```bash
sudo ip link delete br0
sudo ip link add br0 type bridge
```

**Reference**: `docs/03-runc/06-network-integration.md`

---

## Cgroup Errors

### EEXIST - Cgroup Already Exists

**Error**:
```
Os { code: 17, kind: AlreadyExists, message: "File exists" }
```

**Cause**: Trying to create a cgroup that already exists.

**Diagnostic**:
```bash
ls -la /sys/fs/cgroup/your-cgroup-name
```

**Fix**:
- Use a different name
- Delete existing cgroup first (if empty)
- Add unique suffix (PID, timestamp) to names

**Reference**: `docs/02-cgroups/01-cgv2-basics.md`

---

### EBUSY - Cannot Delete Cgroup

**Error**:
```
Os { code: 16, kind: ResourceBusy, message: "Device or resource busy" }
```

**Cause**:
- Processes are still attached to the cgroup
- Child cgroups exist

**Diagnostic**:
```bash
# Check for processes
cat /sys/fs/cgroup/your-cgroup/cgroup.procs

# Check for child cgroups
ls /sys/fs/cgroup/your-cgroup/
```

**Fix**:
```bash
# Move processes to root cgroup
for pid in $(cat /sys/fs/cgroup/your-cgroup/cgroup.procs); do
    echo $pid | sudo tee /sys/fs/cgroup/cgroup.procs
done

# Remove children first (deepest first)
sudo rmdir /sys/fs/cgroup/your-cgroup/child
sudo rmdir /sys/fs/cgroup/your-cgroup
```

**Reference**: `docs/02-cgroups/01-cgv2-basics.md`, `docs/02-cgroups/02-memory.md`

---

### ENOENT - Cgroup or Controller File Not Found

**Error**:
```
Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

**Cause**:
- Cgroup doesn't exist
- Controller not enabled for this cgroup
- Wrong path (missing `/sys/fs/cgroup/` prefix or extra prefix)

**Diagnostic**:
```bash
# Check if cgroup exists
ls /sys/fs/cgroup/your-cgroup/

# Check enabled controllers
cat /sys/fs/cgroup/your-cgroup/cgroup.controllers

# Check if specific file exists
ls /sys/fs/cgroup/your-cgroup/memory.max
```

**Fix**:
- Create the cgroup first: `sudo mkdir /sys/fs/cgroup/your-cgroup`
- Enable the controller: `echo "+memory" | sudo tee /sys/fs/cgroup/cgroup.subtree_control`

**Reference**: `docs/02-cgroups/01-cgv2-basics.md`

---

### ESRCH - No Such Process

**Error**:
```
Os { code: 3, kind: NotFound, message: "No such process" }
```

**Cause**: Writing a PID to `cgroup.procs` that doesn't exist.

**Diagnostic**:
```bash
# Verify process exists
ps -p <PID>
```

**Fix**: Use a valid, running process PID:
```bash
echo $$ | sudo tee /sys/fs/cgroup/your-cgroup/cgroup.procs
```

**Reference**: `docs/02-cgroups/01-cgv2-basics.md`

---

### Controller Not Available

**Error**: Controller files missing (e.g., no `memory.max`, no `cpu.max`).

**Cause**: Controller not enabled in parent's `cgroup.subtree_control`.

**Diagnostic**:
```bash
# Check what controllers are enabled for children
cat /sys/fs/cgroup/cgroup.subtree_control

# Check what controllers this cgroup can use
cat /sys/fs/cgroup/your-cgroup/cgroup.controllers
```

**Fix**: Enable controllers at parent level:
```bash
echo "+cpu +memory +io +pids" | sudo tee /sys/fs/cgroup/cgroup.subtree_control
```

**Reference**: `docs/02-cgroups/01-cgv2-basics.md`

---

### Memory Limit Appears Different

**Issue**: Set `memory.max` to X but reading back shows slightly different value.

**Cause**: Kernel rounds memory values to page size boundaries (typically 4096 bytes).

**Diagnostic**:
```bash
getconf PAGE_SIZE
# Usually 4096
```

**Fix**: This is normal behavior. For exact comparisons, round your expected value to page size.

**Reference**: `docs/02-cgroups/02-memory.md`

---

### CPU Quota Not Taking Effect

**Issue**: Set `cpu.max` but process still uses 100% CPU.

**Cause**:
- Format incorrect (should be "quota period", e.g., "50000 100000" for 50%)
- Process not in the cgroup
- Measurement timing issues

**Diagnostic**:
```bash
# Verify the setting
cat /sys/fs/cgroup/your-cgroup/cpu.max

# Verify process is in cgroup
cat /sys/fs/cgroup/your-cgroup/cgroup.procs

# Check CPU stats
cat /sys/fs/cgroup/your-cgroup/cpu.stat
```

**Fix**: Ensure correct format and process attachment:
```bash
# Set 50% CPU limit (50ms quota per 100ms period)
echo "50000 100000" | sudo tee /sys/fs/cgroup/your-cgroup/cpu.max

# Add process to cgroup
echo $$ | sudo tee /sys/fs/cgroup/your-cgroup/cgroup.procs
```

**Reference**: `docs/02-cgroups/03-cpu.md`

---

### I/O Throttle Not Working

**Issue**: Set `io.max` but I/O speed not limited.

**Cause**:
- Wrong device major:minor numbers
- I/O is buffered (page cache), not direct
- Device doesn't support throttling

**Diagnostic**:
```bash
# Find correct device numbers
lsblk -o NAME,MAJ:MIN

# Check current settings
cat /sys/fs/cgroup/your-cgroup/io.max

# Check I/O stats
cat /sys/fs/cgroup/your-cgroup/io.stat
```

**Fix**: Use correct device numbers and consider using direct I/O for testing:
```bash
# Get device number (e.g., 8:0 for /dev/sda)
echo "8:0 rbps=1048576" | sudo tee /sys/fs/cgroup/your-cgroup/io.max

# Use direct I/O for testing (bypasses page cache)
dd if=/dev/zero of=testfile bs=1M count=100 oflag=direct
```

**Reference**: `docs/02-cgroups/04-io.md`

---

## runc/OCI Errors

### Bundle Directory Already Exists

**Error**:
```
Error: bundle directory already exists: /path/to/bundle
```

**Cause**: Attempting to create a bundle at a path that already exists.

**Fix**: Remove existing bundle or use a different path:
```bash
rm -rf /path/to/bundle
# or
cargo run -p oci-tool -- create /path/to/new-bundle
```

**Reference**: `docs/03-runc/01-oci-bundle.md`

---

### ociVersion Field Missing or Invalid

**Error**:
```
runc: error loading container config: missing ociVersion
```

**Cause**: The `config.json` doesn't have a valid `ociVersion` field.

**Diagnostic**:
```bash
cat bundle/config.json | jq '.ociVersion'
```

**Fix**: Ensure config.json has the required field:
```json
{
    "ociVersion": "1.0.2",
    ...
}
```

**Reference**: `docs/03-runc/01-oci-bundle.md`, `docs/03-runc/02-config-json.md`

---

### serde_json Import Issues

**Error**:
```
error[E0432]: unresolved import `serde_json`
```

**Cause**: `serde_json` crate not added to dependencies.

**Fix**: Add to `Cargo.toml`:
```toml
[dependencies]
serde_json = "1"
serde = { version = "1", features = ["derive"] }
```

**Reference**: `docs/03-runc/01-oci-bundle.md`

---

### runc create Fails with Permission Denied

**Error**:
```
ERRO[0000] container_linux.go: permission denied
```

**Cause**: runc needs root privileges or rootless mode configuration.

**Fix**: Run with sudo:
```bash
sudo runc run -b /path/to/bundle container-id
```

Or configure rootless mode properly.

**Reference**: `docs/03-runc/03-run-basic.md`

---

### Container Rootfs Not Found

**Error**:
```
container rootfs "/path/bundle/rootfs" does not exist
```

**Cause**: The `rootfs` directory doesn't exist or `config.json` points to wrong path.

**Diagnostic**:
```bash
ls -la /path/to/bundle/
cat /path/to/bundle/config.json | jq '.root.path'
```

**Fix**: Create rootfs and populate it:
```bash
mkdir -p /path/to/bundle/rootfs
# Copy minimal filesystem or use alpine rootfs
```

**Reference**: `docs/03-runc/01-oci-bundle.md`

---

### Seccomp Filter Blocks System Call

**Error**:
Process killed by seccomp or operation fails unexpectedly.

**Cause**: The OCI config's seccomp profile blocks a required syscall.

**Diagnostic**:
```bash
# Check which syscall was blocked (if audit is enabled)
dmesg | grep audit | tail -5

# Review seccomp config
cat config.json | jq '.linux.seccomp'
```

**Fix**: Add the required syscall to the allow list in `config.json`:
```json
{
    "linux": {
        "seccomp": {
            "defaultAction": "SCMP_ACT_ERRNO",
            "syscalls": [
                {
                    "names": ["read", "write", "your_syscall_here"],
                    "action": "SCMP_ACT_ALLOW"
                }
            ]
        }
    }
}
```

**Reference**: `docs/03-runc/05-seccomp.md`

---

### Container Lifecycle State Mismatch

**Error**:
```
container "xyz" does not exist
```
or
```
cannot start a container that is not created
```

**Cause**: Attempting lifecycle operations in wrong order or on non-existent container.

**Diagnostic**:
```bash
sudo runc list
sudo runc state container-id
```

**Fix**: Follow correct lifecycle order:
1. `runc create` - creates container (stopped state)
2. `runc start` - starts the container
3. `runc delete` - removes the container

**Reference**: `docs/03-runc/04-lifecycle.md`

---

## Test-Specific Issues

### Tests Pass But Behavior Wrong

**Issue**: `cargo test` shows green, but manual testing shows incorrect behavior.

**Cause**: Tests run without `sudo` and skip privileged operations.

**Diagnostic**: Look for skip messages in test output:
```
Skipping test_xyz: requires root privileges
```

**Fix**: Run tests with sudo:
```bash
sudo -E cargo test -p ns-tool
sudo -E cargo test -p cgroup-tool
```

The `-E` preserves environment variables like `PATH` and `CARGO_HOME`.

**Reference**: All lessons in `docs/01-namespaces/` and `docs/02-cgroups/`

---

### Test Cleanup Signal Messages

**Issue**: Tests print signal-related messages during cleanup.

**Example**:
```
Child exited with signal: SIGTERM
```

**Cause**: Test cleanup sends signals to child processes. This is normal behavior.

**Fix**: No fix needed - this is expected. The test harness ensures processes don't outlive tests.

**Reference**: `docs/01-namespaces/10-join-existing.md`

---

### Orphaned Resources After Test Failure

**Issue**: Failed tests leave cgroups, network namespaces, or mount points.

**Fix**: Manual cleanup:
```bash
# Find and remove test cgroups
for cg in $(ls /sys/fs/cgroup/ | grep "^rust-test-"); do
    # Move any processes first
    for pid in $(cat /sys/fs/cgroup/$cg/cgroup.procs 2>/dev/null); do
        echo $pid | sudo tee /sys/fs/cgroup/cgroup.procs
    done
    sudo rmdir /sys/fs/cgroup/$cg
done

# Remove test network namespaces
for ns in $(ip netns list | grep "^test-" | awk '{print $1}'); do
    sudo ip netns delete $ns
done
```

**Reference**: DevContainer Gotchas section above

---

## Notes

This troubleshooting guide consolidates common errors from all lessons in this learning path. If you encounter an error not listed here:

1. Check the "Common Errors" section in the specific lesson you're working on
2. Search the kernel documentation for the error code
3. Use `strace` to trace system calls and identify where failures occur:
   ```bash
   sudo strace -f cargo run -p ns-tool -- create-ns
   ```

**Useful Resources**:
- Kernel cgroup v2 docs: https://docs.kernel.org/admin-guide/cgroup-v2.html
- Kernel namespaces: https://man7.org/linux/man-pages/man7/namespaces.7.html
- OCI Runtime Spec: https://github.com/opencontainers/runtime-spec
- runc documentation: https://github.com/opencontainers/runc

---

*This is a reference document - not a lesson with tests or implementation tasks.*
