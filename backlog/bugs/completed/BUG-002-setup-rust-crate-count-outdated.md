# Bug: `00-setup-rust` lists the wrong number of crates

## Summary
The setup lesson claims the workspace builds “all four crates”, but the workspace includes more crates (notably `ebpf-tool` and `ebpf-tool-common`).

## Location
- `docs/00-foundations/00-setup-rust.md`

## Problem
The “What this will build” bullet list is out of sync with `Cargo.toml`.

## Steps to reproduce
1. Read `docs/00-foundations/00-setup-rust.md`.
2. Compare to `[workspace].members` in `Cargo.toml`.

## Expected
The doc lists the current workspace crates (or avoids hard-coding a number).

## Actual
It hard-codes “all four crates” and omits eBPF crates.

## Suggested fix
- Update the list (or change language to “build the workspace” and list example crates).

