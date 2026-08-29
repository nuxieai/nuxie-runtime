//! All 17 cases from pinned tests/unit_tests/runtime/raw_text_input_test.cpp.
//! Authority: Rive 4ac7b32798da0482e441ef09304dc3b480ed3ee5.
//!
//! RawTextInput's upstream TESTING measure counter is exposed by the native
//! tools feature. Text shaping, cursor editing, and history use the live owners.
#![cfg(feature = "tools")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    factory::RuntimeFactoryHandle,
    math::{aabb::Aabb, vec2d::Vec2D},
    text::{
        cursor::{Cursor, CursorPosition},
        font_hb::HbFont,
        raw_text_input::{CursorBoundary, RawTextInput},
        text_engine::{FontRef, TextDirection, TextSizing},
    },
};
use std::path::PathBuf;

fn retained_factory() -> RuntimeFactoryHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory")
}

fn load_font(relative_path: &str) -> FontRef {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests")
        .join(relative_path);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned font {}: {error}", path.display()));
    HbFont::decode(&bytes).expect("decode pinned font with native HbFont")
}

// Pinned Catch Approx's defaults: float epsilon * 100, scale zero, margin zero.
// Match its addition-based margin comparison and promotion of float operands to
// double; this helper is used only where the C++ test explicitly uses Approx.
#[track_caller]
fn assert_approx(actual: f32, expected: f32) {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    let scale = if expected.is_infinite() {
        0.0
    } else {
        expected.abs()
    };
    let margin = f64::from(f32::EPSILON) * 100.0 * scale;
    assert!(
        (expected >= actual && actual >= expected)
            || (expected + margin >= actual && actual + margin >= expected),
        "actual {actual:?}, expected Catch Approx({expected:?}), margin {margin:?}"
    );
}

// The upstream CHECK_CURSOR observes the two actual code-point indices.
#[track_caller]
fn assert_cursor(cursor: Cursor, start: u32, end: u32) {
    assert_eq!(cursor.start().code_point_index(), start);
    assert_eq!(cursor.end().code_point_index(), end);
}

// cursor operators work
#[test]
fn cursor_operators_work() {
    let a = CursorPosition::new(0, 1);
    let b = CursorPosition::new(0, 4);
    let c = CursorPosition::new(0, 4);
    assert!(a < b);
    assert!(b > a);
    assert!(c == b);
    assert!(c != a);

    let mut d = CursorPosition::new(0, 1);
    d -= 1;
    assert!(d.code_point_index() == 0);
    d -= 1;
    // Still at 0, no overflow.
    assert!(d.code_point_index() == 0);
}

// cursor's visual position computes correctly
#[test]
fn cursor_s_visual_position_computes_correctly() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    let default_text = "this is some\nmultiline text input\nwith one final line\n";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    let mut position = text_input.cursor_visual_position_at(CursorPosition::zero());

    assert!(position.found());
    assert!(position.x() == 0.0_f32);
    assert!(position.top() == 0.0_f32);
    assert_approx(position.bottom(), 87.11719_f32);

    position = text_input.cursor_visual_position_at(CursorPosition::new(0, 1));

    assert!(position.found());
    assert_approx(position.x(), 23.30859_f32);
    assert!(position.top() == 0.0_f32);
    assert_approx(position.bottom(), 87.11719_f32);

    position = text_input.cursor_visual_position_at(CursorPosition::new(0, 2));

    assert!(position.found());
    assert_approx(position.x(), 65.17969_f32);
    assert!(position.top() == 0.0_f32);
    assert_approx(position.bottom(), 87.11719_f32);

    // When we're passed the last character on the line we should still show the
    // caret on that same line.
    position = text_input.cursor_visual_position_at(CursorPosition::new(0, 12));

    assert!(position.found());
    assert_approx(position.x(), 396.0_f32);
    assert!(position.top() == 0.0_f32);
    assert_approx(position.bottom(), 87.11719_f32);
}

