# Bug: DevContainer runs as root but tutorials assume sudo

## Summary
In the DevContainer, the user is `root` (`id -u` returns `0`), so many tutorial commands that prefix `sudo` are redundant/confusing and can create confusing “run as normal user” guidance conflicts.

## Location
- `docs/00-getting-started.md`
- `docs/00-foundations/*.md` (multiple “run with sudo” examples)

## Problem
The docs frequently require `sudo` and suggest building as a non-root user, but the DevContainer environment is already root.

## Steps to reproduce
1. Open the DevContainer.
2. Run `id -u` and `whoami`.

## Expected
Docs mention that, in the DevContainer, you can omit `sudo` and that “build as non-root” guidance does not apply unless you create a non-root user.

## Actual
Docs consistently instruct `sudo ...` and “build as normal user”, which doesn’t match the default DevContainer user.

## Suggested fix
- Add a short note early in `docs/00-getting-started.md` (and optionally in foundation lessons) clarifying:
  - DevContainer runs as `root`
  - `sudo` is unnecessary there
  - any “non-root build” guidance is for non-container setups

