//! renderer/ore/cmd/ore_make_replay.hpp at e949498e.
#![allow(non_snake_case, non_camel_case_types)]
use super::{
    ore_command_buffer::OreCommandReader, ore_commands::*, ore_handle::*, ore_resource_commands::*,
};
use crate::{
    cmd::command_stream::WirePod,
    context::{CanvasTextureInfo, ContextApi},
    gpu_resource::AnyResourceHandle,
    types::*,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OreKind {
    #[default]
    none,
    buffer,
    texture,
    textureView,
    sampler,
    shaderModule,
    bindGroupLayout,
    pipeline,
    bindGroup,
}
#[derive(Default)]
pub struct OreResident {
    pub objects: Vec<Option<AnyResourceHandle>>,
    pub generations: Vec<u32>,
    pub kinds: Vec<OreKind>,
}
impl OreResident {
    pub fn set(&mut self, id: u32, obj: Option<AnyResourceHandle>, generation: u32, kind: OreKind) {
        let id = id as usize;
        if id > self.objects.len() {
            debug_assert!(false, "non-dense ORE resource id");
            return;
        }
        if id == self.objects.len() {
            self.objects.push(obj);
            self.generations.push(generation);
            self.kinds.push(kind);
            return;
        }
        self.objects[id] = obj;
        self.generations[id] = generation;
        self.kinds[id] = kind;
    }
    pub fn destroy(&mut self, id: u32, generation: u32) {
        let id = id as usize;
        if id < self.objects.len() && self.generations[id] == generation {
            self.objects[id] = None;
        }
    }
    pub fn get(&self, id: u32) -> Option<AnyResourceHandle> {
        self.objects.get(id as usize).and_then(Clone::clone)
    }
    pub fn getAs(&self, id: u32, kind: OreKind) -> Option<AnyResourceHandle> {
        if self.kinds.get(id as usize) == Some(&kind) {
            self.get(id)
        } else {
            None
        }
    }
    pub fn alive(&self, id: u32, generation: u32) -> bool {
        let id = id as usize;
        id < self.objects.len() && self.objects[id].is_some() && self.generations[id] == generation
    }
}
pub fn resolveOre(
    table: &OreResident,
    real: &mut dyn FnMut(u32) -> Option<AnyResourceHandle>,
    h: u32,
    kind: OreKind,
) -> Option<AnyResourceHandle> {
    if h == INVALID_HANDLE {
        None
    } else if h & REAL_RESOURCE_FLAG != 0 {
        real(h)
    } else {
        table.getAs(h, kind)
    }
}
macro_rules! warn_throttled {($($args:tt)*)=>{{static COUNT:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);if COUNT.fetch_add(1,std::sync::atomic::Ordering::Relaxed)%120==0 {eprintln!($($args)*);}}};}
pub(crate) use warn_throttled;
fn blob<'a>(reader: &OreCommandReader<'a>, r: BlobRef) -> Option<&'a [u8]> {
    if r.absent() {
        None
    } else {
        Some(reader.blob_at(r.offset, r.size))
    }
}
fn cstr<'a>(reader: &OreCommandReader<'a>, r: BlobRef) -> Option<&'a str> {
    blob(reader, r).map(|b| {
        std::str::from_utf8(b.split(|v| *v == 0).next().unwrap())
            .expect("recorded C string is UTF-8")
    })
}
pub fn decodePods<T: WirePod>(bytes: &[u8], count: u32) -> Vec<T> {
    (0..count as usize)
        .map(|i| T::decode(&bytes[i * T::SIZE..(i + 1) * T::SIZE]))
        .collect()
}
fn req(
    table: &OreResident,
    real: &mut dyn FnMut(u32) -> Option<AnyResourceHandle>,
    h: u32,
    kind: OreKind,
    unresolved: &mut bool,
) -> Option<AnyResourceHandle> {
    let r = resolveOre(table, real, h, kind);
    if r.is_none() && h != INVALID_HANDLE {
        *unresolved = true;
    }
    r
}
fn skipUnresolvedMake(table: &mut OreResident, id: u32, generation: u32, what: &str) -> bool {
    warn_throttled!(
        "rive ore replay: skip make {} id={} gen={} (unresolved dep, churn)",
        what,
        id,
        generation
    );
    table.set(id, None, generation, OreKind::none);
    true
}

