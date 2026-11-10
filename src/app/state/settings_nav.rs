//! R1 (custom build): state of the redesigned Settings window: which page is
//! open, the top bar's menus, the command palette, undo, the cards' pictures, the big preview
//! and the Home background.

use std::time::Instant;

use crate::frontend::animation::{MotionProfile, MotionTier, Tween};
use crate::frontend::settings::nav::PaletteEntry;
use crate::infrastructure::preview::LiveStreamer;

/// One undoable change.
#[derive(Debug, Clone)]
pub(crate) struct UndoEntry {
    pub(crate) path: String,
    /// The value before the change (`None`: the key was not set).
    pub(crate) before: Option<serde_json::Value>,
    pub(crate) after: serde_json::Value,
    pub(crate) at: Instant,
}

pub(crate) struct UndoToast {
    pub(crate) text: String,
    pub(crate) since: Instant,
    /// Shows "Undo" (false after an undo: "Undone: ...").
    pub(crate) can_undo: bool,
}

pub(crate) struct PaletteState {
    pub(crate) query: String,
    pub(crate) selected: usize,
    pub(crate) entries: Vec<PaletteEntry>,
}

/// R1b: "Preview style: follows the cursor": the live preview above the
/// card the pointer is on.
pub(crate) struct CursorPreview {
    pub(crate) row: usize,
    pub(crate) option: usize,
    pub(crate) shader: String,
    /// Its name in words.
    pub(crate) title: String,
    /// Where the panel is (from the card's left edge) and where the
    /// pointer is; the first follows the second softly.
    pub(crate) x: f32,
    pub(crate) target_x: f32,
}

/// R1b: the card that plays its strip right now.
pub(crate) struct PlayingStrip {
    pub(crate) name: String,
    /// Empty until the strip is read from disk.
    pub(crate) frames: Vec<iced::advanced::image::Handle>,
    pub(crate) frame: usize,
    pub(crate) next_at: Instant,
}

/// R1b: the pictures inside the cards (frame strips rendered once in the
/// background and cached on disk).
#[derive(Default)]
pub(crate) struct CardStrips {
    /// Started the first time a page with cards opens.
    pub(crate) worker: Option<crate::infrastructure::preview::StripWorker>,
    /// Middle frame of each strip read so far.
    pub(crate) middles: std::collections::HashMap<String, iced::advanced::image::Handle>,
    /// Strips already asked for (read from disk or queued for rendering).
    pub(crate) asked: std::collections::HashSet<String>,
    /// Strips still to render.
    pub(crate) pending: std::collections::HashSet<String>,
    pub(crate) playing: Option<PlayingStrip>,
    /// What the open page needs (skips the work while unchanged).
    pub(crate) signature: String,
    /// (row, choice) → strip name of the page's cards.
    pub(crate) tiles: Vec<((usize, usize), String)>,
}

#[derive(Default)]
pub(crate) struct HomeBackdrop {
    /// What the backdrop shows (a picture or video path); empty = none.
    pub(crate) source: String,
    pub(crate) video: bool,
    pub(crate) still: Option<iced::advanced::image::Handle>,
    pub(crate) blurred: Option<iced::advanced::image::Handle>,
    pub(crate) frame: Option<iced::advanced::image::Handle>,
    pub(crate) streamer: Option<LiveStreamer>,
    /// Source the picture decode was started for.
    pub(crate) decoding: String,
    /// The inputs `source` was worked out from (skips the work when unchanged).
    pub(crate) signature: String,
}

