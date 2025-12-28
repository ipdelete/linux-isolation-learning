# Run with runc (10 min)

## What you'll build

Execute an OCI bundle using runc, the reference container runtime.

## Prerequisites

Install runc:
```bash
# Debian/Ubuntu
sudo apt-get install runc

# Or download from GitHub
curl -LO https://github.com/opencontainers/runc/releases/download/v1.1.12/runc.amd64
sudo install runc.amd64 /usr/local/bin/runc
```

## Setup

Create a working bundle with busybox:
```bash
# Create bundle
cargo run -p contain -- oci init /tmp/testcontainer

# Download and extract busybox rootfs
cd /tmp/testcontainer
curl -LO https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox
mkdir -p rootfs/bin rootfs/usr/bin
cp busybox rootfs/bin/
chmod +x rootfs/bin/busybox

# Create symlinks for common commands
for cmd in sh ls cat echo ps mkdir; do
    ln -s /bin/busybox rootfs/bin/$cmd
done
```

## Run it

```bash
cd /tmp/testcontainer

# Run container (interactive)
sudo runc run mycontainer
```

You're now in a container:
```bash
# Inside container
hostname          # -> random or "runc"
echo $$           # -> 1
ps aux            # -> only your processes
ls /              # -> minimal busybox filesystem
exit
```

## Non-interactive run

```bash
# Modify config.json: set terminal to false, change args
cat > /tmp/testcontainer/config.json << 'EOF'
{
    "ociVersion": "1.0.2",
    "process": {
        "terminal": false,
        "args": ["/bin/sh", "-c", "echo Hello from container && ps aux"],
        "cwd": "/"
    },
    "root": {"path": "rootfs", "readonly": false},
    "linux": {
        "namespaces": [
            {"type": "pid"},
            {"type": "mount"}
        ]
    }
}
EOF

# Run
sudo runc run testrun
```

Output:
```
Hello from container
PID   USER     COMMAND
    1 root     /bin/sh -c echo Hello from container && ps aux
    2 root     ps aux
```

## What just happened

runc reads `config.json`, sets up namespaces/cgroups/mounts per the spec, pivots into `rootfs/`, and execs the process. This is exactly what Docker/containerd do under the hood—they just add image management and networking on top.

## Cleanup

```bash
# List containers
sudo runc list

# Delete stopped container
sudo runc delete mycontainer

# Remove bundle
rm -rf /tmp/testcontainer
```

## Next

[10-ebpf-tracing.md](10-ebpf-tracing.md) — Observe containers with eBPF
