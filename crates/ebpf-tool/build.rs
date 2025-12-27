//! Build script for ebpf-tool
//!
//! This script compiles eBPF programs from the sibling crate `ebpf-tool-ebpf`
//! using the Aya framework (pure Rust eBPF, no C toolchain required).
//!
//! # How it Works
//!
//! 1. Locates the `ebpf-tool-ebpf` crate (../ebpf-tool-ebpf relative to this crate)
//! 2. Invokes cargo to compile it with the `bpfel-unknown-none` target (little-endian BPF)
//! 3. Places compiled .o files in OUT_DIR for inclusion via `include_bytes_aligned!`
//! 4. Emits cargo directives so the build reruns when eBPF source changes
//!
//! # Prerequisites
//!
//! - Rust nightly toolchain (for `-Z build-std`)
//! - `bpf-linker` installed: `cargo install bpf-linker`
//! - `rust-src` component: `rustup component add rust-src`
//!
//! # References
//!
//! - Aya documentation: https://aya-rs.dev/book/
//! - aya-build crate: https://docs.rs/aya-build
//! - BPF target triples: bpfel-unknown-none (little-endian), bpfeb-unknown-none (big-endian)

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // TODO: This build script currently uses a manual cargo invocation approach.
    // Once aya-build stabilizes, learners should migrate to using:
    //   aya_build::build_ebpf([&manifest_path])
    //
    // The manual approach is shown here to help learners understand what happens
    // under the hood when compiling eBPF programs.

    // Determine paths
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let ebpf_crate_dir = PathBuf::from(&manifest_dir).join("../ebpf-tool-ebpf");

    // Check if the eBPF crate exists
    // TODO: In lesson 01, learners will create the ebpf-tool-ebpf crate.
    // Until then, this build script will skip compilation gracefully.
    if !ebpf_crate_dir.exists() {
        println!(
            "cargo:warning=ebpf-tool-ebpf crate not found at {:?}",
            ebpf_crate_dir
        );
        println!("cargo:warning=eBPF programs will not be compiled until the crate is created");
        println!("cargo:warning=See: docs/04-ebpf/01-hello-kprobe.md for instructions");

        // Create a placeholder file so the main crate can still compile
        // This allows the `check` subcommand to work before eBPF programs exist
        create_placeholder(&out_dir);
        return;
    }

    // Tell cargo to rerun this build script if the eBPF crate changes
    println!("cargo:rerun-if-changed={}", ebpf_crate_dir.display());

    // Watch all Rust source files in the eBPF crate
    if let Ok(entries) = fs::read_dir(ebpf_crate_dir.join("src")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    // Build the eBPF programs
    //
    // TODO: Learners should understand these key aspects:
    //
    // 1. TARGET: We use `bpfel-unknown-none` for little-endian BPF bytecode.
    //    Most x86_64 and ARM systems are little-endian. Use `bpfeb-unknown-none`
    //    for big-endian systems (rare).
    //
    // 2. BUILD-STD: eBPF programs use `#![no_std]` and need core recompiled
    //    for the BPF target. The `-Z build-std=core` flag handles this.
    //
    // 3. NIGHTLY: The `build-std` feature requires nightly Rust.
    //
    // 4. PROFILE: Release builds are recommended to optimize code size and
    //    avoid hitting BPF verifier limits on instruction count.

    let target = "bpfel-unknown-none";

    // Determine the cargo profile to use for eBPF compilation
    // Note: We always use release for eBPF to avoid verifier issues with debug builds.
    // Debug builds often exceed the BPF verifier's instruction limit.
    //
    // TODO: Learners can experiment with debug builds once they understand
    // verifier limits. Change this to "debug" to see the difference.
    let ebpf_profile = "release";

    // Get the userspace build profile (for reference/logging)
    let _userspace_profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Build command for the eBPF crate
    //
    // TODO: This invokes cargo manually. The aya-build crate provides a cleaner API:
    //   aya_build::build_ebpf([&manifest_path]).expect("failed to build eBPF programs");
    //
    // However, understanding the manual process helps learners debug issues.
    let status = Command::new("cargo")
        .current_dir(&ebpf_crate_dir)
        // Use nightly for build-std support
        .arg("+nightly")
        .arg("build")
        .arg("--target")
        .arg(target)
        // build-std recompiles core for the BPF target
        .arg("-Z")
        .arg("build-std=core")
        .arg("--release")
        // Enable BTF (BPF Type Format) for CO-RE (Compile Once, Run Everywhere)
        // This allows programs to work across different kernel versions
        .env("RUSTFLAGS", "-C debuginfo=2 -C link-arg=--btf")
        // Set CARGO_TARGET_DIR to place output in our OUT_DIR
        // This avoids polluting the eBPF crate's target directory
        .env("CARGO_TARGET_DIR", format!("{}/ebpf-target", out_dir))
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully compiled eBPF programs");

            // Copy the compiled eBPF object to a well-known location in OUT_DIR
            // The main crate will use include_bytes_aligned! to embed it
            //
            // TODO: Learners should update this path when adding new eBPF programs.
            // Each program binary is named after the crate (ebpf-tool-ebpf).
            let ebpf_binary = PathBuf::from(&out_dir)
                .join("ebpf-target")
                .join(target)
                .join(ebpf_profile)
                .join("ebpf-tool-ebpf");

            if ebpf_binary.exists() {
                let dest = PathBuf::from(&out_dir).join("ebpf-tool-ebpf");
                fs::copy(&ebpf_binary, &dest).expect("Failed to copy eBPF binary");
                println!("cargo:warning=eBPF binary available at: {}", dest.display());
            } else {
                println!(
                    "cargo:warning=eBPF binary not found at expected location: {}",
                    ebpf_binary.display()
                );
                create_placeholder(&out_dir);
            }
        }
        Ok(status) => {
            println!(
                "cargo:warning=eBPF compilation failed with status: {}",
                status
            );
            println!("cargo:warning=Ensure you have:");
            println!("cargo:warning=  1. Rust nightly: rustup install nightly");
            println!(
                "cargo:warning=  2. rust-src: rustup component add rust-src --toolchain nightly"
            );
            println!("cargo:warning=  3. bpf-linker: cargo install bpf-linker");
            create_placeholder(&out_dir);
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to run cargo for eBPF compilation: {}",
                e
            );
            println!("cargo:warning=Creating placeholder - eBPF programs will not be available");
            create_placeholder(&out_dir);
        }
    }

    // Export the OUT_DIR path so main.rs can find the compiled eBPF programs
    println!("cargo:rustc-env=EBPF_OUT_DIR={}", out_dir);
}

