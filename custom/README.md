# skwd-wall custom build

Personal changes on top of the official skwd-wall (by liixini), kept on the
`my-custom` branch of three forks:

| Repository | What it is |
|---|---|
| `snietsk6205/skwd-wall` | the picker app and its Settings, plus these scripts |
| `snietsk6205/skwd-deck` | the background service (`skwd-walld`) and `skwd-helm` |
| `snietsk6205/skwd-paper` | the renderers (`skwd-wall-vk` draws all transitions) |
| `liixini/skwd-lens` | unchanged; only needed to build |

New to GitHub or setting up a new laptop? Read the step-by-step guide first:
[`GUIDE.md`](GUIDE.md).

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

- **Seven new wallpaper transitions** (Settings → Motion → Wallpaper transitions, and in the
  Random mix): *Ripple* (water rings), *Honeycomb* (hexagon tiles turn over in a wave, like
  Noctalia's), *Burn* (the old picture burns away along glowing embers), *Page curl*, *Blinds*
  (slanted shutters open one after another), *Slide* (the new picture slides over the old one)
  and *Vortex* (swirls out of the old picture into the new one). Ripple, Honeycomb and Vortex
  follow the transition start point (centre, random, busiest area). They also work after the
  sketch.
- **Layout button** in the bar (it switches to the next layout, like the M key) and
  **Settings → Picker → Bar → Quick setup**: *Minimal* (only the essential buttons) or
  *Everything*.
- **Two new opening animations** (Settings → Launch → Animation): *Cascade* (cards rise in one
  after another from the left) and *Bloom* (cards grow in from the centre outwards).

- **Faster settings** (M1): changing a setting no longer waits for the disk. The config file is
  written in the background (quick clicks are merged into one write), so the picker stays
  smooth even on btrfs.
- **Performance overlay** (Settings → Performance → Power): FPS, frame time, CPU use and how
  long the last settings change took, in a corner.
- **Benchmark**: moves the picker by itself for about 35 seconds and prints a report (also saved
  in `~/.cache/skwd-wall-v2/bench/latest.txt`). It never changes your settings.
- **Smoother layout switches** (M1b): the cross-fade between two layouts no longer redraws the old
  layout on every frame (it is a frozen picture, so it is drawn once), its two screen-sized
  helper images are reused for 10 seconds instead of being made again for every switch, and the
  cross-fade shader is prepared when the picker starts instead of during the first switch.

- **Window size everywhere** (M2): Settings → Picker → Settings window now sets the size, place
  and scale of **every panel that opens over the picker**, not only Settings: the layout studio
  (Shape), playlists (and its "Add to playlist" card picker), theme designer, effects studio,
  schedule editor, tag cloud, scene properties, downloads browser and audio mixer. The picker itself
  still fills the screen. Width and height are shares of the screen (0 = each panel's own
  automatic size), Scale is the size of text and controls (0 = your UI scale), Position is
  Centre, Left, Right, Top or Bottom. Two panels behave a little differently on purpose:
  - the **layout studio** is a side panel next to the live picker, so it never gets wider than
    about half the screen; Left or Right picks its side (the picker moves out of its way);
  - the **tag cloud** stays in its usual place under the cards when Position is Centre, and its
    own width setting (Settings → Picker → Tag panel), if you set one, wins over the window width.

- **Transitions** (M3), all in Settings → Motion → Wallpaper transitions:
  - **31 new transitions** and two new families. *Light*: light leak, flashbulb, aurora, prism,
    god rays, neon edges, sunrise. *Shapes*: diamonds, triangles, checkerboard, stripes,
    halftone, spiral tiles, camera shutter, kaleidoscope. Also liquid, lens, flag, tunnel,
    accordion (Warp), glass, paper tear, CRT switch-off, datamosh (Break up), shadows-first
    fade, dream blur, old-film dissolve (Fade), clock wipe, barn doors, curtain, wave (Wipe).
    God rays, diamonds, shutter, lens, glass and clock wipe follow the transition start point.
  - The **live preview stays at the top** of the page while you scroll, with the name and a
    one-line description of what it plays.
  - **Transition library**: every transition with a short description, a **search** box
    (name or any word, e.g. "light", "tiles", "water") and filters (All, Favourites, In
    playlist, or one family). Each row has **▶ Preview** (plays it in the preview without
    choosing it), **Favourite**, **+ Playlist** and **Use**.
  - **Play in turns** (family *Playlist*): one transition per wallpaper change, **in order**,
    **shuffled** (each once per round) or **random, no repeats** (never one of the last few).
    Move entries earlier/later, remove them, add all favourites, or clear the list. An empty
    list plays Random.
  - **Combine two** (family *Combine two*): pick two transitions and how they work together:
    *one after the other* (the first stops half-way, that moment is frozen, the second finishes
    from it), *blend*, *split screen* (left|right, top/bottom, diagonal, circle, checker),
    *first is a mask* (the first gives the shape, the second the look), *sweep*, *both,
    lighter wins*, *both, darker wins*. **Use** plays it; **+ Playlist** keeps it in the list
    next to others (so the list can hold several different combinations). *Random Mix* (in
    the library) combines two random transitions in a random way every time.
  - All of it also works after the "sketch before transition".

- **Redesigned Settings** (R1 + R1b). Everything from before is still there; it is arranged and
  drawn differently:
  - **Top bar:** Home, Look, Wallpapers, Motion, Playback & Power, Automation & System, as small
    text in the middle. Point at a group (or click it, or use the arrow keys) and a **menu** shows
    its pages as numbered cards. **Find** and **Simple / Advanced** are on the right.
  - **Pages:** a big title, one sentence on the right, then the settings as plain rows: the name
    and one sentence on the left, the control on the right. A small **dot** means you changed it;
    the **↺** next to it puts the default back. **more…** opens a longer explanation where a
    setting is confusing. **Back** and where you are (for example *Motion / Transitions*) are
    at the top; **Esc** goes back one step and closes Settings from Home.
  - **Simple / Advanced:** Simple hides the settings most people never need. Each page says how
    many it hides and has *Show them*.
  - **Cards for everything you choose by eye:** transitions, playlist entries, ways to combine
    two transitions, sketch styles, bar styles and theme presets. A card has a number, its name,
    the **preview inside**, and small tags (family, *Playing now*, *In playlist*, *In use*).
    Point at a card (or move to it with the arrow keys): its outline lights up, the preview
    plays, and its buttons appear: **Preview big**, **Use**, **+ Playlist**; the **+** in the
    corner is *Favourite*. Above the transition cards: a search field and the filters.
  - **Preview big** shows one transition large, live, until you press **Esc**.
  - The small previews in the cards are drawn **once**, in the background, the first time you
    open such a page (a few seconds on a normal graphics chip), then kept in
    `~/.cache/skwd-wall-v2/settings-strips/`. They are not drawn during the benchmark, in
    Low-power mode or with Battery saver. After an update of the renderer they are drawn again.
  - **Home:** your current wallpaper behind everything (a video wallpaper plays), your pinned
    pages as cards at the bottom, *Recently changed* at the top (click one to go back to it).
    *Edit cards*: the arrows on a card move it, *Remove* takes it off, *Add a card* lets you
    pick any page from the menus. The video pauses by itself in Low-power mode, with Battery
    saver and whenever Settings is closed; *Pause background* keeps it still.
  - **Find anything: Ctrl+K** (also straight from the picker). Type a few letters of a setting,
    page, transition, layout or action. **Tab** changes a setting right in the list, **Enter**
    goes there, **Esc** closes.
  - **Undo:** after each change a small note says *Changed X to Y* with **Undo**; **Ctrl+Z**
    undoes the last change, again and again (30 steps).
  - **Keyboard:** arrows move between the bar, a menu, the rows and across a card grid. On a
    card or a setting shown as cards, **Enter** steps into its buttons or cards, the arrows
    choose, **Enter** uses it, **Esc** steps out. **Tab** jumps over a card grid.
  - **Look -> Settings appearance** changes this window: card size (small, medium, large),
    cards per row (automatic, 2, 3, 4), outlined or filled cards, where previews play (in the
    card, or in a small live preview that **follows the cursor**), the faint grid pattern, a
    blurred top bar, menus as a list or a grid, and animations on or off.
  - **Look -> Settings window** still sets the size, place and scale (M2), what Settings opens
    on (Home or where you left off) and the Home background.
  - Settings is always dark and uses one accent colour: your theme's main colour.
  - Commands (for scripts): `~/.local/opt/skwd-custom/bin/skwd-helm ui page motion/transitions`
    opens a page (also `ui page home`, `ui menu motion`).

### Important: use the full path for the custom programs

The custom programs live in `~/.local/opt/skwd-custom/bin/`, which is not on your PATH. Typing
just `skwd-helm` or `skwd-wall-v2` runs the **official** programs.

### Run the benchmark (one command)

Type this in a terminal on the laptop or over SSH, then do not touch the keyboard or mouse for
about 40 seconds:

```bash
~/.local/opt/skwd-custom/bin/skwd-helm ui bench
```

It does everything by itself: if the official picker (or an older custom one) is still running
in the background, it closes it first; it starts the custom picker, moves it around, prints the
report, saves it in
`~/.cache/skwd-wall-v2/bench/latest.txt` (older runs: `ls ~/.cache/skwd-wall-v2/bench/`) and
closes the picker again, so you get your keyboard back. Over SSH it finds your screen by itself.

To see how fast the picker is on the other graphics card (your RTX 3050 instead of the built-in
Intel one), run it once more with:

```bash
~/.local/opt/skwd-custom/bin/skwd-helm ui bench --gpu high
```

How to read the report:

- `frames` and `frame gap`: the frames really drawn and the time between them while something
  moves (pauses between two animations are not counted). On a 144 Hz screen a perfect frame gap
  is 6.9 ms; 13.9 ms means a frame missed one refresh.
- `click -> frame`, `key -> frame`, `switch -> frame`: how long after the action the picture was
  ready.
- `settings-click` and `settings-click-old`: the new way of saving settings and the old one
  (before M1), measured in the same run on your disk.
- `navigate` and `navigate-uncapped`: moving with your FPS limit and without it. If the second
  is clearly faster, the report tells you which Max FPS to set.
- Under `layouts`, one block per layout: how long its first frame took and the slowest frame
  after the switch, with what the renderer did in it and how busy the UI thread was (busy
  close to the frame time = work on the processor; much lower = waiting for the graphics chip).
- `In short:` at the end says it in plain words.

Your Hyprland shortcut must also use the full path, or it opens the official picker. Find it
with `grep -rn skwd ~/.config/hypr/` and change it to
`bind = SUPER, N, exec, ~/.local/opt/skwd-custom/bin/skwd-wall-v2`, then run `hyprctl reload`.

Preview frames and screenshots are in `custom/previews/`.
