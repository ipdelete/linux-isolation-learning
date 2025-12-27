# Rewrite Plan (Rust-first, TDD, learner-implements-code)

Goal: Build a Rust-first, test-driven learning path for Linux isolation (namespaces, cgroups v2, OCI/runc). The tutorials teach using TDD: you write tests first, then implement code to make them pass. Each lesson provides starter crates with TODOs for both tests and implementation. I (the assistant) focus on writing and iterating the docs, exercises, and verification steps—without filling in the TODOs unless you explicitly ask.

## How We'll Work
- **TDD approach**: Each lesson has you write tests first, then implement code to make them pass (red → green).
- You write both tests and implementation in `crates/*` by following the lesson steps.
- I update the docs to be clearer based on your questions, mistakes, and discoveries.
- Each lesson references specific TODO locations for tests and implementation.
- Keep lessons small. If a lesson grows past ~50 minutes, split it.

## Success Criteria (Per Lesson)
- One concept + one small deliverable (usually a single subcommand or helper function).
- **TDD workflow**: Write test(s) first (red) → implement code (green) → refactor if needed.
- "Verify" proves correctness via automated tests + manual inspection (commands + what you should observe).
- "Clean Up" removes created namespaces/cgroups/interfaces/files (when applicable).
- "Common Errors" captures the top 2–4 pitfalls we actually hit.

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

## Conventions We'll Use
- Docs reference code like: `crates/ns-tool/src/main.rs` (TODO: implement `pid` subcommand).
- Docs reference tests like: `crates/ns-tool/tests/pid_test.rs` (TODO: write test for `pid` subcommand).
- Use `cargo test -p <crate>` to run tests; `cargo run -p <crate> -- <args>` for manual verification.
- Prefer `nix` APIs where they map cleanly; drop to `libc` when needed.
- Minimize `unsafe`; when required, keep it in the smallest possible helper and explain why.

## Testing Strategy (TDD Approach)
- **Red/Green/Refactor**: Each lesson follows Test-Driven Development.
- **Write tests first**: Lessons instruct you to write the test(s) before implementing functionality.
- **Test types**:
  - Unit tests for helper functions and utilities
  - Integration tests for subcommands (e.g., namespace creation + attachment)
  - Tests should verify both success cases and expected error conditions
- **Test location**: Tests go in each crate's `tests/` directory (integration) or inline `#[cfg(test)]` modules (unit).
- **Verify section**: Still included for manual checks (e.g., inspecting `/proc`, `ip addr`, cgroup files) that complement automated tests. Some behaviors are easier to verify manually than to assert programmatically.

## Delivery Sequence
1) Foundations: syscall/Rust patterns + `/proc` inspection + ergonomics (errors, logging).
2) Namespaces: PID → UTS/IPC → mount → user → netns (loopback → veth → bridge → NAT) → setns.
3) Cgroups v2: create/attach → memory → cpu → pids → io → combined limits + monitoring.
4) OCI/runc: minimal bundle → config.json editing → lifecycle → integrate netns + cgroups → (later) seccomp.

**Dependencies**: Complete foundations (00-*) before namespaces. Namespaces and cgroups can be done in either order, but both should be complete before OCI/runc. Within each section, lessons build on previous ones (follow the numeric order).

## Assistant Work Items
- Rewrite each `docs/*/*.md` from scaffold → real lesson content.
- Add "Verify", "Clean Up", and "Common Errors" sections everywhere they matter.
- Maintain `todo.md` as a checklist of doc files to complete (not crate TODOs).
- Keep `README.md` accurate as the entry point and lesson index.
- Use `docs/00-foundations/00-lesson-template.md` as a reference for consistent lesson structure (this is assistant-only; not a learner-facing lesson).

## Immediate Next Steps
1) Make `docs/00-foundations/*` "real" (tight steps, TDD workflow, crisp verification, first debugging patterns).
2) Make `docs/01-namespaces/01-pid-namespace.md` a complete lesson that:
   - Instructs you to write the test first
   - Points you to the TODO(s) for implementation
   - Does NOT implement the code for you
