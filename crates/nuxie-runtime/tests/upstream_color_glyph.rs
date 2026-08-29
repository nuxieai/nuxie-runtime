//! Direct ports of all 22 pinned color_glyph_test.cpp cases, plus the retained
//! Rust color-render regression using a character in the fixture's repertoire.
//! Queries and repeated layer extraction use one retained native Font owner;
//! shaping and rendering use translated text owners and the approved backend.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    factory::RuntimeFactoryHandle,
    text::{font_hb::HbFont, raw_text::RawText},
    text_engine::{
        ColorGlyphLayer, ColorGlyphPaintType, Font, FontRef, Paragraph, TextRun, TextSizing,
        with_host_fallback_proc,
    },
};
use std::{cell::RefCell, path::PathBuf};

thread_local! {
    static FALLBACK: RefCell<Option<FontRef>> = const { RefCell::new(None) };
}

fn with_fallback<R>(font: FontRef, work: impl FnOnce() -> R) -> R {
    fn pick(_: u32, index: u32, _: &dyn Font) -> Option<FontRef> {
        if index > 0 {
            return None;
        }
        FALLBACK.with(|font| font.borrow().clone())
    }
    struct Restore(Option<FontRef>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FALLBACK.with(|font| *font.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(FALLBACK.with(|slot| slot.replace(Some(font))));
    with_host_fallback_proc(pick, work)
}

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

fn font(bytes: &[u8]) -> FontRef {
    HbFont::decode(bytes).expect("font decodes")
}

fn append_layers(
    font: &FontRef,
    glyph_id: u16,
    foreground: u32,
    destination: &mut Vec<ColorGlyphLayer>,
) -> usize {
    font.get_color_layers(glyph_id, destination, foreground)
}

fn color_layers(font: &FontRef, glyph_id: u16, foreground: u32) -> Vec<ColorGlyphLayer> {
    let mut layers = Vec::new();
    font.get_color_layers(glyph_id, &mut layers, foreground);
    layers
}

fn solid_color(layer: &ColorGlyphLayer) -> Option<u32> {
    (layer.paint_type == ColorGlyphPaintType::Solid).then_some(layer.color)
}

fn shape(font: &FontRef, text: &str) -> Vec<Paragraph> {
    let unichars = text.chars().map(u32::from).collect::<Vec<_>>();
    let runs = [TextRun {
        font: Some(font.clone()),
        size: 32.0,
        line_height: -1.0,
        letter_spacing: 0.0,
        unichar_count: unichars.len() as u32,
        script: 0,
        style_id: 0,
        level: 0,
    }];
    font.shape_text(&unichars, &runs, -1)
}

fn render(text: &str, font: &FontRef, size: f32, width: f32) -> Vec<&'static str> {
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut factory = PersistentFactory::new(factory);
    let mut raw_text = RawText::new(
        RuntimeFactoryHandle::from_factory(&mut factory).expect("explicit retained test factory"),
    );
    raw_text.set_max_width(width);
    raw_text.set_sizing(TextSizing::AutoHeight);
    raw_text.append(text, None, font.clone(), size, -1.0, 0.0, 0xff00_0000);
    raw_text.render(&mut renderer, None);
    raw_text.debug_command_kinds()
}

#[test]
fn non_emoji_font_reports_no_color_glyphs() {
    let bytes = regular_bytes();
    let font = font(&bytes);
    assert!(!font.has_color_glyphs());
    assert!(!font.is_color_glyph(0));
    let mut layers = Vec::new();
    let count = append_layers(&font, 0, 0xff00_0000, &mut layers);
    assert_eq!(count, 0);
    assert!(layers.is_empty());
}

#[test]
fn emoji_font_reports_color_glyphs() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    assert!(font.has_color_glyphs());
}

#[test]
fn is_color_glyph_returns_false_for_non_color_glyph_in_emoji_font() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    assert!(!font.is_color_glyph(0));
}

#[test]
fn known_color_glyph_ids_are_detected_in_subset_font() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    assert!(font.has_color_glyphs());
    assert!(font.is_color_glyph(2));
    assert!(font.is_color_glyph(3));
}

#[test]
fn get_color_layers_returns_layers_for_a_color_glyph() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    let mut layers = Vec::new();
    let count = append_layers(&font, 2, 0xff00_0000, &mut layers);
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
    let font = font(&bytes);
    let mut layers = Vec::new();
    let count = append_layers(&font, 0, 0xff00_0000, &mut layers);
    assert_eq!(count, 0);
    assert!(layers.is_empty());
}

#[test]
fn color_glyph_layers_are_cached() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    let color_glyph = (1..100)
        .find(|glyph| font.is_color_glyph(*glyph))
        .expect("known color glyph");
    let mut first = Vec::new();
    let first_count = append_layers(&font, color_glyph, 0xff00_0000, &mut first);
    let mut second = Vec::new();
    let second_count = append_layers(&font, color_glyph, 0xff00_0000, &mut second);
    assert_eq!(first_count, second_count);
    assert_eq!(first.len(), second.len());
}

