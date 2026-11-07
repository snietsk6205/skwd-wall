use serde_json::Value;

use super::super::Config;

fn percent(value: f32) -> f64 {
    (f64::from(value) * 100_000.0).round() / 1_000.0
}

impl Config {
    pub fn selector_preset_snapshot(&self) -> Value {
        let mut map = serde_json::Map::new();
        match self.selector_mode().as_str() {
            "grid" => super::preset_grid::snapshot(self, &mut map),
            "hex" => {
                map.insert("hexParallax".into(), Value::from(self.hex_parallax()));
                map.insert("hexRadius".into(), Value::from(self.hex_radius() as f64));
                map.insert("hexRows".into(), Value::from(self.hex_rows() as u64));
                map.insert("hexCols".into(), Value::from(self.hex_cols() as u64));
                map.insert("hexScrollStep".into(), Value::from(self.hex_scroll_step() as u64));
                map.insert("hexArc".into(), Value::from(self.hex_arc()));
                map.insert("hexArcIntensity".into(), Value::from(self.hex_arc_intensity() as f64));
                map.insert("hexCurve".into(), Value::from(self.hex_curve()));
                map.insert("hexShape".into(), Value::from(self.hex_shape()));
                map.insert(
                    "hexCurveFrequency".into(),
                    Value::from(self.hex_curve_frequency() as f64),
                );
                map.insert("hexGapX".into(), Value::from(self.hex_gap_x() as f64));
                map.insert("hexGapY".into(), Value::from(self.hex_gap_y() as f64));
                map.insert("hexAspect".into(), Value::from(percent(self.hex_aspect())));
                map.insert("hexStagger".into(), Value::from(percent(self.hex_stagger())));
                map.insert("hexLens".into(), Value::from(percent(self.hex_lens())));
                map.insert("hexLensRadius".into(), Value::from(self.hex_lens_radius() as f64));
                map.insert("hexOrbit".into(), Value::from(percent(self.hex_orbit())));
                map.insert("hexOrbitRadius".into(), Value::from(self.hex_orbit_radius() as f64));
                map.insert("hexTwist".into(), Value::from(self.hex_twist() as f64));
                map.insert("hexScatter".into(), Value::from(self.hex_scatter() as f64));
                let [x, y, scale, rotation, perspective, shear_x, shear_y, depth_angle] =
                    self.hex_stage();
                map.insert("hexStageX".into(), Value::from(percent(x)));
                map.insert("hexStageY".into(), Value::from(percent(y)));
                map.insert("hexStageScale".into(), Value::from(percent(scale)));
                map.insert("hexStageRotation".into(), Value::from(rotation as f64));
                map.insert("hexStagePerspective".into(), Value::from(percent(perspective)));
                map.insert("hexStageShearX".into(), Value::from(percent(shear_x)));
                map.insert("hexStageShearY".into(), Value::from(percent(shear_y)));
                map.insert("hexStageDepthAngle".into(), Value::from(depth_angle as f64));
            }
            "sandy" => self.snapshot_sandy(&mut map),
            "hand" => self.snapshot_hand(&mut map),
            "depth" => {
                map.insert("depthShadows".into(), Value::from(self.depth_shadows()));
                map.insert("depthSelectionFrame".into(), Value::from(self.depth_selection_frame()));
                map.insert("depthHeight".into(), Value::from(self.depth_height() as f64));
                map.insert("depthWidthPx".into(), Value::from(self.depth_width() as f64));
                map.insert("depthSpacingPx".into(), Value::from(self.depth_spacing() as f64));
                map.insert("depthCount".into(), Value::from(self.depth_count() as f64));
                map.insert("depthFalloffFactor".into(), Value::from(self.depth_falloff() as f64));
                map.insert("depthCorners".into(), Value::from(self.depth_corners() as f64));
                map.insert("depthSkew".into(), Value::from(self.depth_skew() as f64));
                map.insert(
                    "depthNavigationMs".into(),
                    Value::from(self.depth_navigation_ms() as f64),
                );
            }
            "collection" => {
                map.insert("collectionSize".into(), Value::from(self.collection_size() as f64));
                map.insert(
                    "collectionSpacing".into(),
                    Value::from(self.collection_spacing() as f64),
                );
                map.insert("collectionCount".into(), Value::from(self.collection_count() as f64));
                map.insert("collectionTilt".into(), Value::from(self.collection_tilt() as f64));
                map.insert(
                    "collectionCorners".into(),
                    Value::from(self.collection_corners() as f64),
                );
                map.insert("collectionSpeed".into(), Value::from(self.collection_speed() as f64));
            }
            "coverflow" => self.snapshot_coverflow(&mut map),
            "showcase" => self.snapshot_showcase(&mut map),

            _ => self.snapshot_slices(&mut map),
        }
        Value::Object(map)
    }

