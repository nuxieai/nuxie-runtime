//! Upstream tests/unit_tests/renderer/ore_make_recording_test.cpp at 966499ff.
use nuxie_ore_metal::{
    cmd::command_stream::WirePod,
    context::{Context, ContextApi, FrameDescriptor, ShaderTarget},
    gpu_resource::AnyResourceHandle,
    new_context_backend_base,
    ore_cmd::{
        ore_command_buffer::*, ore_commands::*, ore_make_recording::*, ore_make_replay::*,
        ore_resource_commands::*,
    },
    render_pass::RenderPassApi,
    types::*,
};
use std::ffi::c_void;
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

// Every ShaderModuleDesc field has to reach the wire, and the three places
// that carry it are hand written mirrors of this struct. Destructuring all
// members here fails to compile when one is added, which is the reminder to
// carry it through ShaderModuleDescPOD, recordMakeShaderModule, and replay.
#[test]
fn shader_module_desc_fields_are_all_accounted_for() {
    let ShaderModuleDesc {
        code,
        codeSize,
        language,
        stage,
        label,
        hlslSource,
        hlslSourceSize,
        hlslEntryPoint,
        bindingMapBytes,
        bindingMapSize,
        texSamplerPairBytes,
        texSamplerPairSize,
        glFixupBytes,
        glFixupSize,
        shaderAssetId,
    } = ShaderModuleDesc::default();
    let _ = (
        code,
        codeSize,
        language,
        stage,
        label,
        hlslSource,
        hlslSourceSize,
        hlslEntryPoint,
        bindingMapBytes,
        bindingMapSize,
        texSamplerPairBytes,
        texSamplerPairSize,
        glFixupBytes,
        glFixupSize,
        shaderAssetId,
    );
}

#[derive(Debug, PartialEq)]
struct CapturedShaderModuleDesc {
    code: Option<Vec<u8>>,
    codeSize: u32,
    language: ShaderLanguage,
    stage: ShaderStage,
    label: Option<String>,
    hlslSource: Option<String>,
    hlslSourceSize: u32,
    hlslEntryPoint: Option<String>,
    bindingMapBytes: Option<Vec<u8>>,
    bindingMapSize: u32,
    texSamplerPairBytes: Option<Vec<u8>>,
    texSamplerPairSize: u32,
    glFixupBytes: Option<Vec<u8>>,
    glFixupSize: u32,
    shaderAssetId: u32,
}

struct CapturingContext {
    base: Context,
    captured: Option<CapturedShaderModuleDesc>,
}

impl CapturingContext {
    fn new() -> Self {
        Self {
            base: new_context_backend_base(Features::default(), None),
            captured: None,
        }
    }
}

