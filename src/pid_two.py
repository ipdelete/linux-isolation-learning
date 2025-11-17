#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import ctypes
import os
import sys
import signal
import time

STACK_SIZE = 1024 * 1024

libc = ctypes.CDLL('libc.so.6', use_errno=True)

def child_fn(arg):
    """Function to run in the new PID namespace"""
    pid = os.getpid()
    ppid = os.getppid()
    print(f"Child PID: {pid}")
    print(f"Child PPID: {ppid}")

    os.execlp("sleep", "sleep", "100")

    return 1

def main():
    print(f"Parent PID: {os.getpid()}")

    CHILD_FUNC = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p)
    child_callback = CHILD_FUNC(child_fn)

    stack_01 = ctypes.create_string_buffer(STACK_SIZE)
    stack_top_01 = ctypes.c_void_p(ctypes.addressof(stack_01) + STACK_SIZE)

    stack_02 = ctypes.create_string_buffer(STACK_SIZE)
    stack_top_02 = ctypes.c_void_p(ctypes.addressof(stack_02) + STACK_SIZE)

    flags = os.CLONE_NEWPID | signal.SIGCHLD
    child_pid_01 = libc.clone(
        child_callback,
        stack_top_01,
        flags,
        None
    )
    child_pid_02 = libc.clone(
        child_callback,
        stack_top_02,
        flags,
        None
    )

    if child_pid_01 == -1:
        errno = ctypes.get_errno()
        print(f"clone failed: {os.strerror(errno)}", file=sys.stderr)
        return 1
    if child_pid_02 == -1:
        errno = ctypes.get_errno()
        print(f"clone failed: {os.strerror(errno)}", file=sys.stderr)
        return 1
    
    print(f"Created child with PID: {child_pid_01}")
    print(f"Created child with PID: {child_pid_02}")

    os.waitpid(child_pid_01, 0)
    os.waitpid(child_pid_02, 0)
    return 0

if __name__ == '__main__':
    sys.exit(main())

