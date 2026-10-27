//! Original 17 cases plus the seven alignment cases added by 7098a7c8.
//! Authority: Rive 4ac7b32798da0482e441ef09304dc3b480ed3ee5. The last case
//! covers invalidate_shape, a Nuxie addition.
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
        raw_text_input::{CursorBoundary, Flags, RawTextInput},
        text_engine::{FontRef, TextAlign, TextDirection, TextSizing, VerticalTextAlign},
    },
};
use std::path::PathBuf;

fn aligned_input(align: TextAlign, width: f32) -> RawTextInput {
    let mut input = RawTextInput::new();
    input.set_font(Some(load_font("assets/fonts/Inter_18pt-Regular.ttf")));
    input.set_font_size(72.0);
    input.insert("hello");
    input.set_cursor(Cursor::zero());
    input.set_align(align);
    input.set_align_width(width);
    input
}

#[test]
fn single_line_input_aligns_within_its_align_width() {
    let factory = retained_factory();
    let mut left = aligned_input(TextAlign::Left, 800.0);
    left.update(&factory);
    let width = left.bounds().width();
    assert!(width > 0.0 && width < 800.0);
    assert_eq!(
        left.cursor_visual_position_at(CursorPosition::zero()).x(),
        0.0
    );
    assert_eq!(left.bounds().min_x, 0.0);
    assert_approx(
        left.cursor_visual_position_at(CursorPosition::new(0, 5))
            .x(),
        width,
    );
    for (align, start) in [
        (TextAlign::Right, 800.0 - width),
        (TextAlign::Center, (800.0 - width) / 2.0),
    ] {
        let mut input = aligned_input(align, 800.0);
        input.update(&factory);
        assert_approx(
            input.cursor_visual_position_at(CursorPosition::zero()).x(),
            start,
        );
        assert_approx(
            input
                .cursor_visual_position_at(CursorPosition::new(0, 5))
                .x(),
            start + width,
        );
        assert_approx(input.bounds().min_x, start);
        assert_approx(input.bounds().max_x, start + width);
        assert_approx(input.bounds().width(), width);
    }
}

#[test]
fn alignment_falls_back_left_when_text_overflows() {
    let factory = retained_factory();
    for align in [TextAlign::Left, TextAlign::Right, TextAlign::Center] {
        let mut input = aligned_input(align, 10.0);
        input.update(&factory);
        assert_eq!(
            input.cursor_visual_position_at(CursorPosition::zero()).x(),
            0.0
        );
        assert_eq!(input.bounds().min_x, 0.0);
    }
    let mut input = aligned_input(TextAlign::Right, 0.0);
    input.update(&factory);
    assert_eq!(
        input.cursor_visual_position_at(CursorPosition::zero()).x(),
        0.0
    );
}

#[test]
fn hit_testing_round_trips_through_alignment() {
    let factory = retained_factory();
    for align in [TextAlign::Left, TextAlign::Right, TextAlign::Center] {
        let mut input = aligned_input(align, 800.0);
        input.update(&factory);
        for index in 0..=5 {
            let position = input.cursor_visual_position_at(CursorPosition::new(0, index));
            assert!(position.found());
            input.move_cursor_to(
                Vec2D::new(position.x(), (position.top() + position.bottom()) / 2.0),
                false,
            );
            assert_eq!(input.cursor().end().code_point_index(), index);
        }
    }
}

#[test]
fn multiline_alignment_offsets_each_line_independently() {
    let factory = retained_factory();
    let start = |align, index: usize| {
        let mut input = aligned_input(align, 0.0);
        input.set_text("hello\nlonger line here".into());
        input.set_sizing(TextSizing::AutoHeight);
        input.set_max_width(900.0);
        input.update(&factory);
        assert!(input.shape().line_count() as usize > index);
        input.shape().ordered_lines()[index].glyph_line().start_x
    };
    assert_eq!(start(TextAlign::Left, 0), 0.0);
    assert_eq!(start(TextAlign::Left, 1), 0.0);
    let right_short = start(TextAlign::Right, 0);
    let right_long = start(TextAlign::Right, 1);
    assert!(right_short > right_long && right_long > 0.0);
    let center_short = start(TextAlign::Center, 0);
    let center_long = start(TextAlign::Center, 1);
    assert!(center_short > center_long);
    assert_approx(center_short, right_short / 2.0);
    assert_approx(center_long, right_long / 2.0);
}

#[test]
fn input_aligns_vertically_within_its_align_height() {
    let factory = retained_factory();
    let mut top = aligned_input(TextAlign::Left, 0.0);
    top.set_align_height(500.0);
    top.update(&factory);
    let height = top.bounds().height();
    let min_y = top.bounds().min_y;
    let caret_y = top.cursor_visual_position_at(CursorPosition::zero()).top();
    assert!(height < 500.0);
    for (align, field_height, offset) in [
        (VerticalTextAlign::Top, 500.0, 0.0),
        (VerticalTextAlign::Middle, 500.0, (500.0 - height) / 2.0),
        (VerticalTextAlign::Bottom, 500.0, 500.0 - height),
        (VerticalTextAlign::Bottom, 10.0, 0.0),
    ] {
        let mut input = aligned_input(TextAlign::Left, 0.0);
        input.set_align_height(field_height);
        input.set_vertical_align(align);
        input.update(&factory);
        assert_approx(input.shape().vertical_offset(), offset);
        assert_approx(input.bounds().min_y, min_y + offset);
        assert_approx(input.bounds().max_y, min_y + offset + height);
        assert_approx(input.bounds().height(), height);
        assert_approx(
            input
                .cursor_visual_position_at(CursorPosition::zero())
                .top()
                - caret_y,
            offset,
        );
    }
}

