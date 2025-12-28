# Lean Restructure Plan

Goal: Add an approachable fast-track path alongside existing detailed tutorials.

## Validated Format

Prototype proved 86% reduction (2,067 → 287 lines for 3 lessons). See `docs/prototype/`.

```
# Title (10 min)
## What you'll build - one sentence
## The test - 10-20 lines
## The implementation - 10-30 lines
## Run it - commands + expected output
## What just happened - 2-3 sentences
## Next: [link]
```

## Fast Track Lessons (10 total)

New `docs/fast-track/` directory. Uses existing crates.

### Namespaces (4 lessons)
- [x] `01-pid-namespace.md` — process isolation
- [x] `02-mount-namespace.md` — filesystem isolation
- [x] `03-network-namespace.md` — network + veth basics
- [x] `04-combine.md` — mini-container

### Cgroups (3 lessons)
- [x] `05-cgroup-basics.md` — create/attach
- [x] `06-memory-limits.md` — memory constraints
- [x] `07-cpu-limits.md` — CPU constraints

### Container (2 lessons)
- [x] `08-oci-bundle.md` — OCI structure
- [x] `09-runc-run.md` — run with runc

### Observability (1 lesson)
- [x] `10-ebpf-tracing.md` — eBPF basics

## What Stays the Same

- All existing crates (`ns-tool`, `cgroup-tool`, etc.)
- All existing docs (`docs/00-foundations/`, `docs/01-namespaces/`, etc.)
- All existing backlog items and todos
- Existing test scaffolding

## What Changes

- New `docs/fast-track/` with 10 lean lessons
- README gets a "Quick Start" section pointing to fast-track
- Fast-track links to detailed docs: "Want more depth? See [full lesson](../01-namespaces/01-pid-namespace.md)"

## Execution

- [x] Move prototypes from `docs/prototype/` to `docs/fast-track/`
- [x] Write remaining 7 fast-track lessons
- [x] Add fast-track intro/README
- [x] Update main README with dual-path options

## Status: COMPLETE
