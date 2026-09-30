//! Direct ports of all twenty pinned line_break_test.cpp cases.
//! Both shaping and line breaking run through translated production owners.

use nuxie_runtime::source::{
    text::font_hb::HbFont,
    text_engine::{FontRef, GlyphLine, Paragraph, TextDirection, TextRun, TextWordBreak},
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

// Upstream shapeOneRun: shape one 32pt run through the production font.
fn shape_one_run(font: &FontRef, text: &str) -> Vec<Paragraph> {
    let mut unichars = Vec::new();
    let mut runs = Vec::new();
    append(&mut unichars, &mut runs, font, 32.0, text);
    font.shape_text(&unichars, &runs, -1)
}

#[test]
fn word_break_normal_never_splits_a_word() {
    let font = load_font("RobotoFlex.ttf");
    let paragraphs = shape_one_run(&font, "ab");
    assert_eq!(paragraphs.len(), 1);
    for width in [17.0, 1.0, 0.0] {
        let lines = GlyphLine::break_lines_with_word_break(
            &paragraphs[0].runs,
            width,
            TextWordBreak::Normal,
        );
        assert_eq!(lines.len(), 1);
        assert_line(&lines[0], 0, 0, 0, 2);
    }
}

#[test]
fn word_break_normal_still_wraps_between_words() {
    let font = load_font("RobotoFlex.ttf");
    let paragraphs = shape_one_run(&font, "one two three");
    assert_eq!(paragraphs.len(), 1);
    let runs = &paragraphs[0].runs;
    let run = &runs[0];
    let width = (run.xpos[3] + run.xpos[4]) / 2.0;
    let lines = GlyphLine::break_lines_with_word_break(runs, width, TextWordBreak::Normal);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].start_glyph_index, 0);
    assert_eq!(lines[0].end_glyph_index, 3);
    assert_eq!(lines[1].start_glyph_index, 4);
    assert_eq!(lines[1].end_glyph_index, 7);
    assert_eq!(lines[2].start_glyph_index, 8);
    assert_eq!(lines[2].end_glyph_index, 13);
    let broken = GlyphLine::break_lines_with_word_break(runs, width, TextWordBreak::BreakWord);
    assert!(broken.len() > lines.len());
}

#[test]
fn word_break_all_fills_the_line_it_is_already_on() {
    let font = load_font("RobotoFlex.ttf");
    let paragraphs = shape_one_run(&font, "one two three");
    assert_eq!(paragraphs.len(), 1);
    let runs = &paragraphs[0].runs;
    let run = &runs[0];
    let width = (run.xpos[9] + run.xpos[10]) / 2.0;
    let lines = GlyphLine::break_lines_with_word_break(runs, width, TextWordBreak::BreakAll);
    assert!(lines.len() >= 2);
    assert_eq!(lines[0].start_glyph_index, 0);
    assert_eq!(lines[0].end_glyph_index, 9);
    assert_eq!(lines[1].start_glyph_index, 9);
    let broken = GlyphLine::break_lines_with_word_break(runs, width, TextWordBreak::BreakWord);
    assert_eq!(broken[0].end_glyph_index, 7);
}

#[test]
fn word_break_all_never_cuts_inside_the_space_before_a_word() {
    let font = load_font("RobotoFlex.ttf");
    let paragraphs = shape_one_run(&font, "one two three");
    assert_eq!(paragraphs.len(), 1);
    let runs = &paragraphs[0].runs;
    let run = &runs[0];
    for width in [
        (run.xpos[8] + run.xpos[9]) / 2.0,
        (run.xpos[7] + run.xpos[8]) / 2.0,
    ] {
        let lines = GlyphLine::break_lines_with_word_break(runs, width, TextWordBreak::BreakAll);
        assert!(lines.len() >= 2);
        assert_eq!(lines[0].end_glyph_index, 7);
        assert_eq!(lines[1].start_glyph_index, 8);
    }
}