#[test]
fn vertical_alignment_round_trips_through_hit_testing() {
    let factory = retained_factory();
    let mut input = aligned_input(TextAlign::Left, 0.0);
    input.set_align_height(500.0);
    input.set_vertical_align(VerticalTextAlign::Middle);
    input.update(&factory);
    let position = input.cursor_visual_position_at(CursorPosition::new(0, 3));
    assert!(position.found());
    input.move_cursor_to(
        Vec2D::new(position.x(), (position.top() + position.bottom()) / 2.0),
        false,
    );
    assert_eq!(input.cursor().end().code_point_index(), 3);
}

#[test]
fn multiline_alignment_uses_align_width_not_wrap_width() {
    let factory = retained_factory();
    let build = |align| {
        let mut input = aligned_input(align, 0.0);
        input.set_text("hello\nlonger line here".into());
        input.set_sizing(TextSizing::AutoHeight);
        input.set_max_width(900.0);
        input.update(&factory);
        input.set_max_width(input.bounds().width());
        input.set_align_width(900.0);
        input.update(&factory);
        input
    };
    let start = |input: &RawTextInput, index: usize| {
        input.shape().ordered_lines()[index].glyph_line().start_x
    };
    let left = build(TextAlign::Left);
    let width = left.bounds().width();
    assert_eq!(left.shape().line_count(), 2);
    assert!(width < 900.0);
    assert_eq!(start(&left, 0), 0.0);
    assert_eq!(start(&left, 1), 0.0);
    for (align, factor) in [(TextAlign::Right, 1.0), (TextAlign::Center, 0.5)] {
        let input = build(align);
        assert_approx(start(&input, 1), (900.0 - width) * factor);
        assert!(start(&input, 0) > start(&input, 1));
    }
}

// Alignment authority: upstream 7098a7c8. Expectations use the natural font
// metrics, then check the field-relative translation of bounds and caret.
#[test]
fn field_alignment_moves_bounds_and_caret_without_changing_intrinsic_size() {
    let factory = retained_factory();
    let mut input = RawTextInput::new();
    input.set_font(Some(load_font("assets/fonts/Inter_18pt-Regular.ttf")));
    input.set_font_size(24.0);
    input.insert("hello");
    input.update(&factory);
    let natural = input.bounds();
    let caret = input.cursor_visual_position();
    for (horizontal, factor_x) in [
        (TextAlign::Left, 0.0),
        (TextAlign::Center, 0.5),
        (TextAlign::Right, 1.0),
    ] {
        for (vertical, factor_y) in [
            (VerticalTextAlign::Top, 0.0),
            (VerticalTextAlign::Middle, 0.5),
            (VerticalTextAlign::Bottom, 1.0),
        ] {
            input.set_align(horizontal);
            input.set_align_width(500.0);
            input.set_vertical_align(vertical);
            input.set_align_height(200.0);
            input.update(&factory);
            let bounds = input.bounds();
            let position = input.cursor_visual_position();
            let dx = (500.0 - natural.width()) * factor_x;
            let dy = (200.0 - natural.height()) * factor_y;
            assert_approx(bounds.min_x, natural.min_x + dx);
            assert_approx(bounds.min_y, natural.min_y + dy);
            assert_approx(bounds.width(), natural.width());
            assert_approx(bounds.height(), natural.height());
            assert_approx(position.x(), caret.x() + dx);
            assert_approx(position.top(), caret.top() + dy);
            assert_eq!(input.measure(500.0, 200.0), bounds);
        }
    }
    input.set_align_width(1.0);
    input.set_align_height(1.0);
    input.update(&factory);
    assert_approx(input.bounds().min_x, natural.min_x);
    assert_approx(input.bounds().min_y, natural.min_y);
}

// Source-derived coverage for selectedText added by ed92313a; the upstream
// commit did not add tests. Selection offsets are Unicode code-point indices.
#[test]
fn selected_text_orders_and_clamps_code_point_selection_excluding_sentinel() {
    let mut input = RawTextInput::new();
    assert_eq!(input.selected_text(), "");
    input.set_text("aé🙂z".into());
    for (start, end, expected) in [
        (0, 0, ""),
        (1, 3, "é🙂"),
        (3, 1, "é🙂"),
        (0, u32::MAX, "aé🙂z"),
        (u32::MAX, 2, "🙂z"),
        (4, 5, ""),
        (7, 9, ""),
    ] {
        input.set_cursor(Cursor::new(
            CursorPosition::unresolved(start),
            CursorPosition::unresolved(end),
        ));
        assert_eq!(input.selected_text(), expected, "selection {start}..{end}");
    }
    assert_eq!(input.text(), "aé🙂z");
    input.set_text(String::new());
    input.set_cursor(Cursor::new(
        CursorPosition::zero(),
        CursorPosition::unresolved(10),
    ));
    assert_eq!(input.selected_text(), "");
}

#[test]
fn selected_text_preserves_direct_zero_code_point_but_string_edits_stop_at_nul() {
    let mut input = RawTextInput::new();
    input.set_text("a\0ignored".into());
    assert_eq!(input.text(), "a");
    input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(1)));
    input.insert("é\0ignored");
    assert_eq!(input.text(), "aé");
    input.insert_code_point(0);
    input.insert("🙂");
    assert_eq!(input.text(), "aé\0🙂");
    input.set_cursor(Cursor::new(
        CursorPosition::unresolved(1),
        CursorPosition::unresolved(4),
    ));
    assert_eq!(input.selected_text().as_bytes(), "é\0🙂".as_bytes());
}

fn retained_factory() -> RuntimeFactoryHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory")
}

