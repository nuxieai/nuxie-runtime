//! Complete tests/gm/ore_mip_render_target.cpp at upstream 14bdbfaf.
use super::ore_gm_helper::*;
use nuxie_ore_metal::ore_cmd::{
    ore_command_buffer::OreCommandBuffer, ore_render_pass_recording::RenderPassRecording,
    ore_replay::replayCommandBuffer,
};

fn begin_pass(
    host: &GmHost,
    desc: &RenderPassDesc<'_>,
    recording: &Option<Rc<RefCell<OreCommandBuffer>>>,
) -> Box<dyn RenderPassApi> {
    if let Some(recording) = recording {
        Box::new(RenderPassRecording::new(
            Some(host.ore.borrow().contextBase()),
            recording.clone(),
            desc,
        ))
    } else {
        host.ore
            .borrow_mut()
            .beginRenderPass(desc, None)
            .expect("mip pass")
    }
}

fn scene(deferred: bool) -> Vec<u8> {
    let mut host = GmHost::new(0xff000000);
    let color_desc = TextureDesc {
        width: 64,
        height: 64,
        format: TextureFormat::rgba8unorm,
        renderTarget: true,
        numMipmaps: 4,
        label: Some("ore_mip_render_target_color"),
        ..Default::default()
    };
    let color = host
        .ore
        .borrow_mut()
        .makeTexture(&color_desc)
        .expect("color texture");
    let depth = host
        .ore
        .borrow_mut()
        .makeTexture(&TextureDesc {
            format: TextureFormat::depth32float,
            label: Some("ore_mip_render_target_depth"),
            ..color_desc
        })
        .expect("depth texture");
    let mut color_views = Vec::new();
    let mut depth_views = Vec::new();
    for mip in 0..4 {
        for (texture, views) in [(&color, &mut color_views), (&depth, &mut depth_views)] {
            views.push(
                host.ore
                    .borrow_mut()
                    .makeTextureView(&TextureViewDesc {
                        texture: Some(texture),
                        dimension: TextureViewDimension::texture2D,
                        baseMipLevel: mip,
                        mipCount: 1,
                        baseLayer: 0,
                        layerCount: 1,
                        ..Default::default()
                    })
                    .expect("mip view"),
            );
        }
    }
    let vb = vertex_buffer(&mut *host.ore.borrow_mut(), "ore_mip_render_target_vb");
    let flipped: Vec<u8> = [
        [0.0f32, -0.6, 1.0, 0.2, 0.2, 1.0],
        [-0.6, 0.6, 0.2, 1.0, 0.2, 1.0],
        [0.6, 0.6, 0.2, 0.2, 1.0, 1.0],
    ]
    .into_iter()
    .flatten()
    .flat_map(f32::to_ne_bytes)
    .collect();
    let vb_flipped = host
        .ore
        .borrow_mut()
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::vertex,
            size: flipped.len() as u32,
            data: Some(&flipped),
            label: Some("ore_mip_render_target_vb_flipped"),
            immutable: false,
        })
        .expect("flipped vertices");
    let tri_shader = shader(&mut *host.ore.borrow_mut(), 0);
    let attrs = [
        VertexAttribute {
            offset: 0,
            shaderSlot: 0,
            format: VertexFormat::float2,
            ..Default::default()
        },
        VertexAttribute {
            offset: 8,
            shaderSlot: 1,
            format: VertexFormat::float4,
            ..Default::default()
        },
    ];
    let vertices = [VertexBufferLayout {
        stride: 24,
        stepMode: VertexStepMode::vertex,
        attributes: Some(&attrs),
        attributeCount: 2,
    }];
    let mut tri_desc = PipelineDesc {
        vertexModule: Some(&tri_shader),
        fragmentModule: Some(&tri_shader),
        vertexBuffers: Some(&vertices),
        vertexBufferCount: 1,
        topology: PrimitiveTopology::triangleList,
        colorCount: 1,
        label: Some("ore_mip_render_target_tri"),
        ..Default::default()
    };
    tri_desc.colorTargets[0].format = TextureFormat::rgba8unorm;
    tri_desc.depthStencil.format = TextureFormat::depth32float;
    tri_desc.depthStencil.depthCompare = CompareFunction::always;
    tri_desc.depthStencil.depthWriteEnabled = true;
    let tri = host
        .ore
        .borrow_mut()
        .makePipeline(&tri_desc, None)
        .expect("mip triangle pipeline");
    let canvas = host.canvas(256, 256);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    let image_shader = shader(&mut *host.ore.borrow_mut(), 2);
    let layout1 = layout_from_shader(&mut *host.ore.borrow_mut(), &image_shader, 1);
    let layout2 = layout_from_shader(&mut *host.ore.borrow_mut(), &image_shader, 2);
    let layouts = [None, Some(&layout1), Some(&layout2)];
    let mut image_desc = PipelineDesc {
        vertexModule: Some(&image_shader),
        fragmentModule: Some(&image_shader),
        vertexBufferCount: 0,
        topology: PrimitiveTopology::triangleList,
        colorCount: 1,
        bindGroupLayouts: Some(&layouts),
        bindGroupLayoutCount: 3,
        label: Some("ore_mip_render_target_img"),
        ..Default::default()
    };
    image_desc.colorTargets[0].format = target_format(&target);
    image_desc.depthStencil.depthCompare = CompareFunction::always;
    image_desc.depthStencil.depthWriteEnabled = false;
    let image = host
        .ore
        .borrow_mut()
        .makePipeline(&image_desc, None)
        .expect("mip image pipeline");
    let chain = host
        .ore
        .borrow_mut()
        .makeTextureView(&TextureViewDesc {
            texture: Some(&color),
            dimension: TextureViewDimension::texture2D,
            baseMipLevel: 0,
            mipCount: 4,
            baseLayer: 0,
            layerCount: 1,
            ..Default::default()
        })
        .expect("whole mip chain");
    let textures = [TexEntry {
        slot: 0,
        view: Some(&chain),
    }];
    let texture_group = host
        .ore
        .borrow_mut()
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout1),
            textures: &textures,
            textureCount: 1,
            ..Default::default()
        })
        .expect("mip texture group");
    let mut samplers = Vec::new();
    let mut sampler_groups = Vec::new();
    for mip in 0..4 {
        samplers.push(
            host.ore
                .borrow_mut()
                .makeSampler(&SamplerDesc {
                    minFilter: Filter::nearest,
                    magFilter: Filter::nearest,
                    mipmapFilter: Filter::nearest,
                    minLod: mip as f32,
                    maxLod: mip as f32,
                    ..Default::default()
                })
                .expect("mip sampler"),
        );
        let entries = [SampEntry {
            slot: 0,
            sampler: samplers.last(),
        }];
        sampler_groups.push(
            host.ore
                .borrow_mut()
                .makeBindGroup(&BindGroupDesc {
                    layout: Some(&layout2),
                    samplers: &entries,
                    samplerCount: 1,
                    ..Default::default()
                })
                .expect("mip sampler group"),
        );
    }
    host.begin_ore();
    let recording = deferred.then(|| Rc::new(RefCell::new(OreCommandBuffer::default())));
    let clears = [
        [0.10, 0.10, 0.15, 1.0],
        [0.35, 0.10, 0.10, 1.0],
        [0.10, 0.35, 0.10, 1.0],
        [0.10, 0.10, 0.35, 1.0],
    ];
    for mip in 0..4 {
        let mut desc = pass_desc(
            &color_views[mip],
            Some("ore_mip_render_target_level"),
            clears[mip],
        );
        desc.depthStencil.view = Some(&depth_views[mip]);
        desc.depthStencil.depthLoadOp = LoadOp::clear;
        desc.depthStencil.depthStoreOp = StoreOp::store;
        let mut pass = begin_pass(&host, &desc, &recording);
        pass.setPipeline(Some(&tri));
        pass.setVertexBuffer(0, Some(&vb), 0);
        // Deliberately no viewport: the backend must use the attached mip size.
        pass.draw(3, 1, 0, 0);
        pass.finish();
    }
    {
        let mut desc = pass_desc(
            &color_views[1],
            Some("ore_mip_render_target_reload"),
            [0.0; 4],
        );
        desc.colorAttachments[0].loadOp = LoadOp::load;
        desc.depthStencil.view = Some(&depth_views[1]);
        desc.depthStencil.depthLoadOp = LoadOp::load;
        desc.depthStencil.depthStoreOp = StoreOp::store;
        let mut pass = begin_pass(&host, &desc, &recording);
        pass.setPipeline(Some(&tri));
        pass.setVertexBuffer(0, Some(&vb_flipped), 0);
        pass.draw(3, 1, 0, 0);
        pass.finish();
    }
    let desc = pass_desc(
        &target,
        Some("ore_mip_render_target_show"),
        [0.0, 0.0, 0.0, 1.0],
    );
    let mut show = begin_pass(&host, &desc, &recording);
    show.setPipeline(Some(&image));
    show.setBindGroup(1, Some(&texture_group), None, 0);
    for mip in 0..4 {
        show.setViewport(
            (mip % 2) as f32 * 128.0,
            (mip / 2) as f32 * 128.0,
            128.0,
            128.0,
            0.0,
            1.0,
        );
        show.setBindGroup(2, Some(&sampler_groups[mip]), None, 0);
        show.draw(6, 1, 0, 0);
    }
    show.finish();
    if let Some(recording) = recording {
        replayCommandBuffer(&mut *host.ore.borrow_mut(), &recording.borrow(), None);
    }
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    host.finish()
}

#[test]
fn ore_mip_render_target() {
    let immediate = scene(false);
    assert_pixels_equal(&immediate, &scene(true));
    assert_cpp_gm_pixels("ore_mip_render_target", immediate);
}
