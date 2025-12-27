# 01 OCI Bundle

## Goal

Create a minimal OCI (Open Container Initiative) bundle layout using Rust. You will implement the `init` subcommand in `oci-tool` that creates a directory structure containing `config.json` and an empty `rootfs/` directory. By the end of this lesson, you will understand what an OCI bundle is, why container runtimes need this standardized format, and how to generate valid OCI configuration files programmatically.

**Estimated time**: 45-60 minutes

## Prereqs

- Completed `docs/01-namespaces/` section (understanding of Linux isolation primitives)
- Completed `docs/02-cgroups/` section (understanding of resource control)
- Familiarity with JSON serialization in Rust (we will use `serde` and `serde_json`)
- Basic understanding of filesystem operations in Rust

## Background: What is an OCI Bundle?

An **OCI bundle** is a standardized directory structure that container runtimes (like `runc`, `crun`, or `youki`) use to create and run containers. The Open Container Initiative defines this specification to ensure interoperability between container tools.

**The bundle structure:**

```
my-container/
├── config.json    # OCI runtime configuration
└── rootfs/        # Container's root filesystem
```

**config.json**: This JSON file tells the runtime everything it needs to know to create the container:
- What namespaces to create
- What cgroups to use
- What process to run
- Mount points, environment variables, capabilities, and more

