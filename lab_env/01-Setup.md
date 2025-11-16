Good, we’re doing something sane for once.

You want **Option 2: systemd-nspawn Arch inside Omarchy**.
Mental model:

> `systemd-nspawn` is like a **jacked chroot**:
>
> * Looks and feels like a separate Arch install
> * Has its own root filesystem, users, services
> * Shares your host kernel
> * Uses way fewer resources than a VM

So you get “Arch inside Arch” without bullying your laptop.

---

## 0. Install the tools

On Omarchy (Arch-based), run:

```bash
sudo pacman -S arch-install-scripts systemd-container
```

**What this does:**

* `arch-install-scripts` → gives you `pacstrap`, the tool used to bootstrap a fresh Arch filesystem into a directory.
* `systemd-container` → gives you `systemd-nspawn` and friends.

This combo is our “create mini Arch” + “run mini Arch” toolchain.

---

## 1. Create a home for your mini-Arch

We’ll put containers where `systemd` expects them:

```bash
sudo mkdir -p /var/lib/machines/miniarch
```

**What this is:**

* `/var/lib/machines` is the default place `systemd-nspawn` and `machinectl` look for container roots.
* `miniarch` is just the name of this specific container. Call it `goblin` if you want. I don’t care, as long as you remember it.

---

## 2. Bootstrap a minimal Arch into that directory

Now you basically “install Arch” **into a folder**, not onto a partition:

```bash
sudo pacstrap -c /var/lib/machines/miniarch base
```

**What’s happening here:**

* `pacstrap`:

  * Reads the Arch package repos
  * Installs the `base` package group
  * Puts all of that into `/var/lib/machines/miniarch`
* `-c`:

  * Tells it to avoid copying the host’s package cache. Keeps the container cleaner.
* No kernel is installed, because:

  * The container will **reuse your host kernel**, which is the whole point of this being lighter than a VM.

After this, `/var/lib/machines/miniarch` is a valid Arch root filesystem: `/bin`, `/etc`, `/usr`, etc.

You basically just created a tiny Arch system in a directory.

---

## 3. First boot into the container

Now let’s “step into” that Arch world:

```bash
sudo systemd-nspawn -D /var/lib/machines/miniarch
```

**What this does:**

* `systemd-nspawn`:

  * Sets up namespace isolation (PID, mount, etc)
  * Chroots into `/var/lib/machines/miniarch`
  * Starts `systemd` as PID 1 inside that environment
* `-D`:

  * Tells it “this directory is the root of the container”

You’ll end up with a shell prompt that is **inside** the mini-Arch.

You can verify that with:

```bash
hostname
cat /etc/os-release
ps aux
```

Different PID 1, different root FS, same kernel.

---

## 4. Do first-time setup inside the container

You’re now in “baby Arch” land. Do the usual boring stuff:

```bash
# set root password
passwd

# update packages
pacman -Syu

# maybe install some basics
pacman -S vim git sudo
```

Create a normal user (optional but civilized):

```bash
useradd -m -G wheel ian
passwd ian
```

Enable sudo for wheel (inside the container):

```bash
EDITOR=vim visudo
# uncomment:
# %wheel ALL=(ALL:ALL) ALL
```

**What’s happening conceptually:**

* This environment has its **own users**, passwords, configs, services.
* None of that touches your host’s `/etc/passwd`, `/etc/shadow`, etc.
* You can totally screw this Arch install up and your host doesn’t care.

---

## 5. Exit & re-enter like a grown-up

To exit the container:

```bash
exit
```

To get back in later:

```bash
sudo systemd-nspawn -D /var/lib/machines/miniarch
```

If you want to log directly in as your new user:

```bash
sudo systemd-nspawn -D /var/lib/machines/miniarch --user ian
```

---

## 6. Add networking (if it’s not already working)

On many setups, `systemd-nspawn` + `systemd-networkd` just works.
If you get no network inside the container, on the **host**:

```bash
sudo systemctl enable --now systemd-networkd systemd-resolved
```

Inside the container you should then see:

```bash
ping archlinux.org
```

If that works, you’re done. If not, you can bind host’s `/etc/resolv.conf` or tweak networkd configs later, but don’t overcomplicate it until you see a problem.

---

## 7. Add a shared folder (so it’s actually useful)

If you want the container to access your host’s files, run:

```bash
sudo systemd-nspawn \
  -D /var/lib/machines/miniarch \
  --bind /home/ian/projects:/home/ian/projects \
  --user ian
```

**What that does:**

* `--bind host_path:container_path`

  * Mounts your host’s `/home/ian/projects` into the container’s `/home/ian/projects`
* Your user in the container sees the files as if they were local.

This is where it starts to feel powerful:

* you keep your code on the host,
* use the container for isolation, tooling, or “clean Arch” builds.

---

## 8. Optional: make it a managed “machine”

If you want systemd to treat it like a named machine:

Create a config file on the **host**:

```bash
sudo nano /etc/systemd/nspawn/miniarch.nspawn
```

Example:

```ini
[Exec]
Boot=yes

[Files]
Bind=/home/ian/projects:/home/ian/projects

[Network]
VirtualEthernet=yes
```

Then you can:

```bash
sudo machinectl start miniarch
sudo machinectl status miniarch
sudo machinectl shell miniarch
```

Now it behaves more like a managed service/VM hybrid.

---

## TL;DR of what’s going on

* You **installed Arch into a directory** using `pacstrap`.
* You **turn that directory into an isolated environment** with:

  * Namespaces (PID, mount, etc)
  * Its own systemd, users, services
* It **shares your host kernel**, so:

  * It’s way lighter than a VM
  * Perfect for a low-resource laptop
* You can:

  * Blow it away by deleting `/var/lib/machines/miniarch`
  * Clone it by copying that directory
  * Run services inside it without polluting your host

You essentially built a **lightweight Arch lab** inside your Omarchy box.

If you want, next step is: build one specifically for dev (toolchain, dotfiles, shared projects) and one for “let me destroy this system for science.”
