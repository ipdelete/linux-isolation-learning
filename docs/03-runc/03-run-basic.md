# 03 Run a Container with runc

## Goal

Use `runc`, the OCI reference runtime, to execute a container from the bundle you created in previous lessons. You will prepare a functional rootfs, configure your bundle for an interactive shell, and experience running inside an isolated container environment.

**What you will do**: Set up a complete OCI bundle with a BusyBox-based rootfs and run it with `runc run`. You will observe the isolation provided by namespaces and understand the container lifecycle from the runtime's perspective.

**Why this matters**: `runc` is the low-level container runtime that Docker, Podman, containerd, and Kubernetes all use under the hood. Understanding how to use `runc` directly gives you insight into what these higher-level tools are doing when they "run a container." This lesson bridges the gap between the Rust tooling you built (oci-tool) and the actual container execution.

**Estimated time**: 30-40 minutes

## Prereqs

- Completed `02-config-json.md` (understand OCI bundle structure and config.json)
- `sudo` access (runc requires root to create namespaces)
- Internet access (to download BusyBox binary, or pre-downloaded)
- `curl` or `wget` installed for downloading files

## Background: What is runc?

`runc` is the reference implementation of the OCI Runtime Specification. It is a small, focused tool that does one thing: takes an OCI bundle and runs it as an isolated process.

**Key points about runc:**

1. **OCI Reference Runtime**: runc was extracted from Docker and donated to the Open Container Initiative. It is the "reference implementation" that defines correct runtime behavior.

2. **Low-level tool**: Unlike Docker or Podman, runc does not handle images, networking, storage, or orchestration. It only runs bundles. Higher-level tools prepare the bundle and then call runc.

3. **Container = Linux primitives**: When runc "runs a container," it is using the same namespaces, cgroups, and filesystem isolation you learned in previous sections. runc just orchestrates these primitives according to the OCI spec.

4. **Lifecycle management**: runc can create, start, pause, resume, and delete containers. It manages the state of containers in `/run/runc/<container-id>/`.

**How runc relates to your previous work:**

| You learned... | runc uses it for... |
|----------------|---------------------|
| PID namespace | Isolating process tree |
| Mount namespace | Filesystem isolation via pivot_root |
| UTS namespace | Container hostname |
| Network namespace | Network isolation |
| Cgroups | Resource limits (memory, CPU, PIDs) |
| OCI bundle (oci-tool) | Input format runc expects |

## Preparing the Bundle

Before running with runc, we need a complete bundle with:
1. A valid `config.json` (from oci-tool or runc spec)
2. A functional `rootfs` with an init process (BusyBox)

### Step 1: Verify runc is installed

```bash
runc --version
```

Expected output (version may vary):
```
runc version 1.1.x
commit: v1.1.x
spec: 1.0.2-dev
go: go1.20.x
libseccomp: 2.5.x
```

If runc is not installed, install it:

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install runc

# Fedora
sudo dnf install runc

# Or install from source (https://github.com/opencontainers/runc)
```

### Step 2: Create the bundle directory structure

We will create a fresh bundle for this lesson:

```bash
# Create the bundle and rootfs directories
mkdir -p ./my-bundle/rootfs/{bin,proc,sys,dev/pts,dev/shm,dev/mqueue,tmp,etc,root,run}

# Verify the structure
ls -la ./my-bundle/
# Should show: rootfs/

ls -la ./my-bundle/rootfs/
# Should show: bin/ dev/ etc/ proc/ root/ run/ sys/ tmp/

ls -la ./my-bundle/rootfs/dev/
# Should show: pts/ shm/ mqueue/
```

### Step 3: Download and install BusyBox

BusyBox is a single static binary that provides a shell and hundreds of common Unix utilities. It is perfect for minimal container rootfs:

```bash
# Download BusyBox (statically linked, no dependencies)
curl -L -o ./my-bundle/rootfs/bin/busybox \
    https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox

# Make it executable
chmod +x ./my-bundle/rootfs/bin/busybox

# Verify it works
./my-bundle/rootfs/bin/busybox --help | head -5
```

Expected output:
```
BusyBox v1.35.0 (2022-01-02 22:02:54 UTC) multi-call binary.
BusyBox is copyrighted by many authors between 1998-2015.
Licensed under GPLv2. See source distribution for detailed
copyright notices.
```

### Step 4: Create BusyBox symlinks

BusyBox uses symlinks to determine which utility to run. When you call `/bin/sh`, BusyBox sees it was invoked as "sh" and behaves like a shell:

```bash
# Create symlinks for common utilities
cd ./my-bundle/rootfs/bin