#[test]
fn word_break_all_matches_break_word_for_a_lone_word() {
    let font = load_font("RobotoFlex.ttf");
    let paragraphs = shape_one_run(&font, "ab");
    assert_eq!(paragraphs.len(), 1);
    let runs = &paragraphs[0].runs;
    for width in [17.0, 1.0, 0.0] {
        let all = GlyphLine::break_lines_with_word_break(runs, width, TextWordBreak::BreakAll);
        let word = GlyphLine::break_lines_with_word_break(runs, width, TextWordBreak::BreakWord);
        assert_eq!(all.len(), word.len());
        for i in 0..all.len() {
            assert_eq!(all[i], word[i]);
        }
    }
}

#[test]
fn word_break_all_honors_word_joiners() {
    let font = load_font("RobotoFlex.ttf");
    let paragraphs = shape_one_run(&font, "xx abc\u{2060}def");
    assert_eq!(paragraphs.len(), 1);
    let runs = &paragraphs[0].runs;
    let run = &runs[0];
    for i in 3..run.glyphs.len() {
        let lines = GlyphLine::break_lines_with_word_break(
            runs,
            (run.xpos[i] + run.xpos[i + 1]) / 2.0,
            TextWordBreak::BreakAll,
        );
        for line in lines {
            for &joiner in &run.joiners {
                if (line.start_glyph_index as usize) < run.text_indices.len() {
                    assert_ne!(run.text_indices[line.start_glyph_index as usize], joiner);
                }
                if (line.end_glyph_index as usize) < run.text_indices.len() {
                    assert_ne!(run.text_indices[line.end_glyph_index as usize], joiner);
                }
            }
        }
    }
}

#[test]
fn word_break_modes_hold_the_line_invariants() {
    let font = load_font("RobotoFlex.ttf");
    let samples = [
        "one two three",
        "  hello world  ",
        "supercalifragilisticexpialidocious",
        "foo-bar-baz",
        "ab\u{2060}cd ef",
        "auto\u{ad}mobile",
        "a",
        " ",
    ];
    let widths = [-1.0, 0.0, 1.0, 5.0, 17.0, 50.0, 97.0, 191.0, 194.0, 1000.0];
    let modes = [
        TextWordBreak::BreakWord,
        TextWordBreak::Normal,
        TextWordBreak::BreakAll,
    ];
    for sample in samples {
        for paragraph in shape_one_run(&font, sample) {
            let total_glyphs: usize = paragraph.runs.iter().map(|run| run.glyphs.len()).sum();
            for width in widths {
                for mode in modes {
                    let lines =
                        GlyphLine::break_lines_with_word_break(&paragraph.runs, width, mode);
                    assert!(lines.len() <= total_glyphs + 2);
                    let mut previous: Option<&GlyphLine> = None;
                    for line in &lines {
                        assert!(line.end_run_index >= line.start_run_index);
                        let start_run = &paragraph.runs[line.start_run_index as usize];
                        let end_run = &paragraph.runs[line.end_run_index as usize];
                        assert!(line.start_glyph_index as usize <= start_run.xpos.len());
                        assert!(line.end_glyph_index as usize <= end_run.xpos.len());
                        let from = start_run.xpos[line.start_glyph_index as usize];
                        let to = end_run.xpos[line.end_glyph_index as usize];
                        assert!(to >= from);
                        if let Some(previous) = previous {
                            assert!(line.start_run_index >= previous.end_run_index);
                            if line.start_run_index == previous.end_run_index {
                                assert!(line.start_glyph_index >= previous.end_glyph_index);
                            }
                        }
                        if width >= 0.0
                            && mode != TextWordBreak::Normal
                            && line.end_glyph_index > line.start_glyph_index + 1
                        {
                            assert!(to - from <= width);
                        }
                        previous = Some(line);
                    }
                }
            }
        }
    }
}

#[test]
fn word_break_modes_agree_when_there_is_nothing_to_break() {
    let font = load_font("RobotoFlex.ttf");
    let paragraphs = shape_one_run(&font, "one two three");
    assert_eq!(paragraphs.len(), 1);
    let runs = &paragraphs[0].runs;
    for width in [-1.0, 1000.0] {
        let word = GlyphLine::break_lines_with_word_break(runs, width, TextWordBreak::BreakWord);
        for mode in [TextWordBreak::Normal, TextWordBreak::BreakAll] {
            let other = GlyphLine::break_lines_with_word_break(runs, width, mode);
            assert_eq!(other.len(), word.len());
            for i in 0..word.len() {
                assert_eq!(other[i], word[i]);
            }
        }
    }
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