    fn snapshot_coverflow(&self, map: &mut serde_json::Map<String, Value>) {
        use skwd_config::keys::selector as sel;
        let style = self.custom_text(sel::COVERFLOW_STYLE);
        let style = if style.is_empty() { String::from("coverflow") } else { style };
        map.insert("coverflowStyle".into(), Value::from(style));
        for (name, key, default, min, max) in [
            ("coverflowSize", sel::COVERFLOW_SIZE, 46.0, 20.0, 75.0),
            ("coverflowAngle", sel::COVERFLOW_ANGLE, 62.0, 0.0, 85.0),
            ("coverflowSpacing", sel::COVERFLOW_SPACING, 26.0, 8.0, 90.0),
            ("coverflowCount", sel::COVERFLOW_COUNT, 5.0, 1.0, 12.0),
            ("coverflowCorners", sel::COVERFLOW_CORNERS, 10.0, 0.0, 40.0),
            ("coverflowSpeed", sel::COVERFLOW_SPEED, 420.0, 120.0, 1500.0),
        ] {
            map.insert(
                name.into(),
                Value::from(f64::from(self.custom_num(key, default, min, max))),
            );
        }
        map.insert(
            "coverflowReflection".into(),
            Value::from(self.custom_flag(sel::COVERFLOW_REFLECTION, true)),
        );
        map.insert(
            "coverflowBackdrop".into(),
            Value::from(self.custom_flag(sel::COVERFLOW_BACKDROP, true)),
        );
    }

    fn snapshot_showcase(&self, map: &mut serde_json::Map<String, Value>) {
        use skwd_config::keys::selector as sel;
        for (name, key, default, min, max) in [
            ("showcaseHero", sel::SHOWCASE_HERO, 62.0, 35.0, 85.0),
            ("showcaseStrip", sel::SHOWCASE_STRIP, 92.0, 48.0, 180.0),
            ("showcaseCorners", sel::SHOWCASE_CORNERS, 14.0, 0.0, 40.0),
            ("showcaseSpeed", sel::SHOWCASE_SPEED, 380.0, 120.0, 1500.0),
        ] {
            map.insert(
                name.into(),
                Value::from(f64::from(self.custom_num(key, default, min, max))),
            );
        }
        let position = self.custom_text(sel::SHOWCASE_STRIP_POSITION);
        let position = if position == "top" { "top" } else { "bottom" };
        map.insert("showcaseStripPosition".into(), Value::from(position));
        map.insert(
            "showcaseBackdrop".into(),
            Value::from(self.custom_flag(sel::SHOWCASE_BACKDROP, true)),
        );
    }

