//! Upstream tests/unit_tests/renderer/ore_make_recording_test.cpp at 707c4f60.
use nuxie_ore_metal::{
    cmd::command_stream::WirePod,
    ore_cmd::{
        ore_command_buffer::*, ore_commands::*, ore_make_recording::*, ore_make_replay::decodePods,
        ore_resource_commands::*,
    },
    types::*,
};
fn blob<'a>(r: &OreCommandReader<'a>, b: BlobRef) -> &'a [u8] {
    if b.absent() {
        &[]
    } else {
        r.blob_at(b.offset, b.size)
    }
}
fn cstr<'a>(r: &OreCommandReader<'a>, b: BlobRef) -> &'a str {
    std::str::from_utf8(blob(r, b).split(|b| *b == 0).next().unwrap()).unwrap()
}
#[test]
fn make_stream_records_makes_with_the_callers_ids() {
    let mut cb = OreCommandBuffer::default();
    let verts = encodePods(&[1u32, 2, 3, 4]);
    recordMakeBuffer(
        &mut cb,
        0,
        1,
        &BufferDesc {
            usage: BufferUsage::vertex,
            size: verts.len() as u32,
            data: Some(&verts),
            immutable: true,
            label: Some("vb"),
        },
    );
    recordMakeTexture(
        &mut cb,
        1,
        1,
        &TextureDesc {
            width: 256,
            height: 128,
            depthOrArrayLayers: 1,
            format: TextureFormat::rgba8unorm,
            r#type: TextureType::texture2D,
            renderTarget: true,
            numMipmaps: 1,
            sampleCount: 4,
            label: Some("rt"),
        },
    );
    recordMakeSampler(
        &mut cb,
        2,
        3,
        &SamplerDesc {
            minFilter: Filter::linear,
            magFilter: Filter::nearest,
            mipmapFilter: Filter::linear,
            wrapU: WrapMode::repeat,
            wrapV: WrapMode::clampToEdge,
            wrapW: WrapMode::mirrorRepeat,
            compare: CompareFunction::less,
            minLod: 0.5,
            maxLod: 7.,
            maxAnisotropy: 8,
            label: None,
        },
    );
    let mut r = OreCommandReader::new(cb.command_bytes(), cb.blob_bytes());
    assert_eq!(r.next(), Some(CommandType::makeBuffer));
    let h: MakeResourcePOD = r.read();
    assert_eq!(h.id, 0);
    assert_eq!(h.generation, 1);
    let b: BufferDescPOD = r.read();
    assert_eq!(b.usage, BufferUsage::vertex);
    assert_eq!(b.size as usize, verts.len());
    assert!(b.immutable);
    assert_eq!(blob(&r, b.data).len(), verts.len());
    assert_eq!(blob(&r, b.data), verts);
    assert_eq!(blob(&r, b.label).len(), 3);
    assert_eq!(cstr(&r, b.label), "vb");
    assert_eq!(r.next(), Some(CommandType::makeTexture));
    assert_eq!(r.read::<MakeResourcePOD>().id, 1);
    let t: TextureDescPOD = r.read();
    assert_eq!(t.width, 256);
    assert_eq!(t.height, 128);
    assert_eq!(t.format, TextureFormat::rgba8unorm);
    assert_eq!(t.r#type, TextureType::texture2D);
    assert!(t.renderTarget);
    assert_eq!(t.sampleCount, 4);
    assert_eq!(cstr(&r, t.label), "rt");
    assert_eq!(r.next(), Some(CommandType::makeSampler));
    let h: MakeResourcePOD = r.read();
    assert_eq!(h.id, 2);
    assert_eq!(h.generation, 3);
    let s: SamplerDescPOD = r.read();
    assert_eq!(s.minFilter, Filter::linear);
    assert_eq!(s.magFilter, Filter::nearest);
    assert_eq!(s.wrapU, WrapMode::repeat);
    assert_eq!(s.wrapW, WrapMode::mirrorRepeat);
    assert_eq!(s.compare, CompareFunction::less);
    assert_eq!(s.minLod, 0.5);
    assert_eq!(s.maxLod, 7.);
    assert_eq!(s.maxAnisotropy, 8);
    assert!(s.label.absent());
    assert!(r.next::<CommandType>().is_none());
}
#[test]
fn buffer_without_initial_data_is_absent_not_empty() {
    let mut cb = OreCommandBuffer::default();
    recordMakeBuffer(
        &mut cb,
        0,
        0,
        &BufferDesc {
            usage: BufferUsage::uniform,
            size: 64,
            data: None,
            immutable: false,
            label: None,
        },
    );
    let mut r = OreCommandReader::new(cb.command_bytes(), cb.blob_bytes());
    assert!(r.next::<CommandType>().is_some());
    r.read::<MakeResourcePOD>();
    let b: BufferDescPOD = r.read();
    assert_eq!(b.size, 64);
    assert!(b.data.absent());
    assert_eq!(blob(&r, b.data).len(), 0);
}
#[test]
fn make_stream_records_shader_module_layout_view() {
    let mut cb = OreCommandBuffer::default();
    let code = [0xde, 0xad, 0xbe, 0xef, 1, 2, 3, 4];
    let map = [9, 8, 7];
    let pairs = [0, 1, 2, 3, 4, 5, 6, 7];
    recordMakeShaderModule(
        &mut cb,
        0,
        0,
        &ShaderModuleDesc {
            code: Some(&code),
            codeSize: 8,
            language: ShaderLanguage::wgsl,
            stage: ShaderStage::vertex,
            bindingMapBytes: Some(&map),
            bindingMapSize: 3,
            texSamplerPairBytes: Some(&pairs),
            texSamplerPairSize: pairs.len() as u32,
            shaderAssetId: 42,
            ..Default::default()
        },
    );
    let entries = [
        BindGroupLayoutEntry {
            binding: 0,
            kind: BindingKind::uniformBuffer,
            hasDynamicOffset: true,
            ..Default::default()
        },
        BindGroupLayoutEntry {
            binding: 1,
            kind: BindingKind::sampledTexture,
            nativeSlotFS: 5,
            ..Default::default()
        },
    ];
    recordMakeBindGroupLayout(
        &mut cb,
        1,
        0,
        &BindGroupLayoutDesc {
            groupIndex: 2,
            entries: Some(&entries),
            entryCount: 2,
            ..Default::default()
        },
    );
    recordMakeTexture(
        &mut cb,
        2,
        0,
        &TextureDesc {
            width: 64,
            height: 64,
            ..Default::default()
        },
    );
    recordMakeTextureView(
        &mut cb,
        3,
        0,
        &TextureViewDesc {
            dimension: TextureViewDimension::texture2D,
            baseMipLevel: 1,
            mipCount: 2,
            ..Default::default()
        },
        2,
    );
    let mut r = OreCommandReader::new(cb.command_bytes(), cb.blob_bytes());
    assert_eq!(r.next(), Some(CommandType::makeShaderModule));
    r.read::<MakeResourcePOD>();
    let s: ShaderModuleDescPOD = r.read();
    assert_eq!(s.language, ShaderLanguage::wgsl);
    assert_eq!(s.stage, ShaderStage::vertex);
    assert_eq!(s.shaderAssetId, 42);
    assert_eq!(blob(&r, s.code).len(), code.len());
    assert_eq!(blob(&r, s.code), code);
    assert_eq!(blob(&r, s.bindingMapBytes).len(), map.len());
    assert_eq!(blob(&r, s.texSamplerPairBytes), pairs);
    assert!(s.hlslSource.absent());
    assert!(s.label.absent());
    assert_eq!(r.next(), Some(CommandType::makeBindGroupLayout));
    r.read::<MakeResourcePOD>();
    let l: BindGroupLayoutDescPOD = r.read();
    assert_eq!(l.groupIndex, 2);
    assert_eq!(l.entryCount, 2);
    let bytes = blob(&r, l.entries);
    assert_eq!(bytes.len(), 2 * std::mem::size_of::<BindGroupLayoutEntry>());
    let entries = decodePods::<BindGroupLayoutEntry>(bytes, 2);
    assert_eq!(entries[0].binding, 0);
    assert!(entries[0].hasDynamicOffset);
    assert_eq!(entries[1].binding, 1);
    assert_eq!(entries[1].kind, BindingKind::sampledTexture);
    assert_eq!(entries[1].nativeSlotFS, 5);
    assert_eq!(r.next(), Some(CommandType::makeTexture));
    r.read::<MakeResourcePOD>();
    r.read::<TextureDescPOD>();
    assert_eq!(r.next(), Some(CommandType::makeTextureView));
    assert_eq!(r.read::<MakeResourcePOD>().id, 3);
    let v: TextureViewDescPOD = r.read();
    assert_eq!(v.texture, 2);
    assert_eq!(v.baseMipLevel, 1);
    assert_eq!(v.mipCount, 2);
}
#[test]
fn make_stream_records_pipeline_vertex_layouts_and_refs() {
    let mut cb = OreCommandBuffer::default();
    let attrs = [
        VertexAttribute {
            format: VertexFormat::float2,
            offset: 0,
            shaderSlot: 0,
        },
        VertexAttribute {
            format: VertexFormat::float4,
            offset: 8,
            shaderSlot: 1,
        },
    ];
    let vbs = [VertexBufferLayout {
        stride: 24,
        stepMode: VertexStepMode::vertex,
        attributes: Some(&attrs),
        attributeCount: 2,
    }];
    let mut p = PipelineDesc {
        vertexEntryPoint: Some("vs_main"),
        fragmentEntryPoint: Some("fs_main"),
        vertexBuffers: Some(&vbs),
        vertexBufferCount: 1,
        topology: PrimitiveTopology::triangleList,
        colorCount: 1,
        sampleCount: 4,
        label: Some("pipe"),
        ..Default::default()
    };
    p.colorTargets[0].format = TextureFormat::rgba8unorm;
    p.colorTargets[0].blendEnabled = true;
    recordMakePipeline(&mut cb, 0, 0, &p, 10, 11, &[12, 13]);
    let mut r = OreCommandReader::new(cb.command_bytes(), cb.blob_bytes());
    assert_eq!(r.next(), Some(CommandType::makePipeline));
    r.read::<MakeResourcePOD>();
    let p: PipelineDescPOD = r.read();
    assert_eq!(p.vertexModule, 10);
    assert_eq!(p.fragmentModule, 11);
    assert_eq!(p.colorCount, 1);
    assert_eq!(p.colorTargets[0].format, TextureFormat::rgba8unorm);
    assert!(p.colorTargets[0].blendEnabled);
    assert_eq!(p.sampleCount, 4);
    assert_eq!(cstr(&r, p.vertexEntryPoint), "vs_main");
    assert_eq!(p.bindGroupLayoutCount, 2);
    let bytes = blob(&r, p.bindGroupLayouts);
    assert_eq!(bytes.len(), 2 * std::mem::size_of::<u32>());
    let bgl = decodePods::<u32>(bytes, 2);
    assert_eq!(bgl[0], 12);
    assert_eq!(bgl[1], 13);
    assert_eq!(p.vertexBufferCount, 1);
    let bytes = blob(&r, p.vertexBuffers);
    assert_eq!(bytes.len(), std::mem::size_of::<VertexBufferLayoutPOD>());
    let vb = VertexBufferLayoutPOD::decode(bytes);
    assert_eq!(vb.stride, 24);
    assert_eq!(vb.attributeCount, 2);
    let bytes = blob(&r, vb.attributes);
    assert_eq!(bytes.len(), 2 * std::mem::size_of::<VertexAttribute>());
    let attrs = decodePods::<VertexAttribute>(bytes, 2);
    assert_eq!(attrs[0].format, VertexFormat::float2);
    assert_eq!(attrs[1].format, VertexFormat::float4);
    assert_eq!(attrs[1].offset, 8);
    assert_eq!(attrs[1].shaderSlot, 1);
}
#[test]
fn make_stream_records_bind_group_entry_refs() {
    let mut cb = OreCommandBuffer::default();
    let ubos = [UBOEntry {
        slot: 0,
        offset: 16,
        size: 256,
        ..Default::default()
    }];
    let texs = [TexEntry {
        slot: 1,
        ..Default::default()
    }];
    let samps = [SampEntry {
        slot: 2,
        ..Default::default()
    }];
    recordMakeBindGroup(
        &mut cb,
        0,
        0,
        &BindGroupDesc {
            layout: None,
            ubos: &ubos,
            uboCount: 1,
            textures: &texs,
            textureCount: 1,
            samplers: &samps,
            samplerCount: 1,
            label: Some("bg"),
        },
        5,
        &[6],
        &[7],
        &[8],
    );
    let mut r = OreCommandReader::new(cb.command_bytes(), cb.blob_bytes());
    assert_eq!(r.next(), Some(CommandType::makeBindGroup));
    r.read::<MakeResourcePOD>();
    let b: BindGroupDescPOD = r.read();
    assert_eq!(b.layout, 5);
    assert_eq!(b.uboCount, 1);
    assert_eq!(b.textureCount, 1);
    assert_eq!(b.samplerCount, 1);
    let u = UBOEntryPOD::decode(blob(&r, b.ubos));
    assert_eq!(u.slot, 0);
    assert_eq!(u.buffer, 6);
    assert_eq!(u.offset, 16);
    assert_eq!(u.size, 256);
    let t = TexEntryPOD::decode(blob(&r, b.textures));
    assert_eq!(t.slot, 1);
    assert_eq!(t.view, 7);
    let s = SampEntryPOD::decode(blob(&r, b.samplers));
    assert_eq!(s.slot, 2);
    assert_eq!(s.sampler, 8);
}

#[test]
fn make_stream_records_equal_pipelines_byte_for_byte() {
    fn depth_stencil_with_padding(fill: u8) -> DepthStencilState {
        let mut storage = std::mem::MaybeUninit::<DepthStencilState>::uninit();
        unsafe {
            std::ptr::write_bytes(
                storage.as_mut_ptr().cast::<u8>(),
                fill,
                std::mem::size_of::<DepthStencilState>(),
            );
            let ptr = storage.as_mut_ptr();
            std::ptr::addr_of_mut!((*ptr).format).write(TextureFormat::depth24plusStencil8);
            std::ptr::addr_of_mut!((*ptr).depthCompare).write(CompareFunction::always);
            std::ptr::addr_of_mut!((*ptr).depthWriteEnabled).write(true);
            std::ptr::addr_of_mut!((*ptr).depthBias).write(2);
            std::ptr::addr_of_mut!((*ptr).depthBiasSlopeScale).write(0.0);
            std::ptr::addr_of_mut!((*ptr).depthBiasClamp).write(0.0);
            storage.assume_init()
        }
    }

    fn record(stack_fill: u8, cb: &mut OreCommandBuffer) {
        let mut desc = PipelineDesc {
            vertexEntryPoint: Some("vs_main"),
            fragmentEntryPoint: Some("fs_main"),
            colorCount: 1,
            depthStencil: depth_stencil_with_padding(stack_fill),
            sampleCount: 4,
            label: Some("pipe"),
            ..Default::default()
        };
        desc.colorTargets[0].format = TextureFormat::rgba8unorm;
        desc.colorTargets[0].blendEnabled = true;
        desc.stencilFront.compare = CompareFunction::equal;
        recordMakePipeline(cb, 0, 0, &desc, 10, 11, &[12]);
    }

    let mut zeroed = OreCommandBuffer::default();
    let mut dirty = OreCommandBuffer::default();
    record(0, &mut zeroed);
    record(0xab, &mut dirty);
    assert_eq!(zeroed.command_bytes(), dirty.command_bytes());
}

#[test]
fn make_stream_reset_reuses_the_buffer() {
    let mut cb = OreCommandBuffer::default();
    let desc = TextureDesc {
        width: 16,
        height: 16,
        ..Default::default()
    };
    recordMakeTexture(&mut cb, 0, 0, &desc);
    recordMakeTexture(&mut cb, 1, 0, &desc);
    assert!(!cb.empty());
    cb.reset();
    assert!(cb.empty());
    recordMakeTexture(&mut cb, 0, 1, &desc);
    assert!(!cb.empty());
}
