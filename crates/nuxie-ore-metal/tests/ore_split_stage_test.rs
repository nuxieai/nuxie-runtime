#![cfg(all(target_vendor = "apple", feature = "metal-backend"))]
//! Literal Metal lane of upstream 50ba2f5a ore_split_stage_test.cpp.
#[path = "support/depth_sample_shader.rs"]
mod depth_shader;
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
#[cfg(feature = "with-rive-tools")]
fn ore_depth_samplers_use_non_filtering_layouts() {
    for (bytes, expected) in [
        (depth_shader::MSL, depth_shader::MSL_SHA256),
        (depth_shader::MAP, depth_shader::MAP_SHA256),
    ] {
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), expected);
    }
    let Some(device) = MTLCreateSystemDefaultDevice() else {
        eprintln!("no Ore Metal backend available; skipping");
        return;
    };
    let queue = device.newCommandQueue().expect("Metal command queue");
    let mut ctx = *ContextMetal::MakeChecked(Some(device), Some(queue)).expect("Metal ORE context");
    run_depth_sampler_scenario(&mut ctx);
    run_depth_sampler_set_scenario(&mut ctx);
}

#[cfg(feature = "with-rive-tools")]
fn run_depth_sampler_scenario(ctx: &mut ContextMetal) {
    let depth = ctx
        .makeShaderModule(&ShaderModuleDesc {
            code: Some(depth_shader::MSL),
            codeSize: depth_shader::MSL.len() as u32,
            bindingMapBytes: Some(depth_shader::MAP),
            bindingMapSize: depth_shader::MAP.len() as u32,
            texSamplerPairBytes: Some(depth_shader::PAIRS),
            texSamplerPairSize: depth_shader::PAIRS.len() as u32,
            ..Default::default()
        })
        .expect("depth shader");
    let depth_base = depth.shaderModuleBase().unwrap();
    assert!(!depth_base.m_textureSamplerPairs.is_empty());
    let find_sampler = |layout: &BindGroupLayoutHandle| {
        *layout
            .bindGroupLayoutBase()
            .unwrap()
            .entries()
            .iter()
            .find(|entry| entry.kind == BindingKind::sampler)
            .expect("sampler")
    };
    let from_shader = makeBindGroupLayoutFromShader(ctx, Some(depth_base), 1, &[]).unwrap();
    assert!(find_sampler(&from_shader).samplerNonFiltering);
    assert!(
        makeBindGroupLayoutFromShader(ctx, Some(depth_base), 1, &[])
            .unwrap()
            .ptr_eq(&from_shader)
    );
    let vertex = ctx
        .makeShaderModule(&ShaderModuleDesc {
            code: Some(shaders::TRIANGLE_MSL),
            codeSize: shaders::TRIANGLE_MSL.len() as u32,
            bindingMapBytes: Some(shaders::TRIANGLE_MAP),
            bindingMapSize: shaders::TRIANGLE_MAP.len() as u32,
            ..Default::default()
        })
        .expect("vertex shader");
    let merged = bindingMapForStages(vertex.shaderModuleBase(), Some(depth_base));
    let merged_layout = makeBindGroupLayoutFromBindingMap(
        ctx,
        &merged,
        1,
        &[],
        vertex.shaderModuleBase(),
        Some(depth_base),
    )
    .unwrap();
    assert!(find_sampler(&merged_layout).samplerNonFiltering);
    let unflagged =
        makeBindGroupLayoutFromBindingMap(ctx, &depth_base.m_bindingMap, 1, &[], None, None)
            .unwrap();
    assert!(!find_sampler(&unflagged).samplerNonFiltering);
    assert!(!unflagged.ptr_eq(&from_shader));
}

