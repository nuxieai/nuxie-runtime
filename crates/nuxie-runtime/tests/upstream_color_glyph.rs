//! Direct ports of all 22 original color_glyph_test.cpp cases, four additions
//! at caeae0e723f646ee853461b61e18e04a3081fc1d, plus the retained
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

fn colrv1_font() -> FontRef {
    font(&asset("NotoColorEmojiCOLRv1.subset.ttf"))
}

fn catch_approx(actual: f32, expected: f32) {
    assert!(
        (f64::from(actual) - f64::from(expected)).abs()
            <= f64::from(100.0 * f32::EPSILON) * f64::from(expected).abs(),
        "{actual} != Approx({expected})"
    );
}

#[test]
fn colrv1_radial_gradient_maps_to_glyph_space() {
    let font = colrv1_font();
    let mut layers = Vec::new();
    assert_eq!(font.get_color_layers(2, &mut layers, 0xff000000), 10);
    let face = &layers[0];
    assert_eq!(face.paint_type, ColorGlyphPaintType::RadialGradient);
    assert_eq!(face.stops.len(), 3);
    assert_eq!(face.stops[0].color, 0xFFFDE030);
    catch_approx(face.x1, 630.0 / 1024.0);
    catch_approx(face.y1, -360.0 / 1024.0);
    catch_approx(face.r1, 534.0 / 1024.0);
    assert!(
        face.path
            .bounds()
            .contains(nuxie_runtime::source::math::vec2d::Vec2D::new(
                face.x1, face.y1
            ))
    );
}

#[test]
fn colrv1_transformed_clip_glyphs_keep_their_transform() {
    let font = colrv1_font();
    let mut layers = Vec::new();
    assert_eq!(font.get_color_layers(2, &mut layers, 0xff000000), 10);
    let left = layers[2].path.bounds();
    let right = layers[4].path.bounds();
    catch_approx(right.left() - left.left(), (317.625 + 41.906) / 1024.0);
    catch_approx(right.top(), left.top());
}

#[test]
fn colrv1_radial_gradients_keep_their_ellipse() {
    use nuxie_runtime::source::math::vec2d::Vec2D;
    let font = colrv1_font();
    let mut layers = Vec::new();
    assert!(font.get_color_layers(3, &mut layers, 0xff000000) > 0);
    let mut saw_ellipse = false;
    for layer in layers {
        if layer.paint_type != ColorGlyphPaintType::RadialGradient {
            continue;
        }
        let m = layer.radial_transform;
        let center = m * Vec2D::new(0.0, 0.0);
        catch_approx(center.x, layer.x1);
        catch_approx(center.y, layer.y1);
        let sx = Vec2D::new(m[0], m[1]).length();
        let sy = Vec2D::new(m[2], m[3]).length();
        saw_ellipse |= (sx - sy).abs() > 0.2 * sx.max(sy);
    }
    assert!(saw_ellipse);
}

#[test]
fn raw_text_draws_colrv1_gradient_layers_with_gradient_shaders() {
    let font = colrv1_font();
    // The existing recording factory counts the same allocation calls as
    // upstream's GradientCountingFactory, without introducing another factory.
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let mut raw_text =
        RawText::new(RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"));
    raw_text.append("🎉😀", None, font, 32.0, -1.0, 0.0, 0xff000000);
    let mut renderer = nuxie_render_api::NullRenderer::new();
    raw_text.render(&mut renderer, None);
    let stream = factory.borrow().stream();
    assert_eq!(
        stream
            .lines()
            .filter(|line| line.starts_with("makeLinearGradient "))
            .count(),
        1
    );
    assert_eq!(
        stream
            .lines()
            .filter(|line| line.starts_with("makeRadialGradient "))
            .count(),
        1
    );
}

// These numbers are computed from the pinned HarfBuzz sbix.hh get_png_extents,
// CBDT.hh get_extents, and hb-font.hh scale_glyph_extents, then Rive's
// paint_image gInvScale. The existing C++ probe does not expose image extents.
fn harfbuzz_bitmap_asset(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_HARFBUZZ_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(
            "/Users/levi/dev/worktrees/rive-runtime-pin-183df53f/dependencies/rive-app_harfbuzz_rive_13.1.1",
        ));
    std::fs::read(root.join(relative)).expect("pinned HarfBuzz bitmap font")
}

