/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include <cstdint>

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_types.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use core::ops::{BitAnd, BitOr};

use super::super::gpu_resource_hpp::AnyResourceHandle;

// namespace rive::ore

// ============================================================================
// Constants
// ============================================================================

// Maximum number of bind groups Ore supports per pipeline. WebGPU's
// `maxBindGroups` minimum is 4, and Ore sits at that minimum. Backends
// preallocate per-group structures using this
// constant (Vulkan DSLs, WebGPU BGLs, D3D12 root params, RenderPass
// `m_boundGroups`). Single source of truth across every backend's
// per-group array, the public `BindGroupDesc::groupIndex` validity
// range, and Lua-side validation in `gpubindgroup_construct` /
// `setBindGroup`.
pub const kMaxBindGroups: u32 = 4;

// ============================================================================
// Enums
// ============================================================================

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferUsage {
    vertex = 0,
    index = 1,
    uniform = 2,
    upload = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderLanguage {
    glsl = 0, // GLSL via wagyu (GLSLRAW for GLES, GLSL/glslang for Vulkan).
    wgsl = 1, // WGSL — works on both GLES and Vulkan wagyu paths, no pragma needed.
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderStage {
    autoDetect = 0, // Infer from source content (legacy gl_Position heuristic).
    vertex = 1,
    fragment = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureFormat {
    // 8-bit
    r8unorm = 0,
    rg8unorm = 1,
    rgba8unorm = 2,
    rgba8snorm = 3,
    bgra8unorm = 4,

    // 16-bit float
    rgba16float = 5,
    rg16float = 6,
    r16float = 7,

    // 32-bit float
    rgba32float = 8,
    rg32float = 9,
    r32float = 10,

    // Packed
    rgb10a2unorm = 11,
    r11g11b10float = 12,

    // Depth/stencil
    depth16unorm = 13,
    depth24plusStencil8 = 14,
    depth32float = 15,
    depth32floatStencil8 = 16,

    // Compressed (runtime support via Features query)
    bc1unorm = 17,
    bc3unorm = 18,
    bc7unorm = 19,
    etc2rgb8 = 20,
    etc2rgba8 = 21,
    astc4x4 = 22,
    astc6x6 = 23,
    astc8x8 = 24,
}

// Returns bytes per texel for uncompressed formats, or 0 for block-compressed
// formats (which require block-based stride calculation).
#[inline]
pub const fn textureFormatBytesPerTexel(fmt: TextureFormat) -> u32 {
    match fmt {
        TextureFormat::r8unorm => 1,
        TextureFormat::rg8unorm => 2,
        TextureFormat::rgba8unorm => 4,
        TextureFormat::rgba8snorm => 4,
        TextureFormat::bgra8unorm => 4,
        TextureFormat::rgba16float => 8,
        TextureFormat::rg16float => 4,
        TextureFormat::r16float => 2,
        TextureFormat::rgba32float => 16,
        TextureFormat::rg32float => 8,
        TextureFormat::r32float => 4,
        TextureFormat::rgb10a2unorm => 4,
        TextureFormat::r11g11b10float => 4,
        TextureFormat::depth16unorm => 2,
        TextureFormat::depth24plusStencil8 => 4,
        TextureFormat::depth32float => 4,
        TextureFormat::depth32floatStencil8 => 8,
        _ => 0, // block-compressed
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureType {
    texture2D = 0,
    cube = 1,
    texture3D = 2,
    array2D = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureViewDimension {
    texture2D = 0,
    cube = 1,
    texture3D = 2,
    array2D = 3,
    cubeArray = 4,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureAspect {
    all = 0,
    depthOnly = 1,
    stencilOnly = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    nearest = 0,
    linear = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapMode {
    repeat = 0,
    mirrorRepeat = 1,
    clampToEdge = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareFunction {
    none = 0, // Not a comparison (default for samplers = normal filtering).
    never = 1,
    less = 2,
    equal = 3,
    lessEqual = 4,
    greater = 5,
    notEqual = 6,
    greaterEqual = 7,
    always = 8,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveTopology {
    pointList = 0,
    lineList = 1,
    lineStrip = 2,
    triangleList = 3,
    triangleStrip = 4,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexFormat {
    none = 0,
    uint16 = 1,
    uint32 = 2,
}

// 32-bit integer vector vertex formats (sint32, sint32x2..4, uint32x2..4) are
// intentionally omitted: scripts don't expose them, and Unreal RHI only
// supports scalar VET_UInt. Reintroduce per-backend if a real use case appears.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexFormat {
    float1 = 0,
    float2 = 1,
    float3 = 2,
    float4 = 3,
    uint8x4 = 4,
    sint8x4 = 5,
    unorm8x4 = 6,
    snorm8x4 = 7,
    uint16x2 = 8,
    sint16x2 = 9,
    unorm16x2 = 10,
    snorm16x2 = 11,
    uint16x4 = 12,
    sint16x4 = 13,
    float16x2 = 14,
    float16x4 = 15,
    uint32 = 16,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexStepMode {
    vertex = 0,
    instance = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CullMode {
    none = 0,
    front = 1,
    back = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceWinding {
    clockwise = 0,
    counterClockwise = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendFactor {
    zero = 0,
    one = 1,
    srcColor = 2,
    oneMinusSrcColor = 3,
    srcAlpha = 4,
    oneMinusSrcAlpha = 5,
    dstColor = 6,
    oneMinusDstColor = 7,
    dstAlpha = 8,
    oneMinusDstAlpha = 9,
    srcAlphaSaturated = 10,
    blendColor = 11,
    oneMinusBlendColor = 12,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendOp {
    add = 0,
    subtract = 1,
    reverseSubtract = 2,
    min = 3,
    max = 4,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StencilOp {
    keep = 0,
    zero = 1,
    replace = 2,
    incrementClamp = 3,
    decrementClamp = 4,
    invert = 5,
    incrementWrap = 6,
    decrementWrap = 7,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadOp {
    clear = 0,
    load = 1,
    dontCare = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreOp {
    store = 0,
    discard = 1,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorWriteMask(pub u8);

impl ColorWriteMask {
    pub const none: Self = Self(0);
    pub const red: Self = Self(1 << 0);
    pub const green: Self = Self(1 << 1);
    pub const blue: Self = Self(1 << 2);
    pub const alpha: Self = Self(1 << 3);
    pub const all: Self = Self(0xF);

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}

impl BitOr for ColorWriteMask {
    type Output = ColorWriteMask;

    fn bitor(self, rhs: ColorWriteMask) -> ColorWriteMask {
        // Mirrors static_cast<uint8_t>(a) | static_cast<uint8_t>(b).
        ColorWriteMask(self.0 | rhs.0)
    }
}

impl BitAnd for ColorWriteMask {
    type Output = ColorWriteMask;

    fn bitand(self, rhs: ColorWriteMask) -> ColorWriteMask {
        // Mirrors static_cast<uint8_t>(a) & static_cast<uint8_t>(b).
        ColorWriteMask(self.0 & rhs.0)
    }
}

// ============================================================================
// Forward declarations
// ============================================================================

// C++ forward declarations do not create a second type: these names resolve to
// the concrete classes defined by the later headers. Rust descriptors carry an
// intrusive, type-erased owner at this boundary, so name that representation
// explicitly instead of shadowing the concrete translated class identities.
pub type BufferHandle = AnyResourceHandle;
pub type TextureHandle = AnyResourceHandle;
pub type TextureViewHandle = AnyResourceHandle;
pub type SamplerHandle = AnyResourceHandle;
pub type ShaderModuleHandle = AnyResourceHandle;
pub type PipelineHandle = AnyResourceHandle;
pub type BindGroupLayoutHandle = AnyResourceHandle;

// ============================================================================
// Descriptor Structs
// ============================================================================

// These are the safe borrowed descriptor views used by Rust code. They are
// deliberately *not* `repr(C)`: slices, `str`, and `Option<&T>` are not the
// pinned C++ raw pointer/count ABI. `raw_abi` below is the separate exact
// pointer/count/null adapter layer.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DescriptorSizeError;

fn checked_count(len: usize) -> Result<u32, DescriptorSizeError> {
    u32::try_from(len).map_err(|_| DescriptorSizeError)
}

fn checked_len(bytes: &[u8]) -> Result<u32, DescriptorSizeError> {
    checked_count(bytes.len())
}

fn checked_prefix<T>(
    values: Option<&[T]>,
    count: u32,
) -> Result<Option<&[T]>, DescriptorSizeError> {
    match values {
        Some(values) => values
            .get(..count as usize)
            .map(Some)
            .ok_or(DescriptorSizeError),
        None if count == 0 => Ok(None),
        None => Err(DescriptorSizeError),
    }
}

#[derive(Clone, Copy)]
pub struct BufferDesc<'a> {
    pub usage: BufferUsage,
    pub size: u32,
    pub data: Option<&'a [u8]>,
    pub immutable: bool, // GPU-only allocation; data must be non-null; no
    // update() calls allowed after creation.
    pub label: Option<&'a str>,
}

impl<'a> BufferDesc<'a> {
    pub fn uninitialized(usage: BufferUsage, size: u32) -> Self {
        Self {
            usage,
            size,
            data: None,
            immutable: false,
            label: None,
        }
    }

    pub fn initialized(
        usage: BufferUsage,
        data: &'a [u8],
        immutable: bool,
    ) -> Result<Self, DescriptorSizeError> {
        Ok(Self {
            usage,
            size: checked_len(data)?,
            data: Some(data),
            immutable,
            label: None,
        })
    }

    pub const fn size(&self) -> u32 {
        self.size
    }

    pub fn data_prefix(&self) -> Result<Option<&'a [u8]>, DescriptorSizeError> {
        // A null data pointer is the ordinary source spelling for an
        // uninitialized allocation; `size` is still the requested capacity.
        // Only validate a backing span when the caller supplied one.
        match self.data {
            Some(data) => data
                .get(..self.size as usize)
                .map(Some)
                .ok_or(DescriptorSizeError),
            None => Ok(None),
        }
    }

    pub const fn immutable(&self) -> bool {
        self.immutable
    }
}

#[derive(Clone, Copy)]
pub struct TextureDesc<'a> {
    pub width: u32,
    pub height: u32,
    pub depthOrArrayLayers: u32,
    pub format: TextureFormat,
    pub r#type: TextureType,
    pub renderTarget: bool,
    pub numMipmaps: u32,
    pub sampleCount: u32,
    pub label: Option<&'a str>,
}

impl Default for TextureDesc<'_> {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            depthOrArrayLayers: 1,
            format: TextureFormat::rgba8unorm,
            r#type: TextureType::texture2D,
            renderTarget: false,
            numMipmaps: 1,
            sampleCount: 1,
            label: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextureViewDesc<'a> {
    pub texture: Option<&'a TextureHandle>,
    pub dimension: TextureViewDimension,
    pub aspect: TextureAspect,
    pub baseMipLevel: u32,
    pub mipCount: u32,
    pub baseLayer: u32,
    pub layerCount: u32,
}

impl Default for TextureViewDesc<'_> {
    fn default() -> Self {
        Self {
            texture: None,
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextureDataDesc<'a> {
    pub data: Option<&'a [u8]>,
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
}

impl Default for TextureDataDesc<'_> {
    fn default() -> Self {
        Self {
            data: None,
            bytesPerRow: 0,
            rowsPerImage: 0,
            mipLevel: 0,
            layer: 0,
            x: 0,
            y: 0,
            z: 0,
            width: 0,
            height: 0,
            depth: 1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SamplerDesc<'a> {
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
    pub label: Option<&'a str>,
}

impl Default for SamplerDesc<'_> {
    fn default() -> Self {
        Self {
            minFilter: Filter::nearest,
            magFilter: Filter::nearest,
            mipmapFilter: Filter::nearest,
            wrapU: WrapMode::clampToEdge,
            wrapV: WrapMode::clampToEdge,
            wrapW: WrapMode::clampToEdge,
            compare: CompareFunction::none,
            minLod: 0.0,
            maxLod: 32.0,
            maxAnisotropy: 1,
            label: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShaderModuleDesc<'a> {
    pub code: Option<&'a [u8]>,
    pub codeSize: u32,
    pub language: ShaderLanguage,
    pub stage: ShaderStage,
    pub label: Option<&'a str>,

    // D3D11 only: HLSL source for runtime compilation via D3DCompile.
    // When set, the shader is compiled at first use (ensureD3DShaders)
    // rather than from pre-compiled DXBC. This is required on AMD because
    // D3DCompile produces different DXBC per process context.
    pub hlslSource: Option<&'a str>,
    pub hlslSourceSize: u32,
    pub hlslEntryPoint: Option<&'a str>,

    // Required binding-map sidecar bytes from the RSTB (target IDs 10-13,
    // 16, one per source backend). `makeShaderModule` parses them via
    // `ore::BindingMap::fromBlob` into `ShaderModule::m_bindingMap`. The
    // sidecar is mandatory: a missing or unparseable blob is a programming
    // error, not a fallback condition.
    pub bindingMapBytes: Option<&'a [u8]>,
    pub bindingMapSize: u32,

    // GL program-link fixup blob from the RSTB (target IDs 14/15, one per
    // GLSL stage). Consumed by `oreGLFixupProgramBindings` at
    // `glLinkProgram` time to call `glUniformBlockBinding` / `glUniform1i`
    // by uniform name — no runtime string parsing. Required when the
    // module's source target is GLSL (target 1); null for other targets.
    pub glFixupBytes: Option<&'a [u8]>,
    pub glFixupSize: u32,

    // Source ShaderAsset id (FileAsset::assetId()), or 0 if synthesized.
    // Storage on ShaderModule is gated by TRACK_RIVE_SHADER_ID.
    pub shaderAssetId: u32,
}

impl ShaderModuleDesc<'_> {
    pub fn codeSize(&self) -> Result<u32, DescriptorSizeError> {
        checked_prefix(self.code, self.codeSize).map(|_| self.codeSize)
    }

    pub fn hlslSourceSize(&self) -> Result<u32, DescriptorSizeError> {
        checked_prefix(self.hlslSource.map(str::as_bytes), self.hlslSourceSize)
            .map(|_| self.hlslSourceSize)
    }

    pub fn bindingMapSize(&self) -> Result<u32, DescriptorSizeError> {
        checked_prefix(self.bindingMapBytes, self.bindingMapSize).map(|_| self.bindingMapSize)
    }

    pub fn glFixupSize(&self) -> Result<u32, DescriptorSizeError> {
        checked_prefix(self.glFixupBytes, self.glFixupSize).map(|_| self.glFixupSize)
    }
}

impl Default for ShaderModuleDesc<'_> {
    fn default() -> Self {
        Self {
            code: None,
            codeSize: 0,
            language: ShaderLanguage::glsl,
            stage: ShaderStage::autoDetect,
            label: None,
            hlslSource: None,
            hlslSourceSize: 0,
            hlslEntryPoint: None,
            bindingMapBytes: None,
            bindingMapSize: 0,
            glFixupBytes: None,
            glFixupSize: 0,
            shaderAssetId: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VertexAttribute {
    pub format: VertexFormat,
    pub offset: u32,
    pub shaderSlot: u32,
}

impl Default for VertexAttribute {
    fn default() -> Self {
        Self {
            format: VertexFormat::float4,
            offset: 0,
            shaderSlot: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct VertexBufferLayout<'a> {
    pub stride: u32,
    pub stepMode: VertexStepMode,
    pub attributes: &'a [VertexAttribute],
    pub attributeCount: u32,
}

impl VertexBufferLayout<'_> {
    pub fn attributeCount(&self) -> Result<u32, DescriptorSizeError> {
        self.attributes
            .get(..self.attributeCount as usize)
            .map(|_| self.attributeCount)
            .ok_or(DescriptorSizeError)
    }
}

impl Default for VertexBufferLayout<'_> {
    fn default() -> Self {
        Self {
            stride: 0,
            stepMode: VertexStepMode::vertex,
            attributes: &[],
            attributeCount: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BlendState {
    pub srcColor: BlendFactor,
    pub dstColor: BlendFactor,
    pub colorOp: BlendOp,
    pub srcAlpha: BlendFactor,
    pub dstAlpha: BlendFactor,
    pub alphaOp: BlendOp,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            srcColor: BlendFactor::one,
            dstColor: BlendFactor::zero,
            colorOp: BlendOp::add,
            srcAlpha: BlendFactor::one,
            dstAlpha: BlendFactor::zero,
            alphaOp: BlendOp::add,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ColorTargetState {
    pub format: TextureFormat,
    pub blendEnabled: bool,
    pub blend: BlendState,
    pub writeMask: ColorWriteMask,
}

impl Default for ColorTargetState {
    fn default() -> Self {
        Self {
            format: TextureFormat::bgra8unorm,
            blendEnabled: false,
            blend: BlendState::default(),
            writeMask: ColorWriteMask::all,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StencilFaceState {
    pub compare: CompareFunction,
    pub failOp: StencilOp,
    pub depthFailOp: StencilOp,
    pub passOp: StencilOp,
}

impl Default for StencilFaceState {
    fn default() -> Self {
        Self {
            compare: CompareFunction::always,
            failOp: StencilOp::keep,
            depthFailOp: StencilOp::keep,
            passOp: StencilOp::keep,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DepthStencilState {
    // rgba8unorm is the sentinel for "no depth/stencil attachment" (checked by
    // the Vulkan backend in ore_pipeline_vulkan.cpp). Set this to an actual
    // depth format to attach a depth/stencil buffer to the pipeline.
    pub format: TextureFormat,
    pub depthCompare: CompareFunction,
    pub depthWriteEnabled: bool,
    pub depthBias: i32,
    pub depthBiasSlopeScale: f32,
    pub depthBiasClamp: f32,
}

impl Default for DepthStencilState {
    fn default() -> Self {
        Self {
            format: TextureFormat::rgba8unorm,
            depthCompare: CompareFunction::always,
            depthWriteEnabled: false,
            depthBias: 0,
            depthBiasSlopeScale: 0.0,
            depthBiasClamp: 0.0,
        }
    }
}

// ============================================================================
// BindGroupLayout Descriptor, explicit layout, Dawn-shaped.
//
// One layout per WGSL `@group(N)`. Created via `Context::makeBindGroupLayout`
// and consumed by both `PipelineDesc::bindGroupLayouts[]` and
// `BindGroupDesc::layout`. Replaces the previous "BindGroup is built against
// a Pipeline" coupling — the layout is the contract that pipelines and bind
// groups agree on, so a single BindGroup can be used with any pipeline that
// declares the same layout for its corresponding group.
// ============================================================================

// Resource kind a layout entry describes. Values intentionally distinct from
// `ore::ResourceKind` (in ore_binding_map.hpp) so the public layout API
// doesn't drag the binding-map header into every translation unit. The two
// are mapped 1:1 inside the layout-creation code.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingKind {
    uniformBuffer = 0,
    storageBufferRO = 1,
    storageBufferRW = 2,
    sampledTexture = 3,
    storageTexture = 4,
    sampler = 5,
    comparisonSampler = 6,
}

// Stage-visibility bits. Bitwise-OR of `kStageVS` / `kStageFS` (compute is
// reserved for future use). Mirrors WebGPU's `GPUShaderStage` flagset.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageVisibility {
    pub mask: u8,
}

impl StageVisibility {
    pub const kVertex: u8 = 1u8 << 0;
    pub const kFragment: u8 = 1u8 << 1;
    pub const kCompute: u8 = 1u8 << 2;
}

impl Default for StageVisibility {
    fn default() -> Self {
        Self {
            mask: Self::kVertex | Self::kFragment,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BindGroupLayoutEntry {
    // WGSL `@binding(N)` value this entry describes. The (group, binding)
    // pair must agree with the shader's reflected binding map; mismatch
    // is rejected at `makePipeline` time with a clear error.
    pub binding: u32,

    pub kind: BindingKind,

    // Stage-visibility mask. Default vertex+fragment matches the most
    // common case; visibility narrower than the shader's reflected
    // stageMask is rejected at makePipeline (broader is fine).
    pub visibility: StageVisibility,

    // UBO-only: declares the binding will receive a dynamic offset at
    // `setBindGroup` time. Vulkan DSL picks
    // `VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC` vs `UNIFORM_BUFFER`;
    // D3D12 uses a root CBV instead of an in-table CBV; Metal / GL /
    // D3D11 cache the flag for per-draw `setVertexBuffer` offset apply.
    pub hasDynamicOffset: bool,

    // Texture-only: dimension + sample type + multisampled. Drives the
    // WebGPU BGL entry's `texture.{viewDimension, sampleType, multisampled}`.
    // Ignored for non-texture kinds.
    pub textureViewDim: TextureViewDimension,
    // sampleType is one of "float", "unfilterable-float", "depth", "sint",
    // "uint" in WebGPU. Encoded compactly here as a small enum that
    // backends map to the Dawn enum.
    pub textureSampleType: SampleType,
    pub textureMultisampled: bool,

    // UBO-only: smallest valid bind size for this entry. 0 = no minimum
    // (use the full buffer range). Matches WebGPU's
    // `BindGroupLayoutEntry::buffer.minBindingSize`. Currently advisory —
    // backends don't yet enforce.
    pub minBindingSize: u32,

    // Pre-resolved native slots, per-stage. Populated by the caller from
    // the shader's binding map (typically via the GM helper
    // `makeLayoutFromShader(ctx, shader, group)`). Used by backends with
    // no native layout object (Metal: buffer index; D3D11: per-stage
    // register; GL: global slot). Vulkan and WebGPU ignore — those
    // backends use `binding` directly (per-set namespace).
    //
    // 0xFFFFFFFF = `kAbsent` (binding not visible to that stage). Default
    // is "absent in all stages" so a layout-without-shader-resolution
    // works on Vulkan/WebGPU but fails loudly on Metal/D3D11/GL.
    pub nativeSlotVS: u32,
    pub nativeSlotFS: u32,
    pub nativeSlotCS: u32,
}

impl Default for BindGroupLayoutEntry {
    fn default() -> Self {
        Self {
            binding: 0,
            kind: BindingKind::uniformBuffer,
            visibility: StageVisibility::default(),
            hasDynamicOffset: false,
            textureViewDim: TextureViewDimension::texture2D,
            textureSampleType: SampleType::floatFilterable,
            textureMultisampled: false,
            minBindingSize: 0,
            nativeSlotVS: Self::kNativeSlotAbsent,
            nativeSlotFS: Self::kNativeSlotAbsent,
            nativeSlotCS: Self::kNativeSlotAbsent,
        }
    }
}

// Source-nested enum BindGroupLayoutEntry::SampleType. Rust keeps this as a
// source-shaped sibling because Rust has no nested type declarations in a
// struct body.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleType {
    floatFilterable = 0,
    floatUnfilterable = 1,
    depth = 2,
    sint = 3,
    uint = 4,
}

impl BindGroupLayoutEntry {
    pub const kNativeSlotAbsent: u32 = 0xFFFFFFFFu32;
}

#[derive(Clone, Copy)]
pub struct BindGroupLayoutDesc<'a> {
    // WGSL `@group(N)` this layout describes. Valid range [0, kMaxBindGroups).
    pub groupIndex: u32,

    pub entries: &'a [BindGroupLayoutEntry],
    pub entryCount: u32,
    pub label: Option<&'a str>,
}

impl BindGroupLayoutDesc<'_> {
    pub fn entryCount(&self) -> Result<u32, DescriptorSizeError> {
        self.entries
            .get(..self.entryCount as usize)
            .map(|_| self.entryCount)
            .ok_or(DescriptorSizeError)
    }
}

impl Default for BindGroupLayoutDesc<'_> {
    fn default() -> Self {
        Self {
            groupIndex: 0,
            entries: &[],
            entryCount: 0,
            label: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PipelineDesc<'a> {
    pub vertexModule: Option<&'a ShaderModuleHandle>,
    pub vertexEntryPoint: Option<&'a str>,
    pub fragmentModule: Option<&'a ShaderModuleHandle>,
    pub fragmentEntryPoint: Option<&'a str>,

    pub vertexBuffers: Option<&'a [VertexBufferLayout<'a>]>,
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

    // Explicit bind-group layouts — one entry per `@group(N)` the shader
    // declares bindings for. `bindGroupLayouts[g]` is the layout used when
    // a BindGroup is bound at group index `g` via
    // `RenderPass::setBindGroup(g, ...)`. NULL entries are allowed for
    // groups the shader doesn't use.
    //
    // Required (no auto-derive). Mismatch with the shader's reflected
    // binding map causes `makePipeline` to set lastError + return null.
    pub bindGroupLayouts: Option<&'a [Option<&'a BindGroupLayoutHandle>]>,
    pub bindGroupLayoutCount: u32,
    pub label: Option<&'a str>,
}

impl PipelineDesc<'_> {
    pub fn vertexBufferCount(&self) -> Result<u32, DescriptorSizeError> {
        checked_prefix(self.vertexBuffers, self.vertexBufferCount).map(|_| self.vertexBufferCount)
    }

    pub fn bindGroupLayoutCount(&self) -> Result<u32, DescriptorSizeError> {
        checked_prefix(self.bindGroupLayouts, self.bindGroupLayoutCount)
            .map(|_| self.bindGroupLayoutCount)
    }
}

impl Default for PipelineDesc<'_> {
    fn default() -> Self {
        Self {
            vertexModule: None,
            vertexEntryPoint: Some("vs_main"),
            fragmentModule: None,
            fragmentEntryPoint: Some("fs_main"),
            vertexBuffers: None,
            vertexBufferCount: 0,
            topology: PrimitiveTopology::triangleList,
            indexFormat: IndexFormat::none,
            cullMode: CullMode::none,
            winding: FaceWinding::counterClockwise,
            colorTargets: [ColorTargetState::default(); 4],
            colorCount: 1,
            depthStencil: DepthStencilState::default(),
            stencilFront: StencilFaceState::default(),
            stencilBack: StencilFaceState::default(),
            stencilReadMask: 0xFF,
            stencilWriteMask: 0xFF,
            sampleCount: 1,
            bindGroupLayouts: None,
            bindGroupLayoutCount: 0,
            label: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for ClearColor {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ColorAttachment<'a> {
    pub view: Option<&'a TextureViewHandle>,
    pub resolveTarget: Option<&'a TextureViewHandle>,
    pub loadOp: LoadOp,
    pub storeOp: StoreOp,
    pub clearColor: ClearColor,
}

impl Default for ColorAttachment<'_> {
    fn default() -> Self {
        Self {
            view: None,
            resolveTarget: None,
            loadOp: LoadOp::clear,
            storeOp: StoreOp::store,
            clearColor: ClearColor::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct DepthStencilAttachment<'a> {
    pub view: Option<&'a TextureViewHandle>,
    pub depthLoadOp: LoadOp,
    pub depthStoreOp: StoreOp,
    pub depthClearValue: f32,
    pub stencilLoadOp: LoadOp,
    pub stencilStoreOp: StoreOp,
    pub stencilClearValue: u32,
}

impl Default for DepthStencilAttachment<'_> {
    fn default() -> Self {
        Self {
            view: None,
            depthLoadOp: LoadOp::clear,
            depthStoreOp: StoreOp::store,
            depthClearValue: 1.0,
            stencilLoadOp: LoadOp::clear,
            stencilStoreOp: StoreOp::discard,
            stencilClearValue: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RenderPassDesc<'a> {
    pub colorAttachments: [ColorAttachment<'a>; 4],
    pub colorCount: u32,
    pub depthStencil: DepthStencilAttachment<'a>,
    pub label: Option<&'a str>,
}

impl Default for RenderPassDesc<'_> {
    fn default() -> Self {
        Self {
            colorAttachments: [ColorAttachment::default(); 4],
            colorCount: 1,
            depthStencil: DepthStencilAttachment::default(),
            label: None,
        }
    }
}

// ============================================================================
// BindGroup Descriptor
// ============================================================================

#[derive(Clone, Copy)]
pub struct BindGroupDesc<'a> {
    // The layout the BindGroup conforms to. The layout's `groupIndex`
    // determines which `setBindGroup(g, ...)` slot this BindGroup is
    // bindable at. Resource kinds and per-slot expectations come from the
    // layout's entries.
    pub layout: Option<&'a BindGroupLayoutHandle>,

    // Source-nested BindGroupDesc::UBOEntry, TexEntry, and SampEntry are
    // declared as source-shaped siblings immediately below because Rust does
    // not permit nested struct declarations in a struct body.
    pub ubos: &'a [UBOEntry<'a>],
    pub uboCount: u32,
    pub textures: &'a [TexEntry<'a>],
    pub textureCount: u32,
    pub samplers: &'a [SampEntry<'a>],
    pub samplerCount: u32,
    pub label: Option<&'a str>,
}

// These three records are declared in BindGroupDesc in the C++ source. They
// remain source-shaped sibling records because Rust does not permit nested
// struct declarations in a struct body.
#[derive(Clone, Copy)]
pub struct UBOEntry<'a> {
    pub slot: u32, // WGSL @binding within the layout's group.
    pub buffer: Option<&'a BufferHandle>,
    pub offset: u32,
    pub size: u32,
}

#[derive(Clone, Copy)]
pub struct TexEntry<'a> {
    pub slot: u32,
    pub view: Option<&'a TextureViewHandle>,
}

#[derive(Clone, Copy)]
pub struct SampEntry<'a> {
    pub slot: u32,
    pub sampler: Option<&'a SamplerHandle>,
}

impl Default for UBOEntry<'_> {
    fn default() -> Self {
        Self {
            slot: 0,
            buffer: None,
            offset: 0,
            size: 0,
        }
    }
}

impl Default for TexEntry<'_> {
    fn default() -> Self {
        Self {
            slot: 0,
            view: None,
        }
    }
}

impl Default for SampEntry<'_> {
    fn default() -> Self {
        Self {
            slot: 0,
            sampler: None,
        }
    }
}

impl BindGroupDesc<'_> {
    pub fn uboCount(&self) -> Result<u32, DescriptorSizeError> {
        self.ubos
            .get(..self.uboCount as usize)
            .map(|_| self.uboCount)
            .ok_or(DescriptorSizeError)
    }

    pub fn textureCount(&self) -> Result<u32, DescriptorSizeError> {
        self.textures
            .get(..self.textureCount as usize)
            .map(|_| self.textureCount)
            .ok_or(DescriptorSizeError)
    }

    pub fn samplerCount(&self) -> Result<u32, DescriptorSizeError> {
        self.samplers
            .get(..self.samplerCount as usize)
            .map(|_| self.samplerCount)
            .ok_or(DescriptorSizeError)
    }
}

impl Default for BindGroupDesc<'_> {
    fn default() -> Self {
        Self {
            layout: None,
            ubos: &[],
            uboCount: 0,
            textures: &[],
            textureCount: 0,
            samplers: &[],
            samplerCount: 0,
            label: None,
        }
    }
}

// ============================================================================
// Exact raw descriptor ABI adapters
// ============================================================================

/// Raw `repr(C)` spellings of the pinned C++ descriptor aggregates.  This is
/// intentionally separate from the safe borrowed descriptors above: every
/// source pointer remains a thin raw pointer, every authored count remains an
/// independent `u32`, and null is distinguishable from a non-null empty
/// range. Crossing from this layer into a safe descriptor is `unsafe` because
/// the caller must prove the source pointers remain valid for the returned
/// borrow.
pub mod raw_abi {
    use super::*;
    use core::ffi::{c_char, c_void};
    use core::{ptr, slice, str};
    use std::ffi::CStr;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BorrowError {
        NullWithNonZeroCount,
        InvalidUtf8,
    }

    unsafe fn optional_slice<'a, T>(
        pointer: *const T,
        count: u32,
    ) -> Result<Option<&'a [T]>, BorrowError> {
        if pointer.is_null() {
            return if count == 0 {
                Ok(None)
            } else {
                Err(BorrowError::NullWithNonZeroCount)
            };
        }
        // SAFETY: required by this raw adapter's caller.
        Ok(Some(unsafe {
            slice::from_raw_parts(pointer, count as usize)
        }))
    }

    unsafe fn required_slice<'a, T>(pointer: *const T, count: u32) -> Result<&'a [T], BorrowError> {
        // A source null/zero pair normalizes to Rust's empty safe view only at
        // this explicit conversion boundary; the raw record retains null.
        Ok(unsafe { optional_slice(pointer, count)? }.unwrap_or(&[]))
    }

    unsafe fn optional_ref<'a, T>(pointer: *const T) -> Option<&'a T> {
        // SAFETY: required by this raw adapter's caller.
        unsafe { pointer.as_ref() }
    }

    unsafe fn optional_label<'a>(pointer: *const c_char) -> Result<Option<&'a str>, BorrowError> {
        if pointer.is_null() {
            return Ok(None);
        }
        // SAFETY: required by this raw adapter's caller; C labels are NUL
        // terminated in the pinned aggregate ABI.
        unsafe { CStr::from_ptr(pointer) }
            .to_str()
            .map(Some)
            .map_err(|_| BorrowError::InvalidUtf8)
    }

    unsafe fn optional_utf8_span<'a>(
        pointer: *const c_char,
        count: u32,
    ) -> Result<Option<&'a str>, BorrowError> {
        let bytes = unsafe { optional_slice(pointer.cast::<u8>(), count)? };
        bytes
            .map(str::from_utf8)
            .transpose()
            .map_err(|_| BorrowError::InvalidUtf8)
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct BufferDesc {
        pub usage: BufferUsage,
        pub size: u32,
        pub data: *const c_void,
        pub immutable: bool,
        pub label: *const c_char,
    }

    impl BufferDesc {
        pub unsafe fn borrow<'a>(&self) -> Result<super::BufferDesc<'a>, BorrowError> {
            // `BufferDesc::size` is the allocation size, not an independent
            // pointer count: null data with nonzero size is the ordinary
            // uninitialized-buffer source state.
            let data = if self.data.is_null() {
                None
            } else {
                Some(unsafe { slice::from_raw_parts(self.data.cast::<u8>(), self.size as usize) })
            };
            Ok(super::BufferDesc {
                usage: self.usage,
                size: self.size,
                data,
                immutable: self.immutable,
                label: unsafe { optional_label(self.label)? },
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct TextureDesc {
        pub width: u32,
        pub height: u32,
        pub depthOrArrayLayers: u32,
        pub format: TextureFormat,
        pub r#type: TextureType,
        pub renderTarget: bool,
        pub numMipmaps: u32,
        pub sampleCount: u32,
        pub label: *const c_char,
    }

    impl TextureDesc {
        pub unsafe fn borrow<'a>(&self) -> Result<super::TextureDesc<'a>, BorrowError> {
            Ok(super::TextureDesc {
                width: self.width,
                height: self.height,
                depthOrArrayLayers: self.depthOrArrayLayers,
                format: self.format,
                r#type: self.r#type,
                renderTarget: self.renderTarget,
                numMipmaps: self.numMipmaps,
                sampleCount: self.sampleCount,
                label: unsafe { optional_label(self.label)? },
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct TextureViewDesc {
        pub texture: *mut super::TextureHandle,
        pub dimension: TextureViewDimension,
        pub aspect: TextureAspect,
        pub baseMipLevel: u32,
        pub mipCount: u32,
        pub baseLayer: u32,
        pub layerCount: u32,
    }

    impl TextureViewDesc {
        pub unsafe fn borrow<'a>(&self) -> super::TextureViewDesc<'a> {
            super::TextureViewDesc {
                texture: unsafe { optional_ref(self.texture.cast_const()) },
                dimension: self.dimension,
                aspect: self.aspect,
                baseMipLevel: self.baseMipLevel,
                mipCount: self.mipCount,
                baseLayer: self.baseLayer,
                layerCount: self.layerCount,
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct TextureDataDesc {
        pub data: *const c_void,
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
    }

    impl TextureDataDesc {
        /// `byte_count` is supplied by the caller because the pinned C++
        /// aggregate intentionally carries only a pointer, not its span size.
        pub unsafe fn borrow<'a>(
            &self,
            byte_count: usize,
        ) -> Result<super::TextureDataDesc<'a>, BorrowError> {
            let data = if self.data.is_null() {
                None
            } else {
                // SAFETY: required by this raw adapter's caller.
                Some(unsafe { slice::from_raw_parts(self.data.cast::<u8>(), byte_count) })
            };
            Ok(super::TextureDataDesc {
                data,
                bytesPerRow: self.bytesPerRow,
                rowsPerImage: self.rowsPerImage,
                mipLevel: self.mipLevel,
                layer: self.layer,
                x: self.x,
                y: self.y,
                z: self.z,
                width: self.width,
                height: self.height,
                depth: self.depth,
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct SamplerDesc {
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
        pub label: *const c_char,
    }

    impl SamplerDesc {
        pub unsafe fn borrow<'a>(&self) -> Result<super::SamplerDesc<'a>, BorrowError> {
            Ok(super::SamplerDesc {
                minFilter: self.minFilter,
                magFilter: self.magFilter,
                mipmapFilter: self.mipmapFilter,
                wrapU: self.wrapU,
                wrapV: self.wrapV,
                wrapW: self.wrapW,
                compare: self.compare,
                minLod: self.minLod,
                maxLod: self.maxLod,
                maxAnisotropy: self.maxAnisotropy,
                label: unsafe { optional_label(self.label)? },
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct ShaderModuleDesc {
        pub code: *const c_void,
        pub codeSize: u32,
        pub language: ShaderLanguage,
        pub stage: ShaderStage,
        pub label: *const c_char,
        pub hlslSource: *const c_char,
        pub hlslSourceSize: u32,
        pub hlslEntryPoint: *const c_char,
        pub bindingMapBytes: *const u8,
        pub bindingMapSize: u32,
        pub glFixupBytes: *const u8,
        pub glFixupSize: u32,
        pub shaderAssetId: u32,
    }

    impl ShaderModuleDesc {
        pub unsafe fn borrow<'a>(&self) -> Result<super::ShaderModuleDesc<'a>, BorrowError> {
            Ok(super::ShaderModuleDesc {
                code: unsafe { optional_slice(self.code.cast::<u8>(), self.codeSize)? },
                codeSize: self.codeSize,
                language: self.language,
                stage: self.stage,
                label: unsafe { optional_label(self.label)? },
                hlslSource: unsafe { optional_utf8_span(self.hlslSource, self.hlslSourceSize)? },
                hlslSourceSize: self.hlslSourceSize,
                hlslEntryPoint: unsafe { optional_label(self.hlslEntryPoint)? },
                bindingMapBytes: unsafe {
                    optional_slice(self.bindingMapBytes, self.bindingMapSize)?
                },
                bindingMapSize: self.bindingMapSize,
                glFixupBytes: unsafe { optional_slice(self.glFixupBytes, self.glFixupSize)? },
                glFixupSize: self.glFixupSize,
                shaderAssetId: self.shaderAssetId,
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct VertexBufferLayout {
        pub stride: u32,
        pub stepMode: VertexStepMode,
        pub attributes: *const VertexAttribute,
        pub attributeCount: u32,
    }

    impl VertexBufferLayout {
        pub unsafe fn borrow<'a>(&self) -> Result<super::VertexBufferLayout<'a>, BorrowError> {
            Ok(super::VertexBufferLayout {
                stride: self.stride,
                stepMode: self.stepMode,
                attributes: unsafe { required_slice(self.attributes, self.attributeCount)? },
                attributeCount: self.attributeCount,
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct BindGroupLayoutDesc {
        pub groupIndex: u32,
        pub entries: *const BindGroupLayoutEntry,
        pub entryCount: u32,
        pub label: *const c_char,
    }

    impl BindGroupLayoutDesc {
        pub unsafe fn borrow<'a>(&self) -> Result<super::BindGroupLayoutDesc<'a>, BorrowError> {
            Ok(super::BindGroupLayoutDesc {
                groupIndex: self.groupIndex,
                entries: unsafe { required_slice(self.entries, self.entryCount)? },
                entryCount: self.entryCount,
                label: unsafe { optional_label(self.label)? },
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct PipelineDesc {
        pub vertexModule: *mut super::ShaderModuleHandle,
        pub vertexEntryPoint: *const c_char,
        pub fragmentModule: *mut super::ShaderModuleHandle,
        pub fragmentEntryPoint: *const c_char,
        pub vertexBuffers: *const VertexBufferLayout,
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
        pub bindGroupLayouts: *const *mut super::BindGroupLayoutHandle,
        pub bindGroupLayoutCount: u32,
        pub label: *const c_char,
    }

    impl PipelineDesc {
        pub unsafe fn borrow<'a>(
            &'a self,
            vertexScratch: &'a mut Vec<super::VertexBufferLayout<'a>>,
            layoutScratch: &'a mut Vec<Option<&'a super::BindGroupLayoutHandle>>,
        ) -> Result<super::PipelineDesc<'a>, BorrowError> {
            vertexScratch.clear();
            if let Some(layouts) =
                unsafe { optional_slice(self.vertexBuffers, self.vertexBufferCount)? }
            {
                for layout in layouts {
                    vertexScratch.push(unsafe { layout.borrow()? });
                }
            }
            layoutScratch.clear();
            if let Some(layouts) =
                unsafe { optional_slice(self.bindGroupLayouts, self.bindGroupLayoutCount)? }
            {
                layoutScratch.extend(
                    layouts
                        .iter()
                        .map(|layout| unsafe { optional_ref((*layout).cast_const()) }),
                );
            }
            Ok(super::PipelineDesc {
                vertexModule: unsafe { optional_ref(self.vertexModule.cast_const()) },
                vertexEntryPoint: unsafe { optional_label(self.vertexEntryPoint)? },
                fragmentModule: unsafe { optional_ref(self.fragmentModule.cast_const()) },
                fragmentEntryPoint: unsafe { optional_label(self.fragmentEntryPoint)? },
                vertexBuffers: (!self.vertexBuffers.is_null()).then_some(vertexScratch.as_slice()),
                vertexBufferCount: self.vertexBufferCount,
                topology: self.topology,
                indexFormat: self.indexFormat,
                cullMode: self.cullMode,
                winding: self.winding,
                colorTargets: self.colorTargets,
                colorCount: self.colorCount,
                depthStencil: self.depthStencil,
                stencilFront: self.stencilFront,
                stencilBack: self.stencilBack,
                stencilReadMask: self.stencilReadMask,
                stencilWriteMask: self.stencilWriteMask,
                sampleCount: self.sampleCount,
                bindGroupLayouts: (!self.bindGroupLayouts.is_null())
                    .then_some(layoutScratch.as_slice()),
                bindGroupLayoutCount: self.bindGroupLayoutCount,
                label: unsafe { optional_label(self.label)? },
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct ColorAttachment {
        pub view: *mut super::TextureViewHandle,
        pub resolveTarget: *mut super::TextureViewHandle,
        pub loadOp: LoadOp,
        pub storeOp: StoreOp,
        pub clearColor: ClearColor,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct DepthStencilAttachment {
        pub view: *mut super::TextureViewHandle,
        pub depthLoadOp: LoadOp,
        pub depthStoreOp: StoreOp,
        pub depthClearValue: f32,
        pub stencilLoadOp: LoadOp,
        pub stencilStoreOp: StoreOp,
        pub stencilClearValue: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct RenderPassDesc {
        pub colorAttachments: [ColorAttachment; 4],
        pub colorCount: u32,
        pub depthStencil: DepthStencilAttachment,
        pub label: *const c_char,
    }

    impl RenderPassDesc {
        pub unsafe fn borrow<'a>(&self) -> Result<super::RenderPassDesc<'a>, BorrowError> {
            Ok(super::RenderPassDesc {
                colorAttachments: core::array::from_fn(|index| {
                    let source = self.colorAttachments[index];
                    super::ColorAttachment {
                        view: unsafe { optional_ref(source.view.cast_const()) },
                        resolveTarget: unsafe { optional_ref(source.resolveTarget.cast_const()) },
                        loadOp: source.loadOp,
                        storeOp: source.storeOp,
                        clearColor: source.clearColor,
                    }
                }),
                colorCount: self.colorCount,
                depthStencil: super::DepthStencilAttachment {
                    view: unsafe { optional_ref(self.depthStencil.view.cast_const()) },
                    depthLoadOp: self.depthStencil.depthLoadOp,
                    depthStoreOp: self.depthStencil.depthStoreOp,
                    depthClearValue: self.depthStencil.depthClearValue,
                    stencilLoadOp: self.depthStencil.stencilLoadOp,
                    stencilStoreOp: self.depthStencil.stencilStoreOp,
                    stencilClearValue: self.depthStencil.stencilClearValue,
                },
                label: unsafe { optional_label(self.label)? },
            })
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct UBOEntry {
        pub slot: u32,
        pub buffer: *mut super::BufferHandle,
        pub offset: u32,
        pub size: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct TexEntry {
        pub slot: u32,
        pub view: *mut super::TextureViewHandle,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct SampEntry {
        pub slot: u32,
        pub sampler: *mut super::SamplerHandle,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct BindGroupDesc {
        pub layout: *mut super::BindGroupLayoutHandle,
        pub ubos: *const UBOEntry,
        pub uboCount: u32,
        pub textures: *const TexEntry,
        pub textureCount: u32,
        pub samplers: *const SampEntry,
        pub samplerCount: u32,
        pub label: *const c_char,
    }

    impl BindGroupDesc {
        pub unsafe fn borrow<'a>(
            &'a self,
            uboScratch: &'a mut Vec<super::UBOEntry<'a>>,
            textureScratch: &'a mut Vec<super::TexEntry<'a>>,
            samplerScratch: &'a mut Vec<super::SampEntry<'a>>,
        ) -> Result<super::BindGroupDesc<'a>, BorrowError> {
            uboScratch.clear();
            for entry in unsafe { required_slice(self.ubos, self.uboCount)? } {
                uboScratch.push(super::UBOEntry {
                    slot: entry.slot,
                    buffer: unsafe { optional_ref(entry.buffer.cast_const()) },
                    offset: entry.offset,
                    size: entry.size,
                });
            }
            textureScratch.clear();
            for entry in unsafe { required_slice(self.textures, self.textureCount)? } {
                textureScratch.push(super::TexEntry {
                    slot: entry.slot,
                    view: unsafe { optional_ref(entry.view.cast_const()) },
                });
            }
            samplerScratch.clear();
            for entry in unsafe { required_slice(self.samplers, self.samplerCount)? } {
                samplerScratch.push(super::SampEntry {
                    slot: entry.slot,
                    sampler: unsafe { optional_ref(entry.sampler.cast_const()) },
                });
            }
            Ok(super::BindGroupDesc {
                layout: unsafe { optional_ref(self.layout.cast_const()) },
                ubos: uboScratch.as_slice(),
                uboCount: self.uboCount,
                textures: textureScratch.as_slice(),
                textureCount: self.textureCount,
                samplers: samplerScratch.as_slice(),
                samplerCount: self.samplerCount,
                label: unsafe { optional_label(self.label)? },
            })
        }
    }

    // Compile-time ABI-shape assertions for the two most error-prone source
    // distinctions. Thin raw pointers remain one machine word; safe slices
    // remain outside this module and are never transmuted into these records.
    const _: () = {
        assert!(core::mem::size_of::<*const c_void>() == core::mem::size_of::<usize>());
        assert!(core::mem::size_of::<*const VertexAttribute>() == core::mem::size_of::<usize>());
        let _ = ptr::null::<c_void>();
    };
}

// ============================================================================
// Features — runtime capability query
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Features {
    // 32-bit float color render targets (rgba32float etc).
    pub colorBufferFloat: bool,
    // 16-bit float color render targets (rgba16float etc). Implied by
    // colorBufferFloat, but on WebGL can be present without it.
    pub colorBufferHalfFloat: bool,
    pub perTargetBlend: bool,
    pub perTargetWriteMask: bool,
    pub textureViewSampling: bool,
    pub drawBaseInstance: bool,
    pub depthBiasClamp: bool,
    pub anisotropicFiltering: bool,
    pub texture3D: bool,
    pub textureArrays: bool,
    pub computeShaders: bool,
    pub storageBuffers: bool,

    pub bc: bool,
    pub etc2: bool,
    pub astc: bool,

    pub maxColorAttachments: u32,
    pub maxTextureSize2D: u32,
    pub maxTextureSizeCube: u32,
    pub maxTextureSize3D: u32,
    pub maxUniformBufferSize: u32,
    pub maxVertexAttributes: u32,
    pub maxSamplers: u32,
    // Maximum MSAA sample count supported for color render targets.
    // Scripts should query this before creating MSAA textures; values are
    // always a power-of-two (1, 2, 4, 8). Conservative default: 4.
    pub maxSamples: u32,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            colorBufferFloat: false,
            colorBufferHalfFloat: false,
            perTargetBlend: false,
            perTargetWriteMask: false,
            textureViewSampling: false,
            drawBaseInstance: false,
            depthBiasClamp: false,
            anisotropicFiltering: false,
            texture3D: false,
            textureArrays: false,
            computeShaders: false,
            storageBuffers: false,
            bc: false,
            etc2: false,
            astc: false,
            maxColorAttachments: 4,
            maxTextureSize2D: 4096,
            maxTextureSizeCube: 4096,
            maxTextureSize3D: 256,
            maxUniformBufferSize: 16384,
            maxVertexAttributes: 16,
            maxSamplers: 16,
            maxSamples: 4,
        }
    }
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_traits_are_object_safe_thread_safe_and_checked() {
        let buffer = BufferDesc::initialized(BufferUsage::uniform, b"data", true).expect("buffer");
        assert_eq!(buffer.size(), 4);
        assert!(buffer.immutable());
        let texture = TextureDesc::default();
        assert_eq!(texture.depthOrArrayLayers, 1);
    }

    #[test]
    fn retaining_descriptor_positions_expose_exact_strong_owners() {
        let pipeline = PipelineDesc {
            bindGroupLayouts: Some(&[]),
            ..PipelineDesc::default()
        };
        assert!(pipeline.bindGroupLayouts.is_some());
        let ubos: &[UBOEntry<'_>] = &[];
        let bind_group = BindGroupDesc {
            ubos,
            ..BindGroupDesc::default()
        };
        assert!(bind_group.ubos.is_empty());
    }

    #[test]
    fn safe_pointer_size_pairs_derive_exact_lengths() {
        let initialized =
            BufferDesc::initialized(BufferUsage::uniform, b"bytes", true).expect("small buffer");
        assert_eq!(initialized.size(), 5);
        assert_eq!(
            initialized.data_prefix().unwrap(),
            Some(b"bytes".as_slice())
        );
        assert!(initialized.immutable());

        let uninitialized = BufferDesc::uninitialized(BufferUsage::upload, 64);
        assert_eq!(uninitialized.size(), 64);
        assert_eq!(uninitialized.data_prefix().unwrap(), None);
        assert!(!uninitialized.immutable());

        let shader = ShaderModuleDesc {
            code: Some(b"code"),
            codeSize: 4,
            hlslSource: Some("hlsl"),
            hlslSourceSize: 4,
            bindingMapBytes: Some(&[1, 2, 3]),
            bindingMapSize: 3,
            glFixupBytes: Some(&[4, 5]),
            glFixupSize: 2,
            ..ShaderModuleDesc::default()
        };
        assert_eq!(shader.codeSize(), Ok(4));
        assert_eq!(shader.hlslSourceSize(), Ok(4));
        assert_eq!(shader.bindingMapSize(), Ok(3));
        assert_eq!(shader.glFixupSize(), Ok(2));
    }

    #[test]
    fn textureFormatBytesPerTexel_matches_upstream() {
        let uncompressed = [
            (TextureFormat::r8unorm, 1),
            (TextureFormat::rg8unorm, 2),
            (TextureFormat::rgba8unorm, 4),
            (TextureFormat::rgba8snorm, 4),
            (TextureFormat::bgra8unorm, 4),
            (TextureFormat::rgba16float, 8),
            (TextureFormat::rg16float, 4),
            (TextureFormat::r16float, 2),
            (TextureFormat::rgba32float, 16),
            (TextureFormat::rg32float, 8),
            (TextureFormat::r32float, 4),
            (TextureFormat::rgb10a2unorm, 4),
            (TextureFormat::r11g11b10float, 4),
            (TextureFormat::depth16unorm, 2),
            (TextureFormat::depth24plusStencil8, 4),
            (TextureFormat::depth32float, 4),
            (TextureFormat::depth32floatStencil8, 8),
        ];
        for (format, bytes) in uncompressed {
            assert_eq!(textureFormatBytesPerTexel(format), bytes);
        }

        for format in [
            TextureFormat::bc1unorm,
            TextureFormat::bc3unorm,
            TextureFormat::bc7unorm,
            TextureFormat::etc2rgb8,
            TextureFormat::etc2rgba8,
            TextureFormat::astc4x4,
            TextureFormat::astc6x6,
            TextureFormat::astc8x8,
        ] {
            assert_eq!(textureFormatBytesPerTexel(format), 0);
        }
    }

    #[test]
    fn colorWriteMask_operators_match_upstream() {
        assert_eq!((ColorWriteMask::red | ColorWriteMask::green).bits(), 0x3);
        assert_eq!(
            (ColorWriteMask::red | ColorWriteMask::blue | ColorWriteMask::alpha).bits(),
            0xD
        );
        assert_eq!(
            (ColorWriteMask::none | ColorWriteMask::all),
            ColorWriteMask::all
        );
        assert_eq!(
            (ColorWriteMask::red | ColorWriteMask::red),
            ColorWriteMask::red
        );
        assert_eq!(
            (ColorWriteMask::all & ColorWriteMask::red),
            ColorWriteMask::red
        );
        assert_eq!(
            (ColorWriteMask::red | ColorWriteMask::green) & ColorWriteMask::red,
            ColorWriteMask::red
        );
        assert_eq!(
            (ColorWriteMask::none & ColorWriteMask::all),
            ColorWriteMask::none
        );
        assert_eq!(
            ColorWriteMask::red & ColorWriteMask::green,
            ColorWriteMask::none
        );
    }

    #[test]
    fn enum_discriminants_match_source_order() {
        fn assert_contiguous(values: &[u8]) {
            for (expected, value) in values.iter().copied().enumerate() {
                assert_eq!(usize::from(value), expected);
            }
        }

        assert_contiguous(&[
            BufferUsage::vertex as u8,
            BufferUsage::index as u8,
            BufferUsage::uniform as u8,
            BufferUsage::upload as u8,
        ]);
        assert_contiguous(&[ShaderLanguage::glsl as u8, ShaderLanguage::wgsl as u8]);
        assert_contiguous(&[
            ShaderStage::autoDetect as u8,
            ShaderStage::vertex as u8,
            ShaderStage::fragment as u8,
        ]);
        assert_contiguous(&[
            TextureFormat::r8unorm as u8,
            TextureFormat::rg8unorm as u8,
            TextureFormat::rgba8unorm as u8,
            TextureFormat::rgba8snorm as u8,
            TextureFormat::bgra8unorm as u8,
            TextureFormat::rgba16float as u8,
            TextureFormat::rg16float as u8,
            TextureFormat::r16float as u8,
            TextureFormat::rgba32float as u8,
            TextureFormat::rg32float as u8,
            TextureFormat::r32float as u8,
            TextureFormat::rgb10a2unorm as u8,
            TextureFormat::r11g11b10float as u8,
            TextureFormat::depth16unorm as u8,
            TextureFormat::depth24plusStencil8 as u8,
            TextureFormat::depth32float as u8,
            TextureFormat::depth32floatStencil8 as u8,
            TextureFormat::bc1unorm as u8,
            TextureFormat::bc3unorm as u8,
            TextureFormat::bc7unorm as u8,
            TextureFormat::etc2rgb8 as u8,
            TextureFormat::etc2rgba8 as u8,
            TextureFormat::astc4x4 as u8,
            TextureFormat::astc6x6 as u8,
            TextureFormat::astc8x8 as u8,
        ]);
        assert_contiguous(&[
            TextureType::texture2D as u8,
            TextureType::cube as u8,
            TextureType::texture3D as u8,
            TextureType::array2D as u8,
        ]);
        assert_contiguous(&[
            TextureViewDimension::texture2D as u8,
            TextureViewDimension::cube as u8,
            TextureViewDimension::texture3D as u8,
            TextureViewDimension::array2D as u8,
            TextureViewDimension::cubeArray as u8,
        ]);
        assert_contiguous(&[
            TextureAspect::all as u8,
            TextureAspect::depthOnly as u8,
            TextureAspect::stencilOnly as u8,
        ]);
        assert_contiguous(&[Filter::nearest as u8, Filter::linear as u8]);
        assert_contiguous(&[
            WrapMode::repeat as u8,
            WrapMode::mirrorRepeat as u8,
            WrapMode::clampToEdge as u8,
        ]);
        assert_contiguous(&[
            CompareFunction::none as u8,
            CompareFunction::never as u8,
            CompareFunction::less as u8,
            CompareFunction::equal as u8,
            CompareFunction::lessEqual as u8,
            CompareFunction::greater as u8,
            CompareFunction::notEqual as u8,
            CompareFunction::greaterEqual as u8,
            CompareFunction::always as u8,
        ]);
        assert_contiguous(&[
            PrimitiveTopology::pointList as u8,
            PrimitiveTopology::lineList as u8,
            PrimitiveTopology::lineStrip as u8,
            PrimitiveTopology::triangleList as u8,
            PrimitiveTopology::triangleStrip as u8,
        ]);
        assert_contiguous(&[
            IndexFormat::none as u8,
            IndexFormat::uint16 as u8,
            IndexFormat::uint32 as u8,
        ]);
        assert_contiguous(&[
            VertexFormat::float1 as u8,
            VertexFormat::float2 as u8,
            VertexFormat::float3 as u8,
            VertexFormat::float4 as u8,
            VertexFormat::uint8x4 as u8,
            VertexFormat::sint8x4 as u8,
            VertexFormat::unorm8x4 as u8,
            VertexFormat::snorm8x4 as u8,
            VertexFormat::uint16x2 as u8,
            VertexFormat::sint16x2 as u8,
            VertexFormat::unorm16x2 as u8,
            VertexFormat::snorm16x2 as u8,
            VertexFormat::uint16x4 as u8,
            VertexFormat::sint16x4 as u8,
            VertexFormat::float16x2 as u8,
            VertexFormat::float16x4 as u8,
            VertexFormat::uint32 as u8,
        ]);
        assert_contiguous(&[VertexStepMode::vertex as u8, VertexStepMode::instance as u8]);
        assert_contiguous(&[
            CullMode::none as u8,
            CullMode::front as u8,
            CullMode::back as u8,
        ]);
        assert_contiguous(&[
            FaceWinding::clockwise as u8,
            FaceWinding::counterClockwise as u8,
        ]);
        assert_contiguous(&[
            BlendFactor::zero as u8,
            BlendFactor::one as u8,
            BlendFactor::srcColor as u8,
            BlendFactor::oneMinusSrcColor as u8,
            BlendFactor::srcAlpha as u8,
            BlendFactor::oneMinusSrcAlpha as u8,
            BlendFactor::dstColor as u8,
            BlendFactor::oneMinusDstColor as u8,
            BlendFactor::dstAlpha as u8,
            BlendFactor::oneMinusDstAlpha as u8,
            BlendFactor::srcAlphaSaturated as u8,
            BlendFactor::blendColor as u8,
            BlendFactor::oneMinusBlendColor as u8,
        ]);
        assert_contiguous(&[
            BlendOp::add as u8,
            BlendOp::subtract as u8,
            BlendOp::reverseSubtract as u8,
            BlendOp::min as u8,
            BlendOp::max as u8,
        ]);
        assert_contiguous(&[
            StencilOp::keep as u8,
            StencilOp::zero as u8,
            StencilOp::replace as u8,
            StencilOp::incrementClamp as u8,
            StencilOp::decrementClamp as u8,
            StencilOp::invert as u8,
            StencilOp::incrementWrap as u8,
            StencilOp::decrementWrap as u8,
        ]);
        assert_contiguous(&[
            LoadOp::clear as u8,
            LoadOp::load as u8,
            LoadOp::dontCare as u8,
        ]);
        assert_contiguous(&[StoreOp::store as u8, StoreOp::discard as u8]);
        assert_contiguous(&[
            BindingKind::uniformBuffer as u8,
            BindingKind::storageBufferRO as u8,
            BindingKind::storageBufferRW as u8,
            BindingKind::sampledTexture as u8,
            BindingKind::storageTexture as u8,
            BindingKind::sampler as u8,
            BindingKind::comparisonSampler as u8,
        ]);
        assert_contiguous(&[
            SampleType::floatFilterable as u8,
            SampleType::floatUnfilterable as u8,
            SampleType::depth as u8,
            SampleType::sint as u8,
            SampleType::uint as u8,
        ]);
        assert_eq!(ColorWriteMask::none.bits(), 0);
        assert_eq!(ColorWriteMask::red.bits(), 1);
        assert_eq!(ColorWriteMask::green.bits(), 2);
        assert_eq!(ColorWriteMask::blue.bits(), 4);
        assert_eq!(ColorWriteMask::alpha.bits(), 8);
        assert_eq!(ColorWriteMask::all.bits(), 15);
    }

    #[test]
    fn descriptor_defaults_match_source() {
        let buffer = BufferDesc::uninitialized(BufferUsage::vertex, 0);
        assert_eq!(buffer.size(), 0);
        assert!(buffer.data_prefix().unwrap().is_none());
        assert!(!buffer.immutable());
        assert!(buffer.label.is_none());

        let texture = TextureDesc::default();
        assert_eq!(texture.width, 0);
        assert_eq!(texture.height, 0);
        assert_eq!(texture.depthOrArrayLayers, 1);
        assert_eq!(texture.format, TextureFormat::rgba8unorm);
        assert_eq!(texture.r#type, TextureType::texture2D);
        assert!(!texture.renderTarget);
        assert_eq!(texture.numMipmaps, 1);
        assert_eq!(texture.sampleCount, 1);
        assert!(texture.label.is_none());

        let sampler = SamplerDesc::default();
        assert_eq!(sampler.minFilter, Filter::nearest);
        assert_eq!(sampler.wrapU, WrapMode::clampToEdge);
        assert_eq!(sampler.compare, CompareFunction::none);
        assert_eq!(sampler.minLod, 0.0);
        assert_eq!(sampler.maxLod, 32.0);
        assert_eq!(sampler.maxAnisotropy, 1);

        let shader = ShaderModuleDesc::default();
        assert!(shader.code.is_none());
        assert_eq!(shader.language, ShaderLanguage::glsl);
        assert_eq!(shader.stage, ShaderStage::autoDetect);
        assert_eq!(shader.codeSize(), Ok(0));
        assert_eq!(shader.hlslSourceSize(), Ok(0));
        assert_eq!(shader.bindingMapSize(), Ok(0));
        assert_eq!(shader.glFixupSize(), Ok(0));
        assert_eq!(shader.shaderAssetId, 0);

        let attribute = VertexAttribute::default();
        assert_eq!(attribute.format, VertexFormat::float4);
        assert_eq!(attribute.offset, 0);
        assert_eq!(attribute.shaderSlot, 0);

        let vertex_buffer = VertexBufferLayout::default();
        assert_eq!(vertex_buffer.stride, 0);
        assert_eq!(vertex_buffer.stepMode, VertexStepMode::vertex);
        assert!(vertex_buffer.attributes.is_empty());

        let blend = BlendState::default();
        assert_eq!(blend.srcColor, BlendFactor::one);
        assert_eq!(blend.dstColor, BlendFactor::zero);
        assert_eq!(blend.colorOp, BlendOp::add);
        assert_eq!(blend.srcAlpha, BlendFactor::one);
        assert_eq!(blend.dstAlpha, BlendFactor::zero);
        assert_eq!(blend.alphaOp, BlendOp::add);

        let color_target = ColorTargetState::default();
        assert_eq!(color_target.format, TextureFormat::bgra8unorm);
        assert!(!color_target.blendEnabled);
        assert_eq!(color_target.writeMask, ColorWriteMask::all);

        let stencil = StencilFaceState::default();
        assert_eq!(stencil.compare, CompareFunction::always);
        assert_eq!(stencil.failOp, StencilOp::keep);
        assert_eq!(stencil.depthFailOp, StencilOp::keep);
        assert_eq!(stencil.passOp, StencilOp::keep);

        let depth = DepthStencilState::default();
        assert_eq!(depth.format, TextureFormat::rgba8unorm);
        assert_eq!(depth.depthCompare, CompareFunction::always);
        assert!(!depth.depthWriteEnabled);
        assert_eq!(depth.depthBias, 0);
        assert_eq!(depth.depthBiasSlopeScale, 0.0);
        assert_eq!(depth.depthBiasClamp, 0.0);

        let layout = BindGroupLayoutEntry::default();
        assert_eq!(layout.binding, 0);
        assert_eq!(layout.kind, BindingKind::uniformBuffer);
        assert_eq!(
            layout.visibility.mask,
            StageVisibility::kVertex | StageVisibility::kFragment
        );
        assert_eq!(layout.nativeSlotVS, BindGroupLayoutEntry::kNativeSlotAbsent);
        assert_eq!(layout.nativeSlotFS, BindGroupLayoutEntry::kNativeSlotAbsent);
        assert_eq!(layout.nativeSlotCS, BindGroupLayoutEntry::kNativeSlotAbsent);
        assert!(!layout.hasDynamicOffset);
        assert_eq!(layout.textureViewDim, TextureViewDimension::texture2D);
        assert_eq!(layout.textureSampleType, SampleType::floatFilterable);
        assert!(!layout.textureMultisampled);
        assert_eq!(layout.minBindingSize, 0);

        let layout_desc = BindGroupLayoutDesc::default();
        assert_eq!(layout_desc.groupIndex, 0);
        assert!(layout_desc.entries.is_empty());
        assert!(layout_desc.label.is_none());

        let pipeline = PipelineDesc::default();
        assert!(pipeline.vertexModule.is_none());
        assert_eq!(pipeline.vertexEntryPoint, Some("vs_main"));
        assert!(pipeline.fragmentModule.is_none());
        assert_eq!(pipeline.fragmentEntryPoint, Some("fs_main"));
        assert!(pipeline.vertexBuffers.is_none());
        assert_eq!(pipeline.topology, PrimitiveTopology::triangleList);
        assert_eq!(pipeline.indexFormat, IndexFormat::none);
        assert_eq!(pipeline.cullMode, CullMode::none);
        assert_eq!(pipeline.winding, FaceWinding::counterClockwise);
        assert_eq!(pipeline.colorTargets.len(), 4);
        assert_eq!(pipeline.colorCount, 1);
        assert_eq!(pipeline.sampleCount, 1);
        assert_eq!(pipeline.stencilReadMask, 0xFF);
        assert_eq!(pipeline.stencilWriteMask, 0xFF);
        assert!(pipeline.bindGroupLayouts.is_none());

        let clear = ClearColor::default();
        assert_eq!(clear.r, 0.0);
        assert_eq!(clear.g, 0.0);
        assert_eq!(clear.b, 0.0);
        assert_eq!(clear.a, 1.0);

        let render_pass = RenderPassDesc::default();
        assert_eq!(render_pass.colorAttachments.len(), 4);
        assert_eq!(render_pass.colorCount, 1);
        assert!(render_pass.colorAttachments[0].view.is_none());
        assert!(render_pass.colorAttachments[0].resolveTarget.is_none());
        assert_eq!(render_pass.colorAttachments[0].loadOp, LoadOp::clear);
        assert_eq!(render_pass.colorAttachments[0].storeOp, StoreOp::store);
        assert!(render_pass.depthStencil.view.is_none());
        assert_eq!(render_pass.depthStencil.depthClearValue, 1.0);
        assert_eq!(render_pass.depthStencil.stencilStoreOp, StoreOp::discard);
        assert!(render_pass.label.is_none());

        let ubo = UBOEntry::default();
        assert_eq!(ubo.slot, 0);
        assert!(ubo.buffer.is_none());
        assert_eq!(ubo.offset, 0);
        assert_eq!(ubo.size, 0);

        let texture_entry = TexEntry::default();
        assert_eq!(texture_entry.slot, 0);
        assert!(texture_entry.view.is_none());

        let sampler_entry = SampEntry::default();
        assert_eq!(sampler_entry.slot, 0);
        assert!(sampler_entry.sampler.is_none());

        let bind_group = BindGroupDesc::default();
        assert!(bind_group.layout.is_none());
        assert!(bind_group.ubos.is_empty());
        assert!(bind_group.textures.is_empty());
        assert!(bind_group.samplers.is_empty());
        assert!(bind_group.label.is_none());

        let features = Features::default();
        assert!(!features.colorBufferFloat);
        assert_eq!(features.maxColorAttachments, 4);
        assert_eq!(features.maxTextureSize2D, 4096);
        assert_eq!(features.maxTextureSizeCube, 4096);
        assert_eq!(features.maxTextureSize3D, 256);
        assert_eq!(features.maxUniformBufferSize, 16384);
        assert_eq!(features.maxVertexAttributes, 16);
        assert_eq!(features.maxSamplers, 16);
        assert_eq!(features.maxSamples, 4);
    }

    #[test]
    fn source_shapes_are_fixed() {
        assert_eq!(std::mem::size_of::<BufferUsage>(), 1);
        assert_eq!(std::mem::size_of::<TextureFormat>(), 1);
        assert_eq!(std::mem::size_of::<BindingKind>(), 1);
        assert_eq!(std::mem::size_of::<SampleType>(), 1);
        assert_eq!(std::mem::size_of::<ColorWriteMask>(), 1);
        assert_eq!(kMaxBindGroups, 4);
        assert_eq!(PipelineDesc::default().colorTargets.len(), 4);
        assert_eq!(RenderPassDesc::default().colorAttachments.len(), 4);
    }
}
