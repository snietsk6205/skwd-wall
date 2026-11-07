//! Custom build: the Coverflow and Showcase picker layouts.
//!
//! Both are one row of cards navigated with Left/Right, the wheel or a click,
//! like Depth: the camera spring follows `current`, and the per-card
//! `selection` springs (set in `set_current`) drive the smooth focus.

use glam::{Mat4, Vec3};

use crate::frontend::animation::smoothstep;
use crate::frontend::scene::InstanceRaw;
use crate::frontend::scene::hand::{self, Camera, PAD, Quad};
use crate::frontend::scene::layout::{CoverflowStyle, Mode, StripPosition};
use crate::rendering::scene::atlas::AtlasMap;
use crate::rendering::scene::{BACKDROP, PROJECTED, REFLECTION};

use super::layout_helpers::{CardSpec, color4};
use super::model::{RebuildCtx, RebuildSinks, SceneCore};

/// Wallpapers are mostly 16:9; side cards use this shape so the row is even.
const ROW_ASPECT: f32 = 16.0 / 9.0;

/// One card of the Coverflow row, before drawing.
struct Pose {
    idx: usize,
    matrix: Mat4,
    opacity: f32,
    shade: f32,
    depth: f32,
}

impl SceneCore {
    fn uses_ambient_backdrop(&self) -> bool {
        match self.mode {
            Mode::Coverflow => self.xp.coverflow.backdrop,
            Mode::Showcase => self.xp.showcase.backdrop,
            _ => false,
        }
    }

    /// The backdrop cross-fade borrows the Hand layout's timer, which only
    /// Hand advances; the custom layouts advance it here.
    pub(super) fn tick_custom_layouts(&mut self, dt: f32) {
        if !self.uses_ambient_backdrop() {
            return;
        }
        self.hand.backdrop_fade.tick(dt);
        if self.hand.backdrop_fade.settled() {
            self.hand.backdrop_prev = None;
        }
    }

    pub(super) fn custom_layouts_animating(&self) -> bool {
        self.uses_ambient_backdrop() && !self.hand.backdrop_fade.settled()
    }

    fn follow_camera(&mut self, count: usize) -> f32 {
        self.camera.set_zeta(1.0);
        if self.layout_camera_anchor {
            self.camera.snap(self.current as f32);
            self.layout_camera_anchor = false;
        } else {
            self.camera.retarget(self.current as f32);
        }
        self.camera.x.clamp(0.0, count.saturating_sub(1) as f32)
    }

    fn selection_of(&self, idx: usize) -> f32 {
        let fallback = if idx == self.current { 1.0 } else { 0.0 };
        self.card.selection.get(&idx).map_or(fallback, |spring| spring.x.clamp(0.0, 1.0))
    }

    /// The blurred, darkened current wallpaper behind the cards; it fades
    /// between wallpapers as the selection moves.
    fn ambient_backdrop_layers(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) {
        let Some(&store) = ctx.filtered.get(self.current) else {
            return;
        };
        let store = store as usize;
        let ready = ctx.atlas.as_ref().is_some_and(|atlas| atlas.near.ready(store).is_some());
        let (store, prev, fade) = self.hand_backdrop_target(store, ready);
        if let Some(prev) = prev.filter(|_| fade < 0.999) {
            self.ambient_backdrop(ctx, sinks, prev, entrance, 1.2);
        }
        if let Some(store) = store {
            let alpha = if prev.is_some() { fade } else { 1.0 };
            self.ambient_backdrop(ctx, sinks, store, entrance * alpha, 1.2);
        }
    }