// cursor is placed correctly with ltr paragraphs
#[test]
fn cursor_is_placed_correctly_with_ltr_paragraphs() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");

    let max_width = 500.0;
    let mut text_input = RawTextInput::new();
    let default_text = "one two three four five";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(max_width);
    text_input.set_font_size(72.0_f32);

    assert_eq!(text_input.bounds(), Aabb::default());
    let factory = retained_factory();
    text_input.update(&factory);

    assert_eq!(
        text_input.bounds(),
        Aabb::new(0.0, 0.0, 446.51953_f32, 216.0)
    );
    assert_eq!(
        text_input.measure(500.0, 400.0),
        Aabb::new(0.0, 0.0, 446.51953_f32, 216.0)
    );
    assert!(text_input.measure_count == 1);
    // measure count should still be one if we re-measured with same sizes.
    assert_eq!(
        text_input.measure(500.0, 400.0),
        Aabb::new(0.0, 0.0, 446.51953_f32, 216.0)
    );
    assert!(text_input.measure_count == 1);
    assert_eq!(
        text_input.measure(400.0, 400.0),
        Aabb::new(0.0, 0.0, 318.97266_f32, 324.0)
    );
    assert!(text_input.measure_count == 2);
    text_input.set_text("one two three four five six".to_owned());
    assert_eq!(
        text_input.measure(400.0, 400.0),
        Aabb::new(0.0, 0.0, 318.97266_f32, 324.0)
    );
    assert!(text_input.measure_count == 3);
    text_input.set_text("one two three four five".to_owned());

    assert!(text_input.shape().paragraphs().len() == 1);
    let paragraph = &text_input.shape().paragraphs()[0];
    assert!(paragraph.base_direction() == TextDirection::Ltr);
    assert!(text_input.shape().ordered_lines().len() == 2);

    // Ensure that clicking beyond the bounds of each line places the cursor at
    // the begginging/end of the line.
    let second_line_y = text_input.shape().ordered_lines()[1].y();

    assert!(text_input.cursor().start().code_point_index() == 0);
    // Click to the left of the whole line of text.
    text_input.move_cursor_to(Vec2D::new(-20.0_f32, second_line_y), false);

    assert!(
        text_input.cursor().start().code_point_index()
            == text_input.shape().ordered_lines()[1]
                .first_code_point_index(text_input.shape().glyph_lookup())
    );

    // Click to the right of the whole line of text.
    text_input.move_cursor_to(Vec2D::new(max_width + 20.0_f32, second_line_y), false);
    assert!(
        text_input.cursor().start().code_point_index()
            == text_input.shape().ordered_lines()[1]
                .last_code_point_index(text_input.shape().glyph_lookup())
    );
}

// cursor is placed correctly with rtl paragraphs
#[test]
fn cursor_is_placed_correctly_with_rtl_paragraphs() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");

    let max_width = 500.0;
    let mut text_input = RawTextInput::new();
    let default_text = "اربك تكست هو اول موقع يسمح لزواره";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(max_width);
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    assert!(text_input.shape().paragraphs().len() == 1);
    let paragraph = &text_input.shape().paragraphs()[0];
    assert!(paragraph.base_direction() == TextDirection::Rtl);
    assert!(text_input.shape().ordered_lines().len() == 3);

    // Ensure that clicking beyond the bounds of each line places the cursor at
    // the begginging/end of the line.
    let second_line_y = text_input.shape().ordered_lines()[1].y();

    assert!(text_input.cursor().start().code_point_index() == 0);
    // Click to the left of the whole line of text.
    text_input.move_cursor_to(Vec2D::new(-20.0_f32, second_line_y), false);

    assert!(
        text_input.cursor().start().code_point_index()
            == text_input.shape().ordered_lines()[1]
                .first_code_point_index(text_input.shape().glyph_lookup())
    );

    // Click to the right of the whole line of text.
    text_input.move_cursor_to(Vec2D::new(max_width + 20.0_f32, second_line_y), false);
    assert!(
        text_input.cursor().start().code_point_index()
            == text_input.shape().ordered_lines()[1]
                .last_code_point_index(text_input.shape().glyph_lookup())
    );
}

