//! Complete mechanical implementation translation of
//! `renderer/src/ore/wgpu/ore_context_wgpu.cpp`.

#![allow(non_snake_case)]

use super::ore_bind_group_layout_wgpu_decl::BindGroupLayoutWGPU;
use super::ore_bind_group_wgpu_decl::{
    BindGroupWGPU, SampEntry as NativeSampEntry, TexEntry as NativeTexEntry,
    UBOEntry as NativeUBOEntry,
};
use super::ore_buffer_wgpu_decl::{Backing, BufferWGPU};
use super::ore_context_wgpu_decl::{ContextWGPU, WGPUBackend};
use super::ore_pipeline_wgpu_decl::PipelineWGPU;
use super::ore_render_pass_wgpu_decl::RenderPassWGPU;
use super::ore_sampler_wgpu_decl::SamplerWGPU;
use super::ore_shader_module_wgpu_decl::ShaderModuleWGPU;
use super::ore_texture_wgpu_decl::{TextureViewWGPU, TextureWGPU};
use super::render_context_webgpu_decl::RenderTargetWebGPU;
use super::webgpu_cpp_decl::{
    AddressMode as WgpuAddressMode, BackendType as WagyuBackendType,
    BlendFactor as WgpuBlendFactor, BlendOperation as WgpuBlendOperation,
    BufferUsage as WgpuBufferUsage, ColorWriteMask as WgpuColorWriteMask, CommandEncoder,
    CompareFunction as WgpuCompareFunction, CullMode as WgpuCullMode, Device,
    FilterMode as WgpuFilterMode, FrontFace as WgpuFrontFace, IndexFormat as WgpuIndexFormat,
    MipmapFilterMode as WgpuMipmapFilterMode, PrimitiveTopology as WgpuPrimitiveTopology, Queue,
    ShaderModule as WagyuShaderModule, StencilOperation as WgpuStencilOperation,
    TextureAspect as WgpuTextureAspect, TextureDimension as WgpuTextureDimension,
    TextureFormat as WgpuTextureFormat, TextureUsage as WgpuTextureUsage,
    TextureViewDimension as WgpuTextureViewDimension, VertexFormat as WgpuVertexFormat,
    VertexStepMode as WgpuVertexStepMode,
};
use super::webgpu_decl::{
    WGPUBindGroupLayout, WGPUBlendState, WGPUBufferDescriptor, WGPUChainedStruct,
    WGPUColorTargetState, WGPUDepthStencilState, WGPUFragmentState, WGPUIndexFormat_Undefined,
    WGPULoadOp_Clear, WGPULoadOp_Load, WGPUMultisampleState, WGPUOptionalBool_False,
    WGPUOptionalBool_True, WGPUPipelineLayoutDescriptor, WGPUPrimitiveState,
    WGPURenderPassColorAttachment, WGPURenderPassDepthStencilAttachment, WGPURenderPassDescriptor,
    WGPURenderPipelineDescriptor, WGPUSamplerDescriptor, WGPUShaderModuleDescriptor,
    WGPUShaderSourceWGSL, WGPUStoreOp_Discard, WGPUStoreOp_Store, WGPUStringView,
    WGPUTextureDescriptor, WGPUTextureViewDescriptor, WGPUVertexAttribute, WGPUVertexBufferLayout,
    WGPUVertexState, WGPU_FALSE, WGPU_STRLEN, WGPU_TRUE,
};
use super::webgpu_wagyu_decl::{
    WGPUSType_WagyuShaderModuleDescriptor, WGPUWagyuShaderLanguage, WGPUWagyuShaderLanguage_GLSL,
    WGPUWagyuShaderLanguage_GLSLRAW, WGPUWagyuShaderLanguage_WGSL, WGPUWagyuShaderModuleDescriptor,
};
use nuxie_ore_metal::context::{ActiveRenderPass, ContextApi, FrameDescriptor, ShaderTarget};
use nuxie_ore_metal::gpu_resource::{AnyResourceHandle, ResourceHandle};
use nuxie_ore_metal::render_pass::RenderPassApi;
use nuxie_ore_metal::types::{
    kMaxBindGroups, BindGroupDesc, BindGroupLayoutDesc, BindingKind, BlendFactor, BlendOp,
    BufferDesc, BufferUsage, ColorWriteMask, CompareFunction, CullMode, FaceWinding, Features,
    Filter, IndexFormat, LoadOp, PipelineDesc, PrimitiveTopology, RenderPassDesc, SamplerDesc,
    ShaderLanguage, ShaderModuleDesc, StencilOp, StoreOp, TextureAspect, TextureDesc,
    TextureFormat, TextureType, TextureViewDesc, TextureViewDimension, VertexFormat,
    VertexStepMode, WrapMode,
};
use std::rc::Weak as RcWeak;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_context_wgpu.cpp");

pub(crate) fn oreFormatToWGPU(fmt: TextureFormat) -> WgpuTextureFormat {
    match fmt {
        TextureFormat::r8unorm => WgpuTextureFormat::R8Unorm,
        TextureFormat::rg8unorm => WgpuTextureFormat::RG8Unorm,
        TextureFormat::rgba8unorm => WgpuTextureFormat::RGBA8Unorm,
        TextureFormat::rgba8snorm => WgpuTextureFormat::RGBA8Snorm,
        TextureFormat::bgra8unorm => WgpuTextureFormat::BGRA8Unorm,
        TextureFormat::rgba16float => WgpuTextureFormat::RGBA16Float,
        TextureFormat::rg16float => WgpuTextureFormat::RG16Float,
        TextureFormat::r16float => WgpuTextureFormat::R16Float,
        TextureFormat::rgba32float => WgpuTextureFormat::RGBA32Float,
        TextureFormat::rg32float => WgpuTextureFormat::RG32Float,
        TextureFormat::r32float => WgpuTextureFormat::R32Float,
        TextureFormat::rgb10a2unorm => WgpuTextureFormat::RGB10A2Unorm,
        TextureFormat::r11g11b10float => WgpuTextureFormat::RG11B10Ufloat,
        TextureFormat::depth16unorm => WgpuTextureFormat::Depth16Unorm,
        TextureFormat::depth24plusStencil8 => WgpuTextureFormat::Depth24PlusStencil8,
        TextureFormat::depth32float => WgpuTextureFormat::Depth32Float,
        TextureFormat::depth32floatStencil8 => WgpuTextureFormat::Depth32FloatStencil8,
        TextureFormat::bc1unorm => WgpuTextureFormat::BC1RGBAUnorm,
        TextureFormat::bc3unorm => WgpuTextureFormat::BC3RGBAUnorm,
        TextureFormat::bc7unorm => WgpuTextureFormat::BC7RGBAUnorm,
        TextureFormat::etc2rgb8 => WgpuTextureFormat::ETC2RGB8Unorm,
        TextureFormat::etc2rgba8 => WgpuTextureFormat::ETC2RGBA8Unorm,
        TextureFormat::astc4x4 => WgpuTextureFormat::ASTC4x4Unorm,
        TextureFormat::astc6x6 => WgpuTextureFormat::ASTC6x6Unorm,
        TextureFormat::astc8x8 => WgpuTextureFormat::ASTC8x8Unorm,
    }
}

pub(crate) fn oreTypeToWGPUDimension(value: TextureType) -> WgpuTextureDimension {
    match value {
        TextureType::texture2D | TextureType::cube | TextureType::array2D => {
            WgpuTextureDimension::e2D
        }
        TextureType::texture3D => WgpuTextureDimension::e3D,
    }
}

