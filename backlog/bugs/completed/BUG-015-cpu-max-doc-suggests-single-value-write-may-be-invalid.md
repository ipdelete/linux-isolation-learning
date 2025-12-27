# Bug: CPU controller lesson suggests a `cpu.max` single-value write that may be invalid

## Summary
`docs/02-cgroups/03-cpu.md` suggests that writing a single value (e.g. `"100000"`) to `cpu.max` is valid and that the kernel will use the existing/default period. Depending on kernel semantics, `cpu.max` may require the two-field form `QUOTA PERIOD` (or `max PERIOD`).

## Location
- `docs/02-cgroups/03-cpu.md`

## Problem
If a learner follows the lesson and implements the CLI to write arbitrary strings to `cpu.max`, an example like `"100000"` may:
- fail with `EINVAL` on some kernels, or
- appear to work but not mean what the doc claims.

This creates a confusing “my implementation is correct but the example fails” situation.

## Steps to reproduce
1. Follow `docs/02-cgroups/03-cpu.md` guidance and try setting `cpu.max` to a single value like `"100000"`.
2. Observe whether the kernel accepts it on the learner’s system.

## Expected
The lesson should only include examples that are valid across supported kernels, or clearly qualify kernel/version differences.

## Actual
The doc implies the single-field form is generally valid.

## Suggested fix
- Prefer documenting only the canonical forms:
  - `"<quota> <period>"` (numbers) and `"max <period>"`
- If the single-field form is supported on some kernels, call it out explicitly and keep it optional (with clear caveats).

