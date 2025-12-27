# 08 Netns NAT

## Goal
- Give a namespace outbound internet via NAT.

## Build
- Implement `netns-tool nat` to apply iptables rules.

## Verify
```bash
sudo cargo run -q -p netns-tool -- nat br0 eth0
```

## Notes
- We will keep the NAT rules minimal and reversible.
