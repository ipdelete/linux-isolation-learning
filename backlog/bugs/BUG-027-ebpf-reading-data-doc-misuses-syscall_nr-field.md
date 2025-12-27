# Bug: `02-reading-data.md` mixes syscall argument values into `SyscallEvent.syscall_nr`

## Summary
`docs/04-ebpf/02-reading-data.md` suggests populating `SyscallEvent.syscall_nr` with “function argument data” (e.g., `do_sys_openat2` arg0), but later example output treats the value as the syscall number (e.g., `257` for openat). This misrepresents the data model and will produce confusing/incorrect output.

## Location
- `docs/04-ebpf/02-reading-data.md` (eBPF implementation section describing reading args and assigning to `syscall_nr`)
- `docs/04-ebpf/02-reading-data.md` (manual verification “Expected output” showing `syscall_nr=257`)
- `crates/ebpf-tool-common/src/lib.rs` (`SyscallEvent` defines `syscall_nr` as “System call number”)

## Problem
The tutorial conflates “syscall number” with “arbitrary probed function argument”, while the shared type names the field specifically as a syscall number.

## Steps to reproduce
1. Implement `try_read_syscall_args()` as described and assign it into `SyscallEvent.syscall_nr`.
2. Print `syscall_nr` in userspace.
3. Observe values are not syscall numbers (they depend on the probed function arg), contradicting the expected output.

## Expected
Docs should:
- Either treat `syscall_nr` strictly as a syscall number (and explain how to capture it), or
- Introduce a separate field for “captured argument” (e.g., `arg0`) and update printing/tests accordingly.

## Actual
Docs describe one meaning but show expected output for another.

## Suggested fix
- Clarify what `syscall_nr` represents for the chosen attachment point, or refactor the shared event struct/printing so arg values and syscall numbers are distinct.

