//! Literal scene from `tests/gm/gradienttransform.cpp` at 7aa93402.
use super::ore_gm_helper::assert_cpp_gm_pixels_with_size;
use crate::{
    native_metal::{NativeMetalContextOptions, NativeMetalFactory, ShaderCompilationMode},
    RenderMode,
};
use nuxie_render_api::{Aabb, Factory, FillRule, Mat2D, RawPath, RenderPaintStyle, Renderer};

fn translation(x: f32, y: f32) -> Mat2D {
    Mat2D([1.0, 0.0, 0.0, 1.0, x, y])
}
fn scale(x: f32, y: f32) -> Mat2D {
    Mat2D([x, 0.0, 0.0, y, 0.0, 0.0])
}
fn rotation(angle: f32) -> Mat2D {
    Mat2D([
        angle.cos(),
        angle.sin(),
        -angle.sin(),
        angle.cos(),
        0.0,
        0.0,
    ])
}

#[test]
fn gradienttransform() {
    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        1600,
        800,
        RenderMode::RasterOrdering,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..Default::default()
        },
    )
    .expect("Metal GM factory");
    let mut double_square = RawPath::new();
    double_square.add_rect(Aabb::new(0.0, 0.0, 2.0, 2.0));
    let double_square = factory.make_render_path(double_square, FillRule::NonZero);
    let mut rect = factory.make_empty_render_path();
    rect.add_render_path(double_square.as_ref(), scale(0.5, 0.5));
    let colors = [0xffff0000, 0xff00ff00];
    let stops = [0.0, 1.0];
    let shaders = [
        factory.make_linear_gradient(0.0, 0.0, 1.0, 0.0, &colors, &stops),
        factory.make_linear_gradient(0.0, 0.0, 0.0, 1.0, &colors, &stops),
        factory.make_radial_gradient(0.5, 0.5, 0.5, &colors, &stops),
    ];
    let mut paints = shaders
        .iter()
        .map(|shader| {
            let mut paint = factory.make_render_paint();
            paint.shader(Some(shader.as_ref()));
            paint.thickness(0.2);
            paint
        })
        .collect::<Vec<_>>();
    let angle = std::f32::consts::PI * 0.125;
    let iterations = [
        (0, translation(0.25, 0.0)),
        (0, translation(-0.25, 0.0)),
        (0, scale(1.5, 1.0)),
        (0, scale(0.5, 1.0)),
        (0, rotation(angle)),
        (0, rotation(-angle)),
        (1, translation(0.0, 0.25)),
        (1, translation(0.0, -0.25)),
        (1, scale(1.0, 1.5)),
        (1, scale(1.0, 0.5)),
        (1, rotation(angle)),
        (1, rotation(-angle)),
        (2, translation(0.25, 0.0)),
        (2, translation(-0.25, 0.0)),
        (2, translation(0.0, 0.25)),
        (2, translation(0.0, -0.25)),
        (2, scale(1.5, 1.0)),
        (2, scale(0.5, 1.0)),
        (2, scale(1.0, 1.5)),
        (2, scale(1.0, 0.5)),
        (2, rotation(angle)),
        (2, rotation(-angle)),
    ];
    let count = iterations.len() * 2;
    let x_count = ((count as f64 * (1600.0 / 800.0)).sqrt().ceil()) as usize;
    let y_count = (count + x_count - 1) / x_count;
    let space_width = 1600.0 / x_count as f32;
    let space_height = 800.0 / y_count as f32;
    let width = space_width * 0.75;
    let height = space_height * 0.75;
    let _ = factory
        .begin_frame(0xff000000)
        .expect("clear GM")
        .finish()
        .expect("flush clear");
    let mut pixels = Vec::new();
    for i in 0..count {
        let mut renderer = factory
            .begin_frame_preserving()
            .expect("preserved GM frame");
        renderer.save();
        renderer.translate(
            ((i % x_count) as f32 + 0.5) * space_width - width * 0.5,
            ((i / x_count) as f32 + 0.5) * space_height - height * 0.5,
        );
        renderer.scale(width, height);
        let (paint, transform) = iterations[i / 2];
        let paint = &mut paints[paint];
        paint.style(if i % 2 == 0 {
            RenderPaintStyle::Fill
        } else {
            RenderPaintStyle::Stroke
        });
        paint.shader_transform(transform);
        renderer.draw_path(rect.as_ref(), paint.as_ref());
        renderer.restore();
        pixels = renderer.finish().expect("flush GM cell");
    }
    assert_cpp_gm_pixels_with_size("gradienttransform", 1600, 800, pixels);
}
