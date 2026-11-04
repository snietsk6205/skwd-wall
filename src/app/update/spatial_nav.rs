//! Arrow-key navigation by on-screen position.
//!
//! Grid styles such as masonry, brick, justified and editorial, and the hex
//! layouts with arcs, twist or orbit, do not keep "the card below" at
//! `index + columns`. Moving by index made Down jump left or diagonally.
//! Here Up/Down (and Left/Right in masonry) pick the card that is actually
//! next in that direction, keeping a remembered "goal" column so repeated
//! presses run straight instead of drifting.

use std::sync::Mutex;

/// A card's rectangle: index, centre and half size (any consistent space).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub(crate) index: usize,
    pub(crate) cx: f32,
    pub(crate) cy: f32,
    pub(crate) hw: f32,
    pub(crate) hh: f32,
}

impl Rect {
    pub(crate) fn from_place((index, x, y, w, h): (usize, f32, f32, f32, f32)) -> Self {
        Self { index, cx: x + w * 0.5, cy: y + h * 0.5, hw: w * 0.5, hh: h * 0.5 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }
}

/// The goal column (or row, for Left/Right) of the last keyboard move.
struct Goal {
    index: usize,
    vertical: bool,
    at: f32,
}

static GOAL: Mutex<Option<Goal>> = Mutex::new(None);

/// The card next to `current` in direction `dir`, or `None` at the edge.
pub(crate) fn neighbour(rects: &[Rect], current: usize, dir: Dir) -> Option<usize> {
    let from = rects.iter().find(|rect| rect.index == current)?;
    let mut goal = GOAL.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let at = goal
        .as_ref()
        .filter(|goal| goal.index == current && goal.vertical == dir.vertical())
        .map_or(if dir.vertical() { from.cx } else { from.cy }, |goal| goal.at);
    let next = pick(rects, from, dir, at)?;
    *goal = Some(Goal { index: next, vertical: dir.vertical(), at });
    Some(next)
}

fn pick(rects: &[Rect], from: &Rect, dir: Dir, goal: f32) -> Option<usize> {
    let mut best: Option<(f32, usize)> = None;
    for rect in rects.iter().filter(|rect| rect.index != from.index) {
        // Distance along the direction of travel, between centres.
        let (ahead, min_half, lateral_centre, lateral_half) = match dir {
            Dir::Down => (rect.cy - from.cy, rect.hh.min(from.hh), rect.cx, rect.hw),
            Dir::Up => (from.cy - rect.cy, rect.hh.min(from.hh), rect.cx, rect.hw),
            Dir::Right => (rect.cx - from.cx, rect.hw.min(from.hw), rect.cy, rect.hh),
            Dir::Left => (from.cx - rect.cx, rect.hw.min(from.hw), rect.cy, rect.hh),
        };
        if ahead < min_half * 0.6 {
            continue;
        }
        // No sideways cost while the goal line runs through the card.
        let off = (lateral_centre - goal).abs();
        let outside = (off - lateral_half).max(0.0);
        let score = ahead + outside * 2.5 + off * 0.02;
        if best.is_none_or(|(known, index)| {
            score < known - 0.01 || ((score - known).abs() <= 0.01 && rect.index < index)
        }) {
            best = Some((score, rect.index));
        }
    }
    best.map(|(_, index)| index)
}

#[cfg(test)]
pub(crate) fn forget_goal() {
    *GOAL.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

#[cfg(test)]
mod tests;
