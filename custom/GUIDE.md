# Guide: the custom skwd-wall on a new laptop, and keeping it up to date

Written for someone new to GitHub and to building programs from source.
Every command below works in **fish** (your shell) and in bash.

---

## 1. The idea in plain words

- **The official skwd-wall** (`skwd-wall-v2`, installed with `yay`) stays on your laptop. Keep it:
  the custom build borrows its pictures, fonts and background-service setup.
- **The custom build** is my changes on top of it. The changes are kept on **GitHub**, a website
  that stores code. Each project there is called a **repository** ("repo"). There are three:
  `snietsk6205/skwd-wall`, `snietsk6205/skwd-deck`, `snietsk6205/skwd-paper`.
- Inside a repo, a **branch** is one version of the code. All custom work is on the branch
  **`my-custom`**.
- **`git`** is the program that downloads ("clones") a repo and later downloads its changes
  ("pulls"/"fetches").
- **`install.sh`** (in `custom/`) does all the hard work for you: it downloads all the code, builds
  ("compiles") the programs, and installs them in your home folder.

You only ever type **one** command to install or to update. You never edit code yourself.

### Where things end up

| Folder | What it is |
|---|---|
| `~/.local/src/skwd-custom/` | the downloaded source code + build files (several GB) |
| `~/.local/opt/skwd-custom/bin/` | the finished custom programs |
| `~/.config/skwd-wall-v2/config.json` | your settings (shared with the official app) |

Nothing outside your home folder is changed, and your system packages are never touched.

---

## 2. New laptop: first install

### Step 1. The official skwd-wall must work first

You already have `skwd-wall-v2` installed. Check its background service is running:

```
systemctl --user status skwd-walld.service
```

You should see `active (running)` (press `q` to leave). If not:

```
yay -S skwd-wall-v2-bin skwd-lens-bin
```
```
systemctl --user daemon-reload
```
```
systemctl --user enable --now skwd-walld.service
```

### Step 2. Download the installer (you already did this)

```
git clone --branch my-custom https://github.com/snietsk6205/skwd-wall.git ~/.local/src/skwd-custom/skwd-wall
```

This copies the skwd-wall repo (branch `my-custom`) into `~/.local/src/skwd-custom/skwd-wall`.
If you run it a second time it says *"destination path already exists"*: that is fine, skip it.

### Step 3. Run the installer

```
bash ~/.local/src/skwd-custom/skwd-wall/custom/install.sh
```

**If it stops with "some build tools are missing"** (what you saw), it prints the exact command
to install them. Copy that line, run it, type your password, then run the installer again. For you
it was:

```
sudo pacman -S --needed nasm
```
```
bash ~/.local/src/skwd-custom/skwd-wall/custom/install.sh
```

It may stop again for another missing piece (for example `rustup`); do the same: run the printed
`sudo pacman ...` line, then the installer again. Two notes:

- If it says **cargo is missing** after installing `rustup`, run once:
  ```
  rustup default stable
  ```
  then the installer again. If a build later says a toolchain **1.94.1** is not installed, run
  `rustup toolchain install 1.94.1` and the installer again.
- **Never** run the installer itself with `sudo`. Only the `pacman` lines use `sudo`.

The first build takes **15-40 minutes** and the fans will spin. Leave the terminal open. At the
end it prints "Installed" with the next steps. If anything fails, it puts back what was there
before and tells you; copy the last ~40 lines of the terminal and send them to me.

### Step 4. Open the custom picker

The custom programs are **not** on your PATH, so typing just `skwd-wall-v2` opens the
**official** picker. Always use the full path:

```
~/.local/opt/skwd-custom/bin/skwd-wall-v2
```

The app launcher entry is switched to the custom build by the installer.

### Step 5. Your Hyprland shortcut

Find the line that starts the picker:

```
grep -rn skwd ~/.config/hypr/
```

Change the program in that line to the full path, for example:

```
bind = SUPER, N, exec, ~/.local/opt/skwd-custom/bin/skwd-wall-v2
```

(Your Hyprland uses the new Lua config: keep the same style as the line you found, only replace the
program name with the full path.) Then:

```
hyprctl reload
```

### Step 6. Check it is the custom build

Open Settings → Motion → Wallpaper transitions. If you see families like **Light**, **Shapes**,
**Playlist** and **Combine two**, it is the custom build.

---

## 3. Updating after a new session

After each session I **push** (upload) the new code to the `my-custom` branches on GitHub. Your
laptop does not get it automatically. To get it, run the **same installer** again:

```
bash ~/.local/src/skwd-custom/skwd-wall/custom/install.sh
```

It downloads the newest code of all three repos, rebuilds only what changed (usually 5-20
minutes) and restarts the background service. Your settings are kept.

If the picker is open while you update, close it first (or run `pkill -x skwd-wall-v2`).

You do **not** need `git pull`, `git clone` again, or any other git command for updates.

---

## 4. Undo everything (back to the official app)

```
bash ~/.local/src/skwd-custom/skwd-wall/custom/uninstall.sh
```

Then change your Hyprland shortcut back to plain `skwd-wall-v2` and run `hyprctl reload`.
To install again later, start at section 2, step 3.

---

## 5. Handy commands

| What | Command |
|---|---|
| Is the background service running? | `systemctl --user status skwd-walld.service` |
| Restart it | `systemctl --user restart skwd-walld.service` |
| Close a picker stuck in the background | `pkill -x skwd-wall-v2` |
| Speed test (about 40 s, do not touch anything) | `~/.local/opt/skwd-custom/bin/skwd-helm ui bench` |
| Which custom version is installed? | `git -C ~/.local/src/skwd-custom/skwd-wall log -1 --oneline` |
| Free disk space used by the build | `du -sh ~/.local/src/skwd-custom` |

---

## 6. Small git glossary

- **repository (repo)**: a project folder kept on GitHub, with its full history.
- **branch**: one line of versions inside a repo (`my-custom` holds our changes; `v2`/`main` are
  upstream's).
- **clone**: download a repo for the first time.
- **commit**: one saved change, with a message saying what it does.
- **push**: upload commits to GitHub (I do this at the end of each session).
- **fetch / pull**: download new commits from GitHub (the installer does this for you).

---

## 7. If something goes wrong

1. Read the line that starts with **STOPPED:**; it usually says exactly what to do.
2. If a build fails, copy the last ~40 lines of the terminal and send them to me.
3. Your working skwd-wall is not changed by a failed step, so you can keep using it.
4. Full reset of the custom build: run the uninstall command (section 4), delete the source folder
   with `rm -rf ~/.local/src/skwd-custom`, and start again at section 2, step 2.
