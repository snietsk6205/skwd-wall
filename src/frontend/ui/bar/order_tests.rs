use super::*;

#[test]
fn empty_is_upstream_order() {
    assert_eq!(bar_order(""), BAR_GROUPS.to_vec());
}

#[test]
fn listed_groups_come_first_and_rest_follow() {
    let order = bar_order("settings, colors,bogus,colors");
    assert_eq!(&order[..2], &["settings", "colors"]);
    assert_eq!(order.len(), BAR_GROUPS.len());
    assert_eq!(order[2], "types");
}

#[test]
fn moving_swaps_neighbours_and_stops_at_the_ends() {
    assert!(bar_order_moved("", "orient", true).starts_with("orient,types,"));
    assert!(bar_order_moved("", "types", true).starts_with("types,orient,"));
    assert!(bar_order_moved("", "stash", false).ends_with(",settings,stash"));
    assert!(bar_order_moved("", "settings", false).ends_with(",stash,settings"));
}
