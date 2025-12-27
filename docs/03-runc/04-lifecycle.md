# 04 Container Lifecycle

## Goal

Master the complete OCI container lifecycle using runc commands: `create`, `start`, `state`, `list`, `exec`, `kill`, and `delete`. You will understand how containers transition between states and practice managing long-running containers through their entire lifecycle.

**Estimated time**: 30-45 minutes

## Prereqs

- Completed `03-run-basic.md` (you have a working OCI bundle with a rootfs)
- `runc` installed and working (verify with `runc --version`)
- `sudo` access (container operations require root privileges)

## Background: OCI Container States

The OCI runtime specification defines a precise lifecycle for containers. Understanding this lifecycle is essential for building container runtimes, debugging container issues, and integrating with orchestration systems.

**Container States:**

| State | Description |
|-------|-------------|
| `creating` | Container is being created (transient state) |
| `created` | Namespaces and cgroups are set up, but process has not started |
| `running` | Container process is executing |
| `stopped` | Container process has exited |

**State Diagram:**

```
              create
     +-----------------------+
     |                       |
     v                       |
  CREATING -----> CREATED    |
                     |       |
                     | start |
                     v       |
                  RUNNING <--+
                     |       |
                     | exit/ |
                     | kill  |
                     v       |
                  STOPPED    |
                     |       |
                     | delete
                     v
                 (removed)
```

**Two Ways to Run a Container:**

1. **Combined (`runc run`)**: Creates and starts in one step. Simpler but less flexible.
   ```bash
   sudo runc run mycontainer
   ```

2. **Separated (`runc create` + `runc start`)**: Two-step process. Allows hooks, pre-start validation, and coordination with orchestrators.
   ```bash
   sudo runc create mycontainer
   # Container exists but process not started yet
   sudo runc start mycontainer
   # Now process is running
   ```

**Why Use create/start Instead of run?**

The two-step process is essential for:
- **Pre-start hooks**: Run scripts after namespaces are created but before process starts
- **Network setup**: Configure networking after namespaces exist but before the app runs
- **Coordination**: Orchestrators like Kubernetes use this to verify readiness
- **Debugging**: Inspect container state before the process begins

**Lifecycle Commands:**

| Command | Description |
|---------|-------------|
| `runc create <id>` | Create container (sets up namespaces, cgroups) but does not start the process |
| `runc start <id>` | Start the container process |
| `runc state <id>` | Query container state as JSON |
| `runc list` | List all containers and their states |
| `runc exec <id> <cmd>` | Execute an additional process inside a running container |
| `runc kill <id> [signal]` | Send a signal to the container (default: SIGTERM) |
| `runc delete <id>` | Remove container state (container must be stopped) |

## Exercises

This lesson is hands-on practice with runc commands. Rather than writing Rust code, you will manually walk through the container lifecycle to build intuition for what your future Rust code will need to handle.

### Prepare the Bundle for Lifecycle Commands

The `runc run` command handles terminal allocation automatically, but `runc create`/`runc start` requires some config changes. We also need a long-running process to keep the container alive.

**Create a dedicated bundle directory:**

```bash
cd /tmp
mkdir -p lifecycle-bundle/rootfs
cd lifecycle-bundle
```

**Set up the rootfs using BusyBox:**

```bash
# Download and extract BusyBox if you do not already have it
if [ ! -f busybox ]; then
    curl -Lo busybox https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox
    chmod +x busybox
fi

# Copy to rootfs and create symlinks
cp busybox rootfs/
cd rootfs
for cmd in sh sleep ps cat echo ls mkdir; do
    ln -sf busybox $cmd
done
cd ..
```

**Generate and modify config.json:**

```bash
runc spec
```

Now edit `config.json` to make it work with `create`/`start`:

1. **Change the process to a long-running command** (instead of `sh`):

Find the `"args"` line and change it from:
```json
"args": ["sh"]
```

