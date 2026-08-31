//! tests/gm/serialized_replay_2d.cpp at e949498e.
use super::ore_gm_helper::*;
use nuxie_render_api::{
    serialized_replay::{replay_serialized_commands, SerializedReplayHooks},
    SerializingFactory,
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
fn scene(replay: bool) -> Vec<u8> {
    let mut host = GmHost::new(0xff202028);
    let screen = host.screen();
    if replay {
        let mut sf = SerializingFactory::new();
        let mut recorder = sf.make_renderer();
        let path = sf.make_render_path(shape(), FillRule::NonZero);
        let mut paint = sf.make_render_paint();
        paint.color(0xffffa030);
        paint.style(RenderPaintStyle::Fill);
        recorder.draw_path(path.as_ref(), paint.as_ref());
        assert!(replay_serialized_commands(
            &sf.bytes(),
            &mut host.factory,
            screen.borrow_mut().as_mut(),
            &mut SerializedReplayHooks::default()
        ));
    } else {
        let path = host.factory.make_render_path(shape(), FillRule::NonZero);
        let mut paint = host.factory.make_render_paint();
        paint.color(0xffffa030);
        paint.style(RenderPaintStyle::Fill);
        screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    }
    host.finish()
}
#[test]
fn serialized_replay_2d() {
    assert_pixels_equal(&scene(false), &scene(true));
}
