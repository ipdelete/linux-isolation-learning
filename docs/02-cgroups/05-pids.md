# 05 PIDs Controller

## Goal
- Set `pids.max` for a cgroup.

## Build
- Implement `cgroup-tool pids-max`.

## Verify
```bash
sudo cargo run -q -p cgroup-tool -- pids-max /my-test 10
```

## Notes
- This prevents fork bombs inside the cgroup.