To:
```json
"args": ["sleep", "999999"]
```

2. **Disable the terminal** (required for create/start without a console socket):

Find the `"terminal"` line and change it from:
```json
"terminal": true
```

To:
```json
"terminal": false
```

**Why these changes?**

- `sleep 999999`: Keeps the container running so we can inspect and interact with it (portable across BusyBox and GNU coreutils)
- `terminal: false`: The `create`/`start` workflow does not allocate a PTY by default. Using `terminal: true` requires a console socket, which adds complexity we do not need for this lesson.

### Exercise 1: Create and Inspect a Container

**Step 1: Create the container**

```bash
cd /tmp/lifecycle-bundle
sudo runc create mycontainer
```

If successful, you see no output (silent success).

**Step 2: List containers**

```bash
sudo runc list
```

Expected output:
```
ID             PID         STATUS      BUNDLE                      CREATED                         OWNER
mycontainer    12345       created     /tmp/lifecycle-bundle       2024-01-15T10:30:00.000000Z     root
```

Notice the status is `created`, not `running`. The PID shown is the container init process, which exists but is paused.

**Step 3: Check container state**

```bash
sudo runc state mycontainer
```

Expected output (JSON):
```json
{
  "ociVersion": "1.0.2",
  "id": "mycontainer",
  "pid": 12345,
  "status": "created",
  "bundle": "/tmp/lifecycle-bundle",
  "rootfs": "/tmp/lifecycle-bundle/rootfs",
  "created": "2024-01-15T10:30:00.000000000Z",
  "owner": ""
}
```

Key fields:
- `status`: `"created"` - container exists but process not running
- `pid`: The init process PID in the host namespace
- `bundle`: Path to the OCI bundle directory

**Step 4: Verify the process exists but is not running**

```bash
# The process should exist
ps aux | grep -E "^[^ ]+ +$(sudo runc state mycontainer 2>/dev/null | grep '"pid"' | grep -o '[0-9]*')"

# Or using the PID directly:
CONTAINER_PID=$(sudo runc state mycontainer | grep '"pid"' | grep -o '[0-9]*')
echo "Container init PID: $CONTAINER_PID"
cat /proc/$CONTAINER_PID/status | grep State
```

You should see the process exists and is in a stopped/traced state (waiting to start).

### Exercise 2: Start and Monitor the Container

**Step 1: Start the container**

```bash
sudo runc start mycontainer
```

Again, silent success. The container process is now running.

**Step 2: Verify the status changed**

```bash
sudo runc list
```

Expected output:
```
ID             PID         STATUS      BUNDLE                      CREATED                         OWNER
mycontainer    12345       running     /tmp/lifecycle-bundle       2024-01-15T10:30:00.000000Z     root
```

Status is now `running`.

**Step 3: Check state again**

```bash
sudo runc state mycontainer
```

The status field now shows `"running"`.

**Step 4: Verify the process is actually running**

```bash
CONTAINER_PID=$(sudo runc state mycontainer | grep '"pid"' | grep -o '[0-9]*')
cat /proc/$CONTAINER_PID/status | grep State
```

Expected: `State: S (sleeping)` - the sleep command is running and sleeping.

### Exercise 3: Execute Commands in Running Container

The `runc exec` command lets you run additional processes inside a running container. This is how `docker exec` works under the hood.

**Step 1: Run a command to list processes inside the container**

```bash
sudo runc exec mycontainer /ps
```

Expected output:
```
PID   USER     TIME  COMMAND
    1 root      0:00 sleep 999999
    5 root      0:00 /ps
```

Notice:
- `sleep 999999` is PID 1 (the container init process)
- `/ps` is a new process spawned by exec

**Step 2: Explore the container filesystem**

```bash
sudo runc exec mycontainer /ls -la /
```

You should see the minimal rootfs contents (busybox symlinks).

**Step 3: Run an interactive shell (requires terminal support)**