pub(crate) struct SettingsNav {
    /// Index into `nav::PAGES`; `None` is Home.
    pub(crate) page: Option<usize>,
    pub(crate) history: Vec<Option<usize>>,
    /// Open group menu (1..=5).
    pub(crate) menu: Option<usize>,
    pub(crate) menu_item: usize,
    /// Opened by a click or the keyboard: stays open when the pointer leaves.
    pub(crate) menu_sticky: bool,
    pub(crate) menu_close_at: Option<Instant>,
    pub(crate) menu_anim: Tween,
    pub(crate) menu_unavailable: Vec<usize>,
    /// Keyboard highlight in the top bar (0 = Home, 1..=5 groups, 6 = view
    /// switch, 7 = palette).
    pub(crate) bar_item: usize,
    pub(crate) page_anim: Tween,
    /// Show the advanced rows of this page even in Simple view.
    pub(crate) show_all: bool,
    pub(crate) edit_cards: bool,
    pub(crate) card_focus: usize,
    pub(crate) card_hover: Option<usize>,
    pub(crate) card_anim: Tween,
    pub(crate) palette: Option<PaletteState>,
    pub(crate) palette_recent: Vec<String>,
    pub(crate) undo: Vec<UndoEntry>,
    pub(crate) undoing: bool,
    pub(crate) toast: Option<UndoToast>,
    pub(crate) toast_anim: Tween,
    pub(crate) home: HomeBackdrop,
    /// R1b: the card under the pointer (row of the page, choice; 0 for a
    /// card that is a row of its own).
    pub(crate) tile_hover: Option<(usize, usize)>,
    /// R1b: the small live preview that follows the cursor.
    pub(crate) cursor_preview: Option<CursorPreview>,
    /// Where the pointer is on the card it is over (from its left edge).
    pub(crate) cursor_x: f32,
    pub(crate) tile_anim: Tween,
    pub(crate) strips: CardStrips,
    /// R1b: the transition "Preview big" shows in the overlay.
    pub(crate) big_preview: Option<String>,
    pub(crate) big_anim: Tween,
    /// R1b: the window's colours (neutral inks + the theme's accent).
    pub(crate) ink: crate::frontend::theme::Palette,
    /// Expanded "more…" explanations (by row title + path).
    pub(crate) more: std::collections::HashSet<String>,
    /// "Recently changed" lines for Home, rebuilt when a setting changes.
    pub(crate) recent: Vec<crate::frontend::settings::RecentItem>,
    /// Name, choices and unit of every setting by path, built on the first
    /// change (the toast's words). Cleared when the language changes.
    pub(crate) labels: Option<std::collections::HashMap<String, SettingLabel>>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SettingLabel {
    pub(crate) title: String,
    pub(crate) options: Vec<(String, String)>,
    pub(crate) unit: &'static str,
    /// Page (index into `nav::PAGES`) and row (full list) that edit it.
    pub(crate) page: usize,
    pub(crate) row: usize,
}

impl Default for SettingsNav {
    fn default() -> Self {
        let motion = MotionProfile::default();
        Self {
            page: None,
            history: Vec::new(),
            menu: None,
            menu_item: 0,
            menu_sticky: false,
            menu_close_at: None,
            menu_anim: motion.tween(0.0, MotionTier::Fast),
            menu_unavailable: Vec::new(),
            bar_item: 0,
            page_anim: motion.tween(1.0, MotionTier::Standard),
            show_all: false,
            edit_cards: false,
            card_focus: 0,
            card_hover: None,
            card_anim: motion.tween(0.0, MotionTier::Fast),
            palette: None,
            palette_recent: Vec::new(),
            undo: Vec::new(),
            undoing: false,
            toast: None,
            toast_anim: motion.tween(0.0, MotionTier::Fast),
            home: HomeBackdrop::default(),
            tile_hover: None,
            cursor_preview: None,
            cursor_x: 120.0,
            tile_anim: motion.tween(0.0, MotionTier::Fast),
            strips: CardStrips::default(),
            big_preview: None,
            big_anim: motion.tween(0.0, MotionTier::Fast),
            ink: crate::frontend::ui::ink_palette(&crate::frontend::theme::Palette::default()),
            more: std::collections::HashSet::new(),
            recent: Vec::new(),
            labels: None,
        }
    }
}

/// How long the undo toast stays.
pub(crate) const TOAST_SECONDS: f32 = 5.0;
/// Undo keeps this many steps.
pub(crate) const UNDO_STEPS: usize = 30;
/// Hover menus close this long after the pointer leaves.
pub(crate) const MENU_CLOSE_MS: u64 = 260;

impl SettingsNav {
    pub(crate) fn retime(&mut self, motion: MotionProfile) {
        motion.retime_tween(&mut self.menu_anim, MotionTier::Fast);
        motion.retime_tween(&mut self.page_anim, MotionTier::Standard);
        motion.retime_tween(&mut self.card_anim, MotionTier::Fast);
        motion.retime_tween(&mut self.toast_anim, MotionTier::Fast);
        motion.retime_tween(&mut self.tile_anim, MotionTier::Fast);
        motion.retime_tween(&mut self.big_anim, MotionTier::Fast);
    }

    pub(crate) fn animating(&self) -> bool {
        !self.menu_anim.settled()
            || !self.page_anim.settled()
            || !self.card_anim.settled()
            || !self.toast_anim.settled()
            || !self.tile_anim.settled()
            || !self.big_anim.settled()
            || self
                .cursor_preview
                .as_ref()
                .is_some_and(|cursor| (cursor.x - cursor.target_x).abs() > 0.5)
    }

    /// When the window needs a tick without animating: a hover menu's close
    /// delay or the toast's time running out.
    pub(crate) fn deadline(&self) -> Option<Instant> {
        let toast = self
            .toast
            .as_ref()
            .map(|toast| toast.since + std::time::Duration::from_secs_f32(TOAST_SECONDS));
        let strip = self
            .strips
            .playing
            .as_ref()
            .filter(|playing| !playing.frames.is_empty())
            .map(|playing| playing.next_at);
        [self.menu_close_at, toast, strip].into_iter().flatten().min()
    }

    pub(crate) fn tick(&mut self, dt: f32) {
        self.menu_anim.tick(dt);
        self.page_anim.tick(dt);
        self.card_anim.tick(dt);
        self.toast_anim.tick(dt);
        self.tile_anim.tick(dt);
        self.big_anim.tick(dt);
        if let Some(cursor) = self.cursor_preview.as_mut() {
            // A soft spring: most of the gap closes in a tenth of a second.
            cursor.x = crate::frontend::animation::approach(cursor.x, cursor.target_x, dt, 0.07);
        }
    }

    /// Ends every animation at its target (Low-power mode).
    pub(crate) fn settle(&mut self) {
        for tween in [
            &mut self.menu_anim,
            &mut self.page_anim,
            &mut self.card_anim,
            &mut self.toast_anim,
            &mut self.tile_anim,
            &mut self.big_anim,
        ] {
            let target = tween.target;
            tween.snap(target);
        }
        if let Some(cursor) = self.cursor_preview.as_mut() {
            cursor.x = cursor.target_x;
        }
    }
}