fn table_offset(bytes: &[u8], tag: &[u8; 4]) -> usize {
    let count = u16::from_be_bytes(bytes[4..6].try_into().unwrap()) as usize;
    (0..count)
        .map(|index| 12 + index * 16)
        .find_map(|offset| {
            (&bytes[offset..offset + 4] == tag).then(|| {
                u32::from_be_bytes(bytes[offset + 8..offset + 12].try_into().unwrap()) as usize
            })
        })
        .expect("font table")
}

fn sbix_strike(bytes: &[u8], index: usize) -> usize {
    let table = table_offset(bytes, b"sbix");
    table
        + u32::from_be_bytes(
            bytes[table + 8 + index * 4..table + 12 + index * 4]
                .try_into()
                .unwrap(),
        ) as usize
}

fn sbix_glyph(bytes: &[u8], strike: usize, glyph: usize) -> usize {
    strike
        + u32::from_be_bytes(
            bytes[strike + 4 + glyph * 4..strike + 8 + glyph * 4]
                .try_into()
                .unwrap(),
        ) as usize
}

fn image_extents(layer: &ColorGlyphLayer) -> [f32; 4] {
    assert_eq!(layer.paint_type, ColorGlyphPaintType::Image);
    [
        layer.image_bearing_x,
        layer.image_bearing_y,
        layer.image_extent_x,
        layer.image_extent_y,
    ]
}

#[test]
fn sbix_picture_uses_top_edge_and_ignores_glyf_outer_bearings() {
    let bytes = harfbuzz_bitmap_asset("test/api/fonts/chromacheck-sbix.ttf");
    let font = font(&bytes);
    let layers = color_layers(&font, 1, 0xff000000);
    assert_eq!(layers.len(), 1);
    assert_eq!((layers[0].image_width, layers[0].image_height), (300, 300));
    assert_eq!(image_extents(&layers[0]), [0.0, -1.0, 1.0, 1.0]);
}

#[test]
fn sbix_picture_chooses_largest_strike_and_rounds_font_unit_endpoints() {
    let bytes = harfbuzz_bitmap_asset("test/fuzzing/fonts/sbix.ttf");
    let font = font(&bytes);
    let layers = color_layers(&font, 1, 0xff000000);
    assert_eq!(layers.len(), 1);
    assert_eq!((layers[0].image_width, layers[0].image_height), (632, 737));
    assert_eq!(
        image_extents(&layers[0]),
        [0.0, -1475.0 / 2048.0, 1264.0 / 2048.0, 1475.0 / 2048.0]
    );
    // Repeated extraction must retain exactly the same rounded placement.
    assert_eq!(
        image_extents(&color_layers(&font, 1, 0xffffffff)[0]),
        image_extents(&layers[0])
    );
}

#[test]
fn sbix_picture_does_not_fall_back_when_largest_strike_lacks_glyph() {
    let mut bytes = harfbuzz_bitmap_asset("test/fuzzing/fonts/sbix.ttf");
    let strike = sbix_strike(&bytes, 8);
    let next = bytes[strike + 12..strike + 16].to_vec();
    bytes[strike + 8..strike + 12].copy_from_slice(&next);
    let font = font(&bytes);
    assert!(!font.is_color_glyph(1));
    assert!(color_layers(&font, 1, 0xff000000).is_empty());
    assert!(font.is_color_glyph(2));
}

#[test]
fn sbix_picture_keeps_first_strike_when_largest_ppem_ties() {
    let mut bytes = harfbuzz_bitmap_asset("test/fuzzing/fonts/sbix.ttf");
    let strike = sbix_strike(&bytes, 7);
    bytes[strike..strike + 2].copy_from_slice(&1024u16.to_be_bytes());
    let layers = color_layers(&font(&bytes), 1, 0xff000000);
    assert_eq!((layers[0].image_width, layers[0].image_height), (316, 369));
}

#[test]
fn sbix_dupe_uses_target_picture_and_target_offsets() {
    let mut bytes = harfbuzz_bitmap_asset("test/fuzzing/fonts/sbix.ttf");
    let strike = sbix_strike(&bytes, 8);
    let glyph = sbix_glyph(&bytes, strike, 1);
    bytes[glyph..glyph + 4].copy_from_slice(&[0, 80, 0, 90]);
    bytes[glyph + 4..glyph + 8].copy_from_slice(b"dupe");
    bytes[glyph + 8..glyph + 10].copy_from_slice(&2u16.to_be_bytes());
    let font = font(&bytes);
    let source = color_layers(&font, 1, 0xff000000);
    let target = color_layers(&font, 2, 0xff000000);
    assert_eq!(source.len(), 1);
    assert_eq!((source[0].image_width, source[0].image_height), (630, 737));
    assert_eq!(source[0].image_bytes, target[0].image_bytes);
    assert_eq!(image_extents(&source[0]), image_extents(&target[0]));
}

