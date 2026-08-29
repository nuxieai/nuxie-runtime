//! Direct ports of all twelve pinned line_break_test.cpp cases.
//! Both shaping and line breaking run through translated production owners.

use nuxie_runtime::source::{
    text::font_hb::HbFont,
    text_engine::{FontRef, GlyphLine, TextDirection, TextRun},
};
use std::path::PathBuf;

fn load_font(_filename: &str) -> FontRef {
    // Preserve the upstream helper: it ignores its filename argument and
    // opens RobotoFlex.ttf, including the Arabic-named cases.
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/RobotoFlex.ttf");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned font {}: {error}", path.display()));
    HbFont::decode(&bytes).expect("pinned font decodes")
}

fn append(unichars: &mut Vec<u32>, runs: &mut Vec<TextRun>, font: &FontRef, size: f32, text: &str) {
    let start = unichars.len();
    unichars.extend(text.chars().map(u32::from));
    runs.push(TextRun {
        font: Some(font.clone()),
        size,
        line_height: -1.0,
        letter_spacing: 0.0,
        unichar_count: (unichars.len() - start) as u32,
        script: 0,
        style_id: 0,
        level: 0,
    });
}

fn assert_line(line: &GlyphLine, start_run: u32, start_glyph: u32, end_run: u32, end_glyph: u32) {
    assert_eq!(line.start_run_index, start_run);
    assert_eq!(line.start_glyph_index, start_glyph);
    assert_eq!(line.end_run_index, end_run);
    assert_eq!(line.end_glyph_index, end_glyph);
}

#[test]
fn line_breaker_separates_words() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, "one two three");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
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
fn line_breaker_handles_multiple_runs() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, "one two thr");
    append(&mut unichars, &mut runs, &font, 60.0, "ee four");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 2);
    assert_eq!(paragraph.runs[0].breaks.len(), 5);
    assert_eq!(paragraph.runs[0].breaks, [0, 3, 4, 7, 8]);
    assert_eq!(paragraph.runs[1].breaks.len(), 3);
    assert_eq!(paragraph.runs[1].breaks, [2, 3, 7]);
}

#[test]
fn line_breaker_handles_returns() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, "one two thr");
    append(&mut unichars, &mut runs, &font, 60.0, "ee\u{2028} four");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 2);
    assert_eq!(paragraph.runs[0].breaks.len(), 5);
    assert_eq!(paragraph.runs[0].breaks, [0, 3, 4, 7, 8]);
    assert_eq!(paragraph.runs[1].breaks.len(), 5);
    assert_eq!(paragraph.runs[1].breaks, [2, 2, 2, 4, 8]);
}

#[test]
fn line_breaker_builds_lines() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, "one two three");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 1);

    let lines = GlyphLine::break_lines(&paragraph.runs, 194.0);
    assert_eq!(lines.len(), 1);
    assert_line(&lines[0], 0, 0, 0, 13);

    let lines = GlyphLine::break_lines(&paragraph.runs, 191.0);
    assert_eq!(lines.len(), 2);
    assert_line(&lines[0], 0, 0, 0, 7);
    assert_line(&lines[1], 0, 8, 0, 13);
}

#[test]
fn line_breaker_deals_with_extremes() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, "ab");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 1);
    for width in [17.0, 0.0] {
        let lines = GlyphLine::break_lines(&paragraph.runs, width);
        assert_eq!(lines.len(), 2);
        assert_line(&lines[0], 0, 0, 0, 1);
        assert_line(&lines[1], 0, 1, 0, 2);
    }
}

#[test]
fn line_breaker_breaks_return_characters() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(
        &mut unichars,
        &mut runs,
        &font,
        32.0,
        "hello look\u{2028}here",
    );
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.runs.len(), 1);
    assert_eq!(GlyphLine::break_lines(&paragraph.runs, 300.0).len(), 2);
}

#[test]
fn shaper_separates_paragraphs() {
    let font = load_font("RobotoFlex.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(
        &mut unichars,
        &mut runs,
        &font,
        32.0,
        "hello look\u{2028}here\nsecond paragraph",
    );
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(paragraphs[0].runs.len(), 1);
    assert_eq!(paragraphs[0].base_direction(), TextDirection::Ltr);
    assert_eq!(GlyphLine::break_lines(&paragraphs[0].runs, 300.0).len(), 2);
    assert_eq!(paragraphs[1].runs.len(), 1);
    assert_eq!(paragraphs[1].base_direction(), TextDirection::Ltr);
    assert_eq!(GlyphLine::break_lines(&paragraphs[1].runs, 300.0).len(), 1);
}

#[test]
fn shaper_handles_rtl() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    let text = "لمفاتيح ABC DEF";
    append(&mut unichars, &mut runs, &font, 32.0, text);
    let characters = text.chars().collect::<Vec<_>>();
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.base_direction(), TextDirection::Rtl);
    assert_eq!(GlyphLine::break_lines(&paragraph.runs, 300.0).len(), 1);
    let lines = GlyphLine::break_lines(&paragraph.runs, 196.0);
    assert_eq!(lines.len(), 2);
    let line = &lines[1];
    let run = &paragraph.runs[line.start_run_index as usize];
    let index = run.text_indices[line.start_glyph_index as usize];
    assert_eq!(characters[index as usize], 'D');
    assert_eq!(characters[index as usize + 1], 'E');
    assert_eq!(characters[index as usize + 2], 'F');
}

#[test]
fn shaper_handles_empty_space() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, " ");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.base_direction(), TextDirection::Ltr);
    assert_eq!(GlyphLine::break_lines(&paragraph.runs, 300.0).len(), 1);
}

#[test]
fn line_breaker_deals_with_empty_paragraphs() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, "hi\n ");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(paragraphs[0].base_direction(), TextDirection::Ltr);
    assert_eq!(GlyphLine::break_lines(&paragraphs[0].runs, -1.0).len(), 1);
    assert_eq!(paragraphs[1].base_direction(), TextDirection::Ltr);
    let lines = GlyphLine::break_lines(&paragraphs[1].runs, -1.0);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert_eq!(line.start_run_index, 0);
    assert_eq!(
        paragraphs[1].runs[line.start_run_index as usize]
            .glyphs
            .len(),
        1
    );
    assert_eq!(
        paragraphs[1].runs[line.start_run_index as usize]
            .text_indices
            .len(),
        1
    );
    assert_eq!(
        paragraphs[1].runs[line.start_run_index as usize].text_indices[0],
        3
    );
}

#[test]
fn line_breaker_deals_with_space_only_lines() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, "hi\u{2028} ");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.base_direction(), TextDirection::Ltr);
    assert_eq!(GlyphLine::break_lines(&paragraph.runs, -1.0).len(), 2);
}

#[test]
fn line_breaker_deals_with_empty_lines() {
    let font = load_font("IBMPlexSansArabic-Regular.ttf");
    let mut runs = Vec::new();
    let mut unichars = Vec::new();
    append(&mut unichars, &mut runs, &font, 32.0, "hi\n");
    let paragraphs = font.shape_text(&unichars, &runs, -1);
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.base_direction(), TextDirection::Ltr);
    let lines = GlyphLine::break_lines(&paragraph.runs, -1.0);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert_eq!(line.start_run_index, 0);
    assert_eq!(line.start_glyph_index, 0);
    assert_eq!(
        paragraph.runs[line.start_run_index as usize].glyphs.len(),
        3
    );
    assert_eq!(
        paragraph.runs[line.start_run_index as usize]
            .text_indices
            .len(),
        3
    );
    assert_eq!(
        paragraph.runs[line.start_run_index as usize].text_indices[0],
        0
    );
}
