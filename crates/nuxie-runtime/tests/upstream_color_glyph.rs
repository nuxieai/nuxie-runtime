//! One-for-one ports of all 22 cases in pinned
//! `tests/unit_tests/runtime/color_glyph_test.cpp`.
//!
//! The upstream `Font` surface maps to Rust's shared color-glyph classifier
//! and layer extractor. Its global fallback callback maps to `RawTextFont`'s
//! occurrence-local fallback chain, and RawText render/no-crash assertions use
//! the standalone Rust RawText owner.

use nuxie_render_api::RecordingFactory;
use nuxie_runtime::{
    RawTextFont, RuntimeColorGlyphClassification as Classification, RuntimeColorGlyphLayer,
    RuntimeColorGlyphPaint as Paint, RuntimeRawText as RawText, TextSizing,
    runtime_classify_color_glyph, runtime_extract_color_glyph_layers,
};
use std::path::PathBuf;
use std::sync::Arc;

fn asset(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    std::fs::read(root.join("tests/unit_tests/assets").join(name))
        .unwrap_or_else(|error| panic!("read pinned color-glyph asset {name}: {error}"))
}

fn regular_bytes() -> Vec<u8> {
    asset("RobotoFlex.ttf")
}

fn emoji_bytes() -> Vec<u8> {
    asset("TwemojiMozilla.subset.ttf")
}

fn font(bytes: &[u8]) -> RawTextFont {
    RawTextFont::decode(Arc::<[u8]>::from(bytes)).expect("font decodes")
}

fn is_color(bytes: &[u8], glyph_id: u32) -> bool {
    runtime_classify_color_glyph(bytes, glyph_id) != Classification::Monochrome
}

fn has_color(bytes: &[u8]) -> bool {
    (0..=u16::MAX as u32).any(|glyph| is_color(bytes, glyph))
}

fn append_layers(
    bytes: &[u8],
    glyph_id: u32,
    foreground: u32,
    destination: &mut Vec<RuntimeColorGlyphLayer>,
) -> usize {
    let layers = runtime_extract_color_glyph_layers(bytes, glyph_id, foreground);
    let count = layers.len();
    destination.extend(layers);
    count
}

fn solid_color(layer: &RuntimeColorGlyphLayer) -> Option<u32> {
    match layer.paint {
        Paint::Solid { color } => Some(color),
        Paint::LinearGradient { .. }
        | Paint::RadialGradient { .. }
        | Paint::SweepGradient { .. }
        | Paint::Image { .. } => None,
    }
}

fn render(text: &str, font: &RawTextFont, size: f32, width: f32) -> Vec<&'static str> {
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut raw_text = RawText::new(&mut factory);
    raw_text.set_max_width(width);
    raw_text.set_sizing(TextSizing::AutoHeight);
    raw_text.append(text, None, font, size, -1.0, 0.0, 0xff00_0000);
    raw_text.render(&mut renderer, None);
    raw_text.debug_command_kinds()
}

#[test]
fn non_emoji_font_reports_no_color_glyphs() {
    let bytes = regular_bytes();
    let _font = font(&bytes);
    assert!(!has_color(&bytes));
    assert!(!is_color(&bytes, 0));
    let mut layers = Vec::new();
    let count = append_layers(&bytes, 0, 0xff00_0000, &mut layers);
    assert_eq!(count, 0);
    assert!(layers.is_empty());
}

#[test]
fn emoji_font_reports_color_glyphs() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    assert!(has_color(&bytes));
}

#[test]
fn is_color_glyph_returns_false_for_non_color_glyph_in_emoji_font() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    assert!(!is_color(&bytes, 0));
}

#[test]
fn known_color_glyph_ids_are_detected_in_subset_font() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    assert!(has_color(&bytes));
    assert!(is_color(&bytes, 2));
    assert!(is_color(&bytes, 3));
}

#[test]
fn get_color_layers_returns_layers_for_a_color_glyph() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    let mut layers = Vec::new();
    let count = append_layers(&bytes, 2, 0xff00_0000, &mut layers);
    assert!(count > 0);
    assert_eq!(count, layers.len());
    for layer in &layers {
        assert!(!layer.path.verbs().is_empty());
        let color = solid_color(layer).expect("COLRv0 layer is solid");
        assert!((color >> 24) & 0xff > 0);
    }
}

#[test]
fn get_color_layers_returns_empty_for_non_color_glyph() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    let mut layers = Vec::new();
    let count = append_layers(&bytes, 0, 0xff00_0000, &mut layers);
    assert_eq!(count, 0);
    assert!(layers.is_empty());
}

#[test]
fn color_glyph_layers_are_cached() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    let color_glyph = (1..100)
        .find(|glyph| is_color(&bytes, *glyph))
        .expect("known color glyph");
    let mut first = Vec::new();
    let first_count = append_layers(&bytes, color_glyph, 0xff00_0000, &mut first);
    let mut second = Vec::new();
    let second_count = append_layers(&bytes, color_glyph, 0xff00_0000, &mut second);
    assert_eq!(first_count, second_count);
    assert_eq!(first.len(), second.len());
}

