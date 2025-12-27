# Progress Log: Bug Fix Session - Batch 6 (eBPF Build System)

**Date**: 2025-12-27
**Session Time**: 23:12 UTC
**Repository**: linux-isolation-learning
**Branch**: review-docs

## Overview

Completed **Batch 6** of the comprehensive bug fix campaign, addressing 2 coupled eBPF build system bugs. These were critical infrastructure issues affecting all 8 eBPF lesson files, where documentation referenced non-existent build commands and incorrect file paths. This brings the cumulative total to **26/30 bugs fixed (86.7% completion)**.

### Session Results
- **Bugs Fixed**: 2 (Batch 6 - coupled build system issues)
- **Total Fixed (Cumulative)**: 26/30 (86.7% completion)
- **Documentation Files Updated**: 8 eBPF lesson files
- **Build Verification**: ✅ Success (cargo build --all)
- **Commits**: Pending (changes ready for commit)

---

## What We Built

This session didn't create new features but fixed fundamental infrastructure issues that prevented learners from successfully building and running eBPF programs. The fixes ensure that all eBPF tutorial lessons now provide accurate, working build instructions.

### BUG-021: Fixed Non-Existent `cargo xtask build-ebpf` References

**Issue**: All 8 eBPF lesson files referenced `cargo xtask build-ebpf`, a command that doesn't exist in this project.

**Root Cause**: Documentation was written assuming a cargo-xtask workflow, but the project actually uses a `build.rs` script that automatically compiles eBPF programs when building the userspace tool.

**Solution**: Replaced all instances of `cargo xtask build-ebpf` with the correct command: `cargo build -p ebpf-tool`

**Impact**:
- Learners can now successfully build eBPF programs
- Build process is simpler (one command instead of two)
- Matches actual project architecture

**Changes by file**:

1. **docs/04-ebpf/01-hello-kprobe.md** (4 locations)
   - Line 323: Build command in "Step 2: Build the eBPF Program"
   - Line 440: Build command in "Step 5: Run Tests"
   - Line 540: Build command in error section "Common Errors #1"
   - Added explanation of build.rs workflow

2. **docs/04-ebpf/02-reading-data.md** (1 location)
   - Line 366: Build command in "Step 3: Build Everything"

3. **docs/04-ebpf/03-maps.md** (1 location)
   - Line 514: Build command in "Step 4: Build the eBPF Program"

4. **docs/04-ebpf/04-perf-events.md** (1 location)
   - Line 476: Build command in "Step 5: Build and verify"

5. **docs/04-ebpf/05-uprobes.md** (2 locations)
   - Line 332: Build command in "Build the eBPF program" section
   - Line 445: Build command in "Step 4: Build and Run Tests"

6. **docs/04-ebpf/06-tracepoints.md** (1 location)
   - Line 459: Build command in "Part 3: Build the eBPF Program"

7. **docs/04-ebpf/07-perf-sampling.md** (1 location)
   - Line 601: Build command in "Build and Run Tests"

8. **docs/04-ebpf/08-combining.md** (1 location)
   - Line 1051: Build command in "Part 3: Build and Test"

### BUG-022: Fixed Incorrect `include_bytes!` Paths

**Issue**: Documentation showed hardcoded paths like `../../target/bpfel-unknown-none/release/...` that don't match the actual build system output.

**Root Cause**: The project's `build.rs` script compiles eBPF programs and places them in `OUT_DIR`, not in the hardcoded target paths shown in docs.

**Solution**: Updated all `include_bytes_aligned!` macros to use:
```rust
include_bytes_aligned!(concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf"))
```

**Impact**:
- eBPF programs can now be loaded correctly at runtime
- Paths are now dynamic and work regardless of build configuration
- Matches actual build.rs behavior

**Changes by file**:

1. **docs/04-ebpf/01-hello-kprobe.md:352**
   - Updated include_bytes_aligned! path in userspace CLI implementation
   - Added comment explaining build.rs places files in OUT_DIR

2. **docs/04-ebpf/02-reading-data.md:299**
   - Fixed path in kprobe command implementation

3. **docs/04-ebpf/03-maps.md:387**
   - Fixed path, added build.rs explanation comment

4. **docs/04-ebpf/04-perf-events.md:311**
   - Fixed path in perf event loader

5. **docs/04-ebpf/05-uprobes.md:365**
   - Fixed path, removed debug/release conditional (not needed with build.rs)

6. **docs/04-ebpf/06-tracepoints.md:397**
   - Fixed path in tracepoint loader

