// Tests for the `cgroup` subcommands (create, attach, delete)
// Lesson: docs/fast-track/05-cgroup-basics.md
//
// TDD Workflow:
// 1. Write the test below FIRST (RED)
// 2. Implement code in src/cgroup.rs (GREEN)

use assert_cmd::Command;

#[test]
fn test_cgroup_create_and_attach() {
    // TODO: Test that `contain cgroup` commands create and manage cgroups.
    //
    // Steps:
    // 1. Skip if not root (requires write access to /sys/fs/cgroup)
    // 2. Run `contain cgroup create /sys/fs/cgroup/test-cg`
    // 3. Assert success and verify path exists
    // 4. Cleanup with `contain cgroup delete /sys/fs/cgroup/test-cg`
    //
    // Hints:
    // - Check root: nix::unistd::Uid::effective().is_root()
    // - Use std::path::Path::new(cgroup).exists() to verify
    // - Cgroup must be empty before deletion

    todo!("Implement test - see docs/fast-track/05-cgroup-basics.md")
}
