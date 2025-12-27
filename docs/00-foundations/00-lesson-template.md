# Lesson Title

## Goal
State the single concept you'll learn and the one deliverable you'll build (e.g., "Create a PID namespace and observe process isolation").

## Prereqs
- List only what's required for this specific lesson (e.g., "Completed `00-setup-rust.md`", "`sudo` access")
- Mention specific prior lessons if they introduced required concepts

## Write Tests (Red)
**Test file**: `crates/<crate>/tests/<feature>_test.rs`

What the tests should verify:
- Success case: [describe expected behavior]
- Error case: [describe expected failure modes if applicable]

Steps:
1. Open the test file at the path above
2. Find the TODO(s) in the test function(s)
3. Implement the test using `assert_cmd` patterns:
   ```rust
   use assert_cmd::Command;

   let mut cmd = Command::cargo_bin("<crate>").unwrap();
   cmd.arg("<subcommand>")
      .assert()
      .success()
      .stdout(predicates::str::contains("expected output"));
   ```
4. Run the test (expect failure):
   ```bash
   cargo test -p <crate> --test <feature>_test
   ```

Expected output: Tests panic with `todo!()` or fail because implementation is missing (RED phase).

## Build (Green)
**Implementation file**: `crates/<crate>/src/main.rs` (or other file)
**TODO location**: Line ~XX in the `Command::<Variant>` match arm

Steps:
1. Open `crates/<crate>/src/main.rs`
2. Find the `Command::<Variant> => todo!(...)` match arm
3. Replace `todo!()` with implementation:
   - Step 1: [specific code or syscall to add]
   - Step 2: [next specific action]
   - Step 3: [final action - usually printing output]
4. Run tests (expect success):
   ```bash
   cargo test -p <crate> --test <feature>_test
   ```

Expected output: All tests pass (GREEN phase).

## Verify
**Automated verification**:
```bash
cargo test -p <crate>  # All tests pass
```

**Manual verification** (observe the actual behavior):
```bash
# Command to run
sudo cargo run -p <crate> -- <subcommand>

# What you should see
[Describe expected output or system state]

# How to inspect the result
[Commands to check /proc, ip addr, cgroup files, etc.]
```

## Clean Up (if applicable)
```bash
# Commands to remove created resources
[Specific cleanup commands for namespaces/cgroups/files/etc.]
```

## Common Errors
1. **Error message or symptom**
   - Cause: [Why this happens]
   - Fix: [How to resolve it]

2. **Error message or symptom**
   - Cause: [Why this happens]
   - Fix: [How to resolve it]

3. **Error message or symptom** (if applicable)
   - Cause: [Why this happens]
   - Fix: [How to resolve it]

## Notes
- Short clarifications or gotchas specific to this lesson
- Links to man pages or relevant documentation
- Differences between kernel versions if applicable

## Next
`XX-next-lesson.md` - [Brief description of what's next]
