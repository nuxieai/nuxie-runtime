//! Fixed-width ORE enum/descriptor codecs for 966499ff wire PODs.
use crate::cmd::command_stream::WirePod;
use crate::types::*;

impl Default for BufferUsage {
    fn default() -> Self {
        Self::vertex
    }
}
impl WirePod for BufferUsage {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::vertex,
            1 => Self::index,
            2 => Self::uniform,
            3 => Self::upload,
            _ => panic!("invalid BufferUsage in command stream"),
        }
    }
}
impl Default for ShaderLanguage {
    fn default() -> Self {
        Self::glsl
    }
}
impl WirePod for ShaderLanguage {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::glsl,
            1 => Self::wgsl,
            _ => panic!("invalid ShaderLanguage in command stream"),
        }
    }
}
impl Default for ShaderStage {
    fn default() -> Self {
        Self::autoDetect
    }
}
impl WirePod for ShaderStage {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::autoDetect,
            1 => Self::vertex,
            2 => Self::fragment,
            _ => panic!("invalid ShaderStage in command stream"),
        }
    }
}
impl Default for TextureFormat {
    fn default() -> Self {
        Self::r8unorm
    }
}
impl WirePod for TextureFormat {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::r8unorm,
            1 => Self::rg8unorm,
            2 => Self::rgba8unorm,
            3 => Self::rgba8snorm,
            4 => Self::bgra8unorm,
            5 => Self::rgba16float,
            6 => Self::rg16float,
            7 => Self::r16float,
            8 => Self::rgba32float,
            9 => Self::rg32float,
            10 => Self::r32float,
            11 => Self::rgb10a2unorm,
            12 => Self::r11g11b10float,
            13 => Self::depth16unorm,
            14 => Self::depth24plusStencil8,
            15 => Self::depth32float,
            16 => Self::depth32floatStencil8,
            17 => Self::bc1unorm,
            18 => Self::bc3unorm,
            19 => Self::bc7unorm,
            20 => Self::etc2rgb8,
            21 => Self::etc2rgba8,
            22 => Self::astc4x4,
            23 => Self::astc6x6,
            24 => Self::astc8x8,
            _ => panic!("invalid TextureFormat in command stream"),
        }
    }
}
impl Default for TextureType {
    fn default() -> Self {
        Self::texture2D
    }
}
impl WirePod for TextureType {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::texture2D,
            1 => Self::cube,
            2 => Self::texture3D,
            3 => Self::array2D,
            _ => panic!("invalid TextureType in command stream"),
        }
    }
}
impl Default for TextureViewDimension {
    fn default() -> Self {
        Self::texture2D
    }
}
impl WirePod for TextureViewDimension {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::texture2D,
            1 => Self::cube,
            2 => Self::texture3D,
            3 => Self::array2D,
            4 => Self::cubeArray,
            _ => panic!("invalid TextureViewDimension in command stream"),
        }
    }
}
impl Default for TextureAspect {
    fn default() -> Self {
        Self::all
    }
}
impl WirePod for TextureAspect {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::all,
            1 => Self::depthOnly,
            2 => Self::stencilOnly,
            _ => panic!("invalid TextureAspect in command stream"),
        }
    }
}
impl Default for Filter {
    fn default() -> Self {
        Self::nearest
    }
}
impl WirePod for Filter {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::nearest,
            1 => Self::linear,
            _ => panic!("invalid Filter in command stream"),
        }
    }
}
impl Default for WrapMode {
    fn default() -> Self {
        Self::repeat
    }
}
impl WirePod for WrapMode {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::repeat,
            1 => Self::mirrorRepeat,
            2 => Self::clampToEdge,
            _ => panic!("invalid WrapMode in command stream"),
        }
    }
}
impl Default for CompareFunction {
    fn default() -> Self {
        Self::none
    }
}
impl WirePod for CompareFunction {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::none,
            1 => Self::never,
            2 => Self::less,
            3 => Self::equal,
            4 => Self::lessEqual,
            5 => Self::greater,
            6 => Self::notEqual,
            7 => Self::greaterEqual,
            8 => Self::always,
            _ => panic!("invalid CompareFunction in command stream"),
        }
    }
}
impl Default for PrimitiveTopology {
    fn default() -> Self {
        Self::pointList
    }
}
impl WirePod for PrimitiveTopology {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::pointList,
            1 => Self::lineList,
            2 => Self::lineStrip,
            3 => Self::triangleList,
            4 => Self::triangleStrip,
            _ => panic!("invalid PrimitiveTopology in command stream"),
        }
    }
}
impl Default for IndexFormat {
    fn default() -> Self {
        Self::none
    }
}
impl WirePod for IndexFormat {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::none,
            1 => Self::uint16,
            2 => Self::uint32,
            _ => panic!("invalid IndexFormat in command stream"),
        }
    }
}
impl Default for VertexFormat {
    fn default() -> Self {
        Self::float1
    }
}
impl WirePod for VertexFormat {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::float1,
            1 => Self::float2,
            2 => Self::float3,
            3 => Self::float4,
            4 => Self::uint8x4,
            5 => Self::sint8x4,
            6 => Self::unorm8x4,
            7 => Self::snorm8x4,
            8 => Self::uint16x2,
            9 => Self::sint16x2,
            10 => Self::unorm16x2,
            11 => Self::snorm16x2,
            12 => Self::uint16x4,
            13 => Self::sint16x4,
            14 => Self::float16x2,
            15 => Self::float16x4,
            16 => Self::uint32,
            _ => panic!("invalid VertexFormat in command stream"),
        }
    }
}
impl Default for VertexStepMode {
    fn default() -> Self {
        Self::vertex
    }
}
impl WirePod for VertexStepMode {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::vertex,
            1 => Self::instance,
            _ => panic!("invalid VertexStepMode in command stream"),
        }
    }
}
impl Default for CullMode {
    fn default() -> Self {
        Self::none
    }
}
impl WirePod for CullMode {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::none,
            1 => Self::front,
            2 => Self::back,
            _ => panic!("invalid CullMode in command stream"),
        }
    }
}
impl Default for FaceWinding {
    fn default() -> Self {
        Self::clockwise
    }
}
impl WirePod for FaceWinding {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::clockwise,
            1 => Self::counterClockwise,
            _ => panic!("invalid FaceWinding in command stream"),
        }
    }
}
impl Default for BlendFactor {
    fn default() -> Self {
        Self::zero
    }
}
impl WirePod for BlendFactor {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::zero,
            1 => Self::one,
            2 => Self::srcColor,
            3 => Self::oneMinusSrcColor,
            4 => Self::srcAlpha,
            5 => Self::oneMinusSrcAlpha,
            6 => Self::dstColor,
            7 => Self::oneMinusDstColor,
            8 => Self::dstAlpha,
            9 => Self::oneMinusDstAlpha,
            10 => Self::srcAlphaSaturated,
            11 => Self::blendColor,
            12 => Self::oneMinusBlendColor,
            _ => panic!("invalid BlendFactor in command stream"),
        }
    }
}
impl Default for BlendOp {
    fn default() -> Self {
        Self::add
    }
}
impl WirePod for BlendOp {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::add,
            1 => Self::subtract,
            2 => Self::reverseSubtract,
            3 => Self::min,
            4 => Self::max,
            _ => panic!("invalid BlendOp in command stream"),
        }
    }
}
impl Default for StencilOp {
    fn default() -> Self {
        Self::keep
    }
}
impl WirePod for StencilOp {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::keep,
            1 => Self::zero,
            2 => Self::replace,
            3 => Self::incrementClamp,
            4 => Self::decrementClamp,
            5 => Self::invert,
            6 => Self::incrementWrap,
            7 => Self::decrementWrap,
            _ => panic!("invalid StencilOp in command stream"),
        }
    }
}
impl Default for LoadOp {
    fn default() -> Self {
        Self::clear
    }
}
impl WirePod for LoadOp {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::clear,
            1 => Self::load,
            2 => Self::dontCare,
            _ => panic!("invalid LoadOp in command stream"),
        }
    }
}
impl Default for StoreOp {
    fn default() -> Self {
        Self::store
    }
}
impl WirePod for StoreOp {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::store,
            1 => Self::discard,
            _ => panic!("invalid StoreOp in command stream"),
        }
    }
}
impl Default for BindingKind {
    fn default() -> Self {
        Self::uniformBuffer
    }
}
impl WirePod for BindingKind {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::uniformBuffer,
            1 => Self::storageBufferRO,
            2 => Self::storageBufferRW,
            3 => Self::sampledTexture,
            4 => Self::storageTexture,
            5 => Self::sampler,
            6 => Self::comparisonSampler,
            _ => panic!("invalid BindingKind in command stream"),
        }
    }
}
impl Default for SampleType {
    fn default() -> Self {
        Self::floatFilterable
    }
}
impl WirePod for SampleType {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self as u8);
    }
    fn decode(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => Self::floatFilterable,
            1 => Self::floatUnfilterable,
            2 => Self::depth,
            3 => Self::sint,
            4 => Self::uint,
            _ => panic!("invalid SampleType in command stream"),
        }
    }
}

