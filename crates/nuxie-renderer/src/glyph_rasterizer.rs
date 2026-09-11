//! Opt-in macOS font masks for already shaped text. No shaping, layout, file
//! access, or compiler dependency lives here. The default renderers do not use it.
use nuxie_render_api::{BlendMode, Mat2D, RenderGlyphRun};
use std::{ffi::c_void, ptr};

/// A tightly bounded, device-aligned image with straight sRGB RGBA pixels.
/// Composite at (x, y) with unit scale, the caller's clip and opacity 1.
#[derive(Debug)]
pub struct GlyphMask {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

const MAX_GLYPHS: usize = 16_384;
const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
const MAX_SIDE: f64 = 8192.0;
const MAX_MASK_BYTES: usize = 16 * 1024 * 1024;

/// Rasterize one solid, horizontal-font run in device coordinates. `None`
/// declines the entire request; no destination renderer is ever modified.
/// This first profile accepts face zero, no variation coordinates, SrcOver,
/// and finite nonsingular affine transforms. Empty ink returns an empty mask.
/// Masks are bounded to 16 MiB and 8192px per side; fonts to 32 MiB and runs to
/// 16384 glyphs. Cache ownership belongs to the installing host adapter.
pub fn rasterize(run: &RenderGlyphRun<'_>, transform: Mat2D) -> Option<GlyphMask> {
    let m = transform.0.map(f64::from);
    if run.font.face_index != 0
        || !run.font.variations.is_empty()
        || run.font.bytes.is_empty()
        || run.font.bytes.len() > MAX_FONT_BYTES
        || run.blend_mode != BlendMode::SrcOver
        || !run.font_size.is_finite()
        || run.font_size <= 0.0
        || run.font_size > 4096.0
        || run.glyphs.len() > MAX_GLYPHS
        || !m.iter().all(|v| v.is_finite() && v.abs() <= 1e7)
        || (m[0] * m[3] - m[1] * m[2]).abs() < 1e-12
        || !run
            .glyphs
            .iter()
            .all(|g| g.x.is_finite() && g.y.is_finite())
    {
        return None;
    }
    // Chromium/Skia snaps the orthogonal coordinate of axis-aligned horizontal
    // glyph masks to device pixels while retaining horizontal subpixel phase.
    // Snap each final baseline, not the text box or its responsive layout.
    // SkGlyphPositionRoundingSpec::HalfAxisSampleFreq/IgnorePositionMask (kX).
    let baseline_y = |y: f32| {
        let y = f64::from(y);
        if m[1] == 0.0 && m[2] == 0.0 {
            ((m[3] * y + m[5] + 0.5).floor() - m[5]) / m[3]
        } else {
            y
        }
    };
    // All borrowed buffers outlive their CF objects. Every Create reference is
    // released by Owned, in reverse construction order, including early returns.
    unsafe {
        let provider = Owned::new(CGDataProviderCreateWithData(
            ptr::null_mut(),
            run.font.bytes.as_ptr().cast(),
            run.font.bytes.len(),
            None,
        ))?;
        let graphics_font = Owned::new(CGFontCreateWithDataProvider(provider.0))?;
        let font = Owned::new(CTFontCreateWithGraphicsFont(
            graphics_font.0,
            f64::from(run.font_size),
            ptr::null(),
            ptr::null_mut(),
        ))?;
        let glyph_count = CTFontGetGlyphCount(font.0);
        let mut bounds = [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ];
        for glyph in run.glyphs {
            if glyph.id as isize >= glyph_count {
                return None;
            }
            let rect = CTFontGetBoundingRectsForGlyphs(font.0, 1, &glyph.id, ptr::null_mut(), 1);
            if ![
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            ]
            .iter()
            .all(|v| v.is_finite())
            {
                return None;
            }
            if rect.size.width <= 0.0 || rect.size.height <= 0.0 {
                continue;
            }
            for x in [rect.origin.x, rect.origin.x + rect.size.width] {
                for y in [rect.origin.y, rect.origin.y + rect.size.height] {
                    let x = x + f64::from(glyph.x);
                    let y = baseline_y(glyph.y) - y;
                    let dx = m[0] * x + m[2] * y + m[4];
                    let dy = m[1] * x + m[3] * y + m[5];
                    bounds[0] = bounds[0].min(dx);
                    bounds[1] = bounds[1].min(dy);
                    bounds[2] = bounds[2].max(dx);
                    bounds[3] = bounds[3].max(dy);
                }
            }
        }
        if bounds[0] == f64::INFINITY {
            return Some(GlyphMask {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                rgba: Vec::new(),
            });
        }
        // Device-pixel padding preserves antialiasing outside design bounds.
        let left = bounds[0].floor() - 2.0;
        let top = bounds[1].floor() - 2.0;
        let width = bounds[2].ceil() + 2.0 - left;
        let height = bounds[3].ceil() + 2.0 - top;
        if ![left, top, width, height].iter().all(|v| v.is_finite())
            || left.abs() > 1e7
            || top.abs() > 1e7
            || !(1.0..=MAX_SIDE).contains(&width)
            || !(1.0..=MAX_SIDE).contains(&height)
        {
            return None;
        }
        let width = width as usize;
        let height = height as usize;
        let bytes = width.checked_mul(height)?.checked_mul(4)?;
        if bytes > MAX_MASK_BYTES {
            return None;
        }
        let mut rgba = vec![0u8; bytes];
        let color_space = Owned::new(CGColorSpaceCreateWithName(kCGColorSpaceSRGB))?;
        let context = Owned::new(CGBitmapContextCreate(
            rgba.as_mut_ptr().cast(),
            width,
            height,
            8,
            width * 4,
            color_space.0,
            (4 << 12) | 1, // byteOrder32Big | premultipliedLast
        ))?;
        CGContextConcatCTM(
            context.0,
            Affine {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: -1.0,
                tx: -left,
                ty: height as f64 + top,
            },
        );
        CGContextConcatCTM(
            context.0,
            Affine {
                a: m[0],
                b: m[1],
                c: m[2],
                d: m[3],
                tx: m[4],
                ty: m[5],
            },
        );
        CGContextSetAllowsAntialiasing(context.0, true);
        CGContextSetShouldAntialias(context.0, true);
        CGContextSetAllowsFontSmoothing(context.0, true);
        CGContextSetShouldSmoothFonts(context.0, true);
        CGContextSetAllowsFontSubpixelPositioning(context.0, true);
        CGContextSetShouldSubpixelPositionFonts(context.0, true);
        CGContextSetAllowsFontSubpixelQuantization(context.0, false);
        CGContextSetShouldSubpixelQuantizeFonts(context.0, false);
        let c = run.color;
        CGContextSetRGBFillColor(
            context.0,
            f64::from((c >> 16) & 255) / 255.0,
            f64::from((c >> 8) & 255) / 255.0,
            f64::from(c & 255) / 255.0,
            f64::from(c >> 24) / 255.0,
        );
        for glyph in run.glyphs {
            CGContextSaveGState(context.0);
            CGContextConcatCTM(
                context.0,
                Affine {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: -1.0,
                    tx: f64::from(glyph.x),
                    ty: baseline_y(glyph.y),
                },
            );
            CGContextSetTextMatrix(
                context.0,
                Affine {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    tx: 0.0,
                    ty: 0.0,
                },
            );
            CTFontDrawGlyphs(font.0, &glyph.id, &Point { x: 0.0, y: 0.0 }, 1, context.0);
            CGContextRestoreGState(context.0);
        }
        drop(context); // release all access to rgba before reading/moving it
        for pixel in rgba.chunks_exact_mut(4) {
            let alpha = u32::from(pixel[3]);
            if alpha != 0 {
                for channel in &mut pixel[..3] {
                    *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
                }
            }
        }
        Some(GlyphMask {
            x: left as i32,
            y: top as i32,
            width: width as u32,
            height: height as u32,
            rgba,
        })
    }
}

// Minimal ABI surface taken from the installed macOS SDK CoreText/CoreGraphics
// headers. CGFloat is double on supported 64-bit macOS targets. All objects
// below are CF types and obey Create/CFRelease ownership, including CGContext.
type Ref = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    origin: Point,
    size: Size,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}
