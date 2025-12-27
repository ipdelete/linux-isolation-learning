#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import os
import ctypes
import sys

libc = ctypes.CDLL('libc.so.6', use_errno=True)

print(f"Parent PID in original namespace: {os.getpid()}")

result = libc.unshare(os.CLONE_NEWPID)

if result != 0:
    errno = ctypes.get_errno()
    print(f"unshare failed: {os.strerror(errno)}", file=sys.stderr)
    sys.exit(1)

pid = os.fork()

if pid == 0:
    print(f"Child PID after fork (in new namespace): {os.getpid()}")
    print(f"Child PPID: {os.getppid()}")
    sys.exit(0)
else:
    print(f"Parent PID (in new namespace): {os.getpid()}")
    os.waitpid(pid, 0)