#[cfg(feature = "with-rive-tools")]
fn run_depth_sampler_set_scenario(ctx: &mut ContextMetal) {
    use nuxie_ore_metal::binding_map::{
        BindingMap, Entry, ResourceKind, Stage, TextureSampleType, TextureViewDim,
    };
    use nuxie_ore_metal::shader_module::TextureSamplerPair;
    let mut map = BindingMap::default();
    let mut push = |binding, kind, sample_type| {
        let mut entry = Entry {
            group: 1,
            binding,
            kind,
            stageMask: BindingMap::kStageFragment as u8,
            backendSpace: 1,
            ..Default::default()
        };
        entry.backendSlot[Stage::FS as usize] = binding as u16;
        if kind == ResourceKind::SampledTexture {
            entry.textureViewDim = TextureViewDim::D2;
            entry.textureSampleType = sample_type;
        }
        map.push(&entry);
    };
    push(0, ResourceKind::Sampler, TextureSampleType::Undefined);
    push(1, ResourceKind::Sampler, TextureSampleType::Undefined);
    push(2, ResourceKind::SampledTexture, TextureSampleType::Depth);
    push(3, ResourceKind::SampledTexture, TextureSampleType::Float);
    map.finalize();
    map.computeLayoutIds();
    assert_ne!(map.layoutIdForGroup(1), BindingMap::kNoLayoutId);
    let pair = |texture_binding, sampler_binding| TextureSamplerPair {
        textureGroup: 1,
        textureBinding: texture_binding,
        samplerGroup: 1,
        samplerBinding: sampler_binding,
    };
    let mut depth_on_sampler0 = nuxie_ore_metal::new_shader_module_backend_base();
    depth_on_sampler0.m_textureSamplerPairs = vec![pair(2, 0), pair(3, 1)];
    let mut depth_on_sampler1 = nuxie_ore_metal::new_shader_module_backend_base();
    depth_on_sampler1.m_textureSamplerPairs = vec![pair(2, 1), pair(3, 0)];
    let flagged = |layout: &BindGroupLayoutHandle, binding| {
        layout
            .bindGroupLayoutBase()
            .unwrap()
            .entries()
            .iter()
            .find(|entry| entry.kind == BindingKind::sampler && entry.binding == binding)
            .is_some_and(|entry| entry.samplerNonFiltering)
    };
    let first =
        makeBindGroupLayoutFromBindingMap(ctx, &map, 1, &[], None, Some(&depth_on_sampler0))
            .unwrap();
    assert!(flagged(&first, 0));
    assert!(!flagged(&first, 1));
    let second =
        makeBindGroupLayoutFromBindingMap(ctx, &map, 1, &[], None, Some(&depth_on_sampler1))
            .unwrap();
    assert!(!second.ptr_eq(&first));
    assert!(!flagged(&second, 0));
    assert!(flagged(&second, 1));
    assert!(
        makeBindGroupLayoutFromBindingMap(ctx, &map, 1, &[], None, Some(&depth_on_sampler0))
            .unwrap()
            .ptr_eq(&first)
    );
}

#[test]
fn ore_binds_a_fragment_compiled_apart_from_its_vertex() {
    for (bytes, expected) in [
        (
            shaders::TRIANGLE_MSL,
            "cc85c7bc7bf44cf7ee3dc2d079aaaa52f3432b3d63d4c227362c50d57e3e2602",
        ),
        (
            shaders::TRIANGLE_MAP,
            "f32915355c43ff55efe50d9c681cdd9b7ae115dd58ad8f1c07ee293825d61a85",
        ),
        (
            shaders::WITNESS_MSL,
            "0ce0f88f6a5307082f3c190008801f2658382d8de1b9f12d7f756edeaa258d97",
        ),
        (
            shaders::WITNESS_MAP,
            "4cfb974117d1c41d54c801cc327fcceebc34a3ca02491ed0b3272a48d0670025",
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
    let layout = makeBindGroupLayoutFromBindingMap(
        &mut ctx,
        &merged,
        0,
        &[],
        vertex.shaderModuleBase(),
        fragment.shaderModuleBase(),
    )
    .unwrap();
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
