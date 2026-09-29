//! `testing_window.hpp::FrameOptions` / `gm.cpp::GM::run` mode scope and
//! the two `uber_parity*::onDraw` loops at upstream 7e0b60b0.
//! The existing GM host is Metal: its inherited testing setter echoes the
//! requested mode and DOES NOT change pipelines. These are live scene tests,
//! not evidence of GL/Vulkan uber-versus-specialized pixel equivalence.

pub(super) use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    DitherMode, ShaderCompilationMode,
};
use crate::{native_metal::NativeMetalFactory, RenderMode};
pub(super) use nuxie_render_api::*;

pub(super) const WIDTH: u32 = 600;
pub(super) const HEIGHT: u32 = 300;
pub(super) const CELL: f32 = 300.0;
pub(super) const NOMOON: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/renderer/image_paint/nomoon.png"
));

#[derive(Default)]
struct FrameOptions {
    shader_compilation_mode: ShaderCompilationMode,
}

pub(super) fn rect(
    factory: &mut dyn Factory,
    renderer: &mut dyn Renderer,
    bounds: Aabb,
    paint: &dyn RenderPaint,
) {
    let mut raw = RawPath::new();
    raw.add_rect(bounds);
    renderer.draw_path(
        factory.make_render_path(raw, FillRule::NonZero).as_ref(),
        paint,
    );
}

pub(super) fn circle(
    factory: &mut dyn Factory,
    renderer: &mut dyn Renderer,
    radius: f32,
    paint: &dyn RenderPaint,
) {
    // RawPath's four cubics use gmutils.cpp's exact 0.551915... constant,
    // clockwise direction and rightmost starting point.
    let mut raw = RawPath::new();
    raw.add_oval(Aabb::new(
        150.0 - radius,
        150.0 - radius,
        150.0 + radius,
        150.0 + radius,
    ));
    renderer.draw_path(
        factory.make_render_path(raw, FillRule::NonZero).as_ref(),
        paint,
    );
}

pub(super) fn run(
    draw_scene: fn(&mut dyn Factory, &mut dyn Renderer, f32),
    dither: DitherMode,
) -> Vec<u8> {
    let mut lifecycle = Vec::new();
    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        WIDTH,
        HEIGHT,
        RenderMode::RasterOrdering,
        crate::native_metal::NativeMetalContextOptions {
            shader_compilation_mode: crate::native_metal::ShaderCompilationMode::AlwaysSynchronous,
            ..Default::default()
        },
    )
    .expect("live Metal uber parity GM factory");
    factory.use_deterministic_validation_thresholds();
    let mut options = FrameOptions::default();
    // Both GMs' updateFrameOptions overrides this default.
    options.shader_compilation_mode = ShaderCompilationMode::onlyUbershaders;
    let previous_mode = if options.shader_compilation_mode != ShaderCompilationMode::standard {
        lifecycle.push("scope:onlyUbershaders");
        Some(
            factory
                .testing_only_set_shader_compilation_mode(ShaderCompilationMode::onlyUbershaders),
        )
    } else {
        None
    };
    let mut renderer = factory.begin_frame(0xff404040).expect("GM beginFrame");
    lifecycle.push("begin:clear");
    renderer.save(); // GM::draw, preserved across both context-frame breaks.
    for i in 0..2 {
        renderer.flush_gm_frame().expect("GM flushPLSContext");
        lifecycle.push("flush");
        factory.testing_only_set_shader_compilation_mode(if i == 0 {
            ShaderCompilationMode::onlyUbershaders
        } else {
            ShaderCompilationMode::alwaysSynchronous
        });
        lifecycle.push(if i == 0 {
            "mode:onlyUbershaders"
        } else {
            "mode:alwaysSynchronous"
        });
        renderer
            .begin_gm_frame_preserving(dither)
            .expect("GM beginFrame preserveRenderTarget");
        lifecycle.push("begin:preserve");
        draw_scene(&mut factory, &mut renderer, i as f32 * CELL);
        lifecycle.push(if i == 0 { "draw:left" } else { "draw:right" });
    }
    renderer.restore();
    let pixels = renderer.finish().expect("GM endFrame");
    lifecycle.push("end");
    if let Some(previous_mode) = previous_mode {
        factory.testing_only_set_shader_compilation_mode(previous_mode);
        lifecycle.push("scope:restore");
    }
    assert_eq!(
        lifecycle,
        [
            "scope:onlyUbershaders",
            "begin:clear",
            "flush",
            "mode:onlyUbershaders",
            "begin:preserve",
            "draw:left",
            "flush",
            "mode:alwaysSynchronous",
            "begin:preserve",
            "draw:right",
            "end",
            "scope:restore"
        ]
    );
    pixels
}

pub(super) fn assert_scene_rendered(pixels: &[u8]) {
    assert_eq!(pixels.len(), (WIDTH * HEIGHT * 4) as usize);
    // Basic execution qualification, not an independently sourced golden.
    for x in [20, 320] {
        let base = ((100 * WIDTH + x) * 4) as usize;
        assert_ne!(&pixels[base..base + 3], &[64, 64, 64]);
    }
}
