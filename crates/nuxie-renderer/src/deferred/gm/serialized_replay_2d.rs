//! tests/gm/serialized_replay_2d.cpp at e949498e.
use super::ore_gm_helper::*;
use nuxie_render_api::{
    SerializingFactory,
    serialized_replay::{SerializedReplayHooks, replay_serialized_commands},
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
    let mut paint = factory.make_render_paint();
    paint.color(0xffffa030);
    paint.style(RenderPaintStyle::Fill);
    renderer.draw_path(path.as_ref(), paint.as_ref());
    let mut raw = RawPath::new();
    raw.move_to(30.0, 30.0);
    raw.line_to(226.0, 30.0);
    raw.line_to(226.0, 90.0);
    raw.line_to(30.0, 90.0);
    raw.close();
    let path = factory.make_render_path(raw, FillRule::NonZero);
    let gradient = factory.make_linear_gradient(
        30.0,
        30.0,
        226.0,
        90.0,
        &[0xff00e0a0, 0xffe000a0],
        &[0.0, 1.0],
    );
    let mut paint = factory.make_render_paint();
    paint.style(RenderPaintStyle::Fill);
    paint.shader(Some(gradient.as_ref()));
    paint.shader_transform(Mat2D([0.8, 0.3, -0.3, 0.8, 20.0, -10.0]));
    renderer.draw_path(path.as_ref(), paint.as_ref());
}
fn scene(replay: bool) -> Vec<u8> {
    let mut host = GmHost::new(0xff202028);
    let screen = host.screen();
    if replay {
        let mut sf = SerializingFactory::new();
        let mut recorder = sf.make_renderer();
        draw_scene(&mut sf, &mut recorder);
        assert!(replay_serialized_commands(
            &sf.bytes(),
            &mut host.factory,
            screen.borrow_mut().as_mut(),
            &mut SerializedReplayHooks::default()
        ));
    } else {
        draw_scene(&mut host.factory, screen.borrow_mut().as_mut());
    }
    host.finish()
}
#[test]
fn serialized_replay_2d() {
    assert_pixels_equal("serialized_replay_2d", 1, &scene(false), &scene(true));
}
