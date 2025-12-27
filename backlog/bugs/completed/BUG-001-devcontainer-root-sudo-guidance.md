# Bug: DevContainer runs as root but tutorials assume sudo [COMPLETED]

## Summary
In the DevContainer, the user is `root` (`id -u` returns `0`), so many tutorial commands that prefix `sudo` are redundant/confusing and can create confusing "run as normal user" guidance conflicts.

## Location
- `docs/00-getting-started.md`
- `docs/00-foundations/*.md` (multiple "run with sudo" examples)
- `.devcontainer/devcontainer.json`
- `.devcontainer/validation.md`

## Problem
The docs frequently require `sudo` and suggest building as a non-root user, but the DevContainer environment is already root.

## Steps to reproduce
1. Open the DevContainer.
2. Run `id -u` and `whoami`.

## Expected
Docs mention that, in the DevContainer, you can omit `sudo` and that "build as non-root" guidance does not apply unless you create a non-root user.

## Actual
Docs consistently instruct `sudo ...` and "build as normal user", which doesn't match the default DevContainer user.

## Resolution

### Changes Made

1. **docs/00-getting-started.md**
   - Added "DevContainer vs. Native Linux" section explaining the difference
   - Clarifies that `sudo` is needed on native Linux but not in DevContainer
   - Shows examples of both environments

2. **.devcontainer/devcontainer.json**
   - Expanded comments to explain root access policy
   - Clearly states "In DevContainer: You ARE root" vs "On native Linux: You are a regular user"
   - Notes that lessons show `sudo` because they're written for native Linux
   - Directs users to validation.md for examples

3. **.devcontainer/validation.md**
   - Added comprehensive "Root Access in DevContainer vs. Native Linux" section at top
   - Shows side-by-side examples of commands in both environments
   - Explains why the container runs as root (privileged operations needed)
   - Clarifies that "just omit sudo in DevContainer"

4. **docs/00-foundations/00-setup-rust.md**
   - Updated Prerequisites section to clarify root requirements differ by environment
   - Added bullets: "In DevContainer: Commands run as root, no sudo needed"
   - Added bullets: "On native Linux: You'll need sudo or root access"

5. **docs/00-foundations/00-lesson-template.md**
   - Updated Prereqs note to guide DevContainer users on sudo handling
   - Updated "Manual verification" section with conditional guidance
   - Shows both `cargo run` (DevContainer) and `sudo cargo run` (native Linux) examples

### Impact

The documentation now clearly distinguishes between:
- **DevContainer environment**: Running as root with no sudo needed
- **Native Linux environment**: Running as regular user with sudo prefix for privileged operations

This resolves confusion for learners using the DevContainer, while maintaining clarity for those running on native Linux systems.

## Status
✅ COMPLETED - All documentation updated with clear root/sudo guidance for both environments.

## Testing
Verified that:
- DevContainer validation guide now has prominent section on root access
- Getting started guide explains DevContainer vs. native Linux difference
- Foundation lesson template guides users on sudo handling
- devcontainer.json comments explain the policy clearly
