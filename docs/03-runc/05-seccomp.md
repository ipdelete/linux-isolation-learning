# 05 Seccomp Filtering

## Goal

Add a minimal seccomp profile to `config.json` and observe a blocked syscall. By the end of this lesson, you will understand how seccomp-bpf works, how OCI containers use seccomp for syscall filtering, and how to create both allowlist and denylist policies.

**Estimated time**: 30-45 minutes

## Prereqs

- Completed `04-lifecycle.md` (you should have a working OCI bundle with busybox)
- `runc` installed and working
- `sudo` access
- Basic understanding of Linux syscalls

## Background: What is Seccomp?

**Seccomp** (Secure Computing Mode) is a Linux kernel feature that restricts the syscalls a process can make. It provides a powerful security boundary by limiting what a process can ask the kernel to do.

### The Evolution of Seccomp

1. **Strict mode** (original, Linux 2.6.12+): Process can only use `read()`, `write()`, `exit()`, and `sigreturn()`. Any other syscall kills the process. Too restrictive for most uses.

2. **Filter mode / seccomp-bpf** (Linux 3.5+): Uses Berkeley Packet Filter (BPF) programs to define custom syscall policies. This is what containers use.

### Why Seccomp Matters for Containers

Even with namespaces and capabilities restricted, a container process still talks to the host kernel. Every syscall is a potential attack surface. Seccomp provides defense-in-depth by:

- **Reducing attack surface**: Block syscalls that the container does not need
- **Preventing privilege escalation**: Block dangerous syscalls like `mount`, `reboot`, `kexec_load`
- **Mitigating kernel exploits**: Many kernel CVEs require specific syscalls to trigger

### How Seccomp-BPF Works

When a process makes a syscall, the kernel:

1. Checks if a seccomp filter is installed
2. Runs the BPF program with syscall number and arguments
3. Takes action based on the filter's verdict:
   - **ALLOW**: Syscall proceeds normally
   - **ERRNO**: Syscall fails with specified errno (e.g., EPERM)
   - **KILL**: Process is terminated immediately (SIGSYS)
   - **LOG**: Syscall is allowed but logged to audit log
   - **TRAP**: Sends SIGSYS signal (can be caught by handler)
   - **TRACE**: Notifies ptrace tracer if attached

### OCI Seccomp Configuration

The OCI runtime specification defines seccomp profiles in `config.json` under `linux.seccomp`. The key fields are:

| Field | Description |
|-------|-------------|
| `defaultAction` | What to do for syscalls not matched by any rule |
| `architectures` | CPU architectures this profile applies to |
| `syscalls` | Array of rules matching syscall names to actions |

**Rule matching**: Rules are evaluated in order. The first matching rule determines the action. If no rule matches, `defaultAction` applies.

## Exercises

This lesson is hands-on configuration work rather than Rust implementation. You will modify `config.json` directly and observe the effects.

### Setup: Prepare Your Bundle

If you do not have a working bundle from previous lessons, create one:

```bash
# Create bundle directory
mkdir -p /tmp/seccomp-test/rootfs

# Copy busybox rootfs (from lesson 03)
# Option 1: Extract busybox docker image
docker export $(docker create busybox) | tar -C /tmp/seccomp-test/rootfs -xf -

# Option 2: If you still have rootfs from previous lesson
# cp -a /path/to/previous/bundle/rootfs/* /tmp/seccomp-test/rootfs/

# Generate base config.json
cd /tmp/seccomp-test
runc spec
```

Verify the bundle works without seccomp:

```bash
# Run a simple command
sudo runc run --rm test-no-seccomp <<< "echo 'Bundle works!'"
```

### Exercise 1: Block a Specific Syscall (Denylist)

The simplest seccomp policy: allow everything except specific syscalls.

**Step 1**: Open `config.json` and find the `linux` section. Add a `seccomp` block:

```json
{
  "linux": {
    "seccomp": {
      "defaultAction": "SCMP_ACT_ALLOW",
      "architectures": [
        "SCMP_ARCH_X86_64",
        "SCMP_ARCH_X86"
      ],
      "syscalls": [
        {
          "names": ["reboot"],
          "action": "SCMP_ACT_ERRNO",
          "errnoRet": 1
        }
      ]
    }
  }
}
```

**Explanation**:
- `defaultAction: SCMP_ACT_ALLOW` - Allow all syscalls by default
- `architectures` - Include both x86_64 and x86 for 32-bit compatibility
- `syscalls` - Block `reboot` by returning EPERM (errno 1)

**Step 2**: Edit the actual config.json file:

