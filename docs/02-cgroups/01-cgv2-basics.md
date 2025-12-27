# 01 Cgroup v2 Basics

## Goal
- Create and delete a cgroup v2 directory.

## Prereqs
- Completed namespace lessons (or can be done in parallel)

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `cgroup-tool create` and `cgroup-tool delete`.

## Verify
- Automated: `cargo test -p cgroup-tool`
- Manual:
```bash
sudo cargo run -q -p cgroup-tool -- create /my-test
```

## Clean Up
- TBD: How to remove the created cgroup

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We use the unified hierarchy at `/sys/fs/cgroup`.

## Next
- `02-memory.md`