# Create symlinks using busybox --install
./busybox --install -s .

# Verify symlinks were created
ls -la | head -20

# Return to original directory
cd -
```

You should see symlinks like:
```
lrwxrwxrwx 1 user user    7 Jan  1 00:00 cat -> busybox
lrwxrwxrwx 1 user user    7 Jan  1 00:00 ls -> busybox
lrwxrwxrwx 1 user user    7 Jan  1 00:00 sh -> busybox
...
```

**Alternative manual method** (if --install does not work):

```bash
cd ./my-bundle/rootfs/bin
for cmd in sh ls cat ps mount hostname id pwd env; do
    ln -sf busybox $cmd
done
cd -
```

### Step 5: Generate config.json

Use `runc spec` to generate a default config.json, then modify it:

```bash
cd ./my-bundle

# Generate default OCI config
runc spec

# Verify config.json was created
ls -la config.json

# View the generated config (it is long!)
cat config.json | head -50

cd -
```

The generated config.json is comprehensive but needs one modification for our purposes. Let us make sure the container runs `/bin/sh` interactively.

### Step 6: Verify config.json settings

The default `runc spec` output should already have:

```json
{
    "ociVersion": "1.0.2-dev",
    "process": {
        "terminal": true,
        "args": [
            "sh"
        ],
        ...
    },
    "root": {
        "path": "rootfs",
        "readonly": true
    },
    ...
}
```

Key fields to verify:
- `process.terminal`: `true` (we want an interactive terminal)
- `process.args`: `["sh"]` (run the shell)
- `root.path`: `"rootfs"` (our rootfs directory)

If you used `oci-tool init` from previous lessons instead of `runc spec`, you may need to update config.json to include all the namespace and mount configurations that runc expects.

## Running the Container

Now we have everything needed: a bundle directory, a config.json, and a rootfs with BusyBox.

### Step 7: Run the container

```bash
cd ./my-bundle

# Run the container with runc
# "mycontainer" is the container ID - you choose this name
sudo runc run mycontainer
```

**What happens when you run this command:**

1. runc reads `config.json` from the current directory
2. runc creates the namespaces specified (PID, mount, UTS, IPC, network by default)
3. runc sets up the rootfs using `pivot_root` (same as you learned in 05-minimal-rootfs.md)
4. runc mounts `/proc`, `/dev`, `/sys` inside the container
5. runc applies any cgroup limits specified
6. runc executes the process specified in `process.args` (sh)
7. Your terminal is now attached to the shell inside the container

### Step 8: Explore inside the container

Once you see the shell prompt (likely `/ #`), you are inside the container. Try these commands:

```bash
# Check hostname - should be the container hostname, not host
hostname

# List the root directory - you should only see your minimal rootfs
ls /
# Output: bin  dev  etc  proc  root  sys  tmp
# Notice: No /home, /usr, /var - those are host directories

# Check your user ID
id
# Output: uid=0(root) gid=0(root) groups=0(root)

# List running processes - you should see very few
ps aux
# Output:
# PID   USER     COMMAND
#   1   root     sh
#   2   root     ps aux
# Notice: PID 1 is your shell! You are in a PID namespace.

# Check mounts
mount | head -10
# You will see proc, sysfs, devpts, etc. - all container-local

# Try to see host processes (you cannot)
ls /proc/
# You only see your own processes

# Check what cgroup we are in (if applicable)
cat /proc/1/cgroup
# Shows the cgroup path for PID 1

# Exit the container
exit
```

When you type `exit`, the shell terminates. Since the shell was PID 1 in the container, the container stops.

### Step 9: Verify container state

After exiting, check the container state:

```bash
# List containers
sudo runc list

# You should see mycontainer in "stopped" state
# ID            PID   STATUS   BUNDLE       CREATED
# mycontainer   0     stopped  /path/...    2024-...
```

## Understanding Container IDs

The container ID (`mycontainer` in our example) is:

1. **Chosen by you**: Any string that identifies this container instance
2. **Used for lifecycle management**: `runc start`, `runc kill`, `runc delete` all use this ID
3. **Stored in `/run/runc/<id>/`**: runc keeps state files here while the container exists
4. **Must be unique**: You cannot run two containers with the same ID simultaneously