    fn snapshot_hand(&self, map: &mut serde_json::Map<String, Value>) {
        let (x, y) = self.hand_position();
        map.insert("handStageX".into(), Value::from(percent(x)));
        map.insert("handStageY".into(), Value::from(percent(y)));
        map.insert("handCount".into(), Value::from(self.hand_count() as u64));
        map.insert("handCardWidth".into(), Value::from(self.hand_card_width() as f64));
        map.insert("handCardHeight".into(), Value::from(self.hand_card_height() as f64));
        map.insert("handSpread".into(), Value::from(self.hand_spread() as f64));
        map.insert("handRibbons".into(), Value::from(self.hand_ribbons() as u64));
        map.insert("handFanAngle".into(), Value::from(self.hand_fan_angle() as f64));
        map.insert("handFanRoll".into(), Value::from(self.hand_fan_roll() as f64));
        map.insert("handArch".into(), Value::from(self.hand_arch() as f64));
        map.insert("handCornerRadius".into(), Value::from(self.hand_corner_radius() as f64));
        map.insert("handSkew".into(), Value::from(self.hand_skew() as f64));
        map.insert(
            "handBackdropBlur".into(),
            Value::from((self.hand_backdrop_blur() * 100.0) as f64),
        );
        map.insert("handRibbonAxis".into(), Value::from(self.hand_ribbon_axis()));
        map.insert("handCut".into(), Value::from(self.hand_cut()));
        map.insert("handCutVariance".into(), Value::from(self.hand_cut_variance()));
        map.insert("handMove".into(), Value::from(self.hand_move()));
        let [corkscrew, cascade, shuffle, ribbon, spiral] = self.hand_moves();
        map.insert("handMoveCorkscrew".into(), Value::from(corkscrew));
        map.insert("handMoveCascade".into(), Value::from(cascade));
        map.insert("handMoveShuffle".into(), Value::from(shuffle));
        map.insert("handMoveRibbon".into(), Value::from(ribbon));
        map.insert("handMoveSpiral".into(), Value::from(spiral));
        map.insert("handSpeed".into(), Value::from((self.hand_speed() * 100.0) as f64));
        map.insert("handTilt".into(), Value::from((self.hand_tilt() * 100.0) as f64));
        map.insert("handPerspective".into(), Value::from(self.hand_perspective() as f64));
        map.insert("handGhosts".into(), Value::from(self.hand_ghosts()));
        map.insert("handBob".into(), Value::from(self.hand_bob()));
        map.insert("handBackdrop".into(), Value::from(self.hand_backdrop()));
        map.insert("handRevealFill".into(), Value::from(self.hand_reveal_fill()));
    }

    fn snapshot_sandy(&self, map: &mut serde_json::Map<String, Value>) {
        let (x, y) = self.sandy_position();
        map.insert("sandyStageX".into(), Value::from(percent(x)));
        map.insert("sandyStageY".into(), Value::from(percent(y)));
        map.insert("sandyCenter".into(), Value::from(self.sandy_center() as f64));
        map.insert("sandySliceWidth".into(), Value::from(self.sandy_slice_width() as f64));
        map.insert("sandySliceHeight".into(), Value::from(self.sandy_slice_height() as f64));
        map.insert("sandySkew".into(), Value::from(self.sandy_skew() as f64));
        map.insert("sandySpacing".into(), Value::from(self.sandy_spacing() as f64));
        map.insert("sandyDuration".into(), Value::from(self.sandy_duration() as f64));
        map.insert("sandyBlend".into(), Value::from(self.sandy_blend() as f64));
        map.insert("sandyStrands".into(), Value::from(self.sandy_strands() as f64));
        map.insert("sandyTwist".into(), Value::from((self.sandy_twist() * 100.0) as f64));
        map.insert("sandyOrbit".into(), Value::from((self.sandy_orbit() * 100.0) as f64));
        map.insert("sandyTurbulence".into(), Value::from((self.sandy_turbulence() * 100.0) as f64));
        map.insert("sandyWaist".into(), Value::from((self.sandy_waist() * 100.0) as f64));
        map.insert("sandyFront".into(), Value::from((self.sandy_front() / 0.65 * 100.0) as f64));
        map.insert("sandyFan".into(), Value::from((self.sandy_fan() / 0.6 * 100.0) as f64));
        map.insert("sandyArc".into(), Value::from((self.sandy_arc() * 100.0) as f64));
        map.insert("sandySwapLoop".into(), Value::from(self.sandy_swap_loop()));
        map.insert("sandySwapStyle".into(), Value::from(self.sandy_swap_style()));
        map.insert(
            "sandyEdgeSpeed".into(),
            Value::from((self.sandy_edge_speed() / 14.0 * 100.0) as f64),
        );
        map.insert("sandyRingSpin".into(), Value::from((self.sandy_ring_spin() * 100.0) as f64));
        map.insert("sandyRingSize".into(), Value::from((self.sandy_ring_size() * 100.0) as f64));
        map.insert("sandyRingWave".into(), Value::from((self.sandy_ring_wave() * 100.0) as f64));
        map.insert("sandyRingSoft".into(), Value::from((self.sandy_ring_soft() * 100.0) as f64));
        map.insert("sandyRingBlend".into(), Value::from((self.sandy_ring_blend() * 100.0) as f64));
        map.insert("sandyRingHold".into(), Value::from((self.sandy_ring_hold() * 1000.0) as f64));
        map.insert("sandyGrain".into(), Value::from(self.sandy_grain() as f64));
        map.insert("sandyResScale".into(), Value::from((self.sandy_res_scale() * 100.0) as f64));
        map.insert("sandyLod".into(), Value::from(self.sandy_lod() as f64));
        map.insert("sandyLodAuto".into(), Value::from(self.sandy_lod_auto()));
        map.insert("sandyOutgoingLive".into(), Value::from(self.sandy_video_out_live()));
    }

