use std::sync::Arc;

use crate::frontend::animation::Spring;

use super::InstanceRaw;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LaunchAnim {
    None,
    Fade,
    Rise,
    Zoom,
    /// Custom build: cards rise in one after another, left to right.
    Cascade,
    /// Custom build: cards grow in from the centre of the screen outwards.
    Bloom,
}

impl LaunchAnim {
    pub fn from_key(kind: &str) -> Self {
        match kind {
            "none" => Self::None,
            "rise" => Self::Rise,
            "zoom" => Self::Zoom,
            "cascade" => Self::Cascade,
            "bloom" => Self::Bloom,
            _ => Self::Fade,
        }
    }
}

pub(crate) struct Transition {
    pub(crate) old: Arc<Vec<InstanceRaw>>,
    pub(crate) progress: Spring,
    pub(crate) kind: u32,
    pub(crate) origin: [f32; 2],
}

pub(crate) fn apply_entrance_motion(
    instances: &mut [InstanceRaw],
    animation: LaunchAnim,
    entrance: f32,
    viewport_width: f32,
    viewport_height: f32,
) {
    if entrance >= 0.999 {
        return;
    }
    match animation {
        LaunchAnim::Rise => {
            let offset_y = (1.0 - entrance).powi(2) * viewport_height * 0.03;
            for instance in instances {
                instance.rect[1] += offset_y;
            }
        }
        LaunchAnim::Zoom => {
            let scale = 0.94 + 0.06 * entrance;
            let (center_x, center_y) = (viewport_width * 0.5, viewport_height * 0.5);
            for instance in instances {
                instance.rect[0] = center_x + (instance.rect[0] - center_x) * scale;
                instance.rect[1] = center_y + (instance.rect[1] - center_y) * scale;
                instance.rect[2] *= scale;
                instance.rect[3] *= scale;
            }
        }
        LaunchAnim::Cascade | LaunchAnim::Bloom => {
            stagger_entrance(instances, animation, entrance, viewport_width, viewport_height);
        }
        LaunchAnim::None | LaunchAnim::Fade => {}
    }
}

/// Copies of the renderer's instance flags (`rendering::scene`), which the
/// frontend layer may not import; `app` tests check they still match.
pub(crate) const BACKDROP_FLAG: u32 = 16_384;
pub(crate) const PROJECTED_FLAG: u32 = 128;

/// Share of the entrance spent waiting for the last card's turn.
const STAGGER_SPAN: f32 = 0.5;

/// Custom build: per-card entrance. Each card waits for its turn (by
/// position), then eases in; the full-screen backdrop only fades.
fn stagger_entrance(
    instances: &mut [InstanceRaw],
    animation: LaunchAnim,
    entrance: f32,
    viewport_width: f32,
    viewport_height: f32,
) {
    let (mid_x, mid_y) = (viewport_width * 0.5, viewport_height * 0.5);
    let reach = (mid_x * mid_x + mid_y * mid_y).sqrt().max(1.0);
    let entrance = entrance.clamp(0.0, 1.0);
    for instance in instances {
        if instance.misc[3] & BACKDROP_FLAG != 0 {
            continue;
        }
        let projected = instance.misc[3] & PROJECTED_FLAG != 0;
        let (center_x, center_y) = if projected {
            (
                (instance.quad_a[0] + instance.quad_a[2] + instance.quad_b[0] + instance.quad_b[2])
                    * 0.25,
                (instance.quad_a[1] + instance.quad_a[3] + instance.quad_b[1] + instance.quad_b[3])
                    * 0.25,
            )
        } else {
            (instance.rect[0], instance.rect[1])
        };
        let turn = match animation {
            LaunchAnim::Cascade => (center_x / viewport_width.max(1.0)).clamp(0.0, 1.0),
            _ => (((center_x - mid_x).powi(2) + (center_y - mid_y).powi(2)).sqrt() / reach)
                .clamp(0.0, 1.0),
        };
        let local = ((entrance - turn * STAGGER_SPAN) / (1.0 - STAGGER_SPAN)).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - local).powi(3);
        // Layouts already multiply opacity by `entrance`; swap it for this card's own.
        instance.params[2] =
            if entrance > 0.001 { (instance.params[2] / entrance * eased).min(1.0) } else { 0.0 };
        let (shift_y, scale) = match animation {
            LaunchAnim::Cascade => ((1.0 - eased) * viewport_height * 0.07, 1.0),
            _ => (0.0, 0.8 + 0.2 * eased),
        };
        let place = |x: f32, y: f32| {
            (center_x + (x - center_x) * scale, center_y + (y - center_y) * scale + shift_y)
        };
        instance.rect[0] = center_x + (instance.rect[0] - center_x) * scale;
        instance.rect[1] += shift_y;
        instance.rect[2] *= scale;
        instance.rect[3] *= scale;
        if projected {
            for (x, y) in [(0, 1), (2, 3)] {
                let (ax, ay) = place(instance.quad_a[x], instance.quad_a[y]);
                instance.quad_a[x] = ax;
                instance.quad_a[y] = ay;
                let (bx, by) = place(instance.quad_b[x], instance.quad_b[y]);
                instance.quad_b[x] = bx;
                instance.quad_b[y] = by;
            }
        }
    }
}
