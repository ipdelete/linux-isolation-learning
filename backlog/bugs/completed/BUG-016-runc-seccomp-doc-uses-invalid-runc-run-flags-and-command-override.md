# Bug: Seccomp lesson uses invalid `runc run` flags and Docker-style command override patterns

## Summary
`docs/03-runc/05-seccomp.md` uses `runc run --rm ...` and also passes commands after the container ID (e.g. `runc run ... sh -c '...'`). These patterns are not supported by `runc run` and will cause learners to fail when following the lesson.

## Location
- `docs/03-runc/05-seccomp.md`

## Problem
The lesson shows several invocations like:
- `sudo runc run --rm <id> ...`
- `sudo runc run --rm <id> sh -c '...'`

`--rm` is not a `runc run` flag, and `runc run` does not accept a command override after the container ID the way Docker does. With `runc`, the executed command is defined by `config.json` (`process.args`) (or by using `runc exec` after start).

## Steps to reproduce
1. Follow `docs/03-runc/05-seccomp.md` exactly.
2. Run any of the `sudo runc run --rm ...` commands.

## Expected
Commands in the lesson should be valid for `runc` and runnable as written.

## Actual
Commands fail with errors due to unsupported flags/arguments.

## Suggested fix
- Remove `--rm` usage; replace with explicit cleanup (`runc delete` after exit) or use `runc delete -f <id>` where appropriate.
- Avoid Docker-style command overrides; instead:
  - update `config.json` `process.args` to run the test command, or
  - run the container normally and then use `runc exec` to run commands inside it.