pub(crate) fn oreViewDimToWGPU(value: TextureViewDimension) -> WgpuTextureViewDimension {
    match value {
        TextureViewDimension::texture2D => WgpuTextureViewDimension::e2D,
        TextureViewDimension::cube => WgpuTextureViewDimension::Cube,
        TextureViewDimension::texture3D => WgpuTextureViewDimension::e3D,
        TextureViewDimension::array2D => WgpuTextureViewDimension::e2DArray,
        TextureViewDimension::cubeArray => WgpuTextureViewDimension::CubeArray,
    }
}

pub(crate) fn oreAspectToWGPU(value: TextureAspect) -> WgpuTextureAspect {
    match value {
        TextureAspect::all => WgpuTextureAspect::All,
        TextureAspect::depthOnly => WgpuTextureAspect::DepthOnly,
        TextureAspect::stencilOnly => WgpuTextureAspect::StencilOnly,
    }
}

pub(crate) fn oreViewFormatForAspect(
    texFormat: TextureFormat,
    aspect: TextureAspect,
) -> WgpuTextureFormat {
    if aspect == TextureAspect::depthOnly
        && matches!(
            texFormat,
            TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
        )
    {
        return WgpuTextureFormat::Undefined;
    }
    if aspect == TextureAspect::stencilOnly {
        return WgpuTextureFormat::Undefined;
    }
    oreFormatToWGPU(texFormat)
}

pub(crate) fn oreFilterToWGPU(value: Filter) -> WgpuFilterMode {
    if value == Filter::linear {
        WgpuFilterMode::Linear
    } else {
        WgpuFilterMode::Nearest
    }
}

pub(crate) fn oreMipmapFilterToWGPU(value: Filter) -> WgpuMipmapFilterMode {
    if value == Filter::linear {
        WgpuMipmapFilterMode::Linear
    } else {
        WgpuMipmapFilterMode::Nearest
    }
}

pub(crate) fn oreWrapToWGPU(value: WrapMode) -> WgpuAddressMode {
    match value {
        WrapMode::repeat => WgpuAddressMode::Repeat,
        WrapMode::mirrorRepeat => WgpuAddressMode::MirrorRepeat,
        WrapMode::clampToEdge => WgpuAddressMode::ClampToEdge,
    }
}

pub(crate) fn oreCompareFunctionToWGPU(value: CompareFunction) -> WgpuCompareFunction {
    match value {
        CompareFunction::none => WgpuCompareFunction::Undefined,
        CompareFunction::never => WgpuCompareFunction::Never,
        CompareFunction::less => WgpuCompareFunction::Less,
        CompareFunction::equal => WgpuCompareFunction::Equal,
        CompareFunction::lessEqual => WgpuCompareFunction::LessEqual,
        CompareFunction::greater => WgpuCompareFunction::Greater,
        CompareFunction::notEqual => WgpuCompareFunction::NotEqual,
        CompareFunction::greaterEqual => WgpuCompareFunction::GreaterEqual,
        CompareFunction::always => WgpuCompareFunction::Always,
    }
}

pub(crate) fn oreTopologyToWGPU(value: PrimitiveTopology) -> WgpuPrimitiveTopology {
    match value {
        PrimitiveTopology::pointList => WgpuPrimitiveTopology::PointList,
        PrimitiveTopology::lineList => WgpuPrimitiveTopology::LineList,
        PrimitiveTopology::lineStrip => WgpuPrimitiveTopology::LineStrip,
        PrimitiveTopology::triangleList => WgpuPrimitiveTopology::TriangleList,
        PrimitiveTopology::triangleStrip => WgpuPrimitiveTopology::TriangleStrip,
    }
}

pub(crate) fn oreIndexFormatToWGPU(value: IndexFormat) -> WgpuIndexFormat {
    match value {
        IndexFormat::none | IndexFormat::uint16 => WgpuIndexFormat::Uint16,
        IndexFormat::uint32 => WgpuIndexFormat::Uint32,
    }
}

pub(crate) fn oreCullModeToWGPU(value: CullMode) -> WgpuCullMode {
    match value {
        CullMode::none => WgpuCullMode::None,
        CullMode::front => WgpuCullMode::Front,
        CullMode::back => WgpuCullMode::Back,
    }
}

pub(crate) fn oreWindingToWGPU(value: FaceWinding) -> WgpuFrontFace {
    if value == FaceWinding::counterClockwise {
        WgpuFrontFace::CCW
    } else {
        WgpuFrontFace::CW
    }
}

pub(crate) fn oreBlendFactorToWGPU(value: BlendFactor) -> WgpuBlendFactor {
    match value {
        BlendFactor::zero => WgpuBlendFactor::Zero,
        BlendFactor::one => WgpuBlendFactor::One,
        BlendFactor::srcColor => WgpuBlendFactor::Src,
        BlendFactor::oneMinusSrcColor => WgpuBlendFactor::OneMinusSrc,
        BlendFactor::srcAlpha => WgpuBlendFactor::SrcAlpha,
        BlendFactor::oneMinusSrcAlpha => WgpuBlendFactor::OneMinusSrcAlpha,
        BlendFactor::dstColor => WgpuBlendFactor::Dst,
        BlendFactor::oneMinusDstColor => WgpuBlendFactor::OneMinusDst,
        BlendFactor::dstAlpha => WgpuBlendFactor::DstAlpha,
        BlendFactor::oneMinusDstAlpha => WgpuBlendFactor::OneMinusDstAlpha,
        BlendFactor::srcAlphaSaturated => WgpuBlendFactor::SrcAlphaSaturated,
        BlendFactor::blendColor => WgpuBlendFactor::Constant,
        BlendFactor::oneMinusBlendColor => WgpuBlendFactor::OneMinusConstant,
    }
}

pub(crate) fn oreBlendOpToWGPU(value: BlendOp) -> WgpuBlendOperation {
    match value {
        BlendOp::add => WgpuBlendOperation::Add,
        BlendOp::subtract => WgpuBlendOperation::Subtract,
        BlendOp::reverseSubtract => WgpuBlendOperation::ReverseSubtract,
        BlendOp::min => WgpuBlendOperation::Min,
        BlendOp::max => WgpuBlendOperation::Max,
    }
}

pub(crate) fn oreStencilOpToWGPU(value: StencilOp) -> WgpuStencilOperation {
    match value {
        StencilOp::keep => WgpuStencilOperation::Keep,
        StencilOp::zero => WgpuStencilOperation::Zero,
        StencilOp::replace => WgpuStencilOperation::Replace,
        StencilOp::incrementClamp => WgpuStencilOperation::IncrementClamp,
        StencilOp::decrementClamp => WgpuStencilOperation::DecrementClamp,
        StencilOp::invert => WgpuStencilOperation::Invert,
        StencilOp::incrementWrap => WgpuStencilOperation::IncrementWrap,
        StencilOp::decrementWrap => WgpuStencilOperation::DecrementWrap,
    }
}