7. **docs/04-ebpf/07-perf-sampling.md:469**
   - Fixed path in CPU sampling implementation

8. **docs/04-ebpf/08-combining.md**: Already correct (no change needed)

---

## Files Created/Modified

### Files Created
- **None** (Batch 6 only modified existing documentation)

### Files Modified (8 documentation files)

All modifications were in `docs/04-ebpf/`:

1. **01-hello-kprobe.md**
   - Build commands: Lines 323, 440, 540
   - Include path: Line 352
   - Added build.rs workflow explanation

2. **02-reading-data.md**
   - Build command: Line 366
   - Include path: Line 299

3. **03-maps.md**
   - Build command: Line 514
   - Include path: Line 387

4. **04-perf-events.md**
   - Build command: Line 476
   - Include path: Line 311

5. **05-uprobes.md**
   - Build commands: Lines 332, 445
   - Include path: Line 365

6. **06-tracepoints.md**
   - Build command: Line 459
   - Include path: Line 397

7. **07-perf-sampling.md**
   - Build command: Line 601
   - Include path: Line 469

8. **08-combining.md**
   - Build command: Line 1051

### Bug Reports (moved to completed/)
- `BUG-021-ebpf-docs-reference-missing-xtask-build-ebpf.md`
- `BUG-022-ebpf-docs-include-bytes-paths-dont-match-build-rs-out-dir.md`

---

## Key Concepts Explained

### The Actual eBPF Build System

The project uses a **build.rs script** pattern, not cargo-xtask:

```
User runs:
  cargo build -p ebpf-tool
       ↓
  build.rs executes BEFORE compiling userspace code
       ↓
  build.rs compiles: crates/ebpf-tool-ebpf/
       ↓
  eBPF bytecode placed in: $OUT_DIR/ebpf-tool-ebpf
       ↓
  Userspace code embeds bytecode via:
    include_bytes_aligned!(concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf"))
       ↓
  Final binary contains both userspace + eBPF code
```

**Why this approach?**
1. **Single command**: `cargo build` handles everything
2. **Correct dependencies**: Cargo knows to rebuild eBPF when it changes
3. **Standard Rust**: No custom tools (xtask) required
4. **OUT_DIR**: Environment variable provided by Cargo, always correct

### Build.rs Workflow

The `crates/ebpf-tool/build.rs` script:

1. **Detects eBPF source**: Locates `ebpf-tool-ebpf` crate
2. **Invokes cargo**: Runs cargo build with eBPF-specific flags:
   - `--target bpfel-unknown-none` (BPF target)
   - `-Z build-std=core` (rebuild core for BPF)
   - `--release` (always release mode for eBPF)
3. **Copies output**: Places compiled `.o` file in `OUT_DIR`
4. **Sets rerun triggers**: Tells Cargo when to rebuild

**Error handling**: If eBPF compilation fails (missing tools), build.rs creates a placeholder file and emits warnings, allowing the project to build for documentation/testing purposes.

### include_bytes_aligned! Macro

This Aya-provided macro:
- Embeds binary data (eBPF bytecode) directly into the Rust binary
- Ensures 8-byte alignment required by BPF loader
- Uses `concat!` and `env!` to build path at compile time

