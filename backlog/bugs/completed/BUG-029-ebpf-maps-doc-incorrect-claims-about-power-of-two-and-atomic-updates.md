# Bug: `03-maps.md` includes incorrect claims about map sizing and safe increments

## Summary
`docs/04-ebpf/03-maps.md` states that `MAX_MAP_ENTRIES` “must be a power of 2” and implies the naive `get` + `insert` pattern is safe for concurrent incrementing. In this repo, `MAX_MAP_ENTRIES` is `10240` (not a power of 2), and the documented increment logic can lose updates under concurrency.

## Location
- `docs/04-ebpf/03-maps.md` (Map sizing section and HashMap increment example)
- `crates/ebpf-tool-common/src/lib.rs` (`MAX_MAP_ENTRIES: u32 = 10240;`)

## Problem
The doc’s statements are misleading:
- The “must be a power of 2” claim conflicts with the repo constant.
- `count = get(); insert(count + 1)` is not an atomic increment and can drop increments when multiple CPUs update the same key.

## Steps to reproduce
1. Follow the doc’s `get`/`insert` increment approach for a hot key.
2. Generate load from multiple CPUs.
3. Observe counts can be lower than expected due to lost updates.

## Expected
Docs should be accurate about:
- Map sizing constraints, and
- Correct concurrency patterns for counters (e.g., per-CPU maps, or kernel-side atomic add patterns where applicable).

## Actual
Docs assert incorrect constraints and suggest an unsafe counter update pattern.

## Suggested fix
- Remove/correct the “power of 2” requirement.
- Update counter guidance to use `PerCpuHashMap` / per-CPU counters, or another concurrency-safe counting pattern consistent with Aya and the kernel map type.

