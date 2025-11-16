#!/usr/bin/env python3

import ctypes
import os
import sys
import signal

CLONE_NEWPID = 0x20000000
STACK_SIZE = 1024 * 1024

libc = ctypes.CDLL('libc.so.6', use_errno=True)

def child_fn(arg):
    """Function to run in the new PID namespace"""
    pid = os.getpid()
    ppid = os.getppid()
    print(f"Child PID: {pid}")
    print(f"Child PPID: {ppid}")

    os.execlp("ps", "ps", "aux")
    return 1

def main():
    print(f"Parrent PID: {os.getpid()}")

    CHILD_FUNC = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p)
    child_callback = CHILD_FUNC(child_fn)

    stack = ctypes.create_string_buffer(STACK_SIZE)
    stack_top = ctypes.c_void_p(ctypes.addressof(stack) + STACK_SIZE)

    flags = CLONE_NEWPID | signal.SIGCHLD
    child_pid = libc.clone(
        child_callback,
        stack_top,
        flags,
        None
    )

    if child_pid == -1:
        errno = ctypes.get_errno()
        print(f"clone failed: {os.strerror(errno)}", file=sys.stderr)
        return 1
    
    print(f"Created child with PID: {child_pid}")

    os.waitpid(child_pid, 0)
    return 0

if __name__ == '__main__':
    sys.exit(main())
