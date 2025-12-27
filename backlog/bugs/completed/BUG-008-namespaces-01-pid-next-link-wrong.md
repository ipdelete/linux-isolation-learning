# Bug: `01-pid-namespace.md` "Next" points to a non-existent lesson file

## Summary
The "Next" section at the end of `docs/01-namespaces/01-pid-namespace.md` references `02-uts-namespace.md`, but that file does not exist in `docs/01-namespaces/`.

## Location
- `docs/01-namespaces/01-pid-namespace.md`

## Problem
Learners following the tutorial sequence will hit a broken link / incorrect next-step reference.

## Steps to reproduce
1. Open `docs/01-namespaces/01-pid-namespace.md`.
2. Scroll to the `## Next` section.
3. Observe the referenced filename.
4. List `docs/01-namespaces/` and confirm there is no `02-uts-namespace.md`.

## Expected
`## Next` should reference the actual next lesson in this section, likely:
- `docs/01-namespaces/02-unshare-vs-clone.md`, or
- `docs/01-namespaces/03-uts-ipc.md` if UTS/IPC is intended next.

## Actual
`## Next` references `02-uts-namespace.md`, which does not exist.

## Suggested fix
- Update the `## Next` line in `docs/01-namespaces/01-pid-namespace.md` to the correct filename and description consistent with the intended ordering.

