# 02 Reading OCI config.json

## Goal

Read and display OCI `config.json` files, understanding the OCI runtime specification structure. You will implement the `show` subcommand in `oci-tool` that reads a bundle's configuration, validates it as JSON, and pretty-prints it to stdout.

**Estimated time**: 40-50 minutes

## Prereqs

- Completed `01-oci-bundle.md` (you should have a working `oci-tool init` command)
- Basic understanding of JSON structure
- Familiarity with Rust's `serde` and `serde_json` crates (introduced in foundations)

## Background: The OCI Runtime Specification

The Open Container Initiative (OCI) defines a standard format for container bundles. At the heart of every OCI bundle is the `config.json` file, which describes how the container should be created and run.

### Bundle Structure Recap

An OCI bundle is a directory containing:

```
my-bundle/
├── config.json    # Container configuration (this lesson's focus)
└── rootfs/        # Root filesystem for the container
```

### config.json Structure

The `config.json` file follows the OCI Runtime Specification. Here is a minimal example with the most important fields:

```json
{
    "ociVersion": "1.0.0",
    "root": {
        "path": "rootfs",
        "readonly": false
    },
    "process": {
        "terminal": true,
        "cwd": "/",
        "args": ["/bin/sh"],
        "env": [
            "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            "TERM=xterm"
        ]
    },
    "linux": {
        "namespaces": [
            {"type": "pid"},
            {"type": "network"},
            {"type": "ipc"},
            {"type": "uts"},
            {"type": "mount"}
        ],
        "resources": {
            "memory": {
                "limit": 536870912
            }
        }
    },
    "mounts": [
        {
            "destination": "/proc",
            "type": "proc",
            "source": "proc"
        }
    ]
}
```

### Key Sections Explained

| Section | Required | Purpose |
|---------|----------|---------|
| `ociVersion` | Yes | Specifies which version of the OCI spec this config follows (e.g., "1.0.0") |
| `root` | Yes | Defines the container's root filesystem location and whether it is read-only |
| `process` | Yes | Describes what process to run, its working directory, arguments, and environment |
| `linux` | No* | Linux-specific settings: namespaces, cgroups, seccomp, capabilities |
| `mounts` | No | Additional filesystem mounts (proc, sysfs, tmpfs, bind mounts) |

*The `linux` section is required on Linux platforms but the spec also defines `windows`, `solaris`, and other platform-specific sections.

### Required vs Optional Fields

**Always required:**
- `ociVersion` - String indicating spec version
- `root.path` - Path to the root filesystem (relative to bundle)
- `process.cwd` - Working directory inside the container
- `process.args` - Array of strings for the command to run

**Commonly used but optional:**
- `process.terminal` - Whether to allocate a pseudo-TTY (default: false)
- `process.env` - Environment variables
- `root.readonly` - Whether root filesystem is read-only (default: false)
- `linux.namespaces` - Which namespaces to create
- `linux.resources` - Cgroup resource limits
- `mounts` - Additional mount points

### The linux.namespaces Array

This section connects directly to what you learned in the namespaces lessons:

```json
"namespaces": [
    {"type": "pid"},              // PID namespace (01-pid-namespace.md)
    {"type": "network"},          // Network namespace (06-netns-basics.md)
    {"type": "ipc"},              // IPC namespace (03-uts-ipc.md)
    {"type": "uts"},              // UTS namespace (03-uts-ipc.md)
    {"type": "mount"},            // Mount namespace (04-mount-namespace.md)
    {"type": "user"},             // User namespace (optional)
    {"type": "cgroup"}            // Cgroup namespace (optional)
]
```

Each namespace type corresponds to the `CLONE_NEW*` flags you used with `unshare()` and `clone()`.

### The linux.resources Section

This connects to the cgroups lessons:

```json
"resources": {
    "memory": {
        "limit": 536870912,       // 512 MB (02-memory.md)
        "swap": 536870912
    },
    "cpu": {
        "quota": 50000,           // 50% of one CPU (03-cpu.md)
        "period": 100000
    },
    "pids": {
        "limit": 100              // Max 100 processes (05-pids.md)
    }
}
```