For an interactive session, we need to specify that exec should allocate a terminal:

```bash
sudo runc exec -t mycontainer /sh
```

Inside the container:
```bash
# Look at processes
/ps aux

# Check our PID
echo $$

# Look at filesystem
/ls -la

# Exit the shell
exit
```

**Step 4: Verify the main process is still running**

```bash
sudo runc list
```

The container should still be `running`. The exec sessions are temporary - they do not affect the main container process.

### Exercise 4: Stop the Container with Signals

**Step 1: Send SIGTERM (graceful shutdown)**

```bash
sudo runc kill mycontainer SIGTERM
```

**Step 2: Check the status**

```bash
sudo runc list
```

Expected output:
```
ID             PID         STATUS      BUNDLE                      CREATED                         OWNER
mycontainer    0           stopped     /tmp/lifecycle-bundle       2024-01-15T10:30:00.000000Z     root
```

The status is now `stopped` and the PID is 0 (process no longer exists).

**Step 3: Check state**

```bash
sudo runc state mycontainer
```

Status shows `"stopped"`.

**Note**: If the container does not stop with SIGTERM (some processes ignore it), use SIGKILL:

```bash
sudo runc kill mycontainer SIGKILL
```

SIGKILL cannot be ignored and always terminates the process.

### Exercise 5: Delete the Container

**Step 1: Try to exec into a stopped container (expect failure)**

```bash
sudo runc exec mycontainer /ps
```

Expected error:
```
cannot exec into a stopped container
```

**Step 2: Delete the container state**

```bash
sudo runc delete mycontainer
```

**Step 3: Verify it is gone**

```bash
sudo runc list
```

Expected: Empty list or the container is no longer shown.

```bash
sudo runc state mycontainer
```

Expected error:
```
container "mycontainer" does not exist
```

### Exercise 6: Full Lifecycle Practice

Now run through the complete lifecycle from scratch to reinforce the commands:

```bash
cd /tmp/lifecycle-bundle

# 1. Create
sudo runc create lifecycle-test
echo "Status after create:"
sudo runc list

# 2. Start
sudo runc start lifecycle-test
echo "Status after start:"
sudo runc list

# 3. Exec
echo "Processes inside container:"
sudo runc exec lifecycle-test /ps

# 4. Kill
sudo runc kill lifecycle-test SIGTERM
echo "Status after kill:"
sudo runc list

# 5. Delete
sudo runc delete lifecycle-test
echo "Status after delete:"
sudo runc list
```

## Verify

Run through these verification steps to confirm you understand the lifecycle:

**1. Lifecycle sequence check:**

```bash
cd /tmp/lifecycle-bundle

# Create and verify created state
sudo runc create verify-test
test "$(sudo runc state verify-test | grep '"status"' | grep -o '"[^"]*"$')" = '"created"' && echo "PASS: created state" || echo "FAIL"

# Start and verify running state
sudo runc start verify-test
test "$(sudo runc state verify-test | grep '"status"' | grep -o '"[^"]*"$')" = '"running"' && echo "PASS: running state" || echo "FAIL"

# Kill and verify stopped state
sudo runc kill verify-test SIGTERM
sleep 1
test "$(sudo runc state verify-test | grep '"status"' | grep -o '"[^"]*"$')" = '"stopped"' && echo "PASS: stopped state" || echo "FAIL"

# Delete
sudo runc delete verify-test
sudo runc state verify-test 2>&1 | grep -q "does not exist" && echo "PASS: deleted" || echo "FAIL"
```

All checks should say "PASS".

**2. Verify you can distinguish created vs running:**

```bash
# Create but do not start
sudo runc create state-test

# Should be created, not running
sudo runc list | grep state-test | grep -q "created" && echo "PASS: correctly shows created" || echo "FAIL"

# Now start it
sudo runc start state-test

# Should be running now
sudo runc list | grep state-test | grep -q "running" && echo "PASS: correctly shows running" || echo "FAIL"

# Clean up
sudo runc kill state-test SIGKILL
sudo runc delete state-test
```

