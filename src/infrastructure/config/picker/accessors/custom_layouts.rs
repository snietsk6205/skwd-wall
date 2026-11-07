//! Custom build: plain readers for the custom layouts (built into params in
//! `app/startup.rs`) and the bar.

use skwd_config::keys::filter_bar;

use super::super::Config;

impl Config {
    pub(crate) fn custom_num(&self, key: &str, default: f64, min: f64, max: f64) -> f32 {
        let value = self.num_at(key, default);
        if value.is_finite() { value.clamp(min, max) as f32 } else { default as f32 }
    }

    pub(crate) fn custom_flag(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(serde_json::Value::as_bool).unwrap_or(default)
    }

    pub(crate) fn custom_text(&self, key: &str) -> String {
        self.get(key).and_then(serde_json::Value::as_str).unwrap_or_default().to_string()
    }

    /// Size of the bar relative to the UI scale (1 = normal).
    pub fn filter_bar_scale(&self) -> f32 {
        self.custom_num(filter_bar::SCALE, 1.0, 0.6, 1.8)
    }

    /// Where the bar sits: `auto` (upstream placement), `top`, `bottom`, `left` or `right`.
    pub fn filter_bar_anchor(&self) -> String {
        match self.custom_text(filter_bar::ANCHOR).as_str() {
            anchor @ ("top" | "bottom" | "left" | "right") => anchor.to_string(),
            _ => String::from("auto"),
        }
    }

    /// Whether the bar is a vertical rail, taking the anchor into account.
    pub fn filter_bar_effective_vertical(&self, mode: crate::contracts::picker::Mode) -> bool {
        match self.filter_bar_anchor().as_str() {
            "left" | "right" => true,
            "top" | "bottom" => false,
            _ => {
                self.filter_bar_orientation() == "vertical"
                    && mode != crate::contracts::picker::Mode::Sandy
            }
        }
    }

    /// Whether the bar's menus open upwards (the bar sits low on the screen).
    pub fn filter_bar_menu_up(&self, mode: crate::contracts::picker::Mode) -> bool {
        match self.filter_bar_anchor().as_str() {
            "bottom" => true,
            "top" | "left" | "right" => false,
            _ => mode == crate::contracts::picker::Mode::Sandy,
        }
    }

    /// The bar's button group order as saved (see `frontend::ui::bar_order`).
    pub fn filter_bar_order_text(&self) -> String {
        self.custom_text(filter_bar::ORDER)
    }
}
