#![cfg(all(target_vendor = "apple", feature = "metal-backend"))]
//! Literal Metal lane of upstream d7fff883 ore_split_stage_test.cpp.
#[path = "support/split_stage_shaders.rs"]
mod shaders;
use nuxie_ore_metal::bind_group_layout::{
    bindingMapForStages, makeBindGroupLayoutFromBindingMap, makeBindGroupLayoutFromShader,
};
use nuxie_ore_metal::metal::context::ContextMetal;
use nuxie_ore_metal::types::*;
use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};
use sha2::{Digest, Sha256};

#[test]
fn ore_binds_a_fragment_compiled_apart_from_its_vertex() {
    for (bytes, expected) in [
        (
            shaders::TRIANGLE_MSL,
            "cc85c7bc7bf44cf7ee3dc2d079aaaa52f3432b3d63d4c227362c50d57e3e2602",
        ),
        (
            shaders::TRIANGLE_MAP,
            "29468fb994f1fcdcdf80747eeff0bcf28c65cc34339ef785726d7b0cad435e6c",
        ),
        (
            shaders::WITNESS_MSL,
            "0ce0f88f6a5307082f3c190008801f2658382d8de1b9f12d7f756edeaa258d97",
        ),
        (
            shaders::WITNESS_MAP,
            "a4a2dd1b36656b0546ccb60d65de5a4cc09cafd7d3df873d1f412dd387541385",
        ),
    ] {
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), expected);
    }
    let Some(device) = MTLCreateSystemDefaultDevice() else {
        eprintln!("no Ore Metal backend available; skipping");
        return;
    };
    let queue = device.newCommandQueue().expect("Metal command queue");
    let mut ctx = *ContextMetal::MakeChecked(Some(device), Some(queue)).expect("Metal ORE context");
    let mut load = |code: &[u8], map: &[u8]| {
        ctx.makeShaderModule(&ShaderModuleDesc {
            code: Some(code),
            codeSize: code.len() as u32,
            bindingMapBytes: Some(map),
            bindingMapSize: map.len() as u32,
            ..ShaderModuleDesc::default()
        })
        .expect("compile exact upstream module")
    };
    let vertex = load(shaders::TRIANGLE_MSL, shaders::TRIANGLE_MAP);
    let fragment = load(shaders::WITNESS_MSL, shaders::WITNESS_MAP);
    assert!(!std::ptr::eq(
        vertex.shaderModuleBase().unwrap(),
        fragment.shaderModuleBase().unwrap()
    ));
    let vertex_only =
        makeBindGroupLayoutFromShader(&mut ctx, vertex.shaderModuleBase(), 0, &[]).unwrap();
    assert!(
        vertex_only
            .bindGroupLayoutBase()
            .unwrap()
            .entries()
            .is_empty()
    );
    let merged = bindingMapForStages(vertex.shaderModuleBase(), fragment.shaderModuleBase());
    let layout = makeBindGroupLayoutFromBindingMap(&mut ctx, &merged, 0, &[]).unwrap();
    let entries = layout.bindGroupLayoutBase().unwrap().entries();
    assert_eq!(entries.len(), 2);
    for entry in entries {
        assert_ne!(entry.visibility.mask & StageVisibility::kFragment, 0);
        assert_ne!(entry.nativeSlotFS, BindGroupLayoutEntry::kNativeSlotAbsent);
    }
    let layouts = [Some(&layout)];
    let attrs = [
        VertexAttribute {
            offset: 0,
            shaderSlot: 0,
            format: VertexFormat::float2,
            ..VertexAttribute::default()
        },
        VertexAttribute {
            offset: 8,
            shaderSlot: 1,
            format: VertexFormat::float4,
            ..VertexAttribute::default()
        },
    ];
    let vertex_buffers = [VertexBufferLayout {
        stride: 24,
        stepMode: VertexStepMode::vertex,
        attributes: Some(&attrs),
        attributeCount: 2,
    }];
    let mut desc = PipelineDesc::default();
    desc.vertexModule = Some(&vertex);
    desc.fragmentModule = Some(&fragment);
    desc.vertexEntryPoint = Some("vs_main");
    desc.fragmentEntryPoint = Some("fs_main");
    desc.vertexBuffers = Some(&vertex_buffers);
    desc.vertexBufferCount = 1;
    desc.topology = PrimitiveTopology::triangleList;
    desc.colorTargets[0].format = TextureFormat::rgba8unorm;
    desc.colorCount = 1;
    desc.depthStencil.depthCompare = CompareFunction::always;
    desc.depthStencil.depthWriteEnabled = false;
    desc.bindGroupLayouts = Some(&layouts);
    desc.bindGroupLayoutCount = 1;
    desc.label = Some("ore_split_stage_pipeline");
    let mut error = String::new();
    let pipeline = ctx.makePipeline(&desc, Some(&mut error));
    assert!(pipeline.is_some(), "makePipeline: {error}");
    let low: Vec<u8> = [0.3f32, 0.0, 0.0, 0.0]
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect();
    let high: Vec<u8> = [0.0f32, 0.6, 0.0, 0.0]
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect();
    let low_buffer = ctx
        .makeBuffer(&BufferDesc::initialized(BufferUsage::uniform, &low, false).unwrap())
        .unwrap();
    let high_buffer = ctx
        .makeBuffer(&BufferDesc::initialized(BufferUsage::uniform, &high, false).unwrap())
        .unwrap();
    let ubos = [
        UBOEntry {
            slot: entries[0].binding,
            buffer: Some(&low_buffer),
            size: 16,
            ..UBOEntry::default()
        },
        UBOEntry {
            slot: entries[1].binding,
            buffer: Some(&high_buffer),
            size: 16,
            ..UBOEntry::default()
        },
    ];
    let bind_group = ctx.makeBindGroup(&BindGroupDesc {
        layout: Some(&layout),
        ubos: &ubos,
        uboCount: 2,
        ..BindGroupDesc::default()
    });
    assert!(bind_group.is_some(), "makeBindGroup: {}", ctx.lastError());
}
