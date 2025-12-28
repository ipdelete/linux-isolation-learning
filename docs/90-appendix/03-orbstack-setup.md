# OrbStack Setup Guide

This guide walks you through setting up OrbStack on macOS for the cgroup lessons (06-07).

## What is OrbStack?

OrbStack is a fast, lightweight way to run Linux VMs and Docker containers on macOS. Unlike UTM or VirtualBox, it:
- Requires no ISO downloads
- Creates VMs in seconds
- Auto-configures SSH access
- Uses minimal resources

## Prerequisites

- macOS 12.3+ (Monterey or later)
- Apple Silicon (M1/M2/M3) or Intel Mac
- ~2GB disk space

## Step 1: Download and Install OrbStack

1. **Download**: Visit https://orbstack.dev/ and click "Download"
   - Or use Homebrew: `brew install orbstack`

2. **Install**: Open the downloaded `.dmg` and drag OrbStack to Applications

3. **Launch**: Open OrbStack from Applications
   - Grant permissions when prompted (Full Disk Access for file sharing)
   - The OrbStack icon appears in your menu bar

## Step 2: Create a Linux Machine

Open Terminal and run:

```bash
orb create ubuntu cgroup-vm
```

This downloads Ubuntu and creates a VM named `cgroup-vm`. Takes ~30 seconds on first run.

**What just happened?**
- OrbStack pulled an Ubuntu image (no ISO needed)
- Created a lightweight VM with systemd
- Auto-configured SSH access
- Added entries to your `~/.ssh/config`

## Step 3: Access Your Machine

**Option A: Direct shell**
```bash
orb -m cgroup-vm
```

**Option B: SSH (recommended for VS Code)**
```bash
ssh cgroup-vm@orb
```

You're now in a full Ubuntu environment with systemd.

## Step 4: Verify Cgroup Support

Inside the machine, check that cgroup delegation works:

```bash
# Check your user's cgroup slice
cat /sys/fs/cgroup/user.slice/user-$(id -u).slice/cgroup.subtree_control
```

**Expected output**: `cpu cpuset io memory hugetlb pids rdma misc`

If you see `memory` in the list, cgroup memory limits will work!

## Step 5: Install Rust Toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Accept defaults (option 1)
# Then reload your shell
source ~/.cargo/env

# Verify
rustc --version
cargo --version
```

## Step 6: Clone the Repository

```bash
git clone https://github.com/YOUR_USERNAME/linux-isolation-learning.git
cd linux-isolation-learning
cargo build
```

## Step 7: Connect VS Code

1. Install the **Remote - SSH** extension in VS Code

2. Open Command Palette (`Cmd+Shift+P`)

3. Type "Remote-SSH: Connect to Host"

4. Select `cgroup-vm@orb` (OrbStack added this automatically)

5. VS Code opens a new window connected to your Linux VM

6. Open the cloned repository folder

## Managing Your Machine

**Start/Stop**:
```bash
orb start cgroup-vm    # Start the machine
orb stop cgroup-vm     # Stop the machine
```

**List machines**:
```bash
orb list
```

**Delete machine** (if needed):
```bash
orb delete cgroup-vm
```

**Resource limits** (optional):
```bash
orb config set memory_mib 4096   # Set max memory to 4GB
orb config set cpu 4              # Set max CPU cores to 4
```

## Troubleshooting

### "Permission denied" when writing to cgroup files

Make sure you're using `sudo`:
```bash
sudo -E cargo test -p contain --test cgroup_test
```

### Cgroup controllers not showing

Check if you're in the correct cgroup path:
```bash
cat /proc/self/cgroup
# Should show something like: 0::/user.slice/user-1000.slice/...
```

### VS Code can't connect via SSH

Verify SSH works from terminal first:
```bash
ssh cgroup-vm@orb
```

If that fails, restart OrbStack from the menu bar.

### Machine won't start

```bash
orb stop cgroup-vm
orb start cgroup-vm
```

Or reset: `orb delete cgroup-vm && orb create ubuntu cgroup-vm`

## Next Steps

Return to the lesson you were working on:
- [Lesson 06: Memory Limits](../fast-track/06-memory-limits.md)
- [Lesson 07: CPU Limits](../fast-track/07-cpu-limits.md)
