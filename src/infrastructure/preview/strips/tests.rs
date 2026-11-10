#![cfg(test)]

use super::{
    FRAME_BYTES, STRIP_FRAMES, STRIP_H, STRIP_MIDDLE, STRIP_W, cache_key, decode_strip,
    encode_strip, kept_frames, strip_file_name,
};

fn frame(shade: u8) -> Vec<u8> {
    [shade, shade, shade, 255].repeat(FRAME_BYTES / 4)
}

#[test]
fn file_names_are_safe() {
    assert_eq!(strip_file_name("burn"), "burn.jpg");
    assert_eq!(strip_file_name("page-curl"), "page-curl.jpg");
    assert_eq!(strip_file_name("combo:split-lr:burn:ripple"), "combo_split-lr_burn_ripple.jpg");
    assert_eq!(strip_file_name("../x"), "___x.jpg");
}

#[test]
fn cache_key_follows_the_renderer() {
    let key = cache_key((1000, 5), (10, 20));
    assert_eq!(key.len(), 16);
    assert_eq!(key, cache_key((1000, 5), (10, 20)));
    assert_ne!(key, cache_key((1001, 5), (10, 20)));
    assert_ne!(key, cache_key((1000, 6), (10, 20)));
    assert_ne!(key, cache_key((1000, 5), (11, 20)));
}

#[test]
fn keeps_the_frames_between_the_two_pictures() {
    let mut raw = Vec::new();
    for shade in 0..(STRIP_FRAMES as u8 + 2) {
        raw.extend(frame(shade * 10));
    }
    let kept = kept_frames(&raw).unwrap();
    assert_eq!(kept.len(), STRIP_FRAMES);
    assert_eq!(kept[0][0], 10);
    assert_eq!(kept[STRIP_FRAMES - 1][0], STRIP_FRAMES as u8 * 10);
    // A renderer that stopped early gives no strip.
    assert!(kept_frames(&raw[..FRAME_BYTES * 5]).is_none());
}

#[test]
fn strip_survives_saving() {
    let frames: Vec<Vec<u8>> = (0..STRIP_FRAMES as u8).map(|index| frame(index * 20)).collect();
    let refs: Vec<&[u8]> = frames.iter().map(Vec::as_slice).collect();
    let bytes = encode_strip(&refs).unwrap();
    let back = decode_strip(&bytes).unwrap();
    assert_eq!(back.len(), STRIP_FRAMES);
    assert!(back.iter().all(|frame| frame.len() == (STRIP_W * STRIP_H * 4) as usize));
    let middle = i32::from(back[STRIP_MIDDLE][0]);
    assert!((middle - STRIP_MIDDLE as i32 * 20).abs() <= 3, "{middle}");
    assert!(decode_strip(b"not a picture").is_none());
    assert!(encode_strip(&[&[0u8; 8]]).is_none());
}
