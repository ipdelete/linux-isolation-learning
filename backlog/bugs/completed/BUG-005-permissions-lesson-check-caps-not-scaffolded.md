# Bug: Permissions lesson describes `check-caps` subcommand not scaffolded in CLI

## Summary
The permissions lesson instructs adding a `check-caps` subcommand and tests, but the current `ns-tool` CLI (`Command` enum in `src/main.rs`) has no `CheckCaps` variant (only `caps_test.rs` exists as TODO tests).

## Location
- `docs/00-foundations/04-permissions-and-sudo.md`
- `crates/ns-tool/src/main.rs`
- `crates/ns-tool/tests/caps_test.rs`

## Problem
Following the doc as-written leads to a mismatch: tests are TODOs and the CLI doesn’t have a stub subcommand to implement, breaking the “scaffold → tests → implementation” teaching loop.

## Expected
Either:
- `CheckCaps` exists as a `todo!()` match arm in `crates/ns-tool/src/main.rs`, matching the lesson, or
- the lesson is rewritten to match the repo’s current scaffolding (and use existing TODOs).

## Actual
`caps_test.rs` exists but is all `todo!()`, and there is no `check-caps` subcommand in the CLI.

## Suggested fix
- Add `Command::CheckCaps => todo!(...)` with lesson/test pointers, or adjust the lesson to the repo’s actual state.
- Fix “expected failure output” text in the lesson: an invalid subcommand does not produce “no tests to run”.

