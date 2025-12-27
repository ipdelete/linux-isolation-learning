# Bug: `02-reading-data.md` contradicts itself about `#[ignore]` and `--ignored`

## Summary
`docs/04-ebpf/02-reading-data.md` instructs learners to remove `#[ignore]` from tests, but then tells them to run tests with `-- --ignored`, which only runs ignored tests. This is internally inconsistent and will cause confusion.

## Location
- `docs/04-ebpf/02-reading-data.md` (“Enable the Tests” section and the “Run Tests” commands)

## Problem
The doc suggests mutually exclusive actions:
- Remove `#[ignore]` attributes, and
- Run only ignored tests using `-- --ignored`.

## Steps to reproduce
1. Follow the doc: remove `#[ignore]` from the specified tests.
2. Run the provided command: `sudo -E cargo test -p ebpf-tool test_kprobe_reads -- --ignored`.
3. Observe the tests are not selected (because they are no longer ignored).

## Expected
Docs should either:
- Keep tests ignored and run them with `-- --ignored`, or
- Un-ignore the tests and run them normally (without `-- --ignored`).

## Actual
Docs mix both approaches.

## Suggested fix
- Pick one workflow and update the instructions and commands accordingly (prefer un-ignoring and running normally).

