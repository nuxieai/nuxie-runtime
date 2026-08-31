//! Exact string/enum mappings from upstream lua_gpu.cpp at e949498e.
use super::*;
pub(super) fn buffer_usage(value: &str) -> Result<BufferUsage> {
    match value {
        "vertex" => Ok(BufferUsage::vertex),
        "index" => Ok(BufferUsage::index),
        "uniform" => Ok(BufferUsage::uniform),
        _ => Err(Error::runtime(format!(
            "invalid BufferUsage '{value}' (expected 'vertex', 'index', or 'uniform')"
        ))),
    }
}
pub(super) fn texture_format(value: &str) -> Result<TextureFormat> {
    match value {
        "r8unorm" => Ok(TextureFormat::r8unorm),
        "rg8unorm" => Ok(TextureFormat::rg8unorm),
        "rgba8unorm" => Ok(TextureFormat::rgba8unorm),
        "bgra8unorm" => Ok(TextureFormat::bgra8unorm),
        "rgba16float" => Ok(TextureFormat::rgba16float),
        "rg16float" => Ok(TextureFormat::rg16float),
        "r16float" => Ok(TextureFormat::r16float),
        "rgba32float" => Ok(TextureFormat::rgba32float),
        "rgb10a2unorm" => Ok(TextureFormat::rgb10a2unorm),
        "rg11b10ufloat" => Ok(TextureFormat::r11g11b10float),
        "depth16unorm" => Ok(TextureFormat::depth16unorm),
        "depth24plus-stencil8" => Ok(TextureFormat::depth24plusStencil8),
        "depth32float" => Ok(TextureFormat::depth32float),
        "depth32float-stencil8" => Ok(TextureFormat::depth32floatStencil8),
        "bc1-rgba-unorm" => Ok(TextureFormat::bc1unorm),
        "bc3-rgba-unorm" => Ok(TextureFormat::bc3unorm),
        "bc7-rgba-unorm" => Ok(TextureFormat::bc7unorm),
        "etc2-rgb8unorm" => Ok(TextureFormat::etc2rgb8),
        "etc2-rgba8unorm" => Ok(TextureFormat::etc2rgba8),
        "astc-4x4-unorm" => Ok(TextureFormat::astc4x4),
        "astc-6x6-unorm" => Ok(TextureFormat::astc6x6),
        "astc-8x8-unorm" => Ok(TextureFormat::astc8x8),
        _ => Err(Error::runtime(format!("invalid TextureFormat: {value}"))),
    }
}
pub(super) fn texture_type(value: &str) -> Result<TextureType> {
    match value {
        "2d" => Ok(TextureType::texture2D),
        "cube" => Ok(TextureType::cube),
        "3d" => Ok(TextureType::texture3D),
        "2d-array" => Ok(TextureType::array2D),
        _ => Err(Error::runtime(format!("invalid TextureType: {value}"))),
    }
}
pub(super) fn compare(value: &str) -> Result<CompareFunction> {
    match value {
        "never" => Ok(CompareFunction::never),
        "less" => Ok(CompareFunction::less),
        "equal" => Ok(CompareFunction::equal),
        "less-equal" => Ok(CompareFunction::lessEqual),
        "greater" => Ok(CompareFunction::greater),
        "not-equal" => Ok(CompareFunction::notEqual),
        "greater-equal" => Ok(CompareFunction::greaterEqual),
        "always" => Ok(CompareFunction::always),
        _ => Err(Error::runtime(format!("invalid CompareFunction: {value}"))),
    }
}
pub(super) fn filter(value: &str) -> Result<Filter> {
    match value {
        "nearest" => Ok(Filter::nearest),
        "linear" => Ok(Filter::linear),
        _ => Err(Error::runtime(format!("invalid Filter: {value}"))),
    }
}
pub(super) fn wrap_mode(value: &str) -> Result<WrapMode> {
    match value {
        "repeat" => Ok(WrapMode::repeat),
        "mirror-repeat" => Ok(WrapMode::mirrorRepeat),
        "clamp-to-edge" => Ok(WrapMode::clampToEdge),
        _ => Err(Error::runtime(format!("invalid WrapMode: {value}"))),
    }
}
pub(super) fn vertex_format(value: &str) -> Result<VertexFormat> {
    match value {
        "float32" => Ok(VertexFormat::float1),
        "float32x2" => Ok(VertexFormat::float2),
        "float32x3" => Ok(VertexFormat::float3),
        "float32x4" => Ok(VertexFormat::float4),
        "uint8x4" => Ok(VertexFormat::uint8x4),
        "unorm8x4" => Ok(VertexFormat::unorm8x4),
        "snorm8x4" => Ok(VertexFormat::snorm8x4),
        "float16x2" => Ok(VertexFormat::float16x2),
        "float16x4" => Ok(VertexFormat::float16x4),
        _ => Err(Error::runtime(format!("invalid VertexFormat: {value}"))),
    }
}
pub(super) fn cull_mode(value: &str) -> Result<CullMode> {
    match value {
        "none" => Ok(CullMode::none),
        "front" => Ok(CullMode::front),
        "back" => Ok(CullMode::back),
        _ => Err(Error::runtime(format!("invalid CullMode: {value}"))),
    }
}
pub(super) fn topology(value: &str) -> Result<PrimitiveTopology> {
    match value {
        "triangle-list" => Ok(PrimitiveTopology::triangleList),
        "triangle-strip" => Ok(PrimitiveTopology::triangleStrip),
        "line-list" => Ok(PrimitiveTopology::lineList),
        "line-strip" => Ok(PrimitiveTopology::lineStrip),
        "point-list" => Ok(PrimitiveTopology::pointList),
        _ => Err(Error::runtime(format!(
            "invalid PrimitiveTopology: {value}"
        ))),
    }
}
pub(super) fn blend_factor(value: &str) -> Result<BlendFactor> {
    match value {
        "zero" => Ok(BlendFactor::zero),
        "one" => Ok(BlendFactor::one),
        "src" => Ok(BlendFactor::srcColor),
        "one-minus-src" => Ok(BlendFactor::oneMinusSrcColor),
        "src-alpha" => Ok(BlendFactor::srcAlpha),
        "one-minus-src-alpha" => Ok(BlendFactor::oneMinusSrcAlpha),
        "dst" => Ok(BlendFactor::dstColor),
        "one-minus-dst" => Ok(BlendFactor::oneMinusDstColor),
        "dst-alpha" => Ok(BlendFactor::dstAlpha),
        "one-minus-dst-alpha" => Ok(BlendFactor::oneMinusDstAlpha),
        "src-alpha-saturated" => Ok(BlendFactor::srcAlphaSaturated),
        "constant" => Ok(BlendFactor::blendColor),
        "one-minus-constant" => Ok(BlendFactor::oneMinusBlendColor),
        _ => Err(Error::runtime(format!("invalid BlendFactor: {value}"))),
    }
}
pub(super) fn blend_op(value: &str) -> Result<BlendOp> {
    match value {
        "add" => Ok(BlendOp::add),
        "subtract" => Ok(BlendOp::subtract),
        "reverse-subtract" => Ok(BlendOp::reverseSubtract),
        "min" => Ok(BlendOp::min),
        "max" => Ok(BlendOp::max),
        _ => Err(Error::runtime(format!("invalid BlendOp: {value}"))),
    }
}
pub(super) fn winding(value: &str) -> Result<FaceWinding> {
    match value {
        "cw" => Ok(FaceWinding::clockwise),
        "ccw" => Ok(FaceWinding::counterClockwise),
        _ => Err(Error::runtime(format!("invalid FaceWinding: {value}"))),
    }
}
pub(super) fn stencil_op(value: &str) -> Result<StencilOp> {
    match value {
        "keep" => Ok(StencilOp::keep),
        "zero" => Ok(StencilOp::zero),
        "replace" => Ok(StencilOp::replace),
        "increment-clamp" => Ok(StencilOp::incrementClamp),
        "decrement-clamp" => Ok(StencilOp::decrementClamp),
        "invert" => Ok(StencilOp::invert),
        "increment-wrap" => Ok(StencilOp::incrementWrap),
        "decrement-wrap" => Ok(StencilOp::decrementWrap),
        _ => Err(Error::runtime(format!("invalid StencilOp: {value}"))),
    }
}
