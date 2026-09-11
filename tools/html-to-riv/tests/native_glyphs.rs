#![cfg(all(target_os = "macos", feature = "native-glyph-controls"))]
use nuxie_render_api::{
    BlendMode, GlyphFontRef, GlyphFontVariation, Mat2D, PositionedGlyph, RenderGlyphRun,
};
use nuxie_renderer::glyph_rasterizer::rasterize;
use std::sync::Arc;

fn font() -> Arc<[u8]> {
    Arc::from(include_bytes!("assets/Inter-Regular.ttf").as_slice())
}
fn glyph(bytes: &[u8], ch: char) -> u16 {
    ttf_parser::Face::parse(bytes, 0)
        .unwrap()
        .glyph_index(ch)
        .unwrap()
        .0
}
fn run<'a>(bytes: &'a Arc<[u8]>, glyphs: &'a [PositionedGlyph]) -> RenderGlyphRun<'a> {
    RenderGlyphRun {
        font: GlyphFontRef {
            bytes,
            face_index: 0,
            variations: &[],
        },
        glyphs,
        font_size: 20.0,
        color: 0xff000000,
        blend_mode: BlendMode::SrcOver,
    }
}
const IDENTITY: Mat2D = Mat2D([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
#[test]
fn bounded_masks_preserve_integer_translation_and_fractional_phase() {
    let bytes = font();
    let glyphs = [PositionedGlyph {
        id: glyph(&bytes, 'H'),
        x: 0.25,
        y: 22.25,
    }];
    let run = run(&bytes, &glyphs);
    let a = rasterize(&run, IDENTITY).unwrap();
    let b = rasterize(&run, Mat2D([1.0, 0.0, 0.0, 1.0, 100.0, -70.0])).unwrap();
    assert_eq!((b.x, b.y), (a.x + 100, a.y - 70));
    assert_eq!((a.width, a.height), (b.width, b.height));
    assert_eq!(a.rgba, b.rgba);
    assert!(a.width < 30 && a.height < 30);
    assert!(a.rgba.chunks_exact(4).any(|p| p[3] > 0));
    // A subpixel change must survive mask cropping, rather than snapping text.
    let c = rasterize(&run, Mat2D([1.0, 0.0, 0.0, 1.0, 0.5, 0.0])).unwrap();
    assert_ne!(a.rgba, c.rgba);
}
#[test]
fn masks_preserve_color_and_apply_alpha_once() {
    let bytes = font();
    let glyphs = [PositionedGlyph {
        id: glyph(&bytes, 'H'),
        x: 0.0,
        y: 22.0,
    }];
    let mut run = run(&bytes, &glyphs);
    run.color = 0x80336699;
    let mask = rasterize(&run, IDENTITY).unwrap();
    let pixel = mask.rgba.chunks_exact(4).max_by_key(|p| p[3]).unwrap();
    assert_eq!(pixel[3], 128);
    for (actual, expected) in pixel[..3].iter().zip([51i16, 102, 153]) {
        assert!((i16::from(*actual) - expected).abs() <= 1);
    }
    assert_eq!(mask.rgba[3], 0);
}
#[test]
fn unsupported_and_excessive_requests_are_declined() {
    let bytes = font();
    let glyphs = [PositionedGlyph {
        id: glyph(&bytes, 'H'),
        x: 0.0,
        y: 22.0,
    }];
    let mut request = run(&bytes, &glyphs);
    request.font.face_index = 1;
    assert!(rasterize(&request, IDENTITY).is_none());
    request.font.face_index = 0;
    let variations = [GlyphFontVariation {
        tag: u32::from_be_bytes(*b"wght"),
        value: 400.0,
    }];
    request.font.variations = &variations;
    assert!(rasterize(&request, IDENTITY).is_none());
    request.font.variations = &[];
    request.font_size = f32::NAN;
    assert!(rasterize(&request, IDENTITY).is_none());
    request.font_size = 20.0;
    assert!(rasterize(&request, Mat2D([10000.0, 0.0, 0.0, 10000.0, 0.0, 0.0])).is_none());
    assert!(rasterize(&request, Mat2D([0.0; 6])).is_none());
    let bad = [PositionedGlyph {
        id: u16::MAX,
        x: 0.0,
        y: 0.0,
    }];
    request.glyphs = &bad;
    assert!(rasterize(&request, IDENTITY).is_none());
    let many = vec![glyphs[0]; 16_385];
    request.glyphs = &many;
    assert!(rasterize(&request, IDENTITY).is_none());
}
#[test]
fn empty_ink_does_not_allocate_a_viewport() {
    let bytes = font();
    let glyphs = [PositionedGlyph {
        id: glyph(&bytes, ' '),
        x: 9000.0,
        y: 9000.0,
    }];
    let mask = rasterize(&run(&bytes, &glyphs), IDENTITY).unwrap();
    assert_eq!((mask.width, mask.height, mask.rgba.len()), (0, 0, 0));
}

#[test]
fn malformed_font_declines_without_releasing_a_null_cf_object() {
    let bytes: Arc<[u8]> = Arc::from([0u8; 16]);
    assert!(rasterize(&run(&bytes, &[]), IDENTITY).is_none());
}

#[test]
fn axis_aligned_baselines_snap_in_device_space_without_snapping_horizontal_phase() {
    let bytes = font();
    let mut a = [PositionedGlyph {
        id: glyph(&bytes, 'H'),
        x: 0.25,
        y: 22.25,
    }];
    let mut b = a;
    b[0].y = 22.0;
    let aa = rasterize(&run(&bytes, &a), IDENTITY).unwrap();
    let bb = rasterize(&run(&bytes, &b), IDENTITY).unwrap();
    assert_eq!((aa.x, aa.y, aa.rgba), (bb.x, bb.y, bb.rgba));
    // The same rule is applied after scale and translation, not in local space.
    a[0].y = 22.25;
    b[0].y = 22.4;
    let scale = Mat2D([1.25, 0.0, 0.0, 1.25, 0.0, 0.0]);
    let aa = rasterize(&run(&bytes, &a), scale).unwrap();
    let bb = rasterize(&run(&bytes, &b), scale).unwrap();
    assert_eq!((aa.x, aa.y, aa.rgba), (bb.x, bb.y, bb.rgba));
}
