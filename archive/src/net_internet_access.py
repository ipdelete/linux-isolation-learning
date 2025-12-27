#!/usr/bin/env python
"""
Create a network namespace with internet access via NAT.
Demonstrates how containers get internet access.
"""

import subprocess
import sys
import time

def run_cmd(cmd, description="", check=True):
    """Run a command and print output"""
    if description:
        print(f'\n{description}')
        print(f'Running: {' '.join(cmd)}')

    result = subprocess.run(cmd, capture_output=True, text=True)

    if check and result.returncode != 0:
        print(f'Error: {result.stderr}', file=sys.stderr)
        return False
    
    if result.stdout:
        print(result.stdout.strip())

    return True

def setup_nat_network():
    """Setup namespace with internet access"""

    namespace = "inet-test"
    host_veth = "veth-host-inet"
    ns_veth = "veth-inet"
    host_ip = "10.0.1.1"
    ns_ip = "10.0.1.2"
    subnet = "10.0.1.0/24"

    # Create namesapce and veth pair
    run_cmd(["ip", "netns", "add", namespace], "Create Namespace")
    run_cmd(
        ["ip", "link", "add", host_veth, "type", "veth", "peer", "name", ns_veth],
        "Create veth pair"
    )
    run_cmd(["ip", "link", "set", ns_veth, "netns", namespace], "Move veth to namespace")

    # Configure IPs
    run_cmd(["ip", "addr", "add", f"{host_ip}/24", "dev", host_veth], "Set host IP")
    run_cmd(["ip", "link", "set", host_veth, "up"], "Bring up host veth")

    run_cmd(
        ["ip", "netns", "exec", namespace, "ip", "addr", "add", f"{ns_ip}/24", "dev", ns_veth],
        "Set namespace IP"
    )

    

