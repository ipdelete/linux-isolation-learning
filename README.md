# Linux Isolation Learning (Rust-first Rewrite)

This is a Rust-first, small-file learning path for Linux isolation: namespaces, cgroups, and OCI/runc. Each lesson builds a tiny tool or subcommand so you learn both Rust and the kernel surface area as you go.

## How This Is Structured
- Short lessons (10–50 min) in `docs/`
- Minimal narrative, heavy on "build + verify"
- Rust tools in `crates/` reused across lessons

## Start Here
1) `docs/00-foundations/00-setup-rust.md`
2) Follow the numbered lessons in order

## Docs Layout
- `docs/00-foundations/` Rust + syscall basics
- `docs/01-namespaces/` PID/UTS/IPC/MNT/NET/USER/TIME
- `docs/02-cgroups/` cgroup v2 controllers
- `docs/03-runc/` OCI bundles + lifecycle
- `docs/90-appendix/` cheatsheets + troubleshooting

## Tools You Will Build
- `crates/ns-tool/` namespace examples
- `crates/netns-tool/` network namespace setup
- `crates/cgroup-tool/` cgroup v2 helpers
- `crates/oci-tool/` OCI bundle helpers

## Tracking
- `plan.md` high-level rewrite plan
- `todo.md` step-by-step work list

## Safety
These exercises require root and can impact system state. Use a VM or disposable environment.
