# 03 Procfs Intro

## Goal
- Read namespace data from `/proc` using Rust.

## Prereqs
- `ns-tool` builds.

## Build
1) Run the `ns-tool proc` command.
2) Compare the output with a raw `readlink`.

## Verify
```bash
cargo run -q -p ns-tool -- proc
readlink /proc/$$/ns/pid
```

## Notes
- The symlink inode values are how we compare namespaces.
