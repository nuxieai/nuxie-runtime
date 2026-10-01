//! tests/gm/ore_deferred_target.cpp at 6a2e3ab7.
use super::{ore_gm_helper::*, ore_gm_sink::GMFrameSink};
use crate::deferred::cmd::{
    deferred_replayer::{DeferredReplayer, snapshot_frame},
    deferred_session::{DeferredSession, ReplayCaps},
};

fn draw_background(factory: &mut dyn Factory, renderer: &mut dyn Renderer) {
    let mut paint = factory.make_render_paint();
    paint.color(0xff1ab38c);
    let mut path = factory.make_empty_render_path();
    path.move_to(-1.0, -1.0);
    path.line_to(257.0, -1.0);
    path.line_to(257.0, 257.0);
    path.line_to(-1.0, 257.0);
    path.close();
    renderer.draw_path(path.as_ref(), paint.as_ref());
}
fn draw_path(factory: &mut dyn Factory, renderer: &mut dyn Renderer) {
    let mut paint = factory.make_render_paint();
    paint.color(0xc0e02040);
    let mut path = factory.make_empty_render_path();
    path.move_to(128.0, 96.0);
    path.line_to(224.0, 160.0);
    path.line_to(128.0, 224.0);
    path.line_to(32.0, 160.0);
    path.close();
    renderer.draw_path(path.as_ref(), paint.as_ref());
}
fn record_triangle(ctx: &mut dyn ContextApi, target: &AnyResourceHandle) {
    let Some(module) = try_shader(ctx, 0) else {
        return;
    };
    let vb = try_vertex_buffer(ctx, "ore_deferred_target_vb");
    let pipeline = try_triangle_pipeline_with_stride(
        ctx,
        &module,
        target_format(target),
        "ore_deferred_target_pipeline",
        24,
    );
    let (Some(vb), Some(pipeline)) = (vb, pipeline) else {
        return;
    };
    let desc = pass_desc(
        target,
        Some("ore_deferred_target_pass"),
        [0.10, 0.70, 0.55, 1.0],
    );
    let mut pass = ctx.beginRenderPass(&desc, None).expect("GM target pass");
    triangle_pass(pass.as_mut(), &pipeline, &vb);
}
fn scene(deferred: bool) -> Vec<u8> {
    let mut host = GmHost::with_screen(0, false);
    host.open_preserving_screen();
    let desc = host.factory.borrow().ore_target_desc();
    let target_info = host.factory.borrow().ore_render_target();
    if desc.width == 0 {
        let screen = host.screen();
        draw_background(&mut host.factory, screen.borrow_mut().as_mut());
        draw_path(&mut host.factory, screen.borrow_mut().as_mut());
    } else if deferred {
        let mut session = DeferredSession::with_caps(ReplayCaps::from(&*host.ore.borrow()));
        session.ore_context.borrow_mut().setTarget(desc);
        let target = session
            .ore_context
            .borrow_mut()
            .targetView()
            .expect("exposed GM target");
        record_triangle(&mut *session.ore_context.borrow_mut(), &target);
        session.record_ore_replay_marker();
        let mut renderer = session.make_screen_renderer(0);
        draw_path(&mut session, renderer.as_mut());
        let frame = snapshot_frame(&mut session);
        DeferredReplayer::default().replay_frame(
            &frame,
            &mut GMFrameSink {
                host: &mut host,
                target: target_info,
            },
        );
    } else {
        host.begin_ore();
        // The factory owns the native target for the entire GM frame.
        let target =
            target_info.and_then(|info| unsafe { host.ore.borrow_mut().wrapRenderTarget(info) });
        if let Some(target) = target.as_ref() {
            record_triangle(&mut *host.ore.borrow_mut(), target);
        }
        host.end_ore();
        let screen = host.screen();
        if target.is_none() {
            draw_background(&mut host.factory, screen.borrow_mut().as_mut());
        }
        draw_path(&mut host.factory, screen.borrow_mut().as_mut());
    }
    host.finish()
}
#[test]
fn ore_deferred_target_parity() {
    let immediate = scene(false);
    let deferred = scene(true);
    assert_pixels_equal("ore_deferred_target", 1, &immediate, &deferred);
    assert_cpp_gm_pixels("ore_deferred_target", deferred);
}
