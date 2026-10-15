//! tests/gm/canvas_paint_image.cpp at fa2cd6e3.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_replayer::{DeferredReplayer, snapshot_frame},
    deferred_session::DeferredSession,
};

fn oval(session: &mut DeferredSession, bounds: Aabb) -> Box<dyn RenderPath> {
    let mut raw = RawPath::new();
    raw.add_oval(bounds);
    session.make_render_path(raw, FillRule::NonZero)
}

fn render(reversed: bool) -> Vec<u8> {
    let mut host = GmHost::with_screen(0xff202028, false);
    let mut session = DeferredSession::with_caps(Default::default());
    session.bind_render_context(host.factory.persistent_context());
    let pattern = DeferredCanvasHost::make_content_canvas(&mut session, 32, 32).unwrap();
    let tile = DeferredCanvasHost::make_content_canvas(&mut session, 128, 128).unwrap();
    let mut green = session.make_render_paint();
    green.color(0xff30c060);
    let mut patterned = session.make_render_paint();
    patterned.color(0xffffffff);
    patterned.modulated_image(
        Some(pattern.borrow().render_image().as_ref()),
        ImageSampler {
            wrap_x: ImageWrap::Repeat,
            wrap_y: ImageWrap::Repeat,
            filter: ImageFilter::Nearest,
        },
        Mat2D([32.0, 0.0, 0.0, 32.0, 0.0, 0.0]),
    );
    let dot = oval(&mut session, Aabb::new(4.0, 4.0, 28.0, 28.0));
    let big = oval(&mut session, Aabb::new(8.0, 8.0, 120.0, 120.0));
    let record_pattern = |session: &mut DeferredSession| {
        let mut renderer = session
            .begin_canvas_content(pattern.clone(), 0xff103050)
            .unwrap();
        renderer.draw_path(dot.as_ref(), green.as_ref());
        session.end_canvas_content(&pattern);
    };
    let record_tile = |session: &mut DeferredSession| {
        let mut renderer = session
            .begin_canvas_content(tile.clone(), 0xff501030)
            .unwrap();
        renderer.draw_path(big.as_ref(), patterned.as_ref());
        session.end_canvas_content(&tile);
    };
    if reversed {
        record_tile(&mut session);
        record_pattern(&mut session);
    } else {
        record_pattern(&mut session);
        record_tile(&mut session);
    }
    let mut screen = session.make_screen_renderer(0);
    draw_canvas(screen.as_mut(), &tile, 0.0, 64.0);
    screen.save();
    screen.translate(128.0, 64.0);
    screen.draw_path(big.as_ref(), patterned.as_ref());
    screen.restore();
    let frame = snapshot_frame(&mut session);
    session.reset_frame();
    DeferredReplayer::default().replay_frame(&frame, &mut host);
    host.finish()
}

#[test]
fn canvas_paint_image() {
    assert_cpp_gm_pixels("canvas_paint_image", render(false));
}
#[test]
fn canvas_paint_image_reversed() {
    assert_cpp_gm_pixels("canvas_paint_image_reversed", render(true));
}
