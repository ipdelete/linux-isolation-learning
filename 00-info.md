# Creating a uv Script

## Overview

`uv` allows you to create standalone Python scripts with inline dependency management following PEP 723. Scripts are self-contained and portable without requiring manual virtual environment setup.

## Steps to Create a uv Script

### 1. Initialize a new script

```bash
uv init --script <filename> [--python VERSION]
```

Example:
```bash
uv init --script example.py --python 3.12
```

### 2. Add dependencies

```bash
uv add --script <filename> <dependency> [<dependency2> ...]
```

Example:
```bash
uv add --script example.py 'requests<3' 'rich'
```

This generates a special TOML comment block at the top of your file containing dependency information.

### 3. Add the shebang (optional but recommended)

Add this shebang line at the very top of your script to make it directly executable:

```bash
#!/usr/bin/env -S uv run --script
```

Full script structure:
```python
#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "requests<3",
#   "rich",
# ]
# ///

import requests
from rich.console import Console

# Your code here
```

### 4. Make the script executable

```bash
chmod +x <filename>
```

### 5. Run the script

With shebang:
```bash
./<filename>
```

Or without shebang:
```bash
uv run <filename>
```

## Key Benefits

- **Portable**: Dependencies are declared within the script itself
- **Isolated**: uv automatically creates isolated environments
- **PEP 723 compliant**: Follows Python Enhancement Proposal 723
- **Self-documenting**: Dependencies are visible at a glance
