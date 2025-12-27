#!/bin/bash
# DevContainer Validation Script
# Run this inside the devcontainer to verify everything is set up correctly

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass=0
fail=0

check() {
    if eval "$2" &>/dev/null; then
        echo -e "${GREEN}✓${NC} $1"
        ((pass++))
    else
        echo -e "${RED}✗${NC} $1"
        ((fail++))
    fi
}

section() {
    echo ""
    echo -e "${YELLOW}=== $1 ===${NC}"
}

echo "╔════════════════════════════════════════════════════════╗"
echo "║   Linux Isolation DevContainer Validation             ║"
echo "╚════════════════════════════════════════════════════════╝"

# Determine if we need sudo
SUDO=""
if [ $(id -u) -ne 0 ]; then
    SUDO="sudo"
    echo -e "${YELLOW}Note: Running as non-root user, will use sudo for privileged operations${NC}"
    echo ""
fi

section "System Information"
echo "Kernel: $(uname -r)"
echo "OS: $(cat /etc/os-release | grep PRETTY_NAME | cut -d= -f2 | tr -d '\"')"
echo "User: $(whoami) (UID: $(id -u))"
if [ $(id -u) -eq 0 ]; then
    echo -e "${GREEN}Running as root - no sudo needed${NC}"
else
    echo -e "${YELLOW}Running as $(whoami) - sudo required for namespace operations${NC}"
fi

section "Required Tools"
check "unshare" "command -v unshare"
check "nsenter" "command -v nsenter"
check "lsns" "command -v lsns"
check "ip (iproute2)" "command -v ip"
check "iptables" "command -v iptables"
check "ping" "command -v ping"
check "busybox" "command -v busybox"
check "cargo" "command -v cargo"
check "rustc" "command -v rustc"
check "git" "command -v git"
check "strace" "command -v strace"
if [ $(id -u) -ne 0 ]; then
    check "sudo" "command -v sudo"
fi

section "Namespace Support"
check "PID namespace" "$SUDO unshare --pid --fork /bin/true"
check "UTS namespace" "$SUDO unshare --uts /bin/true"
check "IPC namespace" "$SUDO unshare --ipc /bin/true"
check "Mount namespace" "$SUDO unshare --mount /bin/true"
check "Network namespace" "$SUDO unshare --net /bin/true"
check "User namespace" "unshare --user /bin/true"

section "Cgroup v2"
check "Cgroup v2 mounted" "mount | grep -q 'cgroup2 on /sys/fs/cgroup'"
check "Cgroup controllers" "test -f /sys/fs/cgroup/cgroup.controllers"
check "Can create cgroups" "$SUDO mkdir -p /sys/fs/cgroup/test-validation && $SUDO rmdir /sys/fs/cgroup/test-validation"

section "Network Capabilities"
check "Create network namespace" "$SUDO ip netns add test-val && $SUDO ip netns delete test-val"
check "Create veth pair" "$SUDO ip link add veth-t0 type veth peer name veth-t1 && $SUDO ip link delete veth-t0"
check "Access iptables NAT" "$SUDO iptables -t nat -L -n"

section "Project Build"
if [ -d "/workspaces/linux-isolation-learning" ]; then
    cd /workspaces/linux-isolation-learning
elif [ -d "$HOME/linux-isolation-learning" ]; then
    cd "$HOME/linux-isolation-learning"
else
    echo -e "${RED}✗${NC} Cannot find project directory"
    ((fail++))
fi

if [ -f "Cargo.toml" ]; then
    check "Cargo.toml exists" "true"
    check "Project builds" "cargo build --quiet 2>&1 | grep -v warning"
else
    echo -e "${RED}✗${NC} Cargo.toml not found"
    ((fail++))
fi

section "Summary"
echo ""
total=$((pass + fail))
if [ $fail -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   ✓ ALL CHECKS PASSED ($pass/$total)                        ║${NC}"
    echo -e "${GREEN}║   Your devcontainer is ready for the tutorials!       ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Read docs/00-getting-started.md"
    echo "  2. Complete docs/00-foundations/"
    echo "  3. Start docs/01-namespaces/01-pid-namespace.md"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║   ✗ VALIDATION FAILED ($fail/$total checks failed)          ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "See devcontainer-validation.md for troubleshooting."
    exit 1
fi