/// Create a placeholder file when eBPF compilation is not available.
///
/// This allows the userspace CLI to compile even when:
/// - The ebpf-tool-ebpf crate doesn't exist yet
/// - The BPF toolchain isn't installed
/// - Compilation fails for some reason
///
/// The CLI's `check` subcommand can detect this and warn the user.
fn create_placeholder(out_dir: &str) {
    let placeholder_path = PathBuf::from(out_dir).join("ebpf-tool-ebpf");

    // Write a minimal placeholder that will cause a clear error if loaded
    // We use an empty file - Aya will fail gracefully when trying to load it
    fs::write(&placeholder_path, b"").expect("Failed to create placeholder");

    println!(
        "cargo:warning=Created placeholder at: {}",
        placeholder_path.display()
    );
    println!("cargo:warning=The `ebpf-tool check` command will detect missing eBPF programs");
}

// =============================================================================
// Build Script Learning Notes
// =============================================================================
//
// ## Understanding build.rs for eBPF
//
// This build script demonstrates the key concepts for compiling eBPF programs
// as part of a Rust project. Here's what learners should understand:
//
// ### Why a Build Script?
//
// eBPF programs run inside the Linux kernel, not in userspace. They must be:
// 1. Compiled to BPF bytecode (not native machine code)
// 2. Embedded into the userspace binary (via include_bytes!)
// 3. Loaded into the kernel at runtime (via the bpf() syscall)
//
// The build script handles steps 1-2. The main.rs handles step 3.
//
// ### The BPF Target Triple
//
// - `bpfel-unknown-none`: Little-endian BPF, no OS, no std library
// - `bpfeb-unknown-none`: Big-endian BPF (rarely used)
//
// The "el" suffix stands for "endian little". Most modern systems are
// little-endian, so this is the default choice.
//
// ### no_std and build-std
//
// eBPF programs cannot use the Rust standard library because:
// - No heap allocator (only 512 bytes of stack!)
// - No system calls (eBPF has its own helper functions)
// - No threads, no I/O, no networking APIs
//
// They can only use `core` (the dependency-free subset of std).
// The `-Z build-std=core` flag tells cargo to recompile core for BPF.
//
// ### BTF (BPF Type Format)
//
// BTF enables CO-RE (Compile Once, Run Everywhere):
// - Embeds type information in the compiled program
// - Allows Aya to relocate struct field offsets at load time
// - Programs compiled on one kernel version work on others
//
// The `--btf` linker flag and `debuginfo=2` enable this.
//
// ### The bpf-linker
//
// Normal linkers (like `ld`) don't understand BPF. The `bpf-linker` is a
// special LLVM-based linker that:
// - Links BPF object files
// - Generates BTF information
// - Produces the final .o file for kernel loading
//
// Install it with: `cargo install bpf-linker`
//
// =============================================================================