// rive-runtime bec99be4: mask at shaping/measurement, never in the editable buffer.
#[test]
fn obscured_text_shapes_and_measures_bullets_without_replacing_source() {
    let factory = retained_factory();
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let mut input = RawTextInput::new();
    input.set_font(Some(font.clone()));
    let mut bullets = RawTextInput::new();
    bullets.set_font(Some(font));
    for value in ["hunter2", "日本語", "a\u{301}🙂", "", "a b\nc"] {
        input.set_text(value.into());
        input.set_obscured(true);
        bullets.set_text("•".repeat(value.chars().count()));
        input.update(&factory);
        bullets.update(&factory);
        assert_eq!(input.text(), value);
        assert_eq!(input.measure(300.0, 200.0), bullets.measure(300.0, 200.0));
        assert_eq!(input.bounds(), bullets.bounds());
        let glyphs = |input: &RawTextInput| {
            input
                .shape()
                .paragraphs()
                .iter()
                .flat_map(|paragraph| {
                    paragraph
                        .runs
                        .iter()
                        .flat_map(|run| run.glyphs.iter().copied())
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            glyphs(&input),
            glyphs(&bullets),
            "rendered glyphs must be bullets, including newlines"
        );
        input.select_all();
        assert_eq!(
            input.selected_text(),
            value,
            "raw layer retains actual text"
        );
        input.set_obscured(false);
        assert_eq!(input.text(), value);
        assert_cursor(input.cursor(), 0, value.chars().count() as u32);
    }
}

#[test]
fn obscured_toggle_preserves_edit_history_and_invalidates_cursor_lines() {
    let factory = retained_factory();
    let mut input = RawTextInput::new();
    input.set_font(Some(load_font("assets/fonts/Inter_18pt-Regular.ttf")));
    input.set_font_size(32.0);
    input.insert("ab\ncd");
    input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(4)));
    input.update(&factory);
    assert_eq!(input.cursor().end().line_index(), 1);
    input.insert("!");
    input.update(&factory);
    assert_eq!(input.text(), "ab\nc!d");
    input.set_obscured(true);
    assert_eq!(input.cursor().end().line_index(), u32::MAX);
    input.update(&factory);
    input.undo();
    input.update(&factory);
    assert_eq!(input.text(), "ab\ncd");
    assert_eq!(input.cursor().end().line_index(), 0);
    assert!(input.cursor_visual_position().found());
    input.redo();
    assert_eq!(input.text(), "ab\nc!d");
    assert_eq!(input.cursor().end().line_index(), u32::MAX);
    input.set_obscured(false);
    assert_eq!(input.text(), "ab\nc!d");
}

#[test]
fn obscured_word_navigation_does_not_disclose_internal_spaces() {
    let mut input = RawTextInput::new();
    input.set_font(Some(load_font("assets/fonts/Inter_18pt-Regular.ttf")));
    input.set_font_size(32.0);
    input.insert("ab cd");
    input.set_obscured(true);
    input.update(&retained_factory());
    input.set_cursor(Cursor::collapsed(CursorPosition::new(0, 3)));
    input.select_word();
    assert_eq!(input.selected_text(), "ab cd");
    input.set_cursor(Cursor::zero());
    input.cursor_right(CursorBoundary::Word, false);
    assert_eq!(input.cursor().end().code_point_index(), 5);
}

// Direct bec99be4 case: newlines mask too, and edits preserve the original text.
#[test]
fn obscured_input_shapes_as_bullets_over_intact_text() {
    let mut input = RawTextInput::new();
    input.set_font(Some(load_font("assets/fonts/Inter_18pt-Regular.ttf")));
    input.set_font_size(32.0);
    input.insert("ab\ncd");
    let factory = retained_factory();
    input.update(&factory);
    assert_eq!(input.shape().paragraphs().len(), 2);
    input.set_obscured(true);
    assert!(input.obscured());
    input.update(&factory);
    assert_eq!(input.shape().paragraphs().len(), 1);
    let run = &input.shape().paragraphs()[0].runs[0];
    assert!(run.glyphs.len() >= 5);
    for index in 1..5 {
        assert_eq!(run.glyphs[index], run.glyphs[0]);
    }
    assert_eq!(input.text(), "ab\ncd");
    input.insert("!");
    assert_eq!(input.text(), "ab\ncd!");
    input.backspace(-1);
    input.backspace(-1);
    assert_eq!(input.text(), "ab\nc");
    input.set_obscured(false);
    input.update(&factory);
    assert_eq!(input.shape().paragraphs().len(), 2);
}

#[test]
fn obscured_toggle_re_resolves_cursor_lines() {
    let mut input = RawTextInput::new();
    input.set_font(Some(load_font("assets/fonts/Inter_18pt-Regular.ttf")));
    input.set_font_size(32.0);
    input.insert("ab\ncd");
    let factory = retained_factory();
    input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(4)));
    input.update(&factory);
    assert_eq!(input.cursor().end().line_index(), 1);
    assert!(input.cursor_visual_position().found());
    input.set_obscured(true);
    input.update(&factory);
    assert_eq!(input.cursor().end().line_index(), 0);
    assert_eq!(input.cursor().end().code_point_index(), 4);
    assert!(input.cursor_visual_position().found());
    input.set_obscured(false);
    input.update(&factory);
    assert_eq!(input.cursor().end().line_index(), 1);
    assert!(input.cursor_visual_position().found());
}