**rootfs/**: This directory becomes the container's `/` (root filesystem). It contains the files the containerized process will see. For now, we create it empty; later lessons will populate it with a minimal Linux filesystem.

**Why this matters:**

1. **Standardization**: Any OCI-compliant runtime can run any OCI bundle
2. **Separation of concerns**: Image tools (like Docker) create bundles; runtimes execute them
3. **Debugging**: You can inspect and modify the bundle before running
4. **Learning**: Understanding bundles helps you understand what container runtimes actually do

**The OCI runtime specification:**

The [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec) defines:
- The bundle format (what we are building in this lesson)
- The runtime lifecycle (create, start, kill, delete)
- Configuration schema (all possible fields in config.json)

Our minimal config.json will include only the required fields:
- `ociVersion`: The OCI spec version (e.g., "1.0.2")
- `root.path`: Path to the rootfs directory
- `process.terminal`: Whether to allocate a pseudo-terminal
- `process.cwd`: Working directory inside the container
- `process.args`: Command to run

## Write Tests (Red)

**Test file**: `crates/oci-tool/tests/init_test.rs`

What the tests should verify:
- Success case: The `init` subcommand creates a bundle directory
- Success case: The bundle contains a valid `config.json` file
- Success case: The bundle contains an empty `rootfs/` directory
- Success case: The config.json has all required OCI fields
- Error case: Attempting to initialize an existing bundle fails

### Step 1: Implement the basic bundle creation test

Open `crates/oci-tool/tests/init_test.rs` and find `test_init_creates_bundle_directory`. Replace the `todo!()` with:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn test_init_creates_bundle_directory() {
    // Create a unique test directory to avoid conflicts
    let test_dir = format!("/tmp/oci-test-bundle-{}", std::process::id());

    // Ensure we start clean
    let _ = fs::remove_dir_all(&test_dir);

    // Run the init command
    let mut cmd = Command::cargo_bin("oci-tool").unwrap();
    cmd.arg("init")
        .arg(&test_dir)
        .assert()
        .success();

    // Verify the bundle directory was created
    assert!(
        Path::new(&test_dir).exists(),
        "Bundle directory should exist at {}",
        test_dir
    );

    // Verify config.json exists
    let config_path = format!("{}/config.json", test_dir);
    assert!(
        Path::new(&config_path).exists(),
        "config.json should exist at {}",
        config_path
    );

    // Verify rootfs directory exists
    let rootfs_path = format!("{}/rootfs", test_dir);
    assert!(
        Path::new(&rootfs_path).is_dir(),
        "rootfs should be a directory at {}",
        rootfs_path
    );

    // Clean up
    fs::remove_dir_all(&test_dir).expect("Failed to clean up test bundle");
}
```

### Step 2: Implement the config.json validation test

Find `test_init_creates_valid_config_json`. Remove the `#[ignore]` attribute and replace the `todo!()`:

```rust
#[test]
fn test_init_creates_valid_config_json() {
    let test_dir = format!("/tmp/oci-test-valid-json-{}", std::process::id());
    let _ = fs::remove_dir_all(&test_dir);

    // Create the bundle
    Command::cargo_bin("oci-tool")
        .unwrap()
        .arg("init")
        .arg(&test_dir)
        .assert()
        .success();

    // Read and parse config.json
    let config_path = format!("{}/config.json", test_dir);
    let config_content = fs::read_to_string(&config_path)
        .expect("Failed to read config.json");

    // Verify it parses as valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&config_content)
        .expect("config.json should be valid JSON");

    // Verify it is a JSON object (not array, string, etc.)
    assert!(
        parsed.is_object(),
        "config.json should be a JSON object"
    );

    // Verify ociVersion is present
    assert!(
        parsed.get("ociVersion").is_some(),
        "config.json should have ociVersion field"
    );

    // Clean up
    fs::remove_dir_all(&test_dir).expect("Failed to clean up");
}
```

**Note**: Add these imports at the top of the test file if not present:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
```

### Step 3: Implement the rootfs verification test

Find `test_init_creates_minimal_rootfs`. Remove `#[ignore]` and replace:

```rust
#[test]
fn test_init_creates_minimal_rootfs() {
    let test_dir = format!("/tmp/oci-test-rootfs-{}", std::process::id());
    let _ = fs::remove_dir_all(&test_dir);

    Command::cargo_bin("oci-tool")
        .unwrap()
        .arg("init")
        .arg(&test_dir)
        .assert()
        .success();

    let rootfs_path = format!("{}/rootfs", test_dir);

    // Verify rootfs is a directory
    assert!(
        Path::new(&rootfs_path).is_dir(),
        "rootfs should be a directory"
    );

    // Verify rootfs is empty (initially)
    let entries: Vec<_> = fs::read_dir(&rootfs_path)
        .expect("Should be able to read rootfs")
        .collect();

    assert!(
        entries.is_empty(),
        "rootfs should be empty initially, found {} entries",
        entries.len()
    );

    fs::remove_dir_all(&test_dir).expect("Failed to clean up");
}
```

### Step 4: Implement the error handling test

Find `test_init_fails_if_bundle_exists`. Remove `#[ignore]` and replace:

```rust
#[test]
fn test_init_fails_if_bundle_exists() {
    let test_dir = format!("/tmp/oci-test-exists-{}", std::process::id());
    let _ = fs::remove_dir_all(&test_dir);

    // Create the bundle first time - should succeed
    Command::cargo_bin("oci-tool")
        .unwrap()
        .arg("init")
        .arg(&test_dir)
        .assert()
        .success();

    // Try to create again - should fail
    Command::cargo_bin("oci-tool")
        .unwrap()
        .arg("init")
        .arg(&test_dir)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("exists")
                .or(predicate::str::contains("already"))
                .or(predicate::str::contains("EEXIST"))
        );

    // Clean up
    fs::remove_dir_all(&test_dir).expect("Failed to clean up");
}
```

### Step 5: Implement the OCI spec compliance test

Find `test_init_config_has_required_fields`. Remove `#[ignore]` and replace:

```rust
#[test]
fn test_init_config_has_required_fields() {
    let test_dir = format!("/tmp/oci-test-fields-{}", std::process::id());
    let _ = fs::remove_dir_all(&test_dir);

    Command::cargo_bin("oci-tool")
        .unwrap()
        .arg("init")
        .arg(&test_dir)
        .assert()
        .success();

    let config_path = format!("{}/config.json", test_dir);
    let config_content = fs::read_to_string(&config_path)
        .expect("Failed to read config.json");

    let config: serde_json::Value = serde_json::from_str(&config_content)
        .expect("config.json should be valid JSON");

    // Check required OCI fields

    // ociVersion (string)
    let oci_version = config.get("ociVersion")
        .expect("ociVersion is required");
    assert!(
        oci_version.is_string(),
        "ociVersion should be a string"
    );

    // root.path (string)
    let root = config.get("root")
        .expect("root is required");
    let root_path = root.get("path")
        .expect("root.path is required");
    assert!(
        root_path.is_string(),
        "root.path should be a string"
    );
    assert_eq!(
        root_path.as_str().unwrap(),
        "rootfs",
        "root.path should be 'rootfs'"
    );

    // process.terminal (boolean)
    let process = config.get("process")
        .expect("process is required");
    let terminal = process.get("terminal")
        .expect("process.terminal is required");
    assert!(
        terminal.is_boolean(),
        "process.terminal should be a boolean"
    );

    // process.cwd (string)
    let cwd = process.get("cwd")
        .expect("process.cwd is required");
    assert!(
        cwd.is_string(),
        "process.cwd should be a string"
    );

    // process.args (array of strings)
    let args = process.get("args")
        .expect("process.args is required");
    assert!(
        args.is_array(),
        "process.args should be an array"
    );
    let args_arr = args.as_array().unwrap();
    assert!(
        !args_arr.is_empty(),
        "process.args should not be empty"
    );

    fs::remove_dir_all(&test_dir).expect("Failed to clean up");
}
```

### Step 6: Run the tests (expect failure)

```bash
cargo test -p oci-tool --test init_test
```

Expected output:

```
running 5 tests
test test_init_creates_bundle_directory ... FAILED
test test_init_creates_minimal_rootfs ... FAILED
test test_init_creates_valid_config_json ... FAILED
test test_init_fails_if_bundle_exists ... FAILED
test test_init_config_has_required_fields ... FAILED

failures:

---- test_init_creates_bundle_directory stdout ----
thread 'test_init_creates_bundle_directory' panicked at crates/oci-tool/src/main.rs:44:13:
not yet implemented: Implement OCI bundle initialization - write tests first! (bundle: /tmp/oci-test-bundle-...)
```

This is the **RED** phase - your tests are written and failing because the implementation does not exist yet.

## Build (Green)

**Implementation file**: `crates/oci-tool/src/main.rs`
**TODO location**: Line 43-45 in the `Command::Init { bundle }` match arm

Now implement the bundle initialization to make all tests pass.

### Step 1: Add necessary imports

At the top of `crates/oci-tool/src/main.rs`, ensure you have these imports:

```rust
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;
```

### Step 2: Implement the init command

Find the `Command::Init { bundle }` match arm and replace the `todo!()` with:

```rust
Command::Init { bundle } => {
    let bundle_path = Path::new(&bundle);

    // Step 1: Check if bundle already exists
    if bundle_path.exists() {
        anyhow::bail!(
            "Bundle directory already exists: {}. \
             Remove it first or choose a different name.",
            bundle
        );
    }

    // Step 2: Create the bundle directory
    fs::create_dir_all(bundle_path)
        .with_context(|| format!("Failed to create bundle directory: {}", bundle))?;

    println!("Created bundle directory: {}", bundle);

    // Step 3: Create the rootfs directory
    let rootfs_path = bundle_path.join("rootfs");
    fs::create_dir(&rootfs_path)
        .with_context(|| format!("Failed to create rootfs directory: {}", rootfs_path.display()))?;

    println!("Created rootfs directory: {}/rootfs", bundle);

    // Step 4: Create a minimal config.json
    // Following OCI runtime-spec for required fields
    let config = json!({
        "ociVersion": "1.0.2",
        "root": {
            "path": "rootfs",
            "readonly": false
        },
        "process": {
            "terminal": true,
            "cwd": "/",
            "args": [
                "/bin/sh"
            ],
            "env": [
                "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                "TERM=xterm"
            ]
        },
        "linux": {
            "namespaces": [
                { "type": "pid" },
                { "type": "mount" },
                { "type": "ipc" },
                { "type": "uts" },
                { "type": "network" }
            ]
        }
    });

    // Step 5: Write config.json to the bundle
    let config_path = bundle_path.join("config.json");
    let config_json = serde_json::to_string_pretty(&config)
        .context("Failed to serialize config.json")?;

    let mut file = fs::File::create(&config_path)
        .with_context(|| format!("Failed to create config.json: {}", config_path.display()))?;

    file.write_all(config_json.as_bytes())
        .context("Failed to write config.json")?;

    println!("Created config.json: {}/config.json", bundle);

    println!("\nOCI bundle initialized successfully!");
    println!("Next steps:");
    println!("  1. Populate rootfs/ with a container filesystem");
    println!("  2. Edit config.json to customize the container");
    println!("  3. Run with: runc run -b {} <container-id>", bundle);

    Ok(())
}
```

### Step 3: Verify the implementation compiles

```bash
cargo build -p oci-tool
```

Expected: Build succeeds with no errors.

### Step 4: Run all tests

```bash
cargo test -p oci-tool --test init_test
```

Expected output:

```
running 5 tests
test test_init_creates_bundle_directory ... ok
test test_init_creates_minimal_rootfs ... ok
test test_init_creates_valid_config_json ... ok
test test_init_fails_if_bundle_exists ... ok
test test_init_config_has_required_fields ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

This is the **GREEN** phase - all tests pass!

## Verify

**Automated verification**:

```bash
# Run all oci-tool tests
cargo test -p oci-tool

# Run just the init tests
cargo test -p oci-tool --test init_test
```

All tests should pass.

**Manual verification** (observe the actual behavior):

1. Create an OCI bundle:

```bash
cargo run -p oci-tool -- init ./my-bundle
```

Expected output:

```
Created bundle directory: ./my-bundle
Created rootfs directory: ./my-bundle/rootfs
Created config.json: ./my-bundle/config.json

OCI bundle initialized successfully!
Next steps:
  1. Populate rootfs/ with a container filesystem
  2. Edit config.json to customize the container
  3. Run with: runc run -b ./my-bundle <container-id>
```

2. Inspect the bundle structure:

```bash
ls -la ./my-bundle/
```

Expected:

```
total 8
drwxr-xr-x 3 user user 4096 ... .
drwxr-xr-x 5 user user 4096 ... ..
-rw-r--r-- 1 user user  XXX ... config.json
drwxr-xr-x 2 user user 4096 ... rootfs
```

3. View the generated config.json:

```bash
cat ./my-bundle/config.json
```

Expected (formatted JSON):

```json
{
  "ociVersion": "1.0.2",
  "root": {
    "path": "rootfs",
    "readonly": false
  },
  "process": {
    "terminal": true,
    "cwd": "/",
    "args": [
      "/bin/sh"
    ],
    "env": [
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
      "TERM=xterm"
    ]
  },
  "linux": {
    "namespaces": [
      { "type": "pid" },
      { "type": "mount" },
      { "type": "ipc" },
      { "type": "uts" },
      { "type": "network" }
    ]
  }
}
```

4. Verify the rootfs is empty:

```bash
ls ./my-bundle/rootfs/
```

Expected: No output (empty directory).

5. Try to initialize the same bundle again (should fail):

```bash
cargo run -p oci-tool -- init ./my-bundle
```

Expected: Error message about bundle already existing.

6. Compare with `runc spec` output (optional, if runc is installed):

```bash
# Create a reference bundle with runc
mkdir /tmp/runc-bundle && cd /tmp/runc-bundle
runc spec
cat config.json | head -50
```

You will notice that `runc spec` generates a much more complete config.json with all optional fields. Our minimal version includes only the essential fields needed for a basic container.

## Clean Up

Remove the test bundle created during manual verification:

```bash
rm -rf ./my-bundle
```

If you created other test bundles:

```bash
# Find and remove test bundles
rm -rf /tmp/oci-test-*
```

## Common Errors

1. **`Bundle directory already exists` when running init**
   - Cause: You already created a bundle with that name
   - Fix: Remove the existing bundle first: `rm -rf ./my-bundle`
   - Or choose a different name: `cargo run -p oci-tool -- init ./my-bundle-2`

2. **`Permission denied` when creating bundle in restricted directory**
   - Cause: You do not have write permission in the target directory
   - Fix: Use a directory you own (e.g., `/tmp`, `./`, or `~/`)
   - Alternatively: `sudo cargo run -p oci-tool -- init /var/lib/my-bundle`

3. **`serde_json` not found or import errors**
   - Cause: Missing dependency or import statements
   - Fix: Verify `Cargo.toml` includes `serde_json = "1.0"` in dependencies
   - Ensure you have `use serde_json::json;` at the top of main.rs

4. **Test fails with "config.json should have ociVersion field"**
   - Cause: The `json!()` macro structure does not match expectations
   - Fix: Verify the JSON structure matches the OCI spec exactly
   - Check that `ociVersion` is spelled correctly (camelCase)

## Notes

**Understanding the config.json structure:**

The OCI runtime spec defines many optional fields, but only a few are truly required for a minimal container:

| Field | Description | Our Value |
|-------|-------------|-----------|
| `ociVersion` | OCI spec version | "1.0.2" |
| `root.path` | Path to rootfs relative to bundle | "rootfs" |
| `root.readonly` | Mount rootfs read-only | false |
| `process.terminal` | Allocate a PTY | true |
| `process.cwd` | Working directory | "/" |
| `process.args` | Command to run | ["/bin/sh"] |
| `linux.namespaces` | Namespaces to create | pid, mount, ipc, uts, network |

**Why we include namespaces in config.json:**

The `linux.namespaces` section tells the runtime which namespaces to create. We include the common ones:
- `pid`: Process isolation (each container has its own PID space)
- `mount`: Filesystem isolation (container has its own mount table)
- `ipc`: IPC isolation (System V IPC, POSIX message queues)
- `uts`: Hostname isolation (container can have its own hostname)
- `network`: Network isolation (container has its own network stack)

We omit `user` namespace for now (it requires additional UID/GID mapping configuration).

**The rootfs directory:**

An empty rootfs is valid for bundle creation, but `runc` will fail to run the container because there is no `/bin/sh` to execute. Later lessons will cover:
- Creating a minimal rootfs with busybox
- Using a base image filesystem
- Bind-mounting the host filesystem

**serde_json::json! macro:**

The `json!()` macro provides a convenient way to construct JSON values:

```rust
let config = json!({
    "key": "value",
    "nested": {
        "array": [1, 2, 3]
    }
});
```

This creates a `serde_json::Value` that can be serialized to a string.

**OCI spec versions:**

- `1.0.0`: Original specification (2017)
- `1.0.1`: Bug fixes and clarifications
- `1.0.2`: Current stable version (recommended)

We use `1.0.2` for compatibility with modern runtimes.

**Relevant documentation:**

- [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec/blob/main/spec.md)
- [config.json Schema](https://github.com/opencontainers/runtime-spec/blob/main/config.md)
- [runc man page](https://github.com/opencontainers/runc/blob/main/man/runc.8.md)
- [serde_json documentation](https://docs.rs/serde_json/)

## Next

`02-config-json.md` - Implement the `show` subcommand to read and display config.json, and learn about parsing OCI configuration files