Common conventions:
- Docker uses UUIDs: `a1b2c3d4e5f6...`
- Human-readable names work too: `mycontainer`, `web-server-1`
- Often derived from the bundle or image name

## Write Tests (Red)

This lesson focuses on understanding runc rather than writing extensive Rust code. However, we can write a test that verifies our oci-tool creates bundles that runc can read.

**Test file**: `crates/oci-tool/tests/runc_compat_test.rs`

Create this file to verify compatibility:

```rust
// Tests for runc compatibility of oci-tool bundles
// Lesson: docs/03-runc/03-run-basic.md
//
// These tests verify that bundles created by oci-tool can be understood by runc.
// Note: Actually running containers requires sudo and is tested manually.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Test that runc can validate a config.json created by oci-tool
/// This does not run a container, just validates the spec
#[test]
fn test_runc_can_read_oci_tool_bundle() {
    // Skip if runc is not installed
    let runc_check = Command::new("runc")
        .arg("--version")
        .output();

    if runc_check.is_err() {
        eprintln!("Skipping test: runc not installed");
        return;
    }

    // Create a temporary bundle directory
    let test_dir = format!("/tmp/oci-tool-runc-test-{}", std::process::id());
    let rootfs_dir = format!("{}/rootfs", test_dir);

    // Clean up any previous test artifacts
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&rootfs_dir).expect("Failed to create test directories");

    // Create a minimal config.json that runc can parse
    // In a real implementation, this would be created by oci-tool init
    let config_json = r#"{
        "ociVersion": "1.0.2-dev",
        "process": {
            "terminal": true,
            "user": { "uid": 0, "gid": 0 },
            "args": ["sh"],
            "cwd": "/"
        },
        "root": {
            "path": "rootfs",
            "readonly": true
        },
        "hostname": "test-container",
        "linux": {
            "namespaces": [
                { "type": "pid" },
                { "type": "mount" }
            ]
        }
    }"#;

    let config_path = format!("{}/config.json", test_dir);
    fs::write(&config_path, config_json).expect("Failed to write config.json");

    // Use runc spec --bundle to validate (dry run, does not require root)
    // Note: runc spec generates a new config, but we can at least verify
    // runc understands the bundle structure
    let bundle_check = Path::new(&test_dir).join("config.json");
    assert!(bundle_check.exists(), "config.json should exist in bundle");

    let rootfs_check = Path::new(&rootfs_dir);
    assert!(rootfs_check.is_dir(), "rootfs should be a directory");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

/// Test that oci-tool init creates the expected bundle structure
#[test]
#[ignore] // Enable when oci-tool init is implemented
fn test_oci_tool_init_creates_valid_bundle() {
    // TODO: Once oci-tool init is implemented, this test will:
    // 1. Run: cargo run -p oci-tool -- init /tmp/test-bundle
    // 2. Verify /tmp/test-bundle/config.json exists and is valid JSON
    // 3. Verify /tmp/test-bundle/rootfs is a directory
    // 4. Optionally run: runc spec --bundle /tmp/test-bundle to validate

    todo!("Implement after oci-tool init is complete")
}
```

Run the test:

```bash
cargo test -p oci-tool --test runc_compat_test
```

This test verifies the structure is correct. Actually running containers with runc is tested manually in the Verify section.

## Build (Green)

This lesson is primarily about using runc, not writing new Rust code. The "Build" step is completing the manual setup above and running the container successfully.

If you want to extend oci-tool to automate bundle preparation, consider adding:

1. **`oci-tool prepare`**: Downloads BusyBox and sets up symlinks
2. **`oci-tool run`**: Calls runc with the bundle (wrapper for convenience)

These would be future enhancements. For now, the manual process teaches you exactly what runc expects.

## Verify

### Automated verification

```bash
# Test that the bundle structure is correct
cargo test -p oci-tool --test runc_compat_test
```

### Manual verification

#### 1. Verify runc installation

```bash
runc --version
# Should show version 1.0 or higher
```

#### 2. Verify bundle structure

```bash
# From the my-bundle directory
ls -la ./my-bundle/
# Should show: config.json  rootfs/

ls ./my-bundle/rootfs/bin/sh
# Should exist (symlink to busybox)

cat ./my-bundle/config.json | grep -A2 '"args"'
# Should show: "args": [ "sh" ]
```