pub(crate) fn oreColorWriteMaskToWGPU(mask: ColorWriteMask) -> WgpuColorWriteMask {
    let mut result = WgpuColorWriteMask::None;
    if mask.0 & ColorWriteMask::red.0 != 0 {
        result |= WgpuColorWriteMask::Red;
    }
    if mask.0 & ColorWriteMask::green.0 != 0 {
        result |= WgpuColorWriteMask::Green;
    }
    if mask.0 & ColorWriteMask::blue.0 != 0 {
        result |= WgpuColorWriteMask::Blue;
    }
    if mask.0 & ColorWriteMask::alpha.0 != 0 {
        result |= WgpuColorWriteMask::Alpha;
    }
    result
}

pub(crate) fn oreVertexFormatToWGPU(value: VertexFormat) -> WgpuVertexFormat {
    match value {
        VertexFormat::float1 => WgpuVertexFormat::Float32,
        VertexFormat::float2 => WgpuVertexFormat::Float32x2,
        VertexFormat::float3 => WgpuVertexFormat::Float32x3,
        VertexFormat::float4 => WgpuVertexFormat::Float32x4,
        VertexFormat::uint8x4 => WgpuVertexFormat::Uint8x4,
        VertexFormat::sint8x4 => WgpuVertexFormat::Sint8x4,
        VertexFormat::unorm8x4 => WgpuVertexFormat::Unorm8x4,
        VertexFormat::snorm8x4 => WgpuVertexFormat::Snorm8x4,
        VertexFormat::uint16x2 => WgpuVertexFormat::Uint16x2,
        VertexFormat::sint16x2 => WgpuVertexFormat::Sint16x2,
        VertexFormat::unorm16x2 => WgpuVertexFormat::Unorm16x2,
        VertexFormat::snorm16x2 => WgpuVertexFormat::Snorm16x2,
        VertexFormat::uint16x4 => WgpuVertexFormat::Uint16x4,
        VertexFormat::sint16x4 => WgpuVertexFormat::Sint16x4,
        VertexFormat::float16x2 => WgpuVertexFormat::Float16x2,
        VertexFormat::float16x4 => WgpuVertexFormat::Float16x4,
        VertexFormat::uint32 => WgpuVertexFormat::Uint32,
    }
}

pub(crate) fn oreStepModeToWGPU(value: VertexStepMode) -> WgpuVertexStepMode {
    if value == VertexStepMode::instance {
        WgpuVertexStepMode::Instance
    } else {
        WgpuVertexStepMode::Vertex
    }
}

fn stringView(value: Option<&str>) -> WGPUStringView {
    match value {
        None => WGPUStringView {
            data: std::ptr::null(),
            length: WGPU_STRLEN,
        },
        Some(value) => WGPUStringView {
            data: value.as_ptr().cast(),
            length: value.len(),
        },
    }
}

fn stringViewOrEmpty(value: Option<&str>) -> WGPUStringView {
    static EMPTY: &[u8] = b"\0";
    value.map_or(
        WGPUStringView {
            data: EMPTY.as_ptr().cast(),
            length: WGPU_STRLEN,
        },
        |value| WGPUStringView {
            data: value.as_ptr().cast(),
            length: value.len(),
        },
    )
}

fn managerAndDomain(
    context: &ContextWGPU,
) -> Option<(
    nuxie_ore_metal::gpu_resource::GPUResourceManager,
    nuxie_ore_metal::gpu_resource::ResourceDomain,
)> {
    Some((
        nuxie_ore_metal::context_backend_manager(&context.base)?,
        nuxie_ore_metal::context_backend_domain(&context.base),
    ))
}

pub(crate) fn makeBuffer(
    context: &mut ContextWGPU,
    desc: &BufferDesc<'_>,
) -> Option<AnyResourceHandle> {
    let (manager, domain) = managerAndDomain(context)?;
    let mut buffer = BufferWGPU::new(manager.clone(), desc.size, desc.usage);
    {
        let state = buffer
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *state.m_wgpuQueue = (&*context.m_wgpuQueue).clone();
        *state.m_wgpuDevice = (&*context.m_wgpuDevice).clone();
        state.m_ctx = context;

        let mut usage = WgpuBufferUsage::CopyDst;
        match desc.usage {
            BufferUsage::vertex => usage |= WgpuBufferUsage::Vertex,
            BufferUsage::index => usage |= WgpuBufferUsage::Index,
            BufferUsage::uniform => usage |= WgpuBufferUsage::Uniform,
            BufferUsage::upload => {
                usage |= WgpuBufferUsage::MapWrite | WgpuBufferUsage::CopySrc;
            }
        }
        state.m_wgpuUsage = usage;

        let mut wDesc = WGPUBufferDescriptor::default();
        wDesc.label = stringView(desc.label);
        wDesc.size = u64::from(desc.size);
        wDesc.usage = usage.into();
        wDesc.mappedAtCreation = WGPU_FALSE;
        let native = unsafe { context.m_wgpuDevice.CreateBuffer(&wDesc) };
        if native.Get().is_null() {
            return None;
        }
        state.m_pool.push(Backing {
            buffer: native,
            frameStamp: 0,
        });

        let initialData = match desc.data_prefix() {
            Ok(data) => data,
            Err(_) => {
                context.setLastError("makeBuffer: data span is shorter than desc.size");
                return None;
            }
        };
        if let Some(data) = initialData {
            unsafe {
                context.m_wgpuQueue.WriteBuffer(
                    state.m_pool[0].buffer.Get(),
                    0,
                    data.as_ptr().cast(),
                    desc.size as usize,
                );
            }
            state.m_shadow.extend_from_slice(data);
        }
    }
    Some(ResourceHandle::new_buffer_in_domain(Some(manager), domain, buffer).erase())
}

