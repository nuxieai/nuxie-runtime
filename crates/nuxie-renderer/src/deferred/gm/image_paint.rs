//! `tests/gm/image_paint.cpp` at `3ed35ee0ded0d58fb8d380930a156041a4624a2f`.

use crate::{
    native_metal::{NativeMetalContextOptions, NativeMetalFactory, ShaderCompilationMode},
    RenderMode,
};
use nuxie_render_api::{
    Aabb, Factory, FillRule, ImageFilter, ImageSampler, ImageWrap, Mat2D, RawPath, RenderPaint,
    RenderPaintStyle, Renderer,
};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const BATDUDE_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/renderer/image_paint/batdude.png"
));
const NOMOON_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/renderer/image_paint/nomoon.png"
));
const IMAGE_PAINT_REFERENCE_PNG: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/renderer/reference/metal/gm/image_paint.png"
);

fn multiply(lhs: Mat2D, rhs: Mat2D) -> Mat2D {
    let a = lhs.0;
    let b = rhs.0;
    Mat2D([
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ])
}

fn scale(sx: f32, sy: f32) -> Mat2D {
    Mat2D([sx, 0.0, 0.0, sy, 0.0, 0.0])
}

fn translate(tx: f32, ty: f32) -> Mat2D {
    Mat2D([1.0, 0.0, 0.0, 1.0, tx, ty])
}

fn rotation(radians: f32) -> Mat2D {
    let sin = radians.sin();
    let cos = radians.cos();
    Mat2D([cos, sin, -sin, cos, 0.0, 0.0])
}

fn draw_rect(
    factory: &mut dyn Factory,
    renderer: &mut dyn Renderer,
    bounds: Aabb,
    paint: &dyn RenderPaint,
) {
    let mut raw = RawPath::new();
    raw.add_rect(bounds);
    let path = factory.make_render_path(raw, FillRule::NonZero);
    renderer.draw_path(path.as_ref(), paint);
}

fn draw_oval(
    factory: &mut dyn Factory,
    renderer: &mut dyn Renderer,
    bounds: Aabb,
    paint: &dyn RenderPaint,
) {
    let mut raw = RawPath::new();
    raw.add_oval(bounds);
    let path = factory.make_render_path(raw, FillRule::NonZero);
    renderer.draw_path(path.as_ref(), paint);
}

fn render_image_paint() -> Vec<u8> {
    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        WIDTH,
        HEIGHT,
        RenderMode::RasterOrdering,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..NativeMetalContextOptions::default()
        },
    )
    .expect("live Metal image_paint GM factory");
    let mut renderer = factory
        .begin_frame(0)
        .expect("live Metal image_paint GM frame");
    let _r = Aabb::new(0.0, 0.0, 250.0, 250.0);

    {
        let mut paint = factory.make_render_paint();
        paint.color(0xff2f2f2f);
        draw_rect(
            &mut factory,
            &mut renderer,
            Aabb::new(0.0, 0.0, 512.0, 512.0),
            paint.as_ref(),
        );
    }

    renderer.save();
    renderer.translate(256.0, 256.0);

    let image0 = factory
        .decode_image(BATDUDE_PNG)
        .expect("upstream image_paint batdude.png decode");
    let image1 = factory
        .decode_image(NOMOON_PNG)
        .expect("upstream image_paint nomoon.png decode");

    {
        let mut paint = factory.make_render_paint();
        paint.color(0xffb299ff);
        let gradient_colors = [0xffff9b9b, 0xffc5ff8c, 0xff70a6ff];
        let gradient_stops = [0.0, 0.5, 1.0];
        paint.modulated_image(
            Some(image0.as_ref()),
            ImageSampler {
                wrap_x: ImageWrap::Mirror,
                wrap_y: ImageWrap::Repeat,
                filter: ImageFilter::Bilinear,
            },
            multiply(
                multiply(scale(128.0, 128.0), translate(0.5, 0.5)),
                rotation(45.0 * std::f32::consts::PI / 180.0),
            ),
        );

        let gradient = factory.make_linear_gradient(
            0.0,
            -100.0,
            0.0,
            100.0,
            &gradient_colors,
            &gradient_stops,
        );
        paint.shader(Some(gradient.as_ref()));

        draw_oval(
            &mut factory,
            &mut renderer,
            Aabb::new(-220.0, -220.0, 220.0, 220.0),
            paint.as_ref(),
        );

        paint.modulated_image(
            Some(image1.as_ref()),
            ImageSampler {
                wrap_x: ImageWrap::Repeat,
                wrap_y: ImageWrap::Mirror,
                filter: ImageFilter::Bilinear,
            },
            scale(128.0, 128.0),
        );
        paint.color(0xffffa5f3);
        paint.style(RenderPaintStyle::Stroke);
        paint.thickness(30.0);
        paint.feather(30.0);
        draw_oval(
            &mut factory,
            &mut renderer,
            Aabb::new(-220.0, -220.0, 220.0, 220.0),
            paint.as_ref(),
        );
    }
    renderer.restore();
    renderer
        .finish()
        .expect("live Metal image_paint GM readback")
}

#[test]
fn image_paint() {
    let expected = pixel_compare::RgbaImage::read_png(IMAGE_PAINT_REFERENCE_PNG)
        .expect("authoritative upstream C++ image_paint reference");
    let actual = pixel_compare::RgbaImage::new(WIDTH, HEIGHT, render_image_paint())
        .expect("live Rust Metal image_paint frame");
    let report = pixel_compare::compare(&expected, &actual, pixel_compare::Tolerance::EXACT)
        .expect("image_paint dimensions");
    assert!(
        report.within_tolerance,
        "image_paint differs from upstream C++ reference: {} pixels, max channel delta {}",
        report.different_pixels, report.max_channel_delta
    );
}
