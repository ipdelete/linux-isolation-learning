# 07 Veth + Bridge

## Goal
- Wire a namespace to the host with a veth pair and a bridge.

## Prereqs
- Completed `06-netns-basics.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `netns-tool veth` and `netns-tool bridge`.

## Verify
- Automated: `cargo test -p netns-tool`
- Manual:
```bash
sudo cargo run -q -p netns-tool -- bridge br0
sudo cargo run -q -p netns-tool -- veth veth0 ns1
```

## Clean Up
- TBD: How to remove veth pair and bridge

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will add IP configuration in this lesson.

## Next
- `08-netns-nat.md`
