# skwd-wall customisation project: handoff notes

Read this first in a new session.

## The software
- skwd-wall v2 (Linux wallpaper manager, Rust, by liixini, MIT licence). Base: **v1.0.0-beta.26**
  (still the newest upstream tag on 2026-11-04; upstream had no newer code and no official
  "drawing" transition yet).
- Four repos must sit **side by side in one folder** (Cargo paths like `../skwd-deck`):
  - `skwd-wall`: picker app + Settings UI. Binary `skwd-wall`, installed as `skwd-wall-v2`.
  - `skwd-deck`: daemon `skwd-walld`, config schema, `skwd-helm` CLI.
  - `skwd-paper`: renderers. `skwd-wall-vk` (Vulkan) draws videos, WE scenes and all
    transitions. Shaders in `crates/paper-shaders`, compiled by `crates/paper-vk/build.rs`.
  - `skwd-lens`: unchanged, use `liixini/skwd-lens` at the same tag.
- All custom work is on branch **`my-custom`** of `snietsk6205/skwd-wall`, `skwd-deck`,
  `skwd-paper` (forked at beta.26; skwd-wall from its `v2` branch).
- Config: `~/.config/skwd-wall-v2/config.json`. User machine: CachyOS + Hyprland, over SSH.

## Install / undo / update (see custom/README.md)
- `custom/install.sh`: clones/updates the three forks (my-custom) + skwd-lens at the matching
  tag into `~/.local/src/skwd-custom`, builds **all** programs, installs into
  `~/.local/opt/skwd-custom/{bin,lib/skwd-paper}` (+ `share` symlink to the official data),
  writes the systemd drop-in `skwd-walld.service.d/skwd-custom.conf` and a launcher override.
  Rolls back if the service does not come up. Re-run = update. Never touches packages.
- `custom/uninstall.sh`: removes all of it and restarts the official daemon.
- Tested end-to-end on a simulated fresh system (non-root user, fake official package,
  systemctl stand-in): install, then uninstall.

## Phase 1 (done): sketch before transition
- Renderer (`skwd-paper`):
  - `paper-vk/src/vk/draw/shared/sketch.rs`: on the first frame of a transition, freeze the
    incoming frame into an offscreen texture, analyse it once into two cached textures
    (`sketch/analysis.glsl`), then draw each sketch frame cheaply (`sketch/compose.glsl`,
    alpha-blended over the outgoing wallpaper). When done, the real transition (sand, fade or
    any effect) runs from the finished sketch. Hooked into `render_fade_to`,
    `render_effect_to`, `render_sand_to` in `targets.rs`, so every path gets it.
  - Start point: transition push block grew 64 -> 80 bytes (`u_origin`, `u_origin_on`,
    `u_seed`). Point-based effects (`ORIGIN_EFFECTS` in `paper-shaders/src/effects/vulkan.rs`)
    and all sand styles follow it. "Busiest area" = 64x36 density map read back once.
  - Settings come from env `SKWD_PAPER_SKETCH` / `SKWD_PAPER_TRANSITION_ORIGIN`
    (`paper-vk/src/sketch.rs`), no more reading config.json.
  - Test hook `SKWD_PAPER_PREVIEW_TEST=steps=N;flicker=1;media=video` for `--preview-stream`.
- Daemon (`skwd-deck`): `value/transition/sketch.rs` builds the env values; `launch.rs` sets
  them on every renderer; `policy.rs` includes them in the renderer policy signature.
  New keys: `transition.sketchVideos`, `transition.sketchScenes`, `transition.origin`
  (plus the old `sketchEnabled`, `sketchStyle`, `sketchDurationMs`).
- Picker (`skwd-wall`): Settings card + preview env (`app/update/lifecycle.rs`).

## Phase 2 (done)
- Keyboard navigation cause (upstream bug, also in the official build): Up/Down did
  `index ± columns`, wrong for Grid styles masonry/brick/justified/editorial and for hex layouts
  (arc/twist/orbit/scatter; also Right moved *down* a hex column). Fix:
  `app/update/spatial_nav.rs` (+ `spatial_nav/tests.rs`) picks the card that is really in that
  direction (Grid: `grid_placements` positions; Hex: cards on screen) with a remembered goal
  column. Grid Left/Right stay in reading order except in masonry.
- Effects studio is easier to find: E key (`Studio` default `shift+right-click, e`) and a bar
  button (`BarAction::Effects` → `Message::KeyStudio`, toggle `filterBar.show.effects`).
  Note: upstream's `InputAction::Effects` is "Choose displays" (Ctrl+click); the Palette/Duotone
  panel is the *Studio*.
- Settings → Picker → "Selection" (`components.wallpaperSelector.gridFocusWidth`, default 3 px;
  upstream marked the selected Grid card with a 1 px border) and "Settings window"
  (`settingsWindow.widthPercent/heightPercent/scale/position/columns/compactIndex`), applied in
  `frontend/settings/folio/folio.rs` (`SettingsWindow`). New sections are appended *after*
  upstream's so `is_picker_layout_section` indexes stay valid.
- All new keys are registered in `skwd-config` (key_catalog.rs, schema.rs, keys_tests.rs).

## Open items for the next session
- Selection ring: drawn only while `scene.kb_nav` is true; a mouse move clears it
  (`app/update/picker.rs`). In the headless test it never showed, likely because sway sends a
  pointer event; check on the real machine (does the ring appear after arrow keys?) and consider
  ignoring pointer events that do not move the pointer.
- Phases 3-5 not started (see REQUESTS.md and the session prompt).

## Testing in a cloud session
- Software Vulkan (llvmpipe) works; `skwd-wall-vk --preview-stream` renders frames.
- A headless sway (`WLR_BACKENDS=headless WLR_RENDERER=pixman`) runs the picker; `grim`
  screenshots; `skwd-helm ui <command>` drives it and `skwd-helm ui state` reports state.
- Known failing tests that also fail on pristine upstream in a container (environmental):
  `paper-scene hlsl::tests::packed_audio_spectrum...`, `skwd-paper manager::tests::span_spawn_failure`,
  `skwd-e2e --test semantic`.
- Disk is limited (~40 GB): use one shared `CARGO_TARGET_DIR` and `CARGO_INCREMENTAL=0`.
