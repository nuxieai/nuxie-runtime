//! tests/gm/ore_deferred_context.cpp at e949498e.
use super::ore_gm_helper::*;
use crate::deferred::ore::ore_deferred_context::DeferredOreContext;
fn scene(deferred: bool) -> Vec<u8> {
    let mut host = GmHost::new(0xff000000);
    let canvas = host.canvas(256, 256);
    let mut dctx = DeferredOreContext::fromReal(Some(host.ore.clone()));
    let (target, module, vb, pipeline) = {
        // Only the chosen API receiver changes, just as in the source GM.
        let mut real = (!deferred).then(|| host.ore.borrow_mut());
        let ctx: &mut dyn ContextApi = if deferred {
            &mut dctx
        } else {
            &mut **real.as_mut().unwrap()
        };
        let target = wrap_canvas(ctx, &canvas);
        let module = shader(ctx, 0);
        let vb = vertex_buffer(ctx, "ore_deferred_context_vb");
        let pipeline = triangle_pipeline(
            ctx,
            &module,
            target_format(&target),
            "ore_deferred_context_pipeline",
        );
        (target, module, vb, pipeline)
    };
    let desc = pass_desc(
        &target,
        Some("ore_deferred_context_pass"),
        [0.1, 0.1, 0.15, 1.0],
    );
    if deferred {
        let mut pass = dctx.beginRenderPass(&desc, None).expect("GM deferred pass");
        triangle_pass(pass.as_mut(), &pipeline, &vb);
        drop(pass);
        host.begin_ore();
        dctx.replay(&mut *host.ore.borrow_mut());
        host.end_ore();
    } else {
        host.begin_ore();
        let mut pass = host
            .ore
            .borrow_mut()
            .beginRenderPass(&desc, None)
            .expect("GM immediate pass");
        triangle_pass(pass.as_mut(), &pipeline, &vb);
        host.end_ore();
    }
    let _keep_module = module;
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    host.finish()
}
#[test]
fn ore_deferred_context() {
    assert_pixels_equal(&scene(false), &scene(true));
}
