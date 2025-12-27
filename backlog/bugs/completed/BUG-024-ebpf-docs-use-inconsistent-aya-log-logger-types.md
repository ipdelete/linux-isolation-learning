# Bug: eBPF docs inconsistently reference `aya_log::EbpfLogger` vs `aya_log::BpfLogger`

## Summary
Across `docs/04-ebpf/*`, some lessons use `aya_log::EbpfLogger::init(&mut bpf)` while others use `aya_log::BpfLogger::init(&mut bpf)`. At least one of these is incorrect for the `aya-log` crate version used in this repo, causing copy/paste snippets to fail to compile.

## Location
- `docs/04-ebpf/01-hello-kprobe.md` (`aya_log::EbpfLogger`)
- `docs/04-ebpf/02-reading-data.md` (`aya_log::EbpfLogger`)
- `docs/04-ebpf/05-uprobes.md` (`aya_log::BpfLogger`)
- `docs/04-ebpf/07-perf-sampling.md` (`aya_log::EbpfLogger`)
- `docs/04-ebpf/08-combining.md` (`aya_log::BpfLogger`)

## Problem
The docs are not internally consistent about the correct logger type for userspace initialization.

## Steps to reproduce
1. Copy a snippet from one lesson into `crates/ebpf-tool/src/main.rs`.
2. `cargo build -p ebpf-tool`.
3. Observe compilation errors if the logger type does not exist for the configured `aya-log` version.

## Expected
Docs consistently use the correct `aya-log` userspace initialization API for this repository.

## Actual
Docs mix two different API spellings, leading to compilation failures depending on which snippet is used.

## Suggested fix
- Standardize on the correct logger type and init call for `aya-log = 0.2` (workspace dependency), and update all docs to match.

## Resolution
**Fixed**: All instances of `aya_log::EbpfLogger::init()` have been updated to `aya_log::BpfLogger::init()`. This is the correct type for aya-log 0.2.x.

### Files Changed:
- ✓ `docs/04-ebpf/01-hello-kprobe.md` - Updated 2 instances (line 358, line 647)
- ✓ `docs/04-ebpf/02-reading-data.md` - Updated 1 instance (line 305)
- ✓ `docs/04-ebpf/06-tracepoints.md` - Updated 2 instances (line 404, line 576)
- ✓ `docs/04-ebpf/07-perf-sampling.md` - Updated 1 instance (line 475)
- ✓ `docs/04-ebpf/05-uprobes.md` - Already correct (uses BpfLogger)
- ✓ `docs/04-ebpf/08-combining.md` - Already correct (uses BpfLogger)

### Why `BpfLogger` is Correct:
`BpfLogger` is the standard logger type exposed by aya-log 0.2.x for initializing the logging system in userspace eBPF programs. It provides the interface for receiving and processing log messages sent from eBPF programs via perf buffers.
