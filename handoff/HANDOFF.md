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

## User's M1b benchmark results (2026-11-08, second report, read in the M2 session)
- Two runs of the M1b one-command bench, **on battery** (fastfetch said "Discharging"), so the
  battery saver was active: header "fps cap 60" (= `performance.batteryFps` default, not the
  user's Max FPS) and "GPU preference low" in the default run.
- Intel UHD (default, GPU preference low): idle 0 redraws, CPU 0.2 % (goal met). Navigate 43 fps,
  frame gap p50/p95 23.1/25.0 ms, smooth 0 % (smooth = gaps <= 20 ms), key -> frame p50 13.8 ms;
  uncapped identical (43 fps) → the iGPU itself needs ~23 ms per picker frame at 1280x720 logical
  (1920x1080 physical). Layouts: every first frame <= 18.2 ms (showcase 18.2: waited 17.8 for the
  frame to start; sandy 14.5: atlas growth 10.3 ms), goal < 50 ms met. Slowest frames after a
  switch: sandy 34.1 ms (UI busy 21 of 40 → CPU+GPU), hand/coverflow medians ~23 ms with an idle UI
  thread (GPU-bound). Settings click new 2.8 ms vs old 16.3 ms (one save on their disk 13.2 ms):
  M1 fix confirmed on real hardware.
- RTX 3050 (`--gpu high`): navigate 70 fps, p50 14.2 ms, smooth 99.4 %, key -> frame 8.3 ms;
  uncapped 71 fps. Layouts 61 fps, smooth 98.2 %, first frames <= 10.3 ms, slowest 26.4 ms (depth,
  UI busy 4 of 33 → waiting for GPU/compositor). Settings click 2.5 vs 16.1 ms.
- Plain reading given to the user: the picker is fine on idle and on clicks everywhere; for
  smooth motion use the RTX 3050 (Settings → Performance → GPU preference "high"), especially when
  plugged in; neither chip reaches 144 fps (needs 6.9 ms per frame), and the FPS cap is not what
  limits it (capped = uncapped), so raising Max FPS does not help yet.
- **Benchmark flaws found in this report (fix in a later speed pass, not done in M2):**
  - The "Screen: ... about N Hz" line is the 5th-percentile frame gap, i.e. the GPU's own best
    frame time when every frame is slow: it said 45 Hz (Intel) and 108 Hz (RTX) on a 144 Hz
    screen. Read the real refresh rate (wl_output mode / layer-shell output) instead and print it.
  - The advice "Compare with the other graphics card (skwd-helm ui bench --gpu high)" is printed
    even when the run already used `--gpu high` (script.rs `refresh_summary`).
  - The header should say when the battery saver changed the FPS cap / GPU preference.
  - With "fps cap 60" the RTX run still showed 70 fps: the cap is a tick limiter, not a hard
    present limit; check `effective_picker_fps` use in `runtime/ticks.rs` if it matters.

## Session rule (from the user, applies to every session)
- **One milestone per session.** When it is done: full build, tests, screenshots, commit, push
  to `my-custom`, update this file, then stop and give the user the exact prompt for the next
  session in a copyable box. Stop early (work pushed + noted here) if the session gets long.

## Roadmap agreed with the user (priority order; one per session)
- **M1 Speed (done, Phase 6 below).**
- **M1b Speed follow-up (done, Phase 7 below).**
- **M2 Window size everywhere (done, Phase 8 below).**
- **M3 Transitions (done, Phase 9 below).**
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
- Checks at the end of M1b: clippy clean (pedantic, all targets) on skwd-wall, skwd-helm,
  wall-proto; tests: skwd-wall 1629 unit + all guard tests pass; skwd-deck 1816 pass, only the
  known `skwd-e2e --test semantic` fails; skwd-paper (unchanged) 901 pass, only the known
  `paper-scene hlsl::tests::packed_audio_spectrum...` fails. Release build of all 11 programs
  (installer's package list, `--locked`) OK in 23 min; the release one-command bench ran OK.
- Next time the user sends a report, read per layout: "first frame" (should be < 50 ms) and
  "slowest frame after it" with "UI thread busy" to decide CPU vs GPU; `navigate` vs
  `navigate-uncapped` for the FPS question.

## Phase 8 = M2 (done): window size everywhere
- What the user gets: Settings → Picker → Settings window (width %, height %, scale, position)
  now applies to **every panel over the picker**: Settings, layout studio (Shape), playlists (+ its
  playlist-picker dialog and filter help), card picker ("Add to playlist"), theme designer,
  effects studio (+ Choose displays), schedule editor, tag cloud, scene properties, downloads
  browser, audio mixer. The picker itself still fills the screen. Defaults (0 / Centre) give
  exactly the old automatic sizes.
- `frontend/ui/misc/sheet.rs` `SheetWindow` (+ `SheetPosition`), pure geometry, **no globals**
  (passed explicitly; the handoff's thread-local idea was dropped to match M1b's no-global style):
  scale, width/height shares (0 = automatic), position; `dims(viewport)` (folio size 1390x870 x
  scale or the shares), `fit(viewport, auto)`, `width_or/height_or` (share x screen, max screen -
  22, min 220x160, as the old Settings logic), `align_x/align_y`, `edge_gap` (11 px x scale when
  docked), `origin()`, `place(panel)`, `align(container)`, `with_scale`.
- Reader `frontend/settings/layout.rs` `sheet_window(&dyn SettingsSource, ui_scale)` (scale < 0.5
  = UI scale, max 2). App side `App::sheet_window()` (`app/helpers/application.rs`), built once per
  view in `app/view/panels.rs`, `app/view/settings.rs`, `app/view/tags.rs`.
- `folio_sheet()` takes `window: SheetWindow` instead of `scale`; every panel view takes `window`
  instead of `scale` (`let scale = window.scale();`). Schedule keeps its roomy x1.18 on screens >=
  1280x760 via `window.with_scale(schedule_scale(..))`. Effects `monitor::tile_is_wide(panel_width,
  scale)`. Browser: `wall_aperture_dims(viewport, grid, window)` (also used in `runtime/ticks.rs`
  for the embedded wall's viewport) and `browser_wall_click` hit-tests card buttons with
  `sheet_window().scale()` (the overlay draws them with it).
- Settings: `ChromeCtx { viewport, window, rtl, entrance, palette }` (was `scale`); the old
  `SettingsWindow` struct keeps only `one_column`/`compact_index`.
- Layout studio = side panel next to the live picker (`picker_layout_studio_width/height/align/
  inset/on_right`): width = window width or automatic 440 x scale, **always capped at 48 % of the
  screen**; height = window height or full height - 36; Left/Right choose the side, other
  positions keep the reading-start side; Top/Bottom align it when a height is set. The picker moves
  out of the way on that side: new scene field `inset_on_right` (`SceneCore::set_inset_side`, used
  by `center_layout` and `view_hex`; before, the side was `reading_rtl`), set in `post_update.rs`
  only while the inset is > 0 so the picker does not jump while sliding back. (Grid/"wall" mode does
  not move its cards for the inset at all, upstream behaviour, so the studio covers some cards
  there either way.)
- Tag cloud (`app/view/tags.rs`): window scale; width = its own `tagCloudWidth` if set, else the
  window width if set, else automatic; height capped by the window height; Centre = usual place
  under the cards, other positions dock it to that edge (`cloud_dock`, its offsets still apply,
  clamped to the screen).
- Card picker and playlist-picker dialog: window size replaces the automatic size when set (card
  picker height is a max: short lists stay short); playlists filter help capped to the sheet.
- Settings text (`window_tab.rs`, English-only like the other custom cards) says it covers every
  panel, that the studio stops at about half the screen and that Centre keeps the tag cloud under
  the cards.
- New picker commands (tests, also handy by hand): `skwd-helm ui open card-picker` (selected
  card), `skwd-helm ui open scene-properties` (selected card must be a WE scene).
- Not in the list, left as is: theme audition (full-screen view), keyboard help, card back.
- Tests: `frontend/ui/misc/tests.rs` (`sheet_window_*`), `frontend/settings/layout/tests.rs`
  (reader + studio), `app/view/tags/tests.rs` (`cloud_dock`), effects `monitor/tests.rs`.
- Screenshots `custom/previews/m2-*.jpg`: each panel small (50 x 60 %, scale 0.85, Right) next to
  large (96 x 94 %, 1.2, Centre) at 1280x720. Script recipe: set the four keys with `skwd-helm ui
  set`, `ui open <panel>` (settings: `ui tab picker` + `ui section 8` = Settings window card,
  `section 1` = layout studio), wait 3 s, `grim`, `ui dismiss` x3. A fake WE scene for scene
  properties: `<paths.steamWorkshop>/2900000001/project.json` with `"type":"scene"`, a
  `preview` png and `general.properties` (bool/slider/color/combo) is enough.
- Checks at the end of M2: `cargo fmt`; clippy clean (workspace lints, all targets) on skwd-wall;
  tests: skwd-wall 1639 unit + all guard tests pass; skwd-deck (unchanged) 1816 pass, only the
  known `skwd-e2e --test semantic` fails; skwd-paper (unchanged) 901 pass, only the known `paper-scene hlsl::tests::packed_audio_spectrum...` fails. Release build of
  all 11 programs (installer's package list, `--locked`) OK in 23 min; the release picker docked a
  45 x 70 % playlists sheet on the left as expected. skwd-deck and skwd-paper have no M2 changes.

## Phase 9 = M3 (done): transitions
- What the user gets (Settings → Motion → Wallpaper transitions, one section so the preview stays in
  view): 31 new original transitions, families Light and Shapes (+ pseudo families Playlist and
  Combine two), a pinned live preview with caption, a library (search, filters, per-row ▶ Preview /
  Favourite / + Playlist / Use), "Play in turns" (in order / shuffled / random no repeats) and
  "Combine two" (7 ways, split has 5 shapes). User text: custom/README.md "Transitions (M3)".
- **Renderer (skwd-paper)**:
  - New GLSL 330 effects in `paper-shaders/src/effects/resources/{light,shapes,extra}.rs`, listed in
    `effects/catalog.rs` (EFFECTS is now 68). Start-point edits in `ORIGIN_EFFECTS` (17 now):
    god-rays, diamonds, shutter, lens, glass, clock-wipe. Every extra light fades out before the
    end. All compile with glslc (build.rs compiles every entry).
  - `paper-shaders/src/plan.rs` (+ `plan_tests.rs`): **one parser for every `--shader` value**:
    `Plan::Single(Part::{Fade, Sand, Effect})` or `Plan::Combo { mode, first, second }`. Names:
    `combo:<mode>:<first>:<second>`, modes `then, blend, split-{lr,tb,diagonal,circle,checker},
    mask, sweep, light, dark`; parts may be "random". `random-mix` = random mode + two random parts.
    paper-vk calls it through `sketch::transition_plan()` (uses `random_u32`); the old three copies
    of the "random" roll (dmabuf_helpers, scene.rs, preview.rs) are gone; scene.rs
    `TransitionStyle` is now an alias of `Plan`; `FadeState` and the video streamer hold `plan`.
  - `Renderer::render_plan_to` (`vk/draw/shared/combo.rs`) is the single entry for every path
    (videos, images, scenes, both previews); `prepare_plan` builds pipelines ahead. Combos keep
    their state in `vk/combo.rs` `ComboState`: targets `first`, `second` (renderer size, made with
    the sketch's `create_sketch_target` and the scene render pass), 8x8 `black`/`white` mask
    sources, a 3-set descriptor pool, the compose pipeline (`effects/combo.glsl`, built on
    `VK_EFFECT_HEADER`, u_style = mode, u_pid_base = split shape, texB = old wallpaper), `held`,
    `last` (freed after 3 s idle from `render_to`, like the sketch). Non-"then" modes record both
    parts into their targets and the compose pass in ONE command buffer (parts reuse
    `desc_set`/`desc_set_b`; mask mode draws the first part from black to white = its mask).
    "then": first half draws part A at progress t (0 → 0.5) directly; at the hand-over A's
    half-way frame is frozen once into `first` (the sketch freeze idea) and part B runs from it.
    The sketch prelude works in front of combos too (`sketch_step`). Fade parts use the
    `crossfade` effect. Effect pipelines (built for the main render pass) are used in the scene
    pass: same format, compatible.
- **Daemon (skwd-deck)**:
  - `wall-proto/src/transitions.rs`: families Random, Playlist, Combo, Fade, Wipe, Warp, Break,
    Light, Shapes, Sand; `TransitionSpec.desc` (one sentence each); `PSEUDO_KEYS` (random,
    random-mix, playlist, combo); `COMBO_MODES`, `COMBO_SPLITS`, `PLAYLIST_ORDERS`; helpers
    `spec`, `is_shader`, `combo_name(mode, split, first, second)`, `label_of`, `pretty` ("crt" →
    "CRT"). Some upstream entries moved family: chromatic-bloom → Light; honeycomb, mosaic-tumble,
    polka-dots-curtain, puzzle-right, randomsquares → Shapes.
  - Keys (skwd-config key_catalog/schema/keys_tests): `transition.favourites`, `.playlist` (comma
    lists like the bar order), `.playlistOrder` (in-order/shuffle/no-repeat), `.comboFirst`,
    `.comboSecond`, `.comboMode`, `.comboSplit`; Settings-only `.librarySearch`, `.libraryFilter`,
    `.comboFirstFamily`, `.comboSecondFamily`.
  - `value/transition/deck.rs` (keys spelled out, test `transition_deck_keys_match_the_schema`;
    value/transition.rs is at its 90-line guard limit): `favourites()`, `playlist()`,
    `playlist_order()`, `combo()`, `concrete_shader(next)`: "playlist" → next entry ("random" if
    empty), "combo" → combination name. Pure deck logic `domain/wallpaper/transition_deck.rs`
    (+ tests); the cursor lives in `ApplyRuntime::next_transition` (in memory: a daemon restart
    starts the order again). **Called where the shader is first read**: walld
    `composition/apply.rs` (static, video, WE: three places) and core `TransitionSelection::
    Configured` + `we/adapter.rs`; downstream paths get it as Explicit, so one pick per apply.
    Checked live: in-order playlist "burn,aurora,combo,lens" gave aurora, combo:mask:honeycomb:burn,
    lens, burn, aurora, … on six applies (the first pick went to the start-up restore).
  - Policy/sand-scope code still reads the configured name ("playlist"), so a sand style picked
    from a playlist uses the "all outputs" scope. Fine for now.
- **Picker (skwd-wall)**:
  - `frontend/settings/tabs/transition_library.rs` (+ `transition_library/tests.rs`): cards
    LIBRARY_CARD, PLAYLIST_CARD, COMBO_CARD (English-only like other custom cards); composed into
    the Wallpaper transitions section with `segmented_rows` headings (`tabs/builder.rs`). Library
    lists all entries except random/playlist/combo (random-mix included). Combo parts = family
    chips + transition chips (writing `comboFirstFamily` picks that family's first transition:
    `combo_part_for_family` in `app/update/settings.rs`).
  - `transition_tab.rs`: new families get their English catalogue label (upstream ones keep their
    tr() keys); summary rows for Playlist / Combo; `pretty` delegates to wall-proto.
  - Actions (`frontend/settings/action.rs`): PreviewTransition, UseTransition, FavouriteTransition,
    PlaylistToggle/Preview/Move/Remove/Clear/AddFavourites, ComboSwap → `app/update/
    settings_transitions.rs`. Preview focus = `TransitionPreviewState.focus` (app state, not
    config); `lifecycle::preview_shader` resolves it ("playlist" → first entry, "combo" → name).
  - Reading pane (`folio.rs`): when a section has the preview, header + `pinned_preview` (16:9 stage
    + caption from `app/view/settings.rs::transition_caption`) stay on top and only the rows scroll.
    The scrollable has id `READING_PANE`; new picker command **`skwd-helm ui settings-scroll <px>`**
    (for screenshots; headless sway has no pointer to wheel with).
- Previews: `custom/previews/m3-*.jpg` (5 sheets + 6 Settings screenshots, see its README). Sheet
  recipe: `SKWD_PAPER_PREVIEW_TEST=steps=10 skwd-wall-vk '*' NEW --transition-from OLD --shader
  NAME --preview-stream --preview-size 320x180 --preview-once --stream-no-header --duration-ms
  1000 > raw.rgba`, cut frames 1..9 with dd, `convert -size 320x180 -depth 8 rgba:`, montage rows.
- Ideas not done (candidates for later): combos of three; per-entry durations in the playlist;
  keeping the playlist cursor across daemon restarts; translating the new English texts; a
  "random from favourites" order; first combo frame builds the compose pipeline lazily (one small
  hitch per renderer).
- Checks at the end of M3: `cargo fmt` on all three; clippy clean (workspace lints, all targets) on
  paper-shaders + skwd-wall-vk, wall-proto + skwd-config + skwd-wall-core + skwd-walld +
  skwd-helm, and skwd-wall. Tests: skwd-wall 1644 unit + all guard tests pass (one upstream
  source-scan test, `preview_uses_full_stage`, now checks the pinned preview); skwd-deck 1836
  pass, only the known `skwd-e2e --test semantic` fails; skwd-paper 910 pass, only the known
  `paper-scene hlsl::tests::packed_audio_spectrum...` fails. Release build of all 11 programs
  (installer's package list, `--locked`) OK (55 min here, under load from the tests); the release
  renderer drew `combo:split-circle:aurora:shutter`. Live: Settings in headless sway (screenshots),
  playlist order through the daemon (see above).

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
- Settings screenshots (M3): `skwd-helm ui open settings`, `ui tab motion`, `ui section 3`, set keys
  with `ui set`, scroll with `ui settings-scroll <px>`, wait ~6 s, `grim`. The daemon also needs
  `skwd-wall-still` (build `-p skwd-wall-still`) to apply images.
