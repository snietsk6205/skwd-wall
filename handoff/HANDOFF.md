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
  command while it is open. **Since M1b the benchmark is ONE command** (fish and bash, local or
  SSH): `~/.local/opt/skwd-custom/bin/skwd-helm ui bench` (optional `--gpu high|low`). Helm
  stops a picker without the benchmark (official/older: it answers `bench-run` with nothing),
  starts the sibling `skwd-wall-v2`, finds `WAYLAND_DISPLAY` itself, waits for the report on
  one `bench-run` query (no polling) and sends `hide` at the end. See Phase 7.
  User's machine (from their log): NVIDIA GPU, 988 wallpapers, Noctalia shell also running.

## User's machine and M1 benchmark results (2026-11-08)
- Lenovo LOQ 15IAX9: i5-12450HX, RTX 3050 6GB Laptop + Intel UHD (ADL-S), 23 GB RAM, btrfs,
  1920x1080 @ 1.5x scale (picker sees 1280x720), **144 Hz**, Hyprland 0.57.1 (Lua config),
  fish 4.9, kitty, Noctalia shell. The picker runs on the **Intel iGPU**. 988 wallpapers.
- `skwd-helm ui bench` (new build): idle CPU 0.4 %; navigate p50/p95 12.6/14.8 ms, smooth
  99.5 %, CPU 19 %; settings click: update.settings ~1.0 ms, view.settings 0.45 ms,
  init_inputs 0.7 ms, apply_layout 0.2 ms; slider p95 17.5 ms, smooth 97 %; **layouts p99
  ~200 ms (smooth 86 %)**; one synchronous config save on their btrfs disk: **10.9 ms** (the
  old per-click UI stall that M1 moved off the UI thread).
- What the numbers mean / known flaws of the benchmark (all fixed in M1b, see Phase 7):
  - The `SKWD_WALL_SYNC_CONFIG=1` comparison run is identical by design: the bench runs on a
    transient config, so it never saves. The old per-click cost is the "config save" line.
  - "settings-click smooth 46 %, p95 149 ms" is an artefact: the clicks are 150 ms apart and
    nothing animates between them, so the gap between frames is the click spacing. Measure
    click-to-next-frame latency instead.
  - The 3-4 idle redraws are the benchmark's own polling (`skwd-helm` sends `perf-query` every
    second and each query runs update + view). Without polling idle is 0 redraws (checked in
    the cloud: 0 CPU ticks in 5 s).
  - Navigation runs at ~80 fps on the 144 Hz screen (p50 12.6 ms): check the picker FPS cap /
    tick interval (`general.maxFps` default?) — maybe allow matching the refresh rate.
- M1 read the layouts result as "a ~200 ms hitch per switch" (p99 206 ms). **M1b found that
  this number was mostly the M1 benchmark's own artefact** (see Phase 7): M1 measured gaps
  between ticks and counted every gap under 250 ms as a frame, so the idle pause between a
  layout settling and the next switch (e.g. Hand settles after ~900 ms, next switch at 1100 ms)
  and slower intended frame rates (30/20 fps preview/passive ticks) counted as slow frames.
  Real per-switch costs that M1b did find and fix are in Phase 7. The user's next report
  (new method) will show the real frames on their iGPU.

## Session rule (from the user, applies to every session)
- **One milestone per session.** When it is done: full build, tests, screenshots, commit, push
  to `my-custom`, update this file, then stop and give the user the exact prompt for the next
  session in a copyable box. Stop early (work pushed + noted here) if the session gets long.

## Roadmap agreed with the user (priority order; one per session)
- **M1 Speed (done, Phase 6 below).**
- **M1b Speed follow-up (done, Phase 7 below).**
- **M2 Window size everywhere.** The Settings window size/position/scale must apply to
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

## Phase 7 = M1b (done): render-side measurement, layout-switch fixes, honest benchmark
- **Render-side frame log** (the "proof" measurement): `contracts/rendering/stats.rs`
  `RenderStats` (shared through `ScenePrimitive.stats`, no global; the browser's embedded scene
  passes `None`). `ScenePrimitive::prepare` records one `RenderFrame` per drawn frame: start
  time, gap since the previous frame, and `RenderWork` (prepare total, thumbnail uploads
  count/bytes/time, pipeline creation, render-target creation, atlas growth, and `render`'s
  encode time). App side: `PerfState.render` (`app/state/perf.rs`); the overlay's FPS now comes
  from it too. `perf::thread_cpu_time()` (CLOCK_THREAD_CPUTIME_ID) samples the UI thread at
  every tick, so the report can say "UI thread busy X of Y ms" (busy ≈ wall: CPU work; much
  lower: waiting for the GPU/driver/screen).
- **What it showed (cloud, llvmpipe, headless sway at 512x288@144Hz, 40 wallpapers):**
  - The M1 "~200 ms hitch" is mostly a measurement artefact: in the same run the old method
    (`tick_gaps_m1`, kept in the report as "the first benchmark's method would say") reports
    p99 ~150 ms for settings clicks and ~225-246 ms for layouts while the real frames show no
    such stall on clicks. The pauses between animations were counted as frames.
  - Real per-switch costs: (1) the cross-fade pipeline (`shaders/transition.wgsl`) was
    compiled during the FIRST switch (5.4 ms on llvmpipe); (2) the two screen-sized cross-fade
    targets were created and freed on EVERY switch; (3) every cross-fade frame drew BOTH layouts
    (old into A, new into B) although the old one is a frozen snapshot; (4) atlas growth
    (`recreate_near`, 7 ms on llvmpipe) when Coverflow first needs more near layers (left as is:
    a one-off, and growing lazily saves memory).
  - The slow frames after a switch had an idle UI thread → GPU-bound (on llvmpipe Hand and
    Sandy frames are 200-250 ms, other layouts 20-45 ms at 512x288).
