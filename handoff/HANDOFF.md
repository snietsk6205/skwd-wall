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

## Phase 4 (done): transitions and UI animations
- Renderer (`skwd-paper`): `paper-shaders/src/effects/resources/custom.rs` adds `ripple`,
  `honeycomb`, `burn`, `page-curl`, `blinds`, `slide`, `vortex` (GLSL 330 dialect like upstream;
  `paper-vk/build.rs` compiles every entry of `EFFECTS` with glslc). `ripple`, `honeycomb`,
  `vortex` have start-point edits in `ORIGIN_EFFECTS` (`effects/vulkan.rs`). "random" picks
  from them automatically.
- Daemon (`skwd-deck`): `wall-proto/src/transitions.rs` lists them (family Warp/Wipe/Break), so
  the daemon accepts them and the Settings list/families show them (labels come from the key).
- Picker: `LaunchAnim::{Cascade, Bloom}` (`frontend/scene/transition.rs`, `stagger_entrance`):
  per-card delay by x (Cascade) or distance from the centre (Bloom); swaps the layouts'
  `entrance` opacity for the card's own; moves projected quads too; the backdrop only fades.
  Locale keys `settings-launch-animation-cascade/bloom` in all 14 locales (English text).
- Preview sheets: a small script renders N frames via `skwd-wall-vk --preview-stream NEW
  --transition-from OLD --shader NAME --preview-size 384x216 --preview-once --stream-no-header`
  with `SKWD_PAPER_PREVIEW_TEST=steps=30` (raw RGBA on stdout) and tiles them with ImageMagick.

## Phase 5 (done): designer pass
- Research, ranked list of what was added and what to do next: `handoff/DESIGN.md`.
- Bar layout button: `BarAction::CycleLayout` → `Message::CycleMode`, group "layout" in
  `bar/order.rs`, key `filterBar.show.layout` (default true; Filter & search → Buttons).
- Bar quick setup: `ActionId::BarPreset(minimal)` (Settings → Picker → Bar).

## Gotcha: give the user full paths
- The installer does **not** put `~/.local/opt/skwd-custom/bin` on PATH, so `skwd-helm` and
  `skwd-wall-v2` typed in a shell are the **official** programs (the user hit this in M1:
  `skwd-helm ui bench` printed `sent: bench` from the official helm). Always give
  `~/.local/opt/skwd-custom/bin/<program>`. The picker stays resident after closing: stop it
  with `pkill -x skwd-wall-v2` before starting it with different env. Over SSH, start the
  picker with `WAYLAND_DISPLAY=wayland-1 setsid -f <full path> >/dev/null 2>&1` (socket name
  from `ls /run/user/(id -u)/`). Do NOT use `hyprctl dispatch exec`: the user's Hyprland uses
  the new Lua config, where `hyprctl dispatch` expects Lua (`hl.dispatch(...)`). The user's
  shell is fish (`(cmd)` substitution; `VAR=value cmd` works).
- On the user's machine the picker grabs the keyboard and covers the screen while open, and
  closing it exits the process (`exit_picker` → `hard_exit`), so the user cannot type a second
  command while it is open. Benchmark one-liner that works (fish and bash):
  `pkill -x skwd-wall-v2; WAYLAND_DISPLAY=wayland-1 ~/.local/opt/skwd-custom/bin/skwd-wall-v2 >/dev/null 2>&1 & sleep 3; ~/.local/opt/skwd-custom/bin/skwd-helm ui bench`
  User's machine (from their log): NVIDIA GPU, 988 wallpapers, Noctalia shell also running.
- Possible small improvement (not done): `skwd-helm ui bench` could start the picker itself when
  none is running.
- Possible small improvement (not done): `skwd-helm ui bench` could detect the official picker
  (it gives an empty reply to `perf-query`) and say so instead of waiting 2 minutes.

## Session rule (from the user, applies to every session)
- **One milestone per session.** When it is done: full build, tests, screenshots, commit, push
  to `my-custom`, update this file, then stop and give the user the exact prompt for the next
  session in a copyable box. Stop early (work pushed + noted here) if the session gets long.

