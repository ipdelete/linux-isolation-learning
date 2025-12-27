# 10 Join Existing Namespaces

## Goal
- Use `setns()` to join an existing namespace.

## Prereqs
- Completed `09-combine-ns.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `ns-tool setns` with a PID target.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
sudo cargo run -q -p ns-tool -- setns
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- This is how tools like `nsenter` work.

## Next
- Move to cgroups lessons: `../02-cgroups/01-cgv2-basics.md`
