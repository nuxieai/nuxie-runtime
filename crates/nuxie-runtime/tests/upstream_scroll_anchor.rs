//! Direct translation of scroll_anchor_test.cpp at dc75beed.
#[path = "support/virtual_scroll.rs"]
mod support;
use support::*;
fn anchor(offset: f32) -> ScrollFixture {
    let f = ScrollFixture::new("component_list_virtualized.riv");
    boolean(&f.scroll, Scroll::INFINITE_PROPERTY_KEY, false);
    f.offset_x(offset);
    f.settle();
    f
}
fn measure(f: &ScrollFixture, index: i32, width: f32) {
    write::<ArtboardComponentList, _>(&f.list, |list| {
        list.set_item_size(Vec2D::new(width, 60.0), index)
    });
    ScrollConstraint::constrain_virtualized_occurrence(&f.scroll, true);
}
#[test]
fn scroll_anchoring_keeps_first_item_in_place() {
    let f = anchor(-500.0);
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_x()),
        -500.0,
    );
    measure(&f, 1, 150.0);
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_x()),
        -550.0,
    );
}
#[test]
fn scroll_anchoring_ignores_items_after_first_on_screen() {
    let f = anchor(-500.0);
    measure(&f, 12, 150.0);
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_x()),
        -500.0,
    );
}
#[test]
fn scroll_anchoring_lets_content_grow_below_at_start() {
    let f = anchor(0.0);
    measure(&f, 1, 150.0);
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_x()),
        0.0,
    );
}