// Nuxie addition: a host whose font fallbacks changed asks for shaping again
// without changing the font or the text.
#[test]
fn invalidate_shape_reshapes_unchanged_text_on_the_next_update() {
    let factory = retained_factory();
    let reshaped = Flags::ShapeDirty | Flags::SelectionDirty;
    let mut input = RawTextInput::new();
    input.set_font(Some(load_font("assets/fonts/Inter_18pt-Regular.ttf")));
    input.set_font_size(72.0);
    input.insert("hello");
    assert_eq!(input.update(&factory) & reshaped, reshaped);
    input.measure(800.0, 800.0);
    let measured = input.measure_count;
    assert_eq!(
        input.update(&factory) & reshaped,
        0,
        "clean text is not shaped again"
    );
    input.measure(800.0, 800.0);
    assert_eq!(input.measure_count, measured, "nor measured again");
    // The same font is no change, so only the explicit invalidation reshapes,
    // refreshes the selection and measures again.
    input.set_font(input.font());
    assert_eq!(input.update(&factory) & reshaped, 0);
    input.invalidate_shape();
    assert_eq!(input.update(&factory) & reshaped, reshaped);
    input.measure(800.0, 800.0);
    assert_eq!(input.measure_count, measured + 1);
    assert_eq!(input.update(&factory) & reshaped, 0);
    assert_eq!(input.text(), "hello");
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

// Upstream 074bfb13: moving left from a selection lands before its first character.
#[test]
fn moving_left_from_a_selection_lands_before_its_first_character() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");
    let mut text_input = RawTextInput::new();
    text_input.insert("hello world");
    text_input.set_font(Some(font));
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(500.0);
    text_input.set_font_size(72.0);
    let factory = retained_factory();
    text_input.update(&factory);

    // Forward selection collapses off its leading edge.
    text_input.set_cursor(Cursor::new(
        CursorPosition::new(0, 2),
        CursorPosition::new(0, 5),
    ));
    text_input.cursor_left(CursorBoundary::Character, false);
    assert_cursor(text_input.cursor(), 1, 1);

    // Backward selection has the same leading edge.
    text_input.set_cursor(Cursor::new(
        CursorPosition::new(0, 5),
        CursorPosition::new(0, 2),
    ));
    text_input.cursor_left(CursorBoundary::Character, false);
    assert_cursor(text_input.cursor(), 1, 1);

    // The beginning clamps the leftward move.
    text_input.set_cursor(Cursor::new(
        CursorPosition::new(0, 0),
        CursorPosition::new(0, 5),
    ));
    text_input.cursor_left(CursorBoundary::Character, false);
    assert_cursor(text_input.cursor(), 0, 0);

    // Word movement also starts from the leading edge.
    text_input.set_cursor(Cursor::new(
        CursorPosition::new(0, 6),
        CursorPosition::new(0, 9),
    ));
    text_input.cursor_left(CursorBoundary::Word, false);
    assert_cursor(text_input.cursor(), 0, 0);

    // Shift+left extends from the active end.
    text_input.set_cursor(Cursor::new(
        CursorPosition::new(0, 2),
        CursorPosition::new(0, 5),
    ));
    text_input.cursor_left(CursorBoundary::Character, true);
    assert_cursor(text_input.cursor(), 2, 4);

    // Rightward movement still collapses off the active end.
    text_input.set_cursor(Cursor::new(
        CursorPosition::new(0, 2),
        CursorPosition::new(0, 5),
    ));
    text_input.cursor_right(CursorBoundary::Character, false);
    assert_cursor(text_input.cursor(), 6, 6);
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

thread_local! {
    static COLOR_INPUT_FALLBACK: std::cell::RefCell<Option<FontRef>> = const { std::cell::RefCell::new(None) };
}

fn with_color_input_fallback<R>(font: FontRef, work: impl FnOnce() -> R) -> R {
    use nuxie_runtime::source::text_engine::{Font, with_host_fallback_proc};
    fn pick(_: u32, index: u32, _: &dyn Font) -> Option<FontRef> {
        (index == 0)
            .then(|| COLOR_INPUT_FALLBACK.with(|font| font.borrow().clone()))
            .flatten()
    }
    struct Restore(Option<FontRef>);
    impl Drop for Restore {
        fn drop(&mut self) {
            COLOR_INPUT_FALLBACK.with(|font| *font.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(COLOR_INPUT_FALLBACK.with(|slot| slot.replace(Some(font))));
    with_host_fallback_proc(pick, work)
}

// The pre-extension outline result, including clockwise contour normalization.
// Comparing its vertices by f32 bits catches any change to Latin or mono emoji.
fn legacy_input_outline(input: &RawTextInput, excluded_text_indices: &[u32]) -> Vec<u32> {
    use nuxie_runtime::source::{
        math::mat2d::Mat2D, shapes::paint::shape_paint_path::ShapePaintPath,
    };
    let mut path = ShapePaintPath::default();
    let shape = input.shape();
    let line = &shape.paragraph_lines()[0][0];
    let mut x = line.start_x;
    for (run, index) in &shape.ordered_lines()[0] {
        let index = index as usize;
        let font = run.font.as_ref().unwrap();
        // The fixture's explicit emoji text index is independent of production
        // color classification. A broken classification must change this result.
        if !excluded_text_indices.contains(&run.text_indices[index]) {
            let mut glyph = font.get_path(run.glyphs[index]);
            glyph.transform_in_place(Mat2D::new(
                run.size,
                0.0,
                0.0,
                run.size,
                x + run.offsets[index].x,
                shape.vertical_offset() + line.baseline + run.offsets[index].y,
            ));
            path.add_path_clockwise(&glyph, None);
        }
        x += run.advances[index];
    }
    outline_bits(path.raw_path())
}

fn outline_bits(path: &nuxie_runtime::source::math::raw_path::RawPath) -> Vec<u32> {
    path.verbs_u8()
        .iter()
        .map(|verb| *verb as u32)
        .chain(
            path.points()
                .iter()
                .flat_map(|point| [point.x.to_bits(), point.y.to_bits()]),
        )
        .collect()
}

fn draw_input_recording(input: &mut RawTextInput) -> String {
    use nuxie_runtime::source::math::mat2d::Mat2D;
    let recording = RecordingFactory::new();
    let mut renderer = recording.make_renderer();
    let mut factory = PersistentFactory::new(recording);
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    input.update(&retained);
    let mut text = retained.with_factory_mut(|factory| factory.make_render_paint());
    text.color(0xff123456);
    let mut selection = retained.with_factory_mut(|factory| factory.make_render_paint());
    let mut cursor = retained.with_factory_mut(|factory| factory.make_render_paint());
    input.draw(
        &retained,
        &mut renderer,
        &Mat2D::default(),
        text.as_mut(),
        selection.as_mut(),
        cursor.as_mut(),
    );
    factory.borrow().stream()
}

fn assert_input_color_layers_follow_outline(stream: &str, has_selection: bool) -> Vec<String> {
    let draws: Vec<_> = stream
        .lines()
        .filter(|line| line.starts_with("drawPath "))
        .collect();
    let outline_index = usize::from(has_selection);
    assert!(draws.len() > outline_index + 2, "{stream}");
    assert!(
        draws[outline_index].contains("color=0xff123456,"),
        "{stream}"
    );
    // The selection precedes text, and the caret follows all emoji layers.
    // Their default paint is distinct from the deliberately colored outline.
    if has_selection {
        assert!(!draws[0].contains("color=0xff123456,"), "{stream}");
    }
    let colors: Vec<_> = draws[outline_index + 1..draws.len() - 1]
        .iter()
        .map(|line| {
            line.split("color=")
                .nth(1)
                .unwrap()
                .split(',')
                .next()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert!(
        colors
            .iter()
            .any(|color| { !matches!(color.as_str(), "0x00000000" | "0xff000000" | "0xff123456") }),
        "emoji layers must retain their native colors: {stream}"
    );
    assert!(
        !draws.last().unwrap().contains("color=0xff123456,"),
        "{stream}"
    );
    colors
}

#[test]
fn input_fallback_color_emoji_draws_layers_and_preserves_latin_outline_bits() {
    let emoji = load_font("assets/NotoColorEmojiCOLRv1.subset.ttf");
    with_color_input_fallback(emoji, || {
        let mut input = RawTextInput::new();
        input.set_font(Some(load_font("assets/RobotoFlex.ttf")));
        input.set_text("A😀B".into());
        let stream = draw_input_recording(&mut input);
        assert_eq!(
            outline_bits(input.text_path().raw_path()),
            legacy_input_outline(&input, &[1])
        );
        assert!(!input.text_path().empty());
        let normal_colors = assert_input_color_layers_follow_outline(&stream, false);

        input.set_separate_selection_text(true);
        input.set_cursor(Cursor::new(
            CursorPosition::unresolved(1),
            CursorPosition::unresolved(2),
        ));
        let selected_stream = draw_input_recording(&mut input);
        assert!(
            input.selected_text_path().empty(),
            "selected emoji must retain its own colors"
        );
        assert_eq!(
            outline_bits(input.text_path().raw_path()),
            legacy_input_outline(&input, &[1])
        );
        assert_eq!(
            normal_colors,
            assert_input_color_layers_follow_outline(&selected_stream, true),
            "selection must not recolor the emoji"
        );
    });
}

#[test]
fn input_outline_only_emoji_is_byte_for_byte_legacy_outline() {
    let font = HbFont::decode(include_bytes!("fixtures/harfbuzz/Zycon.ttf")).unwrap();
    assert!(!font.has_color_glyphs());
    let mut input = RawTextInput::new();
    input.set_font(Some(font));
    input.set_text("🌝".into());
    let stream = draw_input_recording(&mut input);
    let (run, glyph) = (&input.shape().ordered_lines()[0])
        .into_iter()
        .next()
        .unwrap();
    assert_ne!(
        run.glyphs[glyph as usize], 0,
        "fixture must map the emoji, not .notdef"
    );
    assert!(!input.text_path().empty());
    assert_eq!(
        outline_bits(input.text_path().raw_path()),
        legacy_input_outline(&input, &[])
    );
    assert_eq!(
        stream
            .lines()
            .filter(|line| line.starts_with("drawPath "))
            .count(),
        2
    );
}

#[test]
fn input_host_picture_suppresses_cluster_followers_and_reuses_image_after_selection_changes() {
    use nuxie_runtime::source::{
        math::mat2d::Mat2D,
        text::font_hb::{HostGlyphPicture, HostPictureScope},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let callback_calls = calls.clone();
    let plain = load_font("assets/RobotoFlex.ttf");
    let host = plain
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(
            Arc::new(move |_, cluster| {
                if cluster != [65, 0xfe0f] {
                    return None;
                }
                callback_calls.fetch_add(1, Ordering::Relaxed);
                Some(HostGlyphPicture {
                    png: vec![
                        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1,
                        0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84,
                        120, 156, 99, 248, 15, 4, 0, 9, 251, 3, 253, 251, 94, 107, 43, 0, 0, 0, 0,
                        73, 69, 78, 68, 174, 66, 96, 130,
                    ],
                    width: 1,
                    height: 1,
                    bearing_x: 0.125,
                    bearing_y: -0.875,
                    extent_x: 1.0,
                    extent_y: 1.0,
                })
            }),
            HostPictureScope::Cluster,
        );
    let host_lifetime = Arc::downgrade(&host);
    let recording = RecordingFactory::new();
    let mut renderer = recording.make_renderer();
    let mut factory = PersistentFactory::new(recording);
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let mut input = RawTextInput::new();
    input.set_font(Some(host));
    input.set_text("A\u{fe0f}A\u{fe0f}".into());
    input.set_separate_selection_text(true);
    input.update(&retained);
    assert_eq!(
        input.shape().ordered_lines()[0]
            .into_iter()
            .filter(|(run, index)| run.text_indices[*index as usize] < 4)
            .count(),
        4,
        "each A+VS16 cluster must include its default-ignorable space glyph"
    );
    assert!(input.text_path().empty());
    let mut text = retained.with_factory_mut(|factory| factory.make_render_paint());
    let mut selection = retained.with_factory_mut(|factory| factory.make_render_paint());
    let mut cursor = retained.with_factory_mut(|factory| factory.make_render_paint());
    for selected in [false, true] {
        if selected {
            input.set_cursor(Cursor::new(
                CursorPosition::unresolved(0),
                CursorPosition::unresolved(4),
            ));
            input.update(&retained);
            assert!(input.selected_text_path().empty());
        }
        input.draw(
            &retained,
            &mut renderer,
            &Mat2D::default(),
            text.as_mut(),
            selection.as_mut(),
            cursor.as_mut(),
        );
    }
    input.set_font(Some(plain));
    input.update(&retained);
    assert!(
        host_lifetime.upgrade().is_none(),
        "font replacement must evict obsolete cached images and fonts"
    );
    let stream = factory.borrow().stream();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(
        stream
            .lines()
            .filter(|line| line.starts_with("decodeImage "))
            .count(),
        1,
        "{stream}"
    );
    assert_eq!(
        stream
            .lines()
            .filter(|line| line.starts_with("drawImage "))
            .count(),
        4,
        "{stream}"
    );
    assert!(
        stream.contains("transform matrix=[1,0,0,1,0.125,-0.875]"),
        "{stream}"
    );
}

// Complete 17-case max-limits and empty-line delta from upstream 8ea42e0e.

#[test]
fn maxcharacters_caps_typing_and_truncates_pastes() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_max_characters(5);
    assert!(text_input.max_characters() == 5);

    assert!(text_input.insert("abc"));
    assert!(text_input.text() == "abc");

    // A paste keeps only what fits.
    assert!(text_input.insert("defgh"));
    assert!(text_input.text() == "abcde");
    assert!(text_input.cursor().end().code_point_index() == 5);

    // Full: nothing more goes in.
    assert!(!text_input.insert_code_point('x' as u32));
    assert!(!text_input.insert("xyz"));
    assert!(text_input.text() == "abcde");

    // Deleting makes room again.
    text_input.update(&factory);
    text_input.backspace(-1);
    assert!(text_input.text() == "abcd");
    assert!(text_input.insert_code_point('x' as u32));
    assert!(text_input.text() == "abcdx");

    // Replacing a selection counts the selected code points as free.
    text_input.set_cursor(Cursor::new(
        CursorPosition::unresolved(0),
        CursorPosition::unresolved(2),
    ));
    assert!(text_input.insert("XYZ"));
    assert!(text_input.text() == "XYcdx");

    // Code points, not bytes: each of these is several UTF-8 bytes.
    let mut unicode = RawTextInput::new();
    unicode.set_max_characters(3);
    assert!(unicode.insert("é€🙂é"));
    assert!(unicode.length() == 3);
    assert!(unicode.text() == "é€🙂");
}

#[test]
fn maxcharacters_of_0_is_unlimited() {
    let mut text_input = RawTextInput::new();
    assert!(text_input.max_characters() == 0);
    assert!(text_input.max_lines() == 0);
    let long_text = "a".repeat(500);
    assert!(text_input.insert(&long_text));
    assert!(text_input.length() == 500);
}

#[test]
fn a_rejected_insert_leaves_selection_and_undo_history_alone() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_max_characters(3);
    assert!(text_input.insert("abc"));
    text_input.set_cursor(Cursor::new(
        CursorPosition::unresolved(1),
        CursorPosition::unresolved(1),
    ));
    assert!(!text_input.insert_code_point('x' as u32));
    assert!(text_input.cursor().start().code_point_index() == 1);
    assert!(text_input.cursor().end().code_point_index() == 1);

    // The rejected insert added no journal entry: one undo takes back the
    // "abc" insert.
    text_input.undo();
    assert!(text_input.text() == "");
    text_input.redo();
    assert!(text_input.text() == "abc");
}

#[test]
fn text_set_from_outside_is_kept_even_over_the_limits() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(10000.0);
    text_input.set_max_characters(3);
    text_input.set_max_lines(1);

    text_input.set_text("abcdef\nghi".into());
    assert!(text_input.text() == "abcdef\nghi");
    text_input.set_text_preserve_cursor("abcdef\nghij".into());
    assert!(text_input.text() == "abcdef\nghij");

    // Still over, so a selection replaced by something just as long is
    // refused...
    text_input.set_cursor(Cursor::new(
        CursorPosition::unresolved(0),
        CursorPosition::unresolved(1),
    ));
    assert!(!text_input.insert_code_point('x' as u32));
    assert!(text_input.text() == "abcdef\nghij");

    // ...while deleting always works.
    text_input.update(&factory);
    text_input.backspace(-1);
    assert!(text_input.text() == "bcdef\nghij");
}

#[test]
fn maxlines_caps_line_breaks_and_truncates_pastes() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(10000.0);
    text_input.set_max_lines(2);
    assert!(text_input.max_lines() == 2);

    // A trailing newline puts the cursor on a second (empty) line.
    assert!(text_input.insert("one\n"));
    assert!(text_input.text() == "one\n");
    assert!(!text_input.insert("\n"));
    assert!(text_input.text() == "one\n");

    // The paste stops before the line break that would start a third line.
    assert!(text_input.insert("two\nthree\nfour"));
    assert!(text_input.text() == "one\ntwo");
    text_input.update(&factory);
    assert!(text_input.shape().line_count() == 2);

    // More on the same line still goes in.
    assert!(text_input.insert("!"));
    assert!(text_input.text() == "one\ntwo!");
}

#[test]
fn maxlines_counts_wrapped_lines() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let word_width;
    {
        let mut probe = RawTextInput::new();
        probe.set_font(Some(font.clone()));
        probe.set_font_size(32.0);
        probe.insert("word");
        probe.update(&factory);
        word_width = probe.bounds().width();
    }
    assert!(word_width > 0.0);

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    // Room for one word per line, never two.
    text_input.set_max_width(word_width * 1.5);

    // Without a limit the paste wraps onto four lines.
    let paste = "word word word word";
    assert!(text_input.insert(paste));
    text_input.update(&factory);
    assert!(text_input.shape().line_count() == 4);

    text_input.set_text("".into());
    text_input.set_max_lines(2);
    assert!(text_input.insert(paste));
    let kept = text_input.text();
    assert!(kept.len() < paste.len());
    assert!(kept.starts_with("word word"));
    text_input.update(&factory);
    assert!(text_input.shape().line_count() <= 2);

    // A line break on the last allowed line is refused.
    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(0)));
    assert!(!text_input.insert("\n"));
    assert!(text_input.text() == kept);
}