#[test]
fn sbix_dupe_cycle_stops_without_a_picture() {
    let mut bytes = harfbuzz_bitmap_asset("test/fuzzing/fonts/sbix.ttf");
    let strike = sbix_strike(&bytes, 8);
    let glyph = sbix_glyph(&bytes, strike, 1);
    bytes[glyph + 4..glyph + 8].copy_from_slice(b"dupe");
    bytes[glyph + 8..glyph + 10].copy_from_slice(&1u16.to_be_bytes());
    let font = font(&bytes);
    assert!(!font.is_color_glyph(1));
    assert!(color_layers(&font, 1, 0xff000000).is_empty());
}

#[test]
fn sbix_picture_applies_signed_origin_offsets_before_two_stage_rounding() {
    let mut bytes = harfbuzz_bitmap_asset("test/api/fonts/chromacheck-sbix.ttf");
    let strike = sbix_strike(&bytes, 0);
    let glyph = sbix_glyph(&bytes, strike, 1);
    bytes[glyph..glyph + 2].copy_from_slice(&(-1i16).to_be_bytes());
    bytes[glyph + 2..glyph + 4].copy_from_slice(&(-1i16).to_be_bytes());
    let layers = color_layers(&font(&bytes), 1, 0xff000000);
    // roundf(-800/300)=-3; fixed 2048/800 scaling maps -3 to -8,
    // and roundf(299*800/300)=797 maps to 2040.
    assert_eq!(
        image_extents(&layers[0]),
        [-8.0 / 2048.0, -2040.0 / 2048.0, 1.0, 1.0]
    );
}

#[test]
fn cbdt_picture_preserves_harfbuzz_rounding_order_for_nonstandard_upem() {
    let mut bytes = harfbuzz_bitmap_asset("test/api/fonts/chromacheck-cbdt.ttf");
    let head = table_offset(&bytes, b"head");
    bytes[head + 18..head + 20].copy_from_slice(&1000u16.to_be_bytes());
    let layers = color_layers(&font(&bytes), 1, 0xff000000);
    assert_eq!(layers.len(), 1);
    assert_eq!((layers[0].image_width, layers[0].image_height), (80, 80));
    // Pixel 80 -> fixed 2048/1000 scaling =164 -> roundf(164*1000/80)=2050.
    assert_eq!(
        image_extents(&layers[0]),
        [0.0, -2050.0 / 2048.0, 2050.0 / 2048.0, 2050.0 / 2048.0]
    );
}

#[test]
fn cbdt_png_length_is_clamped_to_table_after_validating_indexed_entry() {
    let mut bytes = harfbuzz_bitmap_asset("test/api/fonts/chromacheck-cbdt.ttf");
    let baseline = color_layers(&font(&bytes), 1, 0xff000000);
    let cblc = table_offset(&bytes, b"CBLC");
    // Keep the fixed metrics+PNG-length header indexed, but leave the PNG
    // bytes outside the entry. HB's reference_png sub-blob uses table bounds.
    bytes[cblc + 76..cblc + 80].copy_from_slice(&9u32.to_be_bytes());
    let font = font(&bytes);
    assert!(font.is_color_glyph(1));
    let layers = color_layers(&font, 1, 0xff000000);
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].image_bytes, baseline[0].image_bytes);
    assert_eq!(image_extents(&layers[0]), image_extents(&baseline[0]));
}

#[test]
fn cbdt_unsupported_index_format_two_is_not_a_native_color_glyph() {
    let mut bytes = harfbuzz_bitmap_asset("test/api/fonts/chromacheck-cbdt.ttf");
    let cblc = table_offset(&bytes, b"CBLC");
    // Convert the single format-1 index to a valid format-2 index. skrifa
    // supports it, but the pinned HB get_image_data supports only 1 and 3.
    bytes.extend_from_slice(&[0; 4]);
    bytes[40..44].copy_from_slice(&84u32.to_be_bytes()); // CBLC directory length
    bytes[cblc + 12..cblc + 16].copy_from_slice(&28u32.to_be_bytes());
    bytes[cblc + 64..cblc + 66].copy_from_slice(&2u16.to_be_bytes());
    bytes[cblc + 72..cblc + 76].copy_from_slice(&97u32.to_be_bytes());
    bytes[cblc + 76..cblc + 84].copy_from_slice(&[80, 80, 0, 80, 80, 0, 0, 0]);
    let font = font(&bytes);
    assert!(!font.is_color_glyph(1));
    assert!(color_layers(&font, 1, 0xff000000).is_empty());
}

