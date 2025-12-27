# 09 Combine Namespaces

## Goal
- Combine PID + NET + MNT into a single isolated process.

## Prereqs
- Completed `08-netns-nat.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Add a new `ns-tool` subcommand that stacks flags.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
sudo cargo run -q -p ns-tool -- net
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- This is our first "mini container".

## Next
- `10-join-existing.md`
