//! renderer/ore/cmd/ore_make_recording.hpp at 966499ff.
#![allow(non_snake_case)]
use super::{
    ore_command_buffer::OreCommandBuffer, ore_commands::*, ore_handle::*, ore_resource_commands::*,
};
use crate::{cmd::command_stream::WirePod, types::*};

pub fn encodePods<T: WirePod>(values: &[T]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * T::SIZE);
    for value in values {
        value.encode(&mut bytes);
    }
    bytes
}
fn appendPods<T: WirePod>(cb: &mut OreCommandBuffer, values: &[T], absent: bool) -> BlobRef {
    let bytes = encodePods(values);
    cb.appendBlobRef(Some(&bytes), bytes.len() as u32, absent)
}
pub fn recordMakeBuffer(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    desc: &BufferDesc<'_>,
) {
    let pod = BufferDescPOD {
        data: cb.appendBlobRef(
            desc.data,
            if desc.data.is_some() { desc.size } else { 0 },
            desc.data.is_none(),
        ),
        label: cb.appendStringRef(desc.label),
        size: desc.size,
        usage: desc.usage,
        immutable: desc.immutable,
        pad: [0; 2],
    };
    cb.append(CommandType::makeBuffer, &MakeResourcePOD { id, generation });
    cb.appendPayload(&pod);
}
pub fn recordMakeTexture(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    desc: &TextureDesc<'_>,
) {
    let pod = TextureDescPOD {
        label: cb.appendStringRef(desc.label),
        width: desc.width,
        height: desc.height,
        depthOrArrayLayers: desc.depthOrArrayLayers,
        numMipmaps: desc.numMipmaps,
        sampleCount: desc.sampleCount,
        format: desc.format,
        r#type: desc.r#type,
        renderTarget: desc.renderTarget,
        pad: [0; 1],
    };
    cb.append(
        CommandType::makeTexture,
        &MakeResourcePOD { id, generation },
    );
    cb.appendPayload(&pod);
}
pub fn recordMakeSampler(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    desc: &SamplerDesc<'_>,
) {
    let pod = SamplerDescPOD {
        label: cb.appendStringRef(desc.label),
        minLod: desc.minLod,
        maxLod: desc.maxLod,
        maxAnisotropy: desc.maxAnisotropy,
        minFilter: desc.minFilter,
        magFilter: desc.magFilter,
        mipmapFilter: desc.mipmapFilter,
        wrapU: desc.wrapU,
        wrapV: desc.wrapV,
        wrapW: desc.wrapW,
        compare: desc.compare,
        pad: [0; 5],
    };
    cb.append(
        CommandType::makeSampler,
        &MakeResourcePOD { id, generation },
    );
    cb.appendPayload(&pod);
}
pub fn recordMakeShaderModule(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    desc: &ShaderModuleDesc<'_>,
) {
    let pod = ShaderModuleDescPOD {
        code: cb.appendBlobRef(desc.code, desc.codeSize, desc.code.is_none()),
        label: cb.appendStringRef(desc.label),
        hlslSource: cb.appendBlobRef(
            desc.hlslSource.map(str::as_bytes),
            desc.hlslSourceSize,
            desc.hlslSource.is_none(),
        ),
        hlslEntryPoint: cb.appendStringRef(desc.hlslEntryPoint),
        bindingMapBytes: cb.appendBlobRef(
            desc.bindingMapBytes,
            desc.bindingMapSize,
            desc.bindingMapBytes.is_none(),
        ),
        texSamplerPairBytes: cb.appendBlobRef(
            desc.texSamplerPairBytes,
            desc.texSamplerPairSize,
            desc.texSamplerPairBytes.is_none(),
        ),
        glFixupBytes: cb.appendBlobRef(
            desc.glFixupBytes,
            desc.glFixupSize,
            desc.glFixupBytes.is_none(),
        ),
        shaderAssetId: desc.shaderAssetId,
        language: desc.language,
        stage: desc.stage,
        pad: [0; 2],
    };
    cb.append(
        CommandType::makeShaderModule,
        &MakeResourcePOD { id, generation },
    );
    cb.appendPayload(&pod);
}
pub fn recordMakeBindGroupLayout(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    desc: &BindGroupLayoutDesc<'_>,
) {
    let entries = &desc.entries.unwrap_or(&[])[..desc.entryCount as usize];
    let pod = BindGroupLayoutDescPOD {
        entries: appendPods(cb, entries, desc.entries.is_none()),
        label: cb.appendStringRef(desc.label),
        groupIndex: desc.groupIndex,
        entryCount: desc.entryCount,
    };
    cb.append(
        CommandType::makeBindGroupLayout,
        &MakeResourcePOD { id, generation },
    );
    cb.appendPayload(&pod);
}
pub fn recordMakeTextureView(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    desc: &TextureViewDesc<'_>,
    textureHandle: ResourceHandle,
) {
    let pod = TextureViewDescPOD {
        texture: textureHandle,
        baseMipLevel: desc.baseMipLevel,
        mipCount: desc.mipCount,
        baseLayer: desc.baseLayer,
        layerCount: desc.layerCount,
        dimension: desc.dimension,
        aspect: desc.aspect,
        pad: [0; 2],
    };
    cb.append(
        CommandType::makeTextureView,
        &MakeResourcePOD { id, generation },
    );
    cb.appendPayload(&pod);
}
pub fn recordMakePipeline(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    desc: &PipelineDesc<'_>,
    vertexModule: ResourceHandle,
    fragmentModule: ResourceHandle,
    bindGroupLayouts: &[ResourceHandle],
) {
    let mut vbPods = Vec::with_capacity(desc.vertexBufferCount as usize);
    for vb in &desc.vertexBuffers.unwrap_or(&[])[..desc.vertexBufferCount as usize] {
        vbPods.push(VertexBufferLayoutPOD {
            attributes: appendPods(
                cb,
                &vb.attributes.unwrap_or(&[])[..vb.attributeCount as usize],
                vb.attributes.is_none(),
            ),
            stride: vb.stride,
            attributeCount: vb.attributeCount,
            stepMode: vb.stepMode,
            pad: [0; 7],
        });
    }
    let pod = PipelineDescPOD {
        vertexEntryPoint: cb.appendStringRef(desc.vertexEntryPoint),
        fragmentEntryPoint: cb.appendStringRef(desc.fragmentEntryPoint),
        vertexBuffers: appendPods(cb, &vbPods, vbPods.is_empty()),
        bindGroupLayouts: appendPods(cb, bindGroupLayouts, bindGroupLayouts.is_empty()),
        label: cb.appendStringRef(desc.label),
        vertexModule,
        fragmentModule,
        vertexBufferCount: desc.vertexBufferCount,
        colorCount: desc.colorCount,
        sampleCount: desc.sampleCount,
        bindGroupLayoutCount: bindGroupLayouts.len() as u32,
        depthStencil: desc.depthStencil,
        colorTargets: desc.colorTargets,
        stencilFront: desc.stencilFront,
        stencilBack: desc.stencilBack,
        topology: desc.topology,
        indexFormat: desc.indexFormat,
        cullMode: desc.cullMode,
        winding: desc.winding,
        stencilReadMask: desc.stencilReadMask,
        stencilWriteMask: desc.stencilWriteMask,
        pad: [0; 6],
    };
    cb.append(
        CommandType::makePipeline,
        &MakeResourcePOD { id, generation },
    );
    cb.appendPayload(&pod);
}
pub fn recordMakeBindGroup(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    desc: &BindGroupDesc<'_>,
    layout: ResourceHandle,
    uboBuffers: &[ResourceHandle],
    texViews: &[ResourceHandle],
    sampSamplers: &[ResourceHandle],
) {
    let ubos: Vec<_> = desc.ubos[..desc.uboCount as usize]
        .iter()
        .enumerate()
        .map(|(i, entry)| UBOEntryPOD {
            slot: entry.slot,
            buffer: uboBuffers.get(i).copied().unwrap_or(INVALID_HANDLE),
            offset: entry.offset,
            size: entry.size,
        })
        .collect();
    let textures: Vec<_> = desc.textures[..desc.textureCount as usize]
        .iter()
        .enumerate()
        .map(|(i, entry)| TexEntryPOD {
            slot: entry.slot,
            view: texViews.get(i).copied().unwrap_or(INVALID_HANDLE),
        })
        .collect();
    let samplers: Vec<_> = desc.samplers[..desc.samplerCount as usize]
        .iter()
        .enumerate()
        .map(|(i, entry)| SampEntryPOD {
            slot: entry.slot,
            sampler: sampSamplers.get(i).copied().unwrap_or(INVALID_HANDLE),
        })
        .collect();
    let pod = BindGroupDescPOD {
        ubos: appendPods(cb, &ubos, ubos.is_empty()),
        textures: appendPods(cb, &textures, textures.is_empty()),
        samplers: appendPods(cb, &samplers, samplers.is_empty()),
        label: cb.appendStringRef(desc.label),
        layout,
        uboCount: desc.uboCount,
        textureCount: desc.textureCount,
        samplerCount: desc.samplerCount,
    };
    cb.append(
        CommandType::makeBindGroup,
        &MakeResourcePOD { id, generation },
    );
    cb.appendPayload(&pod);
}
pub fn recordBufferUpdate(
    cb: &mut OreCommandBuffer,
    handle: ResourceHandle,
    data: Option<&[u8]>,
    size: u32,
    offset: u32,
) {
    let pod = BufferUpdatePOD {
        handle,
        offset,
        bytes: cb.appendBlobRef(data, size, data.is_none()),
    };
    cb.append(CommandType::bufferUpdate, &pod);
}
pub fn recordTextureUpload(
    cb: &mut OreCommandBuffer,
    handle: ResourceHandle,
    desc: &TextureDataDesc<'_>,
) {
    let rows = if desc.rowsPerImage != 0 {
        desc.rowsPerImage
    } else {
        desc.height
    };
    let size = desc.bytesPerRow.wrapping_mul(rows);
    let pod = TextureUploadPOD {
        handle,
        bytesPerRow: desc.bytesPerRow,
        rowsPerImage: desc.rowsPerImage,
        mipLevel: desc.mipLevel,
        layer: desc.layer,
        x: desc.x,
        y: desc.y,
        z: desc.z,
        width: desc.width,
        height: desc.height,
        depth: desc.depth,
        pad: 0,
        bytes: cb.appendBlobRef(desc.data, size, desc.data.is_none() || size == 0),
    };
    cb.append(CommandType::textureUpload, &pod);
}
pub fn recordWrapCanvasView(
    cb: &mut OreCommandBuffer,
    id: ResourceHandle,
    generation: u32,
    canvasId: u32,
    mode: WrapCanvasViewMode,
) {
    cb.append(
        CommandType::wrapCanvasView,
        &WrapCanvasViewPOD {
            id,
            generation,
            canvasId,
            mode: mode as u32,
        },
    );
}
pub fn recordDestroyResource(cb: &mut OreCommandBuffer, handle: ResourceHandle, generation: u32) {
    cb.append(
        CommandType::destroyResource,
        &DestroyResourcePOD { handle, generation },
    );
}