// UNIV-4921 additive host picture boundary. A 1x1 raster represents one em;
// dimensions describe pixels, bearings/extents describe normalized geometry.
fn host_picture() -> nuxie_runtime::source::text::font_hb::HostGlyphPicture {
    use nuxie_runtime::source::text::font_hb::HostGlyphPicture;
    HostGlyphPicture {
        png: vec![
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4,
            0, 9, 251, 3, 253, 251, 94, 107, 43, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
        ],
        width: 1,
        height: 1,
        bearing_x: 0.125,
        bearing_y: -0.875,
        extent_x: 1.0,
        extent_y: 1.0,
    }
}

#[test]
fn host_picture_answers_are_once_per_glyph_and_cluster_across_threads() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let plain = font(&regular_bytes());
    let glyph = shape(&plain, "A")[0].runs[0].glyphs[0];
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let host = plain
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(move |_, cluster| {
            count.fetch_add(1, Ordering::SeqCst);
            assert_eq!(cluster, &[u32::from('A')]);
            Some(host_picture())
        }));
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let host = host.clone();
            scope.spawn(move || assert!(host.is_color_glyph_with_text(glyph, &[65])));
        }
    });
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(
        !host.is_color_glyph(glyph),
        "glyph-only query is native and order independent"
    );
    let mut layers = Vec::new();
    host.get_color_layers_with_text(glyph, &[65], &mut layers, 0xff000000);
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].paint_type, ColorGlyphPaintType::Image);
    assert_eq!(layers[0].image_bearing_y, -0.875);
    let count = calls.clone();
    let absent = plain
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(move |_, _| {
            count.fetch_add(1, Ordering::SeqCst);
            None
        }));
    assert!(!absent.is_color_glyph_with_text(glyph, &[65]));
    assert!(!absent.is_color_glyph_with_text(glyph, &[66]));
    assert!(!absent.is_color_glyph_with_text(glyph, &[66]));
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(
        absent.get_path(glyph).verbs(),
        plain.get_path(glyph).verbs()
    );
    assert_eq!(
        absent.get_path(glyph).points(),
        plain.get_path(glyph).points()
    );
}

#[test]
fn native_color_never_asks_host_and_host_sources_compose() {
    use std::sync::Arc;
    let native = colrv1_font();
    let hosted = native
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(|_, _| panic!("native COLR wins")));
    assert!(hosted.is_color_glyph_with_text(2, &[0x1f600]));
    assert_eq!(color_layers(&hosted, 2, 0xff000000).len(), 10);
    let plain = font(&regular_bytes());
    for picture_first in [false, true] {
        let hb = plain.as_any().downcast_ref::<HbFont>().unwrap();
        let first = if picture_first {
            hb.with_host_pictures(Arc::new(|_, _| Some(host_picture())))
        } else {
            hb.with_host_outline(Arc::new(|_, _| Default::default()))
        };
        let hb = first.as_any().downcast_ref::<HbFont>().unwrap();
        let both = if picture_first {
            hb.with_host_outline(Arc::new(|_, _| Default::default()))
        } else {
            hb.with_host_pictures(Arc::new(|_, _| Some(host_picture())))
        };
        assert!(both.is_color_glyph_with_text(7, &[65]));
        assert!(
            both.with_options(&[], &[])
                .is_color_glyph_with_text(7, &[65])
        );
    }
}

#[test]
fn host_cluster_text_retains_ligatures_combining_marks_and_rtl() {
    use std::sync::Arc;
    let plain = font(&regular_bytes());
    let host = plain
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(|_, _| None));
    for text in ["ffi", "A\u{301}", "אב", "A\nB"] {
        let paragraphs = shape(&host, text);
        let points: Vec<u32> = text.chars().map(u32::from).collect();
        for p in paragraphs {
            for run in p.runs {
                for i in 0..run.glyphs.len() {
                    let start = run.text_indices[i] as usize;
                    let cluster = run.cluster_text(i);
                    assert!(!cluster.is_empty());
                    assert_eq!(cluster, &points[start..start + cluster.len()]);
                    if cluster.contains(&10) {
                        // The shaper retains the paragraph separator as its
                        // own glyph; it must not leak into a neighbour's cluster.
                        assert_eq!(cluster, &[10]);
                    }
                }
            }
        }
    }
    let ligature = shape(&host, "ffi");
    assert!(
        ligature[0]
            .runs
            .iter()
            .any(|r| r.cluster_text(0) == [102, 102, 105])
    );
}

