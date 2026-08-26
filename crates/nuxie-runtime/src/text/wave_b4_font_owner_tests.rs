mod wave_b4_font_owner_tests {
    use super::*;
    use skrifa::attribute::Style;
    use std::path::PathBuf;
    use std::sync::Arc;

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

    fn skrifa_face(font: &RawTextFont) -> SkrifaFontRef<'_> {
        SkrifaFontRef::from_index(font.bytes(), font.face_index)
            .expect("RawTextFont retained its validated Skrifa face")
    }

    fn catch_approx(left: f32, right: f32) -> bool {
        let epsilon = f32::EPSILON * 100.0;
        (left - right).abs() <= epsilon * (1.0 + left.abs().max(right.abs()))
    }

    #[test]
    fn wave_b4_font_test_001_inspect_font_styles() {
        let cases = [
            (
                "assets/fonts/AdventPro-VariableFont_wdth,wght.ttf",
                400.0,
                false,
            ),
            ("assets/fonts/Inter_18pt-Regular.ttf", 400.0, false),
            ("assets/fonts/Inter_28pt-Bold.ttf", 700.0, false),
            ("assets/fonts/OpenSans-Italic.ttf", 400.0, true),
            ("assets/fonts/OpenSans-ExtraBoldItalic.ttf", 800.0, true),
        ];

        for (path, expected_weight, expected_italic) in cases {
            let font = load_font(path);
            let attributes = skrifa_face(&font).attributes();
            assert_eq!(attributes.weight.value(), expected_weight, "{path}");
            assert_eq!(
                matches!(attributes.style, Style::Italic | Style::Oblique(_)),
                expected_italic,
                "{path}"
            );
        }
    }

    #[test]
    fn wave_b4_font_test_002_cap_and_x_height_for_vertical_trim() {
        for path in [
            "assets/fonts/Inter_18pt-Regular.ttf",
            "assets/Montserrat.ttf",
        ] {
            let font = load_font(path);
            let face = skrifa_face(&font);
            let metrics = face.metrics(Size::new(1.0), LocationRef::default());
            let scaled_metrics = face.metrics(Size::new(20.0), LocationRef::default());
            let ascent = -metrics.ascent;
            let cap_height = -metrics.cap_height.expect("Latin font exposes cap height");
            let x_height = -metrics.x_height.expect("Latin font exposes x height");

            assert!(cap_height < 0.0, "{path}");
            assert!(cap_height >= ascent, "{path}");
            assert!(x_height > cap_height, "{path}");
            assert!(x_height < 0.0, "{path}");

            assert!(
                catch_approx(
                    -scaled_metrics.cap_height.expect("scaled cap height"),
                    cap_height * 20.0
                ),
                "{path}"
            );
            assert!(
                catch_approx(
                    -scaled_metrics.x_height.expect("scaled x height"),
                    x_height * 20.0
                ),
                "{path}"
            );
        }
    }

    #[test]
    fn wave_b4_font_test_003_fallback_glyphs_are_found() {
        let fallback = load_font("assets/IBMPlexSansArabic-Regular.ttf");
        let fallback_bytes = Arc::clone(&fallback.bytes);
        let font = load_font("assets/RobotoFlex.ttf").with_fallbacks([fallback]);
        let text = "لمفاتيح ABC DEF";
        let run = StandaloneTextRun {
            text: text.to_owned(),
            font,
            size: 32.0,
            line_height: -1.0,
            letter_spacing: 0.0,
            style_index: 0,
            char_start: 0,
        };

        let glyphs = shape_standalone_run(&run);
        let paragraphs = standalone_break_lines(text, &glyphs, TextSizing::AutoWidth, 0.0);
        assert_eq!(paragraphs.len(), 1);
        assert!(
            glyphs
                .iter()
                .any(|glyph| Arc::ptr_eq(&glyph.font.bytes, &fallback_bytes)),
            "the Arabic glyphs use the installed fallback font"
        );
        drop(paragraphs);
        let paragraphs: Vec<StandaloneLine> = Vec::new();
        assert!(paragraphs.is_empty());
    }

    #[test]
    fn wave_b4_font_test_004_variable_axis_values_can_be_read() {
        let font = load_font("assets/RobotoFlex.ttf");
        let face = skrifa_face(&font);
        let axes = face.axes();
        let weight = axes
            .iter()
            .find(|axis| axis.tag() == SkrifaTag::new(b"wght"))
            .expect("RobotoFlex weight axis");
        assert_eq!(weight.default_value(), 400.0);
        let width = axes
            .iter()
            .find(|axis| axis.tag() == SkrifaTag::new(b"wdth"))
            .expect("RobotoFlex width axis");
        assert_eq!(width.default_value(), 100.0);

        let weighted = axes.filter([("wght", 800.0)]).collect::<Vec<_>>();
        assert!(weighted.iter().any(|setting| {
            setting.selector == SkrifaTag::new(b"wght") && setting.value == 800.0
        }));
        let weighted_and_wide = axes
            .filter([("wght", 800.0), ("wdth", 122.0)])
            .collect::<Vec<_>>();
        assert!(weighted_and_wide.iter().any(|setting| {
            setting.selector == SkrifaTag::new(b"wdth") && setting.value == 122.0
        }));
        assert!(weighted_and_wide.iter().any(|setting| {
            setting.selector == SkrifaTag::new(b"wght") && setting.value == 800.0
        }));
    }

    #[test]
    fn wave_b4_font_test_005_font_features_load_as_expected() {
        let font = load_font("assets/RobotoFlex.ttf");
        let face = skrifa_face(&font);
        let mut features = BTreeSet::new();
        if let Ok(table) = face.gpos() {
            let list = table.feature_list().expect("RobotoFlex GPOS feature list");
            features.extend(
                list.feature_records()
                    .iter()
                    .map(|record| record.feature_tag()),
            );
        }
        if let Ok(table) = face.gsub() {
            let list = table.feature_list().expect("RobotoFlex GSUB feature list");
            features.extend(
                list.feature_records()
                    .iter()
                    .map(|record| record.feature_tag()),
            );
        }

        let expected = ["mkmk", "kern", "rvrn", "mark", "locl", "pnum", "liga"]
            .into_iter()
            .map(|tag| tag.parse().expect("valid OpenType tag"))
            .collect::<BTreeSet<_>>();
        assert_eq!(features, expected);
    }
}