struct Owned(Ref);
impl Owned {
    fn new(value: Ref) -> Option<Self> {
        if value.is_null() {
            None
        } else {
            Some(Self(value))
        }
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0);
        }
    }
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(value: Ref);
}
#[link(name = "CoreText", kind = "framework")]
extern "C" {
    fn CTFontCreateWithGraphicsFont(
        font: Ref,
        size: f64,
        matrix: *const Affine,
        attributes: Ref,
    ) -> Ref;
    fn CTFontGetGlyphCount(font: Ref) -> isize;
    fn CTFontGetBoundingRectsForGlyphs(
        font: Ref,
        orientation: u32,
        glyphs: *const u16,
        rects: *mut Rect,
        count: isize,
    ) -> Rect;
    fn CTFontDrawGlyphs(
        font: Ref,
        glyphs: *const u16,
        positions: *const Point,
        count: usize,
        context: Ref,
    );
}
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    static kCGColorSpaceSRGB: Ref;
    fn CGDataProviderCreateWithData(
        info: Ref,
        data: *const c_void,
        size: usize,
        release: Option<unsafe extern "C" fn(Ref, *const c_void, usize)>,
    ) -> Ref;
    fn CGFontCreateWithDataProvider(provider: Ref) -> Ref;
    fn CGColorSpaceCreateWithName(name: Ref) -> Ref;
    fn CGBitmapContextCreate(
        data: Ref,
        width: usize,
        height: usize,
        bits: usize,
        stride: usize,
        space: Ref,
        info: u32,
    ) -> Ref;
    fn CGContextConcatCTM(context: Ref, matrix: Affine);
    fn CGContextSetTextMatrix(context: Ref, matrix: Affine);
    fn CGContextSetRGBFillColor(context: Ref, r: f64, g: f64, b: f64, a: f64);
    fn CGContextSaveGState(context: Ref);
    fn CGContextRestoreGState(context: Ref);
    fn CGContextSetAllowsAntialiasing(context: Ref, value: bool);
    fn CGContextSetShouldAntialias(context: Ref, value: bool);
    fn CGContextSetAllowsFontSmoothing(context: Ref, value: bool);
    fn CGContextSetShouldSmoothFonts(context: Ref, value: bool);
    fn CGContextSetAllowsFontSubpixelPositioning(context: Ref, value: bool);
    fn CGContextSetShouldSubpixelPositionFonts(context: Ref, value: bool);
    fn CGContextSetAllowsFontSubpixelQuantization(context: Ref, value: bool);
    fn CGContextSetShouldSubpixelQuantizeFonts(context: Ref, value: bool);
}