## Write Tests (Red)

**Test file**: `crates/oci-tool/tests/show_test.rs`

We will implement four tests that verify the `show` subcommand correctly reads and displays `config.json` files. Following TDD, we write the tests first, watch them fail, then implement the code.

### Part 1: Test basic config display

Open `crates/oci-tool/tests/show_test.rs` and replace the first test:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn test_show_displays_config() {
    // Create a temporary test bundle with a known config.json
    let test_bundle = "/tmp/oci-show-test-basic";
    let config_path = format!("{}/config.json", test_bundle);
    let rootfs_path = format!("{}/rootfs", test_bundle);

    // Clean up any previous test artifacts
    let _ = fs::remove_dir_all(test_bundle);

    // Create the bundle structure manually for this test
    fs::create_dir_all(&rootfs_path).expect("Failed to create test bundle");

    // Write a minimal config.json
    let config_content = r#"{
    "ociVersion": "1.0.0",
    "root": {
        "path": "rootfs",
        "readonly": false
    },
    "process": {
        "terminal": true,
        "cwd": "/",
        "args": ["/bin/sh"]
    }
}"#;

    fs::write(&config_path, config_content).expect("Failed to write config.json");

    // Run the show command
    let mut cmd = Command::cargo_bin("oci-tool").unwrap();
    cmd.arg("show")
        .arg(test_bundle)
        .assert()
        .success()
        .stdout(predicate::str::contains("ociVersion"))
        .stdout(predicate::str::contains("1.0.0"))
        .stdout(predicate::str::contains("rootfs"))
        .stdout(predicate::str::contains("/bin/sh"));

    // Clean up
    fs::remove_dir_all(test_bundle).expect("Failed to clean up test bundle");
}
```

### Part 2: Test pretty-printed output

Remove the `#[ignore]` attribute from the second test and replace the `todo!()`:

```rust
#[test]
fn test_show_formats_json_pretty() {
    let test_bundle = "/tmp/oci-show-test-pretty";
    let config_path = format!("{}/config.json", test_bundle);
    let rootfs_path = format!("{}/rootfs", test_bundle);

    let _ = fs::remove_dir_all(test_bundle);
    fs::create_dir_all(&rootfs_path).expect("Failed to create test bundle");

    // Write compact JSON (no formatting)
    let compact_config = r#"{"ociVersion":"1.0.0","root":{"path":"rootfs"},"process":{"cwd":"/","args":["/bin/sh"]}}"#;
    fs::write(&config_path, compact_config).expect("Failed to write config.json");

    // Run the show command
    let mut cmd = Command::cargo_bin("oci-tool").unwrap();
    let output = cmd
        .arg("show")
        .arg(test_bundle)
        .assert()
        .success();

    // Get the stdout as a string
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Pretty-printed JSON should have newlines and indentation
    assert!(
        stdout.contains('\n'),
        "Output should contain newlines for pretty printing"
    );
    assert!(
        stdout.contains("  ") || stdout.contains('\t'),
        "Output should contain indentation"
    );

    // Clean up
    fs::remove_dir_all(test_bundle).expect("Failed to clean up");
}
```

### Part 3: Test error when bundle is missing

Remove the `#[ignore]` attribute from the third test and replace:

```rust
#[test]
fn test_show_fails_if_bundle_missing() {
    // Use a path that definitely does not exist
    let nonexistent_bundle = "/tmp/oci-show-test-nonexistent-bundle-xyz";

    // Ensure it really does not exist
    let _ = fs::remove_dir_all(nonexistent_bundle);

    let mut cmd = Command::cargo_bin("oci-tool").unwrap();
    cmd.arg("show")
        .arg(nonexistent_bundle)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No such file")
                .or(predicate::str::contains("not found"))
                .or(predicate::str::contains("does not exist"))
                .or(predicate::str::contains("Failed to read")),
        );
}
```

### Part 4: Test error when config.json is missing

Remove the `#[ignore]` attribute from the fourth test and replace:

```rust
#[test]
fn test_show_fails_if_config_missing() {
    let test_bundle = "/tmp/oci-show-test-no-config";
    let rootfs_path = format!("{}/rootfs", test_bundle);

    let _ = fs::remove_dir_all(test_bundle);

    // Create bundle directory with rootfs but WITHOUT config.json
    fs::create_dir_all(&rootfs_path).expect("Failed to create test bundle");

    // Verify config.json does not exist
    let config_path = format!("{}/config.json", test_bundle);
    assert!(
        !Path::new(&config_path).exists(),
        "config.json should not exist for this test"
    );

    let mut cmd = Command::cargo_bin("oci-tool").unwrap();
    cmd.arg("show")
        .arg(test_bundle)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("config.json")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("not found")),
        );

    // Clean up
    fs::remove_dir_all(test_bundle).expect("Failed to clean up");
}
```

### Run the tests (expect failure)

```bash
cargo test -p oci-tool --test show_test
```

Expected output:

```
running 4 tests
test test_show_displays_config ... FAILED
test test_show_formats_json_pretty ... FAILED
test test_show_fails_if_bundle_missing ... FAILED
test test_show_fails_if_config_missing ... FAILED

failures:

---- test_show_displays_config stdout ----
thread 'test_show_displays_config' panicked at crates/oci-tool/src/main.rs:62:13:
not yet implemented: Implement config.json display - write tests first! (bundle: /tmp/oci-show-test-basic)
```

This is the **RED** phase. The tests fail because the `show` command is not yet implemented.

## Build (Green)

**Implementation file**: `crates/oci-tool/src/main.rs`

**TODO location**: Line ~61 in the `Command::Show { bundle }` match arm

### Step 1: Add necessary imports

If not already present at the top of `main.rs`, ensure you have:

```rust
use anyhow::{Context, Result};
```

### Step 2: Implement the show command

Find the `Command::Show { bundle }` match arm and replace the `todo!()` with:

```rust
Command::Show { bundle } => {
    use std::fs;
    use std::path::Path;

    // Construct the path to config.json
    let bundle_path = Path::new(&bundle);
    let config_path = bundle_path.join("config.json");

    // Read the config file
    let config_content = fs::read_to_string(&config_path)
        .with_context(|| {
            format!(
                "Failed to read config.json from bundle '{}'. \
                 Does the bundle exist and contain a config.json file?",
                bundle
            )
        })?;

    // Parse as JSON to validate it
    let config_json: serde_json::Value = serde_json::from_str(&config_content)
        .with_context(|| {
            format!(
                "Failed to parse config.json as valid JSON. \
                 The file exists but contains invalid JSON."
            )
        })?;

    // Pretty-print the JSON to stdout
    let pretty_output = serde_json::to_string_pretty(&config_json)
        .with_context(|| "Failed to format JSON for display")?;

    println!("{}", pretty_output);
}
```

### Understanding the Implementation

Let us break down what this code does:

1. **Path construction**: We use `Path::join()` to safely concatenate the bundle path with "config.json". This handles path separators correctly across platforms.

2. **Reading the file**: `fs::read_to_string()` reads the entire file into a `String`. We use `with_context()` to provide a helpful error message if the file cannot be read.

3. **JSON parsing**: We parse the string as a `serde_json::Value`, which is a dynamic JSON type that can represent any valid JSON. This validates that the file contains valid JSON without requiring a specific structure.

4. **Pretty-printing**: `serde_json::to_string_pretty()` formats the JSON with indentation and newlines, making it human-readable.

### Why use serde_json::Value?

For the `show` command, we use the dynamic `Value` type rather than defining a struct for the entire OCI spec. This has two advantages:

1. **Simplicity**: We do not need to define structs for every possible field in the OCI spec
2. **Forward compatibility**: The command will work with any valid JSON, even if new fields are added to the spec

In later lessons, when we need to access specific fields (like `process.args` or `linux.namespaces`), we will define proper structs with `#[derive(Deserialize)]`.

### Step 3: Verify the code compiles

```bash
cargo build -p oci-tool
```

### Step 4: Run the tests

```bash
cargo test -p oci-tool --test show_test
```

