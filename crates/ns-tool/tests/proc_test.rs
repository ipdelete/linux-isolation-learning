// Tests for the `proc` subcommand (/proc/self/ns inspection)
// Lesson: docs/00-foundations/03-procfs-intro.md
//
// TDD Workflow:
// 1. Write the test(s) below (RED - implement these TODOs)
// 2. Tests will pass because implementation already exists (GREEN)
// 3. This teaches testing patterns using an existing implementation
//
// This is the primary deliverable for lesson 03-procfs-intro.
// Lessons 01 and 02 focus on understanding concepts, not implementing tests.

#[test]
fn test_proc_lists_namespaces() {
    // TODO: Write a test that verifies `ns-tool proc` outputs namespace information
    //
    // Hints:
    // - Use assert_cmd::Command to run the binary
    // - Check that output includes expected namespace names (pid, net, mnt, uts, ipc, etc.)
    // - Each namespace should show its inode number in brackets
    //
    // Example pattern:
    //   let mut cmd = Command::cargo_bin("ns-tool").unwrap();
    //   cmd.arg("proc")
    //      .assert()
    //      .success()
    //      .stdout(predicates::str::contains("pid"));

    todo!("Implement test for proc subcommand - verify it lists namespaces from /proc/self/ns")
}

#[test]
fn test_proc_shows_inode_numbers() {
    // TODO: Write a test that verifies namespace inodes are shown in the format: namespace:[inode]
    //
    // Hints:
    // - Look for the pattern like "pid -> pid:[4026531836]"
    // - You can use regex or simple string matching

    todo!("Implement test that verifies inode numbers are displayed correctly")
}