// cursor is placed correctly with mixed bidi paragraphs
#[test]
fn cursor_is_placed_correctly_with_mixed_bidi_paragraphs() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");

    let max_width = 500.0;
    let mut text_input = RawTextInput::new();
    let default_text = "one two three four اربك تكست هو اول موقع يسمح لزواره الكرام بتحويل";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(max_width);
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    assert!(text_input.shape().paragraphs().len() == 1);
    let paragraph = &text_input.shape().paragraphs()[0];
    assert!(paragraph.base_direction() == TextDirection::Ltr);
    assert!(text_input.shape().ordered_lines().len() == 5);

    // Ensure that clicking beyond the bounds of each line places the cursor at
    // the begginging/end of the line.
    let second_line_y = text_input.shape().ordered_lines()[1].y();

    assert!(text_input.cursor().start().code_point_index() == 0);
    // Click to the left of the whole line of text.
    text_input.move_cursor_to(Vec2D::new(-20.0_f32, second_line_y), false);

    assert!(
        text_input.cursor().start().code_point_index()
            == text_input.shape().ordered_lines()[1]
                .first_code_point_index(text_input.shape().glyph_lookup())
    );

    // Click to the right of the whole line of text.
    text_input.move_cursor_to(Vec2D::new(max_width + 20.0_f32, second_line_y), false);
    assert!(
        text_input.cursor().start().code_point_index()
            == text_input.shape().ordered_lines()[1]
                .last_code_point_index(text_input.shape().glyph_lookup())
    );
}

// cursor moves correctly
#[test]
fn cursor_moves_correctly() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    let default_text = "this is some\nmultiline text input\nwith one final line";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(0, 1));

    for _ in 0..12 {
        text_input.cursor_right(CursorBoundary::Character, false);
    }
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(1, 13));
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(1, 15));

    // Up once takes us to the previous line and the closest glyph's codepoint.
    text_input.cursor_up(false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(0, 4));

    // Up again goes to the start of the text.
    text_input.cursor_up(false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(0, 0));

    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(0, 3));

    text_input.cursor_down(false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(1, 14));

    // Next cursor down takes us to the closest codePoint on the last line.
    text_input.cursor_down(false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(2, 36));

    // Next cursor down should reach the end of the last line since we're
    // already on the last line.
    text_input.cursor_down(false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(2, 53));
}

// text inputs correctly
#[test]
fn text_inputs_correctly() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    let default_text = "hello ";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);
    assert!(text_input.text() == "hello ");
    // Quickly goes to end.
    text_input.cursor_down(false);
    text_input.insert("world");
    assert!(text_input.text() == "hello world");

    text_input.set_text("foo".to_owned());
    assert!(text_input.text() == "foo");
}

// cursor home/end works
#[test]
fn cursor_home_end_works() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");

    let max_width = 500.0;
    let mut text_input = RawTextInput::new();
    let default_text = "one two three four five";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(max_width);
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    assert!(text_input.shape().ordered_lines().len() == 2);
    text_input.cursor_down(false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(1, 14));

    text_input.cursor_right(CursorBoundary::Line, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(1, 23));

    text_input.cursor_left(CursorBoundary::Line, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start() == CursorPosition::new(1, 14));
}

