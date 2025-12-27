// Tests for the `ipc` subcommand (IPC namespace for System V IPC isolation)
// Lesson: docs/01-namespaces/03-ipc-namespace.md
//
// TDD Workflow:
// 1. Write the test(s) below FIRST (RED - they will fail)
// 2. Implement the code in src/main.rs to make tests pass (GREEN)
// 3. Refactor if needed
//
// NOTE: These tests require root privileges.
// Run with: sudo -E cargo test -p ns-tool

#[test]
fn test_ipc_namespace_message_queue_isolation() {
    // TODO: Write a test that verifies IPC message queue isolation
    //
    // Hints:
    // - The `ipc` subcommand should unshare(CLONE_NEWIPC)
    // - Create a message queue in the parent namespace
    // - Verify the child in new IPC namespace cannot see the parent's queue
    // - Check using /proc/sysvipc/msg or similar
    //
    // Test approach:
    // 1. Create a message queue before running command
    // 2. Run `ns-tool ipc` which should list IPC objects
    // 3. Verify the parent's message queue is NOT visible in output

    todo!("Implement test for IPC namespace message queue isolation")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_ipc_namespace_shared_memory_isolation() {
    // TODO: Write a test that verifies shared memory segment isolation
    //
    // Hints:
    // - Similar to message queues, but using shared memory (shmget/shmat)
    // - Check /proc/sysvipc/shm for shared memory segments

    todo!("Implement test for IPC namespace shared memory isolation")
}

#[test]
#[ignore] // Remove this attribute after implementing the test
fn test_ipc_namespace_semaphore_isolation() {
    // TODO: Write a test that verifies semaphore isolation
    //
    // Hints:
    // - Check /proc/sysvipc/sem for semaphore sets

    todo!("Implement test for IPC namespace semaphore isolation")
}