    pub(super) fn ambient_backdrop(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        store: usize,
        alpha: f32,
        blur: f32,
    ) {
        let (vw, vh) = self.viewport;
        let mut inst = self.body_instance(
            ctx,
            sinks.wanted,
            store,
            [vw * 0.5, vh * 0.5, vw * 0.5, vh * 0.5],
            [0.0; 4],
            0.0,
            0.0,
            true,
        );
        if inst.misc[0] == 3
            && let Some(id) = ctx.atlas.as_ref().and_then(|atlas| atlas.near.ready(store))
        {
            let (offset, scale, layer) = AtlasMap::near_uv(id);
            inst.misc[0] = 1;
            inst.misc[1] = layer;
            inst.uv = [offset[0], offset[1], scale[0], scale[1]];
            inst.params[3] = 1.0;
        }
        if inst.misc[0] != 1 {
            return;
        }
        inst.misc[3] |= BACKDROP;
        inst.params[2] = alpha;
        inst.flip[3] = blur;
        inst.border = [0.0; 4];
        sinks.instances.push(inst);
    }

    pub(super) fn rebuild_coverflow(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) {
        let count = ctx.filtered.len();
        if count == 0 {
            self.vis_lo = 0;
            self.vis_hi = 0;
            return;
        }
        self.current = self.current.min(count - 1);
        let center = self.follow_camera(count);
        let params = self.xp.coverflow;
        if params.backdrop {
            self.ambient_backdrop_layers(ctx, sinks, entrance);
        }
        let side = params.count.round().max(1.0);
        let reach = match params.style {
            CoverflowStyle::Stack => side * 2.0,
            _ => side,
        };
        let low = ((center - reach - 1.0).floor().max(0.0) as usize).min(count - 1);
        let high = ((center + reach + 1.0).ceil() as usize).min(count - 1);
        self.vis_lo = low;
        self.vis_hi = high;
        let (_, vh) = self.viewport;
        let mut card_h = vh * params.size / 100.0;
        let mut card_w = card_h * ROW_ASPECT;
        let (fit, mid_y) = self.coverflow_fit(card_w, side, params.style);
        card_h *= fit;
        card_w *= fit;
        let cx = self.center_x();
        let cy = match params.style {
            // Centre the whole receding stack between the bar and the bottom.
            CoverflowStyle::Stack => vh * 0.6 - mid_y * card_w,
            _ if params.reflection => vh * 0.47,
            _ => vh * 0.5,
        };
        let camera = Camera {
            d: coverflow_camera_distance(card_w, side, params.spacing, params.style),
            origin: [cx, cy],
            shift: [0.0, 0.0],
        };
        let mut poses: Vec<Pose> = (low..=high)
            .filter_map(|idx| self.coverflow_pose(idx, center, reach, card_w, card_h, entrance))
            .collect();
        poses.sort_by(|a, b| a.depth.total_cmp(&b.depth));
        self.card.filter_cache.clear();
        for pose in poses {
            self.coverflow_card(ctx, sinks, &camera, &pose, card_w, card_h);
        }
    }

    /// Scale that keeps the row on screen, measured by projecting the
    /// outermost cards of a row whose cards are 1 unit wide.
    fn coverflow_fit(&self, card_w: f32, side: f32, style: CoverflowStyle) -> (f32, f32) {
        let half_view = (self.avail_w() * 0.5 - 24.0).max(1.0);
        let unit_h = 1.0 / ROW_ASPECT;
        let camera = Camera {
            d: coverflow_camera_distance(1.0, side, self.xp.coverflow.spacing, style),
            origin: [0.0, 0.0],
            shift: [0.0, 0.0],
        };
        let corners = hand::card_corners(0.5, unit_h * 0.5, 0.0, 0.0);
        let reach = if style == CoverflowStyle::Stack { side * 2.0 } else { side };
        let mut spread = 0.5f32;
        let (mut top, mut bottom) = (-unit_h * 0.5, unit_h * 0.5);
        for step in 0..=(reach as usize) {
            let Some(pose) = self.coverflow_pose(step, 0.0, reach, 1.0, unit_h, 1.0) else {
                continue;
            };
            let quad = hand::project_quad(&camera, &pose.matrix, &corners);
            for point in quad.pts {
                spread = spread.max(point[0].abs());
                top = top.min(point[1]);
                bottom = bottom.max(point[1]);
            }
        }
        // The carousel ring may run past the screen edges a little.
        let allowance = if style == CoverflowStyle::Carousel { 1.3 } else { 1.0 };
        let width_fit = half_view * allowance / spread / card_w.max(1.0);
        let room = if style == CoverflowStyle::Stack { 0.62 } else { 0.84 };
        let tallest =
            if style == CoverflowStyle::Stack { bottom - top } else { 2.0 * top.abs().max(bottom) };
        let height_fit = self.viewport.1 * room / tallest / card_w.max(1.0);
        (width_fit.min(height_fit).min(1.0), (top + bottom) * 0.5)
    }