#[test]
fn maxlines_wraps_against_the_space_the_layout_offers_not_the_hug() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();
    let available = 900.0;

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_hugs_width(true);
    text_input.insert("123\n123\n123");

    // Lay out the way TextInput does: measure against the space available,
    // then wrap at the width the text actually took (it hugs its text).
    let layout = |text_input: &mut RawTextInput| {
        let measured = text_input.measure(available, f32::MAX);
        text_input.set_max_width(measured.width());
        text_input.update(&factory);
    };
    layout(&mut text_input);
    assert!(text_input.shape().line_count() == 3);
    text_input.set_max_lines(3);

    // Widening the widest line just grows the field, it doesn't wrap.
    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(3)));
    assert!(text_input.insert_code_point('4' as u32));
    assert!(text_input.text() == "1234\n123\n123");
    layout(&mut text_input);
    assert!(text_input.shape().line_count() == 3);

    // Running past the space available does wrap, so it's held back.
    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(4)));
    assert!(text_input.insert(
        " and a much longer run of words that cannot possibly fit in the space that is available"
    ));
    layout(&mut text_input);
    assert!(text_input.shape().line_count() == 3);
    assert!(text_input.text().starts_with("1234 and a"));
    assert!(
      text_input.text().len() <
      "1234 and a much longer run of words that cannot possibly fit in the space that is available\n123\n123"
          .len());
}

