use std::sync::Arc;
use std::time::Instant;

use crate::frontend::animation::MotionTier;

use crate::frontend::scene::cell_key;
use crate::frontend::scene::layout::Mode;

use super::layout_helpers::{SCROLL_DECAY, ramp_toward};
use super::model::{RebuildCtx, SceneCore};

impl SceneCore {
    pub fn conceal_hero(&mut self, until: Instant) {
        if self.mode == Mode::Sandy && self.sandy_hero_animating() {
            self.sandy.hero_fade.snap(0.0);
            self.sandy_settle_now();
        }
        self.sandy.hero_fade.retarget(0.0);
        self.sandy.hero_restore_at = Some(until);
        self.motion.needs_frame = true;
    }

    pub fn reveal_hero(&mut self) {
        if self.mode == Mode::Sandy && self.sandy_hero_animating() {
            self.sandy_settle_now();
        }
        self.sandy.hero_restore_at = None;
        self.sandy.hero_fade.retarget(1.0);
        self.motion.needs_frame = true;
    }

    pub fn hero_concealed(&self) -> bool {
        self.sandy.hero_restore_at.is_some() || self.sandy.hero_fade.target < 0.5
    }

    pub(super) fn hero_fade_tick(&mut self, now: Instant, dt: f32) {
        self.sandy.hero_fade.tick(dt);
        if let Some(at) = self.sandy.hero_restore_at
            && now >= at
        {
            self.reveal_hero();
        }
    }

    pub fn tick(&mut self, now: Instant, mut ctx: RebuildCtx<'_>) {
        let dt =
            self.motion.last_tick.map_or(1.0 / 60.0, |prev| now.duration_since(prev).as_secs_f32());
        self.motion.last_tick = Some(now);
        let rebuild_needed = self.is_animating();
        if rebuild_needed
            && log::log_enabled!(log::Level::Debug)
            && self
                .motion
                .last_anim_log
                .is_none_or(|prev| now.duration_since(prev).as_millis() >= 1000)
        {
            self.motion.last_anim_log = Some(now);
            log::debug!("anim held by: {}", self.anim_reason());
        }
        self.scroll_vel *= (-dt * SCROLL_DECAY).exp();
        if self.scroll_vel < 0.05 {
            self.scroll_vel = 0.0;
        }
        self.card.flip.tick(dt);
        self.card.tag_open.tick(dt);
        self.card.chip_pop.tick(dt);
        if self.card.det_p != self.card.det_target {
            let seconds = self.motion_ms(MotionTier::Standard) / 1_000.0;
            self.card.det_p =
                ramp_toward(self.card.det_p, self.card.det_target, dt.min(1.0 / 30.0) / seconds);
        }
        if (self.card.fav_fill - self.card.fav_target).abs() > 0.0001 {
            let seconds = self.motion_ms(MotionTier::Fast) / 1_000.0;
            self.card.fav_fill =
                ramp_toward(self.card.fav_fill, self.card.fav_target, dt.min(1.0 / 30.0) / seconds);
        }
        if self.flip_closed() {
            self.card.flipped = None;
        }
        self.tick_springs(dt);
        self.tick_sandy(&ctx, now, dt);
        self.tick_hand(&ctx, dt);
        self.tick_custom_layouts(dt);
        self.tick_transition(dt);
        if self.hidden() {
            if self.render.vis != 0.0 {
                Arc::make_mut(&mut self.render).vis = 0.0;
            }
            self.teardown_previews();
        } else {
            let preview_dirty = self.manage_preview(&mut ctx);
            if rebuild_needed || preview_dirty {
                self.rebuild(ctx);
            }
        }
        self.motion.needs_frame = false;
        if !self.motion.first_frame_logged {
            self.motion.first_frame_logged = true;
            crate::infrastructure::observability::log_startup_checkpoint("first_scene_frame");
            crate::shell::on_first_gpu_frame();
        }
        self.start_pending_entrance();
        if !self.motion.ready_logged && self.render.instances.iter().any(|inst| inst.misc[0] > 0) {
            self.motion.ready_logged = true;
            crate::infrastructure::observability::log_startup_checkpoint("first_thumbs_visible");
        }
    }

    fn flip_closed(&self) -> bool {
        if matches!(self.mode, Mode::Slices | Mode::Depth | Mode::Sandy | Mode::Hand) {
            return self.card.flip.settled() && self.card.flip.target == 0.0;
        }
        self.card.det_target == 0.0 && self.card.det_p <= 0.001
    }

    fn tick_springs(&mut self, dt: f32) {
        self.motion.entrance.tick(dt);
        let step = dt.min(0.05) * 1000.0 / self.card.filter_ms.max(50.0);
        if self.card.filter_wave < 10.0 {
            self.card.filter_wave += step;
        }
        if !self.card.filter_old.is_empty() {
            for (_, _, t) in &mut self.card.filter_old {
                *t += step;
            }
            self.card.filter_old.retain(|(_, _, t)| *t < 1.0);
            self.card.filter_cell.clear();
            for (idx, (inst, _, _)) in self.card.filter_old.iter().enumerate() {
                self.card.filter_cell.insert(cell_key(inst.rect[0], inst.rect[1]), idx);
            }
        }
        if !self.card.filter_in.is_empty() {
            for t in self.card.filter_in.values_mut() {
                *t += step;
            }
            self.card.filter_in.retain(|_, t| *t < 1.0);
        }
        self.motion.visibility.tick(dt);
        self.motion.center_inset.tick(dt);
        self.morph_params(dt);
        self.camera.tick_with_precision(dt, self.camera_precision());
        self.collection_open.tick(dt);
        if self.collection_open.target == 0.0 && self.collection_open.settled() {
            self.collection_card = None;
        }
        for spr in self.card.widths.values_mut() {
            spr.tick(dt);
        }
        let sp = self.sp;
        let current = self.current;
        self.card
            .widths
            .retain(|&idx, spr| !(spr.settled() && spr.target == sp.slice_w && idx != current));
        for spr in self.card.fades.values_mut() {
            spr.tick(dt);
        }
        self.card.fades.retain(|_, spr| !spr.settled());
        for spr in self.card.hex_scales.values_mut() {
            spr.tick(dt);
        }
        self.card.hex_scales.retain(|_, spr| !(spr.settled() && spr.target == 0.0));
        let precision = if matches!(self.mode, Mode::Depth | Mode::Coverflow | Mode::Showcase) {
            0.001
        } else {
            0.05
        };
        for spr in self.card.selection.values_mut() {
            spr.tick_with_precision(dt, precision);
        }
        self.card.selection.retain(|_, spr| !spr.settled_with_precision(precision));
    }

    fn tick_transition(&mut self, dt: f32) {
        if let Some(trans) = self.motion.transition.as_mut() {
            trans.progress.tick(dt);
            if trans.progress.settled() {
                self.motion.transition = None;
            }
        }
    }
}