```bash
cd /tmp/seccomp-test

# Use jq to add seccomp (or edit manually)
cat config.json | jq '.linux.seccomp = {
  "defaultAction": "SCMP_ACT_ALLOW",
  "architectures": ["SCMP_ARCH_X86_64", "SCMP_ARCH_X86"],
  "syscalls": [{
    "names": ["reboot"],
    "action": "SCMP_ACT_ERRNO",
    "errnoRet": 1
  }]
}' > config.json.tmp && mv config.json.tmp config.json
```

**Step 3**: Test that normal commands still work:

```bash
sudo runc run --rm seccomp-test1
# Inside container:
echo "Hello from seccomp container"
ls /
exit
```

Expected: Commands work normally because most syscalls are allowed.

**Step 4**: Test the blocked syscall:

```bash
# The reboot command uses the reboot() syscall
sudo runc run --rm seccomp-test2
# Inside container:
reboot
```

Expected output:
```
reboot: Operation not permitted
```

The syscall failed with EPERM (errno 1) as configured.

### Exercise 2: Use SCMP_ACT_KILL Instead of ERRNO

Change the action to immediately terminate the process on a blocked syscall.

**Step 1**: Modify the seccomp rule:

```bash
cat config.json | jq '.linux.seccomp.syscalls[0].action = "SCMP_ACT_KILL_PROCESS"' > config.json.tmp && mv config.json.tmp config.json
```

Note: We use `SCMP_ACT_KILL_PROCESS` (kills the entire process) rather than `SCMP_ACT_KILL` (kills only the thread) for cleaner behavior.

**Step 2**: Test:

```bash
sudo runc run --rm seccomp-kill-test
# Inside container:
reboot
```

Expected: The container exits immediately with no error message (killed by SIGSYS).

Check the exit status:

```bash
sudo runc run --rm seccomp-kill-test2 sh -c 'reboot; echo "This will not print"'
echo "Exit code: $?"
```

Expected exit code: 137 (128 + 9, killed by signal).

### Exercise 3: Block Multiple Syscalls

Block several dangerous syscalls that containers should not need:

```bash
cat config.json | jq '.linux.seccomp.syscalls = [
  {
    "names": ["reboot", "swapon", "swapoff", "mount", "umount2", "pivot_root"],
    "action": "SCMP_ACT_ERRNO",
    "errnoRet": 1
  },
  {
    "names": ["kexec_load", "kexec_file_load"],
    "action": "SCMP_ACT_KILL_PROCESS"
  }
]' > config.json.tmp && mv config.json.tmp config.json
```

**Explanation**:
- First rule: Common dangerous syscalls return EPERM
- Second rule: Extremely dangerous kernel-replace syscalls kill the process

**Test**:

```bash
sudo runc run --rm seccomp-multi-test
# Inside container:
mount -t proc proc /proc    # Should fail with EPERM
reboot                      # Should fail with EPERM
exit
```

### Exercise 4: Allowlist (Default Deny)

A more secure approach: only allow syscalls you explicitly list.

**Warning**: This is tricky because even a simple shell needs many syscalls. Start with a minimal set and add as needed.

**Step 1**: Create a minimal allowlist profile:

```bash
cat config.json | jq '.linux.seccomp = {
  "defaultAction": "SCMP_ACT_ERRNO",
  "defaultErrnoRet": 1,
  "architectures": ["SCMP_ARCH_X86_64", "SCMP_ARCH_X86"],
  "syscalls": [{
    "names": [
      "read", "write", "close", "fstat", "mmap", "mprotect",
      "munmap", "brk", "ioctl", "access", "pipe", "dup", "dup2",
      "clone", "fork", "vfork", "execve", "exit", "exit_group",
      "wait4", "kill", "uname", "fcntl", "flock", "fsync",
      "ftruncate", "getcwd", "chdir", "readlink", "stat", "lstat",
      "lseek", "getpid", "getuid", "getgid", "geteuid", "getegid",
      "getppid", "getpgrp", "setsid", "setpgid", "getgroups",
      "setgroups", "setresuid", "setresgid", "capget", "capset",
      "rt_sigaction", "rt_sigprocmask", "rt_sigreturn", "sigaltstack",
      "prctl", "arch_prctl", "futex", "set_tid_address",
      "set_robust_list", "get_robust_list", "nanosleep", "clock_gettime",
      "clock_getres", "sched_getaffinity", "sched_yield",
      "openat", "newfstatat", "readlinkat", "faccessat", "mkdirat",
      "unlinkat", "renameat", "fchownat", "fchmodat", "pread64",
      "pwrite64", "getdents64", "pipe2", "getrandom", "prlimit64",
      "rseq", "clone3", "close_range"
    ],
    "action": "SCMP_ACT_ALLOW"
  }]
}' > config.json.tmp && mv config.json.tmp config.json
```

