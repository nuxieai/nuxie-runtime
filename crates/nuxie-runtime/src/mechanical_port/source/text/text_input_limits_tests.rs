//! Public text_input_test.cpp additions at 8ea42e0e.
//! The legacy Yoga hug-encoding case is excluded by our CSS/Taffy contract.
use super::{
    TextInput,
    cpp_probe_text_test_support::{fixture, property_key, with_input},
};
use crate::source::{
    core::{CoreArena, CoreHandle},
    generated::core_registry::CoreRegistry,
    input::focusable::{Key, KeyModifiers},
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    layout_component::{LayoutComponent, content_sizing_layout},
    math::vec2d::Vec2D,
    text::cursor::{Cursor, CursorPosition},
};

fn set_text(input: &CoreHandle, text: &str) {
    assert!(CoreRegistry::set_string_handle(input, 817, text.into()));
}
fn limit(input: &CoreHandle, key: i32, value: u32) {
    assert!(CoreRegistry::set_uint_handle(input, key, value));
}
fn sizing_layout(input: &CoreHandle) -> CoreHandle {
    content_sizing_layout(with_input(input, |i| i.base.parent_handle())).unwrap()
}

#[test]
fn max_limits_reach_raw_text_input() {
    let (_file, artboard, input) = fixture();
    with_input(&input, |i| {
        assert_eq!(i.base.max_characters(), 0);
        assert_eq!(i.base.max_lines(), 0);
        assert_eq!(i.raw_text_input().max_characters(), 0);
        assert_eq!(i.raw_text_input().max_lines(), 0);
    });
    set_text(&input, "");
    limit(&input, 489, 4);
    limit(&input, 490, 2);
    with_input(&input, |i| {
        assert_eq!(i.raw_text_input().max_characters(), 4);
        assert_eq!(i.raw_text_input().max_lines(), 2);
    });
    artboard.advance_default(0.0);
    with_input(&input, |i| {
        assert!(i.text_input("hello"));
        assert_eq!(i.base.text(), "hell");
        assert!(i.text_input("!"));
        assert_eq!(i.base.text(), "hell");
    });
    limit(&input, 489, 0);
    with_input(&input, |i| {
        i.raw_text_input()
            .set_cursor(Cursor::collapsed(CursorPosition::unresolved(4)));
        assert!(i.key_input(Key::ENTER, KeyModifiers::NONE, true, false));
        assert_eq!(i.base.text(), "hell\n");
        assert!(i.key_input(Key::ENTER, KeyModifiers::NONE, true, false));
        assert_eq!(i.base.text(), "hell\n");
        assert_eq!(i.raw_text_input().text(), "hell\n");
    });
    limit(&input, 489, 2);
    set_text(&input, "much longer\nthan\nthe limits");
    artboard.advance_default(0.0);
    with_input(&input, |i| {
        assert_eq!(i.raw_text_input().text(), "much longer\nthan\nthe limits")
    });
}

#[test]
fn hugs_width_only_in_hugging_layout() {
    let (_file, artboard, input) = fixture();
    artboard.advance_default(0.0);
    with_input(&input, |i| {
        assert!(!i.raw_text_input().hugs_width());
        i.control_size(
            Vec2D::new(120.0, 40.0),
            LayoutScaleType::Hug,
            LayoutScaleType::Hug,
            LayoutDirection::Ltr,
        );
        assert!(i.raw_text_input().hugs_width());
        i.control_size(
            Vec2D::new(120.0, 40.0),
            LayoutScaleType::Fill,
            LayoutScaleType::Hug,
            LayoutDirection::Ltr,
        );
        assert!(!i.raw_text_input().hugs_width());
    });
}

#[test]
fn refused_keystroke_preserves_single_line_source() {
    let (_file, artboard, input) = fixture();
    assert!(CoreRegistry::set_bool_handle(&input, 979, false));
    set_text(&input, "a\nb");
    limit(&input, 489, 3);
    artboard.advance_default(0.0);
    with_input(&input, |i| {
        assert_eq!(i.raw_text_input().text(), "a b");
        assert!(i.text_input("x"));
        assert_eq!(i.raw_text_input().text(), "a b");
        assert_eq!(i.base.text(), "a\nb");
    });
}

#[test]
fn max_lines_lets_full_field_lines_grow_wider() {
    let (_file, artboard, input) = fixture();
    with_input(&input, |i| assert!(i.base.multiline()));
    set_text(&input, "123\n123\n123");
    limit(&input, 490, 3);
    artboard.advance_default(0.0);
    artboard.advance_default(0.0);
    with_input(&input, |i| {
        assert_eq!(i.raw_text_input().shape().line_count(), 3);
        i.raw_text_input()
            .set_cursor(Cursor::collapsed(CursorPosition::unresolved(3)));
        assert!(i.text_input("4"));
        assert_eq!(i.base.text(), "1234\n123\n123");
    });
    artboard.advance_default(0.0);
    with_input(&input, |i| {
        assert_eq!(i.raw_text_input().shape().line_count(), 3)
    });
}

#[test]
fn css_owner_uses_current_width_despite_stored_rive_hug() {
    let (_file, artboard, input) = fixture();
    artboard.advance_default(0.0);
    let layout = sizing_layout(&input);
    let style = layout
        .with_downcast::<LayoutComponent, _>(LayoutComponent::style_handle)
        .unwrap()
        .unwrap();
    assert!(CoreRegistry::set_uint_handle(
        &style,
        property_key("LayoutComponentStyle", "layoutWidthScaleType"),
        LayoutScaleType::Hug as u32
    ));
    let arena = CoreArena::default();
    let css = arena.insert(crate::css_layout::record::LayoutCssStyle::default());
    layout
        .with_downcast_mut::<LayoutComponent, _>(|layout| layout.add_layout_style_applier(css))
        .unwrap();
    with_input(&input, |i| {
        i.measure_layout(
            1000.0,
            LayoutMeasureMode::AtMost,
            40.0,
            LayoutMeasureMode::AtMost,
        );
        i.control_size(
            Vec2D::new(120.0, 40.0),
            LayoutScaleType::Hug,
            LayoutScaleType::Hug,
            LayoutDirection::Ltr,
        );
        assert!(!i.raw_text_input().hugs_width());
        assert_eq!(i.raw_text_input().max_width(), 120.0);
    });
}
