//! renderer/ore/cmd/ore_commands.hpp at 707c4f60.
#![allow(non_snake_case, non_camel_case_types)]
use super::ore_handle::ResourceHandle;
use super::ore_resource_commands::*;
use crate::cmd::command_stream::WirePod;
use crate::types::*;
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CommandType {
    #[default]
    beginRenderPass = 0,
    setPipeline = 1,
    setVertexBuffer = 2,
    setIndexBuffer = 3,
    setBindGroup = 4,
    setViewport = 5,
    setScissorRect = 6,
    setStencilReference = 7,
    setBlendColor = 8,
    draw = 9,
    drawIndexed = 10,
    finish = 11,
    makeBuffer = 12,
    makeTexture = 13,
    makeSampler = 14,
    makeShaderModule = 15,
    makeBindGroupLayout = 16,
    makeTextureView = 17,
    makePipeline = 18,
    makeBindGroup = 19,
    bufferUpdate = 20,
    textureUpload = 21,
    destroyResource = 22,
    wrapCanvasView = 23,
}
impl WirePod for CommandType {
    const SIZE: usize = 4;
    fn encode(&self, bytes: &mut Vec<u8>) {
        (*self as u32).encode(bytes);
    }
    fn decode(bytes: &[u8]) -> Self {
        match u32::decode(bytes) {
            0 => Self::beginRenderPass,
            1 => Self::setPipeline,
            2 => Self::setVertexBuffer,
            3 => Self::setIndexBuffer,
            4 => Self::setBindGroup,
            5 => Self::setViewport,
            6 => Self::setScissorRect,
            7 => Self::setStencilReference,
            8 => Self::setBlendColor,
            9 => Self::draw,
            10 => Self::drawIndexed,
            11 => Self::finish,
            12 => Self::makeBuffer,
            13 => Self::makeTexture,
            14 => Self::makeSampler,
            15 => Self::makeShaderModule,
            16 => Self::makeBindGroupLayout,
            17 => Self::makeTextureView,
            18 => Self::makePipeline,
            19 => Self::makeBindGroup,
            20 => Self::bufferUpdate,
            21 => Self::textureUpload,
            22 => Self::destroyResource,
            23 => Self::wrapCanvasView,
            _ => panic!("invalid ORE command opcode"),
        }
    }
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WrapCanvasViewMode {
    #[default]
    colorView = 0,
    sampleView = 1,
    imageView = 2,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MakeResourcePOD {
    pub id: ResourceHandle,
    pub generation: u32,
}
crate::impl_wire_pod!(MakeResourcePOD {
    id: ResourceHandle,
    generation: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BufferUpdatePOD {
    pub handle: ResourceHandle,
    pub offset: u32,
    pub bytes: BlobRef,
}
crate::impl_wire_pod!(BufferUpdatePOD {
    handle: ResourceHandle,
    offset: u32,
    bytes: BlobRef
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TextureUploadPOD {
    pub handle: ResourceHandle,
    pub bytesPerRow: u32,
    pub rowsPerImage: u32,
    pub mipLevel: u32,
    pub layer: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub pad: u32,
    pub bytes: BlobRef,
}
crate::impl_wire_pod!(TextureUploadPOD {
    handle: ResourceHandle,
    bytesPerRow: u32,
    rowsPerImage: u32,
    mipLevel: u32,
    layer: u32,
    x: u32,
    y: u32,
    z: u32,
    width: u32,
    height: u32,
    depth: u32,
    pad: u32,
    bytes: BlobRef
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct WrapCanvasViewPOD {
    pub id: ResourceHandle,
    pub generation: u32,
    pub canvasId: u32,
    pub mode: u32,
}
crate::impl_wire_pod!(WrapCanvasViewPOD {
    id: ResourceHandle,
    generation: u32,
    canvasId: u32,
    mode: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DestroyResourcePOD {
    pub handle: ResourceHandle,
    pub generation: u32,
}
crate::impl_wire_pod!(DestroyResourcePOD {
    handle: ResourceHandle,
    generation: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ColorAttachmentPOD {
    pub view: ResourceHandle,
    pub resolveTarget: ResourceHandle,
    pub loadOp: LoadOp,
    pub storeOp: StoreOp,
    pub clearR: f32,
    pub clearG: f32,
    pub clearB: f32,
    pub clearA: f32,
}
crate::impl_wire_pod!(ColorAttachmentPOD {
    view: ResourceHandle,
    resolveTarget: ResourceHandle,
    loadOp: LoadOp,
    storeOp: StoreOp,
    clearR: f32,
    clearG: f32,
    clearB: f32,
    clearA: f32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DepthStencilAttachmentPOD {
    pub view: ResourceHandle,
    pub depthLoadOp: LoadOp,
    pub depthStoreOp: StoreOp,
    pub depthClearValue: f32,
    pub stencilLoadOp: LoadOp,
    pub stencilStoreOp: StoreOp,
    pub stencilClearValue: u32,
}
crate::impl_wire_pod!(DepthStencilAttachmentPOD {
    view: ResourceHandle,
    depthLoadOp: LoadOp,
    depthStoreOp: StoreOp,
    depthClearValue: f32,
    stencilLoadOp: LoadOp,
    stencilStoreOp: StoreOp,
    stencilClearValue: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BeginRenderPassCmd {
    pub colorCount: u32,
    pub colors: [ColorAttachmentPOD; 4],
    pub depthStencil: DepthStencilAttachmentPOD,
}
crate::impl_wire_pod!(BeginRenderPassCmd {
    colorCount: u32,
    colors: [ColorAttachmentPOD; 4],
    depthStencil: DepthStencilAttachmentPOD
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SetPipelineCmd {
    pub pipeline: ResourceHandle,
}
crate::impl_wire_pod!(SetPipelineCmd {
    pipeline: ResourceHandle
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SetVertexBufferCmd {
    pub slot: u32,
    pub buffer: ResourceHandle,
    pub offset: u32,
}
crate::impl_wire_pod!(SetVertexBufferCmd {
    slot: u32,
    buffer: ResourceHandle,
    offset: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
// `pad` trails the real fields so struct initialization can leave it zeroed.
pub struct SetIndexBufferCmd {
    pub buffer: ResourceHandle,
    pub offset: u32,
    pub format: IndexFormat,
    pub pad: [u8; 3],
}
const _: [(); 3 * std::mem::size_of::<u32>()] = [(); std::mem::size_of::<SetIndexBufferCmd>()];
crate::impl_wire_pod!(SetIndexBufferCmd {
    buffer: ResourceHandle,
    offset: u32,
    format: IndexFormat,
    pad: [u8; 3]
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SetBindGroupCmd {
    pub groupIndex: u32,
    pub bindGroup: ResourceHandle,
    pub dynamicOffsetStart: u64,
    pub dynamicOffsetCount: u32,
    pub pad: u32,
}
crate::impl_wire_pod!(SetBindGroupCmd {
    groupIndex: u32,
    bindGroup: ResourceHandle,
    dynamicOffsetStart: u64,
    dynamicOffsetCount: u32,
    pad: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SetViewportCmd {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub minDepth: f32,
    pub maxDepth: f32,
}
crate::impl_wire_pod!(SetViewportCmd {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    minDepth: f32,
    maxDepth: f32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SetScissorRectCmd {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
crate::impl_wire_pod!(SetScissorRectCmd {
    x: u32,
    y: u32,
    width: u32,
    height: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SetStencilReferenceCmd {
    pub reference: u32,
}
crate::impl_wire_pod!(SetStencilReferenceCmd { reference: u32 });

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SetBlendColorCmd {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
crate::impl_wire_pod!(SetBlendColorCmd {
    r: f32,
    g: f32,
    b: f32,
    a: f32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DrawCmd {
    pub vertexCount: u32,
    pub instanceCount: u32,
    pub firstVertex: u32,
    pub firstInstance: u32,
}
crate::impl_wire_pod!(DrawCmd {
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32
});

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DrawIndexedCmd {
    pub indexCount: u32,
    pub instanceCount: u32,
    pub firstIndex: u32,
    pub baseVertex: i32,
    pub firstInstance: u32,
}
crate::impl_wire_pod!(DrawIndexedCmd {
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    baseVertex: i32,
    firstInstance: u32
});

pub const fn ore_payload_size_of(command: CommandType) -> usize {
    match command {
        CommandType::beginRenderPass => BeginRenderPassCmd::SIZE + 0,
        CommandType::setPipeline => SetPipelineCmd::SIZE + 0,
        CommandType::setVertexBuffer => SetVertexBufferCmd::SIZE + 0,
        CommandType::setIndexBuffer => SetIndexBufferCmd::SIZE + 0,
        CommandType::setBindGroup => SetBindGroupCmd::SIZE + 0,
        CommandType::setViewport => SetViewportCmd::SIZE + 0,
        CommandType::setScissorRect => SetScissorRectCmd::SIZE + 0,
        CommandType::setStencilReference => SetStencilReferenceCmd::SIZE + 0,
        CommandType::setBlendColor => SetBlendColorCmd::SIZE + 0,
        CommandType::draw => DrawCmd::SIZE + 0,
        CommandType::drawIndexed => DrawIndexedCmd::SIZE + 0,
        CommandType::finish => 0 + 0,
        CommandType::makeBuffer => MakeResourcePOD::SIZE + BufferDescPOD::SIZE,
        CommandType::makeTexture => MakeResourcePOD::SIZE + TextureDescPOD::SIZE,
        CommandType::makeSampler => MakeResourcePOD::SIZE + SamplerDescPOD::SIZE,
        CommandType::makeShaderModule => MakeResourcePOD::SIZE + ShaderModuleDescPOD::SIZE,
        CommandType::makeBindGroupLayout => MakeResourcePOD::SIZE + BindGroupLayoutDescPOD::SIZE,
        CommandType::makeTextureView => MakeResourcePOD::SIZE + TextureViewDescPOD::SIZE,
        CommandType::makePipeline => MakeResourcePOD::SIZE + PipelineDescPOD::SIZE,
        CommandType::makeBindGroup => MakeResourcePOD::SIZE + BindGroupDescPOD::SIZE,
        CommandType::bufferUpdate => BufferUpdatePOD::SIZE + 0,
        CommandType::textureUpload => TextureUploadPOD::SIZE + 0,
        CommandType::destroyResource => DestroyResourcePOD::SIZE + 0,
        CommandType::wrapCanvasView => WrapCanvasViewPOD::SIZE + 0,
    }
}
const _: () = assert!(TextureUploadPOD::SIZE == 16 * 4);
const _: () = assert!(SetBindGroupCmd::SIZE == 6 * 4);
