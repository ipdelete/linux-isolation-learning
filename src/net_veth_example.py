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

def cleanup(namespace):
    """Clean up the network namespace"""
    pass

def main():
    pass

if __name__ == '__main__':
    if os.geteuid() != 0:
        print("This script must be run as root", file=sys.stderr)
        sys.exit(1)
        
    sys.exit(main())
    
