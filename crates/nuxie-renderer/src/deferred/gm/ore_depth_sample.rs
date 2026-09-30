//! Complete Metal lane of tests/gm/ore_depth_sample.cpp at upstream 50ba2f5a.
use super::ore_gm_helper::*;

fn scene() -> Vec<u8> {
    let mut host = GmHost::with_size(0xff000000, true, 128, 128);
    let uniforms: Vec<u8> = [0.0f32, 1.0, 0.0, 1.0]
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect();
    let ubo = host
        .ore
        .borrow_mut()
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::uniform,
            size: 16,
            data: Some(&uniforms),
            immutable: false,
            label: Some("depth_sample_ubo"),
        })
        .expect("depth sample uniforms");
    let depth = host
        .ore
        .borrow_mut()
        .makeTexture(&TextureDesc {
            width: 1,
            height: 1,
            format: TextureFormat::depth32float,
            r#type: TextureType::texture2D,
            renderTarget: true,
            numMipmaps: 1,
            sampleCount: 1,
            label: Some("depth_sample_depth"),
            ..Default::default()
        })
        .expect("depth texture");
    let depth_view = host
        .ore
        .borrow_mut()
        .makeTextureView(&TextureViewDesc {
            texture: Some(&depth),
            dimension: TextureViewDimension::texture2D,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
            ..Default::default()
        })
        .expect("depth view");
    let sampler = host
        .ore
        .borrow_mut()
        .makeSampler(&SamplerDesc {
            minFilter: Filter::nearest,
            magFilter: Filter::nearest,
            label: Some("depth_sample_point"),
            ..Default::default()
        })
        .expect("point sampler");
    let shader = shader(&mut *host.ore.borrow_mut(), K_DEPTH_SAMPLE_WITNESS);
    let layout0 = layout_from_shader(&mut *host.ore.borrow_mut(), &shader, 0);
    let layout1 = layout_from_shader(&mut *host.ore.borrow_mut(), &shader, 1);
    let layouts = [Some(&layout0), Some(&layout1)];
    let mut pipeline_desc = PipelineDesc {
        vertexModule: Some(&shader),
        fragmentModule: Some(&shader),
        vertexBufferCount: 0,
        topology: PrimitiveTopology::triangleList,
        colorCount: 1,
        bindGroupLayouts: Some(&layouts),
        bindGroupLayoutCount: 2,
        label: Some("ore_depth_sample_pipeline"),
        ..Default::default()
    };
    pipeline_desc.colorTargets[0].format = TextureFormat::rgba8unorm;
    let mut error = String::new();
    let pipeline = host
        .ore
        .borrow_mut()
        .makePipeline(&pipeline_desc, Some(&mut error))
        .unwrap_or_else(|| panic!("depth sample pipeline: {error}"));
    let ubos = [UBOEntry {
        slot: 0,
        buffer: Some(&ubo),
        offset: 0,
        size: 16,
    }];
    let bg0 = host
        .ore
        .borrow_mut()
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout0),
            ubos: &ubos,
            uboCount: 1,
            label: Some("depth_sample_bg_uniforms"),
            ..Default::default()
        })
        .expect("uniform bind group");
    let textures = [TexEntry {
        slot: 1,
        view: Some(&depth_view),
    }];
    let samplers = [SampEntry {
        slot: 0,
        sampler: Some(&sampler),
    }];
    let bg1 = host
        .ore
        .borrow_mut()
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout1),
            textures: &textures,
            textureCount: 1,
            samplers: &samplers,
            samplerCount: 1,
            label: Some("depth_sample_bg_textures"),
            ..Default::default()
        })
        .expect("texture bind group");
    let canvas = host.canvas(128, 128);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    host.begin_ore();
    let mut clear_desc = RenderPassDesc {
        colorCount: 0,
        label: Some("ore_depth_sample_clear_pass"),
        ..Default::default()
    };
    clear_desc.depthStencil.view = Some(&depth_view);
    clear_desc.depthStencil.depthLoadOp = LoadOp::clear;
    clear_desc.depthStencil.depthStoreOp = StoreOp::store;
    clear_desc.depthStencil.depthClearValue = 1.0;
    let mut clear_pass = host
        .ore
        .borrow_mut()
        .beginRenderPass(&clear_desc, None)
        .expect("depth clear pass");
    clear_pass.finish();
    let desc = pass_desc(&target, Some("ore_depth_sample_pass"), [1.0, 0.0, 0.0, 1.0]);
    let mut pass = host
        .ore
        .borrow_mut()
        .beginRenderPass(&desc, None)
        .expect("depth sample pass");
    pass.setPipeline(Some(&pipeline));
    pass.setBindGroup(0, Some(&bg0), None, 0);
    pass.setBindGroup(1, Some(&bg1), None, 0);
    pass.setViewport(0.0, 0.0, 128.0, 128.0, 0.0, 1.0);
    pass.draw(3, 1, 0, 0);
    pass.finish();
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    host.finish()
}

#[test]
fn ore_depth_sample() {
    assert_cpp_gm_pixels_with_size("ore_depth_sample", 128, 128, scene());
}
