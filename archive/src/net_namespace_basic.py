#!/usr/bin/env python3
"""
Basic network namespace creation and inspection.
Demonstrates the minimal state of a new network namespace.
"""

import os
import sys
import subprocess

def create_and_inspect_netns():
    """Create a network namespace and show it's initial state."""
    
    # Create network namespace
    print(f"Creating network namespace 'test-ns'...")
    result = subprocess.run(
        ["sudo", "ip", "netns", "add", "test-ns"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"Failed to create network namespace: {result.stderr}", file=sys.stderr)
        return 1
    
    # List all namespaces
    print("\nListing all network namespaces:")
    subprocess.run(["ip", "netns", "list"])
    
    # Inspect the new namespace
    print("\nInspecting the new network namespace 'test-ns':")
    subprocess.run(["sudo", "ip", "netns", "exec", "test-ns", "ip", "addr", "show"])
    
    print("\nNotice: Only the loopback interface 'lo' is present in the new namespace.")
    
    # Clean up: delete the created namespace
    print("\nCleaning up: Deleting network namespace 'test-ns'...")
    subprocess.run(["sudo", "ip", "netns", "del", "test-ns"])
    
    return 0

if __name__ == "__main__":
    sys.exit(create_and_inspect_netns())