- **Fixes** (`rendering/scene/pipeline/`): the cross-fade pipeline is built in
  `ScenePipeline::new` (`create.rs`); the old layout is drawn into target A once per switch
  (`old_drawn: AtomicBool`, cleared by `invalidate_old_layout()` when A is re-created or the old
  snapshot changes; `render` swaps it); targets are kept for `TARGETS_KEEP` = 10 s after the
  last switch and freed when the picker hides (`release_targets_due`, `targets_used`).
  A/B on the same headless setup: median frame gap during the layouts phase **68.4 → 39.2 ms**,
  frames drawn 53 → 73, first-switch pipeline time 5.4 → 0 ms. The first cross-fade frame still
  draws both layouts once (inherent).
- **Why ~80 fps on 144 Hz** (simulation `pace.py` in that session + code reading): the
  layer-shell loop presents only after the compositor's frame callback (one frame in flight;
  `iced_layershell` `take_present_slot`/`request_next_present`), so each frame's CPU draw + GPU
  work must fit in ONE refresh (6.9 ms at 144 Hz) or it waits a whole extra refresh (13.9 ms =
  72 fps). The user's p50 12.6 / p95 14.8 ms is that mix. Also `general.maxFps` defaults to
  120 (< 144), which alone caps it and causes uneven pacing. Not changed by default; the bench
  now runs `navigate` and `navigate-uncapped` (cap 1000 on the transient config) and prints
  advice: raise Max FPS if uncapped is clearly faster, else try `--gpu high` (RTX 3050 via
  `WGPU_POWER_PREF=high`, the existing Settings → Performance → GPU preference "high").
- **Benchmark rewrite** (`app/update/bench/`: `script.rs` steps + report, `report.rs` pure
  maths + `report/tests.rs`): frame gaps from the render log; gaps across pauses (`after_tick`
  records when the picker is not animating at full rate, `BenchPhase.pauses`) are not frames;
  `click/key/slider/switch -> frame` latency = action → end of the renderer's work on the first
  frame after it; `settings-click-old` does each click plus `Config::measure_save()` on the UI
  thread (the pre-M1 behaviour, on the user's disk) next to `settings-click`; layouts start
  after the user's own layout and end on it (every step a real switch) with one block per
  layout (first frame, slowest frame + when, median, idle-after time, renderer work, UI busy);
  idle redraws exclude the bench's own step; "In short:" plain-language summary; header shows
  fps cap, GPU preference, present mode. `perf-query` no longer ticks; `skwd-helm ui perf` also
  returns the last 20 render frames.
- **One command**: picker query `bench-run` (`dispatcher.rs` → `bench::start_and_reply`, reply
  stored in `PerfState.bench_waiters`, answered with `{report, path}` JSON at the end;
  `shell.rs` `attach_reply` accepts it). Helm `crates/skwd-helm/src/bench.rs`; wall-proto
  `picker_control::send_query_waiting(cmd, wait)`. `SKWD_WALL_SYNC_CONFIG=1` still exists for
  diagnosis but the README no longer suggests a second run.
- Tested in the cloud: the one command with no picker running and no `WAYLAND_DISPLAY`
  (starts, benchmarks, closes: 33 s), and with a picker open plus `--gpu low` (stops it,
  restarts on the other GPU preference, header shows it). Sample report:
  `custom/previews/m1b-bench-cloud.txt` (llvmpipe: absolute numbers meaningless).
- Next time the user sends a report, read per layout: "first frame" (should be < 50 ms) and
  "slowest frame after it" with "UI thread busy" to decide CPU vs GPU; `navigate` vs
  `navigate-uncapped` for the FPS question.

## Testing in a cloud session
- Scripts from the last session (recreate in the scratchpad): a headless sway + `skwd-walld` +
  `skwd-wall` (+ `skwd-wall-scan` on PATH, or there are no thumbnails) with `HOME`,
  `XDG_RUNTIME_DIR`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` pointing at a fake home whose config sets
  `paths.wallpaper` to a folder of generated test images (ImageMagick gradients); sway config
  `output HEADLESS-1 mode 512x288@144Hz` keeps llvmpipe fast enough for pacing tests. Then `skwd-helm ui set <key> <value>`,
  `skwd-helm ui nav right`, `skwd-helm ui tab picker`, `skwd-helm ui section N`, and `grim`.
  Wait for the picker to go idle (CPU ~0) before `grim`.
- Software Vulkan (llvmpipe) works; `skwd-wall-vk --preview-stream` renders frames.
- A headless sway (`WLR_BACKENDS=headless WLR_RENDERER=pixman`) runs the picker; `grim`
  screenshots; `skwd-helm ui <command>` drives it and `skwd-helm ui state` reports state.
- Known failing tests that also fail on pristine upstream in a container (environmental):
  `paper-scene hlsl::tests::packed_audio_spectrum...`, `skwd-paper manager::tests::span_spawn_failure`,
  `skwd-e2e --test semantic`.
- Disk is limited (~40 GB): use one shared `CARGO_TARGET_DIR` and `CARGO_INCREMENTAL=0`.
