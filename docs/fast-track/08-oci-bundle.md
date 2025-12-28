# OCI Bundle (10 min)

## What you'll build

Create an OCI-compliant container bundle that runc can execute.

## The test

**File**: `crates/contain/tests/oci_test.rs`

```rust
#[test]
fn test_oci_bundle_init() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().join("mybundle");

    Command::cargo_bin("contain").unwrap()
        .args(["oci", "init", bundle.to_str().unwrap()])
        .assert().success();

    // Verify structure
    assert!(bundle.join("config.json").exists());
    assert!(bundle.join("rootfs").exists());
}
```

Run it: `cargo test -p contain --test oci_test`

## The implementation

**File**: `crates/contain/src/oci.rs`

```rust
OciCommand::Init { path } => {
    use std::fs;

    // Create bundle directory
    fs::create_dir_all(&path)?;

    // Create rootfs
    let rootfs = format!("{}/rootfs", path);
    fs::create_dir_all(&rootfs)?;

    // Create minimal config.json
    let config = r#"{
    "ociVersion": "1.0.2",
    "process": {
        "terminal": true,
        "args": ["/bin/sh"],
        "cwd": "/"
    },
    "root": {
        "path": "rootfs",
        "readonly": false
    },
    "linux": {
        "namespaces": [
            {"type": "pid"},
            {"type": "mount"},
            {"type": "uts"},
            {"type": "ipc"}
        ]
    }
}"#;

    fs::write(format!("{}/config.json", path), config)?;

    println!("Created OCI bundle at {}", path);
    println!("  - config.json");
    println!("  - rootfs/");
    Ok(())
}
```

## Run it

```bash
# Create bundle
cargo run -p contain -- oci init /tmp/mybundle

# Check structure
ls -la /tmp/mybundle/
cat /tmp/mybundle/config.json

# Add a minimal rootfs (busybox)
mkdir -p /tmp/mybundle/rootfs/bin
cp /bin/busybox /tmp/mybundle/rootfs/bin/
ln -s busybox /tmp/mybundle/rootfs/bin/sh

# Verify
ls /tmp/mybundle/rootfs/bin/
```

## What just happened

An OCI bundle has two parts: `config.json` (container spec) and `rootfs/` (filesystem). The config defines namespaces, process args, mounts, and limits. This is the standard format that runc, containerd, and other runtimes understand.

## Next

[09-runc-run.md](09-runc-run.md) — Run the bundle with runc
