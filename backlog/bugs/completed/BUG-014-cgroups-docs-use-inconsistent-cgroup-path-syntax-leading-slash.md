# Bug: Cgroups docs inconsistently use leading “/” in cgroup-tool path arguments

## Summary
Some examples in the cgroups docs pass a leading slash in the cgroup path argument (e.g. `/io-test`), while other lessons describe/assume the CLI uses a relative path (e.g. `io-test`). This inconsistency can cause confusion or failures depending on how the CLI is implemented.

## Location
- `docs/02-cgroups/04-io.md` (examples include `io-max /io-test ...`)
- (Cross-cutting) `docs/02-cgroups/*.md` generally describe cgroup paths as relative to `/sys/fs/cgroup`

## Problem
The docs do not clearly define whether the cgroup path argument:
- must be relative (recommended for CLIs: `my-cgroup`, `parent/child`), or
- may optionally begin with `/` (interpreted as a cgroup-relative absolute path like `/my-cgroup`), or
- may be a full filesystem path (e.g. `/sys/fs/cgroup/my-cgroup`).

If the implementation only supports relative paths, docs that use `/io-test` will mislead learners.

## Steps to reproduce
1. Read `docs/02-cgroups/04-io.md` and follow manual verification steps using `/io-test`.
2. Compare with other lessons’ text stating the tool expects a path relative to `/sys/fs/cgroup`.

## Expected
Docs should use one consistent convention and state it explicitly (with 1–2 examples).

## Actual
Mixed usage across lessons, with some examples including a leading slash.

## Suggested fix
- Pick one convention (prefer: relative paths like `io-test` and `parent/child`) and update examples accordingly.
- If supporting both, document the accepted forms clearly (and ensure all subcommands behave consistently).

