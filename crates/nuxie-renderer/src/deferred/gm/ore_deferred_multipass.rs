//! tests/gm/ore_deferred_multipass.cpp at e949498e.
use super::ore_gm_helper::*;
use nuxie_ore_metal::ore_cmd::{
    ore_command_buffer::OreCommandBuffer, ore_render_pass_recording::RenderPassRecording,
    ore_replay::replayCommandBuffer,
};
fn image_pass(
    pass: &mut dyn RenderPassApi,
    pipeline: &AnyResourceHandle,
    texture: &AnyResourceHandle,
    sampler: &AnyResourceHandle,
) {
    pass.setPipeline(Some(pipeline));
    pass.setBindGroup(1, Some(texture), None, 0);
    pass.setBindGroup(2, Some(sampler), None, 0);
    pass.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
    pass.draw(6, 1, 0, 0);
    pass.finish();
}
fn scene(deferred: bool) -> Vec<u8> {
    let mut host = GmHost::new(0xff000000);
    let canvas_a = host.canvas(256, 256);
    let canvas_b = host.canvas(256, 256);
    let target_a = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas_a);
    let target_b = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas_b);
    let vb = vertex_buffer(&mut *host.ore.borrow_mut(), "ore_deferred_multipass_vb");
    let tri_shader = shader(&mut *host.ore.borrow_mut(), 0);
    let tri_pipeline = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &tri_shader,
        target_format(&target_a),
        "ore_deferred_multipass_tri",
    );
    let sampler = host
        .ore
        .borrow_mut()
        .makeSampler(&SamplerDesc {
            minFilter: Filter::nearest,
            magFilter: Filter::nearest,
            ..Default::default()
        })
        .expect("GM sampler");
    let image_shader = shader(&mut *host.ore.borrow_mut(), 2);
    let layout1 = layout_from_shader(&mut *host.ore.borrow_mut(), &image_shader, 1);
    let layout2 = layout_from_shader(&mut *host.ore.borrow_mut(), &image_shader, 2);
    let layouts = [None, Some(&layout1), Some(&layout2)];
    let mut pd = PipelineDesc {
        vertexModule: Some(&image_shader),
        fragmentModule: Some(&image_shader),
        vertexBufferCount: 0,
        topology: PrimitiveTopology::triangleList,
        bindGroupLayouts: Some(&layouts),
        bindGroupLayoutCount: 3,
        label: Some("ore_deferred_multipass_img"),
        ..Default::default()
    };
    pd.colorTargets[0].format = target_format(&target_b);
    pd.depthStencil.depthCompare = CompareFunction::always;
    pd.depthStencil.depthWriteEnabled = false;
    let img_pipeline = host
        .ore
        .borrow_mut()
        .makePipeline(&pd, None)
        .expect("GM image pipeline");
    let tex_entries = [TexEntry {
        slot: 0,
        view: Some(&target_a),
    }];
    let tex = host
        .ore
        .borrow_mut()
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout1),
            textures: &tex_entries,
            textureCount: 1,
            ..Default::default()
        })
        .expect("GM texture group");
    let samp_entries = [SampEntry {
        slot: 0,
        sampler: Some(&sampler),
    }];
    let samp = host
        .ore
        .borrow_mut()
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout2),
            samplers: &samp_entries,
            samplerCount: 1,
            ..Default::default()
        })
        .expect("GM sampler group");
    let desc_a = pass_desc(
        &target_a,
        Some("ore_deferred_multipass_passA"),
        [0.1, 0.1, 0.15, 1.0],
    );
    let desc_b = pass_desc(
        &target_b,
        Some("ore_deferred_multipass_passB"),
        [0.0, 0.0, 0.0, 1.0],
    );
    host.begin_ore();
    if deferred {
        let buffer = Rc::new(RefCell::new(OreCommandBuffer::default()));
        {
            let mut p1 = RenderPassRecording::new(
                Some(host.ore.borrow().contextBase()),
                buffer.clone(),
                &desc_a,
            );
            triangle_pass(&mut p1, &tri_pipeline, &vb);
            let mut p2 = RenderPassRecording::new(
                Some(host.ore.borrow().contextBase()),
                buffer.clone(),
                &desc_b,
            );
            image_pass(&mut p2, &img_pipeline, &tex, &samp);
        }
        replayCommandBuffer(&mut *host.ore.borrow_mut(), &buffer.borrow(), None);
    } else {
        let mut p1 = host
            .ore
            .borrow_mut()
            .beginRenderPass(&desc_a, None)
            .expect("GM producer pass");
        triangle_pass(p1.as_mut(), &tri_pipeline, &vb);
        let mut p2 = host
            .ore
            .borrow_mut()
            .beginRenderPass(&desc_b, None)
            .expect("GM consumer pass");
        image_pass(p2.as_mut(), &img_pipeline, &tex, &samp);
    }
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas_b);
    host.finish()
}
#[test]
fn ore_deferred_multipass() {
    assert_pixels_equal(&scene(false), &scene(true));
}
