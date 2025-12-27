# 08 Netns NAT

## Goal
- Give a namespace outbound internet via NAT.

## Prereqs
- Completed `07-veth-bridge.md`

## Write Tests (Red)
- TBD: Test location and structure

## Build (Green)
- Implement `netns-tool nat` to apply iptables rules.

## Verify
- Automated: `cargo test -p netns-tool`
- Manual:
```bash
sudo cargo run -q -p netns-tool -- nat br0 eth0
```

## Clean Up
- TBD: How to remove NAT rules

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- We will keep the NAT rules minimal and reversible.

## Next
- `09-combine-ns.md`