## Roadmap agreed with the user (priority order; one per session)
- **M1 Speed (done, Phase 6 below).**
- **M2 Window size everywhere (next).** The Settings window size/position/scale must apply to
  every panel that opens over the picker: Shape/layout studio, playlists, card picker, theme
  designer, effects studio, schedule editor, tag cloud, scene properties, downloads browser,
  audio mixer. The picker itself keeps filling the screen.
  Notes from M1's reading (saves time): most panels build their sheet with
  `frontend/ui/misc/folio.rs` `folio_sheet()` and size it with `frontend/ui/misc/layout.rs`
  `folio_sheet_dims(viewport, scale)` (1390x870 x scale, minus 22 px), then centre it
  (`.center_x/.center_y`) over an `inert_backdrop` scrim. Users: playlists, theme designer,
  schedule editor, scene properties, audio panel, effects `displays.rs`; the effects studio
  (`effects/presentation/studio.rs`) and `monitor.rs` copy the same pattern by hand. The
  Settings window logic lives in `frontend/settings/folio/folio.rs` `struct SettingsWindow`
  (dims/align/edge_gap). Plan: move it to `frontend/ui` as a shared `SheetWindow` (thread-local
  set by `app/view/composition.rs` each view, since frontend may not read config), make
  `folio_sheet_dims` and a new `place_sheet()` use it, pass the window scale to every panel in
  `app/view/panels.rs`. Separate cases: the Shape/layout studio (`picker_layout_workbench`,
  full-height side panel; `post_update.rs` insets the picker by `picker_layout_studio_width`),
  the downloads browser (`frontend/browser/view.rs`), tag cloud, card picker.
- **M3 Transitions:** many more original ones; families, favourites, search, live preview per
  transition; play several in turns (in order / shuffled / random without repeats); combine two
  (A then B, blend, split screen, one as a mask for the other, more); reuse the sketch code's
  offscreen freeze/cache for multi-pass.
- **M4 Sketch:** genuinely different original styles (only pencil is good today); controls for
  drawing order (by brightness/depth layers, edges first, one continuous line, scribble,
  stipple), stroke width, hatch direction, paper texture, colour vs mono, speed; tick several
  styles/orders and pick a different one each time.