// cursor word movement works
#[test]
fn cursor_word_movement_works() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");

    let max_width = 500.0;
    let mut text_input = RawTextInput::new();
    let default_text = "one two three fo4ur five";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(max_width);
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    // "|one two three fo4ur five"
    assert!(text_input.cursor().start().code_point_index() == 0);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "one| two three fo4ur five"
    assert!(text_input.cursor().start().code_point_index() == 3);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "one two| three fo4ur five"
    assert!(text_input.cursor().start().code_point_index() == 7);
    text_input.cursor_left(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "one |two three fo4ur five"
    assert!(text_input.cursor().start().code_point_index() == 4);
    text_input.cursor_left(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "|one two three fo4ur five"
    assert!(text_input.cursor().start().code_point_index() == 0);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "one two three fo4ur| five"
    assert!(text_input.cursor().start().code_point_index() == 19);
    text_input.cursor_left(CursorBoundary::Character, false);
    text_input.cursor_left(CursorBoundary::Character, false);
    text_input.update(&factory);
    // "one two three fo4|ur five"
    assert!(text_input.cursor().start().code_point_index() == 17);
    text_input.cursor_left(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "one two three |fo4ur five"
    assert!(text_input.cursor().start().code_point_index() == 14);
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    // "one two three fo|4ur five"
    assert!(text_input.cursor().start().code_point_index() == 16);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "one two three fo4ur| five"
    assert!(text_input.cursor().start().code_point_index() == 19);
}

// cursor sub-word movement works
#[test]
fn cursor_sub_word_movement_works() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");

    let max_width = 500.0;
    let mut text_input = RawTextInput::new();
    let default_text = "oneTwo threeFo+ur fi--ve";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(max_width);
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    // "|oneTwo threeFo+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 0);
    text_input.cursor_right(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "one|Two threeFo+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 3);
    text_input.cursor_right(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo| threeFo+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 6);
    text_input.cursor_right(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo three|Fo+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 12);
    text_input.cursor_right(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo threeFo|+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 14);
    text_input.cursor_right(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo threeFo+|ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 15);
    text_input.cursor_right(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo threeFo+ur| fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 17);
    text_input.cursor_left(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo threeFo+|ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 15);
    text_input.cursor_left(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo threeFo|+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 14);
    text_input.cursor_left(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo three|Fo+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 12);
    text_input.cursor_left(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo |threeFo+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 7);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "oneTwo threeFo|+ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 14);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "oneTwo threeFo+|ur fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 15);
    text_input.cursor_right(CursorBoundary::Word, false);
    text_input.update(&factory);
    // "oneTwo threeFo+ur| fi--ve"
    assert!(text_input.cursor().start().code_point_index() == 17);
    text_input.cursor_right(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo threeFo+ur fi|--ve"
    assert!(text_input.cursor().start().code_point_index() == 20);
    text_input.cursor_right(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo threeFo+ur fi--|ve"
    assert!(text_input.cursor().start().code_point_index() == 22);
    text_input.cursor_left(CursorBoundary::SubWord, false);
    text_input.update(&factory);
    // "oneTwo threeFo+ur fi|--ve"
    assert!(text_input.cursor().start().code_point_index() == 20);
}

// cursor skips multi-codepoint glyphs
#[test]
fn cursor_skips_multi_codepoint_glyphs() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    // "cafe\u0301s" = "cafés" where é = e + combining acute accent (2
    // codepoints, 1 glyph)
    // Indices: c=0 a=1 f=2 e=3 \u0301=4 s=5
    let mut default_text = "cafés"; // UTF-8 for café with precomposed é
    // Actually we need the decomposed form: e + combining acute accent
    // e = 0x65, combining acute = 0xCC 0x81 in UTF-8
    default_text = "cafe\u{301}s";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    // Move right: c(0) -> a(1) -> f(2) -> e(3) -> s(5) (should skip index 4)
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start().code_point_index() == 1);

    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start().code_point_index() == 2);

    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start().code_point_index() == 3);

    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    // Should skip index 4 (combining accent) and land on 5
    assert!(text_input.cursor().start().code_point_index() == 5);
}

// cursor left skips multi-codepoint glyphs
#[test]
fn cursor_left_skips_multi_codepoint_glyphs() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    let default_text = "cafe\u{301}s";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    // Go to end
    text_input.cursor_down(false);
    text_input.update(&factory);

    // Move left from end(6) -> s(5) -> é(3) (skip 4) -> f(2)
    text_input.cursor_left(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start().code_point_index() == 5);

    text_input.cursor_left(CursorBoundary::Character, false);
    text_input.update(&factory);
    // Should skip index 4 and land on 3
    assert!(text_input.cursor().start().code_point_index() == 3);

    text_input.cursor_left(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start().code_point_index() == 2);
}

// backspace deletes entire multi-codepoint glyph
#[test]
fn backspace_deletes_entire_multi_codepoint_glyph() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    let default_text = "cafe\u{301}s";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    // Move cursor to after the é (before 's')
    text_input.cursor_down(false);
    text_input.update(&factory);
    text_input.cursor_left(CursorBoundary::Character, false); // at 's' = index 5
    text_input.update(&factory);
    assert!(text_input.cursor().start().code_point_index() == 5);

    // Backspace should delete both codepoints of é (indices 3 and 4)
    text_input.backspace(-1);
    text_input.update(&factory);
    assert!(text_input.text() == "cafs");
    assert!(text_input.cursor().start().code_point_index() == 3);
}

