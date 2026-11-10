mod anchored;
mod dir;
mod fade_frame;
mod feedback;
mod folio;
mod gallery;
mod layout;
mod overlay;
mod r1_style;
mod sheet;
mod style;
mod typography;

pub use anchored::{AnchorSide, anchored};
pub(crate) use dir::row;
pub use dir::{Flow, end, logical_padding, mirror_x, mirror_x_for, rtl, start, start_for};
#[cfg(test)]
pub use dir::{end_for, logical_padding_for, ordered};
pub use fade_frame::FadeFrame;
pub use feedback::Spinner;
pub use folio::{
    BUTTON_HEIGHT as FOLIO_BUTTON_HEIGHT, EYE_CLOSED, FOLIO_INDEX_WIDTH, TWISTY_CLOSED,
    TWISTY_OPEN, diamond, folio_action, folio_action_bar, folio_action_width,
    folio_action_width_in, folio_action_wrap, folio_destructive_action, folio_details, folio_field,
    folio_ghost_field, folio_index_shell, folio_index_shell_tinted, folio_inline_bar,
    folio_masthead, folio_scrim_style, folio_sheet, folio_sheet_panel_style, folio_stack_bar,
};
pub(crate) use folio::{folio_diagonal_edges, folio_diagonal_wipe};
#[cfg(test)]
pub use gallery::accent_on_ink;
pub use gallery::{
    INK, choice_button_style, filter_button_style, floating_panel_style, grid_pattern,
    hairline_style, ink_fill, ink_palette, plain_row_style, quiet_input_style, sheet_style,
    switch_thumb_style, switch_track_style, tag_style, text_button_style, tile_action_style,
    tile_button_style, tile_style, veil_style,
};
pub(crate) use layout::FOLIO_RULE_THICKNESS;
#[cfg(test)]
pub use layout::MIN_VIEWPORT;
pub use layout::{
    FOLIO_RULE_ALPHA, folio_horizontal_rule, folio_rule, folio_scroll_padding, folio_sheet_dims,
    hud, wrap_rows,
};
pub use overlay::{field_input, inert_backdrop};
pub use r1_style::{
    focus_row_style, ghost_button_style, rounded_button_style, soft_button_style, surface_style,
};
pub use sheet::{SheetPosition, SheetWindow};
pub use style::{
    bg_style, box_style, flat_button_style, folio_button_style, folio_line_button_style,
    ghost_input_style, panel_style, scrim, scrim_style, workbench_input_style,
};
#[cfg(test)]
pub use typography::sentence_case;
pub use typography::{
    DISPLAY_FONT, MONO_FONT, NERD_FONT, TEXT_FONT, TEXT_STRONG_FONT, TYPE_SMALL, UI_FONT,
    UI_FONT_FAMILY, label, legible_type_scale, mid_text, typed,
};
pub(crate) use typography::{ellipsize_text, glyph_width, text_width};

mod tests;
