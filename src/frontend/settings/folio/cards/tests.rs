#![cfg(test)]

use super::{card_height, columns_for, wide_page_width};

#[test]
fn four_cards_per_row_on_the_small_screen() {
    // The user's screen gives the picker 1280x720: a 1258 px wide window.
    let width = wide_page_width(1258.0, 1.0);
    assert!((width - 1146.0).abs() < f32::EPSILON);
    assert_eq!(columns_for(width, 1.0), 4);
}

#[test]
fn columns_follow_the_width() {
    assert_eq!(columns_for(300.0, 1.0), 2);
    assert_eq!(columns_for(620.0, 1.0), 2);
    assert_eq!(columns_for(800.0, 1.0), 3);
    assert_eq!(columns_for(1180.0, 1.0), 4);
    assert_eq!(columns_for(4000.0, 1.0), 6);
    // A bigger scale makes bigger cards, so fewer fit.
    assert_eq!(columns_for(1180.0, 1.5), 3);
}

#[test]
fn page_width_has_limits() {
    assert!((wide_page_width(200.0, 1.0) - 320.0).abs() < f32::EPSILON);
    assert!((wide_page_width(3000.0, 1.0) - 1180.0).abs() < f32::EPSILON);
}

#[test]
fn two_rows_of_cards_fit_under_the_header() {
    // 720 px screen: a 698 px window, a 46 px bar, and 176 px of page header
    // and toolbar above the grid (measured on a screenshot). `card_height`
    // includes the gap under a row; the last row needs none.
    let width = wide_page_width(1258.0, 1.0);
    let row = card_height(width, 1.0);
    assert!(2.0 * row - 14.0 <= 698.0 - 46.0 - 176.0, "{row}");
}
