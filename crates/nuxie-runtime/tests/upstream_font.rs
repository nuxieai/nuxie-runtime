//! One-for-one expected-red ports of all five cases in pinned
//! `tests/unit_tests/runtime/font_test.cpp`.
//!
//! The retained text pipeline consumes fonts internally, but Rust has no
//! standalone owner for the upstream public `Font` inspection, fallback, and
//! coordinate APIs. The fixtures and complete assertion bodies remain here.

use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
struct LineMetrics {
    ascent: f32,
    cap_height: f32,
    x_height: f32,
}

#[derive(Clone, Debug, Default)]
struct Axis {
    tag: u32,
    default: f32,
}

#[derive(Clone, Debug, Default)]
struct Font {
    weight: u16,
    italic: bool,
    line_metrics: LineMetrics,
    axes: Vec<Axis>,
    axis_values: Vec<(u32, f32)>,
    features: Vec<u32>,
}

#[derive(Debug, Default)]
struct Paragraph;

fn pinned_asset(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests").join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned font {}: {error}", path.display()))
}

fn load_font(name: &str) -> Font {
    let bytes = pinned_asset(name);
    decode_font(&bytes)
}

fn decode_font(_: &[u8]) -> Font {
    panic!("Rust has no standalone owner corresponding to the upstream public Font API")
}

fn has_glyph(_: &Font, _: char) -> bool {
    missing_font_owner()
}

fn shape_text_with_fallback<F>(_: &Font, _: &str, _: f32, _: F) -> Vec<Paragraph>
where
    F: FnMut(char, u32, &Font) -> Option<Font>,
{
    missing_font_owner()
}

fn make_at_coords(_: &Font, _: &[(u32, f32)]) -> Font {
    missing_font_owner()
}

fn cap_height(_: &Font, _: f32) -> f32 {
    missing_font_owner()
}

fn x_height(_: &Font, _: f32) -> f32 {
    missing_font_owner()
}

fn missing_font_owner() -> ! {
    panic!("Rust has no standalone owner corresponding to the upstream public Font API")
}

fn axis_value(font: &Font, tag: u32) -> f32 {
    font.axis_values
        .iter()
        .find_map(|(candidate, value)| (*candidate == tag).then_some(*value))
        .unwrap_or_else(|| missing_font_owner())
}

fn tag_to_string(tag: u32) -> String {
    tag.to_be_bytes().into_iter().map(char::from).collect()
}

#[test]
#[ignore = "expected-red: Rust has no standalone public Font inspection owner"]
fn inspect_font_styles() {
    let test_cases = [
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
    for (font_path, expected_weight, expected_italic) in test_cases {
        let font = load_font(font_path);
        assert_eq!(font.weight, expected_weight, "{font_path}");
        assert_eq!(font.italic, expected_italic, "{font_path}");
    }
}

#[test]
#[ignore = "expected-red: Rust has no standalone public Font metrics owner"]
fn font_exposes_cap_and_x_height_for_vertical_trim() {
    for path in [
        "assets/fonts/Inter_18pt-Regular.ttf",
        "assets/Montserrat.ttf",
    ] {
        let font = load_font(path);
        let metrics = &font.line_metrics;
        assert!(metrics.cap_height < 0.0, "{path}");
        assert!(metrics.cap_height >= metrics.ascent, "{path}");
        assert!(metrics.x_height > metrics.cap_height, "{path}");
        assert!(metrics.x_height < 0.0, "{path}");
        assert!((cap_height(&font, 20.0) - metrics.cap_height * 20.0).abs() <= f32::EPSILON);
        assert!((x_height(&font, 20.0) - metrics.x_height * 20.0).abs() <= f32::EPSILON);
    }
}

#[test]
#[ignore = "expected-red: Rust has no standalone public Font fallback owner"]
fn fallback_glyphs_are_found() {
    let mut fallback_fonts = Vec::<Font>::new();
    assert!(fallback_fonts.is_empty());
    let font = load_font("assets/RobotoFlex.ttf");
    let fallback_font = load_font("assets/IBMPlexSansArabic-Regular.ttf");
    fallback_fonts.push(fallback_font);

    let mut paragraphs = shape_text_with_fallback(
        &font,
        "لمفاتيح ABC DEF",
        32.0,
        |missing, fallback_index, _| {
            if fallback_index > 0 {
                return None;
            }
            fallback_fonts
                .iter()
                .skip(fallback_index as usize)
                .find(|fallback| has_glyph(fallback, missing))
                .cloned()
        },
    );
    assert_eq!(paragraphs.len(), 1);
    paragraphs = Vec::new();
    assert!(paragraphs.is_empty());
    fallback_fonts.clear();
}

#[test]
#[ignore = "expected-red: Rust has no standalone public Font variation-axis owner"]
fn variable_axis_values_can_be_read() {
    let fallback_fonts = Vec::<Font>::new();
    assert!(fallback_fonts.is_empty());
    let font = load_font("assets/RobotoFlex.ttf");

    let mut has_weight = false;
    for axis in &font.axes {
        if axis.tag == 2_003_265_652 {
            assert_eq!(axis.default, 400.0);
            has_weight = true;
            break;
        }
    }
    assert!(has_weight);
    assert_eq!(axis_value(&font, 2_003_265_652), 400.0);
    assert_eq!(axis_value(&font, 2_003_072_104), 100.0);

    let varied = make_at_coords(&font, &[(2_003_265_652, 800.0)]);
    assert_eq!(axis_value(&varied, 2_003_265_652), 800.0);
    let varied_twice = make_at_coords(&varied, &[(2_003_072_104, 122.0)]);
    assert_eq!(axis_value(&varied_twice, 2_003_072_104), 122.0);
    assert_eq!(axis_value(&varied_twice, 2_003_265_652), 800.0);
}

#[test]
#[ignore = "expected-red: Rust has no standalone public Font feature owner"]
fn font_features_load_as_expected() {
    let fallback_fonts = Vec::<Font>::new();
    assert!(fallback_fonts.is_empty());
    let font = load_font("assets/RobotoFlex.ttf");
    let feature_strings = font
        .features
        .iter()
        .copied()
        .map(tag_to_string)
        .collect::<Vec<_>>();
    assert_eq!(font.features.len(), 7);
    assert!(feature_strings.iter().any(|tag| tag == "mkmk"));
    assert!(feature_strings.iter().any(|tag| tag == "kern"));
    assert!(feature_strings.iter().any(|tag| tag == "rvrn"));
    assert!(feature_strings.iter().any(|tag| tag == "mark"));
    assert!(feature_strings.iter().any(|tag| tag == "locl"));
    assert!(feature_strings.iter().any(|tag| tag == "pnum"));
    assert!(feature_strings.iter().any(|tag| tag == "liga"));
}
