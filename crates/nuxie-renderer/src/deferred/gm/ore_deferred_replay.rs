//! tests/gm/ore_deferred_replay.cpp at e949498e.
use super::ore_gm_helper::*;
use nuxie_ore_metal::ore_cmd::{
    ore_command_buffer::OreCommandBuffer,
    ore_deferred_render_pass::beginRenderPassRecordingOrImmediate,
    ore_render_pass_recording::RenderPassRecording, ore_replay::replayCommandBuffer,
};
enum ReplayMode {
    Immediate,
    RecordReplay,
    InlineDeferred,
}
fn scene(mode: ReplayMode) -> Vec<u8> {
    let mut host = GmHost::new(0xff000000);
    let canvas = host.canvas(256, 256);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    let vb = vertex_buffer(&mut *host.ore.borrow_mut(), "ore_deferred_replay_vb");
    let module = shader(&mut *host.ore.borrow_mut(), 0);
    let pipeline = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &module,
        target_format(&target),
        "ore_deferred_replay_pipeline",
    );
    let desc = pass_desc(
        &target,
        Some("ore_deferred_replay_pass"),
        [0.1, 0.1, 0.15, 1.0],
    );
    host.begin_ore();
    match mode {
        ReplayMode::RecordReplay => {
            let buffer = Rc::new(RefCell::new(OreCommandBuffer::default()));
            {
                let mut pass = RenderPassRecording::new(
                    Some(host.ore.borrow().contextBase()),
                    buffer.clone(),
                    &desc,
                );
                triangle_pass(&mut pass, &pipeline, &vb);
            }
            replayCommandBuffer(&mut *host.ore.borrow_mut(), &buffer.borrow(), None);
        }
        ReplayMode::InlineDeferred => {
            host.ore.borrow().setDeferredRecording(true);
            let mut pass = beginRenderPassRecordingOrImmediate(host.ore.clone(), &desc, None)
                .expect("GM inline pass");
            triangle_pass(pass.as_mut(), &pipeline, &vb);
            host.ore.borrow().setDeferredRecording(false);
        }
        ReplayMode::Immediate => {
            let mut pass = host
                .ore
                .borrow_mut()
                .beginRenderPass(&desc, None)
                .expect("GM immediate pass");
            triangle_pass(pass.as_mut(), &pipeline, &vb);
        }
    }
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    host.finish()
}
#[test]
fn ore_deferred_replay() {
    let immediate = scene(ReplayMode::Immediate);
    assert_pixels_equal(&immediate, &scene(ReplayMode::RecordReplay));
    assert_pixels_equal(&immediate, &scene(ReplayMode::InlineDeferred));
}