#[test]
fn raw_text_draws_host_images_and_reuses_decoded_image() {
    use std::sync::Arc;
    let plain = font(&regular_bytes());
    let host = plain
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(|_, cluster| {
            assert_eq!(cluster, &[65]);
            Some(host_picture())
        }));
    let recording = RecordingFactory::new();
    let mut renderer = recording.make_renderer();
    let mut factory = PersistentFactory::new(recording);
    let mut raw = RawText::new(RuntimeFactoryHandle::from_factory(&mut factory).unwrap());
    raw.append("AA", None, host, 32.0, -1.0, 0.0, 0xff000000);
    raw.render(&mut renderer, None);
    raw.render(&mut renderer, None);
    let stream = factory.borrow().stream();
    assert_eq!(
        stream
            .lines()
            .filter(|l| l.starts_with("decodeImage "))
            .count(),
        1
    );
    assert_eq!(
        stream
            .lines()
            .filter(|l| l.starts_with("drawImage "))
            .count(),
        4
    );
    assert_eq!(raw.debug_command_kinds(), ["color", "color"]);
}

#[test]
fn ordinary_text_draws_host_picture_from_retained_shaped_cluster() {
    use nuxie_runtime::{
        File,
        source::{
            assets::font_asset::FontAsset,
            generated::{core_registry::CoreRegistry, text::text_value_run_base::TextValueRunBase},
            text::{
                text::{Text, TextValueRunHandle},
                text_style::TextStyle,
            },
        },
    };
    use std::sync::Arc;
    let recording = RecordingFactory::new();
    let mut renderer = recording.make_renderer();
    let mut factory = PersistentFactory::new(recording);
    let file = File::import(
        &asset("new_text.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let board = file.with_file(File::artboard_default).unwrap();
    let text = board.with_artboard(|b| b.find_all_handles::<Text>())[0].clone();
    let plain = font(&regular_bytes());
    let host = plain
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(|_, cluster| (cluster == [65]).then(host_picture)));
    for style in board.with_artboard(|b| b.find_all_handles::<TextStyle>()) {
        let asset = style.insert_sibling(FontAsset::default()).unwrap();
        FontAsset::set_font_occurrence(&asset, Some(host.clone()));
        TextStyle::set_asset_occurrence(&style, Some(asset));
    }
    for (i, run) in text
        .with_downcast::<Text, _>(|t| t.runs().to_vec())
        .unwrap()
        .iter()
        .enumerate()
    {
        let TextValueRunHandle::Core(run) = run else {
            panic!("imported run")
        };
        CoreRegistry::set_string_handle(
            run,
            i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
            if i == 0 { "A".into() } else { String::new() },
        );
    }
    board.update_pass(true);
    text.with_downcast_mut::<Text, _>(|t| t.draw(&mut renderer))
        .unwrap();
    let stream = factory.borrow().stream();
    assert!(
        stream.lines().any(|l| l.starts_with("drawImage ")),
        "{stream}"
    );
}

#[test]
fn ordinary_text_reuses_images_across_rebuilds_and_releases_replaced_host_fonts() {
    use nuxie_runtime::{
        File,
        source::{
            assets::font_asset::FontAsset,
            generated::{core_registry::CoreRegistry, text::text_value_run_base::TextValueRunBase},
            text::{
                text::{Text, TextValueRunHandle},
                text_style::TextStyle,
            },
        },
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Context(Arc<AtomicUsize>);
    impl Drop for Context {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let recording = RecordingFactory::new();
    let mut renderer = recording.make_renderer();
    let mut factory = PersistentFactory::new(recording);
    let file = File::import(
        &asset("new_text.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let board = file.with_file(File::artboard_default).unwrap();
    let text = board.with_artboard(|b| b.find_all_handles::<Text>())[0].clone();
    let plain = font(&regular_bytes());
    let drops = Arc::new(AtomicUsize::new(0));
    let context = Context(drops.clone());
    let host = plain
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(move |_, cluster| {
            let _keep_context = &context;
            (cluster == [65]).then(host_picture)
        }));
    let weak = Arc::downgrade(&host);
    let mut font_assets = Vec::new();
    for style in board.with_artboard(|b| b.find_all_handles::<TextStyle>()) {
        let font_asset = style.insert_sibling(FontAsset::default()).unwrap();
        FontAsset::set_font_occurrence(&font_asset, Some(host.clone()));
        TextStyle::set_asset_occurrence(&style, Some(font_asset.clone()));
        font_assets.push(font_asset);
    }
    for (i, run) in text
        .with_downcast::<Text, _>(|t| t.runs().to_vec())
        .unwrap()
        .iter()
        .enumerate()
    {
        let TextValueRunHandle::Core(run) = run else {
            panic!("imported run")
        };
        CoreRegistry::set_string_handle(
            run,
            i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
            if i == 0 { "A".into() } else { String::new() },
        );
    }
    drop(host);
    board.update_pass(true);
    text.with_downcast_mut::<Text, _>(|t| t.draw(&mut renderer))
        .unwrap();
    // Reshape the same glyph and rebuild its commands: keep the decoded image.
    text.with_downcast_mut::<Text, _>(|t| t.mark_shape_dirty())
        .unwrap();
    board.update_pass(true);
    text.with_downcast_mut::<Text, _>(|t| t.draw(&mut renderer))
        .unwrap();
    assert_eq!(
        factory
            .borrow()
            .stream()
            .lines()
            .filter(|line| line.starts_with("decodeImage "))
            .count(),
        1
    );
    assert!(weak.upgrade().is_some());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    for font_asset in font_assets {
        FontAsset::set_font_occurrence(&font_asset, Some(plain.clone()));
    }
    text.with_downcast_mut::<Text, _>(|t| t.mark_shape_dirty())
        .unwrap();
    board.update_pass(true);
    text.with_downcast_mut::<Text, _>(|t| t.draw(&mut renderer))
        .unwrap();
    assert!(
        weak.upgrade().is_none(),
        "Text's decoded image cache must release replaced fonts"
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

// Replace only cmap in an existing outline face. This font has no emoji GSUB
// ligatures or colour tables; A/B/C/D/E outlines become fixed emoji glyphs.
// Rebuilding the directory record leaves all shaping/outline tables untouched.
fn unjoined_emoji_outline_font() -> FontRef {
    let mut bytes = regular_bytes();
    let source = font(&bytes);
    let gid = |s: &str| shape(&source, s)[0].runs[0].glyphs[0];
    let mut mappings = vec![
        (0x20, gid(" ")),
        (65, gid("A")),
        (66, gid("B")),
        (0x2764, gid("E")),
        (0x1f44d, gid("A")),
        (0x1f3fd, gid("B")),
        (0x1f469, gid("C")),
        (0x1f4bb, gid("D")),
    ];
    mappings.sort_unstable();
    let mut cmap = Vec::new();
    for value in [0u16, 1, 3, 10] {
        cmap.extend_from_slice(&value.to_be_bytes());
    }
    cmap.extend_from_slice(&12u32.to_be_bytes());
    cmap.extend_from_slice(&12u16.to_be_bytes());
    cmap.extend_from_slice(&0u16.to_be_bytes());
    cmap.extend_from_slice(&(16u32 + 12 * mappings.len() as u32).to_be_bytes());
    cmap.extend_from_slice(&0u32.to_be_bytes());
    cmap.extend_from_slice(&(mappings.len() as u32).to_be_bytes());
    for (cp, glyph) in mappings {
        for value in [cp, cp, u32::from(glyph)] {
            cmap.extend_from_slice(&value.to_be_bytes());
        }
    }
    while bytes.len() % 4 != 0 {
        bytes.push(0);
    }
    let offset = bytes.len();
    bytes.extend_from_slice(&cmap);
    let count = u16::from_be_bytes(bytes[4..6].try_into().unwrap()) as usize;
    let record = (0..count)
        .map(|i| 12 + i * 16)
        .find(|i| &bytes[*i..*i + 4] == b"cmap")
        .unwrap();
    let checksum = cmap
        .chunks(4)
        .map(|word| {
            let mut data = [0; 4];
            data[..word.len()].copy_from_slice(word);
            u32::from_be_bytes(data)
        })
        .fold(0u32, u32::wrapping_add);
    bytes[record + 4..record + 8].copy_from_slice(&checksum.to_be_bytes());
    bytes[record + 8..record + 12].copy_from_slice(&(offset as u32).to_be_bytes());
    bytes[record + 12..record + 16].copy_from_slice(&(cmap.len() as u32).to_be_bytes());
    font(&bytes)
}

#[test]
fn host_clusters_replace_vs16_skin_tone_and_unjoined_zwj_once_in_every_text_owner() {
    use nuxie_runtime::{
        File,
        source::{
            assets::font_asset::FontAsset,
            generated::{core_registry::CoreRegistry, text::text_value_run_base::TextValueRunBase},
            text::{
                raw_text_input::RawTextInput,
                text::{Text, TextValueRunHandle},
                text_style::TextStyle,
            },
            text_engine::GlyphRenderKind,
        },
    };
    use std::sync::{Arc, Mutex};
    let plain = unjoined_emoji_outline_font();
    for text in ["❤️", "👍🏽", "👩\u{200d}💻"] {
        let points: Vec<_> = text.chars().map(u32::from).collect();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let queried = calls.clone();
        let expected = points.clone();
        let host = plain
            .as_any()
            .downcast_ref::<HbFont>()
            .unwrap()
            .with_host_pictures(Arc::new(move |glyph, cluster| {
                assert_ne!(glyph, 0);
                if cluster == [0x200b] {
                    return None;
                } // RawTextInput sentinel.
                assert_eq!(cluster, expected);
                queried.lock().unwrap().push((glyph, cluster.to_vec()));
                Some(host_picture())
            }));
        let paragraphs = shape(&host, text);
        let runs: Vec<_> = paragraphs.iter().flat_map(|p| &p.runs).collect();
        assert_eq!(runs.len(), 1, "fixture sequence must share one shaped run");
        let run = runs[0];
        assert!(
            run.glyphs.len() > 1,
            "must exercise an unjoined multi-glyph cluster: {text}"
        );
        assert!(run.glyphs.iter().all(|id| *id != 0));
        assert!(run.text_indices.iter().all(|i| *i == 0));
        assert_eq!(
            run.glyph_render_kind(run.glyphs.len() - 1),
            GlyphRenderKind::Suppressed
        );
        assert_eq!(run.glyph_render_kind(0), GlyphRenderKind::Color);
        for i in 1..run.glyphs.len() {
            assert_eq!(run.glyph_render_kind(i), GlyphRenderKind::Suppressed);
        }
        // Reading a follower first must still query the leader, never its space id.
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert_eq!(calls.lock().unwrap()[0].0, run.glyphs[0]);

        let recording = RecordingFactory::new();
        let mut renderer = recording.make_renderer();
        let mut factory = PersistentFactory::new(recording);
        let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let mut raw = RawText::new(retained.clone());
        raw.append(text, None, host.clone(), 32.0, -1.0, 0.0, 0xff123456);
        raw.render(&mut renderer, None);
        assert_eq!(raw.debug_command_kinds(), ["color"]);
        let stream = factory.borrow().stream();
        assert_eq!(
            stream
                .lines()
                .filter(|l| l.starts_with("drawImage "))
                .count(),
            1,
            "{stream}"
        );
        assert!(
            !stream.lines().any(|l| l.starts_with("drawPath ")),
            "suppressed followers must not add outlines"
        );
        let mut input = RawTextInput::new();
        input.set_font(Some(host.clone()));
        input.set_text(text.into());
        input.update(&retained);
        assert!(input.text_path().empty());
        assert!(input.selected_text_path().empty());
        let mut text_paint = retained.with_factory_mut(|f| f.make_render_paint());
        let mut selection_paint = retained.with_factory_mut(|f| f.make_render_paint());
        let mut cursor_paint = retained.with_factory_mut(|f| f.make_render_paint());
        let before = factory
            .borrow()
            .stream()
            .lines()
            .filter(|l| l.starts_with("drawImage "))
            .count();
        input.draw(
            &retained,
            &mut renderer,
            &Default::default(),
            text_paint.as_mut(),
            selection_paint.as_mut(),
            cursor_paint.as_mut(),
        );
        assert_eq!(
            factory
                .borrow()
                .stream()
                .lines()
                .filter(|l| l.starts_with("drawImage "))
                .count(),
            before + 1
        );
        let file = File::import(&asset("new_text.riv"), retained, None, None, None).unwrap();
        let board = file.with_file(File::artboard_default).unwrap();
        let owner = board.with_artboard(|b| b.find_all_handles::<Text>())[0].clone();
        for style in board.with_artboard(|b| b.find_all_handles::<TextStyle>()) {
            let asset = style.insert_sibling(FontAsset::default()).unwrap();
            FontAsset::set_font_occurrence(&asset, Some(host.clone()));
            TextStyle::set_asset_occurrence(&style, Some(asset));
        }
        // Clear every imported Text so no unrelated cluster reaches this host.
        for t in board.with_artboard(|b| b.find_all_handles::<Text>()) {
            for (i, r) in t
                .with_downcast::<Text, _>(|t| t.runs().to_vec())
                .unwrap()
                .iter()
                .enumerate()
            {
                let TextValueRunHandle::Core(r) = r else {
                    panic!("imported")
                };
                CoreRegistry::set_string_handle(
                    r,
                    i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
                    if t == owner && i == 0 {
                        text.into()
                    } else {
                        String::new()
                    },
                );
            }
        }
        board.update_pass(true);
        let before = factory
            .borrow()
            .stream()
            .lines()
            .filter(|l| l.starts_with("drawImage "))
            .count();
        owner
            .with_downcast_mut::<Text, _>(|t| t.draw(&mut renderer))
            .unwrap();
        assert_eq!(
            factory
                .borrow()
                .stream()
                .lines()
                .filter(|l| l.starts_with("drawImage "))
                .count(),
            before + 1
        );
        assert_eq!(
            calls.lock().unwrap().len(),
            1,
            "owners share one per-key answer"
        );
    }
}

#[test]
fn host_zero_is_never_queried_and_shared_glyphs_keep_distinct_cluster_answers() {
    use std::sync::{Arc, Mutex};
    let plain = unjoined_emoji_outline_font();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let observed = calls.clone();
    let host = plain
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(move |glyph, cluster| {
            assert_ne!(glyph, 0);
            observed.lock().unwrap().push((glyph, cluster.to_vec()));
            let mut p = host_picture();
            p.extent_x = cluster.len() as f32;
            Some(p)
        }));
    assert!(!host.is_color_glyph_with_text(0, &[0x1f9e1]));
    assert!(!host.is_color_glyph_with_text(0, &[0x1f4a9]));
    assert!(calls.lock().unwrap().is_empty());
    let first = shape(&host, "👍")[0].runs[0].glyphs[0];
    let second = shape(&host, "👍🏽")[0].runs[0].glyphs[0];
    assert_eq!(first, second, "same base glyph, different cluster texts");
    let plain_picture = host.host_picture_layer(first, &[0x1f44d]).unwrap();
    let toned_picture = host.host_picture_layer(first, &[0x1f44d, 0x1f3fd]).unwrap();
    assert_eq!(plain_picture.image_extent_x, 1.0);
    assert_eq!(toned_picture.image_extent_x, 2.0);
    assert_eq!(calls.lock().unwrap().len(), 2);
    let repeated = host.host_picture_layer(first, &[0x1f44d]).unwrap();
    assert!(Arc::ptr_eq(&repeated, &plain_picture));
    let mut layers = Vec::new();
    host.get_color_layers_with_text(first, &[0x1f44d], &mut layers, 0);
    assert!(
        Arc::ptr_eq(&layers[0].image_bytes, &plain_picture.image_bytes),
        "PNG storage is shared"
    );
}

#[test]
fn picture_backed_face_is_chosen_by_vs16_forced_fallback() {
    use std::sync::Arc;
    let primary = unjoined_emoji_outline_font();
    let fallback = unjoined_emoji_outline_font();
    assert!(!primary.has_color_glyphs());
    assert!(!fallback.has_color_glyphs());
    let pictures = fallback
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_host_pictures(Arc::new(|_, _| Some(host_picture())));
    assert!(pictures.has_color_glyphs());
    with_fallback(pictures.clone(), || {
        let p = shape(&primary, "❤️");
        assert!(
            p[0].runs
                .iter()
                .any(|r| Arc::ptr_eq(r.font.as_ref().unwrap(), &pictures)),
            "VS16 must select the host-picture face"
        );
    });
}

#[test]
fn cbdt_and_cblc_accept_only_major_versions_two_and_three() {
    for tag in [b"CBDT", b"CBLC"] {
        for major in [0u16, 1, 2, 3, 4, 65535] {
            let mut bytes = harfbuzz_bitmap_asset("test/api/fonts/chromacheck-cbdt.ttf");
            let table = table_offset(&bytes, tag);
            bytes[table..table + 2].copy_from_slice(&major.to_be_bytes());
            let f = font(&bytes);
            assert_eq!(
                f.has_color_glyphs(),
                matches!(major, 2 | 3),
                "{tag:?} face capability major {major}"
            );
            assert_eq!(
                f.is_color_glyph(1),
                matches!(major, 2 | 3),
                "{tag:?} major {major}"
            );
            assert_eq!(!color_layers(&f, 1, 0).is_empty(), matches!(major, 2 | 3));
        }
    }
}