pub(crate) fn makeTexture(
    context: &mut ContextWGPU,
    desc: &TextureDesc<'_>,
) -> Option<AnyResourceHandle> {
    let (manager, domain) = managerAndDomain(context)?;
    let mut texture = TextureWGPU::new(manager.clone(), desc);
    *texture.m_wgpuQueue = (&*context.m_wgpuQueue).clone();

    let mut usage = WgpuTextureUsage::TextureBinding | WgpuTextureUsage::CopyDst;
    let mut usage = usage.intoBitmask();
    if desc.renderTarget {
        usage |= WgpuTextureUsage::RenderAttachment;
    }
    let mut wDesc = WGPUTextureDescriptor::default();
    wDesc.label = stringView(desc.label);
    wDesc.usage = usage.into();
    wDesc.dimension = oreTypeToWGPUDimension(desc.r#type).into();
    wDesc.size.width = desc.width;
    wDesc.size.height = desc.height;
    wDesc.size.depthOrArrayLayers = desc.depthOrArrayLayers;
    wDesc.format = oreFormatToWGPU(desc.format).into();
    wDesc.mipLevelCount = desc.numMipmaps;
    wDesc.sampleCount = desc.sampleCount;
    *texture.m_wgpuTexture = unsafe { context.m_wgpuDevice.CreateTexture(&wDesc) };
    Some(ResourceHandle::new_texture_in_domain(Some(manager), domain, texture).erase())
}

pub(crate) fn makeTextureView(
    context: &mut ContextWGPU,
    desc: &TextureViewDesc<'_>,
) -> Option<AnyResourceHandle> {
    let textureOwner = desc.texture?.clone();
    let texture = textureOwner.downcast_ref::<TextureWGPU>()?;
    let (manager, domain) = managerAndDomain(context)?;
    let mut view = TextureViewWGPU::new(manager.clone(), textureOwner.clone(), desc);
    let mut wDesc = WGPUTextureViewDescriptor::default();
    wDesc.dimension = oreViewDimToWGPU(desc.dimension).into();
    wDesc.aspect = oreAspectToWGPU(desc.aspect).into();
    wDesc.baseMipLevel = desc.baseMipLevel;
    wDesc.mipLevelCount = desc.mipCount;
    wDesc.baseArrayLayer = desc.baseLayer;
    wDesc.arrayLayerCount = desc.layerCount;
    wDesc.format = oreViewFormatForAspect(texture.base.format(), desc.aspect).into();
    *view.m_wgpuTextureView = unsafe { texture.m_wgpuTexture.CreateView(&wDesc) };
    Some(ResourceHandle::new_in_domain(Some(manager), domain, view).erase())
}

pub(crate) fn makeSampler(
    context: &mut ContextWGPU,
    desc: &SamplerDesc<'_>,
) -> Option<AnyResourceHandle> {
    let (manager, domain) = managerAndDomain(context)?;
    let mut sampler = SamplerWGPU::new();
    let mut wDesc = WGPUSamplerDescriptor::default();
    wDesc.label = stringView(desc.label);
    wDesc.addressModeU = oreWrapToWGPU(desc.wrapU).into();
    wDesc.addressModeV = oreWrapToWGPU(desc.wrapV).into();
    wDesc.addressModeW = oreWrapToWGPU(desc.wrapW).into();
    wDesc.magFilter = oreFilterToWGPU(desc.magFilter).into();
    wDesc.minFilter = oreFilterToWGPU(desc.minFilter).into();
    wDesc.mipmapFilter = oreMipmapFilterToWGPU(desc.mipmapFilter).into();
    wDesc.lodMinClamp = desc.minLod;
    wDesc.lodMaxClamp = desc.maxLod;
    wDesc.compare = oreCompareFunctionToWGPU(desc.compare).into();
    wDesc.maxAnisotropy = desc.maxAnisotropy.max(1) as u16;
    *sampler.m_wgpuSampler = unsafe { context.m_wgpuDevice.CreateSampler(&wDesc) };
    Some(ResourceHandle::new_in_domain(Some(manager), domain, sampler).erase())
}

fn compileWagyuShader(
    device: &Device,
    source: &[u8],
    codeSize: u32,
    language: WGPUWagyuShaderLanguage,
) -> WagyuShaderModule {
    let codeSize = if codeSize > 0 {
        codeSize as usize
    } else {
        source.len()
    };
    let mut wagyuDesc = WGPUWagyuShaderModuleDescriptor {
        chain: WGPUChainedStruct {
            next: std::ptr::null_mut(),
            sType: WGPUSType_WagyuShaderModuleDescriptor,
        },
        codeSize,
        code: source.as_ptr().cast(),
        language,
        compilationHintCount: 0,
        compilationHints: std::ptr::null(),
    };
    let mut descriptor = WGPUShaderModuleDescriptor::default();
    descriptor.nextInChain = (&mut wagyuDesc.chain as *mut WGPUChainedStruct).cast();
    unsafe { device.CreateShaderModule(&descriptor) }
}

fn compileWGSLShader(device: &Device, source: &[u8], codeSize: u32) -> WagyuShaderModule {
    let mut wgslDesc = WGPUShaderSourceWGSL::default();
    wgslDesc.code.data = source.as_ptr().cast();
    wgslDesc.code.length = if codeSize > 0 {
        codeSize as usize
    } else {
        WGPU_STRLEN
    };
    let mut descriptor = WGPUShaderModuleDescriptor::default();
    descriptor.nextInChain = (&mut wgslDesc.chain as *mut WGPUChainedStruct).cast();
    unsafe { device.CreateShaderModule(&descriptor) }
}

pub(crate) fn makeShaderModule(
    context: &mut ContextWGPU,
    desc: &ShaderModuleDesc<'_>,
) -> Option<AnyResourceHandle> {
    let source = desc.code?;
    let codeSize = if desc.codeSize > 0 {
        desc.codeSize().ok()?
    } else {
        source
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(source.len()) as u32
    };
    let language = if desc.language == ShaderLanguage::wgsl {
        WGPUWagyuShaderLanguage_WGSL
    } else if context.m_wgpuBackend == WGPUBackend::OpenGLES {
        WGPUWagyuShaderLanguage_GLSLRAW
    } else {
        WGPUWagyuShaderLanguage_GLSL
    };
    let (manager, domain) = managerAndDomain(context)?;
    let mut module = ShaderModuleWGPU::new();
    *module.m_wgpuShaderModule =
        compileWagyuShader(&context.m_wgpuDevice, source, codeSize, language);
    assert!(
        !module.m_wgpuShaderModule.Get().is_null(),
        "Ore WGPU wagyu shader compilation failed"
    );
    module.applyBindingMapFromDesc(desc);
    Some(ResourceHandle::new_in_domain(Some(manager), domain, module).erase())
}

pub(crate) fn makePipeline(
    context: &mut ContextWGPU,
    desc: &PipelineDesc<'_>,
    mut outError: Option<&mut String>,
) -> Option<AnyResourceHandle> {
    const kMaxBuffers: usize = 8;
    const kMaxAttribs: usize = 32;
    let vertexBufferCount = desc.vertexBufferCount().ok()? as usize;
    let bindGroupLayoutCount = desc.bindGroupLayoutCount().ok()? as usize;
    assert!(vertexBufferCount <= kMaxBuffers);
    assert!(desc.colorCount as usize <= 4);
    assert!(bindGroupLayoutCount <= kMaxBindGroups as usize);
    let (manager, domain) = managerAndDomain(context)?;
    let mut pipeline = PipelineWGPU::new(manager.clone(), desc)?;

    let mut wAttribs: [WGPUVertexAttribute; kMaxAttribs] =
        std::array::from_fn(|_| WGPUVertexAttribute::default());
    let mut wBuffers: [WGPUVertexBufferLayout; kMaxBuffers] =
        std::array::from_fn(|_| WGPUVertexBufferLayout::default());
    let mut attrIdx = 0usize;
    for (bufferIndex, layout) in desc
        .vertexBuffers
        .unwrap_or(&[])
        .iter()
        .take(vertexBufferCount)
        .enumerate()
    {
        let attributeCount = layout.attributeCount().ok()? as usize;
        assert!(attrIdx + attributeCount <= kMaxAttribs);
        wBuffers[bufferIndex].arrayStride = u64::from(layout.stride);
        wBuffers[bufferIndex].stepMode = oreStepModeToWGPU(layout.stepMode).into();
        wBuffers[bufferIndex].attributeCount = attributeCount;
        wBuffers[bufferIndex].attributes = unsafe { wAttribs.as_ptr().add(attrIdx) };
        for attr in layout.attributes.iter().take(attributeCount) {
            let wa = &mut wAttribs[attrIdx];
            *wa = WGPUVertexAttribute::default();
            wa.format = oreVertexFormatToWGPU(attr.format).into();
            wa.offset = u64::from(attr.offset);
            wa.shaderLocation = attr.shaderSlot;
            attrIdx += 1;
        }
    }

    let vertexModuleOwner = desc.vertexModule?;
    let vertexModule = vertexModuleOwner
        .downcast_ref::<ShaderModuleWGPU>()
        .expect("WebGPU pipeline vertex module must be ShaderModuleWGPU");
    // Rust's erased handle does exact concrete downcasts; restore the C++
    // derived-to-base ShaderModule binding-map copy performed by Pipeline.
    *pipeline.m_bindingMap = vertexModule.m_bindingMap.clone();
    let mut vertexState = WGPUVertexState::default();
    vertexState.module = vertexModule.m_wgpuShaderModule.Get();
    vertexState.entryPoint = stringView(desc.vertexEntryPoint);
    vertexState.bufferCount = vertexBufferCount;
    vertexState.buffers = if vertexBufferCount > 0 {
        wBuffers.as_ptr()
    } else {
        std::ptr::null()
    };

    let mut primitiveState = WGPUPrimitiveState::default();
    primitiveState.topology = oreTopologyToWGPU(desc.topology).into();
    primitiveState.stripIndexFormat = if matches!(
        desc.topology,
        PrimitiveTopology::triangleStrip | PrimitiveTopology::lineStrip
    ) {
        oreIndexFormatToWGPU(desc.indexFormat).into()
    } else {
        WGPUIndexFormat_Undefined
    };
    primitiveState.frontFace = oreWindingToWGPU(desc.winding).into();
    primitiveState.cullMode = oreCullModeToWGPU(desc.cullMode).into();

    let mut depthStencilState = WGPUDepthStencilState::default();
    let hasDepth = desc.depthStencil.format != TextureFormat::rgba8unorm;
    if hasDepth {
        depthStencilState.format = oreFormatToWGPU(desc.depthStencil.format).into();
        depthStencilState.depthWriteEnabled = if desc.depthStencil.depthWriteEnabled {
            WGPUOptionalBool_True
        } else {
            WGPUOptionalBool_False
        };
        depthStencilState.depthCompare =
            oreCompareFunctionToWGPU(desc.depthStencil.depthCompare).into();
        depthStencilState.stencilFront.compare =
            oreCompareFunctionToWGPU(desc.stencilFront.compare).into();
        depthStencilState.stencilFront.failOp = oreStencilOpToWGPU(desc.stencilFront.failOp).into();
        depthStencilState.stencilFront.depthFailOp =
            oreStencilOpToWGPU(desc.stencilFront.depthFailOp).into();
        depthStencilState.stencilFront.passOp = oreStencilOpToWGPU(desc.stencilFront.passOp).into();
        depthStencilState.stencilBack.compare =
            oreCompareFunctionToWGPU(desc.stencilBack.compare).into();
        depthStencilState.stencilBack.failOp = oreStencilOpToWGPU(desc.stencilBack.failOp).into();
        depthStencilState.stencilBack.depthFailOp =
            oreStencilOpToWGPU(desc.stencilBack.depthFailOp).into();
        depthStencilState.stencilBack.passOp = oreStencilOpToWGPU(desc.stencilBack.passOp).into();
        depthStencilState.stencilReadMask = u32::from(desc.stencilReadMask);
        depthStencilState.stencilWriteMask = u32::from(desc.stencilWriteMask);
        depthStencilState.depthBias = desc.depthStencil.depthBias;
        depthStencilState.depthBiasSlopeScale = desc.depthStencil.depthBiasSlopeScale;
        depthStencilState.depthBiasClamp = desc.depthStencil.depthBiasClamp;
    }

    let mut multisampleState = WGPUMultisampleState::default();
    multisampleState.count = desc.sampleCount;
    multisampleState.mask = 0xFFFF_FFFF;

    let mut blendStates: [WGPUBlendState; 4] = std::array::from_fn(|_| WGPUBlendState::default());
    let mut colorTargets: [WGPUColorTargetState; 4] =
        std::array::from_fn(|_| WGPUColorTargetState::default());
    for index in 0..desc.colorCount as usize {
        let source = &desc.colorTargets[index];
        let target = &mut colorTargets[index];
        target.format = oreFormatToWGPU(source.format).into();
        target.writeMask = oreColorWriteMaskToWGPU(source.writeMask).into();
        if source.blendEnabled {
            blendStates[index].color.operation = oreBlendOpToWGPU(source.blend.colorOp).into();
            blendStates[index].color.srcFactor = oreBlendFactorToWGPU(source.blend.srcColor).into();
            blendStates[index].color.dstFactor = oreBlendFactorToWGPU(source.blend.dstColor).into();
            blendStates[index].alpha.operation = oreBlendOpToWGPU(source.blend.alphaOp).into();
            blendStates[index].alpha.srcFactor = oreBlendFactorToWGPU(source.blend.srcAlpha).into();
            blendStates[index].alpha.dstFactor = oreBlendFactorToWGPU(source.blend.dstAlpha).into();
            target.blend = &blendStates[index];
        } else {
            target.blend = std::ptr::null();
        }
    }

    let mut fragmentState = WGPUFragmentState::default();
    if let Some(fragmentModuleOwner) = desc.fragmentModule {
        let fragmentModule = fragmentModuleOwner
            .downcast_ref::<ShaderModuleWGPU>()
            .expect("WebGPU pipeline fragment module must be ShaderModuleWGPU");
        fragmentState.module = fragmentModule.m_wgpuShaderModule.Get();
        fragmentState.entryPoint = stringView(desc.fragmentEntryPoint);
        fragmentState.targetCount = desc.colorCount as usize;
        fragmentState.targets = if desc.colorCount > 0 {
            colorTargets.as_ptr()
        } else {
            std::ptr::null()
        };
    }

    let mut error = String::new();
    let layoutsValid = nuxie_ore_metal::mechanical_port::source::renderer::src::ore::ore_bind_group_layout_cpp::validateLayoutsAgainstBindingMap(
        &pipeline.m_bindingMap,
        desc.bindGroupLayouts,
        desc.bindGroupLayoutCount,
        Some(&mut error),
    );
    let colorsValid = nuxie_ore_metal::mechanical_port::source::renderer::src::ore::ore_bind_group_layout_cpp::validateColorRequiresFragment(
        desc.colorCount,
        desc.fragmentModule.is_some(),
        Some(&mut error),
    );
    if !layoutsValid || !colorsValid {
        if let Some(output) = outError.as_deref_mut() {
            *output = error;
        } else {
            context.setLastError(&format!("makePipeline: {error}"));
        }
        return None;
    }

    let mut rawBGLs: [WGPUBindGroupLayout; kMaxBindGroups as usize] =
        [std::ptr::null_mut(); kMaxBindGroups as usize];
    for (index, layout) in desc
        .bindGroupLayouts
        .unwrap_or(&[])
        .iter()
        .take(bindGroupLayoutCount)
        .enumerate()
    {
        if let Some(layout) = layout.and_then(|layout| layout.downcast_ref::<BindGroupLayoutWGPU>())
        {
            rawBGLs[index] = layout.m_wgpuBGL.Get();
        }
    }
    let mut plDesc = WGPUPipelineLayoutDescriptor::default();
    plDesc.label = stringViewOrEmpty(desc.label);
    plDesc.bindGroupLayoutCount = bindGroupLayoutCount;
    plDesc.bindGroupLayouts = if bindGroupLayoutCount > 0 {
        rawBGLs.as_ptr()
    } else {
        std::ptr::null()
    };
    *pipeline.m_wgpuPipelineLayout = unsafe { context.m_wgpuDevice.CreatePipelineLayout(&plDesc) };

    let mut rpDesc = WGPURenderPipelineDescriptor::default();
    rpDesc.label = stringView(desc.label);
    rpDesc.layout = pipeline.m_wgpuPipelineLayout.Get();
    rpDesc.vertex = vertexState;
    rpDesc.primitive = primitiveState;
    rpDesc.depthStencil = if hasDepth {
        &depthStencilState
    } else {
        std::ptr::null()
    };
    rpDesc.multisample = multisampleState;
    rpDesc.fragment = if desc.fragmentModule.is_some() {
        &fragmentState
    } else {
        std::ptr::null()
    };
    *pipeline.m_wgpuDevice = (&*context.m_wgpuDevice).clone();
    *pipeline.m_wgpuPipeline = unsafe { context.m_wgpuDevice.CreateRenderPipeline(&rpDesc) };
    if pipeline.m_wgpuPipeline.Get().is_null() {
        if let Some(output) = outError.as_deref_mut() {
            *output = "CreateRenderPipeline returned null".to_owned();
        }
        return None;
    }
    Some(ResourceHandle::new_in_domain(Some(manager), domain, pipeline).erase())
}

pub(crate) fn makeBindGroupLayout(
    context: &mut ContextWGPU,
    desc: &BindGroupLayoutDesc<'_>,
) -> Option<AnyResourceHandle> {
    if desc.groupIndex >= kMaxBindGroups {
        context.setLastError(&format!(
            "makeBindGroupLayout: groupIndex {} out of range [0, {})",
            desc.groupIndex, kMaxBindGroups
        ));
        return None;
    }
    let entryCount = desc.entryCount().ok()? as usize;
    let (manager, domain) = managerAndDomain(context)?;
    let mut layout = BindGroupLayoutWGPU::new();
    nuxie_ore_metal::install_bind_group_layout_backend_parts(
        &mut layout,
        &context.base,
        desc.groupIndex,
        desc.entries[..entryCount].to_vec(),
    );
    *layout.m_wgpuBGL =
        super::ore_wgpu_layout_decl::buildWGPUBindGroupLayoutFromDesc(&context.m_wgpuDevice, desc);
    if layout.m_wgpuBGL.Get().is_null() {
        context.setLastError(&format!(
            "makeBindGroupLayout: CreateBindGroupLayout returned null (group={})",
            desc.groupIndex
        ));
        return None;
    }
    Some(ResourceHandle::new_in_domain(Some(manager), domain, layout).erase())
}

fn samplerKindsInterchangeable(actual: BindingKind, expected: BindingKind) -> bool {
    matches!(
        actual,
        BindingKind::sampler | BindingKind::comparisonSampler
    ) && matches!(
        expected,
        BindingKind::sampler | BindingKind::comparisonSampler
    )
}

pub(crate) fn makeBindGroup(
    context: &mut ContextWGPU,
    desc: &BindGroupDesc<'_>,
) -> Option<AnyResourceHandle> {
    let layoutOwner = match desc.layout {
        Some(layout) => layout.clone(),
        None => {
            context.setLastError("makeBindGroup: BindGroupDesc::layout is null");
            return None;
        }
    };
    let layout = layoutOwner
        .downcast_ref::<BindGroupLayoutWGPU>()
        .expect("WebGPU bind groups require BindGroupLayoutWGPU");
    if layout.groupIndex() >= kMaxBindGroups {
        context.setLastError(&format!(
            "makeBindGroup: layout->groupIndex {} out of range [0, {})",
            layout.groupIndex(),
            kMaxBindGroups
        ));
        return None;
    }
    let uboCount = desc.uboCount().ok()? as usize;
    let textureCount = desc.textureCount().ok()? as usize;
    let samplerCount = desc.samplerCount().ok()? as usize;
    let dynamicCount = layout
        .entries()
        .iter()
        .filter(|entry| entry.kind == BindingKind::uniformBuffer && entry.hasDynamicOffset)
        .count() as u32;

    let mut retainedBuffers = Vec::new();
    let mut retainedViews = Vec::new();
    let mut retainedSamplers = Vec::new();
    let mut nativeUBOs = Vec::new();
    let mut nativeTextures = Vec::new();
    let mut nativeSamplers = Vec::new();

    let mut checkLayout = |binding: u32, expected: BindingKind| {
        let Some(entry) = layout.findEntry(binding) else {
            context.setLastError(&format!(
                "makeBindGroup: (group={}, binding={}) not declared in BindGroupLayout",
                layout.groupIndex(),
                binding
            ));
            return false;
        };
        if entry.kind != expected && !samplerKindsInterchangeable(entry.kind, expected) {
            context.setLastError(&format!(
                "makeBindGroup: (group={}, binding={}) layout kind mismatch",
                layout.groupIndex(),
                binding
            ));
            return false;
        }
        true
    };

    for ubo in &desc.ubos[..uboCount] {
        if !checkLayout(ubo.slot, BindingKind::uniformBuffer) {
            continue;
        }
        let owner = ubo
            .buffer
            .expect("WebGPU UBO entry requires a buffer")
            .clone();
        let buffer = owner
            .downcast_ref::<BufferWGPU>()
            .expect("WebGPU UBO entry requires BufferWGPU");
        nativeUBOs.push(NativeUBOEntry {
            buffer: std::ptr::NonNull::from(buffer),
            binding: ubo.slot,
            offset: u64::from(ubo.offset),
            size: u64::from(if ubo.size > 0 {
                ubo.size
            } else {
                buffer.base.size()
            }),
        });
        retainedBuffers.push(owner);
    }
    for texture in &desc.textures[..textureCount] {
        if !checkLayout(texture.slot, BindingKind::sampledTexture) {
            continue;
        }
        let owner = texture
            .view
            .expect("WebGPU texture entry requires a view")
            .clone();
        let view = owner
            .downcast_ref::<TextureViewWGPU>()
            .expect("WebGPU texture entry requires TextureViewWGPU");
        nativeTextures.push(NativeTexEntry {
            binding: texture.slot,
            view: (&*view.m_wgpuTextureView).clone(),
        });
        retainedViews.push(owner);
    }
    for samplerEntry in &desc.samplers[..samplerCount] {
        if !checkLayout(samplerEntry.slot, BindingKind::sampler) {
            continue;
        }
        let owner = samplerEntry
            .sampler
            .expect("WebGPU sampler entry requires a sampler")
            .clone();
        let samplerResource = owner
            .downcast_ref::<SamplerWGPU>()
            .expect("WebGPU sampler entry requires SamplerWGPU");
        nativeSamplers.push(NativeSampEntry {
            binding: samplerEntry.slot,
            sampler: (&*samplerResource.m_wgpuSampler).clone(),
        });
        retainedSamplers.push(owner);
    }

    let (manager, domain) = managerAndDomain(context)?;
    let nativeLayout = (&*layout.m_wgpuBGL).clone();
    let mut group = BindGroupWGPU::new(manager.clone());
    nuxie_ore_metal::install_bind_group_backend_parts(
        &mut group,
        dynamicCount,
        Some(layoutOwner),
        retainedBuffers,
        retainedViews,
        retainedSamplers,
    );
    group.m_ctx = context;
    *group.m_uboEntries = nativeUBOs;
    *group.m_texEntries = nativeTextures;
    *group.m_sampEntries = nativeSamplers;
    *group.m_wgpuBGL = nativeLayout;
    *group.m_label = desc.label.unwrap_or("").to_owned();
    if group.resolveBindGroup().Get().is_null() {
        return None;
    }
    Some(ResourceHandle::new_in_domain(Some(manager), domain, group).erase())
}

pub(crate) fn beginRenderPass(
    context: &mut ContextWGPU,
    desc: &RenderPassDesc<'_>,
    _outError: Option<&mut String>,
) -> Option<Box<dyn nuxie_ore_metal::render_pass::RenderPassApi>> {
    context.base.finishActiveRenderPass();
    assert!(
        !context.m_wgpuCommandEncoder.Get().is_null(),
        "beginFrame must be called before beginRenderPass"
    );
    assert!(desc.colorCount <= 4);

    let pass = RenderPassWGPU::new(context);
    let mut state = pass.inner.borrowState();

    let mut colorFormats = [TextureFormat::r8unorm; 4];
    let mut sampleCount = 1;
    let mut colorAttachments: [WGPURenderPassColorAttachment; 4] =
        std::array::from_fn(|_| WGPURenderPassColorAttachment::default());
    for index in 0..desc.colorCount as usize {
        let source = &desc.colorAttachments[index];
        let target = &mut colorAttachments[index];
        if let Some(view) = source
            .view
            .and_then(|owner| owner.downcast_ref::<TextureViewWGPU>())
        {
            target.view = view.m_wgpuTextureView.Get();
            if let Some(texture) = view.texture().downcast_ref::<TextureWGPU>() {
                colorFormats[index] = texture.base.format();
                sampleCount = texture.base.sampleCount();
            }
        }
        if let Some(resolveTarget) = source
            .resolveTarget
            .and_then(|owner| owner.downcast_ref::<TextureViewWGPU>())
        {
            target.resolveTarget = resolveTarget.m_wgpuTextureView.Get();
        }
        target.loadOp = if source.loadOp == LoadOp::clear {
            WGPULoadOp_Clear
        } else {
            WGPULoadOp_Load
        };
        target.storeOp = if source.storeOp == StoreOp::store {
            WGPUStoreOp_Store
        } else {
            WGPUStoreOp_Discard
        };
        target.clearValue.r = f64::from(source.clearColor.r);
        target.clearValue.g = f64::from(source.clearColor.g);
        target.clearValue.b = f64::from(source.clearColor.b);
        target.clearValue.a = f64::from(source.clearColor.a);
    }

    let mut depthStencilAttachment = WGPURenderPassDepthStencilAttachment::default();
    let mut depthFormat = TextureFormat::r8unorm;
    let mut hasDepth = false;
    if let Some(depthViewOwner) = desc.depthStencil.view {
        let source = &desc.depthStencil;
        let view = depthViewOwner
            .downcast_ref::<TextureViewWGPU>()
            .expect("ContextWGPU requires TextureViewWGPU depth attachments");
        let texture = view
            .texture()
            .downcast_ref::<TextureWGPU>()
            .expect("TextureViewWGPU retains TextureWGPU");
        depthFormat = texture.base.format();
        hasDepth = true;
        if desc.colorCount == 0 {
            sampleCount = texture.base.sampleCount();
        }
        depthStencilAttachment.view = view.m_wgpuTextureView.Get();
        depthStencilAttachment.depthLoadOp = if source.depthLoadOp == LoadOp::clear {
            WGPULoadOp_Clear
        } else {
            WGPULoadOp_Load
        };
        depthStencilAttachment.depthStoreOp = if source.depthStoreOp == StoreOp::store {
            WGPUStoreOp_Store
        } else {
            WGPUStoreOp_Discard
        };
        depthStencilAttachment.depthClearValue = source.depthClearValue;
        let hasStencil = matches!(
            depthFormat,
            TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
        );
        if hasStencil {
            depthStencilAttachment.stencilLoadOp = if source.stencilLoadOp == LoadOp::clear {
                WGPULoadOp_Clear
            } else {
                WGPULoadOp_Load
            };
            depthStencilAttachment.stencilStoreOp = if source.stencilStoreOp == StoreOp::store {
                WGPUStoreOp_Store
            } else {
                WGPUStoreOp_Discard
            };
            depthStencilAttachment.stencilClearValue = source.stencilClearValue;
        } else {
            depthStencilAttachment.stencilReadOnly = WGPU_TRUE;
        }
    }
    nuxie_ore_metal::render_pass_install_attachment_metadata(
        &mut state.base,
        colorFormats,
        desc.colorCount,
        depthFormat,
        hasDepth,
        sampleCount,
    );

    let mut passDesc = WGPURenderPassDescriptor::default();
    passDesc.label = stringView(desc.label);
    passDesc.colorAttachmentCount = desc.colorCount as usize;
    passDesc.colorAttachments = colorAttachments.as_ptr();
    passDesc.depthStencilAttachment = if hasDepth {
        &depthStencilAttachment
    } else {
        std::ptr::null()
    };
    *state.m_wgpuPassEncoder = unsafe { context.m_wgpuCommandEncoder.BeginRenderPass(&passDesc) };
    drop(state);
    Some(Box::new(pass))
}

pub(crate) unsafe fn wrapCanvasTexture(
    context: &mut ContextWGPU,
    canvas: *mut core::ffi::c_void,
) -> Option<AnyResourceHandle> {
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;

    let canvas = unsafe { canvas.cast::<RenderCanvas>().as_mut() }?;
    let target = unsafe { canvas.renderTarget().cast::<RenderTargetWebGPU>().as_mut() }?;
    let format = match target.framebufferFormat() {
        WgpuTextureFormat::BGRA8Unorm => TextureFormat::bgra8unorm,
        WgpuTextureFormat::RGBA16Float => TextureFormat::rgba16float,
        WgpuTextureFormat::RGB10A2Unorm => TextureFormat::rgb10a2unorm,
        _ => TextureFormat::rgba8unorm,
    };
    let textureDesc = TextureDesc {
        width: canvas.width(),
        height: canvas.height(),
        format,
        r#type: TextureType::texture2D,
        renderTarget: true,
        numMipmaps: 1,
        sampleCount: 1,
        ..TextureDesc::default()
    };
    let (manager, domain) = managerAndDomain(context)?;
    let mut texture = TextureWGPU::new(manager.clone(), &textureDesc);
    *texture.m_wgpuTexture = target.targetTexture();
    let textureOwner =
        ResourceHandle::new_texture_in_domain(Some(manager.clone()), domain.clone(), texture)
            .erase();
    let viewDesc = TextureViewDesc {
        texture: Some(&textureOwner),
        dimension: TextureViewDimension::texture2D,
        aspect: TextureAspect::all,
        baseMipLevel: 0,
        mipCount: 1,
        baseLayer: 0,
        layerCount: 1,
    };
    let mut view = TextureViewWGPU::new(manager.clone(), textureOwner.clone(), &viewDesc);
    *view.m_wgpuTextureView = target.targetTextureView();
    Some(ResourceHandle::new_in_domain(Some(manager), domain, view).erase())
}

pub(crate) unsafe fn wrapRiveTexture(
    context: &mut ContextWGPU,
    gpuTexture: *mut core::ffi::c_void,
    width: u32,
    height: u32,
) -> Option<AnyResourceHandle> {
    use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture as RiveTexture;

    let gpuTexture = unsafe { gpuTexture.cast::<RiveTexture>().as_ref() }?;
    let rawTexture: super::webgpu_decl::WGPUTexture = gpuTexture.nativeHandle().cast();
    if rawTexture.is_null() {
        return None;
    }
    let nativeTexture = unsafe { super::webgpu_cpp_decl::Texture::FromBorrowed(rawTexture) };
    let textureDesc = TextureDesc {
        width,
        height,
        format: TextureFormat::rgba8unorm,
        r#type: TextureType::texture2D,
        renderTarget: false,
        numMipmaps: 1,
        sampleCount: 1,
        ..TextureDesc::default()
    };
    let (manager, domain) = managerAndDomain(context)?;
    let mut texture = TextureWGPU::new(manager.clone(), &textureDesc);
    *texture.m_wgpuTexture = nativeTexture;
    let textureOwner =
        ResourceHandle::new_texture_in_domain(Some(manager.clone()), domain.clone(), texture)
            .erase();
    let viewDesc = TextureViewDesc {
        texture: Some(&textureOwner),
        dimension: TextureViewDimension::texture2D,
        aspect: TextureAspect::all,
        baseMipLevel: 0,
        mipCount: 1,
        baseLayer: 0,
        layerCount: 1,
    };
    let texture = textureOwner.downcast_ref::<TextureWGPU>()?;
    let mut nativeViewDesc = WGPUTextureViewDescriptor::default();
    nativeViewDesc.format = WgpuTextureFormat::RGBA8Unorm.into();
    nativeViewDesc.dimension = WgpuTextureViewDimension::e2D.into();
    nativeViewDesc.baseMipLevel = 0;
    nativeViewDesc.mipLevelCount = 1;
    nativeViewDesc.baseArrayLayer = 0;
    nativeViewDesc.arrayLayerCount = 1;
    let nativeView = unsafe { texture.m_wgpuTexture.CreateView(&nativeViewDesc) };
    let mut view = TextureViewWGPU::new(manager.clone(), textureOwner.clone(), &viewDesc);
    *view.m_wgpuTextureView = nativeView;
    Some(ResourceHandle::new_in_domain(Some(manager), domain, view).erase())
}

fn webGPUFeatures() -> Features {
    let mut f = Features::default();
    f.colorBufferFloat = true;
    f.perTargetBlend = true;
    f.perTargetWriteMask = true;
    f.textureViewSampling = true;
    f.drawBaseInstance = true;
    f.depthBiasClamp = true;
    f.anisotropicFiltering = false;
    f.texture3D = true;
    f.textureArrays = true;
    f.computeShaders = true;
    f.storageBuffers = true;
    f.bc = false;
    f.etc2 = true;
    f.astc = false;
    f.maxColorAttachments = 4;
    f.maxTextureSize2D = 8192;
    f.maxTextureSizeCube = 8192;
    f.maxTextureSize3D = 2048;
    f.maxUniformBufferSize = 65536;
    f.maxVertexAttributes = 16;
    f.maxSamplers = 16;
    f
}

pub(crate) fn make(
    device: Device,
    queue: Queue,
    backendType: WagyuBackendType,
) -> Option<Box<ContextWGPU>> {
    let mut ctx = Box::new(ContextWGPU::new_base(webGPUFeatures()));
    *ctx.m_wgpuDevice = device;
    *ctx.m_wgpuQueue = queue;
    ctx.m_wgpuBackend = if backendType == WagyuBackendType::OpenGLES {
        WGPUBackend::OpenGLES
    } else {
        WGPUBackend::Vulkan
    };
    Some(ctx)
}

pub(crate) fn destroy(context: &mut ContextWGPU) {
    // The source destructor body is empty. Rust's manager sidecar only drains
    // the source-equivalent deferred resource queue before member teardown.
    context.m_managerOwner.shutdown();
}

pub(crate) fn beginFrame(context: &mut ContextWGPU, desc: &FrameDescriptor) {
    let mut encoder = desc
        .externalCommandBuffer
        .expect("ContextWGPU::beginFrame requires an external command encoder")
        .cast::<CommandEncoder>();
    *context.m_wgpuCommandEncoder = unsafe { std::mem::take(encoder.as_mut()) };
    context.m_frameSerial = context.m_frameSerial.wrapping_add(1);
}

pub(crate) fn beginFrameExternal(context: &mut ContextWGPU, externalEncoder: CommandEncoder) {
    *context.m_wgpuCommandEncoder = externalEncoder;
    context.m_frameSerial = context.m_frameSerial.wrapping_add(1);
}

pub(crate) fn waitForGPU(_context: &mut ContextWGPU) {}

pub(crate) fn endFrame(context: &mut ContextWGPU) {
    assert!(!context.m_wgpuCommandEncoder.Get().is_null());
    *context.m_wgpuCommandEncoder = CommandEncoder::default();
}

impl ContextApi for ContextWGPU {
    fn features(&self) -> Features {
        self.base.features()
    }
    fn lastError(&self) -> String {
        self.base.lastError()
    }
    fn activeRenderPass(&self) -> Option<RcWeak<dyn ActiveRenderPass>> {
        self.base.activeRenderPass()
    }
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.base.setActiveRenderPass(pass)
    }
    fn finishActiveRenderPass(&self) {
        self.base.finishActiveRenderPass()
    }
    fn clearLastError(&self) {
        self.base.clearLastError()
    }
    fn setLastError(&self, message: &str) {
        self.base.setLastError(message)
    }
    fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        makeBuffer(self, desc)
    }
    fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        makeTexture(self, desc)
    }
    fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        makeTextureView(self, desc)
    }
    fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        makeSampler(self, desc)
    }
    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        makeShaderModule(self, desc)
    }
    fn makeBindGroupLayout(&mut self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        makeBindGroupLayout(self, desc)
    }
    fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        makePipeline(self, desc, outError)
    }
    fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        makeBindGroup(self, desc)
    }
    fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        beginRenderPass(self, desc, outError)
    }
    fn beginFrame(&mut self, descriptor: &FrameDescriptor) {
        beginFrame(self, descriptor)
    }
    fn endFrame(&mut self) {
        endFrame(self)
    }
    fn waitForGPU(&mut self) {
        waitForGPU(self)
    }
    unsafe fn wrapCanvasTexture(
        &mut self,
        canvas: *mut core::ffi::c_void,
    ) -> Option<AnyResourceHandle> {
        unsafe { wrapCanvasTexture(self, canvas) }
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        texture: *mut core::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        unsafe { wrapRiveTexture(self, texture, width, height) }
    }
    fn shaderTarget(&self) -> ShaderTarget {
        ShaderTarget::wgsl
    }
}

pub(crate) const SOURCE_CONVERSION_HELPER_COUNT: usize = 20;
pub(crate) const SOURCE_CONTEXT_METHOD_DEFINITION_COUNT: usize = 16;
pub(crate) const SOURCE_FEATURE_ASSIGNMENT_COUNT: usize = 21;
const _: [(); 47000] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_source_byte_and_feature_denominators_are_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 1268);
        assert_eq!(SOURCE_CONVERSION_HELPER_COUNT, 20);
        assert_eq!(SOURCE_CONTEXT_METHOD_DEFINITION_COUNT, 16);
        assert_eq!(SOURCE_FEATURE_ASSIGNMENT_COUNT, 21);
        let features = webGPUFeatures();
        assert!(features.colorBufferFloat);
        assert!(features.etc2);
        assert!(!features.bc);
        assert!(!features.astc);
        assert_eq!(features.maxTextureSize2D, 8192);
        assert_eq!(features.maxUniformBufferSize, 65536);
    }
}
