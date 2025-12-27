# 05 PIDs Controller

## Goal
- Set `pids.max` for a cgroup.

## Prereqs
- Completed `04-io.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `cgroup-tool pids-max`.

## Verify
- Automated: `cargo test -p cgroup-tool`
- Manual:
```bash
sudo cargo run -q -p cgroup-tool -- pids-max /my-test 10
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- This prevents fork bombs inside the cgroup.

## Next
- `06-multi-resource.md`
