//! renderer/ore/cmd/ore_resource_commands.hpp at e949498e.
#![allow(non_snake_case)]
use super::ore_handle::ResourceHandle;
use crate::types::*;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BlobRef {
    pub offset: u64,
    pub size: u32,
    pub pad: u32,
}
crate::impl_wire_pod!(BlobRef {
    offset: u64,
    size: u32,
    pad: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BufferDescPOD {
    pub usage: BufferUsage,
    pub size: u32,
    pub immutable: bool,
    pub data: BlobRef,
    pub label: BlobRef,
}
crate::impl_wire_pod!(BufferDescPOD {
    usage: BufferUsage,
    size: u32,
    immutable: bool,
    data: BlobRef,
    label: BlobRef
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TextureDescPOD {
    pub width: u32,
    pub height: u32,
    pub depthOrArrayLayers: u32,
    pub format: TextureFormat,
    pub r#type: TextureType,
    pub renderTarget: bool,
    pub numMipmaps: u32,
    pub sampleCount: u32,
    pub label: BlobRef,
}
crate::impl_wire_pod!(TextureDescPOD {
    width: u32,
    height: u32,
    depthOrArrayLayers: u32,
    format: TextureFormat,
    r#type: TextureType,
    renderTarget: bool,
    numMipmaps: u32,
    sampleCount: u32,
    label: BlobRef
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SamplerDescPOD {
    pub minFilter: Filter,
    pub magFilter: Filter,
    pub mipmapFilter: Filter,
    pub wrapU: WrapMode,
    pub wrapV: WrapMode,
    pub wrapW: WrapMode,
    pub compare: CompareFunction,
    pub minLod: f32,
    pub maxLod: f32,
    pub maxAnisotropy: u32,
    pub label: BlobRef,
}
crate::impl_wire_pod!(SamplerDescPOD {
    minFilter: Filter,
    magFilter: Filter,
    mipmapFilter: Filter,
    wrapU: WrapMode,
    wrapV: WrapMode,
    wrapW: WrapMode,
    compare: CompareFunction,
    minLod: f32,
    maxLod: f32,
    maxAnisotropy: u32,
    label: BlobRef
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ShaderModuleDescPOD {
    pub code: BlobRef,
    pub language: ShaderLanguage,
    pub stage: ShaderStage,
    pub label: BlobRef,
    pub hlslSource: BlobRef,
    pub hlslEntryPoint: BlobRef,
    pub bindingMapBytes: BlobRef,
    pub glFixupBytes: BlobRef,
    pub shaderAssetId: u32,
}
crate::impl_wire_pod!(ShaderModuleDescPOD {
    code: BlobRef,
    language: ShaderLanguage,
    stage: ShaderStage,
    label: BlobRef,
    hlslSource: BlobRef,
    hlslEntryPoint: BlobRef,
    bindingMapBytes: BlobRef,
    glFixupBytes: BlobRef,
    shaderAssetId: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BindGroupLayoutDescPOD {
    pub groupIndex: u32,
    pub entries: BlobRef,
    pub entryCount: u32,
    pub label: BlobRef,
}
crate::impl_wire_pod!(BindGroupLayoutDescPOD {
    groupIndex: u32,
    entries: BlobRef,
    entryCount: u32,
    label: BlobRef
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TextureViewDescPOD {
    pub texture: ResourceHandle,
    pub dimension: TextureViewDimension,
    pub aspect: TextureAspect,
    pub baseMipLevel: u32,
    pub mipCount: u32,
    pub baseLayer: u32,
    pub layerCount: u32,
}
crate::impl_wire_pod!(TextureViewDescPOD {
    texture: ResourceHandle,
    dimension: TextureViewDimension,
    aspect: TextureAspect,
    baseMipLevel: u32,
    mipCount: u32,
    baseLayer: u32,
    layerCount: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VertexBufferLayoutPOD {
    pub stride: u32,
    pub stepMode: VertexStepMode,
    pub attributeCount: u32,
    pub attributes: BlobRef,
}
crate::impl_wire_pod!(VertexBufferLayoutPOD {
    stride: u32,
    stepMode: VertexStepMode,
    attributeCount: u32,
    attributes: BlobRef
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PipelineDescPOD {
    pub vertexModule: ResourceHandle,
    pub vertexEntryPoint: BlobRef,
    pub fragmentModule: ResourceHandle,
    pub fragmentEntryPoint: BlobRef,
    pub vertexBuffers: BlobRef,
    pub vertexBufferCount: u32,
    pub topology: PrimitiveTopology,
    pub indexFormat: IndexFormat,
    pub cullMode: CullMode,
    pub winding: FaceWinding,
    pub colorTargets: [ColorTargetState; 4],
    pub colorCount: u32,
    pub depthStencil: DepthStencilState,
    pub stencilFront: StencilFaceState,
    pub stencilBack: StencilFaceState,
    pub stencilReadMask: u8,
    pub stencilWriteMask: u8,
    pub sampleCount: u32,
    pub bindGroupLayouts: BlobRef,
    pub bindGroupLayoutCount: u32,
    pub label: BlobRef,
}
crate::impl_wire_pod!(PipelineDescPOD {
    vertexModule: ResourceHandle,
    vertexEntryPoint: BlobRef,
    fragmentModule: ResourceHandle,
    fragmentEntryPoint: BlobRef,
    vertexBuffers: BlobRef,
    vertexBufferCount: u32,
    topology: PrimitiveTopology,
    indexFormat: IndexFormat,
    cullMode: CullMode,
    winding: FaceWinding,
    colorTargets: [ColorTargetState; 4],
    colorCount: u32,
    depthStencil: DepthStencilState,
    stencilFront: StencilFaceState,
    stencilBack: StencilFaceState,
    stencilReadMask: u8,
    stencilWriteMask: u8,
    sampleCount: u32,
    bindGroupLayouts: BlobRef,
    bindGroupLayoutCount: u32,
    label: BlobRef
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct UBOEntryPOD {
    pub slot: u32,
    pub buffer: ResourceHandle,
    pub offset: u32,
    pub size: u32,
}
crate::impl_wire_pod!(UBOEntryPOD {
    slot: u32,
    buffer: ResourceHandle,
    offset: u32,
    size: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TexEntryPOD {
    pub slot: u32,
    pub view: ResourceHandle,
}
crate::impl_wire_pod!(TexEntryPOD {
    slot: u32,
    view: ResourceHandle
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SampEntryPOD {
    pub slot: u32,
    pub sampler: ResourceHandle,
}
crate::impl_wire_pod!(SampEntryPOD {
    slot: u32,
    sampler: ResourceHandle
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BindGroupDescPOD {
    pub layout: ResourceHandle,
    pub ubos: BlobRef,
    pub uboCount: u32,
    pub textures: BlobRef,
    pub textureCount: u32,
    pub samplers: BlobRef,
    pub samplerCount: u32,
    pub label: BlobRef,
}
crate::impl_wire_pod!(BindGroupDescPOD {
    layout: ResourceHandle,
    ubos: BlobRef,
    uboCount: u32,
    textures: BlobRef,
    textureCount: u32,
    samplers: BlobRef,
    samplerCount: u32,
    label: BlobRef
});

impl BlobRef {
    pub const ABSENT: u32 = u32::MAX;
    pub fn absent(&self) -> bool {
        self.size == Self::ABSENT
    }
}
pub const NO_BLOB: BlobRef = BlobRef {
    offset: 0,
    size: BlobRef::ABSENT,
    pad: 0,
};