    fn coverflow_pose(
        &self,
        idx: usize,
        center: f32,
        reach: f32,
        card_w: f32,
        card_h: f32,
        entrance: f32,
    ) -> Option<Pose> {
        let params = self.xp.coverflow;
        let n = idx as f32 - center;
        let edge = (reach + 1.0 - n.abs()).clamp(0.0, 1.0);
        let (matrix, mut opacity, shade, depth) = match params.style {
            CoverflowStyle::Coverflow => {
                let t = smoothstep(n.abs().min(1.0));
                let s = n.signum();
                let turned = card_w * 0.5 * params.angle.to_radians().cos().max(0.25);
                let x = s * t * (card_w * 0.5 + turned * 0.9)
                    + s * (n.abs() - 1.0).max(0.0) * card_w * params.spacing / 100.0;
                let z = -t * card_w * 0.42;
                let angle = -s * t * params.angle.to_radians();
                let matrix =
                    Mat4::from_translation(Vec3::new(x, 0.0, z)) * Mat4::from_rotation_y(angle);
                (matrix, edge, (n.abs() * 0.07).min(0.45), -n.abs())
            }
            CoverflowStyle::Carousel => {
                let (radius, step) = carousel_ring(card_w, reach, params.spacing);
                let theta = n * step;
                if theta.abs() > std::f32::consts::FRAC_PI_2 * 1.08 {
                    return None;
                }
                let x = radius * theta.sin();
                let z = radius * (theta.cos() - 1.0);
                let tilt = Mat4::from_rotation_x(-(8.0f32).to_radians());
                let matrix = tilt
                    * Mat4::from_translation(Vec3::new(x, 0.0, z))
                    * Mat4::from_rotation_y(theta * params.angle / 62.0);
                let back = (1.0 - theta.cos()).clamp(0.0, 1.0);
                (matrix, edge, back * 0.55, z)
            }
            CoverflowStyle::Stack => {
                let spacing = params.spacing / 26.0;
                let (x, y, z, fade) = if n >= 0.0 {
                    (0.0, -n * card_h * 0.22 * spacing, -n * card_w * 0.45 * spacing, edge)
                } else {
                    // Cards already passed fly towards you and vanish.
                    let t = -n;
                    (0.0, t * card_h * 0.25, t * card_w * 0.55, (1.0 - t).clamp(0.0, 1.0))
                };
                let matrix = Mat4::from_translation(Vec3::new(x, y, z))
                    * Mat4::from_rotation_y(-(params.angle * 0.12).to_radians());
                (matrix, fade, (n.max(0.0) * 0.09).min(0.6), -n)
            }
        };
        opacity *= entrance;
        (opacity > 0.002).then_some(Pose { idx, matrix, opacity, shade, depth })
    }

