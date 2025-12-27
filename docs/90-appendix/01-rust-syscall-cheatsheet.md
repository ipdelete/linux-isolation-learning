# Rust Syscall Cheatsheet

Quick reference for Linux system calls used throughout this tutorial series. This is a reference document, not a lesson with tests/implementation.

## Table of Contents

- [Overview: nix vs libc vs std](#overview-nix-vs-libc-vs-std)
- [Namespace Syscalls](#namespace-syscalls)
  - [unshare](#unshare)
  - [clone](#clone)
  - [setns](#setns)
  - [pivot_root](#pivot_root)
  - [mount](#mount)
  - [umount](#umount)
- [Process Syscalls](#process-syscalls)
  - [fork](#fork)
  - [execve](#execve)
  - [wait/waitpid](#waitwaitpid)
  - [getpid/getppid](#getpidgetppid)
  - [prctl](#prctl)
- [Cgroup Operations](#cgroup-operations)
  - [File-based cgroup v2 interface](#file-based-cgroup-v2-interface)
- [Network Syscalls](#network-syscalls)
  - [socket](#socket)
  - [Netlink operations](#netlink-operations)
- [Capability Syscalls](#capability-syscalls)
  - [capget/capset](#capgetcapset)
  - [prctl for capabilities](#prctl-for-capabilities)
- [File Descriptor Operations](#file-descriptor-operations)
  - [open/close](#openclose)
  - [dup2](#dup2)
  - [pipe](#pipe)
- [Using unsafe Safely](#using-unsafe-safely)
- [Quick Reference Table](#quick-reference-table)

---

## Overview: nix vs libc vs std

This tutorial uses three approaches for syscalls, in order of preference:

| Crate | When to Use | Safety |
|-------|-------------|--------|
| `nix` | Default choice. Type-safe wrappers for POSIX/Linux APIs | Safe (mostly) |
| `std` | File I/O, directories, basic process operations | Safe |
| `libc` | When `nix` lacks coverage or you need raw control | Requires `unsafe` |

**Decision tree:**

```
Need kernel interaction?
    |
    +-- No: Use std (files, threads, networking)
    |
    +-- Yes: Is it in nix?
            |
            +-- Yes: Use nix (namespaces, signals, mount)
            |
            +-- No: Use libc with unsafe {}
```

**Cargo.toml dependencies:**

```toml
[dependencies]
nix = { version = "0.27", features = ["sched", "mount", "signal", "process"] }
libc = "0.2"
```

---

## Namespace Syscalls

### unshare

**Purpose**: Create new namespaces for the calling process.

**Covered in**: `docs/01-namespaces/01-pid-namespace.md`, `docs/01-namespaces/02-unshare-vs-clone.md`

**Rust crate**: `nix::sched::unshare`

```rust
use nix::sched::{unshare, CloneFlags};

// Create a new PID namespace (requires CAP_SYS_ADMIN or root)
unshare(CloneFlags::CLONE_NEWPID)?;

// Create multiple namespaces at once
unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUTS)?;
```

**Common flags:**

| Flag | Namespace | Description |
|------|-----------|-------------|
| `CLONE_NEWPID` | PID | New process ID namespace |
| `CLONE_NEWUTS` | UTS | New hostname/domainname namespace |
| `CLONE_NEWIPC` | IPC | New IPC namespace (semaphores, message queues) |
| `CLONE_NEWNS` | Mount | New mount namespace |
| `CLONE_NEWNET` | Network | New network namespace |
| `CLONE_NEWUSER` | User | New user namespace (unprivileged) |
| `CLONE_NEWCGROUP` | Cgroup | New cgroup namespace |
| `CLONE_NEWTIME` | Time | New time namespace (kernel 5.6+) |

**Important notes:**
- `CLONE_NEWPID` affects only child processes, not the calling process
- Always fork after `unshare(CLONE_NEWPID)` to enter the new PID namespace
- `CLONE_NEWUSER` can be used without root privileges

**Man page**: [unshare(2)](https://man7.org/linux/man-pages/man2/unshare.2.html)

---

### clone

**Purpose**: Create a new process with fine-grained control over shared resources.

**Covered in**: `docs/01-namespaces/02-unshare-vs-clone.md`

**Rust crate**: `nix::sched::clone` or `libc::clone`

The `nix` crate provides `clone`, but for the newer `clone3` syscall, you may need `libc`:

```rust
use nix::sched::{clone, CloneFlags};
use nix::sys::signal::Signal;

// Using nix::sched::clone (older interface)
let mut stack = vec![0u8; 1024 * 1024]; // 1MB stack

let cb = Box::new(|| {
    println!("Child process running!");
    0  // Exit code
});

let pid = clone(
    cb,
    &mut stack,
    CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWUTS,
    Some(Signal::SIGCHLD as i32),
)?;
```

**When to use clone vs unshare+fork:**

| Approach | Use When |
|----------|----------|
| `unshare` + `fork` | Simpler; good for most cases |
| `clone` | Need custom stack, specific signal on exit, or CLONE_VM |

**Man page**: [clone(2)](https://man7.org/linux/man-pages/man2/clone.2.html)

---

### setns

**Purpose**: Join an existing namespace.

**Covered in**: `docs/01-namespaces/10-join-existing.md`

**Rust crate**: `nix::sched::setns`

```rust
use nix::sched::{setns, CloneFlags};
use std::fs::File;
use std::os::unix::io::AsRawFd;

// Open the namespace file
let ns_file = File::open("/proc/1234/ns/net")?;

// Join the namespace
setns(ns_file.as_raw_fd(), CloneFlags::CLONE_NEWNET)?;

// Now this process shares the network namespace with PID 1234
```

**Joining multiple namespaces:**

```rust
use nix::sched::{setns, CloneFlags};
use std::fs::File;
use std::os::unix::io::AsRawFd;

let pid = 1234;

// Join each namespace type separately
for (ns_type, flag) in [
    ("pid", CloneFlags::CLONE_NEWPID),
    ("net", CloneFlags::CLONE_NEWNET),
    ("mnt", CloneFlags::CLONE_NEWNS),
    ("uts", CloneFlags::CLONE_NEWUTS),
    ("ipc", CloneFlags::CLONE_NEWIPC),
] {
    let path = format!("/proc/{}/ns/{}", pid, ns_type);
    let file = File::open(&path)?;
    setns(file.as_raw_fd(), flag)?;
}
```

**Man page**: [setns(2)](https://man7.org/linux/man-pages/man2/setns.2.html)

---

### pivot_root

**Purpose**: Change the root filesystem for the calling process.

**Covered in**: `docs/01-namespaces/04-mount-namespace.md`, `docs/01-namespaces/05-minimal-rootfs.md`

**Rust crate**: `nix::unistd::pivot_root`

```rust
use nix::unistd::pivot_root;
use std::os::unix::fs;

// Assuming we're in a mount namespace with a new root at /new_root

// Create the put_old directory inside new_root
std::fs::create_dir_all("/new_root/.pivot_old")?;

// Pivot the root filesystem
pivot_root("/new_root", "/new_root/.pivot_old")?;

// Change to new root directory
std::env::set_current_dir("/")?;

// Unmount the old root
nix::mount::umount2("/.pivot_old", nix::mount::MntFlags::MNT_DETACH)?;

// Remove the mount point
std::fs::remove_dir("/.pivot_old")?;
```

**Requirements:**
- Must be in a mount namespace (cannot pivot root in the initial namespace)
- Both `new_root` and `put_old` must be directories
- `new_root` must be a mount point (use `mount --bind /new_root /new_root` if needed)
- `put_old` must be under `new_root`

**Man page**: [pivot_root(2)](https://man7.org/linux/man-pages/man2/pivot_root.2.html)

---

### mount

**Purpose**: Mount a filesystem.

**Covered in**: `docs/01-namespaces/04-mount-namespace.md`

**Rust crate**: `nix::mount::mount`

```rust
use nix::mount::{mount, MsFlags};

// Mount proc filesystem
mount(
    Some("proc"),                    // source
    "/proc",                         // target
    Some("proc"),                    // fstype
    MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
    None::<&str>,                    // data (mount options)
)?;

// Bind mount
mount(
    Some("/source"),
    "/destination",
    None::<&str>,                    // No fstype for bind mounts
    MsFlags::MS_BIND,
    None::<&str>,
)?;

// Remount as read-only
mount(
    None::<&str>,
    "/destination",
    None::<&str>,
    MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY | MsFlags::MS_BIND,
    None::<&str>,
)?;

// Mount tmpfs
mount(
    Some("tmpfs"),
    "/tmp",
    Some("tmpfs"),
    MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
    Some("size=64M,mode=1777"),
)?;
```

**Common flags:**

| Flag | Description |
|------|-------------|
| `MS_BIND` | Create a bind mount |
| `MS_RDONLY` | Mount read-only |
| `MS_NOSUID` | Ignore setuid/setgid bits |
| `MS_NOEXEC` | Disallow program execution |
| `MS_NODEV` | Disallow device access |
| `MS_REMOUNT` | Remount with new flags |
| `MS_PRIVATE` | Make mount private (no propagation) |
| `MS_SLAVE` | Make mount a slave (receive propagation) |
| `MS_REC` | Apply recursively |

**Man page**: [mount(2)](https://man7.org/linux/man-pages/man2/mount.2.html)

---

### umount

**Purpose**: Unmount a filesystem.

**Covered in**: `docs/01-namespaces/04-mount-namespace.md`

**Rust crate**: `nix::mount::umount` or `nix::mount::umount2`

```rust
use nix::mount::{umount, umount2, MntFlags};

// Simple unmount
umount("/mnt/point")?;

// Unmount with flags
umount2("/mnt/point", MntFlags::MNT_DETACH)?;  // Lazy unmount

// Force unmount (use with caution)
umount2("/mnt/point", MntFlags::MNT_FORCE)?;
```

**Flags:**

| Flag | Description |
|------|-------------|
| `MNT_DETACH` | Lazy unmount; detach now, cleanup when not busy |
| `MNT_FORCE` | Force unmount (may cause data loss) |
| `MNT_EXPIRE` | Mark for expiration (for autofs) |

**Man page**: [umount(2)](https://man7.org/linux/man-pages/man2/umount.2.html)

---

## Process Syscalls

### fork

**Purpose**: Create a child process as a copy of the parent.

**Covered in**: `docs/01-namespaces/01-pid-namespace.md`

**Rust crate**: `nix::unistd::fork`

```rust
use nix::unistd::{fork, ForkResult};

// fork() is unsafe because it's dangerous in multi-threaded programs
match unsafe { fork() }? {
    ForkResult::Parent { child } => {
        println!("Parent process, child PID: {}", child);
        // Wait for child to prevent zombie
    }
    ForkResult::Child => {
        println!("Child process");
        // Do child work
        std::process::exit(0);
    }
}
```

**Why fork is unsafe:**
- In multi-threaded programs, only the calling thread exists in the child
- Locks held by other threads become permanently locked
- File descriptors may be in inconsistent states
- Safe in single-threaded programs (like most of our examples)

**Man page**: [fork(2)](https://man7.org/linux/man-pages/man2/fork.2.html)

---

### execve

**Purpose**: Replace current process with a new program.

**Covered in**: `docs/03-runc/03-run-basic.md`

**Rust crate**: `nix::unistd::execve` or `nix::unistd::execvp`

```rust
use nix::unistd::execvp;
use std::ffi::CString;

// execvp searches PATH for the program
let program = CString::new("ls")?;
let args = [
    CString::new("ls")?,
    CString::new("-la")?,
];

// This replaces the current process - never returns on success
execvp(&program, &args)?;

// If we reach here, exec failed
```

**Using execve with environment:**

```rust
use nix::unistd::execve;
use std::ffi::CString;

let program = CString::new("/bin/sh")?;
let args = [
    CString::new("sh")?,
    CString::new("-c")?,
    CString::new("echo $HOME")?,
];
let env = [
    CString::new("HOME=/container")?,
    CString::new("PATH=/bin:/usr/bin")?,
];

execve(&program, &args, &env)?;
```

**Variants:**

| Function | Description |
|----------|-------------|
| `execve` | Full path, explicit environment |
| `execvp` | Search PATH, inherit environment |
| `execv` | Full path, inherit environment |

**Man page**: [execve(2)](https://man7.org/linux/man-pages/man2/execve.2.html)

---

### wait/waitpid

**Purpose**: Wait for child process to change state.

**Covered in**: `docs/01-namespaces/01-pid-namespace.md`

**Rust crate**: `nix::sys::wait::waitpid`

```rust
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
use nix::unistd::Pid;

// Wait for specific child
let status = waitpid(child_pid, None)?;

match status {
    WaitStatus::Exited(pid, exit_code) => {
        println!("Child {} exited with code {}", pid, exit_code);
    }
    WaitStatus::Signaled(pid, signal, _) => {
        println!("Child {} killed by signal {:?}", pid, signal);
    }
    WaitStatus::Stopped(pid, signal) => {
        println!("Child {} stopped by signal {:?}", pid, signal);
    }
    _ => {}
}

// Non-blocking wait
match waitpid(child_pid, Some(WaitPidFlag::WNOHANG))? {
    WaitStatus::StillAlive => {
        println!("Child still running");
    }
    status => {
        println!("Child finished: {:?}", status);
    }
}

// Wait for any child
let status = waitpid(Pid::from_raw(-1), None)?;
```

**Man page**: [waitpid(2)](https://man7.org/linux/man-pages/man2/waitpid.2.html)

---

### getpid/getppid

**Purpose**: Get process ID of current process or parent.

**Covered in**: `docs/01-namespaces/01-pid-namespace.md`

**Rust crate**: `nix::unistd::{getpid, getppid}`

```rust
use nix::unistd::{getpid, getppid};

let my_pid = getpid();
let parent_pid = getppid();

println!("My PID: {}, Parent PID: {}", my_pid, parent_pid);

// In a PID namespace, the first process has PID 1
// and parent PID 0 (parent is in different namespace)
```

**Man page**: [getpid(2)](https://man7.org/linux/man-pages/man2/getpid.2.html)

---

### prctl

**Purpose**: Process control operations.

**Covered in**: `docs/03-runc/05-seccomp.md`

**Rust crate**: `nix::sys::prctl` (limited coverage) or `libc::prctl`

```rust
use libc::{prctl, PR_SET_NO_NEW_PRIVS, PR_SET_NAME, PR_GET_NAME};
use std::ffi::CString;

// Prevent gaining new privileges (required before seccomp)
unsafe {
    let ret = prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
    if ret != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
}

// Set process name (visible in ps, top)
let name = CString::new("container")?;
unsafe {
    prctl(PR_SET_NAME, name.as_ptr() as libc::c_ulong, 0, 0, 0);
}
```

**Common prctl operations:**

| Operation | Description |
|-----------|-------------|
| `PR_SET_NO_NEW_PRIVS` | Prevent privilege escalation via exec |
| `PR_SET_NAME` | Set process name |
| `PR_SET_PDEATHSIG` | Signal to send when parent dies |
| `PR_SET_SECCOMP` | Install seccomp filter |
| `PR_CAPBSET_DROP` | Drop capability from bounding set |

**Man page**: [prctl(2)](https://man7.org/linux/man-pages/man2/prctl.2.html)

---

## Cgroup Operations

### File-based cgroup v2 interface

**Purpose**: Control resource limits for groups of processes.

**Covered in**: `docs/02-cgroups/01-cgv2-basics.md` through `docs/02-cgroups/06-multi-resource.md`

**Rust crate**: `std::fs` (cgroups use a filesystem interface)

Cgroups v2 does not use syscalls directly. Instead, you interact through the filesystem at `/sys/fs/cgroup`.

**Creating a cgroup:**

```rust
use std::fs;

let cgroup_path = "/sys/fs/cgroup/my-container";
fs::create_dir(cgroup_path)?;

// Verify it was created
assert!(fs::metadata(format!("{}/cgroup.procs", cgroup_path)).is_ok());
```

**Attaching a process:**

```rust
use std::fs;
use std::io::Write;

// Write PID to cgroup.procs to move a process
let mut file = fs::OpenOptions::new()
    .write(true)
    .open("/sys/fs/cgroup/my-container/cgroup.procs")?;
writeln!(file, "{}", pid)?;

// Verify
let procs = fs::read_to_string("/sys/fs/cgroup/my-container/cgroup.procs")?;
assert!(procs.contains(&pid.to_string()));
```

**Setting memory limit:**

```rust
use std::fs;

// Set 100MB memory limit
fs::write(
    "/sys/fs/cgroup/my-container/memory.max",
    "104857600"  // 100 * 1024 * 1024
)?;

// Verify
let limit = fs::read_to_string("/sys/fs/cgroup/my-container/memory.max")?;
println!("Memory limit: {}", limit.trim());
```

**Setting CPU limit:**

```rust
use std::fs;

// 50% of one CPU (50000 microseconds per 100000 microsecond period)
fs::write(
    "/sys/fs/cgroup/my-container/cpu.max",
    "50000 100000"
)?;
```

**Setting PIDs limit:**

```rust
use std::fs;

// Maximum 100 processes
fs::write(
    "/sys/fs/cgroup/my-container/pids.max",
    "100"
)?;
```

**Setting I/O limit:**

```rust
use std::fs;

// Limit device 8:0 (typically /dev/sda) to 1MB/s read/write
fs::write(
    "/sys/fs/cgroup/my-container/io.max",
    "8:0 rbps=1048576 wbps=1048576"
)?;
```

**Deleting a cgroup:**

```rust
use std::fs;

// Cgroup must be empty (no processes, no child cgroups)
fs::remove_dir("/sys/fs/cgroup/my-container")?;
```

**Important cgroup files:**

| File | Description | Read/Write |
|------|-------------|------------|
| `cgroup.procs` | PIDs in this cgroup | RW |
| `cgroup.controllers` | Available controllers | R |
| `cgroup.subtree_control` | Controllers for children | RW |
| `memory.max` | Memory limit in bytes | RW |
| `memory.current` | Current memory usage | R |
| `cpu.max` | CPU quota (microseconds) | RW |
| `pids.max` | Max number of processes | RW |
| `io.max` | I/O bandwidth limits | RW |

**Documentation**: [cgroups(7)](https://man7.org/linux/man-pages/man7/cgroups.7.html), [Kernel cgroup-v2 docs](https://docs.kernel.org/admin-guide/cgroup-v2.html)

---

## Network Syscalls

### socket

**Purpose**: Create a network socket.

**Covered in**: `docs/01-namespaces/06-netns-basics.md`

**Rust crate**: `nix::sys::socket::socket` or `std::net`

```rust
use nix::sys::socket::{socket, AddressFamily, SockType, SockFlag};

// Create a netlink socket for network configuration
let sock = socket(
    AddressFamily::Netlink,
    SockType::Raw,
    SockFlag::SOCK_CLOEXEC,
    None,  // Protocol (NETLINK_ROUTE = 0)
)?;

// Create a TCP socket
let tcp_sock = socket(
    AddressFamily::Inet,
    SockType::Stream,
    SockFlag::SOCK_CLOEXEC,
    None,
)?;
```

**Man page**: [socket(2)](https://man7.org/linux/man-pages/man2/socket.2.html)

---

### Netlink operations

**Purpose**: Configure network interfaces, routes, and addresses.

**Covered in**: `docs/01-namespaces/07-veth-bridge.md`, `docs/01-namespaces/08-netns-nat.md`

**Rust crate**: `rtnetlink` (async) or `netlink-packet-route` (low-level)

For network namespace configuration, we recommend the `rtnetlink` crate which provides a high-level async interface:

```rust
use futures::stream::TryStreamExt;
use rtnetlink::new_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    // List all network interfaces
    let mut links = handle.link().get().execute();
    while let Some(link) = links.try_next().await? {
        println!("Interface: {:?}", link);
    }

    // Create a veth pair
    handle
        .link()
        .add()
        .veth("veth0".into(), "veth1".into())
        .execute()
        .await?;

    // Set interface up
    let mut links = handle.link().get().match_name("veth0".into()).execute();
    if let Some(link) = links.try_next().await? {
        handle
            .link()
            .set(link.header.index)
            .up()
            .execute()
            .await?;
    }

    Ok(())
}
```

**Alternative: Using ip command via std::process:**

For simpler cases, shelling out to `ip` may be more practical:

```rust
use std::process::Command;

// Create veth pair
Command::new("ip")
    .args(["link", "add", "veth0", "type", "veth", "peer", "name", "veth1"])
    .status()?;

// Move veth1 to namespace
Command::new("ip")
    .args(["link", "set", "veth1", "netns", "my-namespace"])
    .status()?;

// Add IP address
Command::new("ip")
    .args(["addr", "add", "10.0.0.1/24", "dev", "veth0"])
    .status()?;

// Bring interface up
Command::new("ip")
    .args(["link", "set", "veth0", "up"])
    .status()?;
```

**Man page**: [netlink(7)](https://man7.org/linux/man-pages/man7/netlink.7.html), [rtnetlink(7)](https://man7.org/linux/man-pages/man7/rtnetlink.7.html)

---

## Capability Syscalls

### capget/capset

**Purpose**: Get or set process capabilities.

**Covered in**: `docs/03-runc/05-seccomp.md`

**Rust crate**: `caps` or `libc`

Using the `caps` crate (recommended):

```rust
use caps::{CapSet, Capability, has_cap, drop, raise};

// Check if we have a capability
if has_cap(None, CapSet::Effective, Capability::CAP_SYS_ADMIN)? {
    println!("We have CAP_SYS_ADMIN");
}

// Drop a capability
drop(None, CapSet::Effective, Capability::CAP_SYS_ADMIN)?;

// Raise a capability (if in permitted set)
raise(None, CapSet::Effective, Capability::CAP_NET_BIND_SERVICE)?;
```

Using `libc` directly:

```rust
use libc::{__user_cap_data_struct, __user_cap_header_struct, capget, capset};
use libc::{_LINUX_CAPABILITY_VERSION_3, CAP_SYS_ADMIN};

// Get capabilities
let mut header = __user_cap_header_struct {
    version: _LINUX_CAPABILITY_VERSION_3,
    pid: 0,  // 0 = current process
};
let mut data = [__user_cap_data_struct {
    effective: 0,
    permitted: 0,
    inheritable: 0,
}; 2];

unsafe {
    if capget(&mut header, data.as_mut_ptr()) != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
}

println!("Effective capabilities: {:032b}", data[0].effective);
```

**Common capabilities:**

| Capability | Description |
|------------|-------------|
| `CAP_SYS_ADMIN` | Broad admin privileges, namespace creation |
| `CAP_NET_ADMIN` | Network configuration |
| `CAP_NET_BIND_SERVICE` | Bind to ports < 1024 |
| `CAP_NET_RAW` | Raw sockets |
| `CAP_SYS_PTRACE` | Trace/debug processes |
| `CAP_SETUID/SETGID` | Change UID/GID |
| `CAP_CHOWN` | Change file ownership |
| `CAP_DAC_OVERRIDE` | Bypass file permissions |

**Man page**: [capabilities(7)](https://man7.org/linux/man-pages/man7/capabilities.7.html)

---

### prctl for capabilities

**Purpose**: Manipulate the capability bounding set.

```rust
use libc::{prctl, PR_CAPBSET_DROP, PR_CAPBSET_READ};

// Check if capability is in bounding set
let cap_sys_admin = 21;  // CAP_SYS_ADMIN
unsafe {
    let result = prctl(PR_CAPBSET_READ, cap_sys_admin, 0, 0, 0);
    if result == 1 {
        println!("CAP_SYS_ADMIN is in bounding set");
    }
}

// Drop capability from bounding set (cannot be re-added)
unsafe {
    let result = prctl(PR_CAPBSET_DROP, cap_sys_admin, 0, 0, 0);
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
}
```

---

## File Descriptor Operations

### open/close

**Purpose**: Open and close files.

**Rust crate**: `nix::fcntl::open` or `std::fs::File`

```rust
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::close;

// Open with nix (gives raw fd)
let fd = open(
    "/proc/self/ns/pid",
    OFlag::O_RDONLY | OFlag::O_CLOEXEC,
    Mode::empty(),
)?;

// Close when done
close(fd)?;

// Using std (preferred for most cases)
use std::fs::File;
let file = File::open("/proc/self/ns/pid")?;
// Automatically closed when dropped
```

**Man page**: [open(2)](https://man7.org/linux/man-pages/man2/open.2.html)

---

### dup2

**Purpose**: Duplicate a file descriptor to a specific number.

**Covered in**: `docs/03-runc/03-run-basic.md`

**Rust crate**: `nix::unistd::dup2`

```rust
use nix::unistd::dup2;
use std::os::unix::io::AsRawFd;
use std::fs::File;

// Redirect stdout to a file
let file = File::create("/tmp/output.log")?;
dup2(file.as_raw_fd(), 1)?;  // 1 = stdout

// Now println! writes to the file
println!("This goes to /tmp/output.log");
```

**Man page**: [dup2(2)](https://man7.org/linux/man-pages/man2/dup2.2.html)

---

### pipe

**Purpose**: Create a unidirectional communication channel.

**Rust crate**: `nix::unistd::pipe`

```rust
use nix::unistd::{pipe, read, write, close};

// Create pipe: returns (read_fd, write_fd)
let (read_fd, write_fd) = pipe()?;

// In parent: close read end, write to pipe
close(read_fd)?;
write(write_fd, b"Hello from parent")?;
close(write_fd)?;

// In child: close write end, read from pipe
close(write_fd)?;
let mut buf = [0u8; 64];
let n = read(read_fd, &mut buf)?;
println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
close(read_fd)?;
```

**Man page**: [pipe(2)](https://man7.org/linux/man-pages/man2/pipe.2.html)

---

## Using unsafe Safely

### When unsafe is Required

The following operations require `unsafe` in Rust:

| Operation | Why Unsafe |
|-----------|------------|
| `fork()` | Creates process with shared memory; dangerous in multithreaded programs |
| `libc::*` | Raw FFI calls bypass Rust's safety checks |
| Raw pointer dereference | Used in some low-level syscall wrappers |

### Minimizing unsafe Scope

**Bad** (too much in unsafe):

```rust
unsafe {
    let result = libc::unshare(libc::CLONE_NEWPID);
    if result == -1 {
        let errno = *libc::__errno_location();
        panic!("unshare failed with errno {}", errno);
    }
    // More code that doesn't need to be unsafe...
}
```

**Good** (minimal unsafe):

```rust
let result = unsafe { libc::unshare(libc::CLONE_NEWPID) };
if result == -1 {
    return Err(std::io::Error::last_os_error().into());
}
```

**Best** (use nix when possible):

```rust
use nix::sched::{unshare, CloneFlags};
unshare(CloneFlags::CLONE_NEWPID)?;  // No unsafe needed!
```

### Safe Wrapper Pattern

When you must use `libc`, create a safe wrapper:

```rust
use std::io::{Error, Result};

/// Set the process name visible in `ps` and `top`.
///
/// # Safety
/// This function wraps an unsafe prctl call but is safe because:
/// - PR_SET_NAME only reads up to 16 bytes from the pointer
/// - The CString ensures null-termination
/// - No memory is written by the kernel
pub fn set_process_name(name: &str) -> Result<()> {
    use libc::{prctl, PR_SET_NAME};
    use std::ffi::CString;

    let name = CString::new(name).map_err(|_| {
        Error::new(std::io::ErrorKind::InvalidInput, "name contains null byte")
    })?;

    let result = unsafe { prctl(PR_SET_NAME, name.as_ptr() as libc::c_ulong, 0, 0, 0) };

    if result == 0 {
        Ok(())
    } else {
        Err(Error::last_os_error())
    }
}
```

---

## Quick Reference Table

| Syscall | Crate | Function | Lesson |
|---------|-------|----------|--------|
| `unshare` | `nix` | `nix::sched::unshare` | `01-namespaces/01-pid-namespace.md` |
| `clone` | `nix` | `nix::sched::clone` | `01-namespaces/02-unshare-vs-clone.md` |
| `setns` | `nix` | `nix::sched::setns` | `01-namespaces/10-join-existing.md` |
| `pivot_root` | `nix` | `nix::unistd::pivot_root` | `01-namespaces/04-mount-namespace.md` |
| `mount` | `nix` | `nix::mount::mount` | `01-namespaces/04-mount-namespace.md` |
| `umount` | `nix` | `nix::mount::umount2` | `01-namespaces/04-mount-namespace.md` |
| `fork` | `nix` | `nix::unistd::fork` | `01-namespaces/01-pid-namespace.md` |
| `execve` | `nix` | `nix::unistd::execve` | `03-runc/03-run-basic.md` |
| `waitpid` | `nix` | `nix::sys::wait::waitpid` | `01-namespaces/01-pid-namespace.md` |
| `getpid` | `nix` | `nix::unistd::getpid` | `01-namespaces/01-pid-namespace.md` |
| `prctl` | `libc` | `libc::prctl` | `03-runc/05-seccomp.md` |
| `sethostname` | `nix` | `nix::unistd::sethostname` | `01-namespaces/03-uts-ipc.md` |
| cgroups | `std` | `std::fs::*` | `02-cgroups/*` |
| netlink | `rtnetlink` | various | `01-namespaces/07-veth-bridge.md` |
| capabilities | `caps` | `caps::*` | `03-runc/05-seccomp.md` |

---

## Additional Resources

### Man Pages

- [namespaces(7)](https://man7.org/linux/man-pages/man7/namespaces.7.html) - Overview of Linux namespaces
- [cgroups(7)](https://man7.org/linux/man-pages/man7/cgroups.7.html) - Control groups overview
- [capabilities(7)](https://man7.org/linux/man-pages/man7/capabilities.7.html) - Process capabilities
- [pid_namespaces(7)](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html) - PID namespace details
- [mount_namespaces(7)](https://man7.org/linux/man-pages/man7/mount_namespaces.7.html) - Mount namespace details
- [network_namespaces(7)](https://man7.org/linux/man-pages/man7/network_namespaces.7.html) - Network namespace details
- [user_namespaces(7)](https://man7.org/linux/man-pages/man7/user_namespaces.7.html) - User namespace details

### Crate Documentation

- [nix crate](https://docs.rs/nix/latest/nix/) - Safe Unix API bindings
- [libc crate](https://docs.rs/libc/latest/libc/) - Raw libc bindings
- [caps crate](https://docs.rs/caps/latest/caps/) - Linux capabilities
- [rtnetlink crate](https://docs.rs/rtnetlink/latest/rtnetlink/) - Netlink route socket

### Kernel Documentation

- [Cgroup v2](https://docs.kernel.org/admin-guide/cgroup-v2.html) - Authoritative cgroup v2 documentation
- [Namespaces](https://www.kernel.org/doc/html/latest/admin-guide/namespaces/compatibility-list.html) - Namespace compatibility
