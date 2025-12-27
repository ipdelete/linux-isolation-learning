# Rewrite Plan (Rust-first, learner-implements-code)

Goal: Build a Rust-first, small-file learning path for Linux isolation (namespaces, cgroups v2, OCI/runc). The tutorials teach by having you implement the code inside starter crates (which contain TODOs). I (the assistant) focus on writing and iterating the docs, exercises, and verification steps—without filling in the crate TODOs unless you explicitly ask.

## How We’ll Work
- You write the Rust implementation in `crates/*` by following the lesson steps.
- I update the docs to be clearer based on your questions, mistakes, and discoveries.
- Each lesson references specific TODO locations (file + function) and ends with a “Verify” section.
- Keep lessons small. If a lesson grows past ~50 minutes, split it.

## Success Criteria (Per Lesson)
- One concept + one small deliverable (usually a single subcommand or helper function).
- “Verify” proves correctness (commands + what you should observe).
- “Clean Up” removes created namespaces/cgroups/interfaces/files (when applicable).
- “Common Errors” captures the top 2–4 pitfalls we actually hit.

## Repo Layout (Current)
- `docs/`: the tutorial (small lessons)
  - `docs/00-foundations/`: Rust + syscall essentials
  - `docs/01-namespaces/`: namespace lessons (PID/UTS/IPC/MNT/NET/USER/etc.)
  - `docs/02-cgroups/`: cgroup v2 lessons (memory/cpu/io/pids/etc.)
  - `docs/03-runc/`: OCI bundle + runc usage + integration
  - `docs/90-appendix/`: reference + troubleshooting
- `crates/`: learner-implemented tools (scaffolded with TODOs)
  - `crates/ns-tool/`
  - `crates/netns-tool/`
  - `crates/cgroup-tool/`
  - `crates/oci-tool/`
- `archive/`: prior version for reference only

## Conventions We’ll Use
- Docs reference code like: `crates/ns-tool/src/main.rs` (TODO: implement `pid` subcommand).
- Use `cargo run -p <crate> -- <args>` in “Verify”.
- Prefer `nix` APIs where they map cleanly; drop to `libc` when needed.
- Minimize `unsafe`; when required, keep it in the smallest possible helper and explain why.

## Delivery Sequence
1) Foundations: syscall/Rust patterns + `/proc` inspection + ergonomics (errors, logging).
2) Namespaces: PID → UTS/IPC → mount → user → netns (loopback → veth → bridge → NAT) → setns.
3) Cgroups v2: create/attach → memory → cpu → pids → io → combined limits + monitoring.
4) OCI/runc: minimal bundle → config.json editing → lifecycle → integrate netns + cgroups → (later) seccomp.

## Assistant Work Items
- Rewrite each `docs/*/*.md` from scaffold → real lesson content.
- Add “Verify”, “Clean Up”, and “Common Errors” sections everywhere they matter.
- Maintain `todo.md` as a checklist of doc files to complete (not crate TODOs).
- Keep `README.md` accurate as the entry point and lesson index.

## Immediate Next Steps
1) Make `docs/00-foundations/*` “real” (tight steps, crisp verification, first debugging patterns).
2) Make `docs/01-namespaces/01-pid-namespace.md` a complete lesson that points you to the TODO(s), without implementing them for you.
