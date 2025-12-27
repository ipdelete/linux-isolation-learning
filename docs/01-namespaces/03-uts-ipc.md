# 03 UTS + IPC Namespaces

## Goal
- Change hostname in UTS and create isolated IPC objects.

## Build
- Implement `ns-tool uts` and `ns-tool ipc`.

## Verify
```bash
sudo cargo run -q -p ns-tool -- uts
sudo cargo run -q -p ns-tool -- ipc
```

## Notes
- UTS is the simplest place to practice `unshare()`.