## Clean Up

Remove any containers created during this lesson:

```bash
# List all containers
sudo runc list

# For each container, kill and delete:
for container in $(sudo runc list -q 2>/dev/null); do
    echo "Cleaning up: $container"
    sudo runc kill $container SIGKILL 2>/dev/null
    sudo runc delete $container 2>/dev/null
done

# Remove the bundle directory
rm -rf /tmp/lifecycle-bundle
```

Verify cleanup:
```bash
sudo runc list
# Should show no containers
```

## Common Errors

1. **`container "mycontainer" already exists`**
   - Cause: A container with this ID already exists (even if stopped)
   - Fix: Delete the existing container first: `sudo runc delete mycontainer`
   - Check existing containers: `sudo runc list`

2. **`cannot exec into a stopped container`** or **`container is not running`**
   - Cause: Trying to run `runc exec` on a container that is stopped or not yet started
   - Fix: Start the container first with `runc start`, or check status with `runc list`

3. **`cannot delete container that is not stopped`** or **`container still running`**
   - Cause: Trying to delete a running container
   - Fix: Kill the container first: `sudo runc kill mycontainer SIGKILL`, then delete

4. **`terminal: true` with `runc create` fails with console socket error**
   - Cause: When using the create/start workflow with terminal enabled, runc needs a console socket to connect the PTY
   - Fix: Either set `"terminal": false` in config.json, or provide a console socket with `--console-socket`
   - For this lesson, we use `terminal: false` to keep things simple

5. **Container immediately stops after start**
   - Cause: The process exited immediately (e.g., `sh` with no TTY, or a command that completes quickly)
   - Fix: Use a long-running process like `sleep 999999` and ensure `terminal: false` if not providing a PTY
   - Check container logs if available, or examine the process exit code

6. **`runc start` hangs or times out**
   - Cause: The container init process is waiting for something (terminal input, network, etc.)
   - Fix: Check your config.json, especially `terminal` and `args` settings
   - Use `runc state` from another terminal to check status

7. **Permission denied errors**
   - Cause: Container operations require root privileges
   - Fix: Use `sudo` for all runc commands

## Notes

**Container IDs:**
- The container ID is just a string identifier (like `mycontainer`)
- It must be unique among all containers on the system
- Convention: Use descriptive names or generated UUIDs

**State Storage:**
- runc stores container state in `/run/runc/<container-id>/` by default
- The `--root` flag changes this location
- State includes: PID, status, bundle path, creation time

**Process Lifecycle:**
- The container's "main process" is defined in config.json `process.args`
- When this process exits, the container transitions to `stopped`
- Additional processes from `runc exec` do not affect container lifecycle

**Signal Handling:**
- `SIGTERM` (15): Graceful termination - process can catch and clean up
- `SIGKILL` (9): Immediate termination - cannot be caught
- `SIGHUP` (1): Often used to reload configuration
- `SIGUSR1/SIGUSR2`: Application-specific signals

**The Console Socket (for terminal: true):**
- When terminal is true, runc needs a way to pass the PTY file descriptor
- A console socket is a Unix socket where runc sends the PTY master fd
- Tools like `conmon` or container runtimes handle this
- For learning, `terminal: false` is simpler

**Difference from Docker:**
- `docker run` = `runc run` (combined create + start)
- `docker create` = `runc create`
- `docker start` = `runc start`
- `docker exec` = `runc exec`
- `docker kill` = `runc kill`
- `docker rm` = `runc delete`

**Why This Matters for Rust Integration:**
- Your Rust code will need to spawn runc with these commands
- Understanding states helps you handle edge cases
- The state JSON structure is what your code will parse
- Proper cleanup (kill + delete) prevents resource leaks

## Next

`05-seccomp.md` - Add syscall filtering with seccomp-bpf to restrict what the container can do
