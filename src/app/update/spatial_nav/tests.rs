#![cfg(test)]
use super::*;

fn grid(rects: &[(usize, f32, f32, f32, f32)]) -> Vec<Rect> {
    rects.iter().copied().map(Rect::from_place).collect()
}

#[test]
fn uniform_grid_moves_straight() {
    forget_goal();
    let cards: Vec<_> =
        (0..12).map(|i| (i, (i % 4) as f32 * 110.0, (i / 4) as f32 * 70.0, 100.0, 60.0)).collect();
    let rects = grid(&cards);
    assert_eq!(neighbour(&rects, 5, Dir::Down), Some(9));
    assert_eq!(neighbour(&rects, 9, Dir::Down), None);
    assert_eq!(neighbour(&rects, 9, Dir::Up), Some(5));
    assert_eq!(neighbour(&rects, 5, Dir::Right), Some(6));
    assert_eq!(neighbour(&rects, 3, Dir::Right), None);
}

#[test]
fn masonry_down_stays_in_its_column() {
    forget_goal();
    // Two columns filled shortest-first: index + 2 is not below index.
    let rects = grid(&[
        (0, 0.0, 0.0, 100.0, 200.0),
        (1, 110.0, 0.0, 100.0, 60.0),
        (2, 110.0, 70.0, 100.0, 60.0),
        (3, 110.0, 140.0, 100.0, 60.0),
        (4, 0.0, 210.0, 100.0, 60.0),
        (5, 110.0, 210.0, 100.0, 60.0),
    ]);
    assert_eq!(neighbour(&rects, 1, Dir::Down), Some(2));
    assert_eq!(neighbour(&rects, 2, Dir::Down), Some(3));
    assert_eq!(neighbour(&rects, 3, Dir::Down), Some(5));
    assert_eq!(neighbour(&rects, 0, Dir::Down), Some(4));
    assert_eq!(neighbour(&rects, 2, Dir::Left), Some(0));
}

#[test]
fn brick_rows_do_not_drift() {
    forget_goal();
    // Odd rows shifted by half a card.
    let cards: Vec<_> = (0..20)
        .map(|i| {
            let row = i / 5;
            let shift = if row % 2 == 1 { 55.0 } else { 0.0 };
            (i, (i % 5) as f32 * 110.0 + shift, row as f32 * 70.0, 100.0, 60.0)
        })
        .collect();
    let rects = grid(&cards);
    let mut current = 2;
    let mut columns = Vec::new();
    while let Some(next) = neighbour(&rects, current, Dir::Down) {
        current = next;
        columns.push(rects[current].cx);
    }
    assert_eq!(columns.len(), 3, "reaches the last row");
    // Even rows land back in the starting column every time.
    assert!((columns[1] - rects[2].cx).abs() < 1.0, "{columns:?}");
    assert!(columns.iter().all(|x| (x - rects[2].cx).abs() <= 56.0), "{columns:?}");
}
