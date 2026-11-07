# Designer pass (Phase 5): research, what was added, what to do next

## What other shells and tools do (researched 2026-11)
- **Noctalia shell**: wallpaper transition shaders including *pixelate* and *honeycomb*;
  picking the colour scheme straight from the wallpaper selector, with animated scheme changes;
  starring a wallpaper pins it to the top and remembers its colour scheme; a community
  "Wallcards" picker with live video preview and J/K keys; an open request for separate
  light-theme and dark-theme wallpapers.
- **Caelestia shell**: a hotkey for the picker, `caelestia wallpaper` CLI/IPC, a folder setting,
  dynamic colours that follow the wallpaper.
- **Waypaper** (GUI for swww/hyprpaper/swaybg): restore the last wallpaper at login, works with
  many back ends, GIF/multi-monitor through swww.
- skwd-wall already covers most of this (favourites, playlists, schedules, weather rules,
  theme back ends, Wallhaven/Steam, live previews, a CLI). What it lacked was *presentation*:
  layouts that show a few wallpapers big and beautifully, a tidy bar, and more transitions.

## Added in this session (ranked by how much difference they make)
1. **Coverflow layout** (Coverflow / Carousel ring / Stack) with reflection and a blurred
   backdrop of the selected wallpaper. The most "r/unixporn" thing in the picker now.
2. **Showcase layout**: big preview + filmstrip. The easiest way to actually *see* a wallpaper
   before applying it.
3. **Seven transitions**: honeycomb (Noctalia-style), burn, page curl, ripple, vortex, blinds,
   slide. Burn, honeycomb and page curl are the stand-outs.
4. **Bar control**: order, size, edge position, Pill and Glass styles, a layout button and
   one-click Minimal/Everything presets.
5. **Cascade and Bloom opening animations.**
6. Selection ring keeps showing through small mouse nudges.

## Honest notes
- The big previews (Showcase, Coverflow front card) use the picker's thumbnails, which are at
  most 640x360. On a 1440p/4K screen the big Showcase picture is visibly soft. Fixing that
  properly means loading the full image for the selected card (new texture upload path) —
  worth doing, but it is real work and touches upstream's atlas code.
- The blurred backdrop is cheap on any real GPU but very slow in software rendering.
- New labels are English in every language (the translation files had to get the keys, but I
  did not translate them).
- Not tested on a real Hyprland desktop or GPU here: everything was checked in a headless
  sway with software Vulkan. Please try the layouts and transitions on your machine.

## Suggested next steps (ranked)
1. **Full-resolution preview** for the selected card in Showcase (and a Space-bar "peek"
   that shows it full screen). Biggest visual win left. Medium-high effort.
2. **Spectrum (mood) layout**: wallpapers placed on a colour wheel by hue and saturation;
   the arrow keys already move by position, so navigation would just work. Medium effort.
3. **Transition shuffle list**: "random" picks only from transitions you tick. Small-medium
   (daemon + renderer + a Settings list).
4. **Day/night (light/dark) wallpaper pairs**: the daemon already has schedules; a pair rule
   would be small there. Medium.
5. **Hover tilt and glare** on Grid cards. Fun, small-medium, purely visual.
6. Translate the new labels.

## Worth reporting to liixini (upstream)
- The keyboard-navigation bug fixed in Phase 2 (Up/Down by index in masonry/hex layouts) is
  in the official build too; the fix in `app/update/spatial_nav.rs` is self-contained.
- The REFLECTION shader flag and the Coverflow/Showcase layouts could be offered upstream as
  a pull request if you like them; they follow upstream's structure (new modes, schema keys,
  tests, layer rules).
