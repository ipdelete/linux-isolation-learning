# 01 Cgroup v2 Basics

## Goal
- Create and delete a cgroup v2 directory.

## Build
- Implement `cgroup-tool create` and `cgroup-tool delete`.

## Verify
```bash
sudo cargo run -q -p cgroup-tool -- create /my-test
```

## Notes
- We use the unified hierarchy at `/sys/fs/cgroup`.
