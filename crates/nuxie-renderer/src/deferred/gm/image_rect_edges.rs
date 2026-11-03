//! An image whose edges sit on whole pixels covers every pixel of its rect,
//! its edge pixels included, in each frame mode live Metal draws. Raster
//! ordering draws an image as a unit rect path with an image paint. Atomic
//! mode, the iOS Simulator's, draws it as an `ImageRectDraw`, whose vertex
//! shader insets an inner ring of vertices to where coverage reaches 1. A
//! Screen that fades as one picture is composited this way (UNIV-5629).

use super::group_opacity_pixel::png;
use crate::native_metal::{NativeMetalContextOptions, NativeMetalFactory, ShaderCompilationMode};
use crate::RenderMode;
use nuxie_render_api::{BlendMode, Factory, ImageSampler, Renderer};

const FRAME: u32 = 300;
const WHITE: u32 = 0xffff_ffff;
/// The image's top left corner, on a whole pixel.
const AT: u32 = 10;

/// An opaque black image of `width` by `height` drawn at `AT` over white in
/// `mode`: the frame's pixels.
fn draw_black_image(mode: RenderMode, width: u32, height: u32) -> Vec<u8> {
    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        FRAME,
        FRAME,
        mode,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..NativeMetalContextOptions::default()
        },
    )
    .expect("live Metal image edge factory");
    factory.use_deterministic_validation_thresholds();
    let image = factory
        .decode_image(&png(width, height, |_, _| [0, 0, 0]))
        .expect("the black image decodes");
    let mut renderer = factory
        .begin_frame(WHITE)
        .expect("live Metal image edge frame");
    renderer.save();
    renderer.translate(AT as f32, AT as f32);
    renderer.draw_image(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
    );
    renderer.restore();
    renderer.finish().expect("live Metal image edge readback")
}

/// Every pixel of a `width` by `height` image's rect is inside the image, so
/// each reads its black within one level, the first and last rows and
/// columns as the middle does. `None` when they do; otherwise how many do
/// not, the first of them, and each edge read at its middle.
fn misread(mode: RenderMode, width: u32, height: u32) -> Option<String> {
    let pixels = draw_black_image(mode, width, height);
    let level = |x: u32, y: u32| {
        let i = ((y * FRAME + x) * 4) as usize;
        pixels[i..i + 3].iter().copied().max().unwrap_or(0)
    };
    let (mut wrong, mut first) = (0, None);
    for y in AT..AT + height {
        for x in AT..AT + width {
            let read = level(x, y);
            if read > 1 {
                wrong += 1;
                first.get_or_insert((x, y, read));
            }
        }
    }
    let (x, y, read) = first?;
    let (middle_x, middle_y) = (AT + width / 2, AT + height / 2);
    Some(format!(
        "a {width} by {height} image: {wrong} of {} pixels are not its black, the first at ({x}, {y}) reading {read}; at their middles its first row reads {}, its second row {}, its last row {}, its first column {}, its second column {}, its last column {}",
        width * height,
        level(middle_x, AT),
        level(middle_x, AT + 1),
        level(middle_x, AT + height - 1),
        level(AT, middle_y),
        level(AT + 1, middle_y),
        level(AT + width - 1, middle_y),
    ))
}

/// A tall image and a wide one, each read whole, so a failure names both.
fn assert_covers_its_edges(mode: RenderMode) {
    let misreads: Vec<String> = [(64, 256), (256, 64)]
        .into_iter()
        .filter_map(|(width, height)| misread(mode, width, height))
        .collect();
    assert!(
        misreads.is_empty(),
        "{mode:?}, each pixel of an image should read its black within one level: {}",
        misreads.join("; ")
    );
}

/// Raster ordering draws the image as a rect path with an image paint, whose
/// edges are whole.
#[test]
fn an_image_covers_its_edge_pixels_in_raster_ordering() {
    assert_covers_its_edges(RenderMode::RasterOrdering);
}

/// Clockwise atomic on Metal, which never reports clockwise atomic support,
/// runs as atomic mode, as the iOS Simulator's frames do. Main's
/// `ImageRectDraw` vertex shader insets each axis's edge by the other axis's
/// radius, so a 64 by 256 image's first and last rows read 153 and its second
/// and second to last rows 51, and a 256 by 64 image's columns likewise.
#[test]
fn an_image_covers_its_edge_pixels_in_atomic_mode() {
    assert_covers_its_edges(RenderMode::ClockwiseAtomic);
}
