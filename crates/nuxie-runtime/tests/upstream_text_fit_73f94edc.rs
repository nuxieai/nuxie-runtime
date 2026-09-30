//! tests/unit_tests/runtime/text_test.cpp additions from upstream 73f94edc.
//! Also contains the 7db8b61f Text word-break case, sharing the ellipsis harness.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    core::CoreHandle,
    generated::{
        core_registry::CoreRegistry,
        text::{text_base::TextBase, text_style_base::TextStyleBase},
    },
    text::{text::Text, text_style::TextStyle},
    text_engine::{TextOverflow, TextWordBreak},
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle, RuntimeFileHandle};

fn fixture(name: &str) -> (RuntimeFileHandle, CoreHandle, CoreHandle) {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR pinned upstream checkout"),
    );
    let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(name)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(&bytes, retained, None, None, None).unwrap();
    let artboard = file.with_file(|file| file.artboard()).unwrap();
    let texts = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<Text>())
        .unwrap();
    assert_eq!(texts.len(), 1);
    (file, artboard, texts[0].clone())
}
fn scalar(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(
        owner,
        i32::from(key),
        value
    ));
}
fn integer(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, i32::from(key), value));
}
fn overflow(text: &CoreHandle, value: TextOverflow) {
    let value = match value {
        TextOverflow::Visible => 0,
        TextOverflow::FitFontSize => 5,
        _ => unreachable!("test only uses visible and fitFontSize"),
    };
    integer(text, TextBase::OVERFLOW_VALUE_PROPERTY_KEY, value);
}
fn advance(artboard: &CoreHandle) {
    Artboard::advance_handle(
        artboard,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}

#[test]
fn word_break_reaches_the_line_breaker_from_the_text_object() {
    let (_file, artboard, text) = fixture("ellipsis.riv");
    overflow(&text, TextOverflow::Visible);
    integer(&text, TextBase::SIZING_VALUE_PROPERTY_KEY, 2); // TextSizing::fixed
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, 30.0);
    assert_eq!(
        text.with_downcast::<Text, _>(Text::word_break).unwrap(),
        TextWordBreak::BreakWord
    );
    let line_count = || {
        text.with_downcast::<Text, _>(|text| text.ordered_lines().len())
            .unwrap()
    };
    advance(&artboard);
    let break_word_lines = line_count();
    integer(&text, TextBase::WORD_BREAK_VALUE_PROPERTY_KEY, 1); // normal
    advance(&artboard);
    let normal_lines = line_count();
    integer(&text, TextBase::WORD_BREAK_VALUE_PROPERTY_KEY, 2); // breakAll
    advance(&artboard);
    let break_all_lines = line_count();
    assert!(normal_lines < break_all_lines);
    assert!(break_all_lines < break_word_lines);
    integer(&text, TextBase::WORD_BREAK_VALUE_PROPERTY_KEY, 0); // breakWord
    advance(&artboard);
    assert_eq!(line_count(), break_word_lines);
}
fn first_run(text: &CoreHandle) -> (f32, f32, f32) {
    text.with_downcast::<Text, _>(|text| {
        assert!(!text.shape().is_empty());
        assert!(!text.shape()[0].runs.is_empty());
        let run = &text.shape()[0].runs[0];
        (run.size, run.line_height, run.letter_spacing)
    })
    .unwrap()
}
fn approx(actual: f32, expected: f32) {
    // Pinned Catch Approx promotes operands to double; its relative margin
    // depends only on the expected value (default scale and fixed margin 0).
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    let magnitude = if expected.is_infinite() {
        0.0
    } else {
        expected.abs()
    };
    let margin = f64::from(f32::EPSILON * 100.0) * magnitude;
    assert!(
        actual == expected || (actual + margin >= expected && expected + margin >= actual),
        "{actual} != Approx({expected})"
    );
}

// Shapes ellipsis.riv at font_size in a width x height fitFontSize box.
fn fit_run_size(font_size: f32, width: f32, height: f32) -> f32 {
    let (_file, artboard, text) = fixture("ellipsis.riv");
    let styles = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<TextStyle>())
        .unwrap();
    for style in styles {
        scalar(&style, TextStyleBase::FONT_SIZE_PROPERTY_KEY, font_size);
    }
    integer(&text, TextBase::SIZING_VALUE_PROPERTY_KEY, 2);
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, width);
    scalar(&text, TextBase::HEIGHT_PROPERTY_KEY, height);
    overflow(&text, TextOverflow::FitFontSize);
    advance(&artboard);
    let mut renderer = nuxie_render_api::NullRenderer;
    Artboard::draw_handle(&artboard, &mut renderer);
    first_run(&text).0
}

