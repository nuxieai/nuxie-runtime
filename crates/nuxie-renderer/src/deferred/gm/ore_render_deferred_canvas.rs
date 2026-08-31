//! tests/gm/ore_render_deferred_canvas.cpp at e949498e.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_replayer::{snapshot_frame, DeferredReplayer},
    deferred_session::DeferredSession,
};
fn record_clear(ctx: &mut dyn ContextApi, view: &AnyResourceHandle) {
    let desc = pass_desc(view, None, [0.10, 0.70, 0.55, 1.0]);
    let mut pass = ctx
        .beginRenderPass(&desc, None)
        .expect("GM canvas clear pass");
    pass.setViewport(0.0, 0.0, 200.0, 200.0, 0.0, 1.0);
    pass.finish();
}
fn scene(deferred: bool) -> Vec<u8> {
    let mut host = GmHost::new(0xff202028);
    let canvas = host.canvas(200, 200);
    if deferred {
        let mut session = DeferredSession::new(Some(host.ore.clone()));
        let view = wrap_canvas(&mut *session.ore_context.borrow_mut(), &canvas);
        record_clear(&mut *session.ore_context.borrow_mut(), &view);
        session.record_ore_replay_marker();
        let mut renderer = session.make_screen_renderer(0);
        draw_canvas(renderer.as_mut(), &canvas, 28.0, 28.0, false);
        let frame = snapshot_frame(&mut session);
        DeferredReplayer::default().replay_frame(&frame, &mut host);
    } else {
        let view = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
        host.begin_ore();
        record_clear(&mut *host.ore.borrow_mut(), &view);
        host.end_ore();
        draw_canvas(
            host.screen().borrow_mut().as_mut(),
            &canvas,
            28.0,
            28.0,
            false,
        );
    }
    host.finish()
}
#[test]
fn ore_render_deferred_canvas() {
    assert_pixels_equal(&scene(false), &scene(true));
}
