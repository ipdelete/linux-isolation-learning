# Bug: `03-maps.md` prereqs reference “Lesson 02 (Tracepoints)” but tracepoints are Lesson 06

## Summary
`docs/04-ebpf/03-maps.md` lists “Completed Lesson 02 (Tracepoints)” as a prereq, but tracepoints are covered in `docs/04-ebpf/06-tracepoints.md`. This is confusing and suggests a lesson order different from the actual file numbering.

## Location
- `docs/04-ebpf/03-maps.md` (Prereqs section)

## Problem
Incorrect lesson cross-reference; it undermines the intended sequence and makes it harder for learners to know what to do next.

## Steps to reproduce
1. Open `docs/04-ebpf/03-maps.md` and read the prereqs.
2. Try to find “Lesson 02: Tracepoints”.

## Expected
Prereqs match the actual lesson numbering and titles in `docs/04-ebpf/`.

## Actual
Prereqs reference a mismatched lesson title/number.

## Suggested fix
- Update prereqs to reference the correct prerequisite lessons (likely `02-reading-data.md` and/or `01-hello-kprobe.md`), and mention tracepoints only if actually required.

