mod wave_c2_line_break_owner_tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn font() -> RawTextFont {
        let root = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
        let path = root.join("tests/unit_tests/assets/RobotoFlex.ttf");
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("read pinned font {}: {error}", path.display()));
        RawTextFont::decode(Arc::<[u8]>::from(bytes)).expect("RobotoFlex decodes")
    }

    fn run(font: &RawTextFont, text: &str, size: f32, char_start: usize) -> StandaloneTextRun {
        StandaloneTextRun {
            text: text.to_owned(),
            font: font.clone(),
            size,
            line_height: -1.0,
            letter_spacing: 0.0,
            style_index: 0,
            char_start,
        }
    }

    fn shape(runs: &[StandaloneTextRun]) -> Vec<StandaloneGlyph> {
        runs.iter().flat_map(shape_standalone_run).collect()
    }

    fn break_lines(text: &str, glyphs: &[StandaloneGlyph], width: f32) -> Vec<StandaloneLine> {
        standalone_break_lines(text, glyphs, TextSizing::AutoHeight, width)
    }

    fn shape_annotations(
        text: &str,
        runs: &[StandaloneTextRun],
    ) -> Vec<nuxie_render_api::GlyphRunAnnotations> {
        let indices = runs
            .iter()
            .map(|run| {
                shape_standalone_run(run)
                    .into_iter()
                    .map(|glyph| u32::try_from(glyph.char_index).expect("glyph index fits u32"))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let index_slices = indices.iter().map(Vec::as_slice).collect::<Vec<_>>();
        nuxie_render_api::annotate_glyph_runs(&text.chars().collect::<Vec<_>>(), &index_slices)
            .expect("the production Font::shapeText annotation owner accepts shaped indices")
    }

    #[test]
    fn wave_c2_line_break_001_separates_words() {
        let font = font();
        let text = "one two three";
        let run = run(&font, text, 32.0, 0);
        let glyphs = shape(std::slice::from_ref(&run));
        assert_eq!(glyphs.len(), 13);
        let annotations = shape_annotations(text, std::slice::from_ref(&run));
        assert_eq!(annotations.len(), 1);
        let breaks = &annotations[0].breaks;
        assert_eq!(breaks.len(), 6);
        assert_eq!(breaks[0], 0);
        assert_eq!(breaks[1], 3);
        assert_eq!(breaks[2], 4);
        assert_eq!(breaks[3], 7);
        assert_eq!(breaks[4], 8);
        assert_eq!(breaks[5], 13);
    }

    #[test]
    fn wave_c2_line_break_002_handles_multiple_runs() {
        let font = font();
        let runs = [
            run(&font, "one two thr", 32.0, 0),
            run(&font, "ee four", 60.0, 11),
        ];
        let glyphs = shape(&runs);
        assert_eq!(
            glyphs.iter().filter(|glyph| glyph.char_index < 11).count(),
            11
        );
        assert_eq!(
            glyphs.iter().filter(|glyph| glyph.char_index >= 11).count(),
            7
        );
        let annotations = shape_annotations("one two three four", &runs);
        assert_eq!(annotations.len(), 2);
        let first = &annotations[0].breaks;
        assert_eq!(first.as_slice(), [0, 3, 4, 7, 8]);
        let second = &annotations[1].breaks;
        assert_eq!(second.as_slice(), [2, 3, 7]);
    }

    #[test]
    fn wave_c2_line_break_003_handles_returns() {
        let font = font();
        let runs = [
            run(&font, "one two thr", 32.0, 0),
            run(&font, "ee\u{2028} four", 60.0, 11),
        ];
        let glyphs = shape(&runs);
        assert!(glyphs.iter().any(|glyph| glyph.char_index == 13));
        let annotations = shape_annotations("one two three\u{2028} four", &runs);
        assert_eq!(annotations.len(), 2);
        let first = &annotations[0].breaks;
        assert_eq!(first.as_slice(), [0, 3, 4, 7, 8]);
        let second = &annotations[1].breaks;
        assert_eq!(second.as_slice(), [2, 2, 2, 4, 8]);
    }

    #[test]
    #[ignore = "expected-red: the concrete RawText line owner retains the wrap-space glyph on line one instead of the pinned endGlyphIndex 7 boundary"]
    fn wave_c2_line_break_004_builds_lines() {
        let font = font();
        let text = "one two three";
        let glyphs = shape(&[run(&font, text, 32.0, 0)]);
        let wide = break_lines(text, &glyphs, 194.0);
        assert_eq!(wide.len(), 1);
        assert_eq!((wide[0].char_start, wide[0].char_end), (0, 13));
        let narrow = break_lines(text, &glyphs, 191.0);
        assert_eq!(narrow.len(), 2);
        assert_eq!((narrow[0].char_start, narrow[0].char_end), (0, 7));
        assert_eq!((narrow[1].char_start, narrow[1].char_end), (8, 13));
    }

    #[test]
    #[ignore = "expected-red: standalone_break_lines treats non-positive width as unbounded instead of producing one glyph per line"]
    fn wave_c2_line_break_005_deals_with_extremes() {
        let font = font();
        let text = "ab";
        let glyphs = shape(&[run(&font, text, 32.0, 0)]);
        for width in [17.0, 0.0] {
            let lines = break_lines(text, &glyphs, width);
            assert_eq!(lines.len(), 2);
            assert_eq!((lines[0].char_start, lines[0].char_end), (0, 1));
            assert_eq!((lines[1].char_start, lines[1].char_end), (1, 2));
        }
    }

    #[test]
    #[ignore = "expected-red: standalone_break_lines recognizes only newline as a forced break and keeps U+2028 on one wide line"]
    fn wave_c2_line_break_006_breaks_return_characters() {
        let font = font();
        let text = "hello look\u{2028}here";
        let glyphs = shape(&[run(&font, text, 32.0, 0)]);
        assert_eq!(break_lines(text, &glyphs, 300.0).len(), 2);
    }

    #[test]
    fn wave_c2_line_break_008_shaper_handles_rtl() {
        // Pinned loadFont ignores its filename argument and decodes RobotoFlex.
        let font = font();
        let text = "لمفاتيح ABC DEF";
        let runs = [run(&font, text, 32.0, 0)];
        let glyphs = shape(&runs);
        let mut wide = break_lines(text, &glyphs, 300.0);
        standalone_reorder_bidi(text, &mut wide);
        assert_eq!(
            wide.iter().map(|line| line.paragraph).collect::<Vec<_>>(),
            [0]
        );
        assert_eq!(shape_annotations(text, &runs).len(), 1);
        assert_eq!(wide.len(), 1);
        assert!(paragraph_base_is_rtl(text, wide[0].char_start));

        let mut narrow = break_lines(text, &glyphs, 196.0);
        standalone_reorder_bidi(text, &mut narrow);
        assert_eq!(narrow.len(), 2);
        let second = narrow.last().expect("second line");
        let index = second
            .glyphs
            .first()
            .expect("first visual glyph")
            .char_index;
        assert_eq!(text.chars().nth(index), Some('D'));
        assert_eq!(text.chars().nth(index + 1), Some('E'));
        assert_eq!(text.chars().nth(index + 2), Some('F'));
    }

    #[test]
    fn wave_c2_line_break_009_shaper_handles_empty_space() {
        let font = font();
        let text = " ";
        let runs = [run(&font, text, 32.0, 0)];
        let glyphs = shape(&runs);
        assert_eq!(glyphs.len(), 1);
        assert_eq!(shape_annotations(text, &runs).len(), 1);
        let lines = break_lines(text, &glyphs, 300.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].paragraph, 0);
        assert!(!paragraph_base_is_rtl(text, lines[0].char_start));
    }

    #[test]
    fn wave_c2_line_break_010_deals_with_empty_paragraphs() {
        let font = font();
        let text = "hi\n ";
        let runs = [run(&font, text, 32.0, 0)];
        let glyphs = shape(&runs);
        assert_eq!(shape_annotations(text, &runs).len(), 1);
        let lines = break_lines(text, &glyphs, -1.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].paragraph, 0);
        assert_eq!(lines[1].paragraph, 1);
        assert!(!paragraph_base_is_rtl(text, lines[0].char_start));
        assert!(!paragraph_base_is_rtl(text, lines[1].char_start));
        assert_eq!(lines[1].glyphs.len(), 1);
        assert_eq!(lines[1].glyphs[0].char_index, 3);
    }
}