    fn coverflow_card(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        camera: &Camera,
        pose: &Pose,
        card_w: f32,
        card_h: f32,
    ) {
        let params = self.xp.coverflow;
        let store = ctx.filtered[pose.idx] as usize;
        let item = &ctx.catalog.items[store];
        let aspect = if item.width > 0 && item.height > 0 {
            (item.width as f32 / item.height as f32).clamp(0.5, 2.4)
        } else {
            ROW_ASPECT
        };
        // Keep the row height even; very wide or tall pictures are cropped a little.
        let hh = card_h * 0.5;
        let hw = (card_h * aspect.clamp(1.2, 2.2)).min(card_w * 1.15) * 0.5;
        let corners = hand::card_corners(hw, hh, PAD, PAD);
        let quad = hand::project_quad(camera, &pose.matrix, &corners);
        if !quad.facing() {
            return;
        }
        let radius = params.corners.min(hw.min(hh));
        if params.reflection && params.style != CoverflowStyle::Stack {
            let below = pose.matrix * Mat4::from_translation(Vec3::new(0.0, hh * 2.0 + 6.0, 0.0));
            let mirror = hand::project_quad(camera, &below, &corners);
            let mut inst = self.body_instance(
                ctx,
                sinks.wanted,
                store,
                [mirror.bounds()[0], mirror.bounds()[1], hw, hh],
                [radius; 4],
                0.0,
                0.0,
                false,
            );
            inst.misc[3] |= PROJECTED | REFLECTION;
            inst.params[2] = pose.opacity * 0.9;
            inst.tint = [0.0, 0.0, 0.0, pose.shade];
            set_quad(&mut inst, &mirror, hh);
            sinks.instances.push(inst);
        }
        let mut shadow_quad = quad.pts;
        for point in &mut shadow_quad {
            point[1] += 8.0;
        }
        let shadow = Quad { pts: shadow_quad, depth: quad.depth };
        let mut shadow_inst = InstanceRaw {
            rect: [shadow.bounds()[0], shadow.bounds()[1], hw, hh],
            radii: [radius; 4],
            fill: [0.0, 0.0, 0.0, 0.30],
            params: [0.0, 0.0, pose.opacity, 0.0],
            misc: [0, 0, 0, PROJECTED],
            ..Default::default()
        };
        set_quad(&mut shadow_inst, &shadow, hh);
        sinks.instances.push(shadow_inst);

        let selected = self.selection_of(pose.idx);
        let [bx, by, bhw, bhh] = quad.bounds();
        let chrome_opacity = if pose.idx == self.current { pose.opacity } else { 0.0 };
        let (mut body, mut hit) = self.place_card(
            ctx,
            sinks.wanted,
            sinks.chrome,
            &CardSpec {
                filtered_idx: pose.idx,
                store_idx: store,
                cx: bx,
                cy: by,
                hw,
                hh,
                skew: 0.0,
                edge_tilt: 0.0,
                radii: [radius; 4],
                hex: false,
                view: 0,
                chrome_radius: radius,
                opacity: pose.opacity,
                chrome_opacity,
                body_inset: 0.0,
                near_ok: true,
            },
        );
        body.misc[3] |= PROJECTED;
        set_quad(&mut body, &quad, hh);
        let accent = ctx.palette.primary;
        let hover = Some(pose.idx) == self.hover;
        body.border = if selected > 0.01 || hover {
            color4(accent, (0.9 * selected).max(if hover { 0.6 } else { 0.0 }))
        } else {
            [1.0, 1.0, 1.0, 0.12]
        };
        body.params[1] = 1.0 + 1.5 * selected;
        body.tint = [0.0, 0.0, 0.0, pose.shade * (1.0 - selected)];
        hit.hw = bhw;
        hit.hh = bhh;
        hit.quad = Some(quad.pts.map(|point| [point[0], point[1]]));
        sinks.instances.push(body);
        sinks.hits.push(hit);
    }

