# 07 Veth + Bridge

## Goal
- Wire a namespace to the host with a veth pair and a bridge.

## Build
- Implement `netns-tool veth` and `netns-tool bridge`.

## Verify
```bash
sudo cargo run -q -p netns-tool -- bridge br0
sudo cargo run -q -p netns-tool -- veth veth0 ns1
```

## Notes
- We will add IP configuration in this lesson.
