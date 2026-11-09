# Preview frames

Rendered with the real renderer (`skwd-wall-vk --preview-stream`, software Vulkan).

- `phase1-sheet-old.jpg`: **before** — the old sketch patch turned sand-galaxy into a growing blob.
- `phase1-sheet-galaxy.jpg`, `phase1-sheet-galaxy-end.jpg`: **after** — the pencil sketch is
  drawn stroke by stroke, then the real sand-galaxy transition runs and ends on the wallpaper.
- `phase1-sheet-tests.jpg` (rows): charcoal sketch; ink-brush sketch; a video with the sketch
  turned off (plain inkwell at normal speed); digital style; iris from a random start point.
- `phase1-sheet-ends.jpg`: fade from the finished charcoal sketch; inkwell ripple starting at
  the busiest area of the new wallpaper.
- `phase1-settings.jpg`: the new Settings card (Motion → Sketch before transition), screenshot of the real app.
- `phase2-bar-effects-button.jpg`: the new Effects button (magic wand, between Playlists and Settings).
- `phase2-effects-studio.jpg`: what it opens (also the E key): the Effects studio with Palette, Duotone, ...
- `phase2-settings-window.jpg`: Settings window at 55 % x 72 %, on the right, scale 0.9, one column, narrow index.
- `phase3-coverflow.jpg`: the new **Coverflow** layout with its glossy reflection and the blurred
  selected wallpaper behind (screenshot of the real app, software rendering).
- `phase3-coverflow-carousel.jpg`, `phase3-coverflow-stack.jpg`: its *Carousel ring* and *Stack* styles.
- `phase3-showcase.jpg`, `phase3-showcase-strip-top.jpg`: the new **Showcase** layout (big preview
  and filmstrip), filmstrip at the bottom and at the top.
- `phase3-bar-glass-bottom-and-left-rail.jpg`: the bar in the new *Glass* style pinned to the bottom
  at 1.25x with a custom order, and as a *Pill* rail on the left edge.
- `phase3-settings-layout.jpg`: Settings → Picker → Layout with the Coverflow controls.
- `phase3-settings-bar.jpg`: Settings → Picker → Bar and Bar order.
- `phase3-grid-selection-ring.jpg`: Grid after arrow keys: the selection ring stays visible.
- `phase4-transitions.jpg`: the seven new transitions, rendered by the real renderer
  (`skwd-wall-vk --preview-stream`), 7 frames each from start to end: ripple, honeycomb, burn,
  page-curl, blinds, slide, vortex.
- `phase4-cascade-opening.jpg`: the new *Cascade* opening animation caught mid-way (left cards in place, right ones still rising in).
- `phase5-minimal-bar-layout-button.jpg`: the bar after Quick setup → Minimal, with the new layout button (arrows, left of Settings).
- `phase6-performance-overlay.jpg`: the performance overlay (top right). Cloud screenshot with software rendering, so FPS shows "--" or 0 and CPU is high; on a real GPU it shows the real numbers.
- `phase6-settings-power.jpg`: Settings → Performance → Power with the new Low-power mode and Performance overlay switches.
- `m1b-bench-cloud.txt`: a full report of the new one-command benchmark (`skwd-helm ui bench`), made in the cloud with software rendering at 512x288, so the times are far slower than on a real graphics chip; it shows what each line looks like.
- `m2-*.jpg` (M2, window size everywhere): each panel that opens over the picker, at a **small**
  Settings window (50 % x 60 %, scale 0.85, position Right) on the left and a **large** one
  (96 % x 94 %, scale 1.2, Centre) on the right, screenshots of the real app at 1280x720 (software
  rendering): `m2-settings`, `m2-layout-studio` (side panel, stops at about half the screen so the
  picker stays visible; Right puts it on the right), `m2-playlists`, `m2-card-picker`,
  `m2-theme-designer`, `m2-effects-studio`, `m2-schedule`, `m2-tag-cloud` (Centre keeps it under
  the cards), `m2-downloads` (no internet in the cloud, so it shows a network error),
  `m2-audio-mixer`, `m2-scene-properties` (a made-up test scene).
- **M3 transitions** (real renderer, `skwd-wall-vk --preview-stream`, software Vulkan; 9 frames per
  row from 10 % to 90 %, old picture = plasma with "OLD", new picture = green with a sun and a
  triangle):
  - `m3-light.jpg`: the new *Light* family: light-leak, flashbulb, aurora, prism, god-rays,
    neon-edges, sunrise.
  - `m3-shapes.jpg`: the new *Shapes* family: diamonds, triangles, checkerboard, stripes,
    halftone, spiral-tiles, shutter, kaleidoscope.
  - `m3-warp-break.jpg`: liquid, lens, flag, tunnel, accordion (Warp); glass, paper-tear, crt,
    datamosh (Break up).
  - `m3-fade-wipe.jpg`: luma-fade, dream-blur, film-dissolve (Fade); clock-wipe, barn-doors,
    curtain, wave-wipe (Wipe).
  - `m3-combos.jpg`: every way to combine two: then (burn, then ripple from burn's frozen
    half-way frame), blend, split left|right, top/bottom, diagonal, circle, checker, mask
    (honeycomb's shape, burn's look), sweep, both-lighter, both-darker.
- **M3 Settings** (Motion → Wallpaper transitions, screenshots of the real app at 1280x720):
  - `m3-settings-families.jpg`: the pinned live preview with its caption, the new families
    (Playlist, Combine two, Light, Shapes).
  - `m3-settings-library-light.jpg`: the transition library filtered to *Light*: one row per
    transition with ▶ Preview, Favourite, + Playlist, Use.
  - `m3-settings-library-search.jpg`: the library searching for "tiles".
  - `m3-settings-playlist.jpg`: *Play in turns*: order (in order / shuffled / random, no
    repeats) and the list with Earlier / Later / Remove; the preview shows the first entry.
  - `m3-settings-combo.jpg`, `m3-settings-combo2.jpg`: *Combine two*: family then transition for
    each part, and how to combine them.

## R1: Settings redesign (1280x720 unless noted)

- `r1-home.jpg`: Home with the current (video) wallpaper behind pinned cards and Recently changed.
- `r1-menu-list.jpg`, `r1-menu-grid.jpg`: a group menu as a list (Motion) and as a grid (Wallpapers).
- `r1-page-simple.jpg`, `r1-page-advanced.jpg`: Playback & Power › Power & battery in Simple view
  (3 advanced settings hidden) and in Advanced view; the dot and ↺ mark a changed value.
- `r1-palette.jpg`: the command palette (Ctrl+K) with "battery saver"; Tab flipped the first result.
- `r1-toast.jpg`: the undo note after that change.
- `r1-bubble.jpg`, `r1-bubble-pinned.jpg`: the preview bubble next to a transition (small, pinned).
- `r1-edit-cards.jpg`: Home in Edit cards mode.
- `r1-large-home.jpg`, `r1-large-page.jpg`: the same at 2560x1440.