    pub(super) fn rebuild_showcase(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) {
        let count = ctx.filtered.len();
        if count == 0 {
            self.vis_lo = 0;
            self.vis_hi = 0;
            return;
        }
        self.current = self.current.min(count - 1);
        let center = self.follow_camera(count);
        let params = self.xp.showcase;
        if params.backdrop {
            self.ambient_backdrop_layers(ctx, sinks, entrance);
        }
        let (_, vh) = self.viewport;
        let strip_h = params.strip.min(vh * 0.22).max(24.0);
        let strip_w = strip_h * ROW_ASPECT;
        let stride = strip_w + (strip_h * 0.14).max(6.0);
        let margin = match params.strip_position {
            // Leave room for the theme swatches in the bottom corner.
            StripPosition::Bottom => 64.0,
            StripPosition::Top => 26.0,
        };
        let top_room = 64.0;
        let (strip_y, hero_top, hero_bottom) = match params.strip_position {
            StripPosition::Bottom => {
                let strip_y = vh - margin - strip_h * 0.5;
                (strip_y, top_room, strip_y - strip_h * 0.5 - margin)
            }
            StripPosition::Top => {
                let strip_y = top_room + strip_h * 0.5;
                (strip_y, strip_y + strip_h * 0.5 + margin, vh - margin)
            }
        };
        let cx = self.center_x();
        let half_view = self.avail_w() * 0.5;
        let side = (half_view / stride).ceil() + 1.0;
        let low = ((center - side).floor().max(0.0) as usize).min(count - 1);
        let high = ((center + side).ceil() as usize).min(count - 1);
        self.vis_lo = low;
        self.vis_hi = high;
        self.card.filter_cache.clear();

        // The big preview: every card that is (or was just) selected, cross-fading.
        let hero_room_h = (hero_bottom - hero_top).max(40.0);
        let hero_cy = (hero_top + hero_bottom) * 0.5;
        let mut heroes: Vec<(usize, f32)> = (low..=high)
            .map(|idx| (idx, self.selection_of(idx)))
            .filter(|(_, amount)| *amount > 0.003)
            .collect();
        heroes.sort_by(|a, b| a.1.total_cmp(&b.1));
        for (idx, amount) in heroes {
            self.showcase_hero(ctx, sinks, idx, amount, entrance, cx, hero_cy, hero_room_h);
        }

        // The filmstrip.
        for idx in low..=high {
            let n = idx as f32 - center;
            let x = cx + n * stride;
            if (x - cx).abs() > half_view + stride {
                continue;
            }
            let selected = self.selection_of(idx);
            let edge = 1.0
                - smoothstep((((x - cx).abs() - (half_view - stride)) / stride).clamp(0.0, 1.0));
            let opacity = edge.clamp(0.0, 1.0) * entrance;
            if opacity <= 0.002 {
                continue;
            }
            let lift = match params.strip_position {
                StripPosition::Bottom => -8.0 * selected,
                StripPosition::Top => 8.0 * selected,
            };
            let grow = 1.0 + 0.12 * selected;
            let hw = strip_w * 0.5 * grow;
            let hh = strip_h * 0.5 * grow;
            let store = ctx.filtered[idx] as usize;
            let radius = (params.corners * 0.6).min(hh);
            sinks.instances.push(InstanceRaw {
                rect: [x + 2.0, strip_y + lift + 5.0, hw, hh],
                radii: [radius; 4],
                fill: [0.0, 0.0, 0.0, 0.32],
                params: [0.0, 0.0, opacity, 0.0],
                ..Default::default()
            });
            let (mut body, hit) = self.place_card(
                ctx,
                sinks.wanted,
                sinks.chrome,
                &CardSpec {
                    filtered_idx: idx,
                    store_idx: store,
                    cx: x,
                    cy: strip_y + lift,
                    hw,
                    hh,
                    skew: 0.0,
                    edge_tilt: 0.0,
                    radii: [radius; 4],
                    hex: false,
                    view: 0,
                    chrome_radius: radius,
                    opacity,
                    chrome_opacity: 0.0,
                    body_inset: 0.0,
                    near_ok: false,
                },
            );
            let hover = Some(idx) == self.hover;
            let accent = ctx.palette.primary;
            body.border = if selected > 0.01 || hover {
                color4(accent, (0.95 * selected).max(if hover { 0.7 } else { 0.0 }))
            } else {
                [1.0, 1.0, 1.0, 0.10]
            };
            body.params[1] = 1.0 + 1.5 * selected;
            body.tint = [0.0, 0.0, 0.0, 0.38 * (1.0 - selected) * if hover { 0.4 } else { 1.0 }];
            sinks.instances.push(body);
            sinks.hits.push(hit);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn showcase_hero(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        idx: usize,
        amount: f32,
        entrance: f32,
        cx: f32,
        cy: f32,
        room_h: f32,
    ) {
        let params = self.xp.showcase;
        let store = ctx.filtered[idx] as usize;
        let item = &ctx.catalog.items[store];
        let aspect = if item.width > 0 && item.height > 0 {
            (item.width as f32 / item.height as f32).clamp(0.4, 3.2)
        } else {
            ROW_ASPECT
        };
        let mut width = self.avail_w() * params.hero / 100.0;
        let mut height = width / aspect;
        if height > room_h {
            height = room_h;
            width = height * aspect;
        }
        // The new picture settles from slightly larger; the old one shrinks away.
        let eased = smoothstep(amount);
        let leaving = idx != self.current;
        let scale = if leaving { 0.97 + 0.03 * eased } else { 1.035 - 0.035 * eased };
        let hw = width * 0.5 * scale;
        let hh = height * 0.5 * scale;
        let radius = params.corners.min(hw.min(hh));
        let opacity = eased * entrance;
        sinks.instances.push(InstanceRaw {
            rect: [cx, cy + 14.0, hw, hh],
            radii: [radius; 4],
            fill: [0.0, 0.0, 0.0, 0.45],
            params: [0.0, 22.0, opacity, 0.0],
            misc: [0, 0, 0, 64],
            ..Default::default()
        });
        let (mut body, hit) = self.place_card(
            ctx,
            sinks.wanted,
            sinks.chrome,
            &CardSpec {
                filtered_idx: idx,
                store_idx: store,
                cx,
                cy,
                hw,
                hh,
                skew: 0.0,
                edge_tilt: 0.0,
                radii: [radius; 4],
                hex: false,
                view: 0,
                chrome_radius: radius,
                opacity,
                chrome_opacity: if leaving { 0.0 } else { opacity },
                body_inset: 0.0,
                near_ok: true,
            },
        );
        body.border = [1.0, 1.0, 1.0, 0.14 * opacity];
        body.params[1] = 1.0;
        sinks.instances.push(body);
        if !leaving {
            sinks.hits.push(hit);
        }
    }
}

/// Camera distance (perspective strength) for a row of `card_w` wide cards.
fn coverflow_camera_distance(card_w: f32, side: f32, spacing: f32, style: CoverflowStyle) -> f32 {
    match style {
        CoverflowStyle::Carousel => carousel_ring(card_w, side, spacing).0 + card_w * 2.2,
        CoverflowStyle::Coverflow | CoverflowStyle::Stack => card_w * 2.8,
    }
}

/// Radius and angle step of the Carousel ring for `side` cards on each side.
fn carousel_ring(card_w: f32, side: f32, spacing: f32) -> (f32, f32) {
    let step = (1.55 / side.max(1.0)).clamp(0.12, 1.2);
    let chord = card_w * (1.0 + spacing / 100.0);
    let radius = chord / (2.0 * (step * 0.5).sin()).max(0.05);
    (radius, step)
}

fn set_quad(inst: &mut InstanceRaw, quad: &Quad, hh: f32) {
    let [tl, tr, br, bl] = quad.pts;
    inst.quad_a = [tl[0], tl[1], tr[0], tr[1]];
    inst.quad_b = [br[0], br[1], bl[0], bl[1]];
    inst.quad_w = [tl[2], tr[2], br[2], bl[2]];
    inst.quad_l = hand::card_locals(hh, PAD);
}
