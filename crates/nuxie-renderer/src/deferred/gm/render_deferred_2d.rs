//! tests/gm/render_deferred_2d.cpp at e949498e.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_render_factory::DeferredFactory,
    render_replay::{replay_render_commands, ReplayHooks, ResourceTable},
};
fn shape() -> RawPath {
    let mut p = RawPath::new();
    p.move_to(40.0, 40.0);
    p.line_to(160.0, 70.0);
    p.line_to(120.0, 110.0);
    p.line_to(200.0, 200.0);
    p.line_to(120.0, 160.0);
    p.line_to(60.0, 210.0);
    p.close();
    p
}
fn draw_scene(factory: &mut dyn Factory, renderer: &mut dyn Renderer) {
    let path = factory.make_render_path(shape(), FillRule::NonZero);
    let mut fill = factory.make_render_paint();
    fill.style(RenderPaintStyle::Fill);
    fill.color(0xffffa030);
    renderer.draw_path(path.as_ref(), fill.as_ref());
    let mut stroke = factory.make_render_paint();
    stroke.style(RenderPaintStyle::Stroke);
    stroke.color(0xff3050ff);
    stroke.thickness(10.0);
    stroke.join(StrokeJoin::Round);
    stroke.cap(StrokeCap::Round);
    renderer.draw_path(path.as_ref(), stroke.as_ref());
    let mut raw = RawPath::new();
    raw.move_to(30.0, 30.0);
    raw.line_to(226.0, 30.0);
    raw.line_to(226.0, 90.0);
    raw.line_to(30.0, 90.0);
    raw.close();
    let box_path = factory.make_render_path(raw, FillRule::NonZero);
    let grad = factory.make_linear_gradient(
        30.0,
        30.0,
        226.0,
        90.0,
        &[0xff00e0a0, 0xffe000a0],
        &[0.0, 1.0],
    );
    let mut paint = factory.make_render_paint();
    paint.style(RenderPaintStyle::Fill);
    paint.shader(Some(grad.as_ref()));
    renderer.draw_path(box_path.as_ref(), paint.as_ref());
    let mut tri = factory.make_empty_render_path();
    tri.fill_rule(FillRule::NonZero);
    tri.move_to(20.0, 195.0);
    tri.line_to(80.0, 195.0);
    tri.line_to(50.0, 150.0);
    tri.close();
    let mut paint = factory.make_render_paint();
    paint.style(RenderPaintStyle::Fill);
    paint.color(0xffffffff);
    renderer.draw_path(tri.as_ref(), paint.as_ref());
    if let Ok(img) = factory.decode_image(&fixture("command_queue/batdude.png")) {
        renderer.save();
        renderer.transform(Mat2D([0.12, 0.0, 0.0, 0.12, 150.0, 110.0]));
        renderer.draw_image(
            Some(img.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            BlendMode::SrcOver,
            1.0,
        );
        renderer.restore();
    }
}
fn scene(deferred: bool) -> Vec<u8> {
    let mut host = GmHost::new(0xff202028);
    let screen = host.screen();
    if deferred {
        let mut df = DeferredFactory::new();
        let mut dr = df.make_renderer(None);
        draw_scene(&mut df, &mut dr);
        let buffer = df.buffer.lock().unwrap();
        replay_render_commands(
            &mut host.factory,
            Some(screen.borrow_mut().as_mut()),
            buffer.command_bytes(),
            buffer.blob_bytes(),
            &mut ResourceTable::default(),
            &mut ReplayHooks::default(),
        );
    } else {
        draw_scene(&mut host.factory, screen.borrow_mut().as_mut());
    }
    host.finish()
}
#[test]
fn render_deferred_2d() {
    assert_pixels_equal(&scene(false), &scene(true));
}
