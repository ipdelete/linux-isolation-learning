# 06 Netns Basics

## Goal
- Create a network namespace and bring up loopback.

## Build
- Implement `netns-tool create` and loopback setup.

## Verify
```bash
sudo cargo run -q -p netns-tool -- create ns1
```

## Notes
- We will use `ip` only for inspection at first.
