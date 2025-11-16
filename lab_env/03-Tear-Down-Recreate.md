Good, you’re thinking about how to blow it up and start fresh. That’s the correct sysadmin instinct.

You’ve basically got **three levels** of “clean up miniarch”:

1. Stop it
2. Nuke it
3. Make it easy to recreate

I’ll walk through all three.

---

## 1. Stop `miniarch` cleanly

If it’s running via `systemd-nspawn` directly, you just:

```bash
# inside the container
exit
```

If you started it with `machinectl` or have it booted as a managed machine:

```bash
sudo machinectl list
sudo machinectl terminate miniarch
```

* `terminate` sends a polite SIGTERM to PID 1 in the container.
* If it’s stubborn, you can `kill miniarch` via `machinectl` but usually `terminate` is enough.

---

## 2. Completely remove the container

Your container root lives here (from earlier):

```bash
/var/lib/machines/miniarch
```

To **wipe the whole thing**:

```bash
sudo rm -rf /var/lib/machines/miniarch
```

That’s it. Your “installed Arch inside a directory” is gone.

If you were using a config file for nspawn:

```bash
sudo rm -f /etc/systemd/nspawn/miniarch.nspawn
```

And if `machinectl` still shows a stale entry:

```bash
sudo machinectl remove miniarch
```

That gets rid of the rootfs and metadata.
Think of it as `rm -rf /` but socially acceptable.

> Obvious note: double-check the path before you hit enter.
> `rm -rf /var/lib/machines/miniarch` good.
> `rm -rf /var/lib/machines` bad.
> `rm -rf /var/lib` catastrophic.

---

## 3. Recreate `miniarch` from scratch

Same steps as before, just re-run them.

### 3.1. Create the directory again

```bash
sudo mkdir -p /var/lib/machines/miniarch
```

### 3.2. Bootstrap a fresh Arch

```bash
sudo pacstrap -c /var/lib/machines/miniarch base
```

You now have a brand new baby Arch in that folder.

### 3.3. Jump into it

```bash
sudo systemd-nspawn -D /var/lib/machines/miniarch
```

Inside, do your usual:

```bash
passwd
pacman -Syu
pacman -S vim git sudo python
# etc
```

If you had a `/etc/systemd/nspawn/miniarch.nspawn` config you liked, recreate it once and reuse.

Example:

```ini
# /etc/systemd/nspawn/miniarch.nspawn
[Exec]
Boot=yes

[Files]
Bind=/home/ian/projects:/home/ian/projects

[Network]
VirtualEthernet=yes
```

Then:

```bash
sudo machinectl start miniarch
sudo machinectl shell miniarch
```

---

## 4. “Lab mode”: fast reset pattern

Since you’re doing a **learning lab** and will break stuff on purpose, you can adopt a quick pattern:

### Option A: Hard reset when you’re done

When a lab gets messy:

```bash
sudo machinectl terminate miniarch 2>/dev/null || true
sudo rm -rf /var/lib/machines/miniarch
sudo pacstrap -c /var/lib/machines/miniarch base
sudo systemd-nspawn -D /var/lib/machines/miniarch
```

You can even wrap that in a script:

```bash
sudo tee /usr/local/bin/reset-miniarch << 'EOF'
#!/usr/bin/env bash
set -e

NAME=miniarch
ROOT=/var/lib/machines/$NAME

sudo machinectl terminate "$NAME" 2>/dev/null || true
sudo rm -rf "$ROOT"
sudo mkdir -p "$ROOT"
sudo pacstrap -c "$ROOT" base
echo "Recreated $NAME at $ROOT"
EOF

sudo chmod +x /usr/local/bin/reset-miniarch
```

Then any time you want a fresh lab:

```bash
reset-miniarch
sudo systemd-nspawn -D /var/lib/machines/miniarch
```

### Option B: Use a “golden image” copy

If you have a nice “base lab setup” you like (tools installed, user created, etc):

1. Set up `miniarch` once the way you want.
2. Stop it.
3. Copy it to a template:

```bash
sudo systemctl stop systemd-nspawn@miniarch.service 2>/dev/null || true
cd /var/lib/machines
sudo cp -a miniarch miniarch-template
```

When you trash the live one:

```bash
cd /var/lib/machines
sudo rm -rf miniarch
sudo cp -a miniarch-template miniarch
```

Now you’re back to your known-good baseline instantly.

---

## 5. What this gives you for your lab

For your namespace/cgroup experiments:

* You can experiment aggressively inside `miniarch`.
* If you:

  * misconfigure systemd
  * wreck /etc
  * mess with cgroups in dumb ways
  * install cursed kernels/tools

You just:

```bash
sudo rm -rf /var/lib/machines/miniarch
sudo pacstrap -c /var/lib/machines/miniarch base
```

and you’re back to clean state.

So yes, you get a **resettable sandbox** that still behaves like a real Arch system, without sacrificing your laptop’s actual sanity.
