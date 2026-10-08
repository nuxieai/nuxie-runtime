//! runtime/layout_corner_radius_test.cpp at 955d6a05.
#[path = "support/layout_955.rs"]
mod support;
use support::*;
#[test]
fn a_corner_radius_change_rebuilds_only_its_layout_path() {
    let f = Fixture::new("layout/layout_anim_bound.riv");
    f.advance(0.0);
    let layout = f.layout(LayoutAnimationStyle::Inherit);
    assert!(
        layout
            .with(|o| o.as_layout_component().unwrap().actual_direction() != LayoutDirection::Rtl)
            .unwrap()
    );
    let s = style(&layout);
    boolean(&s, Style::LINK_CORNER_RADIUS_PROPERTY_KEY, true);
    number(&s, Style::CORNER_RADIUS_TL_PROPERTY_KEY, 12.0);
    assert!(
        !f.artboard
            .core_handle()
            .with(|o| o
                .as_component()
                .unwrap()
                .dirt()
                .contains(nuxie_runtime::source::component_dirt::ComponentDirt::LAYOUT_STYLE))
            .unwrap()
    );
    f.advance(0.0);
    assert!(local(&layout) == rounded_rect(&layout, [12.0; 4]));
    number(&s, Style::CORNER_RADIUS_TR_PROPERTY_KEY, 4.0);
    number(&s, Style::CORNER_RADIUS_BR_PROPERTY_KEY, 8.0);
    number(&s, Style::CORNER_RADIUS_BL_PROPERTY_KEY, 16.0);
    boolean(&s, Style::LINK_CORNER_RADIUS_PROPERTY_KEY, false);
    f.advance(0.0);
    assert!(local(&layout) == rounded_rect(&layout, [12.0, 4.0, 8.0, 16.0]));
}
#[test]
fn inherited_layout_tween_runs_while_corner_radius_changes() {
    let f = Fixture::new("layout/layout_anim_bound.riv");
    f.advance(0.0);
    let container = f.layout(LayoutAnimationStyle::Custom);
    let layout = f.layout(LayoutAnimationStyle::Inherit);
    linear_tween(&container);
    f.advance(0.0);
    assert_eq!(
        layout
            .with(|o| o.as_layout_component().unwrap().interpolation_time())
            .unwrap(),
        1.0
    );
    assert_eq!(width(&layout), 100.0);
    number(&layout, LayoutComponentBase::WIDTH_PROPERTY_KEY, 50.0);
    let mut mid_width = 100.0;
    for i in 0..5 {
        number(
            &style(&layout),
            Style::CORNER_RADIUS_TL_PROPERTY_KEY,
            (i + 1) as f32,
        );
        f.advance(0.1);
        let w = width(&layout);
        if w > 50.0 && w < 100.0 {
            mid_width = w;
        }
    }
    assert!(mid_width < 100.0);
    assert!(mid_width > 50.0);
}
