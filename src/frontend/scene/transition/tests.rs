#![cfg(test)]

use super::*;

#[test]
fn launch_anim_key_mapping() {
    assert_eq!(LaunchAnim::from_key("none"), LaunchAnim::None);
    assert_eq!(LaunchAnim::from_key("rise"), LaunchAnim::Rise);
    assert_eq!(LaunchAnim::from_key("zoom"), LaunchAnim::Zoom);
    assert_eq!(LaunchAnim::from_key("fade"), LaunchAnim::Fade);
    assert_eq!(LaunchAnim::from_key("garbage"), LaunchAnim::Fade);
    assert_eq!(LaunchAnim::from_key("cascade"), LaunchAnim::Cascade);
    assert_eq!(LaunchAnim::from_key("bloom"), LaunchAnim::Bloom);
}

#[test]
fn entrance_motion_shapes() {
    let card = InstanceRaw { rect: [100.0, 200.0, 40.0, 30.0], ..Default::default() };

    let mut insts = [card];
    apply_entrance_motion(&mut insts, LaunchAnim::Rise, 0.0, 1920.0, 1080.0);
    assert!(insts[0].rect[1] > 200.0);

    let mut insts = [card];
    apply_entrance_motion(&mut insts, LaunchAnim::Zoom, 0.0, 1920.0, 1080.0);
    assert!(insts[0].rect[2] < 40.0);

    for anim in [LaunchAnim::None, LaunchAnim::Fade] {
        let mut insts = [card];
        apply_entrance_motion(&mut insts, anim, 0.0, 1920.0, 1080.0);
        assert_eq!(insts[0].rect, card.rect);
    }

    let mut insts = [card];
    apply_entrance_motion(&mut insts, LaunchAnim::Zoom, 1.0, 1920.0, 1080.0);
    assert_eq!(insts[0].rect, card.rect);
}

#[test]
fn staggered_entrances_take_turns_and_finish_in_place() {
    let left = InstanceRaw {
        rect: [100.0, 540.0, 40.0, 30.0],
        params: [0.0, 0.0, 0.5, 0.0],
        ..Default::default()
    };
    let right = InstanceRaw { rect: [1800.0, 540.0, 40.0, 30.0], ..left };
    let mut insts = [left, right];
    apply_entrance_motion(&mut insts, LaunchAnim::Cascade, 0.5, 1920.0, 1080.0);
    assert!(insts[0].params[2] > insts[1].params[2], "the left card comes first");
    assert!(insts[1].rect[1] > insts[0].rect[1], "a later card is still lower");

    let centre = InstanceRaw { rect: [960.0, 540.0, 40.0, 30.0], ..left };
    let corner = InstanceRaw { rect: [60.0, 60.0, 40.0, 30.0], ..left };
    let mut insts = [centre, corner];
    apply_entrance_motion(&mut insts, LaunchAnim::Bloom, 0.4, 1920.0, 1080.0);
    assert!(insts[0].rect[2] > insts[1].rect[2], "the centre grows in first");

    for anim in [LaunchAnim::Cascade, LaunchAnim::Bloom] {
        let mut insts = [left, right, corner];
        apply_entrance_motion(&mut insts, anim, 1.0, 1920.0, 1080.0);
        assert_eq!(insts.map(|inst| inst.rect), [left, right, corner].map(|inst| inst.rect));
    }
}
