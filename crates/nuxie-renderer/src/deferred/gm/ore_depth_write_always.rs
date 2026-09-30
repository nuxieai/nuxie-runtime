//! tests/gm/ore_depth_write_always.cpp at 19c0fad3, through the live Metal GM host.
//! The upstream golden is identical to ore_depth: later quads must remain behind
//! the first quad even when its depth comparison is Always.
use super::ore_gm_helper::*;

const DEPTH_VERTICES: [[f32; 7]; 18] = [
    [-0.6, 0.7, 0.2, 1.0, 0.0, 0.0, 1.0],
    [0.3, 0.7, 0.2, 1.0, 0.0, 0.0, 1.0],
    [-0.6, -0.2, 0.2, 1.0, 0.0, 0.0, 1.0],
    [0.3, 0.7, 0.2, 1.0, 0.0, 0.0, 1.0],
    [0.3, -0.2, 0.2, 1.0, 0.0, 0.0, 1.0],
    [-0.6, -0.2, 0.2, 1.0, 0.0, 0.0, 1.0],
    [-0.4, 0.4, 0.5, 0.0, 1.0, 0.0, 1.0],
    [0.5, 0.4, 0.5, 0.0, 1.0, 0.0, 1.0],
    [-0.4, -0.5, 0.5, 0.0, 1.0, 0.0, 1.0],
    [0.5, 0.4, 0.5, 0.0, 1.0, 0.0, 1.0],
    [0.5, -0.5, 0.5, 0.0, 1.0, 0.0, 1.0],
    [-0.4, -0.5, 0.5, 0.0, 1.0, 0.0, 1.0],
    [-0.2, 0.1, 0.8, 0.0, 0.0, 1.0, 1.0],
    [0.7, 0.1, 0.8, 0.0, 0.0, 1.0, 1.0],
    [-0.2, -0.8, 0.8, 0.0, 0.0, 1.0, 1.0],
    [0.7, 0.1, 0.8, 0.0, 0.0, 1.0, 1.0],
    [0.7, -0.8, 0.8, 0.0, 0.0, 1.0, 1.0],
    [-0.2, -0.8, 0.8, 0.0, 0.0, 1.0, 1.0],
];

fn scene(first_always: bool) -> Vec<u8> {
    let mut host = GmHost::new(0xff000000);
    let canvas = host.canvas(256, 256);
    let color_target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    let depth = host
        .ore
        .borrow_mut()
        .makeTexture(&TextureDesc {
            width: 256,
            height: 256,
            format: TextureFormat::depth32float,
            renderTarget: true,
            numMipmaps: 1,
            label: Some("depth_target"),
            ..Default::default()
        })
        .expect("depth texture");
    let depth_view = host
        .ore
        .borrow_mut()
        .makeTextureView(&TextureViewDesc {
            texture: Some(&depth),
            mipCount: 1,
            layerCount: 1,
            ..Default::default()
        })
        .expect("depth view");
    let vertices: Vec<u8> = DEPTH_VERTICES
        .into_iter()
        .flatten()
        .flat_map(f32::to_ne_bytes)
        .collect();
    let vbo = host
        .ore
        .borrow_mut()
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::vertex,
            size: vertices.len() as u32,
            data: Some(&vertices),
            label: Some("ore_depth_write_always_vbo"),
            immutable: false,
        })
        .expect("depth vertices");
    let module = shader(&mut *host.ore.borrow_mut(), 1); // ore_gm::kDepth
    let attrs = [
        VertexAttribute {
            offset: 0,
            shaderSlot: 0,
            format: VertexFormat::float3,
            ..Default::default()
        },
        VertexAttribute {
            offset: 12,
            shaderSlot: 1,
            format: VertexFormat::float4,
            ..Default::default()
        },
    ];
    let layouts = [VertexBufferLayout {
        stride: 28,
        stepMode: VertexStepMode::vertex,
        attributes: Some(&attrs),
        attributeCount: 2,
    }];
    let mut desc = PipelineDesc {
        vertexModule: Some(&module),
        fragmentModule: Some(&module),
        vertexBuffers: Some(&layouts),
        vertexBufferCount: 1,
        topology: PrimitiveTopology::triangleList,
        colorCount: 1,
        label: Some("ore_depth_write_always_test"),
        ..Default::default()
    };
    desc.colorTargets[0].format = TextureFormat::rgba8unorm;
    desc.depthStencil.format = TextureFormat::depth32float;
    desc.depthStencil.depthCompare = CompareFunction::lessEqual;
    desc.depthStencil.depthWriteEnabled = true;
    let pipeline = host
        .ore
        .borrow_mut()
        .makePipeline(&desc, None)
        .expect("lessEqual pipeline");
    desc.depthStencil.depthCompare = CompareFunction::always;
    desc.label = Some("ore_depth_write_always");
    let always = host
        .ore
        .borrow_mut()
        .makePipeline(&desc, None)
        .expect("always pipeline");
    host.begin_ore();
    let mut pass_desc = pass_desc(
        &color_target,
        Some("ore_depth_write_always_pass"),
        [0.1, 0.1, 0.1, 1.0],
    );
    pass_desc.depthStencil.view = Some(&depth_view);
    pass_desc.depthStencil.depthLoadOp = LoadOp::clear;
    pass_desc.depthStencil.depthStoreOp = StoreOp::store;
    pass_desc.depthStencil.depthClearValue = 1.0;
    let mut pass = host
        .ore
        .borrow_mut()
        .beginRenderPass(&pass_desc, None)
        .expect("depth pass");
    pass.setPipeline(Some(if first_always { &always } else { &pipeline }));
    pass.setVertexBuffer(0, Some(&vbo), 0);
    pass.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
    if first_always {
        pass.draw(6, 1, 0, 0);
        pass.setPipeline(Some(&pipeline));
        pass.draw(12, 1, 6, 0);
    } else {
        // The original ore_depth GM, whose golden this new scene must match.
        pass.draw(18, 1, 0, 0);
    }
    pass.finish();
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    host.finish()
}

#[test]
fn ore_depth_write_always() {
    let pixels = scene(true);
    assert_pixels_equal("ore_depth_write_always", 1, &pixels, &scene(false));
    for (x, y, color) in [
        (128, 128, [255, 0, 0, 255]),
        (180, 180, [0, 255, 0, 255]),
        (210, 210, [0, 0, 255, 255]),
    ] {
        let offset = (y * 256 + x) * 4;
        assert_eq!(&pixels[offset..offset + 4], &color);
    }
}