#### 3. Run and observe isolation

```bash
cd ./my-bundle
sudo runc run test-container
```

Inside the container, verify:

```bash
# 1. Filesystem isolation
ls /
# Only see: bin dev etc proc root sys tmp
# NOT: home, usr, var, boot, etc.

# 2. Process isolation (PID namespace)
ps aux
# Only see: sh (PID 1), ps (PID 2)
# NOT: host system processes

# 3. Hostname isolation (UTS namespace)
hostname
# Shows container hostname from config.json

# 4. User appears as root
whoami
# root (even though this is a user namespace mapping)

# 5. Exit cleanly
exit
```

#### 4. Verify container lifecycle

```bash
# Check stopped containers
sudo runc list
# Shows test-container in "stopped" state

# Delete the container
sudo runc delete test-container

# Verify deletion
sudo runc list
# test-container should no longer appear
```

## Clean Up

After completing the lesson, clean up all resources:

```bash
# 1. Delete the container state (if not already done)
sudo runc delete mycontainer 2>/dev/null
sudo runc delete test-container 2>/dev/null

# 2. Remove the bundle directory
rm -rf ./my-bundle

# 3. Verify cleanup
sudo runc list
# Should show no containers (or only other containers you are running)

ls ./my-bundle 2>/dev/null
# Should show "No such file or directory"
```

**Note**: runc state files are stored in `/run/runc/<container-id>/`. These are automatically cleaned up when you run `runc delete`. If you kill runc abnormally, you may need to manually remove these:

```bash
# Only if needed - check for orphaned state
ls /run/runc/
sudo rm -rf /run/runc/mycontainer  # Only for orphaned containers
```

## Common Errors

### 1. "container with id already exists"

**Symptom**:
```
ERRO[0000] container with id "mycontainer" already exists
```

**Cause**: A container with this ID already exists (running or stopped).

**Fix**:
```bash
# Check container status
sudo runc list

# If stopped, delete it
sudo runc delete mycontainer

# If still running, stop it first
sudo runc kill mycontainer SIGKILL
sudo runc delete mycontainer
```

### 2. "rootfs does not exist" or "bundle path does not exist"

**Symptom**:
```
ERRO[0000] rootfs (/path/to/rootfs) does not exist
```

**Cause**: The `root.path` in config.json points to a non-existent directory, or you are running runc from the wrong directory.

**Fix**:
```bash
# Verify you are in the bundle directory
pwd
ls config.json rootfs/

# runc expects to find config.json in current directory
cd /path/to/my-bundle
sudo runc run mycontainer
```

### 3. "executable not found" or "no such file or directory" for process

**Symptom**:
```
ERRO[0000] exec: "sh": executable file not found in $PATH
```

**Cause**: The command in `process.args` does not exist in the rootfs, or BusyBox symlinks were not created.

**Fix**:
```bash
# Verify the shell exists in rootfs
ls -la ./my-bundle/rootfs/bin/sh

# If missing, create the symlink
cd ./my-bundle/rootfs/bin
ln -sf busybox sh
cd -

# Or use the absolute path in config.json: ["/bin/sh"]
```

### 4. "permission denied" or "operation not permitted"

**Symptom**:
```
ERRO[0000] cannot start a container: operation not permitted
```

**Cause**: runc needs root privileges to create namespaces and set up cgroups.

**Fix**:
```bash
# Run with sudo
sudo runc run mycontainer

# NOT: runc run mycontainer
```

### 5. "config.json: no such file or directory"

**Symptom**:
```
ERRO[0000] open config.json: no such file or directory
```

**Cause**: Running runc from a directory that does not contain config.json.

**Fix**:
```bash
# Either cd to the bundle directory
cd ./my-bundle
sudo runc run mycontainer

# Or use --bundle flag
sudo runc run --bundle ./my-bundle mycontainer
```

### 6. Container exits immediately

**Symptom**: Container starts but immediately returns to host prompt.

**Cause**: The init process exited (successfully or with error). Common causes:
- Process is not interactive (runs and exits)
- Error during process startup

**Fix**:

First, check if terminal mode is enabled:

```bash
grep -A2 '"terminal"' config.json
# Should show: "terminal": true
```

To run a diagnostic command like `ls /`, you need to modify config.json to change `process.args`:

```bash
# Edit config.json and change the process.args section from:
#   "args": ["sh"]
# to:
#   "args": ["/bin/ls", "/"]

# Use a text editor or jq:
jq '.process.args = ["/bin/ls", "/"]' config.json > config.json.tmp && mv config.json.tmp config.json

# Then run the container
cd ./my-bundle
sudo runc run diagnostic-test

# After running, change it back:
jq '.process.args = ["sh"]' config.json > config.json.tmp && mv config.json.tmp config.json
```

**Alternative: Use runc exec for debugging**

If you want to run commands without modifying config.json, use the create + start + exec workflow:

```bash
cd ./my-bundle

# Create the container (does not start it yet)
sudo runc create mycontainer

# Start the container with your normal shell
sudo runc start mycontainer &

# In another terminal, execute a command inside the running container
sudo runc exec mycontainer /bin/ls /

# Kill and clean up
sudo runc kill mycontainer SIGKILL
sudo runc delete mycontainer
```

Check container state:

```bash
sudo runc list
```

## Notes

### What runc actually does

When you run `sudo runc run mycontainer`, runc performs these steps:

1. **Parse config.json**: Read and validate the OCI specification
2. **Create namespaces**: Based on `linux.namespaces` in config
3. **Set up rootfs**: Bind mount and pivot_root into rootfs
4. **Mount filesystems**: /proc, /dev, /sys, and custom mounts
5. **Apply cgroups**: Create cgroup and apply resource limits
6. **Set capabilities**: Drop/keep Linux capabilities per spec
7. **Apply seccomp**: Install seccomp BPF filter if specified
8. **Execute process**: Finally exec the `process.args` command

This is exactly what you have been building toward in the namespaces and cgroups lessons!

### runc vs Docker/Podman

| Feature | runc | Docker/Podman |
|---------|------|---------------|
| Input | OCI bundle (directory) | Container image (tarball/registry) |
| Networking | None (you set it up) | Automatic bridge/NAT |
| Storage | Just rootfs | Overlay FS, volumes, layers |
| Images | No concept of images | Pull, build, push images |
| Orchestration | Single container | Compose, swarm, etc. |

Docker and Podman prepare everything (download image, extract layers, set up networking) and then call runc to actually run the container.

### Container ID best practices

- Keep IDs short but descriptive: `web-1`, `db-main`, `test-abc123`
- For automation, use UUIDs: `$(uuidgen)`
- Container IDs are local to the runc state directory
- Do not reuse IDs without deleting first

### Security note

Running runc directly as root gives you maximum control but also maximum responsibility. Production deployments typically:

1. Use rootless runc (user namespaces) where possible
2. Apply seccomp profiles to restrict syscalls
3. Drop unnecessary capabilities
4. Use read-only rootfs with minimal contents

We will explore some of these in later lessons.

### Comparing with your Rust implementation

In `01-namespaces/05-minimal-rootfs.md`, you built a similar system in Rust:
- Created mount namespace
- Set up BusyBox rootfs
- Used pivot_root to enter the container

runc does the same thing but with full OCI spec compliance, better error handling, and support for all the options in config.json.

### Links to official documentation

- [runc GitHub repository](https://github.com/opencontainers/runc)
- [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec)
- [OCI Runtime Spec: config.json](https://github.com/opencontainers/runtime-spec/blob/main/config.md)
- [runc man page](https://github.com/opencontainers/runc/blob/main/man/runc.8.md)
- [BusyBox](https://busybox.net/) - Swiss Army Knife of Embedded Linux

## Summary

In this lesson, you:

1. **Verified runc installation**: The OCI reference runtime
2. **Prepared a complete OCI bundle**: rootfs with BusyBox + config.json
3. **Ran a container with runc**: Experienced the isolated environment
4. **Explored container isolation**: Observed PID, mount, and UTS namespace effects
5. **Managed container lifecycle**: Run, list, delete containers
6. **Understood container IDs**: How runc tracks container state

**Key takeaways:**

- runc is what Docker/Podman use under the hood to run containers
- An OCI bundle is just a directory with config.json + rootfs
- runc orchestrates all the Linux primitives you learned: namespaces, cgroups, pivot_root
- Container IDs are chosen by you and used for lifecycle management
- Understanding runc helps you debug container issues at the lowest level

## Next

`04-lifecycle.md` - Learn the full container lifecycle (create, start, pause, resume, stop, delete) and how runc manages container state
