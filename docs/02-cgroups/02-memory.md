# 02 Memory Controller

## Goal
- Set `memory.max` for a cgroup.

## Prereqs
- Completed `01-cgv2-basics.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `cgroup-tool memory-max`.

## Verify
- Automated: `cargo test -p cgroup-tool`
- Manual:
```bash
sudo cargo run -q -p cgroup-tool -- memory-max /my-test 52428800
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- 50 MB = 52,428,800 bytes.

## Next
- `03-cpu.md`