Expected output:

```
running 4 tests
test test_show_displays_config ... ok
test test_show_formats_json_pretty ... ok
test test_show_fails_if_bundle_missing ... ok
test test_show_fails_if_config_missing ... ok

test result: ok. 4 passed; 0 failed
```

This is the **GREEN** phase!

## Verify

**Automated verification**:

```bash
# Run all show tests
cargo test -p oci-tool --test show_test

# Run all oci-tool tests
cargo test -p oci-tool
```

**Manual verification**:

1. First, create a test bundle using the `init` command from the previous lesson:

```bash
cargo run -p oci-tool -- init /tmp/verify-bundle
```

2. Now use `show` to display the configuration:

```bash
cargo run -p oci-tool -- show /tmp/verify-bundle
```

Expected output (formatted JSON with the bundle's config):

```json
{
  "ociVersion": "1.0.0",
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
  }
}
```

3. Try showing a non-existent bundle (should fail with a clear error):

```bash
cargo run -p oci-tool -- show /tmp/does-not-exist
```

Expected output:

```
Error: Failed to read config.json from bundle '/tmp/does-not-exist'. Does the bundle exist and contain a config.json file?

Caused by:
    No such file or directory (os error 2)
```

4. Test with a real OCI config from `runc spec` (if runc is installed):

```bash
mkdir -p /tmp/runc-bundle
cd /tmp/runc-bundle
runc spec  # Generates a full OCI config.json

cargo run -p oci-tool -- show /tmp/runc-bundle
```

This will display a much more complete configuration with all OCI spec fields.

## Clean Up

Remove test bundles created during verification:

```bash
rm -rf /tmp/verify-bundle
rm -rf /tmp/runc-bundle
rm -rf /tmp/oci-show-test-*
```

## Common Errors

1. **`Failed to read config.json from bundle ... No such file or directory`**
   - Cause: The bundle path does not exist, or config.json is missing
   - Fix: Ensure you created the bundle with `oci-tool init` first
   - Check: `ls /path/to/bundle/` should show both `config.json` and `rootfs/`

2. **`Failed to parse config.json as valid JSON`**
   - Cause: The config.json file exists but contains invalid JSON (syntax error)
   - Fix: Check the file for JSON syntax errors (missing commas, unclosed braces)
   - Debug: Use `cat /path/to/bundle/config.json | jq .` to validate

3. **Compilation error: `use of undeclared crate or module serde_json`**
   - Cause: Missing dependency in Cargo.toml
   - Fix: Ensure `Cargo.toml` includes:
     ```toml
     [dependencies]
     serde_json = "1.0"
     ```

4. **Compilation error: `cannot find function with_context`**
   - Cause: Missing import for `anyhow::Context`
   - Fix: Add at the top of main.rs:
     ```rust
     use anyhow::{Context, Result};
     ```

## Notes

**The OCI Runtime Specification:**
- The full specification is at: https://github.com/opencontainers/runtime-spec
- Version 1.0.0 is widely supported; 1.1.0 adds features like memory.checkBeforeUpdate
- The spec is platform-specific: Linux uses `linux`, Windows uses `windows`

**serde_json::Value vs Typed Structs:**
- `Value` is useful for dynamic JSON manipulation or when you do not know the structure
- Typed structs (`#[derive(Deserialize)]`) are better when you need compile-time safety
- We will use typed structs in later lessons when running containers

**Pretty-printing considerations:**
- `to_string_pretty()` uses 2-space indentation by default
- For custom formatting, use `serde_json::ser::PrettyFormatter`
- The formatted output is valid JSON and can be piped to other tools

**Inspecting real container configs:**
- Docker: `docker inspect <container> | jq '.[0].HostConfig'`
- Podman: `podman inspect <container> --format json`
- runc: Look at `/run/runc/<container-id>/config.json` on a running system

**Cross-platform OCI bundles:**
- The same bundle format works with runc, crun, youki, and other OCI runtimes
- This is the power of standardization: your config.json works everywhere

## Next

`03-run-basic.md` - Use runc to actually run a container from your OCI bundle
