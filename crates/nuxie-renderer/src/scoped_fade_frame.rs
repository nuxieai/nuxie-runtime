//! The frame T2 and T3 read, on live Metal and live Vulkan: an opaque green
//! square fading three ways over a backdrop that is a draw, not the clear. The
//! prepass can hide a draw; the clear it can't.
//!
//! The oracle is CSS arithmetic at alpha 128/255: 128 x (1 - 128/255) reads 64,
//! and 255 x 128/255 + 64 reads 192.

use nuxie_render_api::{
    BlendMode, Factory, FillRule, ImageFilter, ImageSampler, Mat2D, RawPath, RenderCanvas,
    RenderCanvasFrameMode, RenderImage, Renderer,
};

pub(crate) const SIZE: u32 = 200;
/// Opaque black, so a square that blends over the clear reads wrong.
pub(crate) const CLEAR: u32 = 0xff00_0000;
const GRAY: u32 = 0xff80_8080;
const GREEN: u32 = 0xff00_ff00;
const GREEN_AT_128: u32 = 0x8000_ff00;
const SQUARE: u32 = 40;
const TOP: f32 = 80.0;
const LEFT_X: f32 = 20.0;
const MIDDLE_X: f32 = 80.0;
const RIGHT_X: f32 = 140.0;

fn rectangle(left: f32, top: f32, right: f32, bottom: f32) -> RawPath {
    let mut raw = RawPath::new();
    raw.move_to(left, top);
    raw.line_to(right, top);
    raw.line_to(right, bottom);
    raw.line_to(left, bottom);
    raw.close();
    raw
}

fn square(x: f32) -> RawPath {
    rectangle(x, TOP, x + SQUARE as f32, TOP + SQUARE as f32)
}

/// A canvas of the square's size filled with opaque green, drawn in `mode`.
pub(crate) fn make_picture(
    factory: &mut dyn Factory,
    mode: RenderCanvasFrameMode,
) -> Box<dyn RenderCanvas> {
    let mut canvas = factory
        .make_render_canvas(SQUARE, SQUARE)
        .expect("the picture's canvas");
    let path = factory.make_render_path(
        rectangle(0.0, 0.0, SQUARE as f32, SQUARE as f32),
        FillRule::NonZero,
    );
    let mut paint = factory.make_render_paint();
    paint.color(GREEN);
    let mut frame = canvas
        .begin_frame_with_mode(0, mode)
        .expect("the picture's frame");
    frame.draw_path(path.as_ref(), paint.as_ref());
    frame.finish().expect("the picture's finish");
    canvas
}

/// The backdrop, then the fallback, M2's way and the picture.
pub(crate) fn draw(
    factory: &mut dyn Factory,
    renderer: &mut dyn Renderer,
    picture: &dyn RenderImage,
) {
    let backdrop = factory.make_render_path(rectangle(10.0, 10.0, 190.0, 190.0), FillRule::NonZero);
    let mut gray = factory.make_render_paint();
    gray.color(GRAY);
    renderer.draw_path(backdrop.as_ref(), gray.as_ref());

    // The fallback: an opaque paint under a scope.
    let left = factory.make_render_path(square(LEFT_X), FillRule::NonZero);
    let mut green = factory.make_render_paint();
    green.color(GREEN);
    renderer.save();
    renderer.modulate_opacity(0.5);
    renderer.draw_path(left.as_ref(), green.as_ref());
    renderer.restore();

    // The control, M2's way: the paint's own color carries the fade.
    let middle = factory.make_render_path(square(MIDDLE_X), FillRule::NonZero);
    let mut carried = factory.make_render_paint();
    carried.color(GREEN_AT_128);
    renderer.draw_path(middle.as_ref(), carried.as_ref());

    // The picture, composited at the fade.
    renderer.save();
    renderer.transform(Mat2D([1.0, 0.0, 0.0, 1.0, RIGHT_X, TOP]));
    renderer.draw_image(
        Some(picture),
        ImageSampler {
            filter: ImageFilter::Nearest,
            ..Default::default()
        },
        BlendMode::SrcOver,
        0.5,
    );
    renderer.restore();
}

/// Each read that is off by more than 1, naming `what`. Every read is printed,
/// so a failing run shows the whole frame. An expected alpha of 255 is read
/// only when `read_alpha`.
pub(crate) fn wrong_reads(pixels: &[u8], what: &str, read_alpha: bool) -> Vec<String> {
    assert_eq!(pixels.len(), (SIZE * SIZE * 4) as usize, "{what}");
    let faded = [64, 192, 64];
    let mut wrong = Vec::new();
    for (name, (x, y), expected) in [
        ("left, the fallback", (40, 100), faded),
        ("middle, M2's way", (100, 100), faded),
        ("right, the picture", (160, 100), faded),
        ("the backdrop between squares", (70, 100), [128, 128, 128]),
        ("the corner, the clear", (5, 5), [0, 0, 0]),
    ] {
        let i = (y * SIZE as usize + x) * 4;
        let read = &pixels[i..i + 4];
        eprintln!("{what}: {name} at ({x}, {y}) reads {read:?}, expected {expected:?}");
        let color_is_off = (0..3).any(|c| read[c].abs_diff(expected[c]) > 1);
        if color_is_off || (read_alpha && read[3] != 255) {
            wrong.push(format!(
                "{what}: {name} at ({x}, {y}) reads {read:?}, expected {expected:?}"
            ));
        }
    }
    wrong
}