**Step 2**: Test basic functionality:

```bash
sudo runc run --rm seccomp-allowlist
# Inside container:
echo "Basic commands work"
ls /
cat /etc/hostname
exit
```

**Step 3**: Verify that unlisted syscalls fail:

```bash
sudo runc run --rm seccomp-allowlist2
# Inside container - try something that needs network syscalls:
ping localhost   # Should fail (socket syscall not allowed)
```

Expected: `ping: socket: Operation not permitted`

### Exercise 5: Debug with SCMP_ACT_LOG

When developing a seccomp profile, use logging to see which syscalls would be blocked:

```bash
cat config.json | jq '.linux.seccomp = {
  "defaultAction": "SCMP_ACT_LOG",
  "architectures": ["SCMP_ARCH_X86_64", "SCMP_ARCH_X86"],
  "syscalls": [{
    "names": ["read", "write", "exit", "exit_group", "close", "fstat",
              "mmap", "mprotect", "munmap", "brk", "openat", "newfstatat",
              "execve", "arch_prctl", "set_tid_address", "set_robust_list",
              "prlimit64", "getrandom", "rseq", "clock_gettime", "getpid",
              "ioctl", "fcntl", "dup2", "pipe", "wait4", "clone", "rt_sigaction",
              "rt_sigprocmask", "sigaltstack", "getdents64", "lseek"],
    "action": "SCMP_ACT_ALLOW"
  }]
}' > config.json.tmp && mv config.json.tmp config.json
```

**Step 1**: Run a command in the container:

```bash
sudo runc run --rm seccomp-log-test sh -c 'ls / && echo done'
```

**Step 2**: Check the audit log:

```bash
# On systems with auditd:
sudo ausearch -m seccomp --start recent

# Or check dmesg:
sudo dmesg | grep -i seccomp | tail -20
```

Expected: You will see log entries for syscalls that hit the LOG action (those not in the allow list).

This is extremely useful for building allowlist profiles iteratively.

## Verify

**Manual verification checklist**:

1. Container starts successfully with seccomp profile
2. Blocked syscalls return the expected errno
3. KILL action terminates the process immediately
4. Allowlist profiles block unlisted syscalls
5. Normal container operations still work

**Test script** (save as `/tmp/test-seccomp.sh`):

```bash
#!/bin/bash
set -e

BUNDLE=/tmp/seccomp-test
cd "$BUNDLE"

echo "=== Test 1: Blocked syscall returns EPERM ==="
cat config.json | jq '.linux.seccomp = {
  "defaultAction": "SCMP_ACT_ALLOW",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [{"names": ["reboot"], "action": "SCMP_ACT_ERRNO", "errnoRet": 1}]
}' > config.json.tmp && mv config.json.tmp config.json

OUTPUT=$(sudo runc run --rm test-v1 sh -c 'reboot 2>&1 || true')
if echo "$OUTPUT" | grep -q "not permitted\|Operation not permitted"; then
    echo "PASS: reboot blocked with EPERM"
else
    echo "FAIL: Expected 'Operation not permitted', got: $OUTPUT"
    exit 1
fi

echo "=== Test 2: Normal commands still work ==="
OUTPUT=$(sudo runc run --rm test-v2 sh -c 'echo hello && ls / | head -1')
if [ -n "$OUTPUT" ]; then
    echo "PASS: Normal commands work"
else
    echo "FAIL: Normal commands broken"
    exit 1
fi

echo "=== Test 3: Multiple syscalls blocked ==="
cat config.json | jq '.linux.seccomp.syscalls = [
  {"names": ["reboot", "mount"], "action": "SCMP_ACT_ERRNO", "errnoRet": 1}
]' > config.json.tmp && mv config.json.tmp config.json

OUTPUT=$(sudo runc run --rm test-v3 sh -c 'mount -t proc proc /proc 2>&1 || true')
if echo "$OUTPUT" | grep -q "not permitted\|Operation not permitted"; then
    echo "PASS: mount blocked"
else
    echo "FAIL: Expected mount to be blocked, got: $OUTPUT"
    exit 1
fi

echo ""
echo "=== All tests passed ==="
```

Run the test:

```bash
chmod +x /tmp/test-seccomp.sh
sudo /tmp/test-seccomp.sh
```