**Before (incorrect)**:
```rust
include_bytes_aligned!("../../target/bpfel-unknown-none/release/ebpf-tool-ebpf")
```
❌ Problems:
- Hardcoded relative path breaks from different working directories
- Assumes release build (doesn't work with debug)
- Doesn't account for custom target directories

**After (correct)**:
```rust
include_bytes_aligned!(concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf"))
```
✅ Advantages:
- `OUT_DIR` is always correct (Cargo sets it)
- Works from any working directory
- Works in debug and release builds
- Works with custom target directories

---

## How to Use / Verification

### Build the Project

```bash
# Build all crates (eBPF compilation happens automatically via build.rs)
cargo build --all

# Expected output:
#   Compiling ebpf-tool-ebpf...
#   Compiling ebpf-tool...
#   Finished `dev` profile [unoptimized + debuginfo] target(s)

# If eBPF tools not installed, you'll see warnings but build continues:
#   warning: ebpf-tool@0.1.0: eBPF compilation failed
#   warning: ebpf-tool@0.1.0: Ensure you have:
#   warning: ebpf-tool@0.1.0:   1. Rust nightly
#   warning: ebpf-tool@0.1.0:   2. rust-src component
#   warning: ebpf-tool@0.1.0:   3. bpf-linker
```

### Build Just eBPF Tool

```bash
# Single command builds both eBPF and userspace
cargo build -p ebpf-tool

# For release build (recommended for eBPF):
cargo build -p ebpf-tool --release
```

### Verify Documentation Changes

```bash
# Check that no references to xtask remain
grep -r "cargo xtask build-ebpf" docs/04-ebpf/
# Should return nothing

# Check that all include_bytes use OUT_DIR
grep -n "include_bytes_aligned" docs/04-ebpf/*.md
# All should show concat!(env!("OUT_DIR"), ...)

# Verify bug reports moved
ls backlog/bugs/BUG-021* backlog/bugs/BUG-022* 2>/dev/null
# Should return: No such file or directory

ls backlog/bugs/completed/BUG-021* backlog/bugs/completed/BUG-022*
# Should list both bug reports
```

### Test a Lesson (Example: 01-hello-kprobe)

```bash
# Follow lesson instructions (now they work!)
cargo build -p ebpf-tool

# Run tests (requires root for eBPF)
sudo -E cargo test -p ebpf-tool --test kprobe_test

# Run the tool
sudo cargo run -p ebpf-tool -- kprobe do_sys_openat2 -d 5
```

---

## Technical Notes

### No Compilation Errors Introduced

All changes were to documentation files and comments only:
- No Rust source code logic modified
- Build verified: `cargo build --all` succeeded
- Only warnings are pre-existing (unreachable code in todo!() stubs)

### Build System Dependencies

For full eBPF compilation (optional for this tutorial), users need:

1. **Rust nightly toolchain**:
   ```bash
   rustup install nightly
   ```

2. **rust-src component** (for -Z build-std):
   ```bash
   rustup component add rust-src --toolchain nightly
   ```

3. **bpf-linker** (for linking eBPF objects):
   ```bash
   cargo install bpf-linker
   ```

**Important**: If these are missing, build.rs creates a placeholder and continues. Tests will skip eBPF-dependent tests with warnings.

### Documentation Consistency

After these fixes, all eBPF lessons now:
- Use the same build command: `cargo build -p ebpf-tool`
- Use the same include path pattern: `concat!(env!("OUT_DIR"), "/ebpf-tool-ebpf")`
- Explain the build.rs workflow where relevant
- Provide accurate troubleshooting steps

### Why These Bugs Were "Coupled"

BUG-021 and BUG-022 were fixed together because:
1. Both stem from misunderstanding the actual build system
2. Fixing the build command without fixing paths would leave broken code
3. Both affect the same 8 lesson files
4. Testing one fix requires the other to work

This is an example of **dependency coupling** in bug fixes - attempting to fix them separately would result in incomplete solutions.

---

## Remaining Issues After Batch 6

The project now has **4 remaining bugs** (down from 30 original):

**All remaining bugs are eBPF content/code issues** (Batch 7 candidates):

1. **BUG-023**: eBPF hello-kprobe creates nested tokio runtime
   - Severity: Medium
   - Type: Code bug in lesson example
   - Scope: 1 file (01-hello-kprobe.md)

2. **BUG-025**: eBPF docs need Bytes/BytesMut dependency
   - Severity: Low
   - Type: Missing dependency documentation
   - Scope: Multiple eBPF lessons

3. **BUG-026**: eBPF reading-data contradicts ignore flags
   - Severity: Medium
   - Type: Inconsistent documentation
   - Scope: 1 file (02-reading-data.md)

4. **BUG-027**: eBPF reading-data misuses syscall_nr field
   - Severity: Medium
   - Type: Incorrect field usage in example
   - Scope: 1 file (02-reading-data.md)

**These can be fixed in parallel** as they are independent issues affecting different aspects of the eBPF lessons.

---

## Next Steps (Not Implemented)

### Batch 7: eBPF Content Fixes (Recommended Next)

**Approach**: Fix all 4 remaining bugs in parallel (independent, non-conflicting)

**Estimated time**: 15-20 minutes (4 parallel agents or sequential)

**Bug breakdown**:
1. **BUG-023** (tokio runtime):
   - Find nested runtime creation in async context
   - Fix by using `#[tokio::main]` correctly or removing runtime
   - Test with cargo build

2. **BUG-025** (Bytes dependency):
   - Add `bytes = "1"` to Cargo.toml if missing
   - Update relevant lesson docs to mention dependency
   - Verify with cargo check

3. **BUG-026** (ignore flags contradiction):
   - Review ignore/ignored flag usage in reading-data lesson
   - Clarify documentation to remove contradiction
   - Ensure code examples match explanation

4. **BUG-027** (syscall_nr field):
   - Correct field usage in reading-data examples
   - Update to use proper field from tracepoint context
   - Verify against actual tracepoint format file

### Suggested Continuous Integration Improvements

To prevent future build system documentation drift:

1. **CI: Verify build commands in docs**:
   ```bash
   # Extract build commands from markdown
   grep -h "cargo.*build" docs/**/*.md | sort -u > commands.txt

   # Verify only approved commands exist
   # Fail if "cargo xtask" appears in any lesson
   ```

2. **CI: Verify include_bytes paths**:
   ```bash
   # Check that no hardcoded paths remain
   grep -r "include_bytes.*target/" docs/ && exit 1

   # Ensure all use OUT_DIR pattern
   grep -r "include_bytes.*OUT_DIR" docs/ || exit 1
   ```

3. **Documentation linting**:
   - Use markdownlint to enforce consistent code block formatting
   - Validate that code blocks with `bash` tag contain valid commands
   - Check that file paths in docs actually exist

4. **Build.rs validation**:
   - Ensure build.rs always outputs to OUT_DIR
   - Add error messages if eBPF compilation fails
   - Create placeholder files for graceful degradation

---

## Repository Information

**Repository**: linux-isolation-learning
**URL**: /workspaces/linux-isolation-learning/
**Branch**: review-docs

### Current Git State

```bash
$ git status --short
 D backlog/bugs/BUG-021-ebpf-docs-reference-missing-xtask-build-ebpf.md
 D backlog/bugs/BUG-022-ebpf-docs-include-bytes-paths-dont-match-build-rs-out-dir.md
 M docs/04-ebpf/01-hello-kprobe.md
 M docs/04-ebpf/02-reading-data.md
 M docs/04-ebpf/03-maps.md
 M docs/04-ebpf/04-perf-events.md
 M docs/04-ebpf/05-uprobes.md
 M docs/04-ebpf/06-tracepoints.md
 M docs/04-ebpf/07-perf-sampling.md
 M docs/04-ebpf/08-combining.md
?? backlog/bugs/completed/BUG-021-ebpf-docs-reference-missing-xtask-build-ebpf.md
?? backlog/bugs/completed/BUG-022-ebpf-docs-include-bytes-paths-dont-match-build-rs-out-dir.md
```

### Overall Campaign Progress

```
Batch 1-2: 10 bugs fixed (foundation, namespaces, basic issues)
Batch 3-4: 10 bugs fixed (deduplication, scaffolding, runc commands)
Batch 5:    4 bugs fixed (quick wins - rootfs, refs, links)
Batch 6:    2 bugs fixed (eBPF build system)
────────────────────────────────────────────────────────────
Total:     26/30 bugs fixed (86.7% completion)
Remaining:  4 bugs (all eBPF content issues)
```

### Suggested Commit Message

```
docs: fix eBPF build system references (batch 6)

Fixed two coupled build system bugs affecting all 8 eBPF lesson files:

BUG-021: Replace non-existent 'cargo xtask build-ebpf' with correct command
- Changed all references to: cargo build -p ebpf-tool
- Added build.rs workflow explanations where relevant
- Affected: 8 files, 12 locations

BUG-022: Fix include_bytes paths to use OUT_DIR from build.rs
- Updated all include_bytes_aligned! to use concat!(env!("OUT_DIR"), ...)
- Removed hardcoded paths like ../../target/bpfel-unknown-none/...
- Affected: 7 files, 7 locations

The project uses build.rs to automatically compile eBPF programs when
building the userspace tool. Documentation now accurately reflects this
architecture.

Verified: cargo build --all succeeds with no new errors.

Progress: 26/30 bugs fixed (86.7%)
```

---

## Session Summary Statistics

| Metric | Value |
|--------|-------|
| Bugs Fixed (This Session) | 2 |
| Total Bugs Fixed (Cumulative) | 26 of 30 (86.7%) |
| Documentation Files Modified | 8 |
| Build Command Fixes | 12 locations |
| Include Path Fixes | 7 locations |
| Bug Reports Moved to Completed | 2 |
| Wall-Clock Time | ~15 minutes |
| Build Status | ✅ Success (no errors) |
| Code Quality | ✅ No regressions |
| Ready for Commit | ✅ Yes |

---

**Session Status**: ✅ COMPLETE (Batch 6)
**Next Session**: Ready for Batch 7 (eBPF content fixes, ~15-20 minutes estimated)
**Overall Campaign**: 86.7% complete, 4 bugs remaining
