# skwd-wall custom build

Personal changes on top of the official skwd-wall (by liixini), kept on the
`my-custom` branch of three forks:

| Repository | What it is |
|---|---|
| `snietsk6205/skwd-wall` | the picker app and its Settings, plus these scripts |
| `snietsk6205/skwd-deck` | the background service (`skwd-walld`) and `skwd-helm` |
| `snietsk6205/skwd-paper` | the renderers (`skwd-wall-vk` draws all transitions) |
| `liixini/skwd-lens` | unchanged; only needed to build |

## Install (first time)

The official skwd-wall must be installed and working first:

```bash
yay -S skwd-wall-v2-bin skwd-lens-bin
```

```bash
systemctl --user daemon-reload
```

```bash
systemctl --user enable --now skwd-walld.service
```

Then download the custom build's scripts and run the installer:

```bash
git clone --branch my-custom https://github.com/snietsk6205/skwd-wall.git ~/.local/src/skwd-custom/skwd-wall
```

```bash
bash ~/.local/src/skwd-custom/skwd-wall/custom/install.sh
```

If it stops and prints a `sudo pacman -S --needed ...` line, run that line, then
run the installer again. The first build takes 15-40 minutes.

## Update later

```bash
bash ~/.local/src/skwd-custom/skwd-wall/custom/install.sh
```

It downloads the newest `my-custom` code and rebuilds only what changed.

## Undo everything

```bash
bash ~/.local/src/skwd-custom/skwd-wall/custom/uninstall.sh
```

## What the installer touches

Only files in your home folder:

- `~/.local/src/skwd-custom/` source code and build files (several GB)
- `~/.local/opt/skwd-custom/` the custom programs
- `~/.config/systemd/user/skwd-walld.service.d/skwd-custom.conf` makes the
  background service start the custom programs
- `~/.local/share/applications/skwd-wall*.desktop` app launcher entry

Installed system packages are never changed. If something goes wrong while
restarting, the installer goes back to the previous state by itself.

## Features

- **Sketch before transition** (Settings → Motion → Sketch before transition):
  the new wallpaper is drawn stroke by stroke in one of seven styles, then your
  chosen transition (sand, fade, ink, ...) reveals it. Videos are frozen on
  their first frame while drawing, so the drawing is smooth, and you can turn
  the sketch off for videos and Wallpaper Engine scenes.
- **Start point** for transitions that grow from a point: built-in, centre,
  random, or the busiest area of the new wallpaper.

- **Keyboard navigation fixed** in Grid (masonry, brick, justified, editorial) and Hex:
  the arrow keys move to the card that is really above/below/beside.
- **Effects studio** (Palette, Duotone, ...) on the **E** key and a new bar button.
- **Settings → Picker → Settings window**: width, height, scale, position, one column,
  narrow index. **Selection**: width of the ring around the selected card.

- **New picker layouts** (Settings → Picker → Layout & presets → Display mode):
  - **Coverflow**, with three styles: *Coverflow* (the front card faces you, the
    others turn away), *Carousel ring* (cards stand on a turning ring) and *Stack*
    (cards recede behind the front one, like a timeline). Glossy reflection and a
    blurred copy of the selected wallpaper behind the cards (both can be turned off).
    Settings: card size, angle, spacing, cards on each side, corners, step time.
  - **Showcase**: one big preview of the selected wallpaper with a filmstrip of the
    others under (or above) it; pictures cross-fade as you move.
  - Both: Left/Right, Up/Down or the wheel move; click a card to go to it; click the
    front card (or press Enter) to apply it; right-click for details.
- **Bar** (Settings → Picker → Bar and Bar order): size (0.6-1.8x), position
  (automatic, top, bottom, left or right edge), two new styles (*Pill* and *Glass*)
  and the order of the button groups (Earlier / Later buttons, plus Reset). Which
  buttons show is still under Settings → Filters.
- The selection ring in Grid stays visible when the mouse is nudged by a few pixels
  while you use the arrow keys.

Preview frames and screenshots are in `custom/previews/`.
