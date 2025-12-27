# 06 Netns Basics

## Goal
- Create a network namespace and bring up loopback.

## Prereqs
- Completed `05-minimal-rootfs.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `netns-tool create` and loopback setup.

## Verify
- Automated: `cargo test -p netns-tool`
- Manual:
```bash
sudo cargo run -q -p netns-tool -- create ns1
```

## Clean Up
- TBD: How to remove the created namespace

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will use `ip` only for inspection at first.

## Next
- `07-veth-bridge.md`