#[test]
fn the_cursor_lands_on_an_empty_line_not_the_line_after_it() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(900.0);
    text_input.insert("123\n\n123");
    text_input.update(&factory);
    assert!(text_input.shape().line_count() == 3);

    // The empty middle line is just the line break at index 4; index 5 is
    // already the start of the last line.
    let middle = text_input.cursor_visual_position_at(CursorPosition::new(1, 4));
    assert!(middle.found());
    let middle_y = (middle.top() + middle.bottom()) / 2.0;
    for x in [0.0, 60.0] {
        text_input.move_cursor_to(Vec2D::new(x, middle_y), false);
        assert!(text_input.cursor().end().code_point_index() == 4);
        assert!(text_input.cursor().end().line_index() == 1);
    }

    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(8)));
    text_input.update(&factory);
    text_input.cursor_up(false);
    assert!(text_input.cursor().end().code_point_index() == 4);

    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(0)));
    text_input.update(&factory);
    text_input.cursor_down(false);
    assert!(text_input.cursor().end().code_point_index() == 4);

    assert!(text_input.insert_code_point('x' as u32));
    assert!(text_input.text() == "123\nx\n123");
}

#[test]
fn enter_that_leaves_an_empty_line_keeps_the_caret_with_the_text() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    // Enter leaves an empty line behind the caret.
    for (text, enter_at, line_after) in [("123", 0, 1), ("123\n123", 4, 2)] {
        let mut text_input = RawTextInput::new();
        text_input.set_font(Some(font.clone()));
        text_input.set_font_size(32.0);
        text_input.set_sizing(TextSizing::AutoHeight);
        text_input.set_max_width(900.0);
        text_input.insert(text);
        text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(enter_at)));
        text_input.update(&factory);

        assert!(text_input.insert("\n"));
        text_input.update(&factory);
        assert!(text_input.cursor().end().code_point_index() == enter_at + 1);
        assert!(text_input.cursor().end().line_index() == line_after);

        // Where it draws is where it types.
        let drawn = text_input.cursor_visual_position();
        let line =
            text_input.cursor_visual_position_at(CursorPosition::new(line_after, enter_at + 1));
        assert!(drawn.found());
        assert!(drawn.top() == line.top());

        // Home and End stay on the empty line rather than running into the
        // next one.
        text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(enter_at)));
        text_input.update(&factory);
        assert!(text_input.cursor().end().line_index() == line_after - 1);
        text_input.cursor_right(CursorBoundary::Line, false);
        assert!(text_input.cursor().end().code_point_index() == enter_at);
    }
}

