# 00 Setup Rust

## Goal
- Install Rust and verify you can build and run the workspace tools.

## Prereqs
- Linux host or VM (recommended). Many later lessons require `sudo`.
- Basic CLI tools: `git`, `curl`.

## Build
1) Install Rust with rustup (downloads from the internet).
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2) Restart your shell, then verify the toolchain.
```bash
rustc --version
cargo --version
```
3) Build the workspace once (as your normal user, not root).
```bash
cargo build -q
```

## Verify
- Verify you can run a tool (it’s OK if commands are still TODO in code at this stage).
```bash
cargo run -q -p ns-tool -- proc
```

## Common Errors
- TBD (to be filled in based on learner experience)

## Notes
- Prefer: build as your user, then run binaries with `sudo` when required:
  - build: `cargo build -q -p ns-tool`
  - run: `sudo ./target/debug/ns-tool <subcommand>`
- If `cargo build` fails due to missing linker/toolchain, install a C toolchain (e.g., `build-essential` or `base-devel`).
- Later lessons also assume common Linux tooling is installed (`unshare`, `nsenter`, `ip`, `iptables`/`nft`), but we'll call that out when needed.

## Next
- `01-rust-syscall-basics.md`
