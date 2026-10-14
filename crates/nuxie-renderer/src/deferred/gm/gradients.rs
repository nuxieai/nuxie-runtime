//! Literal scenes from `tests/gm/gradients.cpp` at 4672b88b.
use super::clipstrokes::rounded_rect;
use super::ore_gm_helper::assert_cpp_gm_pixels_with_size;
use crate::{
    RenderMode,
    native_metal::{NativeMetalContextOptions, NativeMetalFactory, ShaderCompilationMode},
};
use nuxie_render_api::{
    Aabb, Factory, FillRule, ImageSampler, Mat2D, RawPath, RenderPaintStyle, Renderer, StrokeCap,
    StrokeJoin, StrokePosition,
};

const NOMOON_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/renderer/image_paint/nomoon.png"
));

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
        factory.make_angular_gradient(&colors, &stops),
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
        (3, multiply(translation(0.5, 0.5), translation(0.25, 0.25))),
        (3, multiply(translation(0.5, 0.5), scale(1.0, 1.5))),
        (3, multiply(translation(0.5, 0.5), scale(1.0, 0.5))),
        (3, multiply(translation(0.5, 0.5), rotation(-angle))),
        // Upstream lists the same negative rotation twice.
        (3, multiply(translation(0.5, 0.5), rotation(-angle))),
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
        // Upstream's release build contracts these positioning expressions.
        // Preserve that arithmetic for non-integral grid cell widths.
        renderer.translate(
            ((i % x_count) as f32 + 0.5).mul_add(space_width, -(width * 0.5)),
            ((i / x_count) as f32 + 0.5).mul_add(space_height, -(height * 0.5)),
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

#[test]
fn gradienttypes() {
    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        2048,
        1536,
        RenderMode::RasterOrdering,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..Default::default()
        },
    )
    .expect("Metal GM factory");
    let colors2 = [0xff881177, 0xff22ccbb];
    let stops2 = [0.0, 1.0];
    let colors5 = [0xff99dd55, 0xff00bbcc, 0xff881177, 0xffcc6666, 0xffeedd00];
    let stops5 = [0.0, 0.25, 0.5, 0.75, 1.0];
    let gradients = [
        factory.make_linear_gradient(-100.0, -100.0, 100.0, 100.0, &colors2, &stops2),
        factory.make_radial_gradient(0.0, 0.0, 100.0, &colors2, &stops2),
        factory.make_angular_gradient(&colors2, &stops2),
        factory.make_linear_gradient(-100.0, -100.0, 100.0, 100.0, &colors5, &stops5),
        factory.make_radial_gradient(0.0, 0.0, 100.0, &colors5, &stops5),
        factory.make_angular_gradient(&colors5, &stops5),
    ];
    // The existing helper literally mirrors gmutils.cpp PathBuilder::RRect,
    // including the upstream cubic control-point arithmetic.
    let path = factory.make_render_path(
        rounded_rect(-110.0, -110.0, 110.0, 110.0, 40.0),
        FillRule::Clockwise,
    );
    let image = factory.decode_image(NOMOON_PNG).expect("nomoon.png decode");
    let mut renderer = factory.begin_frame(0xff404040).expect("GM frame");
    for draw_type_index in 0..8 {
        let use_image = draw_type_index & 1 != 0;
        let is_stroke = draw_type_index & 2 != 0;
        let is_feather = draw_type_index & 4 != 0;
        for (gradient_index, gradient) in gradients.iter().enumerate() {
            renderer.save();
            renderer.transform(translation(
                draw_type_index as f32 * 256.0 + 128.0,
                gradient_index as f32 * 256.0 + 128.0,
            ));
            let mut paint = factory.make_render_paint();
            paint.shader(Some(gradient.as_ref()));
            if is_stroke {
                paint.style(RenderPaintStyle::Stroke);
                paint.thickness(10.0);
                paint.join(StrokeJoin::Miter);
                paint.cap(StrokeCap::Butt);
                paint.stroke_position(StrokePosition::Center);
            }
            if is_feather {
                paint.feather(20.0);
            }
            if use_image {
                paint.modulated_image(
                    Some(image.as_ref()),
                    ImageSampler::default(),
                    multiply(translation(-110.0, -110.0), scale(220.0, 220.0)),
                );
            }
            renderer.draw_path(path.as_ref(), paint.as_ref());
            renderer.restore();
        }
    }
    let pixels = renderer.finish().expect("flush GM");
    assert_cpp_gm_pixels_with_size("gradienttypes", 2048, 1536, pixels);
}