impl ContextApi for CapturingContext {
    fn contextBase(&self) -> &Context {
        &self.base
    }
    fn features(&self) -> Features {
        self.base.features()
    }
    fn lastError(&self) -> String {
        self.base.lastError()
    }
    fn activeRenderPass(
        &self,
    ) -> Option<std::rc::Weak<dyn nuxie_ore_metal::context::ActiveRenderPass>> {
        self.base.activeRenderPass()
    }
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.base.setActiveRenderPass(pass);
    }
    fn finishActiveRenderPass(&self) {
        self.base.finishActiveRenderPass();
    }
    fn clearLastError(&self) {
        self.base.clearLastError();
    }
    fn setLastError(&self, message: &str) {
        self.base.setLastError(message);
    }
    fn makeBuffer(&mut self, _: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeTexture(&mut self, _: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeTextureView(&mut self, _: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeSampler(&mut self, _: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        self.captured = Some(CapturedShaderModuleDesc {
            code: desc.code.map(<[u8]>::to_vec),
            codeSize: desc.codeSize,
            language: desc.language,
            stage: desc.stage,
            label: desc.label.map(str::to_owned),
            hlslSource: desc.hlslSource.map(str::to_owned),
            hlslSourceSize: desc.hlslSourceSize,
            hlslEntryPoint: desc.hlslEntryPoint.map(str::to_owned),
            bindingMapBytes: desc.bindingMapBytes.map(<[u8]>::to_vec),
            bindingMapSize: desc.bindingMapSize,
            texSamplerPairBytes: desc.texSamplerPairBytes.map(<[u8]>::to_vec),
            texSamplerPairSize: desc.texSamplerPairSize,
            glFixupBytes: desc.glFixupBytes.map(<[u8]>::to_vec),
            glFixupSize: desc.glFixupSize,
            shaderAssetId: desc.shaderAssetId,
        });
        None
    }
    fn makeBindGroupLayout(&mut self, _: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makePipeline(
        &mut self,
        _: &PipelineDesc<'_>,
        _: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        None
    }
    fn makeBindGroup(&mut self, _: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn beginRenderPass(
        &mut self,
        _: &RenderPassDesc<'_>,
        _: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        None
    }
    fn beginFrame(&mut self, _: &FrameDescriptor) {}
    fn endFrame(&mut self) {}
    fn waitForGPU(&mut self) {}
    unsafe fn wrapCanvasTexture(&mut self, _: *mut c_void) -> Option<AnyResourceHandle> {
        None
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        _: *mut c_void,
        _: u32,
        _: u32,
    ) -> Option<AnyResourceHandle> {
        None
    }
    fn shaderTarget(&self) -> ShaderTarget {
        ShaderTarget::glsl
    }
}

#[test]
fn every_shader_module_desc_field_survives_record_and_replay() {
    let code = [0xde, 0xad, 0xbe, 0xef, 1, 2, 3, 4];
    let bmap = [9, 8, 7];
    let pairs = [1, 1, 0, 0, 1, 2, 0, 0];
    let fixup = [4, 5];
    let hlsl_source = "float4 main() : SV_Target { return 0; }";
    let sent = ShaderModuleDesc {
        code: Some(&code),
        codeSize: code.len() as u32,
        language: ShaderLanguage::wgsl,
        stage: ShaderStage::fragment,
        label: Some("every_field"),
        hlslSource: Some(hlsl_source),
        hlslSourceSize: hlsl_source.len() as u32,
        hlslEntryPoint: Some("main"),
        bindingMapBytes: Some(&bmap),
        bindingMapSize: bmap.len() as u32,
        texSamplerPairBytes: Some(&pairs),
        texSamplerPairSize: pairs.len() as u32,
        glFixupBytes: Some(&fixup),
        glFixupSize: fixup.len() as u32,
        shaderAssetId: 4242,
    };

    let mut cb = OreCommandBuffer::default();
    recordMakeShaderModule(&mut cb, 0, 3, &sent);

    let mut ctx = CapturingContext::new();
    let mut table = OreResident::default();
    let mut r = OreCommandReader::new(cb.command_bytes(), cb.blob_bytes());
    let command = r.next().expect("recorded shader-module command");
    assert_eq!(command, CommandType::makeShaderModule);
    assert!(replayOreLifecycle(
        &mut ctx,
        &mut table,
        command,
        &mut r,
        &mut |_| None,
        &mut |_| None,
        &mut |_| None,
    ));

    assert_eq!(
        ctx.captured,
        Some(CapturedShaderModuleDesc {
            code: Some(code.to_vec()),
            codeSize: sent.codeSize,
            language: sent.language,
            stage: sent.stage,
            label: Some("every_field".to_owned()),
            hlslSource: Some(hlsl_source.to_owned()),
            hlslSourceSize: sent.hlslSourceSize,
            hlslEntryPoint: Some("main".to_owned()),
            bindingMapBytes: Some(bmap.to_vec()),
            bindingMapSize: sent.bindingMapSize,
            texSamplerPairBytes: Some(pairs.to_vec()),
            texSamplerPairSize: sent.texSamplerPairSize,
            glFixupBytes: Some(fixup.to_vec()),
            glFixupSize: sent.glFixupSize,
            shaderAssetId: sent.shaderAssetId,
        })
    );
}

#[test]
fn make_stream_records_shader_module_layout_view() {
    let mut cb = OreCommandBuffer::default();
    let code = [0xde, 0xad, 0xbe, 0xef, 1, 2, 3, 4];
    let map = [9, 8, 7];
    let pairs = [1, 1, 0, 0, 1, 2, 0, 0];
    let fixup = [4, 5];
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
            glFixupBytes: Some(&fixup),
            glFixupSize: fixup.len() as u32,
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
    assert_eq!(blob(&r, s.bindingMapBytes), map);
    assert_eq!(blob(&r, s.texSamplerPairBytes), pairs);
    assert_eq!(blob(&r, s.glFixupBytes).len(), fixup.len());
    assert_eq!(blob(&r, s.glFixupBytes), fixup);
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

const PADDING_TEST_VERTS: [u32; 4] = [1, 2, 3, 4];
const PADDING_TEST_CODE: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

#[inline(never)]
fn dirty_stack() {
    let scratch = [0xabu8; 4096];
    std::hint::black_box(&scratch);
}

fn record_one_of_each(cb: &mut OreCommandBuffer) {
    let verts = encodePods(&PADDING_TEST_VERTS);
    recordMakeBuffer(
        cb,
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
        cb,
        1,
        1,
        &TextureDesc {
            width: 256,
            height: 128,
            format: TextureFormat::rgba8unorm,
            renderTarget: true,
            numMipmaps: 1,
            sampleCount: 4,
            label: Some("rt"),
            ..Default::default()
        },
    );
    recordMakeSampler(
        cb,
        2,
        1,
        &SamplerDesc {
            minFilter: Filter::linear,
            magFilter: Filter::nearest,
            compare: CompareFunction::less,
            minLod: 0.5,
            maxLod: 7.0,
            maxAnisotropy: 8,
            ..Default::default()
        },
    );
    recordMakeShaderModule(
        cb,
        3,
        1,
        &ShaderModuleDesc {
            code: Some(&PADDING_TEST_CODE),
            codeSize: PADDING_TEST_CODE.len() as u32,
            language: ShaderLanguage::wgsl,
            stage: ShaderStage::fragment,
            label: Some("fs"),
            ..Default::default()
        },
    );

    let entries = [
        BindGroupLayoutEntry {
            binding: 0,
            kind: BindingKind::uniformBuffer,
            hasDynamicOffset: true,
            minBindingSize: 64,
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
        cb,
        4,
        1,
        &BindGroupLayoutDesc {
            groupIndex: 2,
            entries: Some(&entries),
            entryCount: 2,
            label: Some("bgl"),
        },
    );
    recordMakeTextureView(
        cb,
        5,
        1,
        &TextureViewDesc {
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            mipCount: 1,
            layerCount: 1,
            ..Default::default()
        },
        1,
    );

    let attrs = [
        VertexAttribute {
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
    let layouts = [VertexBufferLayout {
        stride: 24,
        attributes: Some(&attrs),
        attributeCount: 2,
        ..Default::default()
    }];
    let mut pipeline = PipelineDesc {
        vertexEntryPoint: Some("vs_main"),
        fragmentEntryPoint: Some("fs_main"),
        vertexBuffers: Some(&layouts),
        vertexBufferCount: 1,
        colorCount: 1,
        depthStencil: DepthStencilState {
            format: TextureFormat::depth24plusStencil8,
            depthWriteEnabled: true,
            depthBias: 2,
            ..Default::default()
        },
        sampleCount: 4,
        label: Some("pipe"),
        ..Default::default()
    };
    pipeline.colorTargets[0].format = TextureFormat::rgba8unorm;
    pipeline.colorTargets[0].blendEnabled = true;
    pipeline.stencilFront.compare = CompareFunction::equal;
    recordMakePipeline(cb, 6, 1, &pipeline, 3, 3, &[4]);

    let ubos = [UBOEntry {
        slot: 0,
        size: 256,
        ..Default::default()
    }];
    recordMakeBindGroup(
        cb,
        7,
        1,
        &BindGroupDesc {
            layout: None,
            ubos: &ubos,
            uboCount: 1,
            textures: &[],
            textureCount: 0,
            samplers: &[],
            samplerCount: 0,
            label: Some("bg"),
        },
        4,
        &[0],
        &[],
        &[],
    );
}

#[test]
fn make_stream_records_equal_resources_byte_for_byte() {
    let mut clean = OreCommandBuffer::default();
    let mut dirty = OreCommandBuffer::default();
    record_one_of_each(&mut clean);
    dirty_stack();
    record_one_of_each(&mut dirty);
    assert_eq!(clean.command_bytes(), dirty.command_bytes());
    assert_eq!(clean.blob_bytes(), dirty.blob_bytes());
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
