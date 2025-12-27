# 00 Setup Rust

## Goal
- Install Rust and verify you can build the workspace.

## Prereqs
- A Linux machine or VM (recommended for isolation exercises).

## Build
1) Install Rust with rustup.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2) Restart your shell, then verify toolchain.
```bash
rustc --version
cargo --version
```
3) Build the workspace once.
```bash
cargo build -q
```

## Verify
```bash
cargo run -q -p ns-tool -- proc
```

## Notes
- Later lessons require root. Use a VM or disposable environment.
- If `cargo build -q` fails, install a C toolchain (e.g., `build-essential`).
