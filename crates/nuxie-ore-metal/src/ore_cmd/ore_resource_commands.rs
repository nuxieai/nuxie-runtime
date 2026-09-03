//! renderer/ore/cmd/ore_resource_commands.hpp at 966499ff.
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

// Every wire POD orders its fields widest first. Any trailing padding is a
// named field so recording never depends on compiler-initialized gaps.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BufferDescPOD {
    pub data: BlobRef,
    pub label: BlobRef,
    pub size: u32,
    pub usage: BufferUsage,
    pub immutable: bool,
    pub pad: [u8; 2],
}
const _: [(); 40] = [(); std::mem::size_of::<BufferDescPOD>()];
crate::impl_wire_pod!(BufferDescPOD {
    data: BlobRef,
    label: BlobRef,
    size: u32,
    usage: BufferUsage,
    immutable: bool,
    pad: [u8; 2]
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TextureDescPOD {
    pub label: BlobRef,
    pub width: u32,
    pub height: u32,
    pub depthOrArrayLayers: u32,
    pub numMipmaps: u32,
    pub sampleCount: u32,
    pub format: TextureFormat,
    pub r#type: TextureType,
    pub renderTarget: bool,
    pub pad: [u8; 1],
}
const _: [(); 40] = [(); std::mem::size_of::<TextureDescPOD>()];
crate::impl_wire_pod!(TextureDescPOD {
    label: BlobRef,
    width: u32,
    height: u32,
    depthOrArrayLayers: u32,
    numMipmaps: u32,
    sampleCount: u32,
    format: TextureFormat,
    r#type: TextureType,
    renderTarget: bool,
    pad: [u8; 1]
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SamplerDescPOD {
    pub label: BlobRef,
    pub minLod: f32,
    pub maxLod: f32,
    pub maxAnisotropy: u32,
    pub minFilter: Filter,
    pub magFilter: Filter,
    pub mipmapFilter: Filter,
    pub wrapU: WrapMode,
    pub wrapV: WrapMode,
    pub wrapW: WrapMode,
    pub compare: CompareFunction,
    pub pad: [u8; 5],
}
const _: [(); 40] = [(); std::mem::size_of::<SamplerDescPOD>()];
crate::impl_wire_pod!(SamplerDescPOD {
    label: BlobRef,
    minLod: f32,
    maxLod: f32,
    maxAnisotropy: u32,
    minFilter: Filter,
    magFilter: Filter,
    mipmapFilter: Filter,
    wrapU: WrapMode,
    wrapV: WrapMode,
    wrapW: WrapMode,
    compare: CompareFunction,
    pad: [u8; 5]
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ShaderModuleDescPOD {
    pub code: BlobRef,
    pub label: BlobRef,
    pub hlslSource: BlobRef,
    pub hlslEntryPoint: BlobRef,
    pub bindingMapBytes: BlobRef,
    pub texSamplerPairBytes: BlobRef,
    pub glFixupBytes: BlobRef,
    pub shaderAssetId: u32,
    pub language: ShaderLanguage,
    pub stage: ShaderStage,
    pub pad: [u8; 2],
}
const _: [(); 120] = [(); std::mem::size_of::<ShaderModuleDescPOD>()];
crate::impl_wire_pod!(ShaderModuleDescPOD {
    code: BlobRef,
    label: BlobRef,
    hlslSource: BlobRef,
    hlslEntryPoint: BlobRef,
    bindingMapBytes: BlobRef,
    texSamplerPairBytes: BlobRef,
    glFixupBytes: BlobRef,
    shaderAssetId: u32,
    language: ShaderLanguage,
    stage: ShaderStage,
    pad: [u8; 2]
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BindGroupLayoutDescPOD {
    pub entries: BlobRef,
    pub label: BlobRef,
    pub groupIndex: u32,
    pub entryCount: u32,
}
const _: [(); 40] = [(); std::mem::size_of::<BindGroupLayoutDescPOD>()];
crate::impl_wire_pod!(BindGroupLayoutDescPOD {
    entries: BlobRef,
    label: BlobRef,
    groupIndex: u32,
    entryCount: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TextureViewDescPOD {
    pub texture: ResourceHandle,
    pub baseMipLevel: u32,
    pub mipCount: u32,
    pub baseLayer: u32,
    pub layerCount: u32,
    pub dimension: TextureViewDimension,
    pub aspect: TextureAspect,
    pub pad: [u8; 2],
}
const _: [(); 24] = [(); std::mem::size_of::<TextureViewDescPOD>()];
crate::impl_wire_pod!(TextureViewDescPOD {
    texture: ResourceHandle,
    baseMipLevel: u32,
    mipCount: u32,
    baseLayer: u32,
    layerCount: u32,
    dimension: TextureViewDimension,
    aspect: TextureAspect,
    pad: [u8; 2]
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VertexBufferLayoutPOD {
    pub attributes: BlobRef,
    pub stride: u32,
    pub attributeCount: u32,
    pub stepMode: VertexStepMode,
    pub pad: [u8; 7],
}
const _: [(); 32] = [(); std::mem::size_of::<VertexBufferLayoutPOD>()];
crate::impl_wire_pod!(VertexBufferLayoutPOD {
    attributes: BlobRef,
    stride: u32,
    attributeCount: u32,
    stepMode: VertexStepMode,
    pad: [u8; 7]
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PipelineDescPOD {
    pub vertexEntryPoint: BlobRef,
    pub fragmentEntryPoint: BlobRef,
    pub vertexBuffers: BlobRef,
    pub bindGroupLayouts: BlobRef,
    pub label: BlobRef,
    pub vertexModule: ResourceHandle,
    pub fragmentModule: ResourceHandle,
    pub vertexBufferCount: u32,
    pub colorCount: u32,
    pub sampleCount: u32,
    pub bindGroupLayoutCount: u32,
    pub depthStencil: DepthStencilState,
    pub colorTargets: [ColorTargetState; 4],
    pub stencilFront: StencilFaceState,
    pub stencilBack: StencilFaceState,
    pub topology: PrimitiveTopology,
    pub indexFormat: IndexFormat,
    pub cullMode: CullMode,
    pub winding: FaceWinding,
    pub stencilReadMask: u8,
    pub stencilWriteMask: u8,
    pub pad: [u8; 6],
}
const _: [(); 176] = [(); std::mem::size_of::<PipelineDescPOD>()];
crate::impl_wire_pod!(PipelineDescPOD {
    vertexEntryPoint: BlobRef,
    fragmentEntryPoint: BlobRef,
    vertexBuffers: BlobRef,
    bindGroupLayouts: BlobRef,
    label: BlobRef,
    vertexModule: ResourceHandle,
    fragmentModule: ResourceHandle,
    vertexBufferCount: u32,
    colorCount: u32,
    sampleCount: u32,
    bindGroupLayoutCount: u32,
    depthStencil: DepthStencilState,
    colorTargets: [ColorTargetState; 4],
    stencilFront: StencilFaceState,
    stencilBack: StencilFaceState,
    topology: PrimitiveTopology,
    indexFormat: IndexFormat,
    cullMode: CullMode,
    winding: FaceWinding,
    stencilReadMask: u8,
    stencilWriteMask: u8,
    pad: [u8; 6]
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
    pub ubos: BlobRef,
    pub textures: BlobRef,
    pub samplers: BlobRef,
    pub label: BlobRef,
    pub layout: ResourceHandle,
    pub uboCount: u32,
    pub textureCount: u32,
    pub samplerCount: u32,
}
const _: [(); 80] = [(); std::mem::size_of::<BindGroupDescPOD>()];
crate::impl_wire_pod!(BindGroupDescPOD {
    ubos: BlobRef,
    textures: BlobRef,
    samplers: BlobRef,
    label: BlobRef,
    layout: ResourceHandle,
    uboCount: u32,
    textureCount: u32,
    samplerCount: u32
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
