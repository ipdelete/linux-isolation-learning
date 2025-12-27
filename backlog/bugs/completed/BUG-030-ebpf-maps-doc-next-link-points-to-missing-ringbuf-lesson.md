# Bug: `03-maps.md` “Next” points to a non-existent `04-ringbuf.md`

## Summary
At the end of `docs/04-ebpf/03-maps.md`, the “Next” section points to `04-ringbuf.md`, but there is no such file in `docs/04-ebpf/`. The next actual lesson file is `04-perf-events.md`.

## Location
- `docs/04-ebpf/03-maps.md` (`## Next` section)

## Problem
Broken navigation link; learners following the tutorial sequence hit a dead end.

## Steps to reproduce
1. Open `docs/04-ebpf/03-maps.md`.
2. Scroll to `## Next`.
3. Try to open `docs/04-ebpf/04-ringbuf.md`.

## Expected
“Next” points to an existing lesson file.

## Actual
The referenced file does not exist.

## Suggested fix
- Update “Next” to point to `docs/04-ebpf/04-perf-events.md`, or add the missing `04-ringbuf.md` lesson if that’s the intended sequence.