- **M5 Style editor and themes** (wmkeyboard's approach, ideas only, MIT Kotlin): style editor
  with live preview for every visual part, per-element styles, built-in presets, export/import/
  share themes; a settings search/reference that finds every option; simple vs advanced view;
  every layout deeply customisable (cards, spacing, shapes, shadows, glow, tilt, reflections,
  motion curves).
- **M6 Bar:** three clearly different bars (floating dock, minimal top strip, side rail with
  labels) as presets with screenshots; per-button icon/label/size/colour/hover effect; button
  order by drag.
- **M7 New features:** research, ranked list, build the best: full-resolution Space-bar peek
  (thumbnails are 640x360), colour-wheel mood layout, wallpaper history scrub, day/night pairs,
  shareable theme packs.

## Phase 6 = M1 (done): speed, performance overlay, benchmark, low-power mode
- **Measured cause of the settings lag:** every settings click saved `config.json` on the UI
  thread: clone + pretty JSON + write + **two fsyncs** (file and folder). The benchmark measured
  27 ms per save in the cloud container; on btrfs (CachyOS default) or a busy disk it is often
  more. Each save also made the daemon reload its whole config (and refresh power, playback,
  rotation, weather, playlists) and the picker re-read and re-parse the file it had just
  written. Everything else a click does on the CPU is small: update 0.2-0.6 ms,
  `init_settings_inputs` 0.3 ms, building the Settings view 0.3 ms, `apply_layout` < 0.2 ms.
  Idle picker and daemon: 0 CPU ticks in 5 s (no idle work, no continuous redraw).
- **Fix: background config writer** `src/infrastructure/config/writer/` (+ tests). `persist()`
  now serialises and hands the text to a writer thread; saves within 120 ms are merged into one
  write (max delay 600 ms), so a burst of clicks causes one disk write and one daemon reload.
  `writer::flush()` runs before **every** daemon request (`ipc/client.rs` `call`) and in
  `hard_exit`, so the daemon never acts on an older config. `Config::reload()` skips while our
  own write is pending (it would undo the user's newer change) and skips parsing when the file
  hash equals our last save (`last_saved`, FNV-1a). Unit tests write synchronously (`cfg!(test)`).
  `SKWD_WALL_SYNC_CONFIG=1` restores the old synchronous save for A/B comparison.
- **Timing spans** `src/infrastructure/observability/perf/` (+ tests): `perf::span("name")`
  costs two clock reads + one lock; steps over 4 ms are logged at debug level
  (`SKWD_WALL_LOG=debug`). Instrumented: `update.{frame,settings,mouse,other}`, `view`,
  `view.settings`, `tick`, `apply_layout`, `settings.init_inputs`, `config.save`,
  `config.reload`, `config.adopt`, `config.write_disk` (writer thread), `config.flush_wait`.
- **Performance overlay**: Settings → Performance → Power → "Performance overlay" (key
  `performance.overlay`; also `skwd-helm ui perf on|off`; `SKWD_WALL_HUD=1` still works and
  adds the old memory lines). Shows FPS, frame p50/p95, CPU %, redraws, cost of the last settings
  click / view / tick, layout/inputs/save averages. It is drawn by the existing HUD
  (`app/view/transient.rs` `hud_layer`); while it is on, the picker redraws ~once a second
  (`schedule_frame` in `runtime/ticks.rs`) so the CPU figure stays current. Frame stats come from
  `PerfState::on_frame` (`app/state/perf.rs`), called from `run_tick`.
- **Benchmark**: `skwd-helm ui bench` sends `bench` to the picker, polls the new `perf-query`
  socket query and prints the report (also saved to `~/.cache/skwd-wall-v2/bench/latest.txt`
  and `.json`, plus dated copies). Script in `src/app/update/bench.rs`: idle 3 s, 24 arrow-key
  steps, 12 settings toggles (`filterBar.show.layout`), 30 slider steps (`gridFocusWidth`),
  every layout for 1.1 s, idle 3 s; it runs on a transient config copy (nothing is saved) and
  measures one real config save separately (`Config::measure_save`, writes a probe file next
  to config.json). `skwd-helm ui perf` prints live spans as JSON.
- **Low-power mode**: Settings → Performance → Power → "Low-power mode" (`performance.lowPower`).
  `skwd-config` `saver_active()` = low power OR (battery saver AND on battery), used by
  `effective_picker_fps`, `effective_video_idle_seconds`, `effective_gpu_preference` ("low"),
  `effective_wallpaper_performance`; picker `battery_saver_active()` uses it (no hover video
  previews, FPS cap = battery FPS). Picker: no Coverflow reflection, no blurred backdrop in
  Coverflow/Showcase/Hand (`app/startup.rs`). Daemon: no sketch before transitions
  (`value/transition/sketch.rs`, key spelled out there because of the deck layer guard).
- New keys (skwd-deck `skwd-config`): `performance.lowPower`, `performance.overlay`
  (key_catalog, schema, keys_tests, power_tests). Locale keys
  `settings-performance-{low-power,overlay}-{label,desc}` in all 14 locales (translated for
  most; ar/fa/hi/bn/ur descriptions are English). Helper: `handoff/tools/add_locale_keys.py
  <file.ftl> <keys.json>` adds/replaces keys in every locale.
- **Not measurable here (no GPU):** real frame pacing. With llvmpipe every frame of the 1080p
  picker costs 100-250 ms of CPU, so frame gaps/CPU % in the cloud report are meaningless; the
  per-step CPU costs (update/view/tick spans) are valid. Ask the user for their bench report.
- Not done in M1 (candidates if the user's numbers show a need): caching the Settings cards
  between frames (`source.cards(tab)` is rebuilt on every view, 0.3 ms here), a cheaper
  launch animation in low power, a lower Sandy LOD in low power.

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