#[test]
fn foreground_color_is_applied_for_ffff_color_index() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    let color_glyph = (1..100)
        .find(|glyph| font.is_color_glyph(*glyph))
        .expect("known color glyph");
    let black = color_layers(&font, color_glyph, 0xff00_0000);
    let red = color_layers(&font, color_glyph, 0xffff_0000);
    assert_eq!(black.len(), red.len());
    let mut has_foreground_layer = false;
    for (black, red) in black.iter().zip(&red) {
        if black.use_foreground {
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
    assert_eq!(
        font.as_any().downcast_ref::<HbFont>().unwrap().face_index(),
        0
    );
    assert!(font.has_color_glyphs());
    let sub_font = font.with_options(&[], &[]);
    assert!(sub_font.has_color_glyphs());
}

#[test]
fn raw_text_renders_with_color_glyph_font_without_crashing() {
    let emoji = font(&emoji_bytes());
    let _ = render("A", &emoji, 32.0, 200.0);
}

#[test]
fn shaping_emoji_font_produces_glyphs() {
    let emoji = font(&emoji_bytes());
    let paragraphs = shape(&emoji, "A");
    assert_eq!(paragraphs.len(), 1);
    assert!(!paragraphs[0].runs.is_empty());
    assert!(!paragraphs[0].runs[0].glyphs.is_empty());
}

#[test]
fn shaping_with_fallback_uses_emoji_font_for_missing_glyphs() {
    let emoji = font(&emoji_bytes());
    let regular = font(&regular_bytes());
    with_fallback(emoji, || {
        let paragraphs = shape(&regular, "A❤B");
        assert_eq!(paragraphs.len(), 1);
        assert!(paragraphs[0].runs.iter().any(|run| {
            run.font
                .as_ref()
                .expect("shaped run retains font")
                .has_color_glyphs()
        }));
    });
}

#[test]
fn supported_fallback_color_glyph_emits_color_draw_command() {
    let emoji = font(&emoji_bytes());
    let regular = font(&regular_bytes());
    // The pinned heart string checks font selection: this subset has no heart.
    // U+3297 maps to its color glyph 2 and is absent from the regular font.
    assert!(!regular.has_glyph(0x3297));
    assert!(emoji.has_glyph(0x3297));
    assert!(emoji.is_color_glyph(2));
    with_fallback(emoji, || {
        let commands = render("A㊗B", &regular, 32.0, 400.0);
        assert!(commands.contains(&"color"));
    });
}

#[test]
fn colrv0_layers_have_solid_paint_type() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    let layers = color_layers(&font, 2, 0xff00_0000);
    assert!(!layers.is_empty());
    for layer in layers {
        assert!(matches!(layer.paint_type, ColorGlyphPaintType::Solid));
        assert!(layer.stops.is_empty());
        assert!(layer.image_bytes.is_empty());
    }
}

#[test]
fn all_known_color_glyph_ids_return_layers() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    for glyph in [2, 3] {
        let mut layers = Vec::new();
        let count = append_layers(&font, glyph, 0xff00_0000, &mut layers);
        assert!(count > 0);
        assert_eq!(count, layers.len());
    }
}

#[test]
fn cached_layers_update_foreground_color_correctly() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    let first = color_layers(&font, 2, 0xff00_0000);
    let green = color_layers(&font, 2, 0xff00_ff00);
    assert_eq!(first.len(), green.len());
    for (first, green) in first.iter().zip(&green) {
        if first.use_foreground {
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
    let font = font(&bytes);
    for glyph in 0..10 {
        let mut layers = Vec::new();
        let count = append_layers(&font, glyph, 0xff00_0000, &mut layers);
        assert_eq!(count, 0);
        assert!(layers.is_empty());
    }
}

#[test]
fn get_color_layers_appends_to_existing_vector() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    let mut layers = Vec::new();
    let first = append_layers(&font, 2, 0xff00_0000, &mut layers);
    assert!(first > 0);
    assert_eq!(layers.len(), first);
    let second = append_layers(&font, 3, 0xff00_0000, &mut layers);
    assert!(second > 0);
    assert_eq!(layers.len(), first + second);
}

#[test]
fn is_color_glyph_is_false_for_high_glyph_ids() {
    let bytes = emoji_bytes();
    let font = font(&bytes);
    assert!(!font.is_color_glyph(9999));
    assert!(!font.is_color_glyph(65535));
}

#[test]
fn raw_text_with_multiple_color_glyphs_renders_without_crashing() {
    let emoji = font(&emoji_bytes());
    let _ = render("❤❤❤", &emoji, 32.0, 400.0);
}

#[test]
fn raw_text_with_mixed_regular_and_emoji_text_renders_without_crashing() {
    let emoji = font(&emoji_bytes());
    let regular = font(&regular_bytes());
    with_fallback(emoji, || {
        let _ = render("Hello ❤ World", &regular, 32.0, 400.0);
    });
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