// delete forward removes entire multi-codepoint glyph
#[test]
fn delete_forward_removes_entire_multi_codepoint_glyph() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    let default_text = "cafe\u{301}s";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    // Move cursor to before the é (after 'f') = index 3
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.update(&factory);
    assert!(text_input.cursor().start().code_point_index() == 3);

    // Delete forward should remove both codepoints of é
    text_input.backspace(1);
    text_input.update(&factory);
    assert!(text_input.text() == "cafs");
}

// word selection works
#[test]
fn word_selection_works() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");

    let max_width = 500.0;
    let mut text_input = RawTextInput::new();
    let default_text = "oneTwo three == four";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(max_width);
    text_input.set_font_size(72.0_f32);

    let factory = retained_factory();
    text_input.update(&factory);

    text_input.select_word();
    assert_cursor(text_input.cursor(), 0, 6);

    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(9)));
    text_input.select_word();
    assert_cursor(text_input.cursor(), 7, 12);

    // Right edge of word selects word before it ("three")
    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(12)));
    text_input.select_word();
    assert_cursor(text_input.cursor(), 7, 12);

    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(14)));
    text_input.select_word();
    assert_cursor(text_input.cursor(), 13, 15);
}

// text input journal works
#[test]
fn text_input_journal_works() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");

    let max_width = 500.0;
    let mut text_input = RawTextInput::new();
    let default_text = "oneTwo";
    text_input.insert(default_text);
    text_input.set_cursor(Cursor::zero());
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(max_width);
    text_input.set_font_size(72.0_f32);
    let factory = retained_factory();
    text_input.update(&factory);

    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.cursor_right(CursorBoundary::Character, false);
    text_input.cursor_right(CursorBoundary::Character, false);

    text_input.insert(" ");
    text_input.insert("2");
    text_input.insert(" ");
    text_input.update(&factory);
    assert!(text_input.text() == "one 2 Two");
    assert_cursor(text_input.cursor(), 6, 6);

    text_input.undo();
    assert!(text_input.text() == "one 2Two");
    assert_cursor(text_input.cursor(), 5, 5);

    text_input.undo();
    assert!(text_input.text() == "one Two");
    assert_cursor(text_input.cursor(), 4, 4);

    text_input.undo();
    assert!(text_input.text() == "oneTwo");
    assert_cursor(text_input.cursor(), 3, 3);

    text_input.redo();
    assert!(text_input.text() == "one Two");
    assert_cursor(text_input.cursor(), 4, 4);

    text_input.insert("X");
    assert!(text_input.text() == "one XTwo");
    assert_cursor(text_input.cursor(), 5, 5);

    // Redo does nothing as stack has been cleared by previous insertion
    text_input.redo();
    assert!(text_input.text() == "one XTwo");
    assert_cursor(text_input.cursor(), 5, 5);

    // Undo still works, however.
    text_input.undo();
    assert!(text_input.text() == "one Two");
    assert_cursor(text_input.cursor(), 4, 4);

    text_input.cursor_right(CursorBoundary::Character, true);
    text_input.cursor_right(CursorBoundary::Character, true);
    text_input.cursor_right(CursorBoundary::Character, true);
    assert!(text_input.text() == "one Two");
    assert_cursor(text_input.cursor(), 4, 7);
    text_input.insert("2");
    assert!(text_input.text() == "one 2");
    assert_cursor(text_input.cursor(), 5, 5);

    text_input.undo();
    assert!(text_input.text() == "one Two");
    assert_cursor(text_input.cursor(), 4, 7);
}

// clearSelection collapses to the selection end
#[test]
fn clear_selection_collapses_to_the_selection_end() {
    let mut text_input = RawTextInput::new();
    text_input.insert("hello world");

    text_input.set_cursor(Cursor::new(
        CursorPosition::unresolved(2),
        CursorPosition::unresolved(7),
    ));
    text_input.clear_selection();
    assert!(text_input.cursor().is_collapsed());
    assert_cursor(text_input.cursor(), 7, 7);

    // No-op when already collapsed.
    text_input.clear_selection();
    assert_cursor(text_input.cursor(), 7, 7);
}
