//! Direct translation of component_list_wrap_virtualized_test.cpp at dc75beed.
#[path = "support/virtual_scroll.rs"]
mod support;
use support::*;

#[test]
fn wrapped_virtualized_list_scrolls_along_cross_axis() {
    let f = ScrollFixture::wrap(1);
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.virtual_axis_is_column()));
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        340.0,
    );
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
}
#[test]
fn wrapped_virtualized_list_places_items_on_lines() {
    let f = ScrollFixture::wrap(1);
    let b = f.bounds(5);
    approx(b.left(), 110.0);
    approx(b.top(), 70.0);
    approx(b.width(), 100.0);
    approx(b.height(), 60.0);
    let b = f.bounds(19);
    approx(b.left(), 330.0);
    approx(b.top(), 280.0);
    let origin = f.drawn_at(0);
    let at5 = f.drawn_at(5) - origin;
    approx(at5.x, 110.0);
    approx(at5.y, 70.0);
    let at10 = f.drawn_at(10) - origin;
    approx(at10.x, 220.0);
    approx(at10.y, 140.0);
}
#[test]
fn wrapped_virtualized_list_recycles_whole_lines() {
    let f = ScrollFixture::wrap(1);
    f.offset_y(-100.0);
    f.settle();
    assert_eq!(f.realized(), (4..20).collect::<Vec<_>>());
}
#[test]
fn wrapped_virtualized_list_buffers_lines_not_items() {
    let f = ScrollFixture::wrap(1);
    uint(&f.scroll, Scroll::VIRTUALIZE_BUFFER_PROPERTY_KEY, 1);
    f.settle();
    assert_eq!(f.realized(), (0..16).collect::<Vec<_>>());
}
#[test]
fn hugging_wrapped_list_breaks_lines_at_viewport_width() {
    let f = ScrollFixture::wrap(2);
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
    approx(f.bounds(5).left(), 110.0);
    approx(f.bounds(5).top(), 70.0);
}
#[test]
fn wrapped_list_scrolled_both_ways_indexes_by_line() {
    let f = ScrollFixture::wrap(1);
    f.direction(2);
    f.settle();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.virtual_axis_is_column()));
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
    f.scroll_y(-140.0);
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.scroll_index()),
        8.0,
    );
    f.scroll_y(0.0);
    write::<ScrollConstraint, _>(&f.scroll, |s| s.set_scroll_index(9.0));
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_y()),
        -140.0,
    );
}
#[test]
fn wrapped_list_scrolled_both_ways_moves_along_lines() {
    let f = ScrollFixture::wrap(1);
    f.fixed_wrap_width();
    f.direction(2);
    f.settle();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.max_offset_x()) < 0.0);
    let before = f.drawn_at(1);
    f.scroll_x(-200.0);
    f.settle();
    let after = f.drawn_at(1);
    approx(after.x - before.x, -200.0);
    approx(after.y - before.y, 0.0);
}
#[test]
fn wrapped_carousel_both_ways_loops_only_lines() {
    let f = ScrollFixture::wrap(1);
    f.fixed_wrap_width();
    f.direction(2);
    boolean(&f.scroll, Scroll::INFINITE_PROPERTY_KEY, true);
    f.settle();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.loops_y()));
    assert!(!read::<ScrollConstraint, _>(&f.scroll, |s| s.loops_x()));
    let max_x = read::<ScrollConstraint, _>(&f.scroll, |s| s.max_offset_x());
    assert!(max_x < 0.0);
    assert!(max_x > -800.0);
    write::<ScrollConstraint, _>(&f.scroll, |s| s.scroll_by(Vec2D::new(-1000.0, -1000.0)));
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_x()),
        max_x,
    );
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_y()),
        -1000.0,
    );
    assert!(!read::<ScrollConstraint, _>(&f.scroll, |s| s.can_consume(Vec2D::new(-50.0, 0.0))));
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.can_consume(Vec2D::new(0.0, -50.0))));
}
#[test]
fn wrapped_list_scrolled_only_along_lines_moves() {
    let f = ScrollFixture::wrap(1);
    f.fixed_wrap_width();
    f.direction(0);
    f.settle();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.virtual_axis_is_column()));
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.max_offset_x()) < 0.0);
    let before = f.drawn_at(1);
    f.scroll_x(-200.0);
    f.settle();
    let after = f.drawn_at(1);
    approx(after.x - before.x, -200.0);
    approx(after.y - before.y, 0.0);
}
#[test]
fn virtualization_toggled_realizes_and_recycles_items() {
    let f = ScrollFixture::wrap(1);
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
    boolean(&f.scroll, Scroll::VIRTUALIZE_PROPERTY_KEY, false);
    f.settle();
    assert_eq!(f.realized(), (0..20).collect::<Vec<_>>());
    boolean(&f.scroll, Scroll::VIRTUALIZE_PROPERTY_KEY, true);
    f.settle();
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
}

// Supplemental ownership-boundary coverage of the same upstream toggle case.
// Unlike an empty owner, realization here traverses the live list and layouts.
#[test]
fn populated_virtualization_toggle_preserves_borrowed_and_handle_setter_routes() {
    use nuxie_runtime::source::generated::core_registry::{
        CoreField, CoreRegistry, CoreRegistryObject,
    };
    for route in 0..3 {
        let f = ScrollFixture::wrap(1);
        assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
        for (enabled, count) in [(false, 20), (true, 12)] {
            match route {
                0 => write::<ScrollConstraint, _>(&f.scroll, |s| {
                    CoreRegistry::set_bool(s, i32::from(Scroll::VIRTUALIZE_PROPERTY_KEY), enabled)
                }),
                1 => write::<ScrollConstraint, _>(&f.scroll, |s| {
                    CoreRegistryObject::set_bool(s, CoreField::ScrollConstraintVirtualize, enabled)
                }),
                _ => boolean(&f.scroll, Scroll::VIRTUALIZE_PROPERTY_KEY, enabled),
            }
            f.settle();
            assert_eq!(
                f.realized(),
                (0..count).collect::<Vec<_>>(),
                "route {route}, enabled {enabled}"
            );
        }
    }
}
#[test]
fn hugging_wrapped_list_breaks_inside_padding_and_margins() {
    let f = ScrollFixture::wrap(2);
    let s = f.content_style();
    number(&s, Style::PADDING_LEFT_PROPERTY_KEY, 40.0);
    number(&s, Style::PADDING_RIGHT_PROPERTY_KEY, 40.0);
    f.settle();
    approx(f.bounds(3).left(), 40.0);
    approx(f.bounds(3).top(), 70.0);
    let f = ScrollFixture::wrap(2);
    let s = f.content_style();
    number(&s, Style::MARGIN_LEFT_PROPERTY_KEY, 40.0);
    number(&s, Style::MARGIN_RIGHT_PROPERTY_KEY, 40.0);
    uint(&s, Style::MARGIN_LEFT_UNITS_VALUE_PROPERTY_KEY, 1);
    uint(&s, Style::MARGIN_RIGHT_UNITS_VALUE_PROPERTY_KEY, 1);
    f.settle();
    approx(f.bounds(3).top(), 70.0);
}