pub fn replayOreLifecycle(
    ctx: &mut dyn ContextApi,
    table: &mut OreResident,
    kind: CommandType,
    reader: &mut OreCommandReader<'_>,
    real: &mut dyn FnMut(u32) -> Option<AnyResourceHandle>,
    canvasAt: &mut dyn FnMut(u32) -> Option<CanvasTextureInfo>,
    imageAt: &mut dyn FnMut(u32) -> Option<CanvasTextureInfo>,
) -> bool {
    let mut unresolved = false;
    match kind {
        CommandType::makeBuffer => {
            let m: MakeResourcePOD = reader.read();
            let p: BufferDescPOD = reader.read();
            if table.alive(m.id, m.generation) {
                return true;
            }
            let d = BufferDesc {
                usage: p.usage,
                size: p.size,
                immutable: p.immutable,
                data: blob(reader, p.data),
                label: cstr(reader, p.label),
            };
            table.set(m.id, ctx.makeBuffer(&d), m.generation, OreKind::buffer);
        }
        CommandType::makeTexture => {
            let m: MakeResourcePOD = reader.read();
            let p: TextureDescPOD = reader.read();
            if table.alive(m.id, m.generation) {
                return true;
            }
            let d = TextureDesc {
                width: p.width,
                height: p.height,
                depthOrArrayLayers: p.depthOrArrayLayers,
                format: p.format,
                r#type: p.r#type,
                renderTarget: p.renderTarget,
                numMipmaps: p.numMipmaps,
                sampleCount: p.sampleCount,
                label: cstr(reader, p.label),
            };
            table.set(m.id, ctx.makeTexture(&d), m.generation, OreKind::texture);
        }
        CommandType::makeSampler => {
            let m: MakeResourcePOD = reader.read();
            let p: SamplerDescPOD = reader.read();
            if table.alive(m.id, m.generation) {
                return true;
            }
            let d = SamplerDesc {
                minFilter: p.minFilter,
                magFilter: p.magFilter,
                mipmapFilter: p.mipmapFilter,
                wrapU: p.wrapU,
                wrapV: p.wrapV,
                wrapW: p.wrapW,
                compare: p.compare,
                minLod: p.minLod,
                maxLod: p.maxLod,
                maxAnisotropy: p.maxAnisotropy,
                label: cstr(reader, p.label),
            };
            table.set(m.id, ctx.makeSampler(&d), m.generation, OreKind::sampler);
        }
        CommandType::makeShaderModule => {
            let m: MakeResourcePOD = reader.read();
            let p: ShaderModuleDescPOD = reader.read();
            if table.alive(m.id, m.generation) {
                return true;
            }
            let code = blob(reader, p.code);
            let hlsl = blob(reader, p.hlslSource);
            let bindings = blob(reader, p.bindingMapBytes);
            let pairs = blob(reader, p.texSamplerPairBytes);
            let fixups = blob(reader, p.glFixupBytes);
            let d = ShaderModuleDesc {
                code,
                codeSize: code.map_or(0, |b| b.len() as u32),
                language: p.language,
                stage: p.stage,
                label: cstr(reader, p.label),
                // Unlike labels/entry points, HLSL is an explicitly sized
                // source span. Preserve embedded and trailing NUL bytes.
                hlslSource: hlsl
                    .map(|bytes| std::str::from_utf8(bytes).expect("recorded HLSL is UTF-8")),
                hlslSourceSize: hlsl.map_or(0, |b| b.len() as u32),
                hlslEntryPoint: cstr(reader, p.hlslEntryPoint),
                bindingMapBytes: bindings,
                bindingMapSize: bindings.map_or(0, |b| b.len() as u32),
                texSamplerPairBytes: pairs,
                texSamplerPairSize: pairs.map_or(0, |b| b.len() as u32),
                glFixupBytes: fixups,
                glFixupSize: fixups.map_or(0, |b| b.len() as u32),
                shaderAssetId: p.shaderAssetId,
            };
            table.set(
                m.id,
                ctx.makeShaderModule(&d),
                m.generation,
                OreKind::shaderModule,
            );
        }
        CommandType::makeBindGroupLayout => {
            let m: MakeResourcePOD = reader.read();
            let p: BindGroupLayoutDescPOD = reader.read();
            if table.alive(m.id, m.generation) {
                return true;
            }
            let entries = decodePods::<BindGroupLayoutEntry>(
                blob(reader, p.entries).unwrap_or(&[]),
                p.entryCount,
            );
            let d = BindGroupLayoutDesc {
                groupIndex: p.groupIndex,
                entries: (!p.entries.absent()).then_some(entries.as_slice()),
                entryCount: p.entryCount,
                label: cstr(reader, p.label),
            };
            table.set(
                m.id,
                ctx.makeBindGroupLayout(&d),
                m.generation,
                OreKind::bindGroupLayout,
            );
        }
        CommandType::makeTextureView => {
            let m: MakeResourcePOD = reader.read();
            let p: TextureViewDescPOD = reader.read();
            if table.alive(m.id, m.generation) {
                return true;
            }
            let texture = req(table, real, p.texture, OreKind::texture, &mut unresolved);
            let d = TextureViewDesc {
                texture: texture.as_ref(),
                dimension: p.dimension,
                aspect: p.aspect,
                baseMipLevel: p.baseMipLevel,
                mipCount: p.mipCount,
                baseLayer: p.baseLayer,
                layerCount: p.layerCount,
            };
            if unresolved {
                return skipUnresolvedMake(table, m.id, m.generation, "textureView");
            }
            table.set(
                m.id,
                ctx.makeTextureView(&d),
                m.generation,
                OreKind::textureView,
            );
        }
        CommandType::makePipeline => {
            let m: MakeResourcePOD = reader.read();
            let p: PipelineDescPOD = reader.read();
            if table.alive(m.id, m.generation) {
                return true;
            }
            let vbPods = decodePods::<VertexBufferLayoutPOD>(
                blob(reader, p.vertexBuffers).unwrap_or(&[]),
                p.vertexBufferCount,
            );
            let attributes: Vec<_> = vbPods
                .iter()
                .map(|v| {
                    decodePods::<VertexAttribute>(
                        blob(reader, v.attributes).unwrap_or(&[]),
                        v.attributeCount,
                    )
                })
                .collect();
            let vbs: Vec<_> = vbPods
                .iter()
                .zip(&attributes)
                .map(|(v, a)| VertexBufferLayout {
                    stride: v.stride,
                    stepMode: v.stepMode,
                    attributes: (!v.attributes.absent()).then_some(a.as_slice()),
                    attributeCount: v.attributeCount,
                })
                .collect();
            let bglHandles = decodePods::<u32>(
                blob(reader, p.bindGroupLayouts).unwrap_or(&[]),
                p.bindGroupLayoutCount,
            );
            let bgls: Vec<_> = bglHandles
                .iter()
                .map(|h| req(table, real, *h, OreKind::bindGroupLayout, &mut unresolved))
                .collect();
            let vertex = req(
                table,
                real,
                p.vertexModule,
                OreKind::shaderModule,
                &mut unresolved,
            );
            let fragment = req(
                table,
                real,
                p.fragmentModule,
                OreKind::shaderModule,
                &mut unresolved,
            );
            let layoutRefs: Vec<_> = bgls.iter().map(Option::as_ref).collect();
            let d = PipelineDesc {
                vertexModule: vertex.as_ref(),
                vertexEntryPoint: cstr(reader, p.vertexEntryPoint),
                fragmentModule: fragment.as_ref(),
                fragmentEntryPoint: cstr(reader, p.fragmentEntryPoint),
                vertexBuffers: (!vbs.is_empty()).then_some(vbs.as_slice()),
                vertexBufferCount: p.vertexBufferCount,
                topology: p.topology,
                indexFormat: p.indexFormat,
                cullMode: p.cullMode,
                winding: p.winding,
                colorTargets: p.colorTargets,
                colorCount: p.colorCount,
                depthStencil: p.depthStencil,
                stencilFront: p.stencilFront,
                stencilBack: p.stencilBack,
                stencilReadMask: p.stencilReadMask,
                stencilWriteMask: p.stencilWriteMask,
                sampleCount: p.sampleCount,
                bindGroupLayouts: (!layoutRefs.is_empty()).then_some(layoutRefs.as_slice()),
                bindGroupLayoutCount: p.bindGroupLayoutCount,
                label: cstr(reader, p.label),
            };
            if unresolved {
                return skipUnresolvedMake(table, m.id, m.generation, "pipeline");
            }
            let mut error = String::new();
            let pipeline = ctx.makePipeline(&d, Some(&mut error));
            if pipeline.is_none() {
                warn_throttled!(
                    "rive ore replay: makePipeline id={} gen={} failed: {}",
                    m.id,
                    m.generation,
                    error
                );
            }
            table.set(m.id, pipeline, m.generation, OreKind::pipeline);
        }
        CommandType::makeBindGroup => {
            let m: MakeResourcePOD = reader.read();
            let p: BindGroupDescPOD = reader.read();
            if table.alive(m.id, m.generation) {
                return true;
            }
            let up = decodePods::<UBOEntryPOD>(blob(reader, p.ubos).unwrap_or(&[]), p.uboCount);
            let buffers: Vec<_> = up
                .iter()
                .map(|u| req(table, real, u.buffer, OreKind::buffer, &mut unresolved))
                .collect();
            let ubos: Vec<_> = up
                .iter()
                .zip(&buffers)
                .map(|(u, b)| UBOEntry {
                    slot: u.slot,
                    buffer: b.as_ref(),
                    offset: u.offset,
                    size: u.size,
                })
                .collect();
            let tp =
                decodePods::<TexEntryPOD>(blob(reader, p.textures).unwrap_or(&[]), p.textureCount);
            let views: Vec<_> = tp
                .iter()
                .map(|t| req(table, real, t.view, OreKind::textureView, &mut unresolved))
                .collect();
            let textures: Vec<_> = tp
                .iter()
                .zip(&views)
                .map(|(t, v)| TexEntry {
                    slot: t.slot,
                    view: v.as_ref(),
                })
                .collect();
            let sp =
                decodePods::<SampEntryPOD>(blob(reader, p.samplers).unwrap_or(&[]), p.samplerCount);
            let samplerHandles: Vec<_> = sp
                .iter()
                .map(|s| req(table, real, s.sampler, OreKind::sampler, &mut unresolved))
                .collect();
            let samplers: Vec<_> = sp
                .iter()
                .zip(&samplerHandles)
                .map(|(s, h)| SampEntry {
                    slot: s.slot,
                    sampler: h.as_ref(),
                })
                .collect();
            let layout = req(
                table,
                real,
                p.layout,
                OreKind::bindGroupLayout,
                &mut unresolved,
            );
            let d = BindGroupDesc {
                layout: layout.as_ref(),
                ubos: &ubos,
                uboCount: p.uboCount,
                textures: &textures,
                textureCount: p.textureCount,
                samplers: &samplers,
                samplerCount: p.samplerCount,
                label: cstr(reader, p.label),
            };
            if unresolved {
                return skipUnresolvedMake(table, m.id, m.generation, "bindGroup");
            }
            let group = ctx.makeBindGroup(&d);
            if group.is_none() {
                warn_throttled!(
                    "rive ore replay: makeBindGroup id={} gen={} returned null",
                    m.id,
                    m.generation
                );
            }
            table.set(m.id, group, m.generation, OreKind::bindGroup);
        }
        CommandType::bufferUpdate => {
            let p: BufferUpdatePOD = reader.read();
            let bytes = blob(reader, p.bytes).unwrap_or(&[]);
            if let Some(buffer) = table.get(p.handle) {
                let _ = buffer.update(bytes, bytes.len() as u32, p.offset);
            }
        }
        CommandType::textureUpload => {
            let p: TextureUploadPOD = reader.read();
            let bytes = blob(reader, p.bytes).unwrap_or(&[]);
            let d = TextureDataDesc {
                data: (!bytes.is_empty()).then_some(bytes),
                bytesPerRow: p.bytesPerRow,
                rowsPerImage: p.rowsPerImage,
                mipLevel: p.mipLevel,
                layer: p.layer,
                x: p.x,
                y: p.y,
                z: p.z,
                width: p.width,
                height: p.height,
                depth: p.depth,
            };
            if let Some(texture) = table.get(p.handle) {
                let _ = texture.upload(&d);
            }
        }
        CommandType::wrapCanvasView => {
            let p: WrapCanvasViewPOD = reader.read();
            if table.alive(p.id, p.generation) {
                return true;
            }
            if p.mode == WrapCanvasViewMode::imageView as u32 {
                let image = imageAt(p.canvasId);
                let wrapped = if let Some(image) = image.filter(|i| !i.texture.is_null()) {
                    unsafe { ctx.wrapRiveTexture(image.texture, image.width, image.height) }
                } else {
                    skipUnresolvedMake(table, p.id, p.generation, "wrapImageView");
                    None
                };
                table.set(p.id, wrapped, p.generation, OreKind::textureView);
                return true;
            }
            let canvas = canvasAt(p.canvasId);
            debug_assert!(canvas.is_some(), "missing replay canvas");
            let wrapped = canvas.and_then(|canvas| unsafe {
                if p.mode == WrapCanvasViewMode::sampleView as u32 {
                    ctx.wrapCanvasSampleView(canvas)
                } else {
                    ctx.wrapCanvasTextureInfo(canvas)
                }
            });
            table.set(p.id, wrapped, p.generation, OreKind::textureView);
        }
        CommandType::destroyResource => {
            let p: DestroyResourcePOD = reader.read();
            table.destroy(p.handle, p.generation);
        }
        _ => return false,
    }
    true
}
pub fn skipOreCommand(kind: CommandType, reader: &mut OreCommandReader<'_>) {
    reader.skip(ore_payload_size_of(kind));
}