impl Default for ColorWriteMask {
    fn default() -> Self {
        Self(0)
    }
}
impl WirePod for ColorWriteMask {
    const SIZE: usize = 1;
    fn encode(&self, bytes: &mut Vec<u8>) {
        self.0.encode(bytes);
    }
    fn decode(bytes: &[u8]) -> Self {
        Self(u8::decode(bytes))
    }
}
crate::impl_wire_pod!(VertexAttribute {
    offset: u32,
    shaderSlot: u32,
    format: VertexFormat,
    pad: [u8; 3]
});
crate::impl_wire_pod!(BlendState {
    srcColor: BlendFactor,
    dstColor: BlendFactor,
    colorOp: BlendOp,
    srcAlpha: BlendFactor,
    dstAlpha: BlendFactor,
    alphaOp: BlendOp
});
crate::impl_wire_pod!(ColorTargetState {
    format: TextureFormat,
    blendEnabled: bool,
    blend: BlendState,
    writeMask: ColorWriteMask
});
crate::impl_wire_pod!(StencilFaceState {
    compare: CompareFunction,
    failOp: StencilOp,
    depthFailOp: StencilOp,
    passOp: StencilOp
});
crate::impl_wire_pod!(DepthStencilState {
    format: TextureFormat,
    depthCompare: CompareFunction,
    depthWriteEnabled: bool,
    pad: u8,
    depthBias: i32,
    depthBiasSlopeScale: f32,
    depthBiasClamp: f32
});
crate::impl_wire_pod!(StageVisibility { mask: u8 });
crate::impl_wire_pod!(BindGroupLayoutEntry {
    binding: u32,
    kind: BindingKind,
    visibility: StageVisibility,
    hasDynamicOffset: bool,
    textureViewDim: TextureViewDimension,
    textureSampleType: SampleType,
    textureMultisampled: bool,
    pad: [u8; 2],
    minBindingSize: u32,
    nativeSlotVS: u32,
    nativeSlotFS: u32,
    nativeSlotCS: u32
});
