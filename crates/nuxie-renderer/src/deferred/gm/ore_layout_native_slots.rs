//! Metal scenario from tests/unit_tests/renderer/ore_layout_native_slots_test.cpp
//! at e3b9461b. Uses the live host; unsupported D3D branches are not imported.
use super::ore_gm_helper::*;

fn make_ubo(ctx: &mut dyn ContextApi, value: [f32; 4]) -> AnyResourceHandle {
    let bytes: Vec<u8> = value.into_iter().flat_map(f32::to_ne_bytes).collect();
    ctx.makeBuffer(&BufferDesc {
        usage: BufferUsage::uniform,
        size: bytes.len() as u32,
        data: Some(&bytes),
        immutable: false,
        label: None,
    })
    .expect("uniform buffer")
}

fn make_solid_texture(ctx: &mut dyn ContextApi, texel: [u8; 4]) -> AnyResourceHandle {
    let texture = ctx
        .makeTexture(&TextureDesc {
            width: 2,
            height: 2,
            format: TextureFormat::rgba8unorm,
            ..Default::default()
        })
        .expect("texture");
    let pixels = texel.repeat(4);
    texture
        .upload(&TextureDataDesc {
            data: Some(&pixels),
            dataSize: pixels.len() as u32,
            bytesPerRow: 8,
            width: 2,
            height: 2,
            ..Default::default()
        })
        .expect("upload");
    ctx.makeTextureView(&TextureViewDesc {
        texture: Some(&texture),
        ..Default::default()
    })
    .expect("texture view")
}

#[test]
fn ore_binds_a_layout_reflected_from_another_shader() {
    const SIZE: u32 = 32;
    let mut host = GmHost::with_size(0xff000000, true, SIZE, SIZE);
    let (pipeline, frame_group, material_group, unbound) = {
        let mut ctx = host.ore.borrow_mut();
        let lit = shader(&mut *ctx, 14);
        let shadow = shader(&mut *ctx, 15);
        let frame_layout = layout_from_shader(&mut *ctx, &shadow, 0);
        let material_layout = layout_from_shader(&mut *ctx, &lit, 1);
        assert_eq!(
            material_layout
                .bindGroupLayoutBase()
                .unwrap()
                .entries()
                .len(),
            4
        );
        let layouts = [Some(&frame_layout), Some(&material_layout)];
        let mut desc = PipelineDesc {
            vertexModule: Some(&shadow),
            fragmentModule: Some(&shadow),
            vertexEntryPoint: Some("vs_main"),
            fragmentEntryPoint: Some("fs_main"),
            topology: PrimitiveTopology::triangleList,
            colorCount: 1,
            bindGroupLayouts: Some(&layouts),
            bindGroupLayoutCount: 2,
            label: Some("ore_layout_native_slots_shadow"),
            ..Default::default()
        };
        desc.colorTargets[0].format = TextureFormat::rgba8unorm;
        desc.depthStencil.depthCompare = CompareFunction::always;
        desc.depthStencil.depthWriteEnabled = false;
        let mut error = String::new();
        let pipeline = ctx
            .makePipeline(&desc, Some(&mut error))
            .unwrap_or_else(|| panic!("makePipeline: {error}"));
        let frame_ubo = make_ubo(&mut *ctx, [0.0, 0.0, 0.4, 0.0]);
        let material_ubo = make_ubo(&mut *ctx, [1.0, 0.5, 1.0, 1.0]);
        let color_map = make_solid_texture(&mut *ctx, [0x33, 0xcc, 0x66, 0xff]);
        let emissive_map = make_solid_texture(&mut *ctx, [0xff, 0x00, 0xff, 0xff]);
        let sampler = ctx.makeSampler(&SamplerDesc::default()).expect("sampler");
        let frame_entries = [UBOEntry {
            slot: 0,
            buffer: Some(&frame_ubo),
            size: 16,
            ..Default::default()
        }];
        let frame_group = ctx
            .makeBindGroup(&BindGroupDesc {
                layout: Some(&frame_layout),
                ubos: &frame_entries,
                uboCount: 1,
                ..Default::default()
            })
            .unwrap_or_else(|| panic!("makeBindGroup frame: {}", ctx.lastError()));
        let material_entries = [UBOEntry {
            slot: 2,
            buffer: Some(&material_ubo),
            size: 16,
            ..Default::default()
        }];
        let textures = [
            TexEntry {
                slot: 0,
                view: Some(&color_map),
            },
            TexEntry {
                slot: 3,
                view: Some(&emissive_map),
            },
        ];
        let samplers = [SampEntry {
            slot: 1,
            sampler: Some(&sampler),
        }];
        let material_group = ctx
            .makeBindGroup(&BindGroupDesc {
                layout: Some(&material_layout),
                ubos: &material_entries,
                uboCount: 1,
                textures: &textures,
                textureCount: 2,
                samplers: &samplers,
                samplerCount: 1,
                ..Default::default()
            })
            .unwrap_or_else(|| panic!("makeBindGroup material: {}", ctx.lastError()));
        let triangle = shader(&mut *ctx, 0);
        let unbound = triangle_pipeline(
            &mut *ctx,
            &triangle,
            TextureFormat::rgba8unorm,
            "ore_layout_native_slots_unbound",
        );
        (pipeline, frame_group, material_group, unbound)
    };
    let canvas = host.canvas(SIZE, SIZE);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    host.begin_ore();
    let desc = pass_desc(
        &target,
        Some("ore_layout_native_slots_pass"),
        [0.0, 0.0, 1.0, 1.0],
    );
    let mut pass = host
        .ore
        .borrow_mut()
        .beginRenderPass(&desc, None)
        .expect("render pass");
    pass.setBindGroup(1, Some(&material_group), None, 0);
    pass.setPipeline(Some(&unbound));
    pass.setBindGroup(0, Some(&frame_group), None, 0);
    pass.setPipeline(Some(&pipeline));
    pass.setViewport(0.0, 0.0, SIZE as f32, SIZE as f32, 0.0, 1.0);
    pass.draw(3, 1, 0, 0);
    pass.finish();
    host.end_ore();
    // Upstream reads the canvas in the next screen frame.
    host.finish_frame();
    host.open_preserving_screen();
    host.screen().borrow_mut().draw_image(
        Some(canvas.borrow().render_image().as_ref()),
        ImageSampler {
            filter: ImageFilter::Nearest,
            ..Default::default()
        },
        BlendMode::SrcOver,
        1.0,
    );
    let pixels = host.finish();
    assert_eq!(pixels.len(), SIZE as usize * SIZE as usize * 4);
    let center = (SIZE as usize / 2 * SIZE as usize + SIZE as usize / 2) * 4;
    for (actual, expected) in pixels[center..center + 3].iter().zip([0x33u8, 0x66, 0xcc]) {
        assert!(
            actual.abs_diff(expected) <= 0x08,
            "center pixel = {:?}",
            &pixels[center..center + 3]
        );
    }
}
