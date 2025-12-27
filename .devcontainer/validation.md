# DevContainer Validation Guide

This document provides step-by-step validation that your devcontainer is properly configured for the Linux isolation learning tutorials.

## Important: Running as Root in DevContainer

**The devcontainer is configured to run as root** (`"remoteUser": "root"` in `.devcontainer/devcontainer.json`). This is intentional for this learning environment because:

- Almost every lesson requires privileged operations (namespaces, cgroups, network configuration)
- Running as root eliminates the need to prefix every command with `sudo`
- The container is isolated from your host system, so running as root inside is safe
- This reduces friction and lets you focus on learning the concepts

**In production systems**, you would NOT run as root. The lessons will note where `sudo` would be required in real-world usage.

If you prefer to run as a non-root user with sudo:
1. Remove or comment out `"remoteUser": "root"` from `.devcontainer/devcontainer.json`
2. Prefix all namespace/cgroup commands with `sudo -E` (the `-E` preserves environment variables for Cargo)

## Quick Validation

Run this one-liner to check all critical components:

```bash
echo "=== Quick Validation ===" && \
echo "Kernel: $(uname -r)" && \
echo "User: $(whoami) (UID: $(id -u))" && \
which cargo ip iptables ping busybox && \
echo "Namespaces: $(unshare --pid --fork echo 'OK' 2>&1)" && \
echo "Cgroups v2: $(mount | grep cgroup2 | wc -l) mount(s)" && \
echo "=== Validation Complete ==="
```

Expected output should show all tools found, "User: root (UID: 0)", and "Namespaces: OK".

---

## Detailed Validation

Follow these steps to thoroughly validate your environment.

### 1. System Information

Check kernel version and distribution:

```bash
echo "=== System Info ==="
uname -a
cat /etc/os-release | grep -E "^(NAME|VERSION)="
echo ""
```

**Expected**:
- Kernel version >= 5.x (modern namespace and cgroup features)
- Debian Trixie or similar

### 2. Privilege Level

Verify your user privileges:

```bash
echo "=== Privilege Check ==="
echo "Current user: $(whoami)"
echo "User ID: $(id -u)"
echo "Groups: $(groups)"
if [ $(id -u) -eq 0 ]; then
    echo "✓ Running as root - full privileges"
else
    echo "Running as non-root user"
    sudo -n true 2>&1 && echo "✓ Passwordless sudo available" || echo "✗ Sudo may require password"
fi
echo ""
```

**Expected** (in devcontainer):
- User: root
- UID: 0
- Message: "Running as root - full privileges"

If you're not running as root, ensure passwordless sudo works for commands like `cargo test`.

### 3. Required Packages

Verify all required tools are installed:

```bash
echo "=== Package Validation ==="

check_tool() {
    if command -v $1 &> /dev/null; then
        echo "✓ $1: $(command -v $1)"
    else
        echo "✗ $1: NOT FOUND"
    fi
}

# Core system tools
check_tool unshare
check_tool nsenter
check_tool lsns
check_tool findmnt
check_tool mount
check_tool strace

# Network tools
check_tool ip
check_tool iptables
check_tool ping
check_tool ss

# Special tools
check_tool busybox

# Development tools
check_tool cargo
check_tool rustc
check_tool git

echo ""
```

**Expected**: All tools should be found (✓).

### 4. Rust Toolchain

Validate Rust installation:

```bash
echo "=== Rust Toolchain ==="
rustc --version
cargo --version
echo ""

echo "Rust components:"
rustup component list --installed
echo ""
```

**Expected**:
- Rust 1.70+ (project uses Rust 2021 edition)
- cargo should be available
- rust-analyzer recommended

### 5. Namespace Support

Test each namespace type:

```bash
echo "=== Namespace Support ==="

test_ns() {
    local ns_type=$1
    local flag=$2
    if sudo unshare $flag /bin/true 2>/dev/null; then
        echo "✓ $ns_type namespace: supported"
    else
        echo "✗ $ns_type namespace: NOT SUPPORTED"
    fi
}

test_ns "PID" "--pid"
test_ns "UTS" "--uts"
test_ns "IPC" "--ipc"
test_ns "Mount" "--mount"
test_ns "Network" "--net"
test_ns "User" "--user"

echo ""
```

**Expected**: All namespace types should be supported (✓).

### 6. Namespace Verification (Detailed)

Create a PID namespace and verify isolation:

```bash
echo "=== Namespace Isolation Test ==="

echo "Host namespace PID:"
echo "  - My PID: $$"
echo "  - Process count: $(ps aux | wc -l)"

echo ""
echo "Inside new PID namespace:"
unshare --pid --fork --mount-proc /bin/bash -c '
    echo "  - My PID: $$"
    echo "  - Process count: $(ps aux | wc -l)"
    echo "  - Namespace inode: $(readlink /proc/self/ns/pid)"
'

echo ""
echo "Host namespace inode: $(readlink /proc/self/ns/pid)"
echo ""
```

**Expected**:
- Inside namespace: PID should be 1
- Inside namespace: Process count should be much lower (usually 2-4)
- Namespace inodes should be different

**Note**: If you're not running as root, prefix `unshare` with `sudo`.

### 7. Cgroup v2 Support

Verify cgroup v2 is available:

```bash
echo "=== Cgroup v2 Validation ==="

# Detect if we need sudo
SUDO_CMD=""
[ $(id -u) -ne 0 ] && SUDO_CMD="sudo"

# Check if cgroup v2 is mounted
if mount | grep -q "cgroup2 on /sys/fs/cgroup type cgroup2"; then
    echo "✓ Cgroup v2 is mounted at /sys/fs/cgroup"
else
    echo "✗ Cgroup v2 NOT mounted (may use hybrid or v1)"
    mount | grep cgroup
fi

# Check available controllers
if [ -f /sys/fs/cgroup/cgroup.controllers ]; then
    echo ""
    echo "Available controllers:"
    cat /sys/fs/cgroup/cgroup.controllers
else
    echo "✗ Cannot read cgroup controllers"
fi

# Verify we can create a test cgroup
if $SUDO_CMD mkdir -p /sys/fs/cgroup/test-validation 2>/dev/null; then
    echo ""
    echo "✓ Can create cgroups"
    $SUDO_CMD rmdir /sys/fs/cgroup/test-validation
else
    echo "✗ Cannot create cgroups"
fi

echo ""
```

**Expected**:
- Cgroup v2 mounted at `/sys/fs/cgroup`
- Controllers should include: cpu, memory, io, pids
- Should be able to create test cgroups

### 8. Network Configuration

Verify network tools and capabilities:

```bash
echo "=== Network Tools Validation ==="

# Detect if we need sudo
SUDO_CMD=""
[ $(id -u) -ne 0 ] && SUDO_CMD="sudo"

# Check ip command
echo "IP command:"
ip -V

# Test network namespace creation
echo ""
echo "Network namespace test:"
if $SUDO_CMD ip netns add test-validation 2>/dev/null; then
    echo "✓ Can create network namespaces"

    # Test veth pair creation
    if $SUDO_CMD ip link add veth-test0 type veth peer name veth-test1 2>/dev/null; then
        echo "✓ Can create veth pairs"
        $SUDO_CMD ip link delete veth-test0
    else
        echo "✗ Cannot create veth pairs"
    fi

    $SUDO_CMD ip netns delete test-validation
else
    echo "✗ Cannot create network namespaces"
fi

# Check iptables
echo ""
echo "Iptables:"
$SUDO_CMD iptables --version

# Check if we can add rules (without actually adding)
if $SUDO_CMD iptables -t nat -L -n &>/dev/null; then
    echo "✓ Can access iptables NAT table"
else
    echo "✗ Cannot access iptables"
fi

echo ""
```

**Expected**:
- Can create network namespaces
- Can create veth pairs
- Can access iptables NAT table

### 9. BusyBox Validation

Verify BusyBox for rootfs lessons:

```bash
echo "=== BusyBox Validation ==="

if [ -f /bin/busybox ]; then
    echo "✓ BusyBox found: /bin/busybox"
    /bin/busybox | head -2
    echo ""
    echo "BusyBox applets (first 10):"
    /bin/busybox --list | head -10
elif [ -f /usr/bin/busybox ]; then
    echo "✓ BusyBox found: /usr/bin/busybox"
    /usr/bin/busybox | head -2
else
    echo "✗ BusyBox NOT FOUND"
fi

echo ""
```

**Expected**:
- BusyBox should be available
- Should be statically linked (check with: `ldd /bin/busybox`)

### 10. Project Build

Verify the project builds successfully:

```bash
echo "=== Project Build Test ==="

cd /workspaces/linux-isolation-learning || cd ~/linux-isolation-learning || {
    echo "✗ Cannot find project directory"
    exit 1
}

echo "Building workspace..."
if cargo build 2>&1 | tee /tmp/build.log; then
    echo ""
    echo "✓ Project builds successfully"
else
    echo ""
    echo "✗ Build failed - check /tmp/build.log"
    exit 1
fi

echo ""
echo "Available binaries:"
cargo build --message-format=json 2>/dev/null | \
    jq -r 'select(.target.kind[] == "bin") | .target.name' 2>/dev/null | \
    sort -u || \
    find target/debug -maxdepth 1 -type f -executable | grep -v '\.' | head -5

echo ""
```

**Expected**:
- Project should build without errors
- Binaries should be created in `target/debug/`

### 11. Run Sample Test

Execute a simple namespace test from the project:

```bash
echo "=== Sample Test Execution ==="

cd /workspaces/linux-isolation-learning || cd ~/linux-isolation-learning || exit 1

echo "Running a simple cargo check:"
cargo check -p ns-tool

echo ""
echo "Testing basic command structure (no namespaces):"
cargo run -p ns-tool -- --help

echo ""
```

**Expected**:
- `cargo check` should succeed
- Help output should show available subcommands

### 12. Kernel Feature Check

Verify specific kernel features required by lessons:

```bash
echo "=== Kernel Features ==="

check_kernel_feature() {
    local feature=$1
    local path=$2
    if [ -e "$path" ]; then
        echo "✓ $feature: available"
    else
        echo "✗ $feature: NOT AVAILABLE"
    fi
}

check_kernel_feature "Namespaces" "/proc/self/ns"
check_kernel_feature "PID namespace" "/proc/self/ns/pid"
check_kernel_feature "Network namespace" "/proc/self/ns/net"
check_kernel_feature "Mount namespace" "/proc/self/ns/mnt"
check_kernel_feature "Cgroup namespace" "/proc/self/ns/cgroup"
check_kernel_feature "Cgroup v2" "/sys/fs/cgroup/cgroup.controllers"

echo ""
```

**Expected**: All features should be available (✓).

---

## Complete Validation Script

Run everything at once:

```bash
#!/bin/bash
# Save this as: validate-devcontainer.sh
# Run with: bash validate-devcontainer.sh

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║   Linux Isolation DevContainer Validation Suite       ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Paste all the validation sections above here...
# Or source this file from the repo

echo "╔════════════════════════════════════════════════════════╗"
echo "║   Validation Complete!                                 ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo "If all checks passed (✓), you're ready to start the tutorials!"
echo "Begin with: docs/00-getting-started.md"
```

---

## Troubleshooting

### Common Issues

**Issue**: "Permission denied" when creating namespaces
- **Solution 1**: Ensure devcontainer has `--privileged` flag in runArgs
  - **Verify**: Check `.devcontainer/devcontainer.json` has `"runArgs": ["--privileged"]`
- **Solution 2**: Ensure you're running as root or have sudo access
  - **Verify**: Run `whoami` (should show "root") or `sudo -v` (should succeed without password)
  - **Fix**: Add `"remoteUser": "root"` to `.devcontainer/devcontainer.json` or ensure sudo is configured

**Issue**: Cgroup v2 not available
- **Solution**: Devcontainer needs cgroup v2 support from host
- **Verify**: On host, run: `docker info | grep -i cgroup`
- **Note**: Some older Docker versions use cgroup v1 by default

**Issue**: Cannot create network namespaces
- **Solution**: Container needs NET_ADMIN capability
- **Verify**: Privileged mode should grant this automatically

**Issue**: Build fails with missing dependencies
- **Solution**: Run `cargo clean && cargo build` to rebuild from scratch
- **Check**: Verify Rust version is 1.70+

**Issue**: iptables command not found
- **Solution**: Reinstall packages: `sudo apt-get install -y iptables`

**Issue**: BusyBox not statically linked
- **Solution**: Install static version: `sudo apt-get install -y busybox-static`

---

## Next Steps

Once all validations pass:

1. Read `docs/00-getting-started.md`
2. Complete the foundations section: `docs/00-foundations/`
3. Start with namespace tutorials: `docs/01-namespaces/01-pid-namespace.md`
4. Follow the TDD approach: write tests first, then implement

**Pro tip**: Keep this validation file handy. If something breaks during the tutorials, re-run these checks to isolate the issue.
