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

## Phase 3 (done): layouts and bar
- Selection ring: could not reproduce the "never shows" note. In headless sway the ring shows
  after arrow keys (`skwd-helm ui state` now reports `keyboard_focus` and `hover`). Pointer
  jitter < 2 px was already ignored; now, while `kb_nav` is set, the pointer must move
  `KEYBOARD_POINTER_SLACK` (12 px) before it takes over (`app/update/picker.rs`).
- New modes `Mode::Coverflow` ("coverflow", alias "carousel") and `Mode::Showcase`
  ("showcase", alias "filmstrip") in `contracts/picker/mode.rs`; drawing in
  `app/scene/core/view_custom.rs`; params in `frontend/scene/layout/custom.rs`
  (`CoverflowParams` with `CoverflowStyle::{Coverflow, Carousel, Stack}`, `ShowcaseParams`);
  config readers in `infrastructure/config/picker/accessors/custom_layouts.rs`; Settings cards in
  `frontend/settings/tabs/layouts_tab.rs`; presets snapshot both. They behave like Depth:
  camera spring follows `current`, `card.selection` springs drive the focus/cross-fade.
- Coverflow uses projected quads (like Collection) and a new shader flag `REFLECTION`
  (`rendering/scene/pipeline/geometry.rs`, `shaders/item.wgsl`): flipped texture, dark glossy
  fade, opaque enough that nearer reflections hide farther ones. The row is fitted by projecting
  the outer poses (`coverflow_fit`).
- Both layouts can draw the blurred "ambient backdrop" (the Hand backdrop shader). They borrow
  `hand.backdrop_*` state and tick it in `tick_custom_layouts`.
  NOTE for testing: with llvmpipe the backdrop blur samples BC7 textures in software and a frame
  takes ~4 s; switch `coverflowBackdrop`/`showcaseBackdrop` off for headless tests.
- Bar: `filterBar.order` (comma list, parsed by `frontend/ui/bar/order.rs`, groups appended in
  upstream order), `filterBar.scale` (0.6-1.8), `filterBar.anchor` (auto/top/bottom/left/right,
  `anchored_bar` in `app/view/chrome.rs`, `filter_bar_effective_vertical`/`filter_bar_menu_up`),
  new `BarVisualStyle::{Pill, Glass}` (values "pill"/"glass" of `filterBar.visualStyle`; "match"
  maps Coverflow/Showcase to pill). Settings → Picker → Bar / Bar order (`window_tab.rs`,
  `ActionId::MoveBarGroup`, `ActionId::ResetBarOrder`).
- New keys registered in skwd-deck `skwd-config` (key_catalog, schema specs, keys_tests).
  Mode labels `settings-selector-mode-coverflow/showcase` added to all 14 locales (English text
  everywhere: untranslated).

## Open items for the next session
- Phases 4-5 (see REQUESTS.md and the session prompt) unless noted as done below.

## Testing in a cloud session
- Scripts from the last session (recreate in the scratchpad): a headless sway + `skwd-walld` +
  `skwd-wall` with `HOME` pointing at a fake home whose config sets `paths.wallpaper` to a folder
  of generated test images (ImageMagick gradients), then `skwd-helm ui set <key> <value>`,
  `skwd-helm ui nav right`, `skwd-helm ui tab picker`, `skwd-helm ui section N`, and `grim`.
  Wait for the picker to go idle (CPU ~0) before `grim`.
- Software Vulkan (llvmpipe) works; `skwd-wall-vk --preview-stream` renders frames.
- A headless sway (`WLR_BACKENDS=headless WLR_RENDERER=pixman`) runs the picker; `grim`
  screenshots; `skwd-helm ui <command>` drives it and `skwd-helm ui state` reports state.
- Known failing tests that also fail on pristine upstream in a container (environmental):
  `paper-scene hlsl::tests::packed_audio_spectrum...`, `skwd-paper manager::tests::span_spawn_failure`,
  `skwd-e2e --test semantic`.
- Disk is limited (~40 GB): use one shared `CARGO_TARGET_DIR` and `CARGO_INCREMENTAL=0`.
