# Bug: `setns` lesson includes a non-compiling default namespace-types snippet

## Summary
In `docs/01-namespaces/10-join-existing.md`, the example for defaulting `types_to_join` constructs a temporary `Vec<String>` and then takes a reference to it, which will not compile due to borrowing a temporary value.

## Location
- `docs/01-namespaces/10-join-existing.md`

## Problem
Learners implementing the snippet as written will hit a Rust borrow-checker error, interrupting the flow of the lesson.

## Steps to reproduce
1. Follow `docs/01-namespaces/10-join-existing.md`.
2. Implement the `exec_in_namespace()` function exactly as shown.
3. Attempt to compile.

## Expected
The doc’s example compiles as written, or it clearly marks pseudocode vs compilable code.

## Actual
The example uses a pattern like:
- create `default_types` as `Vec<&str>`
- then `unwrap_or(&default_types.iter().map(|s| s.to_string()).collect())`

This takes a reference to a temporary `Vec<String>`, which is dropped immediately.

## Suggested fix
- Use an owned `Vec<String>` for defaults and branch on `ns_types`, e.g.:
  - `let types_to_join: Vec<String> = ns_types.map(|v| v.to_vec()).unwrap_or_else(|| vec![...]);`
  - then iterate over `types_to_join`.
- Or use a `static` default list of `&'static str` and handle conversion without referencing temporaries.

