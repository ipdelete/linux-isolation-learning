# Bug: `03-run-basic.md` suggests a Docker-style `-- <cmd>` override for `runc run`

## Summary
`docs/03-runc/03-run-basic.md` troubleshooting includes `sudo runc run --bundle ./my-bundle test -- /bin/ls /`, which is a Docker-style command override pattern and is not supported by `runc run`.

## Location
- `docs/03-runc/03-run-basic.md` (Troubleshooting section; “Container exits immediately” / “Try running a command that produces output”)

## Problem
`runc run` runs the command specified in `config.json` (`process.args`). It does not take `-- /bin/ls /` to override the configured command.

## Steps to reproduce
1. Follow `docs/03-runc/03-run-basic.md`.
2. Run the suggested command `sudo runc run --bundle ./my-bundle test -- /bin/ls /`.

## Expected
Troubleshooting commands should be valid `runc` usage.

## Actual
The suggested command does not work as written.

## Suggested fix
- Replace with instructions to edit `config.json` `process.args` to `["/bin/ls", "/"]` (or `["sh","-c","ls /"]`) and rerun.
- Alternatively, recommend `runc create` + `runc start` + `runc exec` for interactive debugging.

