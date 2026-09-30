//! tests/gm/ore_nested_pass.cpp and its three-way gmmain parity registration at 65638e57.
//! The producer A must run before consumer B, including when A is nested inside B.
use super::ore_gm_helper::*;
use nuxie_ore_metal::ore_cmd::{
    ore_command_buffer::OreCommandBuffer, ore_deferred_render_pass::beginRecordedRenderPass,
    ore_render_pass_recording::RenderPassRecording, ore_replay::replayCommandBuffer,
};

enum NestMode {
    Immediate,
    Inline,
    Recorded,
}

fn sample_a(
    pass: &mut dyn RenderPassApi,
    texture: &AnyResourceHandle,
    sampler: &AnyResourceHandle,
) {
    pass.setBindGroup(1, Some(texture), None, 0);
    pass.setBindGroup(2, Some(sampler), None, 0);
    pass.draw(6, 1, 0, 0);
    pass.finish();
}

fn scene(mode: NestMode) -> Vec<u8> {
    let mut host = GmHost::new(0xff000000);
    let canvas_a = host.canvas(256, 256);
    let canvas_b = host.canvas(256, 256);
    let target_a = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas_a);
    let target_b = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas_b);
    let vb = vertex_buffer(&mut *host.ore.borrow_mut(), "ore_nested_pass_vb");
    let tri_shader = shader(&mut *host.ore.borrow_mut(), 0);
    let tri_pipeline = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &tri_shader,
        target_format(&target_a),
        "ore_nested_pass_tri",
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
        colorCount: 1,
        bindGroupLayouts: Some(&layouts),
        bindGroupLayoutCount: 3,
        label: Some("ore_nested_pass_img"),
        ..Default::default()
    };
    pd.colorTargets[0].format = target_format(&target_b);
    pd.depthStencil.depthCompare = CompareFunction::always;
    pd.depthStencil.depthWriteEnabled = false;
    let image_pipeline = host
        .ore
        .borrow_mut()
        .makePipeline(&pd, None)
        .expect("GM image pipeline");
    let tex_entries = [TexEntry {
        slot: 0,
        view: Some(&target_a),
    }];
    let texture_group = host
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
    let sampler_group = host
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
        Some("ore_nested_pass_passA"),
        [0.1, 0.1, 0.15, 1.0],
    );
    let desc_b = pass_desc(
        &target_b,
        Some("ore_nested_pass_passB"),
        [0.0, 0.0, 0.0, 1.0],
    );

    host.begin_ore();
    match mode {
        NestMode::Immediate => {
            let mut pass_a = host
                .ore
                .borrow_mut()
                .beginRenderPass(&desc_a, None)
                .expect("GM producer pass");
            triangle_pass(pass_a.as_mut(), &tri_pipeline, &vb);
            drop(pass_a);
            let mut pass_b = host
                .ore
                .borrow_mut()
                .beginRenderPass(&desc_b, None)
                .expect("GM consumer pass");
            pass_b.setPipeline(Some(&image_pipeline));
            pass_b.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
            sample_a(pass_b.as_mut(), &texture_group, &sampler_group);
        }
        NestMode::Inline => {
            let mut pass_b = beginRecordedRenderPass(host.ore.clone(), &desc_b)
                .expect("GM nested consumer pass");
            pass_b.setPipeline(Some(&image_pipeline));
            pass_b.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
            let mut pass_a = beginRecordedRenderPass(host.ore.clone(), &desc_a)
                .expect("GM nested producer pass");
            triangle_pass(pass_a.as_mut(), &tri_pipeline, &vb);
            drop(pass_a);
            sample_a(pass_b.as_mut(), &texture_group, &sampler_group);
        }
        NestMode::Recorded => {
            let buffer = Rc::new(RefCell::new(OreCommandBuffer::default()));
            {
                let mut pass_b = RenderPassRecording::new(
                    Some(host.ore.borrow().contextBase()),
                    buffer.clone(),
                    &desc_b,
                );
                pass_b.setPipeline(Some(&image_pipeline));
                pass_b.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
                let mut pass_a = RenderPassRecording::new(
                    Some(host.ore.borrow().contextBase()),
                    buffer.clone(),
                    &desc_a,
                );
                triangle_pass(&mut pass_a, &tri_pipeline, &vb);
                sample_a(&mut pass_b, &texture_group, &sampler_group);
            }
            replayCommandBuffer(&mut *host.ore.borrow_mut(), &buffer.borrow(), None);
        }
    }
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas_b);
    host.finish()
}

#[test]
fn ore_nested_pass() {
    let immediate = scene(NestMode::Immediate);
    assert_pixels_equal("ore_nested_pass", 1, &immediate, &scene(NestMode::Inline));
    assert_pixels_equal("ore_nested_pass", 2, &immediate, &scene(NestMode::Recorded));
}