#[test]
fn maxlines_lets_a_full_field_s_empty_line_be_typed_into() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_hugs_width(true);
    text_input.insert("123\n\n123");
    let measured = text_input.measure(900.0, f32::MAX);
    text_input.set_max_width(measured.width());
    text_input.update(&factory);
    text_input.set_max_lines(3);

    text_input.cursor_up(false);
    assert!(text_input.insert_code_point('x' as u32));
    assert!(text_input.text() == "123\nx\n123");
}

#[test]
fn maxlines_keeps_the_longest_paste_that_fits_when_wrapping_isn_t_monotonic() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();
    // Wide enough for "aa cc", "cbbbb" and "xbbbb", too narrow for "ccbbbb" and
    // "xxbbbb", which then break mid-word onto a third line.
    let width = 100.0;
    let lines = |text: &str| {
        let mut probe = RawTextInput::new();
        probe.set_font(Some(font.clone()));
        probe.set_font_size(32.0);
        probe.set_sizing(TextSizing::AutoHeight);
        probe.set_max_width(width);
        probe.insert(text);
        probe.update(&factory);
        probe.shape().line_count()
    };
    // Pasting "cc xx" before "bbbb": a prefix ending mid-word joins "bbbb", so
    // the line counts for prefixes 1 to 5 go 2, 3, 2, 2, 3.
    assert!(lines("aa cbbbb") == 2);
    assert!(lines("aa ccbbbb") == 3);
    assert!(lines("aa cc bbbb") == 2);
    assert!(lines("aa cc xbbbb") == 2);
    assert!(lines("aa cc xxbbbb") == 3);

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(width);
    text_input.insert("aa bbbb");
    text_input.set_max_lines(2);
    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(3)));
    assert!(text_input.insert("cc xx"));
    assert!(text_input.text() == "aa cc xbbbb");
}

