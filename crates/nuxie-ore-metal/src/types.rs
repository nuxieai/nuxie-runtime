// Mechanical translation of
// renderer/include/rive/renderer/ore/ore_types.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::any::{Any, TypeId};
use std::ops::{BitAnd, BitOr};
use std::sync::Arc;

// ============================================================================
// Constants
// ============================================================================

// Maximum number of bind groups Ore supports per pipeline. WebGPU's
// maxBindGroups minimum is 4, and Ore sits at that minimum. Backends
// preallocate per-group structures using this constant.
pub const kMaxBindGroups: u32 = 4;

// ============================================================================
// Enums
// ============================================================================

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferUsage {
    vertex,
    index,
    uniform,
    upload,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderLanguage {
    glsl,
    wgsl,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderStage {
    autoDetect,
    vertex,
    fragment,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureFormat {
    // 8-bit
    r8unorm,
    rg8unorm,
    rgba8unorm,
    rgba8snorm,
    bgra8unorm,

    // 16-bit float
    rgba16float,
    rg16float,
    r16float,

    // 32-bit float
    rgba32float,
    rg32float,
    r32float,

    // Packed
    rgb10a2unorm,
    r11g11b10float,

    // Depth/stencil
    depth16unorm,
    depth24plusStencil8,
    depth32float,
    depth32floatStencil8,

    // Compressed (runtime support via Features query)
    bc1unorm,
    bc3unorm,
    bc7unorm,
    etc2rgb8,
    etc2rgba8,
    astc4x4,
    astc6x6,
    astc8x8,
}

// Returns bytes per texel for uncompressed formats, or 0 for block-compressed
// formats (which require block-based stride calculation).
#[allow(non_snake_case)]
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
        _ => 0,
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureType {
    texture2D,
    cube,
    texture3D,
    array2D,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureViewDimension {
    texture2D,
    cube,
    texture3D,
    array2D,
    cubeArray,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureAspect {
    all,
    depthOnly,
    stencilOnly,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    nearest,
    linear,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapMode {
    repeat,
    mirrorRepeat,
    clampToEdge,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareFunction {
    none,
    never,
    less,
    equal,
    lessEqual,
    greater,
    notEqual,
    greaterEqual,
    always,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveTopology {
    pointList,
    lineList,
    lineStrip,
    triangleList,
    triangleStrip,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexFormat {
    none,
    uint16,
    uint32,
}

// 32-bit integer vector vertex formats are intentionally omitted: scripts
// don't expose them, and Unreal RHI only supports scalar VET_UInt.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexFormat {
    float1,
    float2,
    float3,
    float4,
    uint8x4,
    sint8x4,
    unorm8x4,
    snorm8x4,
    uint16x2,
    sint16x2,
    unorm16x2,
    snorm16x2,
    uint16x4,
    sint16x4,
    float16x2,
    float16x4,
    uint32,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexStepMode {
    vertex,
    instance,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CullMode {
    none,
    front,
    back,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceWinding {
    clockwise,
    counterClockwise,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendFactor {
    zero,
    one,
    srcColor,
    oneMinusSrcColor,
    srcAlpha,
    oneMinusSrcAlpha,
    dstColor,
    oneMinusDstColor,
    dstAlpha,
    oneMinusDstAlpha,
    srcAlphaSaturated,
    blendColor,
    oneMinusBlendColor,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendOp {
    add,
    subtract,
    reverseSubtract,
    min,
    max,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StencilOp {
    keep,
    zero,
    replace,
    incrementClamp,
    decrementClamp,
    invert,
    incrementWrap,
    decrementWrap,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadOp {
    clear,
    load,
    dontCare,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreOp {
    store,
    discard,
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
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        // Mirrors static_cast<uint8_t>(a) | static_cast<uint8_t>(b).
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for ColorWriteMask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        // Mirrors static_cast<uint8_t>(a) & static_cast<uint8_t>(b).
        Self(self.0 & rhs.0)
    }
}

// ============================================================================
// Forward declarations
// ============================================================================

/// Type identity for a backend marker chosen by an adapter.
///
/// This keeps cross-backend rejection explicit without defining a backend
/// enum or otherwise turning the portable ORE types into a HAL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BackendId(TypeId);

impl BackendId {
    pub fn of<T: Any>() -> Self {
        Self(TypeId::of::<T>())
    }
}

macro_rules! resource_trait {
    ($name:ident) => {
        pub trait $name: Any + Send + Sync {
            fn backend_id(&self) -> BackendId;
            fn as_any(&self) -> &(dyn Any + Send + Sync);
        }

        impl dyn $name + '_ {
            /// Downcast only after verifying the resource came from the
            /// backend expected by the consuming adapter.
            pub fn downcast_ref<T: $name>(&self, backend_id: BackendId) -> Option<&T> {
                if self.backend_id() != backend_id {
                    return None;
                }
                self.as_any().downcast_ref::<T>()
            }
        }
    };
}

// Narrow backend seams used by the descriptors. Concrete adapters implement
// these traits; the ORE type layer intentionally defines no HAL.
resource_trait!(Buffer);
resource_trait!(Texture);
resource_trait!(TextureView);
resource_trait!(Sampler);
resource_trait!(ShaderModule);
resource_trait!(Pipeline);
resource_trait!(BindGroupLayout);

// ============================================================================
// Descriptor Structs
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DescriptorSizeError;

pub struct BufferDesc<'a> {
    pub usage: BufferUsage,
    size: u32,
    data: Option<&'a [u8]>,
    immutable: bool,
    pub label: Option<&'a str>,
}

impl<'a> BufferDesc<'a> {
    /// Describe mutable storage without initial bytes.
    pub fn uninitialized(usage: BufferUsage, size: u32) -> Self {
        Self {
            usage,
            size,
            data: None,
            immutable: false,
            label: None,
        }
    }

    /// Describe storage initialized from the complete byte slice.
    pub fn initialized(
        usage: BufferUsage,
        data: &'a [u8],
        immutable: bool,
    ) -> Result<Self, DescriptorSizeError> {
        let size = u32::try_from(data.len()).map_err(|_| DescriptorSizeError)?;
        Ok(Self {
            usage,
            size,
            data: Some(data),
            immutable,
            label: None,
        })
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn data(&self) -> Option<&'a [u8]> {
        self.data
    }

    pub fn immutable(&self) -> bool {
        self.immutable
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl<'a> Default for TextureDesc<'a> {
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

#[derive(Clone)]
pub struct TextureViewDesc {
    pub texture: Arc<dyn Texture>,
    pub dimension: TextureViewDimension,
    pub aspect: TextureAspect,
    pub baseMipLevel: u32,
    pub mipCount: u32,
    pub baseLayer: u32,
    pub layerCount: u32,
}

pub struct TextureDataDesc<'a> {
    pub data: &'a [u8],
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

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl<'a> Default for SamplerDesc<'a> {
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

pub struct ShaderModuleDesc<'a> {
    pub code: Option<&'a [u8]>,
    pub language: ShaderLanguage,
    pub stage: ShaderStage,
    pub label: Option<&'a str>,
    pub hlslSource: Option<&'a str>,
    pub hlslEntryPoint: Option<&'a str>,
    pub bindingMapBytes: Option<&'a [u8]>,
    pub glFixupBytes: Option<&'a [u8]>,
    pub shaderAssetId: u32,
}

impl ShaderModuleDesc<'_> {
    pub fn codeSize(&self) -> Result<u32, DescriptorSizeError> {
        checked_optional_len(self.code)
    }

    pub fn hlslSourceSize(&self) -> Result<u32, DescriptorSizeError> {
        checked_optional_len(self.hlslSource.map(str::as_bytes))
    }

    pub fn bindingMapSize(&self) -> Result<u32, DescriptorSizeError> {
        checked_optional_len(self.bindingMapBytes)
    }

    pub fn glFixupSize(&self) -> Result<u32, DescriptorSizeError> {
        checked_optional_len(self.glFixupBytes)
    }
}

fn checked_optional_len(bytes: Option<&[u8]>) -> Result<u32, DescriptorSizeError> {
    u32::try_from(bytes.map_or(0, <[u8]>::len)).map_err(|_| DescriptorSizeError)
}

impl<'a> Default for ShaderModuleDesc<'a> {
    fn default() -> Self {
        Self {
            code: None,
            language: ShaderLanguage::glsl,
            stage: ShaderStage::autoDetect,
            label: None,
            hlslSource: None,
            hlslEntryPoint: None,
            bindingMapBytes: None,
            glFixupBytes: None,
            shaderAssetId: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

impl<'a> Default for VertexBufferLayout<'a> {
    fn default() -> Self {
        Self {
            stride: 0,
            stepMode: VertexStepMode::vertex,
            attributes: &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthStencilState {
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

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingKind {
    uniformBuffer,
    storageBufferRO,
    storageBufferRW,
    sampledTexture,
    storageTexture,
    sampler,
    comparisonSampler,
}

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

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleType {
    floatFilterable,
    floatUnfilterable,
    depth,
    sint,
    uint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindGroupLayoutEntry {
    pub binding: u32,
    pub kind: BindingKind,
    pub visibility: StageVisibility,
    pub hasDynamicOffset: bool,
    pub textureViewDim: TextureViewDimension,
    pub textureSampleType: SampleType,
    pub textureMultisampled: bool,
    pub minBindingSize: u32,
    pub nativeSlotVS: u32,
    pub nativeSlotFS: u32,
    pub nativeSlotCS: u32,
}

impl BindGroupLayoutEntry {
    pub const kNativeSlotAbsent: u32 = 0xFFFF_FFFF;
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

pub struct BindGroupLayoutDesc<'a> {
    pub groupIndex: u32,
    pub entries: &'a [BindGroupLayoutEntry],
    pub label: Option<&'a str>,
}

impl<'a> Default for BindGroupLayoutDesc<'a> {
    fn default() -> Self {
        Self {
            groupIndex: 0,
            entries: &[],
            label: None,
        }
    }
}

pub struct PipelineDesc<'a> {
    pub vertexModule: Option<&'a Arc<dyn ShaderModule>>,
    pub vertexEntryPoint: Option<&'a str>,
    pub fragmentModule: Option<&'a Arc<dyn ShaderModule>>,
    pub fragmentEntryPoint: Option<&'a str>,
    pub vertexBuffers: Option<&'a [VertexBufferLayout<'a>]>,
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
    pub bindGroupLayouts: Option<&'a [Option<Arc<dyn BindGroupLayout>>]>,
    pub label: Option<&'a str>,
}

impl<'a> Default for PipelineDesc<'a> {
    fn default() -> Self {
        Self {
            vertexModule: None,
            vertexEntryPoint: Some("vs_main"),
            fragmentModule: None,
            fragmentEntryPoint: Some("fs_main"),
            vertexBuffers: None,
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
            label: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub view: Option<&'a dyn TextureView>,
    pub resolveTarget: Option<&'a dyn TextureView>,
    pub loadOp: LoadOp,
    pub storeOp: StoreOp,
    pub clearColor: ClearColor,
}

impl<'a> Default for ColorAttachment<'a> {
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
    pub view: Option<&'a dyn TextureView>,
    pub depthLoadOp: LoadOp,
    pub depthStoreOp: StoreOp,
    pub depthClearValue: f32,
    pub stencilLoadOp: LoadOp,
    pub stencilStoreOp: StoreOp,
    pub stencilClearValue: u32,
}

impl<'a> Default for DepthStencilAttachment<'a> {
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

pub struct RenderPassDesc<'a> {
    pub colorAttachments: [ColorAttachment<'a>; 4],
    pub colorCount: u32,
    pub depthStencil: DepthStencilAttachment<'a>,
    pub label: Option<&'a str>,
}

impl<'a> Default for RenderPassDesc<'a> {
    fn default() -> Self {
        Self {
            colorAttachments: [ColorAttachment::default(); 4],
            colorCount: 1,
            depthStencil: DepthStencilAttachment::default(),
            label: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct UBOEntry<'a> {
    pub slot: u32,
    pub buffer: Option<&'a Arc<dyn Buffer>>,
    pub offset: u32,
    pub size: u32,
}

#[derive(Clone, Copy)]
pub struct TexEntry<'a> {
    pub slot: u32,
    pub view: Option<&'a Arc<dyn TextureView>>,
}

#[derive(Clone, Copy)]
pub struct SampEntry<'a> {
    pub slot: u32,
    pub sampler: Option<&'a Arc<dyn Sampler>>,
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

pub struct BindGroupDesc<'a> {
    pub layout: Option<&'a Arc<dyn BindGroupLayout>>,
    pub ubos: &'a [UBOEntry<'a>],
    pub textures: &'a [TexEntry<'a>],
    pub samplers: &'a [SampEntry<'a>],
    pub label: Option<&'a str>,
}

impl<'a> Default for BindGroupDesc<'a> {
    fn default() -> Self {
        Self {
            layout: None,
            ubos: &[],
            textures: &[],
            samplers: &[],
            label: None,
        }
    }
}

// ============================================================================
// Features — runtime capability query
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Features {
    pub colorBufferFloat: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MetalBackend;
    struct OtherBackend;
    struct MetalBuffer;
    struct MetalShaderModule;
    struct MetalBindGroupLayout;

    macro_rules! impl_test_resource {
        ($trait_name:ident, $type_name:ident) => {
            impl $trait_name for $type_name {
                fn backend_id(&self) -> BackendId {
                    BackendId::of::<MetalBackend>()
                }

                fn as_any(&self) -> &(dyn Any + Send + Sync) {
                    self
                }
            }
        };
    }

    impl_test_resource!(Buffer, MetalBuffer);
    impl_test_resource!(ShaderModule, MetalShaderModule);
    impl_test_resource!(BindGroupLayout, MetalBindGroupLayout);

    #[test]
    fn resource_traits_are_object_safe_thread_safe_and_checked() {
        fn assert_any_send_sync<T: ?Sized + Any + Send + Sync>() {}
        assert_any_send_sync::<dyn Buffer>();
        assert_any_send_sync::<dyn Texture>();
        assert_any_send_sync::<dyn TextureView>();
        assert_any_send_sync::<dyn Sampler>();
        assert_any_send_sync::<dyn ShaderModule>();
        assert_any_send_sync::<dyn Pipeline>();
        assert_any_send_sync::<dyn BindGroupLayout>();

        let buffer: Arc<dyn Buffer> = Arc::new(MetalBuffer);
        assert!(
            buffer
                .downcast_ref::<MetalBuffer>(BackendId::of::<MetalBackend>())
                .is_some()
        );
        assert!(
            buffer
                .downcast_ref::<MetalBuffer>(BackendId::of::<OtherBackend>())
                .is_none()
        );
    }

    #[test]
    fn retaining_descriptor_positions_expose_exact_strong_owners() {
        let module: Arc<dyn ShaderModule> = Arc::new(MetalShaderModule);
        let layout: Arc<dyn BindGroupLayout> = Arc::new(MetalBindGroupLayout);
        let layouts = [Some(Arc::clone(&layout))];
        let pipeline = PipelineDesc {
            vertexModule: Some(&module),
            bindGroupLayouts: Some(&layouts),
            ..PipelineDesc::default()
        };

        let retained_module = Arc::clone(pipeline.vertexModule.expect("module owner"));
        let retained_layout = Arc::clone(
            pipeline.bindGroupLayouts.expect("layout owners")[0]
                .as_ref()
                .expect("layout owner"),
        );
        assert!(Arc::ptr_eq(&module, &retained_module));
        assert!(Arc::ptr_eq(&layout, &retained_layout));

        let buffer: Arc<dyn Buffer> = Arc::new(MetalBuffer);
        let ubos = [UBOEntry {
            buffer: Some(&buffer),
            ..UBOEntry::default()
        }];
        let bind_group = BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            ..BindGroupDesc::default()
        };
        let retained_buffer = Arc::clone(bind_group.ubos[0].buffer.expect("buffer owner"));
        assert!(Arc::ptr_eq(&buffer, &retained_buffer));
    }

    #[test]
    fn safe_pointer_size_pairs_derive_exact_lengths() {
        let initialized =
            BufferDesc::initialized(BufferUsage::uniform, b"bytes", true).expect("small buffer");
        assert_eq!(initialized.size(), 5);
        assert_eq!(initialized.data(), Some(b"bytes".as_slice()));
        assert!(initialized.immutable());

        let uninitialized = BufferDesc::uninitialized(BufferUsage::upload, 64);
        assert_eq!(uninitialized.size(), 64);
        assert!(uninitialized.data().is_none());
        assert!(!uninitialized.immutable());

        let shader = ShaderModuleDesc {
            code: Some(b"code"),
            hlslSource: Some("hlsl"),
            bindingMapBytes: Some(&[1, 2, 3]),
            glFixupBytes: Some(&[4, 5]),
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
        assert!(buffer.data().is_none());
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
