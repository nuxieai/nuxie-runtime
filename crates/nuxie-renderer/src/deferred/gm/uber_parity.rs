//! Exact scene from `tests/gm/uber_parity.cpp` at 7e0b60b0.
use super::uber_gm_helper::*;

fn draw_scene(factory: &mut dyn Factory, renderer: &mut dyn Renderer, ox: f32) {
    let stops = [0.0, 1.0];
    renderer.save();
    renderer.translate(ox, 0.0);

    let base = Aabb::new(20.0, 20.0, 280.0, 280.0);
    let mut base_paint = factory.make_render_paint();
    let base_shader = factory.make_linear_gradient(
        base.min_x,
        0.0,
        base.max_x,
        0.0,
        &[0xff2060c0, 0xffc06020],
        &stops,
    );
    base_paint.shader(Some(base_shader.as_ref()));
    rect(factory, renderer, base, base_paint.as_ref());

    let overlay = Aabb::new(50.0, 50.0, 250.0, 250.0);
    let mut overlay_paint = factory.make_render_paint();
    let overlay_shader = factory.make_linear_gradient(
        0.0,
        overlay.min_y,
        0.0,
        overlay.max_y,
        &[0xccff2020, 0x0dffff20],
        &stops,
    );
    overlay_paint.shader(Some(overlay_shader.as_ref()));
    rect(factory, renderer, overlay, overlay_paint.as_ref());

    let mut multiply_paint = factory.make_render_paint();
    multiply_paint.color(0xff90b0d0);
    multiply_paint.blend_mode(BlendMode::Multiply);
    rect(
        factory,
        renderer,
        Aabb::new(140.0, 80.0, 260.0, 200.0),
        multiply_paint.as_ref(),
    );

    if let Ok(image) = factory.decode_image(NOMOON) {
        renderer.save();
        renderer.translate(60.0, 120.0);
        let scale = 140.0 / image.width().max(image.height()) as f32;
        renderer.scale(scale, scale);
        renderer.draw_image(
            Some(image.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            BlendMode::SrcOver,
            0.35,
        );
        renderer.restore();
    }

    for i in 0..3 {
        let mut paint = factory.make_render_paint();
        paint.style(RenderPaintStyle::Stroke);
        paint.thickness(1.5);
        paint.color(0x8020ff80);
        circle(factory, renderer, 60.0 + 25.0 * i as f32, paint.as_ref());
    }
    renderer.restore();
}

#[test]
fn uber_parity() {
    let pixels = run(draw_scene, DitherMode::interleavedGradientNoise);
    assert_scene_rendered(&pixels);
}
