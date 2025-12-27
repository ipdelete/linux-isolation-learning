# Rewrite Plan (Rust-first)

Goal: Replace the current Python/C/bash-heavy learning path with a Rust-first, small-file, tool-driven curriculum that teaches Linux isolation concepts while teaching Rust. The docs guide you to implement the code yourself; tools stay as TODO scaffolds until you fill them in.

## Structure Principles
- Small files (10–50 min each), each ending with a working tool or verification step.
- Single concept per file; minimal narrative, heavy on “do one thing, verify it.”
- Rust-first examples; shell commands only for setup/verification.
- Every concept builds a reusable Rust tool or library function.
- Docs describe the steps; you write the code yourself from TODO scaffolds.

## Proposed Top-Level Layout
- README.md (overview and path index)
- docs/
  - 00-foundations/
  - 01-namespaces/
  - 02-cgroups/
  - 03-runc/
  - 90-appendix/
- crates/
  - ns-tool/        (small CLI for namespace ops)
  - netns-tool/     (veth/bridge/netns ops)
  - cgroup-tool/    (cgroup v2 ops)
  - oci-tool/       (bundle/config tooling)

## Learning Flow (High Level)
1) Rust + Linux syscalls foundations
2) Namespaces as small tools
3) Cgroups as small tools
4) OCI/runc integration and runtime glue

## Rust Teaching Strategy
- Introduce only the Rust features needed for the next syscall/tool.
- Keep unsafe blocks minimal; explain why each is needed.
- Start with std + nix; later add libbpf-rs or aya for eBPF.

## Next Actions
- Split each large markdown into 10–30 small lessons.
- Port each lesson’s tool to Rust.
- Add verification commands and “expected output” per lesson.
