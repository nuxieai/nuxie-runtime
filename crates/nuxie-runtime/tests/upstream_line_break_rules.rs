//! All five line_break_rules_test.cpp cases at upstream 02bea09b.
use nuxie_runtime::source::{
    text::font_hb::HbFont,
    text_engine::{GlyphLine, GlyphRun, TextRun},
};
use std::path::PathBuf;

type Words = Vec<(u32, u32)>;

fn shape_words(text: &str) -> Vec<Words> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/RobotoFlex.ttf");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned font {}: {error}", path.display()));
    let font = HbFont::decode(&bytes).expect("pinned font decodes");
    let text: Vec<u32> = text.chars().map(u32::from).collect();
    let run = TextRun {
        font: Some(font.clone()),
        size: 32.0,
        line_height: -1.0,
        letter_spacing: 0.0,
        unichar_count: text.len() as u32,
        script: 0,
        style_id: 0,
        level: 0,
    };
    font.shape_text(&text, &[run], -1)
        .iter()
        .map(|paragraph| {
            let mut stream = Vec::new();
            let mut base = 0;
            for run in &paragraph.runs {
                stream.extend(run.breaks.iter().map(|index| base + index));
                base += run.glyphs.len() as u32;
            }
            stream
                .chunks_exact(2)
                .map(|pair| (pair[0], pair[1]))
                .collect()
        })
        .collect()
}

#[test]
fn line_breaker_allows_breaks_after_hyphens_dashes_and_slashes() {
    let words = shape_words("state-of-the-art x\u{2014}y a/b");
    assert_eq!(words.len(), 1);
    assert_eq!(
        words[0],
        [
            (0, 6),
            (6, 9),
            (9, 13),
            (13, 16),
            (17, 18),
            (18, 19),
            (19, 20),
            (21, 23),
            (23, 24)
        ]
    );
}

#[test]
fn line_breaker_keeps_no_break_space_soft_hyphen_and_word_joiner_glued() {
    let words = shape_words("100\u{00a0}km su\u{00ad}per 5\u{2060}x");
    assert_eq!(words.len(), 1);
    assert_eq!(words[0], [(0, 6), (7, 13), (14, 17)]);
}

#[test]
fn line_breaker_breaks_cjk_per_character_with_kinsoku() {
    let words = shape_words("\u{65e5}\u{672c}\u{3001}\u{300c}\u{8a9e}\u{300d}\u{3002}x\u{3000}y");
    assert_eq!(words.len(), 1);
    assert_eq!(words[0], [(0, 1), (1, 3), (3, 7), (7, 8), (9, 10)]);
}

#[test]
fn line_breaker_treats_every_hard_break_as_a_forced_break() {
    let words = shape_words("a\r\nb\u{0085}c\u{2029}d");
    assert_eq!(words.len(), 4);
    assert_eq!(words[0], [(0, 1), (2, 2)]);
    assert_eq!(words[1], [(0, 1), (1, 1)]);
    assert_eq!(words[2], [(0, 1), (1, 1)]);
    assert_eq!(words[3], [(0, 1)]);
    let words = shape_words("d e\u{000b}f\u{2028}g");
    assert_eq!(words.len(), 1);
    assert_eq!(words[0], [(0, 1), (2, 3), (3, 3), (4, 5), (5, 5), (6, 7)]);
}

#[test]
fn line_breaker_never_splits_a_glyph_cluster_when_a_word_overflows() {
    let mut run = GlyphRun::new(6);
    let indices = [0, 1, 1, 2, 3, 3];
    for (i, index) in indices.into_iter().enumerate() {
        run.glyphs[i] = (i + 1) as _;
        run.text_indices[i] = index;
        run.advances[i] = 10.0;
        run.xpos[i] = 10.0 * i as f32;
    }
    run.xpos[6] = 60.0;
    run.breaks = vec![0, 6];
    let lines = GlyphLine::break_lines(&[run], 25.0);
    let actual: Words = lines
        .iter()
        .map(|line| (line.start_glyph_index, line.end_glyph_index))
        .collect();
    assert_eq!(actual, [(0, 1), (1, 3), (3, 4), (4, 6)]);
}
