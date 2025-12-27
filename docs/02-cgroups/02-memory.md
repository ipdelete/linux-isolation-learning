# 02 Memory Controller

## Goal
- Set `memory.max` for a cgroup.

## Build
- Implement `cgroup-tool memory-max`.

## Verify
```bash
sudo cargo run -q -p cgroup-tool -- memory-max /my-test 52428800
```

## Notes
- 50 MB = 52,428,800 bytes.