#[test]
fn fit_font_size_keeps_a_non_finite_font_size() {
    assert!(fit_run_size(f32::INFINITY, 300.0, 60.0).is_infinite());
    assert!(fit_run_size(f32::NAN, 300.0, 60.0).is_nan());
}

#[test]
fn fit_font_size_caps_the_search_at_2_to_24() {
    approx(fit_run_size(3e9, 1e38, 1e38), (1 << 24) as f32);
    let fitted = fit_run_size(3e9, 300.0, 60.0);
    assert!(fitted < 60.0);
    approx(fitted, fit_run_size(100.0, 300.0, 60.0));
}

#[test]
fn fit_font_size_scales_custom_line_height_and_letter_spacing_proportionally() {
    let (_file, artboard, text) = fixture("ellipsis.riv");
    let styles = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<TextStyle>())
        .unwrap();
    assert!(!styles.is_empty());
    for style in styles {
        scalar(&style, TextStyleBase::LINE_HEIGHT_PROPERTY_KEY, 40.0);
        scalar(&style, TextStyleBase::LETTER_SPACING_PROPERTY_KEY, 3.0);
    }
    overflow(&text, TextOverflow::Visible);
    advance(&artboard);
    let (authored, line_height, letter_spacing) = first_run(&text);
    assert!(authored > 1.0);
    approx(line_height, 40.0);
    approx(letter_spacing, 3.0);
    overflow(&text, TextOverflow::FitFontSize);
    advance(&artboard);
    let (fitted, line_height, letter_spacing) = first_run(&text);
    assert!(fitted < authored);
    let scale = fitted / authored;
    approx(line_height, 40.0 * scale);
    approx(letter_spacing, 3.0 * scale);
}

#[test]
fn fit_font_size_scales_paragraph_spacing_proportionally() {
    let (_file, artboard, text) = fixture("double_line.riv");
    scalar(&text, TextBase::PARAGRAPH_SPACING_PROPERTY_KEY, 30.0);
    integer(
        &text,
        TextBase::SIZING_VALUE_PROPERTY_KEY,
        2, // TextSizing::fixed
    );
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, 2000.0);
    scalar(&text, TextBase::HEIGHT_PROPERTY_KEY, 2000.0);
    overflow(&text, TextOverflow::Visible);
    advance(&artboard);
    assert!(
        text.with_downcast::<Text, _>(|text| text.shape().len() >= 2)
            .unwrap()
    );
    let authored = first_run(&text).0;
    assert!(authored > 1.0);
    let ys = |text: &CoreHandle| {
        text.with_downcast::<Text, _>(|text| {
            text.ordered_lines()
                .iter()
                .map(|line| line.y())
                .collect::<Vec<_>>()
        })
        .unwrap()
    };
    let authored_y = ys(&text);
    assert!(authored_y.len() >= 2);
    scalar(&text, TextBase::HEIGHT_PROPERTY_KEY, 40.0);
    overflow(&text, TextOverflow::FitFontSize);
    advance(&artboard);
    let fitted = first_run(&text).0;
    assert!(fitted < authored);
    let fitted_y = ys(&text);
    assert_eq!(fitted_y.len(), authored_y.len());
    let scale = fitted / authored;
    for (actual, original) in fitted_y.iter().zip(&authored_y) {
        approx(*actual, *original * scale);
    }
    let authored_span = authored_y.last().unwrap() - authored_y.first().unwrap();
    let fitted_span = fitted_y.last().unwrap() - fitted_y.first().unwrap();
    assert!(authored_span > 0.0);
    assert!(fitted_span < authored_span);
    approx(fitted_span, authored_span * scale);
}

#[test]
fn changing_paragraph_spacing_reshapes_fit_font_size_text() {
    let (_file, artboard, text) = fixture("double_line.riv");
    integer(
        &text,
        TextBase::SIZING_VALUE_PROPERTY_KEY,
        2, // TextSizing::fixed
    );
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, 2000.0);
    scalar(&text, TextBase::HEIGHT_PROPERTY_KEY, 150.0);
    overflow(&text, TextOverflow::FitFontSize);
    scalar(&text, TextBase::PARAGRAPH_SPACING_PROPERTY_KEY, 0.0);
    advance(&artboard);
    let size_without_gap = first_run(&text).0;
    assert!(size_without_gap > 1.0);
    scalar(&text, TextBase::PARAGRAPH_SPACING_PROPERTY_KEY, 60.0);
    advance(&artboard);
    let size_with_gap = first_run(&text).0;
    assert!(size_with_gap < size_without_gap);
}