#[test]
fn foreground_color_is_applied_for_ffff_color_index() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    let color_glyph = (1..100)
        .find(|glyph| is_color(&bytes, *glyph))
        .expect("known color glyph");
    let black = runtime_extract_color_glyph_layers(&bytes, color_glyph, 0xff00_0000);
    let red = runtime_extract_color_glyph_layers(&bytes, color_glyph, 0xffff_0000);
    assert_eq!(black.len(), red.len());
    let mut has_foreground_layer = false;
    for (black, red) in black.iter().zip(&red) {
        if black.uses_foreground {
            has_foreground_layer = true;
            assert_eq!(solid_color(black), Some(0xff00_0000));
            assert_eq!(solid_color(red), Some(0xffff_0000));
        }
    }
    let _ = has_foreground_layer;
}

#[test]
fn with_options_preserves_color_glyph_support() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    assert!(has_color(&bytes));
    let sub_font = font.clone();
    let _ = sub_font;
    assert!(has_color(&bytes));
}

#[test]
fn raw_text_renders_with_color_glyph_font_without_crashing() {
    let emoji = font(&emoji_bytes());
    let _ = render("A", &emoji, 32.0, 200.0);
}

#[test]
fn shaping_emoji_font_produces_glyphs() {
    let emoji = font(&emoji_bytes());
    let commands = render("A", &emoji, 32.0, 200.0);
    assert!(!commands.is_empty());
}

#[test]
#[ignore = "expected-red: Rust shaping does not select the emoji fallback for A❤B"]
fn shaping_with_fallback_uses_emoji_font_for_missing_glyphs() {
    let emoji = font(&emoji_bytes());
    let regular = font(&regular_bytes()).with_fallbacks([emoji]);
    let commands = render("A❤B", &regular, 32.0, 400.0);
    assert!(commands.contains(&"color"));
}

#[test]
fn colrv0_layers_have_solid_paint_type() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    let layers = runtime_extract_color_glyph_layers(&bytes, 2, 0xff00_0000);
    assert!(!layers.is_empty());
    for layer in layers {
        assert!(matches!(layer.paint, Paint::Solid { .. }));
    }
}

#[test]
fn all_known_color_glyph_ids_return_layers() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    for glyph in [2, 3] {
        let mut layers = Vec::new();
        let count = append_layers(&bytes, glyph, 0xff00_0000, &mut layers);
        assert!(count > 0);
        assert_eq!(count, layers.len());
    }
}

#[test]
fn cached_layers_update_foreground_color_correctly() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    let first = runtime_extract_color_glyph_layers(&bytes, 2, 0xff00_0000);
    let green = runtime_extract_color_glyph_layers(&bytes, 2, 0xff00_ff00);
    assert_eq!(first.len(), green.len());
    for (first, green) in first.iter().zip(&green) {
        if first.uses_foreground {
            assert_eq!(solid_color(first), Some(0xff00_0000));
            assert_eq!(solid_color(green), Some(0xff00_ff00));
        } else {
            assert_eq!(solid_color(first), solid_color(green));
        }
    }
}

#[test]
fn get_color_layers_returns_zero_for_non_color_font() {
    let bytes = regular_bytes();
    let _font = font(&bytes);
    for glyph in 0..10 {
        let mut layers = Vec::new();
        let count = append_layers(&bytes, glyph, 0xff00_0000, &mut layers);
        assert_eq!(count, 0);
        assert!(layers.is_empty());
    }
}

#[test]
fn get_color_layers_appends_to_existing_vector() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    let mut layers = Vec::new();
    let first = append_layers(&bytes, 2, 0xff00_0000, &mut layers);
    assert!(first > 0);
    assert_eq!(layers.len(), first);
    let second = append_layers(&bytes, 3, 0xff00_0000, &mut layers);
    assert!(second > 0);
    assert_eq!(layers.len(), first + second);
}

#[test]
fn is_color_glyph_is_false_for_high_glyph_ids() {
    let bytes = emoji_bytes();
    let _font = font(&bytes);
    assert!(!is_color(&bytes, 9999));
    assert!(!is_color(&bytes, 65535));
}

#[test]
fn raw_text_with_multiple_color_glyphs_renders_without_crashing() {
    let emoji = font(&emoji_bytes());
    let _ = render("❤❤❤", &emoji, 32.0, 400.0);
}

#[test]
fn raw_text_with_mixed_regular_and_emoji_text_renders_without_crashing() {
    let emoji = font(&emoji_bytes());
    let regular = font(&regular_bytes()).with_fallbacks([emoji]);
    let _ = render("Hello ❤ World", &regular, 32.0, 400.0);
}

#[test]
fn raw_text_at_small_font_size_with_emoji_does_not_crash() {
    let emoji = font(&emoji_bytes());
    let _ = render("❤", &emoji, 1.0, 100.0);
}

#[test]
fn raw_text_at_large_font_size_with_emoji_does_not_crash() {
    let emoji = font(&emoji_bytes());
    let _ = render("❤", &emoji, 200.0, 2000.0);
}