#[test]
fn maxlines_wraps_at_maxwidth_unless_the_layout_hugs_the_text() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let word_width;
    {
        let mut probe = RawTextInput::new();
        probe.set_font(Some(font.clone()));
        probe.set_font_size(32.0);
        probe.insert("word");
        probe.update(&factory);
        word_width = probe.bounds().width();
    }

    for hugs in [false, true] {
        let mut text_input = RawTextInput::new();
        text_input.set_font(Some(font.clone()));
        text_input.set_font_size(32.0);
        text_input.set_sizing(TextSizing::AutoHeight);
        text_input.set_hugs_width(hugs);
        // Measured with plenty of room, then narrowed to one word per line.
        text_input.measure(10000.0, f32::MAX);
        text_input.set_max_width(word_width * 1.5);
        text_input.set_max_lines(2);

        assert!(text_input.insert("word word word word"));
        if hugs {
            // A hugging field grows to the measured width, where it all fits.
            assert!(text_input.text() == "word word word word");
        } else {
            // A field held at maxWidth keeps what fits on its two lines there.
            text_input.update(&factory);
            assert!(text_input.shape().line_count() <= 2);
            assert!(text_input.text().starts_with("word word"));
            assert!(text_input.text().find("word word word").is_none());
        }
    }
}

#[test]
fn maxlines_refuses_a_large_paste_into_a_full_field_quickly() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(900.0);
    // 999 line breaks make 1000 lines: full.
    let full = "\n".repeat(999);
    text_input.insert(&full);
    text_input.set_max_lines(1000);
    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(0)));
    text_input.limit_shape_count = 0;

    assert!(!text_input.insert(&"\n".repeat(1000)));
    assert!(text_input.text() == full);
    // The paragraphs after the insertion point stay in the bound, so the
    // search never counts down through the paste.
    assert!(text_input.limit_shape_count < 20);
}

#[test]
fn maxlines_keeps_a_large_paste_into_a_long_paragraph_within_the_limit() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");
    let factory = retained_factory();

    let mut words = String::new();
    for _ in 0..200 {
        words.push_str("word ");
    }

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(300.0);
    text_input.insert(&words);
    text_input.update(&factory);
    // One more line than the paragraph takes, then paste at its start: the
    // rest of the paragraph spans far more than the count down's budget.
    let lines = text_input.shape().line_count();
    text_input.set_max_lines(lines + 1);
    text_input.set_cursor(Cursor::collapsed(CursorPosition::unresolved(0)));
    text_input.limit_shape_count = 0;

    assert!(text_input.insert(&words));
    text_input.update(&factory);
    assert!(text_input.shape().line_count() <= lines + 1);
    assert!(text_input.text().len() > words.len());
    assert!(text_input.limit_shape_count < 100);
}

#[test]
fn an_empty_right_to_left_line_holds_its_own_break() {
    let font = load_font("assets/fonts/IBMPlexSansArabic-Regular.ttf");
    let factory = retained_factory();

    // Two line separators inside one right to left run: the middle line is
    // empty and its break is the separator at index 2. Its iterator begins at
    // the glyph before, the separator ending the line above.
    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(900.0);
    text_input.insert("ب\u{2028}\u{2028}ب");
    text_input.update(&factory);
    let shape = text_input.shape();
    assert!(shape.line_count() == 3);
    let middle = &shape.ordered_lines()[1];
    assert!(middle.is_empty());
    assert!(middle.last_run().dir() == TextDirection::Rtl);

    assert!(middle.first_code_point_index(shape.glyph_lookup()) == 2);
    assert!(middle.last_code_point_index(shape.glyph_lookup()) == 2);
    for x in [0.0, 60.0] {
        assert!(CursorPosition::from_line_x(1, x, &shape.layout_view()).code_point_index() == 2);
    }

    // Its line boundaries are where the cursor already is.
    text_input.set_cursor(Cursor::collapsed(CursorPosition::new(1, 2)));
    text_input.cursor_left(CursorBoundary::Line, false);
    assert!(text_input.cursor().end().code_point_index() == 2);
    text_input.cursor_right(CursorBoundary::Line, false);
    assert!(text_input.cursor().end().code_point_index() == 2);
}

#[test]
fn maxlines_counts_the_lines_an_obscured_input_displays() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(10000.0);
    text_input.set_max_lines(1);

    assert!(text_input.insert("a\nb"));
    assert!(text_input.text() == "a");

    // Obscured, the line break displays as a bullet on the same line.
    text_input.set_text("".into());
    text_input.set_obscured(true);
    assert!(text_input.insert("a\nb"));
    assert!(text_input.text() == "a\nb");
}

#[test]
fn the_maxlines_check_leaves_measure_correct() {
    let font = load_font("assets/fonts/Inter_18pt-Regular.ttf");

    let mut text_input = RawTextInput::new();
    text_input.set_font(Some(font.clone()));
    text_input.set_font_size(32.0);
    text_input.set_sizing(TextSizing::AutoHeight);
    text_input.set_max_width(10000.0);
    text_input.insert("hello");
    let before = text_input.measure(10000.0, 10000.0);

    // A rejected insert shapes its candidate in the measuring shape.
    text_input.set_max_lines(1);
    assert!(!text_input.insert("\nsecond line that is much wider"));
    let after = text_input.measure(10000.0, 10000.0);
    assert!(after.width() == before.width());
    assert!(after.height() == before.height());
}

#[test]
fn text_input_decodes_a_cut_short_sequence_inside_its_text() {
    let mut input = RawTextInput::new();
    input.set_text_bytes(b"a\xe2b");
    assert_eq!(input.length(), 3);
    input.insert_bytes(&[0xf0]);
    assert_eq!(input.length(), 4);
    assert_eq!(input.text(), "\u{fffd}a\u{fffd}b");
}
