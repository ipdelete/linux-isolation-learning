# Bug: Several runc lessons create incomplete `rootfs/` directory layout, causing mount failures

## Summary
Multiple runc lessons create a minimal `rootfs/` directory tree that may not include common mount targets expected by `runc spec` (e.g., `/dev/pts`, `/dev/shm`, `/run`). Depending on the generated `config.json` and runtime defaults, containers can fail to start with “no such file or directory” for mount destinations.

## Location
- `docs/03-runc/03-run-basic.md` (creates `rootfs/{bin,proc,sys,dev,tmp,etc,root}`)
- `docs/03-runc/06-network-integration.md` (creates `rootfs/bin rootfs/proc rootfs/sys rootfs/etc`, omits `dev/` and others)
- `docs/03-runc/07-cgroups-integration.md` (creates `rootfs/bin rootfs/proc rootfs/sys`, omits `dev/` and others)

## Problem
`runc spec` commonly includes mounts with destinations under `/proc`, `/sys`, `/dev`, and sometimes `/run`. If the destination path does not exist inside the rootfs, `runc` may fail during setup.

## Steps to reproduce
1. Follow any affected lesson and generate `config.json` via `runc spec`.
2. Build the rootfs directories exactly as instructed.
3. Run `sudo runc run ...`.

## Expected
The container starts reliably with the lesson-provided rootfs layout.

## Actual
The container may fail with mount errors due to missing directories in `rootfs/`.

## Suggested fix
- Standardize a “known-good” minimal rootfs directory list across lessons (aligned with the `runc spec` mounts used in the lesson), e.g. include at least:
  - `rootfs/dev`, `rootfs/dev/pts`, `rootfs/dev/shm`, `rootfs/proc`, `rootfs/sys`, `rootfs/run`, `rootfs/tmp`, plus `rootfs/bin` and any required config files.
- Alternatively, explicitly instruct learners to create the mount destination directories that appear in their generated `config.json`.

