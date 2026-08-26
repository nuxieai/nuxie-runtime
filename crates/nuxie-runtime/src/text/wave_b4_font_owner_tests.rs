mod wave_b4_font_owner_tests {
    use super::*;
    use skrifa::attribute::Style;
    use std::collections::{BTreeMap, BTreeSet};
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

    // Test-only inherent methods keep every assertion on the runtime owner;
    // Skrifa is its implementation detail, like HarfBuzz is for C++ HBFont.
    impl RawTextFont {
        fn wave_b4_face(&self) -> SkrifaFontRef<'_> {
            SkrifaFontRef::from_index(self.bytes(), self.face_index)
                .expect("RawTextFont retained its validated face")
        }

        fn wave_b4_get_weight(&self) -> u16 {
            self.wave_b4_face().attributes().weight.value() as u16
        }

        fn wave_b4_is_italic(&self) -> bool {
            matches!(
                self.wave_b4_face().attributes().style,
                Style::Italic | Style::Oblique(_)
            )
        }

        fn wave_b4_line_metrics(&self) -> WaveB4LineMetrics {
            let metrics = self
                .wave_b4_face()
                .metrics(Size::new(1.0), LocationRef::default());
            WaveB4LineMetrics {
                ascent: -metrics.ascent,
                cap_height: -metrics.cap_height.expect("font exposes cap height"),
                x_height: -metrics.x_height.expect("font exposes x height"),
            }
        }

        fn wave_b4_cap_height(&self, size: f32) -> f32 {
            -self
                .wave_b4_face()
                .metrics(Size::new(size), LocationRef::default())
                .cap_height
                .expect("font exposes cap height")
        }

        fn wave_b4_x_height(&self, size: f32) -> f32 {
            -self
                .wave_b4_face()
                .metrics(Size::new(size), LocationRef::default())
                .x_height
                .expect("font exposes x height")
        }

        fn wave_b4_fallback_count(&self) -> usize {
            self.fallbacks.len()
        }

        fn wave_b4_clear_fallbacks(&mut self) {
            self.fallbacks = Arc::from([]);
        }

        fn wave_b4_get_axis_count(&self) -> u16 {
            self.wave_b4_face().axes().len() as u16
        }

        fn wave_b4_get_axis(&self, index: u16) -> Option<WaveB4Axis> {
            let axis = self.wave_b4_face().axes().get(usize::from(index))?;
            Some(WaveB4Axis {
                tag: u32::from_be_bytes(axis.tag().to_be_bytes()),
                def: axis.default_value(),
            })
        }

        fn wave_b4_get_axis_value(&self, tag: u32) -> Option<f32> {
            let tag = SkrifaTag::from_u32(tag);
            self.variation_coords
                .iter()
                .find_map(|(candidate, value)| (*candidate == tag).then_some(*value))
                .or_else(|| {
                    self.wave_b4_face()
                        .axes()
                        .get_by_tag(tag)
                        .map(|axis| axis.default_value())
                })
        }

        fn wave_b4_make_at_coords(&self, coords: &[(u32, f32)]) -> Self {
            let mut next = self.clone();
            let mut merged = self
                .variation_coords
                .iter()
                .copied()
                .collect::<BTreeMap<_, _>>();
            for (tag, value) in coords {
                merged.insert(SkrifaTag::from_u32(*tag), *value);
            }
            next.variation_coords = merged.into_iter().collect::<Vec<_>>().into();
            next
        }

        fn wave_b4_features(&self) -> Vec<u32> {
            let face = self.wave_b4_face();
            let mut features = BTreeSet::new();
            if let Ok(table) = face.gpos() {
                if let Ok(list) = table.feature_list() {
                    features.extend(
                        list.feature_records()
                            .iter()
                            .map(|record| u32::from_be_bytes(record.feature_tag().to_be_bytes())),
                    );
                }
            }
            if let Ok(table) = face.gsub() {
                if let Ok(list) = table.feature_list() {
                    features.extend(
                        list.feature_records()
                            .iter()
                            .map(|record| u32::from_be_bytes(record.feature_tag().to_be_bytes())),
                    );
                }
            }
            features.into_iter().collect()
        }
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
            assert_eq!(font.wave_b4_get_weight(), expected_weight, "{path}");
            assert_eq!(font.wave_b4_is_italic(), expected_italic, "{path}");
        }
    }

    #[test]
    fn wave_b4_font_test_002_cap_and_x_height_for_vertical_trim() {
        for path in [
            "assets/fonts/Inter_18pt-Regular.ttf",
            "assets/Montserrat.ttf",
        ] {
            let font = load_font(path);
            let metrics = font.wave_b4_line_metrics();
            assert!(metrics.cap_height < 0.0, "{path}");
            assert!(metrics.cap_height >= metrics.ascent, "{path}");
            assert!(metrics.x_height > metrics.cap_height, "{path}");
            assert!(metrics.x_height < 0.0, "{path}");
            assert!(catch_approx_eq(
                font.wave_b4_cap_height(20.0),
                metrics.cap_height * 20.0
            ));
            assert!(catch_approx_eq(
                font.wave_b4_x_height(20.0),
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
    fn wave_b4_font_test_004_variable_axis_values_can_be_read() {
        const WGHT: u32 = 2_003_265_652;
        const WDTH: u32 = 2_003_072_104;
        let font = load_font("assets/RobotoFlex.ttf");
        let mut has_weight = false;
        for index in 0..font.wave_b4_get_axis_count() {
            let axis = font
                .wave_b4_get_axis(index)
                .expect("axis in reported range");
            if axis.tag == WGHT {
                assert_eq!(axis.def, 400.0);
                has_weight = true;
                break;
            }
        }
        assert!(has_weight);
        assert_eq!(font.wave_b4_get_axis_value(WGHT), Some(400.0));
        assert_eq!(font.wave_b4_get_axis_value(WDTH), Some(100.0));
        let weighted = font.wave_b4_make_at_coords(&[(WGHT, 800.0)]);
        assert_eq!(weighted.wave_b4_get_axis_value(WGHT), Some(800.0));
        let weighted_and_wide = weighted.wave_b4_make_at_coords(&[(WDTH, 122.0)]);
        assert_eq!(weighted_and_wide.wave_b4_get_axis_value(WDTH), Some(122.0));
        assert_eq!(weighted_and_wide.wave_b4_get_axis_value(WGHT), Some(800.0));
    }

    #[test]
    fn wave_b4_font_test_005_font_features_load_as_expected() {
        let font = load_font("assets/RobotoFlex.ttf");
        let features = font.wave_b4_features();
        assert_eq!(features.len(), 7);
        for expected in ["mkmk", "kern", "rvrn", "mark", "locl", "pnum", "liga"] {
            let tag = u32::from_be_bytes(expected.as_bytes().try_into().expect("four-byte tag"));
            assert!(features.contains(&tag), "missing feature {expected}");
        }
    }
}
