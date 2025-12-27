# 03 CPU Controller

## Goal
- Set `cpu.max` for a cgroup.

## Prereqs
- Completed `02-memory.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `cgroup-tool cpu-max`.

## Verify
- Automated: `cargo test -p cgroup-tool`
- Manual:
```bash
sudo cargo run -q -p cgroup-tool -- cpu-max /my-test "20000 100000"
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- The format is: quota period.

## Next
- `04-io.md`
