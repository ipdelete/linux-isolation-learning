# Story: DevContainer fully supports `docs/03-runc/*` (OCI/runc) without requiring Docker-in-Docker

## Summary
Ensure the devcontainer environment has all required tooling and permissions to complete `docs/03-runc/*` inside the container, and make the docs/validation explicit about what is (and is not) required (especially Docker).

## Background
`.devcontainer/devcontainer.json` already runs as `root` and is started with `--privileged` and `seccomp=unconfined`, which is necessary for namespaces/cgroups/mount operations.

However, `docs/03-runc/*` currently assumes tools that are not explicitly present in the devcontainer image:
- `runc` (required)
- `jq` (used heavily for editing/validating `config.json`)
- `curl` (used for BusyBox download in `03-run-basic.md`) (may be present in the base image, but not guaranteed)

The `docs/03-runc/*` also include some optional Docker-based steps for obtaining a rootfs (e.g. `docker export ...`). The devcontainer does not currently provide Docker, so those paths should be clearly optional and/or replaced with non-Docker alternatives.

## Goals
- Learners can complete `docs/03-runc/*` inside the devcontainer.
- The devcontainer includes `runc` and `jq` (and any other minimal prerequisites needed for these lessons).
- Validation tooling confirms runc readiness.
- Docs do not require Docker; Docker-based paths are clearly marked optional.

## Non-goals
- Running Docker workloads inside the devcontainer (no DinD requirement for the learning path).
- Building a full image workflow (pull/build/push); the focus is OCI bundles + `runc`.

## Work Items
1. Update `.devcontainer/devcontainer.json` `postCreateCommand` to install:
   - `runc`
   - `jq`
   - `curl` + `ca-certificates` (if not already guaranteed by base image)
2. Update `scripts/validate-devcontainer.sh` to check:
   - `runc` present (`command -v runc`)
   - `jq` present (`command -v jq`)
3. Update `.devcontainer/validation.md` to include:
   - `runc` and `jq` in tool checklist
   - A short “OCI/runc lessons” section documenting expected support constraints
4. Update `docs/03-runc/*` where needed to:
   - Treat Docker-rootfs extraction as optional and provide a non-Docker alternative path
   - Clearly state if any step assumes external network access (BusyBox download)

## Acceptance Criteria
- `scripts/validate-devcontainer.sh` passes in the devcontainer and includes explicit checks for `runc` and `jq`.
- A learner can follow `docs/03-runc/03-run-basic.md` through `docs/03-runc/07-cgroups-integration.md` in the devcontainer without needing Docker.
- Any Docker usage in `docs/03-runc/*` is clearly labeled optional and not required for completion.

## Related Bugs
- `backlog/bugs/BUG-016-runc-seccomp-doc-uses-invalid-runc-run-flags-and-command-override.md`
- `backlog/bugs/BUG-017-runc-run-basic-doc-uses-docker-style-command-override-with-double-dash.md`
- `backlog/bugs/BUG-019-runc-lessons-rootfs-scaffolding-missing-common-mount-target-dirs.md`

