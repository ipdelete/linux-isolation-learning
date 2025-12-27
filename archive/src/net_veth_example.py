#!/usr/bin/env python3

"""
Create a veth pair connecting host and a network namespace.
Demonstrates container-to-host networking.
"""

import os
import subprocess
import sys
import time

def run_cmd(cmd, description=""):
    """Run a command and print output"""
    if description:
        print(f"\n{description}")
        print(f"Running: {' '.join(cmd)}")
        
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"Error: {result.stderr}", file=sys.stderr)
        return False
    
    if result.stdout:
        print(result.stdout)
        
    return True

def setup_veth_network():
    """Set up a veth pair between host and namespace"""
    
    namespace = "blue"
    host_veth = "veth-host"
    ns_veth = "veth-blue"
    host_ip = "10.0.0.1/24"
    ns_ip = "10.0.0.2/24"
    
    # Create namespace
    run_cmd(
        ["ip", "netns", "add", namespace],
        "Step 1: Create network namespace 'blue'"
    )

    # Create veth pair
    run_cmd(
        ["ip", "link", "add", host_veth, "type", "veth", "peer", "name", ns_veth],
        "Step 2: Create veth pair (virtual ethernet cable)"
    )

    # Move one end into namespace
    run_cmd(
        ["ip", "link", "set", ns_veth, "netns", namespace],
        f"Step 3: Move {ns_veth} into {namespace} namespace"
    )

    # Configure Host Side
    run_cmd(
        ["ip", "addr", "add", host_ip, "dev", host_veth],
        f"Step 4: Assign {host_ip} to host side"
    )

    run_cmd(
        ["ip", "link", "set", host_veth, "up"],
        f"Step 5: Bring up {host_veth} on host"
    )

    # Configure namespace side
    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "addr", "add", ns_ip, "dev", ns_veth],
        f"Step 6: Assign {ns_ip} to namespace side"
    )

    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "link", "set", ns_veth, "up"],
        f"Step 7: Bring up {ns_veth} in namespace"
    )

    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "link", "set", "lo", "up"],
        "Step 8: Bring up loopback in namespace"
    )

    # Show configuration
    print("\n" + "="*60)
    print("CONFIGURATION COMPLETE")
    print("="*60)

    print("\nHost side:")
    run_cmd(["ip", "addr", "show", host_veth])

    print("\nNamespace side:")
    run_cmd(["ip", "netns", "exec", namespace, "ip", "addr", "show", ns_veth])

    # Test connectivity
    print("\n" + "="*60)
    print("CONNECTIVITY TEST")
    print("="*60)

    print("\nPing from namespace to host:")
    run_cmd([
        "ip", "netns", "exec", namespace,
        "ping", "-c", "3", "10.0.0.1"
    ])

    print("\nPing from host to namespace:")
    run_cmd(["ping", "-c", "3", "10.0.0.2"])

    return namespace

def cleanup(namespace):
    """Clean up the network namespace"""
    print("\n" + "="*60)
    print("CLEANUP")
    print("="*60)

    run_cmd(
        ["ip", "netns", "del", namespace],
        f"Deleting namespace {namespace} (this also removes veth pair)"
    )

def main():
    try:
        namespace = setup_veth_network()

        print("\n" + "="*60)
        print("Press Ctrl+C to cleanup and exit")
        print("="*60)

        # Keep running so I can experiment
        while True:
            time.sleep(1)

    except KeyboardInterrupt:
        print("\n\nReceived Ctrl+C, cleaning up...")
        cleanup(namespace)

if __name__ == '__main__':
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        sys.exit(1)
        
    sys.exit(main())
    