    fn snapshot_slices(&self, map: &mut serde_json::Map<String, Value>) {
        map.insert("sliceShadows".into(), Value::from(self.slice_shadows()));
        map.insert("sliceParallax".into(), Value::from(self.slice_parallax()));
        let (x, y) = self.slice_position();
        map.insert("sliceStageX".into(), Value::from(percent(x)));
        map.insert("sliceStageY".into(), Value::from(percent(y)));
        map.insert("sliceHeight".into(), Value::from(self.slice_height() as f64));
        map.insert("visibleCount".into(), Value::from(self.visible_count() as u64));
        map.insert("expandedWidth".into(), Value::from(self.expanded_width() as f64));
        map.insert("sliceWidth".into(), Value::from(self.slice_width() as f64));
        map.insert("sliceSpacing".into(), Value::from(self.slice_spacing() as f64));
        map.insert("skewOffset".into(), Value::from(self.skew_offset() as f64));
        map.insert("sliceEdgeTilt".into(), Value::from(self.slice_edge_tilt() as f64));
        map.insert("sliceWobble".into(), Value::from(self.slice_wobble()));
        map.insert(
            "sliceWobbleStrength".into(),
            Value::from((self.slice_wobble_strength() * 100.0) as f64),
        );
        let round = self.bool_false_unless_true(skwd_config::keys::selector::ROUND_CORNERS);
        map.insert("roundCorners".into(), Value::from(round));
        let base = self.num_at(skwd_config::keys::selector::CORNER_RADIUS, 16.0);
        for key in ["cornerTL", "cornerTR", "cornerBR", "cornerBL"] {
            let value = self.num_at(&format!("components.wallpaperSelector.{key}"), base);
            map.insert(key.into(), Value::from(value));
        }
    }

    pub fn selector_presets(&self, mode: &str) -> Vec<(String, Value)> {
        self.get(&format!("components.wallpaperSelector.presets.{mode}"))
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| {
                        Some((
                            entry.get("name")?.as_str()?.to_string(),
                            entry.get("params")?.clone(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn selected_preset(&self, mode: &str) -> Option<String> {
        self.get(&format!("components.wallpaperSelector.activePreset.{mode}"))
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .map(String::from)
    }

    pub fn next_preset_name(&self, mode: &str, candidate: impl FnMut(usize) -> String) -> String {
        let existing = self.selector_presets(mode);
        (1..=existing.len() + 1)
            .map(candidate)
            .find(|name| !existing.iter().any(|(other, _)| other == name))
            .expect("one of n + 1 generated preset names must be available")
    }
}
