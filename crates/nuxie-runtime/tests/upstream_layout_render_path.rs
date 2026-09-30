//! runtime/layout_render_path_test.cpp at 955d6a05.
#[path = "support/layout_955.rs"]
mod support;
use support::*;
#[test]
fn moving_layout_keeps_local_render_path() {
    for radius in [0.0, 12.0, 50.0] {
        let f = Fixture::new("layout/layout_anim_bound.riv");
        let container = f.layout(LayoutAnimationStyle::Custom);
        let layout = f.layout(LayoutAnimationStyle::Inherit);
        boolean(
            &style(&layout),
            Style::LINK_CORNER_RADIUS_PROPERTY_KEY,
            true,
        );
        number(
            &style(&layout),
            Style::CORNER_RADIUS_TL_PROPERTY_KEY,
            radius,
        );
        let mut renderer = nuxie_render_api::NullRenderer;
        f.advance(0.0);
        f.artboard.draw(&mut renderer);
        assert!(has_render_path(&layout));
        let before = world_bounds(&layout);
        let s = style(&container);
        let padding =
            CoreRegistry::get_double_handle(&s, Style::PADDING_LEFT_PROPERTY_KEY.into()).unwrap();
        number(&s, Style::PADDING_LEFT_PROPERTY_KEY, padding + 10.0);
        f.advance(0.0);
        assert!(has_render_path(&layout));
        let after = world_bounds(&layout);
        assert_ne!(after.left(), before.left());
        assert!(
            (f64::from(after.width()) - f64::from(before.width())).abs()
                <= 100.0 * f64::from(f32::EPSILON) * f64::from(before.width()).abs()
        );
    }
}
#[test]
fn tweening_layout_local_path_follows_size() {
    let f = Fixture::new("layout/layout_anim_bound.riv");
    f.advance(0.0);
    let container = f.layout(LayoutAnimationStyle::Custom);
    let layout = f.layout(LayoutAnimationStyle::Inherit);
    linear_tween(&container);
    f.advance(0.0);
    assert_eq!(width(&layout), 100.0);
    let mut renderer = nuxie_render_api::NullRenderer;
    number(&layout, LayoutComponentBase::WIDTH_PROPERTY_KEY, 50.0);
    let mut saw_mid_tween = false;
    for _ in 0..12 {
        f.advance(0.1);
        f.artboard.draw(&mut renderer);
        let w = width(&layout);
        saw_mid_tween |= w > 50.0 && w < 100.0;
        let h = layout
            .with(|o| o.as_layout_component().unwrap().layout_height())
            .unwrap();
        assert!(local(&layout).bounds() == Aabb::from_ltwh(0.0, 0.0, w, h));
    }
    assert!(saw_mid_tween);
    assert_eq!(width(&layout), 50.0);
}
