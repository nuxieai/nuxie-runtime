//! tests/gm/ore_deferred_resource.cpp at e949498e.
use super::ore_gm_helper::*;
use crate::deferred::ore::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::ore_cmd::{
    ore_deferred_resource::DeferredBuffer, ore_make_replay::OreResident,
};
enum ResMode {
    Immediate,
    ReplayBuffer,
    Unified,
}
fn scene(mode: ResMode) -> Vec<u8> {
    let mut host = GmHost::new(0xff000000);
    let canvas = host.canvas(256, 256);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    let module = shader(&mut *host.ore.borrow_mut(), 0);
    let pipeline = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &module,
        target_format(&target),
        "ore_deferred_resource_pipeline",
    );
    let desc = pass_desc(
        &target,
        Some("ore_deferred_resource_pass"),
        [0.1, 0.1, 0.15, 1.0],
    );
    if matches!(mode, ResMode::Unified) {
        let mut dctx = DeferredOreContext::new(Some(host.ore.clone()));
        let vb = vertex_buffer(&mut dctx, "ore_deferred_resource_vb");
        {
            let mut pass = dctx
                .beginRenderPass(&desc, None)
                .expect("GM deferred resource pass");
            triangle_pass(pass.as_mut(), &pipeline, &vb);
        }
        host.begin_ore();
        dctx.replay(&mut *host.ore.borrow_mut());
        host.end_ore();
    } else {
        let mut table = OreResident::default();
        let mut dctx = None;
        let vb = if matches!(mode, ResMode::ReplayBuffer) {
            dctx = Some(DeferredOreContext::new(Some(host.ore.clone())));
            let dctx = dctx.as_mut().unwrap();
            let deferred = vertex_buffer(dctx, "ore_deferred_resource_vb");
            dctx.replayFrame(&mut *host.ore.borrow_mut(), &mut table, &mut |_| None);
            table
                .get(
                    deferred
                        .downcast_ref::<DeferredBuffer>()
                        .unwrap()
                        .clientHandle(),
                )
                .expect("GM replay-created buffer")
        } else {
            vertex_buffer(&mut *host.ore.borrow_mut(), "ore_deferred_resource_vb")
        };
        host.begin_ore();
        let mut pass = host
            .ore
            .borrow_mut()
            .beginRenderPass(&desc, None)
            .expect("GM resource pass");
        triangle_pass(pass.as_mut(), &pipeline, &vb);
        host.end_ore();
        let _keep_context = dctx;
    }
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    host.finish()
}
#[test]
fn ore_deferred_resource() {
    let immediate = scene(ResMode::Immediate);
    assert_pixels_equal(&immediate, &scene(ResMode::ReplayBuffer));
    assert_pixels_equal(&immediate, &scene(ResMode::Unified));
}
