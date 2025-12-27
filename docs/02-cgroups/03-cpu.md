# 03 CPU Controller

## Goal
- Set `cpu.max` for a cgroup.

## Build
- Implement `cgroup-tool cpu-max`.

## Verify
```bash
sudo cargo run -q -p cgroup-tool -- cpu-max /my-test "20000 100000"
```

## Notes
- The format is: quota period.