## Clean Up

```bash
# Remove test containers if any are still running
sudo runc list 2>/dev/null | tail -n +2 | awk '{print $1}' | xargs -r -I {} sudo runc delete -f {}

# Remove the test bundle
rm -rf /tmp/seccomp-test

# Remove the test script
rm -f /tmp/test-seccomp.sh
```

## Common Errors

1. **Container fails to start - "operation not permitted" immediately**
   - Cause: You blocked an essential syscall that runc or the init process needs
   - Common culprits: `execve`, `read`, `write`, `clone`, `exit`, `mmap`, `brk`
   - Fix: Start with `defaultAction: SCMP_ACT_ALLOW` and block specific syscalls, or use LOG to identify required syscalls

2. **`invalid seccomp profile` or JSON parse error**
   - Cause: JSON syntax error or invalid action name
   - Fix: Validate JSON with `jq . config.json`. Action names must be exact (e.g., `SCMP_ACT_ERRNO` not `ERRNO`)
   - Valid actions: `SCMP_ACT_ALLOW`, `SCMP_ACT_ERRNO`, `SCMP_ACT_KILL`, `SCMP_ACT_KILL_PROCESS`, `SCMP_ACT_LOG`, `SCMP_ACT_TRAP`, `SCMP_ACT_TRACE`

3. **Syscall not blocked - container works but syscall still allowed**
   - Cause: Misspelled syscall name (no error, just no match)
   - Fix: Verify syscall names with `ausyscall --dump` or `/usr/include/asm/unistd_64.h`
   - Note: Some syscalls have multiple names (e.g., `stat` vs `__NR_stat`)

4. **32-bit compatibility issues**
   - Cause: Only specified `SCMP_ARCH_X86_64`, but process uses 32-bit syscalls
   - Fix: Include both architectures: `["SCMP_ARCH_X86_64", "SCMP_ARCH_X86"]`
   - Also consider: `SCMP_ARCH_X32` for x32 ABI

5. **`errnoRet` not taking effect**
   - Cause: `errnoRet` is only used with `SCMP_ACT_ERRNO`
   - Fix: Ensure action is `SCMP_ACT_ERRNO`, not `SCMP_ACT_KILL` or other actions
   - Common errno values: 1 (EPERM), 2 (ENOENT), 13 (EACCES), 38 (ENOSYS)

6. **Seccomp profile silently ignored**
   - Cause: Missing `linux.seccomp` path in JSON or seccomp not enabled in runc build
   - Fix: Verify with `sudo runc features | grep seccomp` to confirm support
   - Check JSON path: `linux.seccomp` (not just `seccomp` at root level)

## Notes

**Seccomp and container runtimes**:
- Docker applies a default seccomp profile blocking ~50 dangerous syscalls
- Podman and containerd also use seccomp by default
- The Docker default profile: https://github.com/moby/moby/blob/master/profiles/seccomp/default.json
- You can view Docker's profile with: `docker info --format '{{json .SecurityOptions}}'`

**Finding syscall numbers and names**:
```bash
# List all syscalls
ausyscall --dump

# Find a specific syscall number
ausyscall x86_64 reboot

# Reverse lookup (number to name)
ausyscall x86_64 169
```

**Syscall argument filtering**:
The OCI spec supports filtering based on syscall arguments:
```json
{
  "names": ["clone"],
  "action": "SCMP_ACT_ERRNO",
  "errnoRet": 1,
  "args": [
    {
      "index": 0,
      "value": 2080505856,
      "op": "SCMP_CMP_MASKED_EQ"
    }
  ]
}
```
This is advanced usage for blocking specific clone flags.

**Performance impact**:
- Seccomp adds minimal overhead (BPF is very fast)
- Overhead is per-syscall, so high-syscall workloads may see slight impact
- The kernel caches seccomp filter results for efficiency

**Seccomp and capabilities**:
- Seccomp complements Linux capabilities
- Capabilities control privilege levels; seccomp controls syscall access
- Best practice: Use both - drop capabilities AND apply seccomp

**Debugging with strace**:
```bash
# See what syscalls a command uses
strace -c ls /
```
This helps identify which syscalls to allow in an allowlist profile.

**Reference documentation**:
- `man 2 seccomp` - The seccomp syscall
- `man 2 prctl` - PR_SET_SECCOMP operation
- OCI runtime spec: https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#seccomp
- libseccomp: https://github.com/seccomp/libseccomp (what runc uses internally)

## Next

`06-network-integration.md` - Attach an OCI container to a pre-made network namespace
