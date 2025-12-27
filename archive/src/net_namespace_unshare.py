#!/usr/bin/env python3
"""
Create a network namespace using os.unshare() and run commands inside it.
"""

import os
import sys
import subprocess

def main():
    print(f"Parent PID: {os.getpid()}")
    print(f"Network namespace before unshare:")
    subprocess.run(["readlink", f"/proc/{os.getpid()}/ns/net"])
    
    # Create new network namespace
    print("\nCreating new network namespace with unshare()...")
    try:
        os.unshare(os.CLONE_NEWNET)
    except PermissionError:
        print("Error: This script requires root privileges", file=sys.stderr)
        return 1
    
    print(f"Network namespace after unshare:")
    subprocess.run(["readlink", f"/proc/{os.getpid()}/ns/net"])
    
    # Show network interfaces in new namespace
    print("\nNetwork interfaces in new namespace:")
    subprocess.run(["ip", "addr", "show"])
    
    print("\nNotice: Only loopback, and it's DOWN!")
    print("This is a completely isolated network stack.")
    
    return 0

if __name__ == '__main__':
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        sys.exit(1)
        
    sys.exit(main())
