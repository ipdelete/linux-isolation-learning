# 03 UTS + IPC Namespaces

## Goal
- Change hostname in UTS and create isolated IPC objects.

## Prereqs
- Completed `02-unshare-vs-clone.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `ns-tool uts` and `ns-tool ipc`.

## Verify
- Automated: `cargo test -p ns-tool`
- Manual:
```bash
sudo cargo run -q -p ns-tool -- uts
sudo cargo run -q -p ns-tool -- ipc
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- UTS is the simplest place to practice `unshare()`.

## Next
- `04-mount-namespace.md`
