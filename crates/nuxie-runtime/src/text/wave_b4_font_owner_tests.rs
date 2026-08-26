mod wave_b4_font_owner_tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[derive(Clone, Copy)]
    struct WaveB4LineMetrics {
        ascent: f32,
        cap_height: f32,
        x_height: f32,
    }

    #[derive(Clone, Copy)]
    struct WaveB4Axis {
        tag: u32,
        def: f32,
    }

    // These are the only test-only RawTextFont methods in this wave. They
    // expose the real occurrence-local fallback owner already consumed by
    // production shaping; they do not implement a missing Font API.
    impl RawTextFont {
        fn wave_b4_fallback_count(&self) -> usize {
            self.fallbacks.len()
        }

        fn wave_b4_clear_fallbacks(&mut self) {
            self.fallbacks = Arc::from([]);
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct WaveB4FontStyle {
        weight: u16,
        italic: bool,
    }

    fn inspect_font_style(font: &RawTextFont) -> Result<WaveB4FontStyle, &'static str> {
        assert_eq!(
            font.face_index(),
            0,
            "decoded the exact production font owner"
        );
        Err("RawTextFont has no public Font weight/italic inspection owner")
    }

    fn font_line_metrics(font: &RawTextFont) -> Result<WaveB4LineMetrics, &'static str> {
        assert_eq!(
            font.face_index(),
            0,
            "decoded the exact production font owner"
        );
        Err("RawTextFont has no public Font::lineMetrics owner")
    }

    fn font_cap_height(_font: &RawTextFont, _size: f32) -> Result<f32, &'static str> {
        Err("RawTextFont has no public Font::capHeight owner")
    }

    fn font_x_height(_font: &RawTextFont, _size: f32) -> Result<f32, &'static str> {
        Err("RawTextFont has no public Font::xHeight owner")
    }

    fn font_axis_count(font: &RawTextFont) -> Result<u16, &'static str> {
        assert_eq!(
            font.face_index(),
            0,
            "decoded the exact production font owner"
        );
        Err("RawTextFont has no public Font::getAxisCount owner")
    }

    fn font_axis(_font: &RawTextFont, _index: u16) -> Result<WaveB4Axis, &'static str> {
        Err("RawTextFont has no public Font::getAxis owner")
    }

    fn font_axis_value(_font: &RawTextFont, _tag: u32) -> Result<f32, &'static str> {
        Err("RawTextFont has no public Font::getAxisValue owner")
    }

    fn font_make_at_coords(
        _font: &RawTextFont,
        _coords: &[(u32, f32)],
    ) -> Result<RawTextFont, &'static str> {
        Err("RawTextFont has no public Font::makeAtCoords owner")
    }

    fn font_features(font: &RawTextFont) -> Result<Vec<u32>, &'static str> {
        assert_eq!(
            font.face_index(),
            0,
            "decoded the exact production font owner"
        );
        Err("RawTextFont has no public Font::features owner")
    }

    fn upstream_root() -> PathBuf {
        std::env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
    }

    fn load_font(path: &str) -> RawTextFont {
        let bytes = std::fs::read(upstream_root().join("tests/unit_tests").join(path))
            .unwrap_or_else(|error| panic!("read pinned font {path}: {error}"));
        RawTextFont::decode(Arc::<[u8]>::from(bytes))
            .unwrap_or_else(|error| panic!("decode pinned font {path}: {error}"))
    }

    fn catch_approx_eq(actual: f32, expected: f32) -> bool {
        let actual = f64::from(actual);
        let expected = f64::from(expected);
        let scale = f64::from(f32::EPSILON) * 100.0 * expected.abs();
        (actual - expected).abs() <= scale
    }

    #[test]
    fn wave_b4_font_catch_approx_oracle_rejects_absolute_only_counterexample() {
        let expected = f32::from_bits(0x0072_abfc);
        let actual = f32::from_bits(expected.to_bits() + 90);
        assert!((f64::from(actual) - f64::from(expected)).abs() < f64::from(f32::EPSILON) * 100.0);
        assert!(!catch_approx_eq(actual, expected));
    }

    #[test]
    #[ignore = "expected-red: RawTextFont has no public Font weight/italic inspection owner"]
    fn wave_b4_font_test_001_inspect_font_styles() {
        let cases = [
            (
                "assets/fonts/AdventPro-VariableFont_wdth,wght.ttf",
                400,
                false,
            ),
            ("assets/fonts/Inter_18pt-Regular.ttf", 400, false),
            ("assets/fonts/Inter_28pt-Bold.ttf", 700, false),
            ("assets/fonts/OpenSans-Italic.ttf", 400, true),
            ("assets/fonts/OpenSans-ExtraBoldItalic.ttf", 800, true),
        ];
        for (path, expected_weight, expected_italic) in cases {
            let font = load_font(path);
            let style = inspect_font_style(&font).expect("Font weight/italic owner");
            assert_eq!(style.weight, expected_weight, "{path}");
            assert_eq!(style.italic, expected_italic, "{path}");
        }
    }

    #[test]
    #[ignore = "expected-red: RawTextFont has no public Font lineMetrics/capHeight/xHeight owner"]
    fn wave_b4_font_test_002_cap_and_x_height_for_vertical_trim() {
        for path in [
            "assets/fonts/Inter_18pt-Regular.ttf",
            "assets/Montserrat.ttf",
        ] {
            let font = load_font(path);
            let metrics = font_line_metrics(&font).expect("Font::lineMetrics owner");
            assert!(metrics.cap_height < 0.0, "{path}");
            assert!(metrics.cap_height >= metrics.ascent, "{path}");
            assert!(metrics.x_height > metrics.cap_height, "{path}");
            assert!(metrics.x_height < 0.0, "{path}");
            assert!(catch_approx_eq(
                font_cap_height(&font, 20.0).expect("Font::capHeight owner"),
                metrics.cap_height * 20.0
            ));
            assert!(catch_approx_eq(
                font_x_height(&font, 20.0).expect("Font::xHeight owner"),
                metrics.x_height * 20.0
            ));
        }
    }

    #[test]
    fn wave_b4_font_test_003_fallback_glyphs_are_found() {
        let fallback = load_font("assets/IBMPlexSansArabic-Regular.ttf");
        let fallback_bytes = Arc::clone(&fallback.bytes);
        let mut font = load_font("assets/RobotoFlex.ttf");
        assert_eq!(font.wave_b4_fallback_count(), 0);
        font = font.with_fallbacks([fallback]);
        assert_eq!(font.wave_b4_fallback_count(), 1);
        let text = "لمفاتيح ABC DEF";
        let run = StandaloneTextRun {
            text: text.to_owned(),
            font: font.clone(),
            size: 32.0,
            line_height: -1.0,
            letter_spacing: 0.0,
            style_index: 0,
            char_start: 0,
        };
        let glyphs = shape_standalone_run(&run);
        let mut paragraphs = standalone_break_lines(text, &glyphs, TextSizing::AutoWidth, 0.0);
        assert_eq!(paragraphs.len(), 1);
        assert!(
            glyphs
                .iter()
                .any(|glyph| Arc::ptr_eq(&glyph.font.bytes, &fallback_bytes))
        );
        paragraphs.clear();
        assert!(paragraphs.is_empty());
        drop(paragraphs);
        drop(glyphs);
        drop(run);
        font.wave_b4_clear_fallbacks();
        assert_eq!(font.wave_b4_fallback_count(), 0);
    }

    #[test]
    #[ignore = "expected-red: RawTextFont has no public Font variation-axis owner"]
    fn wave_b4_font_test_004_variable_axis_values_can_be_read() {
        const WGHT: u32 = 2_003_265_652;
        const WDTH: u32 = 2_003_072_104;
        let font = load_font("assets/RobotoFlex.ttf");
        let mut has_weight = false;
        for index in 0..font_axis_count(&font).expect("Font::getAxisCount owner") {
            let axis = font_axis(&font, index).expect("Font::getAxis owner");
            if axis.tag == WGHT {
                assert_eq!(axis.def, 400.0);
                has_weight = true;
                break;
            }
        }
        assert!(has_weight);
        assert_eq!(
            font_axis_value(&font, WGHT).expect("Font::getAxisValue owner"),
            400.0
        );
        assert_eq!(
            font_axis_value(&font, WDTH).expect("Font::getAxisValue owner"),
            100.0
        );
        let weighted =
            font_make_at_coords(&font, &[(WGHT, 800.0)]).expect("Font::makeAtCoords weight owner");
        assert_eq!(
            font_axis_value(&weighted, WGHT).expect("varied weight owner"),
            800.0
        );
        let weighted_and_wide = font_make_at_coords(&weighted, &[(WDTH, 122.0)])
            .expect("Font::makeAtCoords width owner");
        assert_eq!(
            font_axis_value(&weighted_and_wide, WDTH).expect("varied width owner"),
            122.0
        );
        assert_eq!(
            font_axis_value(&weighted_and_wide, WGHT).expect("retained weight owner"),
            800.0
        );
    }

    #[test]
    #[ignore = "expected-red: RawTextFont has no public Font feature owner"]
    fn wave_b4_font_test_005_font_features_load_as_expected() {
        let font = load_font("assets/RobotoFlex.ttf");
        let features = font_features(&font).expect("Font::features owner");
        assert_eq!(features.len(), 7);
        for expected in ["mkmk", "kern", "rvrn", "mark", "locl", "pnum", "liga"] {
            let tag = u32::from_be_bytes(expected.as_bytes().try_into().expect("four-byte tag"));
            assert!(features.contains(&tag), "missing feature {expected}");
        }
    }
}
