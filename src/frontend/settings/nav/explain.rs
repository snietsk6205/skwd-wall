//! R1: longer explanations behind "more…" for the settings that confuse
//! people most. Keyed by config path; plain words, English only.

const MORE: &[(&str, &str)] = &[
    (
        "performance.lowPower",
        "Low-power mode makes everything lighter at once: the picker draws at the battery frame rate, hover video previews stop, Coverflow and Showcase drop their reflections and blurred backdrops, the sketch before a transition is skipped, and Settings stops its animations and its moving Home background. Turn it on when the laptop gets warm or the battery matters; nothing else changes.",
    ),
    (
        "performance.batterySaver",
        "Battery saver does the same as Low-power mode, but only while the laptop runs on battery. Plugged in, everything is back to normal by itself.",
    ),
    (
        "general.maxFps",
        "How many pictures per second the picker draws while something moves. Higher looks smoother but costs more power. On a 144 Hz screen 144 is the most that helps; if your graphics chip cannot keep up, a lower number feels steadier. The benchmark (Ctrl+K, \"Run the benchmark\") tells you which value fits.",
    ),
    (
        "performance.gpuPreference",
        "Which graphics chip draws the picker. \"low\" uses the built-in chip (saves power), \"high\" the stronger card (smoother, for example your RTX 3050). Takes effect the next time the picker starts.",
    ),
    (
        "transition.origin",
        "Some transitions grow from one point (ripple, iris, lens, clock wipe, ...). This picks that point: what the transition does by itself, the middle of the screen, a random place, or the busiest part of the new wallpaper (where the most detail is).",
    ),
    (
        "transition.sketchEnabled",
        "Before the new wallpaper appears, it is drawn as a sketch, stroke by stroke, on top of the old one. When the drawing is done, the transition you chose reveals the real picture from it. Videos are frozen on their first frame while the sketch draws.",
    ),
    (
        "transition.playlistOrder",
        "In order plays the list from top to bottom and starts again. Shuffled plays every entry once per round in a random order. Random, no repeats picks any entry but never one of the last few.",
    ),
    (
        "transition.comboMode",
        "How the two transitions work together. One after the other: the first stops half-way and the second finishes from that moment. Blend mixes both. Split screen plays each on its own part of the screen. First is a mask: the first gives the shape, the second the look. Sweep and the lighter/darker modes mix them in other ways.",
    ),
    (
        "display.fillMode",
        "What happens when the picture does not have the shape of your screen. Fill covers the screen and cuts the edges off. Fit shows the whole picture with bars. Stretch pulls it to the screen's shape. Centre and Tile keep the original size. Span spreads one picture over all screens.",
    ),
    (
        "paper.engine",
        "The program that puts still pictures on the desktop. skwd-paper is the built-in one and the only one that can play every transition; awww is an outside program with its own, simpler transitions.",
    ),
    (
        "paper.performanceMode",
        "How hard moving wallpapers (videos and scenes) may work. Lower settings save power and keep games smooth; higher ones look better.",
    ),
    (
        "playback.processEnabled",
        "Pause moving wallpapers while one of the programs in your list runs (for example a game). Use \"Choose a running program\" to pick one instead of typing its name.",
    ),
    (
        "playback.fullscreen",
        "Pause moving wallpapers while a window covers the whole screen, because nobody can see the wallpaper then. Saves power while you watch films or play.",
    ),
    (
        "playback.maximized",
        "Like the full-screen rule, but also for windows that are only maximised (the bar stays visible).",
    ),
    (
        "playback.muteOnOtherAudio",
        "Turns the wallpaper's sound down while another program plays sound, and back up when it stops.",
    ),
    (
        "theme.policy",
        "Whether the colours of your desktop come from the wallpaper (they change with it) or stay fixed on a theme you chose.",
    ),
    (
        "theme.engine",
        "The tool that turns a wallpaper into a set of colours. They give slightly different results; try one, look at the result, keep the one you like.",
    ),
    (
        "library.profile",
        "A profile is a named choice of folders. Switching the profile changes which wallpapers the picker shows, without deleting anything.",
    ),
    (
        "selector.livePreview",
        "Cards show a moving preview of video wallpapers while you hover them. Turn it off if the picker feels slow.",
    ),
    (
        "general.randomRotate",
        "Changes the wallpaper by itself every few minutes, picking from your library. The interval and which kinds to include are on this page.",
    ),
    (
        "schedule.enabled",
        "Plays wallpapers by the time of day (for example a bright one in the morning, a dark one at night). Open the schedule editor to plan the day.",
    ),
    (
        "weather.enabled",
        "Matches the wallpaper to the weather outside your window (sun, rain, snow, ...). It needs your location; Review wallpapers shows which pictures go with which weather.",
    ),
    (
        "semantic.enabled",
        "Lets you find wallpapers by describing them in words (\"foggy forest\", \"neon city\"). It needs a small model on your computer and some time to look at every picture once.",
    ),
    (
        "tagging.defaultSearchMode",
        "Tags searches the words you or the scanner attached to each wallpaper. Describe searches by meaning, using the semantic model.",
    ),
    (
        "settingsWindow.view",
        "Simple hides the settings most people never need (fine geometry, engine internals, timing details). Nothing is lost: Advanced shows them, every page has \"Show them\", and Ctrl+K always finds them.",
    ),
    (
        "settingsWindow.homeBackground",
        "Current wallpaper shows what is on your desktop now (a video plays). A picture or video uses the file in \"Background file\", or the wallpaper you selected with \"Use selected\". The video stops by itself in Low-power mode, with Battery saver, and whenever Settings is closed.",
    ),
    (
        "settingsWindow.openOn",
        "Home is the calm start page with your cards. Where I left off opens the page you used last, handy while you fine-tune one thing.",
    ),
    (
        "launch.animation",
        "How the cards arrive when the picker opens. Low-power mode keeps the opening short.",
    ),
];

pub fn more_text(path: &str) -> Option<&'static str> {
    MORE.iter().find(|(key, _)| *key == path).map(|(_, text)| *text)
}

#[cfg(test)]
pub(super) fn explained_paths() -> impl Iterator<Item = &'static str> {
    MORE.iter().map(|(path, _)| *path)
}
