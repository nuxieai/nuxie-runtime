//! tests/gm/render_canvas_dag.cpp at e949498e.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_replayer::{snapshot_frame, DeferredFrameSink, DeferredReplayer},
    deferred_session::DeferredSession,
};
fn oval(factory: &mut dyn Factory, bounds: Aabb) -> Box<dyn RenderPath> {
    let mut raw = RawPath::new();
    raw.add_oval(bounds);
    factory.make_render_path(raw, FillRule::NonZero)
}
fn solid(factory: &mut dyn Factory, color: u32) -> Box<dyn RenderPaint> {
    let mut paint = factory.make_render_paint();
    paint.color(color);
    paint
}
fn replay(session: &mut DeferredSession, replayer: &mut DeferredReplayer, host: &mut GmHost) {
    let frame = snapshot_frame(session);
    session.reset_frame();
    replayer.replay_frame(&frame, host);
}
fn screen(session: &DeferredSession, a: &RenderCanvasHandle, b: &RenderCanvasHandle) {
    let mut renderer = session.make_screen_renderer(0);
    // Native Metal's framebufferBottomUp is false.
    draw_canvas(renderer.as_mut(), a, 0.0, 64.0, false);
    draw_canvas(renderer.as_mut(), b, 128.0, 64.0, false);
}
fn chain(reversed: bool) -> Vec<u8> {
    let mut host = GmHost::with_screen(0xff202028, false);
    let a = host.canvas(128, 128);
    let b = host.canvas(128, 128);
    let mut session = DeferredSession::new(None);
    let mut replayer = DeferredReplayer::default();
    let green = solid(&mut session, 0xff30c060);
    let orange = solid(&mut session, 0xffe08830);
    let circle = oval(&mut session, Aabb::new(24.0, 24.0, 104.0, 104.0));
    let dot = oval(&mut session, Aabb::new(8.0, 8.0, 40.0, 40.0));
    let record_a = |session: &mut DeferredSession| {
        let mut r = session.begin_canvas_content(a.clone(), 0xff103050).unwrap();
        r.draw_path(circle.as_ref(), green.as_ref());
        session.end_canvas_content(&a);
    };
    let record_b = |session: &mut DeferredSession| {
        let mut r = session.begin_canvas_content(b.clone(), 0xff501030).unwrap();
        draw_canvas(r.as_mut(), &a, 0.0, 0.0, false);
        r.draw_path(dot.as_ref(), orange.as_ref());
        session.end_canvas_content(&b);
    };
    if reversed {
        record_b(&mut session);
        record_a(&mut session);
    } else {
        record_a(&mut session);
        record_b(&mut session);
    }
    screen(&session, &a, &b);
    replay(&mut session, &mut replayer, &mut host);
    host.finish()
}
fn cycle() -> Vec<u8> {
    let mut host = GmHost::with_screen(0xff202028, false);
    let a = host.canvas(128, 128);
    let b = host.canvas(128, 128);
    let mut session = DeferredSession::new(None);
    let mut replayer = DeferredReplayer::default();
    {
        let green = solid(&mut session, 0xff30c060);
        let orange = solid(&mut session, 0xffe08830);
        let circle = oval(&mut session, Aabb::new(24.0, 24.0, 104.0, 104.0));
        let mut r = session.begin_canvas_content(a.clone(), 0xff103050).unwrap();
        r.draw_path(circle.as_ref(), green.as_ref());
        session.end_canvas_content(&a);
        let mut r = session.begin_canvas_content(b.clone(), 0xff501030).unwrap();
        r.draw_path(circle.as_ref(), orange.as_ref());
        session.end_canvas_content(&b);
        replay(&mut session, &mut replayer, &mut host);
    }
    {
        let white = solid(&mut session, 0xffffffff);
        let dot = oval(&mut session, Aabb::new(4.0, 4.0, 24.0, 24.0));
        let mut r = session.begin_canvas_content(a.clone(), 0xff103050).unwrap();
        r.save();
        r.scale(0.5, 0.5);
        draw_canvas(r.as_mut(), &b, 0.0, 0.0, false);
        r.restore();
        r.draw_path(dot.as_ref(), white.as_ref());
        session.end_canvas_content(&a);
        let mut r = session.begin_canvas_content(b.clone(), 0xff501030).unwrap();
        r.save();
        r.scale(0.5, 0.5);
        draw_canvas(r.as_mut(), &a, 0.0, 0.0, false);
        r.restore();
        r.draw_path(dot.as_ref(), white.as_ref());
        session.end_canvas_content(&b);
        screen(&session, &a, &b);
        replay(&mut session, &mut replayer, &mut host);
    }
    host.finish()
}

// The source cycle GM is an ordinary golden, not one of gmmain's paired GMs.
// Its source schedule demotes A's dependency first (record-order preference),
// so this independent immediate host reference seeds A/B, then renders A/B:
// A samples previous B; B samples current A. No captured pixel is rewritten.
fn cycle_immediate_reference() -> Vec<u8> {
    let mut host = GmHost::with_screen(0xff202028, false);
    let a = host.canvas(128, 128);
    let b = host.canvas(128, 128);
    {
        let green = solid(&mut host.factory, 0xff30c060);
        let orange = solid(&mut host.factory, 0xffe08830);
        let circle = oval(&mut host.factory, Aabb::new(24.0, 24.0, 104.0, 104.0));
        let r = host.begin_canvas_content(a.clone(), 0xff103050).unwrap();
        r.borrow_mut().draw_path(circle.as_ref(), green.as_ref());
        host.end_canvas_content();
        let r = host.begin_canvas_content(b.clone(), 0xff501030).unwrap();
        r.borrow_mut().draw_path(circle.as_ref(), orange.as_ref());
        host.end_canvas_content();
    }
    {
        let white = solid(&mut host.factory, 0xffffffff);
        let dot = oval(&mut host.factory, Aabb::new(4.0, 4.0, 24.0, 24.0));
        let r = host.begin_canvas_content(a.clone(), 0xff103050).unwrap();
        {
            let mut r = r.borrow_mut();
            r.save();
            r.scale(0.5, 0.5);
            draw_canvas(r.as_mut(), &b, 0.0, 0.0, false);
            r.restore();
            r.draw_path(dot.as_ref(), white.as_ref());
        }
        host.end_canvas_content();
        let r = host.begin_canvas_content(b.clone(), 0xff501030).unwrap();
        {
            let mut r = r.borrow_mut();
            r.save();
            r.scale(0.5, 0.5);
            draw_canvas(r.as_mut(), &a, 0.0, 0.0, false);
            r.restore();
            r.draw_path(dot.as_ref(), white.as_ref());
        }
        host.end_canvas_content();
        let r = host.begin_screen_frame(0).unwrap();
        draw_canvas(r.borrow_mut().as_mut(), &a, 0.0, 64.0, false);
        draw_canvas(r.borrow_mut().as_mut(), &b, 128.0, 64.0, false);
    }
    host.finish()
}
#[test]
fn canvas_dag_chain() {
    assert_pixels_equal(&chain(false), &chain(true));
}
#[test]
fn canvas_dag_cycle() {
    assert_pixels_equal(&cycle_immediate_reference(), &cycle());
}
