//! One-for-one expected-red ports of all 12 cases in pinned
//! `tests/unit_tests/runtime/line_break_test.cpp`.
//!
//! Rust shapes retained Text internally but has no standalone public owner for
//! upstream `Font::shapeText` plus `GlyphLine::BreakLines`. Fixtures and full
//! expected structures remain executable up to that missing owner.

use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
struct Font;

#[derive(Clone, Debug)]
struct TextRun {
    text: String,
    size: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Clone, Debug, Default)]
struct GlyphRun {
    breaks: Vec<usize>,
    glyphs: Vec<u32>,
    text_indices: Vec<usize>,
}

#[derive(Clone, Debug)]
struct Paragraph {
    runs: Vec<GlyphRun>,
    base_direction: TextDirection,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GlyphLine {
    start_run_index: usize,
    start_glyph_index: usize,
    end_run_index: usize,
    end_glyph_index: usize,
}

fn pinned_asset(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned font {}: {error}", path.display()))
}

fn load_font(_filename: &str) -> Font {
    // Preserve the upstream helper exactly: it ignores its filename argument
    // and always opens RobotoFlex.ttf, including the Arabic-named cases.
    let bytes = pinned_asset("RobotoFlex.ttf");
    decode_font(&bytes)
}

fn decode_font(_bytes: &[u8]) -> Font {
    Font
}

fn append(runs: &mut Vec<TextRun>, font: &Font, size: f32, text: &str) {
    let _ = font;
    runs.push(TextRun {
        text: text.to_owned(),
        size,
    });
}

fn shape_text(font: &Font, runs: &[TextRun]) -> Vec<Paragraph> {
    let _ = font;
    let _ = runs
        .iter()
        .map(|run| (run.text.chars().count(), run.size))
        .collect::<Vec<_>>();
    panic!("Rust has no standalone Font::shapeText owner")
}

fn break_lines(runs: &[GlyphRun], width: f32) -> Vec<GlyphLine> {
    let _ = (runs, width);
    panic!("Rust has no standalone GlyphLine::BreakLines owner")
}

fn assert_line(
    line: GlyphLine,
    start_run: usize,
    start_glyph: usize,
    end_run: usize,
    end_glyph: usize,
) {
    assert_eq!(line.start_run_index, start_run);
    assert_eq!(line.start_glyph_index, start_glyph);
    assert_eq!(line.end_run_index, end_run);
    assert_eq!(line.end_glyph_index, end_glyph);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_separates_words() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "one two three");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 1);
    let run = &paragraph.runs[0];
    assert_eq!(run.breaks.len(), 6);
    assert_eq!(run.breaks[0], 0);
    assert_eq!(run.breaks[1], 3);
    assert_eq!(run.breaks[2], 4);
    assert_eq!(run.breaks[3], 7);
    assert_eq!(run.breaks[4], 8);
    assert_eq!(run.breaks[5], 13);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_handles_multiple_runs() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "one two thr");
    append(&mut runs, &font, 60.0, "ee four");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 2);
    assert_eq!(paragraph.runs[0].breaks.len(), 5);
    assert_eq!(paragraph.runs[0].breaks, [0, 3, 4, 7, 8]);
    assert_eq!(paragraph.runs[1].breaks.len(), 3);
    assert_eq!(paragraph.runs[1].breaks, [2, 3, 7]);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_handles_returns() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "one two thr");
    append(&mut runs, &font, 60.0, "ee\u{2028} four");
    let paragraphs = shape_text(&font, &runs);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 2);
    assert_eq!(paragraph.runs[0].breaks.len(), 5);
    assert_eq!(paragraph.runs[0].breaks, [0, 3, 4, 7, 8]);
    assert_eq!(paragraph.runs[1].breaks.len(), 5);
    assert_eq!(paragraph.runs[1].breaks, [2, 2, 2, 4, 8]);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_builds_lines() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "one two three");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 1);

    let lines = break_lines(&paragraph.runs, 194.0);
    assert_eq!(lines.len(), 1);
    assert_line(lines[0], 0, 0, 0, 13);

    let lines = break_lines(&paragraph.runs, 191.0);
    assert_eq!(lines.len(), 2);
    assert_line(lines[0], 0, 0, 0, 7);
    assert_line(lines[1], 0, 8, 0, 13);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_deals_with_extremes() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "ab");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 1);
    for width in [17.0, 0.0] {
        let lines = break_lines(&paragraph.runs, width);
        assert_eq!(lines.len(), 2);
        assert_line(lines[0], 0, 0, 0, 1);
        assert_line(lines[1], 0, 1, 0, 2);
    }
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_breaks_return_characters() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "hello look\u{2028}here");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 1);
    assert_eq!(break_lines(&paragraph.runs, 300.0).len(), 2);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn shaper_separates_paragraphs() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    append(
        &mut runs,
        &font,
        32.0,
        "hello look\u{2028}here\nsecond paragraph",
    );
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(paragraphs[0].runs.len(), 1);
    assert_eq!(paragraphs[0].base_direction, TextDirection::LeftToRight);
    assert_eq!(break_lines(&paragraphs[0].runs, 300.0).len(), 2);
    assert_eq!(paragraphs[1].runs.len(), 1);
    assert_eq!(paragraphs[1].base_direction, TextDirection::LeftToRight);
    assert_eq!(break_lines(&paragraphs[1].runs, 300.0).len(), 1);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn shaper_handles_rtl() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    let text = "لمفاتيح ABC DEF";
    append(&mut runs, &font, 32.0, text);
    let unichars = text.chars().collect::<Vec<_>>();
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.base_direction, TextDirection::RightToLeft);
    assert_eq!(break_lines(&paragraph.runs, 300.0).len(), 1);
    let lines = break_lines(&paragraph.runs, 196.0);
    assert_eq!(lines.len(), 2);
    let line = lines[1];
    let run = &paragraph.runs[line.start_run_index];
    let index = run.text_indices[line.start_glyph_index];
    assert_eq!(unichars[index], 'D');
    assert_eq!(unichars[index + 1], 'E');
    assert_eq!(unichars[index + 2], 'F');
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn shaper_handles_empty_space() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, " ");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.base_direction, TextDirection::LeftToRight);
    assert_eq!(break_lines(&paragraph.runs, 300.0).len(), 1);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_deals_with_empty_paragraphs() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "hi\n ");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(paragraphs[0].base_direction, TextDirection::LeftToRight);
    assert_eq!(break_lines(&paragraphs[0].runs, -1.0).len(), 1);
    assert_eq!(paragraphs[1].base_direction, TextDirection::LeftToRight);
    let lines = break_lines(&paragraphs[1].runs, -1.0);
    assert_eq!(lines.len(), 1);
    let line = lines[0];
    assert_eq!(line.start_run_index, 0);
    assert_eq!(paragraphs[1].runs[line.start_run_index].glyphs.len(), 1);
    assert_eq!(
        paragraphs[1].runs[line.start_run_index].text_indices.len(),
        1
    );
    assert_eq!(paragraphs[1].runs[line.start_run_index].text_indices[0], 3);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_deals_with_space_only_lines() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "hi\u{2028} ");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.base_direction, TextDirection::LeftToRight);
    assert_eq!(break_lines(&paragraph.runs, -1.0).len(), 2);
}

#[test]
#[ignore = "expected-red: Rust has no standalone Font::shapeText owner"]
fn line_breaker_deals_with_empty_lines() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    append(&mut runs, &font, 32.0, "hi\n");
    let paragraphs = shape_text(&font, &runs);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.base_direction, TextDirection::LeftToRight);
    let lines = break_lines(&paragraph.runs, -1.0);
    assert_eq!(lines.len(), 1);
    let line = lines[0];
    assert_eq!(line.start_run_index, 0);
    assert_eq!(line.start_glyph_index, 0);
    assert_eq!(paragraph.runs[line.start_run_index].glyphs.len(), 3);
    assert_eq!(paragraph.runs[line.start_run_index].text_indices.len(), 3);
    assert_eq!(paragraph.runs[line.start_run_index].text_indices[0], 0);
}
