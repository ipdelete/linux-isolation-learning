# Rewrite TODO (Rust-first)

## Phase 0: Setup
- [x] Define crate workspace layout under `crates/`
- [x] Decide on baseline Rust deps (likely `nix`, `libc`, `clap`)
- [x] Draft a small lesson template

## Foundations
- [x] Fill `docs/00-foundations/` lessons
- [x] Implement `ns-tool proc` for `/proc` namespace inspection

## Phase 1: Namespaces
- [ ] Split `01-namespaces.md` into small lessons
- [ ] Create Rust-based PID namespace tool
- [ ] Create Rust-based UTS/IPC namespace tools
- [ ] Create Rust-based mount namespace tool
- [ ] Create Rust-based netns tool (veth + bridge)
- [ ] Add verification steps per lesson

## Phase 2: Cgroups
- [ ] Split `02-cgroups.md` into small lessons
- [ ] Create Rust cgroup v2 helper (create, attach, limit)
- [ ] Add memory/cpu/io/pids examples in Rust
- [ ] Add monitoring/inspection commands per lesson

## Phase 3: runc/OCI
- [ ] Split `03-runc.md` into small lessons
- [ ] Create Rust bundle/config helper (read/write config.json)
- [ ] Add examples for create/start/exec/kill
- [ ] Integrate namespaces+cgroups with runc config

## Appendix
- [ ] Add Rust syscall cheatsheet
- [ ] Add troubleshooting guide
