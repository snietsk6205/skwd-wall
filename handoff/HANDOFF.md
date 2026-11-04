# skwd-wall customisation project: handoff notes

Read this first in a new session. It replaces the old conversation.

## The software
- skwd-wall v2 (Linux wallpaper manager, Rust, by liixini, MIT licence). Base version: **v1.0.0-beta.26** tags.
- It is split across four repos that must sit **side by side in one folder**, because Cargo uses paths like `../skwd-deck`:
  - `skwd-wall`: the picker app plus its Settings UI. Binary `skwd-wall`, installed as `skwd-wall-v2`.
  - `skwd-deck`: the daemon `skwd-walld` (a systemd user service), config schema, `skwd-helm` CLI, transition list (`crates/wall-proto/src/transitions.rs`).
  - `skwd-paper`: the renderers. `skwd-wall-vk` (Vulkan) draws videos, Wallpaper Engine scenes and all transitions. Shaders are in `crates/paper-shaders`; `crates/paper-vk/build.rs` compiles them with `glslc`.
  - `skwd-lens`: visual search. Not modified, only needed to build.
- Config: `~/.config/skwd-wall-v2/config.json`.
- User's machine: Arch-based (CachyOS), Hyprland. Opens the picker with `SUPER+N` → `$HOME/.local/opt/skwd-sketch/bin/skwd-wall-v2`, random wallpaper with `SUPER+W` → `skwd-helm random`.

## What exists already (folder `skwd-sketch-v2/` next to this file)
- `skwd-paper.patch`, `skwd-deck.patch`, `skwd-wall.patch`: the "Sketch before transition" feature:
  - Settings: Motion → Sketch before transition.
  - Config keys `transition.sketchEnabled`, `sketchStyle` (pencil, digital, chalk, blueprint, ink-brush, charcoal, notebook), `sketchDurationMs`.
- `install-sketch.sh` / `uninstall-sketch.sh` / `update-picker.sh`:
  - Clones the four repos at the user's installed version, applies the patches, and builds into `~/.local/opt/skwd-sketch/bin`.
  - Redirects the daemon with a systemd drop-in, `~/.config/systemd/user/skwd-walld.service.d/sketch-transition.conf`, which overrides `ExecStart` and sets `SKWD_WALL_PAPER_VK` / `SKWD_PAPER_VK_BIN`.
  - Adds a launcher override in `~/.local/share/applications`.
  - System packages are never touched. Uninstall reverts everything.
- How the sketch works now:
  - `paper-shaders/src/effects/sketch.rs` generates a "sketch variant" of every effect shader. The variant draws the sketch for the first `split` of the timeline, then runs the effect with `texture(u_tex_old,…)` replaced by the finished sketch `sk_final()`.
  - Style and split are packed into the `u_pid_base` push constant.
  - The daemon lengthens `duration_ms` by the sketch time, capped at 10 s.
  - The renderer reads config.json itself (`paper-vk/src/sketch.rs`, re-read every 400 ms). This is a hack and should be passed through the daemon instead.

## Known bugs and their causes (user report, verified in code)
1. **Sand transitions, and plain fades, all became an expanding circle.**
   - Cause: `paper-vk/src/vk/draw/shared/targets.rs`, where `render_sand_to` and `render_fade_to` redirect to `SKETCH_REVEAL_INDEX` whenever the sketch is on.
   - Fix: render the sketch into an offscreen RGBA target, then run the real sand/fade path with that texture as source A.
2. **Every post-sketch transition starts from the same place.**
   - Cause: positions are hard-coded. The sketch and reveal focus is `(0.5, 0.45)`; upstream effects have their own constants (e.g. inkwell impact `(0.35, 0.4)`).
   - Wanted: a random start point, or one at the area where the drawing is densest (edge-density centroid). Expose the choice in Settings.
3. **Video wallpapers lag and the sketch shimmers.**
   - Cause: the sketch is recomputed every frame from the *live* decoded video frame. That's about 25+ texture reads per pixel, doubled by the NV12→RGB conversion, at full resolution, while two videos decode for up to 10 s.
   - Fix: freeze the incoming video's first frame for the sketch phase. Compute edges and draw order once into a cached texture (this matches the upstream author's "what to draw where in what order is cached"). Add an option to skip the sketch for videos and scenes.
4. Settings section initially didn't show: `skwd-wall/src/frontend/settings/tabs/builder.rs::compose_motion` only shows cards it takes by title. Already fixed in `skwd-wall.patch`.

## Context
- Upstream author (u/scandii) announced an official "drawing" transition for "this weekend" in late Oct 2026, with configurable canvas/ink colours that can follow the colour scheme. Check whether it has landed before redoing work; prefer building on it.
- The Effects panel (Palette, Duotone, …) exists upstream. It is opened from a wallpaper card's **back side → Effects**, not from Settings.
