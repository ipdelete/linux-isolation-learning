# 09 Combine Namespaces

## Goal
- Combine PID + NET + MNT into a single isolated process.

## Build
- Add a new `ns-tool` subcommand that stacks flags.

## Verify
```bash
sudo cargo run -q -p ns-tool -- net
```

## Notes
- This is our first "mini container".
