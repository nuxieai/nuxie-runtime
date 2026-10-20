//! A translated ScrollConstraint's content box has a zero automatic minimum
//! on the axes it scrolls (docs/CSS_LAYOUT.md, section 2): a Rive author who
//! Fills scroll content means "as big as the viewport, the items scroll",
//! which CSS says with `min-height: 0` there. Upstream's virtualized lists
//! then keep the viewport-sized content box and the window C++ realizes.
#[path = "support/virtual_scroll.rs"]
mod support;
use support::*;

fn layout_height(f: &ScrollFixture, name: &str) -> f32 {
    let handle = f
        .artboard
        .with_artboard(|artboard| artboard.find_handle::<LayoutComponent>(name))
        .unwrap_or_else(|| panic!("{name} layout"));
    read::<LayoutComponent, _>(&handle, LayoutComponent::layout_height)
}

#[test]
fn virtualized_grid_content_stays_the_viewport_height() {
    let f = ScrollFixture::grid();
    approx(layout_height(&f, "Viewport"), 200.0);
    approx(layout_height(&f, "Content"), 200.0);
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        360.0,
    );
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
}

/// With virtualization off every item is laid out in Content, and with a
/// buffer more are. Content still stays the viewport's height, so turning
/// virtualization back on realizes C++'s window again.
#[test]
fn wrapped_content_stays_the_viewport_height_whatever_is_realized() {
    let f = ScrollFixture::wrap(1);
    approx(layout_height(&f, "Viewport"), 200.0);
    approx(layout_height(&f, "Content"), 200.0);
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        340.0,
    );
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
    boolean(&f.scroll, Scroll::VIRTUALIZE_PROPERTY_KEY, false);
    f.settle();
    assert_eq!(f.realized(), (0..20).collect::<Vec<_>>());
    approx(layout_height(&f, "Content"), 200.0);
    boolean(&f.scroll, Scroll::VIRTUALIZE_PROPERTY_KEY, true);
    f.settle();
    approx(layout_height(&f, "Content"), 200.0);
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
    uint(&f.scroll, Scroll::VIRTUALIZE_BUFFER_PROPERTY_KEY, 1);
    f.settle();
    approx(layout_height(&f, "Content"), 200.0);
    assert_eq!(f.realized(), (0..16).collect::<Vec<_>>());
}
