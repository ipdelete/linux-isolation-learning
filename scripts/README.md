# Scripts Directory

Helper scripts for the Linux Isolation Learning project.

## Available Scripts

### `validate-devcontainer.sh`

Validates that your devcontainer (or Linux environment) is properly configured for the tutorials.

**Usage**:
```bash
bash scripts/validate-devcontainer.sh
```

**What it checks**:
- System tools (unshare, nsenter, lsns, ip, iptables, etc.)
- Namespace support (PID, UTS, IPC, mount, network, user)
- Cgroup v2 availability and controllers
- Network capabilities (creating namespaces, veth pairs, iptables rules)
- Project build success

**Exit codes**:
- `0`: All checks passed
- `1`: One or more checks failed

For detailed validation steps and troubleshooting, see `../devcontainer-validation.md`.

## Adding New Scripts

When adding new utility scripts:
1. Make them executable: `chmod +x scripts/your-script.sh`
2. Add a shebang: `#!/bin/bash` or `#!/usr/bin/env bash`
3. Add usage documentation to this README
4. Consider adding `set -e` to fail on errors
5. Provide helpful error messages
