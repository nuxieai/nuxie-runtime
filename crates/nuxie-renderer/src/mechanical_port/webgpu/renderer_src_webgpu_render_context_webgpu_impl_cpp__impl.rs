//! Mechanical implementation translation of
//! `renderer/src/webgpu/render_context_webgpu_impl.cpp`.
//!
//! This file is completed in source order as one atomic SCC with the four Ore
//! WebGPU owners that call into it.

#![allow(non_snake_case, non_upper_case_globals)]

use super::render_context_webgpu_decl::{
    Capabilities, ContextOptions, DrawPipelineLayout, PixelLocalStorageType,
    RenderContextWebGPUImpl, RenderTargetWebGPU, TextureWebGPUImpl,
};
use super::webgpu_cpp_decl::{
    Adapter, BackendType, BindGroup, BindGroupLayout, Buffer, CommandEncoder, Device, FeatureName,
    Queue, Sampler, ShaderModule, Texture as WagyuTexture, TextureFormat, TextureUsage,
    TextureView,
};
use super::webgpu_decl::{
    WGPUBindGroupDescriptor, WGPUBindGroupEntry, WGPUBindGroupLayout,
    WGPUBindGroupLayoutDescriptor, WGPUBindGroupLayoutEntry, WGPUBufferDescriptor,
    WGPUBufferBindingType_ReadOnlyStorage, WGPUBufferBindingType_Storage,
    WGPUBufferBindingType_Uniform, WGPUCompatibilityModeLimits, WGPUExtent3D, WGPULimits,
    WGPUOrigin3D, WGPUPipelineLayoutDescriptor, WGPUSamplerBindingType_Filtering,
    WGPUShaderModuleDescriptor, WGPUShaderSourceWGSL,
    WGPUShaderStage_Fragment, WGPUShaderStage_Vertex, WGPUStringView,
    WGPUBlendState, WGPUColorTargetState, WGPUFragmentState, WGPURenderPipelineDescriptor,
    WGPUSamplerDescriptor, WGPUVertexAttribute, WGPUVertexBufferLayout,
    WGPUTexelCopyBufferLayout, WGPUTexelCopyTextureInfo, WGPUTextureDescriptor,
    WGPUTextureSampleType_Float,
    WGPUTextureSampleType_Uint, WGPUTextureSampleType_UnfilterableFloat,
    WGPUTextureViewDimension_2D, WGPU_LIMIT_U32_UNDEFINED, WGPU_STRLEN,
};
use super::webgpu_wagyu_decl::{
    wgpuWagyuAdapterGetBackend, wgpuWagyuAdapterGetExtensions, wgpuWagyuDeviceGetExtensions,
    wgpuWagyuStringArrayFreeMembers, WGPUFeatureName_WagyuBlendEquationAdvancedCoherent,
    WGPUSType_WagyuInputTextureBindingLayout, WGPUWagyuInputTextureBindingLayout,
    WGPUTextureUsage_WagyuInputAttachment, WGPUTextureUsage_WagyuMSAAResolveSource,
    WGPUTextureUsage_WagyuTransientAttachment, WGPUWagyuStringArray,
};
use crate::mechanical_port::webgl2::load_store_actions_ext_decl::LoadStoreActionsEXT;
use crate::mechanical_port::webgl2::load_store_actions_ext_impl::{
    BuildLoadActionsEXT, BuildLoadStoreEXTGLSL,
};
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::LiteRttiTypeId;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{static_rcp_cast, RefCntTarget};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferContract, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture as RiveTexture;
use crate::mechanical_port::source::renderer::include::rive::renderer::buffer_ring_hpp::{
    BufferRing, BufferRingContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    kBufferRingSize, StorageBufferStructure, INTERLOCK_MODE_COUNT,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::{
    RenderContextHelperBackendContract, RenderContextHelperBufferFactoryContract,
    RenderContextHelperImpl, RenderContextHelperImplAccess,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    RenderContext, RenderContextContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImpl;
use crate::mechanical_port::source::renderer::src::gpu_cpp::{
    StorageTextureBufferSize, StorageTextureSize,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::{
    RenderTarget, IAABB,
};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_webgpu_render_context_webgpu_impl.cpp");

pub(crate) const RIVE_FRONT_FACE: super::webgpu_cpp_decl::FrontFace =
    super::webgpu_cpp_decl::FrontFace::CW;
pub(crate) const MSAA_SAMPLE_COUNT: u32 = 4;
const PER_FLUSH_BINDINGS_SET: usize = 0;
const PER_DRAW_BINDINGS_SET: usize = 1;
const PLS_TEXTURE_BINDINGS_SET: usize = 2;
const WEBGPU_SAMPLER_BINDINGS_SET: usize = 3;
const WEBGPU_BINDINGS_SET_COUNT: usize = 4;
const FLUSH_UNIFORM_BUFFER_IDX: u32 = 0;
const PATH_BUFFER_IDX: u32 = 2;
const PAINT_BUFFER_IDX: u32 = 3;
const PAINT_AUX_BUFFER_IDX: u32 = 4;
const CONTOUR_BUFFER_IDX: u32 = 5;
const TESS_VERTEX_TEXTURE_IDX: u32 = 7;
const GRAD_TEXTURE_IDX: u32 = 8;
const GAUSSIAN_INTEGRAL_TEXTURE_IDX: u32 = 9;
const FEATHER_ATLAS_TEXTURE_IDX: u32 = 10;
const IMAGE_TEXTURE_IDX: u32 = 11;
const DST_COLOR_TEXTURE_IDX: u32 = 12;
const WEBGPU_IMAGE_SAMPLER_IDX: u32 = 13;
const COLOR_PLANE_IDX: u32 = 0;
const CLIP_PLANE_IDX: u32 = 1;
const COVERAGE_PLANE_IDX: u32 = 3;

fn stringView(value: &str) -> WGPUStringView {
    WGPUStringView {
        data: value.as_ptr().cast(),
        length: value.len(),
    }
}

fn compileShaderModuleWGSL(device: &Device, source: &str, label: &str) -> ShaderModule {
    let mut shaderSource = WGPUShaderSourceWGSL::default();
    shaderSource.code = stringView(source);
    let mut descriptor = WGPUShaderModuleDescriptor::default();
    descriptor.nextInChain = &mut shaderSource.chain;
    descriptor.label = stringView(label);
    unsafe { device.CreateShaderModule(&descriptor) }
}

fn compileShaderModuleWagyu(
    device: &Device,
    source: &str,
    language: super::webgpu_wagyu_decl::WGPUWagyuShaderLanguage,
) -> ShaderModule {
    let mut wagyuDescriptor = super::webgpu_wagyu_decl::WGPUWagyuShaderModuleDescriptor {
        chain: super::webgpu_decl::WGPUChainedStruct {
            next: std::ptr::null_mut(),
            sType: super::webgpu_wagyu_decl::WGPUSType_WagyuShaderModuleDescriptor,
        },
        codeSize: source.len(),
        code: source.as_ptr().cast(),
        language,
        compilationHintCount: 0,
        compilationHints: std::ptr::null(),
    };
    let mut descriptor = WGPUShaderModuleDescriptor::default();
    descriptor.nextInChain =
        (&mut wagyuDescriptor.chain as *mut super::webgpu_decl::WGPUChainedStruct).cast();
    unsafe { device.CreateShaderModule(&descriptor) }
}

fn compileShaderModuleWagyuRaw(device: &Device, source: &str) -> ShaderModule {
    compileShaderModuleWagyu(
        device,
        source,
        super::webgpu_wagyu_decl::WGPUWagyuShaderLanguage_GLSLRAW,
    )
}

// Exact identifiers emitted by the pinned shader minifier. The accompanying
// `*.exports.h` snapshots are frozen beside the minified GLSL and tests below
// pin these values back to those generated outputs.
const GLSL_VERTEX: &str = "DB";
const GLSL_FRAGMENT: &str = "GB";
const GLSL_POST_INVERT_Y: &str = "RC";
const GLSL_DISABLE_SHADER_STORAGE_BUFFERS: &str = "JF";
const GLSL_DRAW_PATH: &str = "ID";
const GLSL_ENABLE_FEATHER: &str = "HB";
const GLSL_ENABLE_INSTANCE_INDEX: &str = "NE";
const GLSL_BASE_INSTANCE_UNIFORM_NAME: &str = "YD";
const GLSL_ATLAS_FEATHERED_FILL: &str = "NC";
const GLSL_ATLAS_FEATHERED_STROKE: &str = "TC";
const GLSL_CLEAR_COLOR: &str = "QE";
const GLSL_LOAD_COLOR: &str = "SE";
const GLSL_STORE_COLOR: &str = "ZD";
const GLSL_CLEAR_COVERAGE: &str = "AE";
const GLSL_CLEAR_CLIP: &str = "QF";
const GLSL_ENABLE_CLIPPING: &str = "I";
const GLSL_ENABLE_CLIP_RECT: &str = "BB";
const GLSL_ENABLE_ADVANCED_BLEND: &str = "AB";
const GLSL_ENABLE_EVEN_ODD: &str = "WC";
const GLSL_ENABLE_NESTED_CLIPPING: &str = "YC";
const GLSL_ENABLE_HSL_BLEND_MODES: &str = "FC";
const GLSL_ENABLE_DITHER: &str = "LB";
const GLSL_TARGET_SPIRV: &str = "DC";
const GLSL_PLS_IMPL_EXT_NATIVE: &str = "LF";
const GLSL_PLS_IMPL_NONE: &str = "NF";
const GLSL_PLS_IMPL_SUBPASS_LOAD: &str = "MF";
const GLSL_DRAW_INTERIOR_TRIANGLES: &str = "EB";
const GLSL_FEATHER_ATLAS_BLIT: &str = "FB";
const GLSL_DRAW_IMAGE: &str = "HE";
const GLSL_DRAW_IMAGE_RECT: &str = "JD";
const GLSL_DRAW_IMAGE_MESH: &str = "OB";
const GLSL_FIXED_FUNCTION_COLOR_OUTPUT: &str = "Q";
const GLSL_CLOCKWISE_FILL: &str = "BE";
const GLSL_BORROWED_COVERAGE_PASS: &str = "EC";
const GLSL_OPTIONALLY_FLAT: &str = "MB";
const BASE_INSTANCE_UNIFORM_NAME: &str = "nrdp_BaseInstance";

const GLSL_GLSL: &str = include_str!("source/generated_glsl/glsl.minified.glsl");
const GLSL_CONSTANTS: &str = include_str!("source/generated_glsl/constants.minified.glsl");
const GLSL_FLUSH_UNIFORMS: &str =
    include_str!("source/generated_glsl/flush_uniforms.minified.glsl");
const GLSL_COMMON: &str = include_str!("source/generated_glsl/common.minified.glsl");
const GLSL_COLOR_RAMP: &str = include_str!("source/generated_glsl/color_ramp.minified.glsl");
const GLSL_BEZIER_UTILS: &str = include_str!("source/generated_glsl/bezier_utils.minified.glsl");
const GLSL_TESSELLATE: &str = include_str!("source/generated_glsl/tessellate.minified.glsl");
const GLSL_RENDER_ATLAS: &str = include_str!("source/generated_glsl/render_atlas.minified.glsl");
const GLSL_PLS_LOAD_STORE_EXT: &str =
    include_str!("source/generated_glsl/pls_load_store_ext.minified.glsl");
const GLSL_ADVANCED_BLEND: &str =
    include_str!("source/generated_glsl/advanced_blend.minified.glsl");
const GLSL_DRAW_PATH_COMMON: &str =
    include_str!("source/generated_glsl/draw_path_common.minified.glsl");
const GLSL_DRAW_PATH_VERT: &str = include_str!("source/generated_glsl/draw_path.minified.vert");
const GLSL_DRAW_RASTER_ORDER_PATH_FRAG: &str =
    include_str!("source/generated_glsl/draw_raster_order_path.minified.frag");
const GLSL_DRAW_CLOCKWISE_PATH_FRAG: &str =
    include_str!("source/generated_glsl/draw_clockwise_path.minified.frag");
const GLSL_DRAW_CLOCKWISE_CLIP_FRAG: &str =
    include_str!("source/generated_glsl/draw_clockwise_clip.minified.frag");
const GLSL_DRAW_IMAGE_MESH_VERT: &str =
    include_str!("source/generated_glsl/draw_image_mesh.minified.vert");
const GLSL_DRAW_MESH_FRAG: &str = include_str!("source/generated_glsl/draw_mesh.minified.frag");

fn appendGlslParts(output: &mut String, parts: &[&str]) {
    for part in parts {
        output.push_str(part);
        output.push('\n');
    }
}

fn loadStoreEXTPipelineKey(actions: LoadStoreActionsEXT, format: TextureFormat) -> u32 {
    const ACTION_BITS: u32 = 5;
    assert_eq!(format.0 << ACTION_BITS >> ACTION_BITS, format.0);
    assert!(actions.0 < 1 << ACTION_BITS);
    format.0 << ACTION_BITS | actions.0
}

const SCRATCH_COLOR_PLANE_IDX: usize = 2;
const PLS_PLANE_COUNT: usize = 4;
const IMAGE_FIRST_ATTRIB_IDX: u32 = 2;
const IMAGE_VIEW_MATRIX_ATTRIB_IDX: u32 = 2;
const IMAGE_CLIP_RECT_INVERSE_MATRIX_ATTRIB_IDX: u32 = 3;
const IMAGE_TRANSLATES_ATTRIB_IDX: u32 = 4;
const IMAGE_PACKED_ATTRIBS_IDX: u32 = 5;
const IMAGE_ATTRIB_COUNT: usize = 4;
const SPECIALIZATION_COUNT: usize = 14;
const SPECIALIZATION_IDS: [&str; SPECIALIZATION_COUNT] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13",
];

fn appendImageDrawInstanceAttribs(attributes: &mut Vec<WGPUVertexAttribute>) {
    for (format, location) in [
        (
            super::webgpu_cpp_decl::VertexFormat::Float32x4,
            IMAGE_VIEW_MATRIX_ATTRIB_IDX,
        ),
        (
            super::webgpu_cpp_decl::VertexFormat::Float32x4,
            IMAGE_CLIP_RECT_INVERSE_MATRIX_ATTRIB_IDX,
        ),
        (
            super::webgpu_cpp_decl::VertexFormat::Float32x4,
            IMAGE_TRANSLATES_ATTRIB_IDX,
        ),
        (
            super::webgpu_cpp_decl::VertexFormat::Uint32x4,
            IMAGE_PACKED_ATTRIBS_IDX,
        ),
    ] {
        let mut attribute = WGPUVertexAttribute::default();
        attribute.format = format.into();
        attribute.offset =
            u64::from(location - IMAGE_FIRST_ATTRIB_IDX) * std::mem::size_of::<u32>() as u64 * 4;
        attribute.shaderLocation = location;
        attributes.push(attribute);
    }
}

fn wgpuCullMode(
    face: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::CullFace,
) -> super::webgpu_decl::WGPUCullMode {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::CullFace;
    match face {
        CullFace::none => super::webgpu_cpp_decl::CullMode::None.into(),
        CullFace::clockwise => super::webgpu_cpp_decl::CullMode::Front.into(),
        CullFace::counterclockwise => super::webgpu_cpp_decl::CullMode::Back.into(),
    }
}

fn wgpuCompareFunction(
    operation: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::StencilCompareOp,
) -> super::webgpu_decl::WGPUCompareFunction {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::StencilCompareOp;
    match operation {
        StencilCompareOp::less => super::webgpu_cpp_decl::CompareFunction::Less.into(),
        StencilCompareOp::equal => super::webgpu_cpp_decl::CompareFunction::Equal.into(),
        StencilCompareOp::lessOrEqual => super::webgpu_cpp_decl::CompareFunction::LessEqual.into(),
        StencilCompareOp::notEqual => super::webgpu_cpp_decl::CompareFunction::NotEqual.into(),
        StencilCompareOp::always => super::webgpu_cpp_decl::CompareFunction::Always.into(),
    }
}

fn wgpuStencilOperation(
    operation: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::StencilOp,
) -> super::webgpu_decl::WGPUStencilOperation {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::StencilOp;
    match operation {
        StencilOp::keep => super::webgpu_cpp_decl::StencilOperation::Keep.into(),
        StencilOp::replace => super::webgpu_cpp_decl::StencilOperation::Replace.into(),
        StencilOp::zero => super::webgpu_cpp_decl::StencilOperation::Zero.into(),
        StencilOp::decrClamp => super::webgpu_cpp_decl::StencilOperation::DecrementClamp.into(),
        StencilOp::incrWrap => super::webgpu_cpp_decl::StencilOperation::IncrementWrap.into(),
        StencilOp::decrWrap => super::webgpu_cpp_decl::StencilOperation::DecrementWrap.into(),
    }
}

fn wgpuStencilFaceState(
    face: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::StencilFaceOps,
) -> super::webgpu_decl::WGPUStencilFaceState {
    super::webgpu_decl::WGPUStencilFaceState {
        compare: wgpuCompareFunction(face.compareOp),
        failOp: wgpuStencilOperation(face.stencilFailOp),
        depthFailOp: wgpuStencilOperation(face.depthFailOp),
        passOp: wgpuStencilOperation(face.depthStencilPassOp),
    }
}

fn disabledStencilFaceState() -> super::webgpu_decl::WGPUStencilFaceState {
    super::webgpu_decl::WGPUStencilFaceState {
        compare: super::webgpu_cpp_decl::CompareFunction::Always.into(),
        failOp: super::webgpu_cpp_decl::StencilOperation::Keep.into(),
        depthFailOp: super::webgpu_cpp_decl::StencilOperation::Keep.into(),
        passOp: super::webgpu_cpp_decl::StencilOperation::Keep.into(),
    }
}

fn wgpuBlendOperation(
    equation: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::BlendEquation,
) -> super::webgpu_decl::WGPUBlendOperation {
    use super::webgpu_wagyu_decl as wagyu;
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::BlendEquation;
    match equation {
        BlendEquation::none | BlendEquation::srcOver | BlendEquation::plus => {
            super::webgpu_cpp_decl::BlendOperation::Add.into()
        }
        BlendEquation::min => super::webgpu_cpp_decl::BlendOperation::Min.into(),
        BlendEquation::max => super::webgpu_cpp_decl::BlendOperation::Max.into(),
        BlendEquation::multiply => wagyu::WGPUBlendOperation_WagyuMultiply,
        BlendEquation::screen => wagyu::WGPUBlendOperation_WagyuScreen,
        BlendEquation::overlay => wagyu::WGPUBlendOperation_WagyuOverlay,
        BlendEquation::darken => wagyu::WGPUBlendOperation_WagyuDarken,
        BlendEquation::lighten => wagyu::WGPUBlendOperation_WagyuLighten,
        BlendEquation::colorDodge => wagyu::WGPUBlendOperation_WagyuColorDodge,
        BlendEquation::colorBurn => wagyu::WGPUBlendOperation_WagyuColorBurn,
        BlendEquation::hardLight => wagyu::WGPUBlendOperation_WagyuHardLight,
        BlendEquation::softLight => wagyu::WGPUBlendOperation_WagyuSoftLight,
        BlendEquation::difference => wagyu::WGPUBlendOperation_WagyuDifference,
        BlendEquation::exclusion => wagyu::WGPUBlendOperation_WagyuExclusion,
        BlendEquation::hue => wagyu::WGPUBlendOperation_WagyuHue,
        BlendEquation::saturation => wagyu::WGPUBlendOperation_WagyuSaturation,
        BlendEquation::color => wagyu::WGPUBlendOperation_WagyuColor,
        BlendEquation::luminosity => wagyu::WGPUBlendOperation_WagyuLuminosity,
    }
}

fn wgpuDstBlendFactor(
    equation: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::BlendEquation,
) -> super::webgpu_decl::WGPUBlendFactor {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::BlendEquation;
    match equation {
        BlendEquation::plus | BlendEquation::min | BlendEquation::max => {
            super::webgpu_cpp_decl::BlendFactor::One.into()
        }
        _ => super::webgpu_cpp_decl::BlendFactor::OneMinusSrcAlpha.into(),
    }
}

fn shaderUsesOverride(source: &str, index: usize) -> bool {
    source.contains(&format!("@id({index}) override"))
}

fn buildConstantEntries(
    source: Option<&str>,
    values: &[f64; SPECIALIZATION_COUNT],
) -> Vec<super::webgpu_decl::WGPUConstantEntry> {
    let Some(source) = source else {
        return Vec::new();
    };
    SPECIALIZATION_IDS
        .iter()
        .enumerate()
        .filter(|(index, _)| shaderUsesOverride(source, *index))
        .map(|(index, key)| {
            let mut entry = super::webgpu_decl::WGPUConstantEntry::default();
            entry.key = stringView(key);
            entry.value = values[index];
            entry
        })
        .collect()
}

fn newLoadStoreEXTPipeline(
    context: &RenderContextWebGPUImpl,
    actions: LoadStoreActionsEXT,
    framebufferFormat: TextureFormat,
) -> super::render_context_webgpu_decl::LoadStoreEXTPipeline {
    let device = context.device();
    let mut bindGroupLayout = BindGroupLayout::default();
    let mut pipelineLayoutDescriptor = WGPUPipelineLayoutDescriptor::default();
    let rawLayout;
    if actions.has(LoadStoreActionsEXT::clearColor) {
        let mut entry = WGPUBindGroupLayoutEntry::default();
        entry.binding = 0;
        entry.visibility = WGPUShaderStage_Fragment;
        entry.buffer.r#type = WGPUBufferBindingType_Uniform;
        bindGroupLayout = createLayout(&device, &[entry]);
        rawLayout = bindGroupLayout.Get();
        pipelineLayoutDescriptor.bindGroupLayoutCount = 1;
        pipelineLayoutDescriptor.bindGroupLayouts = &rawLayout;
    }
    let pipelineLayout = unsafe { device.CreatePipelineLayout(&pipelineLayoutDescriptor) };

    let mut fragmentSource = format!(
        "#version 310 es\n#define {GLSL_FRAGMENT} true\n#define {GLSL_ENABLE_CLIPPING} true\n"
    );
    BuildLoadStoreEXTGLSL(&mut fragmentSource, actions);
    let fragmentModule = compileShaderModuleWagyuRaw(&device, &fragmentSource);
    let mut colorTarget = WGPUColorTargetState::default();
    colorTarget.format = framebufferFormat.into();
    let mut fragmentState = WGPUFragmentState::default();
    fragmentState.module = fragmentModule.Get();
    fragmentState.entryPoint = stringView("main");
    fragmentState.targetCount = 1;
    fragmentState.targets = &colorTarget;
    let mut descriptor = WGPURenderPipelineDescriptor::default();
    descriptor.label = stringView("RIVE_LoadStoreEXTPipeline");
    descriptor.layout = pipelineLayout.Get();
    descriptor.vertex.module = context.m_loadStoreEXTVertexShader.Get();
    descriptor.vertex.entryPoint = stringView("main");
    descriptor.primitive.topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleStrip.into();
    descriptor.primitive.frontFace = RIVE_FRONT_FACE.into();
    descriptor.primitive.cullMode = super::webgpu_cpp_decl::CullMode::None.into();
    descriptor.fragment = &fragmentState;
    let renderPipeline = unsafe { device.CreateRenderPipeline(&descriptor) };
    super::render_context_webgpu_decl::LoadStoreEXTPipeline {
        m_framebufferFormat: framebufferFormat,
        m_bindGroupLayout: std::mem::ManuallyDrop::new(bindGroupLayout),
        m_renderPipeline: std::mem::ManuallyDrop::new(renderPipeline),
    }
}

/// Concrete source subclasses add native-buffer and texture-view access to the
/// complete `BufferRing` virtual contract.
pub(crate) trait BufferRingWebGPUApi: BufferRingContract {
    fn submittedBuffer(&self) -> Buffer;
    fn textureView(&self) -> Option<TextureView> {
        None
    }
}

#[repr(C)]
pub(crate) struct RenderBufferWebGPUImpl {
    pub(crate) base: std::mem::ManuallyDrop<RenderBuffer>,
    pub(crate) m_device: std::mem::ManuallyDrop<Device>,
    pub(crate) m_queue: std::mem::ManuallyDrop<Queue>,
    pub(crate) m_buffers: std::mem::ManuallyDrop<[Buffer; kBufferRingSize as usize]>,
    pub(crate) m_submittedBufferIdx: i32,
    pub(crate) m_stagingBuffer: std::mem::ManuallyDrop<Option<Box<[u8]>>>,
}

impl LiteRttiTypeId for RenderBufferWebGPUImpl {
    const LITE_RTTI_TYPE_ID: u32 =
        crate::mechanical_port::source::include::utils::lite_rtti_hpp::CONST_ID(
            "RenderBufferWebGPUImpl",
        );
}

impl RenderBufferWebGPUImpl {
    pub(crate) fn new(
        device: Device,
        queue: Queue,
        ty: RenderBufferType,
        flags: RenderBufferFlags,
        sizeInBytes: usize,
    ) -> Self {
        let mappedOnce = flags as i32 & RenderBufferFlags::mappedOnceAtInitialization as i32 != 0;
        let bufferCount = if mappedOnce {
            1
        } else {
            kBufferRingSize as usize
        };
        let mut desc = super::webgpu_decl::WGPUBufferDescriptor::default();
        let mut usage = if ty == RenderBufferType::index {
            super::webgpu_cpp_decl::BufferUsage::Index
        } else {
            super::webgpu_cpp_decl::BufferUsage::Vertex
        };
        desc.size = sizeInBytes.next_multiple_of(4) as u64;
        desc.mappedAtCreation = if mappedOnce {
            super::webgpu_decl::WGPU_TRUE
        } else {
            usage |= super::webgpu_cpp_decl::BufferUsage::CopyDst;
            super::webgpu_decl::WGPU_FALSE
        };
        desc.usage = usage.into();
        let mut buffers = std::array::from_fn(|_| Buffer::default());
        for buffer in &mut buffers[..bufferCount] {
            *buffer = unsafe { device.CreateBuffer(&desc) };
        }
        Self {
            base: std::mem::ManuallyDrop::new(unsafe {
                RenderBuffer::new_for_owner::<Self>(ty, flags, sizeInBytes)
            }),
            m_device: std::mem::ManuallyDrop::new(device),
            m_queue: std::mem::ManuallyDrop::new(queue),
            m_buffers: std::mem::ManuallyDrop::new(buffers),
            m_submittedBufferIdx: -1,
            m_stagingBuffer: std::mem::ManuallyDrop::new(None),
        }
    }

    pub(crate) fn submittedBuffer(&self) -> Buffer {
        (&self.m_buffers[self.m_submittedBufferIdx as usize]).clone()
    }
}

impl RenderBufferContract for RenderBufferWebGPUImpl {
    fn onMap(&mut self) -> *mut core::ffi::c_void {
        self.m_submittedBufferIdx = (self.m_submittedBufferIdx + 1) % kBufferRingSize;
        let buffer = &self.m_buffers[self.m_submittedBufferIdx as usize];
        assert!(!buffer.Get().is_null());
        if self.base.flags() as i32 & RenderBufferFlags::mappedOnceAtInitialization as i32 != 0 {
            unsafe { buffer.GetMappedRange(0, super::webgpu_decl::WGPU_WHOLE_MAP_SIZE) }
        } else {
            if self.m_stagingBuffer.is_none() {
                *self.m_stagingBuffer = Some(vec![0; self.base.sizeInBytes()].into_boxed_slice());
            }
            self.m_stagingBuffer
                .as_mut()
                .expect("staging buffer")
                .as_mut_ptr()
                .cast()
        }
    }

    fn onUnmap(&mut self) {
        let buffer = &self.m_buffers[self.m_submittedBufferIdx as usize];
        if self.base.flags() as i32 & RenderBufferFlags::mappedOnceAtInitialization as i32 != 0 {
            unsafe { buffer.Unmap() };
        } else {
            let staging = self
                .m_stagingBuffer
                .as_ref()
                .expect("mapped staging buffer");
            unsafe {
                self.m_queue.WriteBuffer(
                    buffer.Get(),
                    0,
                    staging.as_ptr().cast(),
                    self.base.sizeInBytes(),
                )
            };
        }
    }
}

unsafe impl RefCntTarget for RenderBufferWebGPUImpl {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) };
    }
}

impl Drop for RenderBufferWebGPUImpl {
    fn drop(&mut self) {
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.m_stagingBuffer);
            super::render_context_webgpu_decl::dropArrayReverse(&mut self.m_buffers);
            std::mem::ManuallyDrop::drop(&mut self.m_queue);
            std::mem::ManuallyDrop::drop(&mut self.m_device);
            std::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}

pub(crate) fn makeRenderBuffer(
    context: &RenderContextWebGPUImpl,
    ty: RenderBufferType,
    flags: RenderBufferFlags,
    sizeInBytes: usize,
) -> rcp<RenderBuffer> {
    let derived = make_rcp(|| {
        RenderBufferWebGPUImpl::new(
            (&*context.m_device).clone(),
            (&*context.m_queue).clone(),
            ty,
            flags,
            sizeInBytes,
        )
    });
    unsafe { static_rcp_cast(derived) }
}

#[repr(C)]
pub(crate) struct BufferWebGPU {
    pub(crate) base: std::mem::ManuallyDrop<BufferRing>,
    pub(crate) m_queue: std::mem::ManuallyDrop<Queue>,
    pub(crate) m_buffers: std::mem::ManuallyDrop<[Buffer; kBufferRingSize as usize]>,
}

impl BufferWebGPU {
    pub(crate) fn new(
        device: Device,
        queue: Queue,
        capacityInBytesUnrounded: usize,
        usage: super::webgpu_cpp_decl::BufferUsage,
    ) -> Self {
        let capacity = capacityInBytesUnrounded.max(1).next_multiple_of(4);
        let mut desc = super::webgpu_decl::WGPUBufferDescriptor::default();
        desc.usage = (super::webgpu_cpp_decl::BufferUsage::CopyDst | usage)
            .intoBitmask()
            .into();
        desc.size = capacity as u64;
        let buffers = std::array::from_fn(|_| unsafe { device.CreateBuffer(&desc) });
        Self {
            base: std::mem::ManuallyDrop::new(BufferRing::new(capacity)),
            m_queue: std::mem::ManuallyDrop::new(queue),
            m_buffers: std::mem::ManuallyDrop::new(buffers),
        }
    }
}

impl Drop for BufferWebGPU {
    fn drop(&mut self) {
        unsafe {
            super::render_context_webgpu_decl::dropArrayReverse(&mut self.m_buffers);
            std::mem::ManuallyDrop::drop(&mut self.m_queue);
            std::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl BufferRingContract for BufferWebGPU {
    fn bufferRing(&self) -> &BufferRing {
        &self.base
    }
    fn bufferRingMut(&mut self) -> &mut BufferRing {
        &mut self.base
    }
    fn onMapBuffer(&mut self, _bufferIdx: i32, _mapSizeInBytes: usize) -> *mut core::ffi::c_void {
        self.base.shadowBuffer().cast()
    }
    fn onUnmapAndSubmitBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) {
        unsafe {
            self.m_queue.WriteBuffer(
                self.m_buffers[bufferIdx as usize].Get(),
                0,
                self.base.shadowBuffer().cast(),
                mapSizeInBytes,
            )
        };
    }
}

impl BufferRingWebGPUApi for BufferWebGPU {
    fn submittedBuffer(&self) -> Buffer {
        self.m_buffers[self.base.submittedBufferIdx() as usize].clone()
    }
}

fn storageTextureFormat(bufferStructure: StorageBufferStructure) -> TextureFormat {
    match bufferStructure {
        StorageBufferStructure::uint32x4 => TextureFormat::RGBA32Uint,
        StorageBufferStructure::uint32x2 => TextureFormat::RG32Uint,
        StorageBufferStructure::float32x4 => TextureFormat::RGBA32Float,
    }
}

#[repr(C)]
pub(crate) struct StorageTextureBufferWebGPU {
    pub(crate) base: std::mem::ManuallyDrop<BufferWebGPU>,
    pub(crate) m_bufferStructure: StorageBufferStructure,
    pub(crate) m_texture: std::mem::ManuallyDrop<WagyuTexture>,
    pub(crate) m_textureView: std::mem::ManuallyDrop<TextureView>,
}

impl StorageTextureBufferWebGPU {
    pub(crate) fn new(
        device: Device,
        queue: Queue,
        capacityInBytes: usize,
        bufferStructure: StorageBufferStructure,
    ) -> Self {
        let base = BufferWebGPU::new(
            device.clone(),
            queue,
            StorageTextureBufferSize(capacityInBytes, bufferStructure),
            super::webgpu_cpp_decl::BufferUsage::CopySrc,
        );
        let (width, height) = StorageTextureSize(base.base.capacityInBytes(), bufferStructure);
        let texture = makeTexture(
            &device,
            (TextureUsage::TextureBinding | TextureUsage::CopyDst).intoBitmask(),
            width,
            height,
            storageTextureFormat(bufferStructure),
            1,
        );
        let view = makeView(&texture);
        Self {
            base: std::mem::ManuallyDrop::new(base),
            m_bufferStructure: bufferStructure,
            m_texture: std::mem::ManuallyDrop::new(texture),
            m_textureView: std::mem::ManuallyDrop::new(view),
        }
    }

    pub(crate) fn updateTextureFromBuffer(
        &self,
        bindingSizeInBytes: usize,
        offsetSizeInBytes: usize,
        commandEncoder: &CommandEncoder,
    ) {
        let (width, height) = StorageTextureSize(bindingSizeInBytes, self.m_bufferStructure);
        let mut source = super::webgpu_decl::WGPUTexelCopyBufferInfo::default();
        source.layout.offset = offsetSizeInBytes as u64;
        source.layout.bytesPerRow = 2048
            * crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::StorageBufferElementSizeInBytes(
                self.m_bufferStructure,
            );
        source.buffer = self.base.submittedBuffer().Get();
        let mut destination = WGPUTexelCopyTextureInfo::default();
        destination.texture = self.m_texture.Get();
        let extent = WGPUExtent3D {
            width,
            height,
            depthOrArrayLayers: 1,
        };
        unsafe { commandEncoder.CopyBufferToTexture(&source, &destination, &extent) };
    }
}

impl BufferRingContract for StorageTextureBufferWebGPU {
    fn bufferRing(&self) -> &BufferRing {
        &self.base.base
    }
    fn bufferRingMut(&mut self) -> &mut BufferRing {
        &mut self.base.base
    }
    fn onMapBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) -> *mut core::ffi::c_void {
        self.base.onMapBuffer(bufferIdx, mapSizeInBytes)
    }
    fn onUnmapAndSubmitBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) {
        self.base.onUnmapAndSubmitBuffer(bufferIdx, mapSizeInBytes)
    }
}

impl BufferRingWebGPUApi for StorageTextureBufferWebGPU {
    fn submittedBuffer(&self) -> Buffer {
        self.base.submittedBuffer()
    }
    fn textureView(&self) -> Option<TextureView> {
        Some((&*self.m_textureView).clone())
    }
}

impl Drop for StorageTextureBufferWebGPU {
    fn drop(&mut self) {
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.m_textureView);
            std::mem::ManuallyDrop::drop(&mut self.m_texture);
            std::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}

pub(crate) fn makeUniformBufferRing(
    context: &RenderContextWebGPUImpl,
    capacityInBytes: usize,
) -> Box<dyn BufferRingWebGPUApi> {
    let capacity = capacityInBytes.max(256);
    assert_eq!(capacity % 256, 0);
    Box::new(BufferWebGPU::new(
        (&*context.m_device).clone(),
        (&*context.m_queue).clone(),
        capacity,
        super::webgpu_cpp_decl::BufferUsage::Uniform,
    ))
}

pub(crate) fn makeStorageBufferRing(
    context: &RenderContextWebGPUImpl,
    capacityInBytes: usize,
    bufferStructure: StorageBufferStructure,
) -> Box<dyn BufferRingWebGPUApi> {
    if context.m_capabilities.polyfillVertexStorageBuffers {
        Box::new(StorageTextureBufferWebGPU::new(
            (&*context.m_device).clone(),
            (&*context.m_queue).clone(),
            capacityInBytes,
            bufferStructure,
        ))
    } else {
        Box::new(BufferWebGPU::new(
            (&*context.m_device).clone(),
            (&*context.m_queue).clone(),
            capacityInBytes,
            super::webgpu_cpp_decl::BufferUsage::Storage,
        ))
    }
}

pub(crate) fn makeVertexBufferRing(
    context: &RenderContextWebGPUImpl,
    capacityInBytes: usize,
) -> Box<dyn BufferRingWebGPUApi> {
    Box::new(BufferWebGPU::new(
        (&*context.m_device).clone(),
        (&*context.m_queue).clone(),
        capacityInBytes,
        super::webgpu_cpp_decl::BufferUsage::Vertex,
    ))
}

pub(crate) fn generateMipmaps(context: &RenderContextWebGPUImpl, texture: &WagyuTexture) {
    let encoder = unsafe { context.m_device.CreateCommandEncoder(std::ptr::null()) };
    unsafe {
        super::webgpu_wagyu_decl::wgpuWagyuCommandEncoderGenerateMipmap(
            encoder.Get(),
            texture.Get(),
        )
    };
    let commands = unsafe { encoder.Finish(std::ptr::null()) };
    let raw = commands.Get();
    unsafe { context.m_queue.Submit(1, &raw) };
}

pub(crate) fn makeImageTexture(
    context: &RenderContextWebGPUImpl,
    width: u32,
    height: u32,
    mipLevelCount: u32,
    format: GPUTextureFormat,
    imageData: &[u8],
    blockWidth: u8,
    blockHeight: u8,
    _srgb: bool,
    generateRemainingMips: bool,
) -> rcp<RiveTexture> {
    let (wgpuFormat, bytesPerBlock, compressed) = match format {
        GPUTextureFormat::rgba32 => {
            assert!(blockWidth == 1 && blockHeight == 1);
            (TextureFormat::RGBA8Unorm, 4u32, false)
        }
        GPUTextureFormat::bc7 => (TextureFormat::BC7RGBAUnorm, 16, true),
        GPUTextureFormat::astc => {
            let index = crate::mechanical_port::source::decoders::include::rive::decoders::astc_footprints_hpp::astcFootprintIndex(
                blockWidth,
                blockHeight,
            );
            if index < 0 {
                debug_assert!(false, "unsupported ASTC block footprint");
                return rcp::new();
            }
            (
                TextureFormat(TextureFormat::ASTC4x4Unorm.0 + 2 * index as u32),
                16,
                true,
            )
        }
        GPUTextureFormat::etc2 => (TextureFormat::ETC2RGBA8Unorm, 16, true),
        _ => {
            debug_assert!(false, "unsupported format");
            return rcp::new();
        }
    };
    debug_assert!(
        !(generateRemainingMips && compressed),
        "WebGPU mip generation is undefined on compressed formats"
    );

    let mut usage = (TextureUsage::TextureBinding | TextureUsage::CopyDst).intoBitmask();
    if generateRemainingMips && mipLevelCount > 1 {
        usage |= TextureUsage::CopySrc;
    }
    let mut textureDesc = WGPUTextureDescriptor::default();
    textureDesc.usage = usage.into();
    textureDesc.dimension = super::webgpu_cpp_decl::TextureDimension::e2D.into();
    textureDesc.size.width = width;
    textureDesc.size.height = height;
    textureDesc.size.depthOrArrayLayers = 1;
    textureDesc.format = wgpuFormat.into();
    textureDesc.mipLevelCount = mipLevelCount;
    textureDesc.sampleCount = 1;
    let texture = unsafe { context.m_device.CreateTexture(&textureDesc) };

    let levelsToUpload = if generateRemainingMips {
        1
    } else {
        mipLevelCount
    };
    let mut sourceOffset = 0usize;
    for level in 0..levelsToUpload {
        let levelWidth = (width >> level).max(1);
        let levelHeight = (height >> level).max(1);
        let blocksX = levelWidth.div_ceil(u32::from(blockWidth));
        let blocksY = levelHeight.div_ceil(u32::from(blockHeight));
        let bytesPerRow = blocksX * bytesPerBlock;
        let levelBytes = bytesPerRow as usize * blocksY as usize;
        let Some(levelData) = imageData.get(sourceOffset..sourceOffset + levelBytes) else {
            return rcp::new();
        };
        let mut destination = WGPUTexelCopyTextureInfo::default();
        destination.texture = texture.Get();
        destination.mipLevel = level;
        let mut layout = super::webgpu_decl::WGPUTexelCopyBufferLayout::default();
        layout.bytesPerRow = bytesPerRow;
        let extent = WGPUExtent3D {
            width: levelWidth,
            height: levelHeight,
            depthOrArrayLayers: 1,
        };
        unsafe {
            context.m_queue.WriteTexture(
                &destination,
                levelData.as_ptr().cast(),
                levelBytes,
                &layout,
                &extent,
            )
        };
        sourceOffset += levelBytes;
    }
    if generateRemainingMips && mipLevelCount > 1 {
        generateMipmaps(context, &texture);
    }
    let derived = make_rcp(|| *TextureWebGPUImpl::new(width, height, texture));
    unsafe { static_rcp_cast(derived) }
}

fn resizeSampledRenderTexture(
    device: &Device,
    slot: &mut WagyuTexture,
    viewSlot: &mut TextureView,
    width: u32,
    height: u32,
    format: TextureFormat,
) {
    *slot = makeTexture(
        device,
        (TextureUsage::RenderAttachment | TextureUsage::TextureBinding).intoBitmask(),
        width.max(1),
        height.max(1),
        format,
        1,
    );
    *viewSlot = makeView(slot);
}

pub(crate) fn resizeGradientTexture(
    context: &mut RenderContextWebGPUImpl,
    width: u32,
    height: u32,
) {
    resizeSampledRenderTexture(
        &context.m_device,
        &mut context.m_gradientTexture,
        &mut context.m_gradientTextureView,
        width,
        height,
        TextureFormat::RGBA8Unorm,
    );
}

pub(crate) fn resizeTessellationTexture(
    context: &mut RenderContextWebGPUImpl,
    width: u32,
    height: u32,
) {
    resizeSampledRenderTexture(
        &context.m_device,
        &mut context.m_tessVertexTexture,
        &mut context.m_tessVertexTextureView,
        width,
        height,
        TextureFormat::RGBA32Uint,
    );
}

pub(crate) fn resizeFeatherAtlasTexture(
    context: &mut RenderContextWebGPUImpl,
    width: u32,
    height: u32,
) {
    resizeSampledRenderTexture(
        &context.m_device,
        &mut context.m_featherAtlasTexture,
        &mut context.m_featherAtlasTextureView,
        width,
        height,
        TextureFormat::R16Float,
    );
}

pub(crate) fn resizeAtomicCoverageBacking(
    context: &mut RenderContextWebGPUImpl,
    width: u32,
    height: u32,
) {
    const BUFFER_IMAGE_TILE_SIZE: u64 = 32;
    context.m_atomicPLSBackingBufferSize = u64::from(height)
        .next_multiple_of(BUFFER_IMAGE_TILE_SIZE)
        * u64::from(width).next_multiple_of(BUFFER_IMAGE_TILE_SIZE)
        * std::mem::size_of::<u32>() as u64;
    *context.m_atomicPLSColorBuffer = Buffer::default();
    *context.m_atomicPLSClipBuffer = Buffer::default();
    *context.m_atomicPLSCoverageBuffer = Buffer::default();
}

fn atomicPLSBuffer(device: &Device, slot: &mut Buffer, size: u64) -> Buffer {
    if slot.Get().is_null() {
        let mut desc = super::webgpu_decl::WGPUBufferDescriptor::default();
        desc.usage = super::webgpu_cpp_decl::BufferUsage::Storage.into();
        desc.size = size;
        *slot = unsafe { device.CreateBuffer(&desc) };
    }
    slot.clone()
}

pub(crate) fn atomicPLSColorBuffer(context: &mut RenderContextWebGPUImpl) -> Buffer {
    atomicPLSBuffer(
        &context.m_device,
        &mut context.m_atomicPLSColorBuffer,
        context.m_atomicPLSBackingBufferSize,
    )
}

pub(crate) fn atomicPLSClipBuffer(context: &mut RenderContextWebGPUImpl) -> Buffer {
    atomicPLSBuffer(
        &context.m_device,
        &mut context.m_atomicPLSClipBuffer,
        context.m_atomicPLSBackingBufferSize,
    )
}

pub(crate) fn atomicPLSCoverageBuffer(context: &mut RenderContextWebGPUImpl) -> Buffer {
    atomicPLSBuffer(
        &context.m_device,
        &mut context.m_atomicPLSCoverageBuffer,
        context.m_atomicPLSBackingBufferSize,
    )
}

fn stringViewEquals(view: &WGPUStringView, expected: &[u8]) -> bool {
    if view.data.is_null() {
        return false;
    }
    let actual = unsafe {
        if view.length == WGPU_STRLEN {
            std::ffi::CStr::from_ptr(view.data).to_bytes()
        } else {
            std::slice::from_raw_parts(view.data.cast::<u8>(), view.length)
        }
    };
    actual == expected
}

fn parseVendorExtensions(adapter: &Adapter, device: &Device, caps: &mut Capabilities) {
    let mut extensions = WGPUWagyuStringArray {
        stringCount: 0,
        strings: std::ptr::null_mut(),
    };
    unsafe {
        if caps.backendType == BackendType::Vulkan {
            wgpuWagyuDeviceGetExtensions(device.Get(), &mut extensions);
            for extension in std::slice::from_raw_parts(extensions.strings, extensions.stringCount)
            {
                if stringViewEquals(extension, b"VK_EXT_rasterization_order_attachment_access") {
                    caps.VK_EXT_rasterization_order_attachment_access = true;
                }
            }
        } else if caps.backendType == BackendType::OpenGLES {
            wgpuWagyuAdapterGetExtensions(adapter.Get(), &mut extensions);
            for extension in std::slice::from_raw_parts(extensions.strings, extensions.stringCount)
            {
                if stringViewEquals(extension, b"GL_EXT_shader_pixel_local_storage") {
                    caps.GL_EXT_shader_pixel_local_storage = true;
                } else if stringViewEquals(extension, b"GL_EXT_shader_pixel_local_storage2") {
                    caps.GL_EXT_shader_pixel_local_storage2 = true;
                }
            }
        }
        wgpuWagyuStringArrayFreeMembers(extensions);
    }
}

pub(crate) fn newContext(
    adapter: Adapter,
    device: Device,
    queue: Queue,
    contextOptions: ContextOptions,
) -> RenderContextWebGPUImpl {
    let mut context = RenderContextWebGPUImpl {
        base: std::mem::ManuallyDrop::new(RenderContextHelperImpl::new(
            RenderContextImpl::default(),
        )),
        m_device: std::mem::ManuallyDrop::new(device),
        m_queue: std::mem::ManuallyDrop::new(queue),
        m_contextOptions: contextOptions,
        m_capabilities: Capabilities::default(),
        m_drawPipelineLayouts: std::mem::ManuallyDrop::new(std::array::from_fn(|_| None)),
        m_loadStoreEXTPipelines: std::mem::ManuallyDrop::new(Default::default()),
        m_loadStoreEXTVertexShader: std::mem::ManuallyDrop::new(ShaderModule::default()),
        m_loadStoreEXTUniforms: std::mem::ManuallyDrop::new(None),
        m_colorRampPipeline: std::mem::ManuallyDrop::new(None),
        m_gradientTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_gradientTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_tessellatePipeline: std::mem::ManuallyDrop::new(None),
        m_tessSpanIndexBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
        m_tessVertexTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_tessVertexTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_featherAtlasPipeline: std::mem::ManuallyDrop::new(None),
        m_featherAtlasTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_featherAtlasTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_drawPipelines: std::mem::ManuallyDrop::new(Default::default()),
        m_linearSampler: std::mem::ManuallyDrop::new(Sampler::default()),
        m_imageSamplers: std::mem::ManuallyDrop::new(std::array::from_fn(|_| Sampler::default())),
        m_samplerBindings: std::mem::ManuallyDrop::new(BindGroup::default()),
        m_emptyBindingsLayout: std::mem::ManuallyDrop::new(BindGroupLayout::default()),
        m_pathPatchVertexBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
        m_pathPatchIndexBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
        m_imageRectVertexBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
        m_imageRectIndexBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
        m_gaussianIntegralTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_gaussianIntegralTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_atomicPLSBackingBufferSize: 0,
        m_atomicPLSColorBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
        m_atomicPLSClipBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
        m_atomicPLSCoverageBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
        m_nullTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_nullTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_nullStorageBuffer: std::mem::ManuallyDrop::new(Buffer::default()),
    };

    let mut limits = WGPULimits::default();
    let mut compatibilityLimits = WGPUCompatibilityModeLimits::default();
    if contextOptions.compatibilityMode {
        limits.nextInChain = &mut compatibilityLimits.chain;
    }
    let limitsValid = unsafe { context.m_device.GetLimits(&mut limits).asBool() };
    context.m_capabilities.backendType =
        unsafe { BackendType::from(wgpuWagyuAdapterGetBackend(adapter.Get())) };
    parseVendorExtensions(&adapter, &context.m_device, &mut context.m_capabilities);

    if context
        .m_capabilities
        .VK_EXT_rasterization_order_attachment_access
    {
        context.m_capabilities.plsType =
            PixelLocalStorageType::VK_EXT_rasterization_order_attachment_access;
        context
            .base
            .base
            .m_platformFeatures
            .supportsRasterOrderingMode = true;
    } else if context.m_capabilities.GL_EXT_shader_pixel_local_storage {
        context.m_capabilities.plsType = PixelLocalStorageType::GL_EXT_shader_pixel_local_storage;
        context
            .base
            .base
            .m_platformFeatures
            .supportsRasterOrderingMode = true;
        context.base.base.m_platformFeatures.supportsClockwiseMode = true;
        context
            .base
            .base
            .m_platformFeatures
            .supportsClockwiseFixedFunctionMode =
            context.m_capabilities.GL_EXT_shader_pixel_local_storage2;
    }

    let mut maxStorageBuffersInVertexStage = WGPU_LIMIT_U32_UNDEFINED;
    if limitsValid {
        maxStorageBuffersInVertexStage = if contextOptions.compatibilityMode {
            compatibilityLimits.maxStorageBuffersInVertexStage
        } else {
            limits.maxStorageBuffersPerShaderStage
        };
    }
    if maxStorageBuffersInVertexStage == WGPU_LIMIT_U32_UNDEFINED {
        maxStorageBuffersInVertexStage = if contextOptions.compatibilityMode {
            0
        } else {
            8
        };
    }
    if maxStorageBuffersInVertexStage
        < crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::kMaxStorageBuffers
            as u32
    {
        debug_assert!(contextOptions.compatibilityMode);
        context.m_capabilities.polyfillVertexStorageBuffers = true;
    }

    let features = &mut context.base.base.m_platformFeatures;
    features.supportsAtomicMode = limitsValid && limits.maxStorageBuffersPerShaderStage >= 3;
    if !features.supportsAtomicMode {
        eprint!(
            "WARNING: atomic mode disabled because deviceLimits.maxStorageBuffersPerShaderStage is not at least 3."
        );
    }
    features.atomicPLSInitNeedsDraw = true;
    features.supportsBlendAdvancedKHR = unsafe {
        context
            .m_device
            .HasFeature(WGPUFeatureName_WagyuBlendEquationAdvancedCoherent)
            .asBool()
    };
    features.supportsBlendAdvancedCoherentKHR = features.supportsBlendAdvancedKHR;
    features.supportsClipPlanes = unsafe {
        context
            .m_device
            .HasFeature(FeatureName::ClipDistances.into())
            .asBool()
    };
    features.clipSpaceBottomUp = true;
    features.framebufferBottomUp = false;
    features.msaaColorPreserveNeedsDraw = true;
    features.supportsTextureCompressionBC = unsafe {
        context
            .m_device
            .HasFeature(FeatureName::TextureCompressionBC.into())
            .asBool()
    };
    features.supportsTextureCompressionASTC = unsafe {
        context
            .m_device
            .HasFeature(FeatureName::TextureCompressionASTC.into())
            .asBool()
    };
    features.supportsTextureCompressionETC2 = unsafe {
        context
            .m_device
            .HasFeature(FeatureName::TextureCompressionETC2.into())
            .asBool()
    };
    context
}

fn bufferLayoutEntry(binding: u32, visibility: u64, ty: i32) -> WGPUBindGroupLayoutEntry {
    let mut entry = WGPUBindGroupLayoutEntry::default();
    entry.binding = binding;
    entry.visibility = visibility;
    entry.buffer.r#type = ty;
    entry
}

fn textureLayoutEntry(binding: u32, visibility: u64, sampleType: i32) -> WGPUBindGroupLayoutEntry {
    let mut entry = WGPUBindGroupLayoutEntry::default();
    entry.binding = binding;
    entry.visibility = visibility;
    entry.texture.sampleType = sampleType;
    entry.texture.viewDimension = WGPUTextureViewDimension_2D;
    entry
}

fn samplerLayoutEntry(binding: u32, visibility: u64) -> WGPUBindGroupLayoutEntry {
    let mut entry = WGPUBindGroupLayoutEntry::default();
    entry.binding = binding;
    entry.visibility = visibility;
    entry.sampler.r#type = WGPUSamplerBindingType_Filtering;
    entry
}

fn createLayout(device: &Device, entries: &[WGPUBindGroupLayoutEntry]) -> BindGroupLayout {
    let descriptor = WGPUBindGroupLayoutDescriptor {
        nextInChain: std::ptr::null_mut(),
        label: WGPUStringView::default(),
        entryCount: entries.len(),
        entries: entries.as_ptr(),
    };
    unsafe { device.CreateBindGroupLayout(&descriptor) }
}

fn newDrawPipelineLayout(
    context: &RenderContextWebGPUImpl,
    interlockMode: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode,
) -> DrawPipelineLayout {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode;

    let vertex = WGPUShaderStage_Vertex;
    let fragment = WGPUShaderStage_Fragment;
    let vertexFragment = vertex | fragment;
    let (pathVisibility, paintVisibility) = if interlockMode == InterlockMode::atomics {
        (vertexFragment, fragment)
    } else {
        (vertex, vertex)
    };
    let polyfill = context.m_capabilities.polyfillVertexStorageBuffers;
    let storageOrTexture = |binding, visibility, sampleType| {
        if polyfill {
            textureLayoutEntry(binding, visibility, sampleType)
        } else {
            bufferLayoutEntry(binding, visibility, WGPUBufferBindingType_ReadOnlyStorage)
        }
    };
    let perFlushEntries = [
        bufferLayoutEntry(
            FLUSH_UNIFORM_BUFFER_IDX,
            vertexFragment,
            WGPUBufferBindingType_Uniform,
        ),
        storageOrTexture(PATH_BUFFER_IDX, pathVisibility, WGPUTextureSampleType_Uint),
        storageOrTexture(
            PAINT_BUFFER_IDX,
            paintVisibility,
            WGPUTextureSampleType_Uint,
        ),
        storageOrTexture(
            PAINT_AUX_BUFFER_IDX,
            paintVisibility,
            WGPUTextureSampleType_UnfilterableFloat,
        ),
        storageOrTexture(CONTOUR_BUFFER_IDX, vertex, WGPUTextureSampleType_Uint),
        textureLayoutEntry(
            GAUSSIAN_INTEGRAL_TEXTURE_IDX,
            vertexFragment,
            WGPUTextureSampleType_Float,
        ),
        textureLayoutEntry(TESS_VERTEX_TEXTURE_IDX, vertex, WGPUTextureSampleType_Uint),
        textureLayoutEntry(
            FEATHER_ATLAS_TEXTURE_IDX,
            fragment,
            WGPUTextureSampleType_Float,
        ),
        textureLayoutEntry(GRAD_TEXTURE_IDX, fragment, WGPUTextureSampleType_Float),
        textureLayoutEntry(DST_COLOR_TEXTURE_IDX, fragment, WGPUTextureSampleType_Float),
    ];
    let mut layouts: [BindGroupLayout; WEBGPU_BINDINGS_SET_COUNT] =
        std::array::from_fn(|_| BindGroupLayout::default());
    layouts[PER_FLUSH_BINDINGS_SET] = createLayout(&context.m_device, &perFlushEntries);

    let perDrawEntries = [
        textureLayoutEntry(IMAGE_TEXTURE_IDX, fragment, WGPUTextureSampleType_Float),
        samplerLayoutEntry(WEBGPU_IMAGE_SAMPLER_IDX, fragment),
    ];
    layouts[PER_DRAW_BINDINGS_SET] = createLayout(&context.m_device, &perDrawEntries);

    if interlockMode == InterlockMode::atomics {
        let atomicEntries = [
            bufferLayoutEntry(COLOR_PLANE_IDX, fragment, WGPUBufferBindingType_Storage),
            bufferLayoutEntry(CLIP_PLANE_IDX, fragment, WGPUBufferBindingType_Storage),
            bufferLayoutEntry(COVERAGE_PLANE_IDX, fragment, WGPUBufferBindingType_Storage),
        ];
        layouts[PLS_TEXTURE_BINDINGS_SET] = createLayout(&context.m_device, &atomicEntries);
    } else if interlockMode == InterlockMode::rasterOrdering
        && context.m_capabilities.plsType
            == PixelLocalStorageType::VK_EXT_rasterization_order_attachment_access
    {
        let mut inputLayout = WGPUWagyuInputTextureBindingLayout {
            chain: super::webgpu_decl::WGPUChainedStruct {
                next: std::ptr::null_mut(),
                sType: WGPUSType_WagyuInputTextureBindingLayout,
            },
            viewDimension: WGPUTextureViewDimension_2D,
        };
        let entries: [WGPUBindGroupLayoutEntry; 4] = std::array::from_fn(|index| {
            let mut entry = WGPUBindGroupLayoutEntry::default();
            entry.nextInChain = &mut inputLayout.chain;
            entry.binding = index as u32;
            entry.visibility = fragment;
            entry
        });
        layouts[PLS_TEXTURE_BINDINGS_SET] = createLayout(&context.m_device, &entries);
    } else {
        layouts[PLS_TEXTURE_BINDINGS_SET] = (&*context.m_emptyBindingsLayout).clone();
    }

    let samplerEntries = [
        samplerLayoutEntry(GRAD_TEXTURE_IDX, fragment),
        samplerLayoutEntry(GAUSSIAN_INTEGRAL_TEXTURE_IDX, vertexFragment),
        samplerLayoutEntry(FEATHER_ATLAS_TEXTURE_IDX, fragment),
    ];
    layouts[WEBGPU_SAMPLER_BINDINGS_SET] = createLayout(&context.m_device, &samplerEntries);

    let rawLayouts: [WGPUBindGroupLayout; WEBGPU_BINDINGS_SET_COUNT] =
        std::array::from_fn(|index| layouts[index].Get());
    let mut descriptor = WGPUPipelineLayoutDescriptor::default();
    descriptor.bindGroupLayoutCount = rawLayouts.len();
    descriptor.bindGroupLayouts = rawLayouts.as_ptr();
    let pipelineLayout = unsafe { context.m_device.CreatePipelineLayout(&descriptor) };
    DrawPipelineLayout {
        m_perFlushBindingLayoutEntries: perFlushEntries,
        m_bindGroupLayouts: std::mem::ManuallyDrop::new(layouts),
        m_pipelineLayout: std::mem::ManuallyDrop::new(pipelineLayout),
    }
}

pub(crate) fn drawPipelineLayout(
    context: &mut RenderContextWebGPUImpl,
    mode: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode,
) -> &DrawPipelineLayout {
    let index = mode as usize;
    assert!(index < INTERLOCK_MODE_COUNT);
    if context.m_drawPipelineLayouts[index].is_none() {
        let layout = newDrawPipelineLayout(context, mode);
        context.m_drawPipelineLayouts[index] = Some(Box::new(layout));
    }
    context.m_drawPipelineLayouts[index]
        .as_deref()
        .expect("draw pipeline layout")
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn makeDrawPipeline(
    context: &mut RenderContextWebGPUImpl,
    drawType: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawType,
    shaderFeatures: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::ShaderFeatures,
    interlockMode: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode,
    shaderMiscFlags: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::ShaderMiscFlags,
    framebufferFormat: TextureFormat,
    vertexShaderModule: ShaderModule,
    fragmentShaderModule: ShaderModule,
    vertexShaderSource: Option<&str>,
    fragmentShaderSource: Option<&str>,
    pipelineState: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PipelineState,
) -> super::webgpu_cpp_decl::RenderPipeline {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        BlendEquation, DrawType, ImageDrawInstance, ImageRectVertex, InterlockMode, PatchVertex,
        ShaderFeatures, ShaderMiscFlags, TriangleVertex,
    };

    let mut attributes = Vec::<WGPUVertexAttribute>::new();
    let mut vertexBufferLayouts = Vec::<WGPUVertexBufferLayout>::new();
    let topology;
    match drawType {
        DrawType::midpointFanPatches
        | DrawType::midpointFanCenterAAPatches
        | DrawType::outerCurvePatches
        | DrawType::msaaOuterCubics
        | DrawType::msaaStrokes
        | DrawType::msaaMidpointFanBorrowedCoverage
        | DrawType::msaaDynamicMidpointFans
        | DrawType::msaaMidpointFans
        | DrawType::msaaMidpointFanStencilReset
        | DrawType::msaaMidpointFanPathsStencil
        | DrawType::msaaMidpointFanPathsCover => {
            for index in 0..2 {
                let mut attribute = WGPUVertexAttribute::default();
                attribute.format = super::webgpu_cpp_decl::VertexFormat::Float32x4.into();
                attribute.offset = (index * 4 * std::mem::size_of::<f32>()) as u64;
                attribute.shaderLocation = index as u32;
                attributes.push(attribute);
            }
            let mut layout = WGPUVertexBufferLayout::default();
            layout.attributeCount = attributes.len();
            layout.attributes = attributes.as_ptr();
            layout.arrayStride = std::mem::size_of::<PatchVertex>() as u64;
            layout.stepMode = super::webgpu_cpp_decl::VertexStepMode::Vertex.into();
            vertexBufferLayouts.push(layout);
            topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleList;
        }
        DrawType::clipReset | DrawType::interiorTriangulation | DrawType::featherAtlasBlit => {
            let mut attribute = WGPUVertexAttribute::default();
            attribute.format = super::webgpu_cpp_decl::VertexFormat::Float32x3.into();
            attributes.push(attribute);
            let mut layout = WGPUVertexBufferLayout::default();
            layout.attributeCount = 1;
            layout.attributes = attributes.as_ptr();
            layout.arrayStride = std::mem::size_of::<TriangleVertex>() as u64;
            layout.stepMode = super::webgpu_cpp_decl::VertexStepMode::Vertex.into();
            vertexBufferLayouts.push(layout);
            topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleList;
        }
        DrawType::imageRect => {
            let mut position = WGPUVertexAttribute::default();
            position.format = super::webgpu_cpp_decl::VertexFormat::Float32x4.into();
            attributes.push(position);
            appendImageDrawInstanceAttribs(&mut attributes);
            assert_eq!(attributes.len(), 1 + IMAGE_ATTRIB_COUNT);
            let mut vertices = WGPUVertexBufferLayout::default();
            vertices.attributeCount = 1;
            vertices.attributes = attributes.as_ptr();
            vertices.arrayStride = std::mem::size_of::<ImageRectVertex>() as u64;
            vertices.stepMode = super::webgpu_cpp_decl::VertexStepMode::Vertex.into();
            let mut instances = WGPUVertexBufferLayout::default();
            instances.attributeCount = IMAGE_ATTRIB_COUNT;
            instances.attributes = unsafe { attributes.as_ptr().add(1) };
            instances.arrayStride = std::mem::size_of::<ImageDrawInstance>() as u64;
            instances.stepMode = super::webgpu_cpp_decl::VertexStepMode::Instance.into();
            vertexBufferLayouts.extend([vertices, instances]);
            topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleList;
        }
        DrawType::imageMesh => {
            for location in 0..2 {
                let mut attribute = WGPUVertexAttribute::default();
                attribute.format = super::webgpu_cpp_decl::VertexFormat::Float32x2.into();
                attribute.shaderLocation = location;
                attributes.push(attribute);
            }
            appendImageDrawInstanceAttribs(&mut attributes);
            assert_eq!(attributes.len(), 2 + IMAGE_ATTRIB_COUNT);
            for index in 0..2 {
                let mut layout = WGPUVertexBufferLayout::default();
                layout.attributeCount = 1;
                layout.attributes = unsafe { attributes.as_ptr().add(index) };
                layout.arrayStride = (2 * std::mem::size_of::<f32>()) as u64;
                layout.stepMode = super::webgpu_cpp_decl::VertexStepMode::Vertex.into();
                vertexBufferLayouts.push(layout);
            }
            let mut instances = WGPUVertexBufferLayout::default();
            instances.attributeCount = IMAGE_ATTRIB_COUNT;
            instances.attributes = unsafe { attributes.as_ptr().add(2) };
            instances.arrayStride = std::mem::size_of::<ImageDrawInstance>() as u64;
            instances.stepMode = super::webgpu_cpp_decl::VertexStepMode::Instance.into();
            vertexBufferLayouts.push(instances);
            topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleList;
        }
        DrawType::renderPassInitialize | DrawType::renderPassResolve => {
            topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleStrip;
        }
    }

    let usingPLSInputAttachments = matches!(
        interlockMode,
        InterlockMode::rasterOrdering | InterlockMode::clockwise
    ) && context.m_capabilities.plsType
        == PixelLocalStorageType::VK_EXT_rasterization_order_attachment_access;
    let mut wagyuColorTargetState = super::webgpu_wagyu_decl::WGPUWagyuColorTargetState {
        chain: super::webgpu_decl::WGPUChainedStruct {
            next: std::ptr::null_mut(),
            sType: super::webgpu_wagyu_decl::WGPUSType_WagyuColorTargetState,
        },
        usedAsInput: super::webgpu_decl::WGPUOptionalBool_True,
    };
    let extraColorTargetState = if usingPLSInputAttachments {
        &mut wagyuColorTargetState.chain as *mut _
    } else {
        std::ptr::null_mut()
    };

    let makeBlendComponent = || {
        let mut component = super::webgpu_decl::WGPUBlendComponent::default();
        component.operation = wgpuBlendOperation(pipelineState.blendEquation);
        component.srcFactor = super::webgpu_cpp_decl::BlendFactor::One.into();
        component.dstFactor = wgpuDstBlendFactor(pipelineState.blendEquation);
        component
    };
    let blendState = WGPUBlendState {
        color: makeBlendComponent(),
        alpha: makeBlendComponent(),
    };
    let mut colorAttachments = Vec::<WGPUColorTargetState>::with_capacity(PLS_PLANE_COUNT);
    let mut color = WGPUColorTargetState::default();
    color.nextInChain = extraColorTargetState;
    color.format = framebufferFormat.into();
    color.blend = if pipelineState.blendEquation == BlendEquation::none {
        std::ptr::null()
    } else {
        &blendState
    };
    color.writeMask = if pipelineState.colorWriteEnabled {
        super::webgpu_decl::WGPUColorWriteMask_All
    } else {
        super::webgpu_decl::WGPUColorWriteMask_None
    };
    colorAttachments.push(color);
    if usingPLSInputAttachments {
        for format in [
            TextureFormat::R32Uint,
            framebufferFormat,
            TextureFormat::R32Uint,
        ] {
            let mut attachment = WGPUColorTargetState::default();
            attachment.nextInChain = extraColorTargetState;
            attachment.format = format.into();
            attachment.writeMask = super::webgpu_decl::WGPUColorWriteMask_All;
            colorAttachments.push(attachment);
        }
        assert_eq!(SCRATCH_COLOR_PLANE_IDX, 2);
        assert_eq!(colorAttachments.len(), PLS_PLANE_COUNT);
    }

    let hasFeature = |feature: ShaderFeatures| shaderFeatures.0 & feature.0 != 0;
    let hasMisc = |flag: ShaderMiscFlags| shaderMiscFlags.0 & flag.0 != 0;
    let permutationFlags = [
        hasFeature(ShaderFeatures::ENABLE_CLIPPING) as u8 as f64,
        hasFeature(ShaderFeatures::ENABLE_CLIP_RECT) as u8 as f64,
        hasFeature(ShaderFeatures::ENABLE_ADVANCED_BLEND) as u8 as f64,
        hasFeature(ShaderFeatures::ENABLE_FEATHER) as u8 as f64,
        hasFeature(ShaderFeatures::ENABLE_EVEN_ODD) as u8 as f64,
        hasFeature(ShaderFeatures::ENABLE_NESTED_CLIPPING) as u8 as f64,
        hasFeature(ShaderFeatures::ENABLE_HSL_BLEND_MODES) as u8 as f64,
        hasFeature(ShaderFeatures::ENABLE_DITHER) as u8 as f64,
        hasMisc(ShaderMiscFlags::clockwiseFill) as u8 as f64,
        hasMisc(ShaderMiscFlags::nestedClipUpdateOnly) as u8 as f64,
        hasMisc(ShaderMiscFlags::borrowedCoveragePass) as u8 as f64,
        hasMisc(ShaderMiscFlags::storeColorClear) as u8 as f64,
        hasMisc(ShaderMiscFlags::loadColorFromDstTexture) as u8 as f64,
        0.0,
    ];
    let vertexConstants = buildConstantEntries(vertexShaderSource, &permutationFlags);
    let fragmentConstants = buildConstantEntries(fragmentShaderSource, &permutationFlags);

    let mut fragmentState = WGPUFragmentState::default();
    fragmentState.module = fragmentShaderModule.Get();
    fragmentState.entryPoint = stringView("main");
    fragmentState.constantCount = fragmentConstants.len();
    fragmentState.constants = fragmentConstants.as_ptr();
    fragmentState.targetCount = colorAttachments.len();
    fragmentState.targets = colorAttachments.as_ptr();
    let mut inputAttachments: [super::webgpu_wagyu_decl::WGPUWagyuInputAttachmentState;
        PLS_PLANE_COUNT] =
        std::array::from_fn(
            |index| super::webgpu_wagyu_decl::WGPUWagyuInputAttachmentState {
                format: colorAttachments[index].format,
                usedAsColor: super::webgpu_decl::WGPUOptionalBool_True,
            },
        );
    let mut wagyuFragmentState = super::webgpu_wagyu_decl::WGPUWagyuFragmentState {
        chain: super::webgpu_decl::WGPUChainedStruct {
            next: std::ptr::null_mut(),
            sType: super::webgpu_wagyu_decl::WGPUSType_WagyuFragmentState,
        },
        inputCount: PLS_PLANE_COUNT,
        inputs: inputAttachments.as_mut_ptr(),
        featureFlags: super::webgpu_wagyu_decl::WGPUWagyuFragmentStateFeaturesFlags_RasterizationOrderAttachmentAccess,
    };
    if usingPLSInputAttachments {
        fragmentState.nextInChain = &mut wagyuFragmentState.chain;
    }

    let label = format!(
        "RIVE_Draw{{drawType={},interlockMode={}}}",
        drawType as u8, interlockMode as u8
    );
    let pipelineLayout = drawPipelineLayout(context, interlockMode)
        .m_pipelineLayout
        .Get();
    let mut descriptor = WGPURenderPipelineDescriptor::default();
    descriptor.label = stringView(&label);
    descriptor.layout = pipelineLayout;
    descriptor.vertex.module = vertexShaderModule.Get();
    descriptor.vertex.entryPoint = stringView("main");
    descriptor.vertex.constantCount = vertexConstants.len();
    descriptor.vertex.constants = vertexConstants.as_ptr();
    descriptor.vertex.bufferCount = vertexBufferLayouts.len();
    descriptor.vertex.buffers = if vertexBufferLayouts.is_empty() {
        std::ptr::null()
    } else {
        vertexBufferLayouts.as_ptr()
    };
    descriptor.primitive.topology = topology.into();
    descriptor.primitive.frontFace = RIVE_FRONT_FACE.into();
    descriptor.primitive.cullMode = wgpuCullMode(pipelineState.cullFace);
    descriptor.multisample.count = if interlockMode == InterlockMode::msaa {
        MSAA_SAMPLE_COUNT
    } else {
        1
    };
    descriptor.multisample.mask = u32::MAX;
    descriptor.fragment = &fragmentState;

    let mut depthStencilState = super::webgpu_decl::WGPUDepthStencilState::default();
    if interlockMode == InterlockMode::msaa {
        depthStencilState.format = TextureFormat::Depth24PlusStencil8.into();
        depthStencilState.depthWriteEnabled = if pipelineState.depthWriteEnabled {
            super::webgpu_decl::WGPUOptionalBool_True
        } else {
            super::webgpu_decl::WGPUOptionalBool_False
        };
        depthStencilState.depthCompare = if pipelineState.depthTestEnabled {
            super::webgpu_cpp_decl::CompareFunction::Less.into()
        } else {
            super::webgpu_cpp_decl::CompareFunction::Always.into()
        };
        depthStencilState.stencilFront = if pipelineState.stencilTestEnabled {
            wgpuStencilFaceState(pipelineState.stencilFrontOps)
        } else {
            disabledStencilFaceState()
        };
        depthStencilState.stencilBack = if pipelineState.stencilTestEnabled {
            wgpuStencilFaceState(if pipelineState.stencilDoubleSided {
                pipelineState.stencilBackOps
            } else {
                pipelineState.stencilFrontOps
            })
        } else {
            disabledStencilFaceState()
        };
        depthStencilState.stencilReadMask = u32::from(pipelineState.stencilCompareMask);
        depthStencilState.stencilWriteMask = u32::from(pipelineState.stencilWriteMask);
        descriptor.depthStencil = &depthStencilState;
    }
    unsafe { context.m_device.CreateRenderPipeline(&descriptor) }
}

fn addGlslDefine(output: &mut String, name: &str) {
    output.push_str("#define ");
    output.push_str(name);
    output.push_str(" true\n");
}

#[allow(clippy::too_many_arguments)]
fn compilePLSDrawShaders(
    context: &RenderContextWebGPUImpl,
    drawType: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawType,
    shaderFeatures: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::ShaderFeatures,
    interlockMode: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode,
    shaderMiscFlags: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::ShaderMiscFlags,
    targetIsGLFBO0: bool,
) -> (ShaderModule, ShaderModule) {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        DrawType, InterlockMode, ShaderFeatures, ShaderMiscFlags,
    };

    assert!(matches!(
        interlockMode,
        InterlockMode::rasterOrdering | InterlockMode::clockwise
    ));
    let (language, version) = if context.m_capabilities.backendType == BackendType::OpenGLES {
        (
            super::webgpu_wagyu_decl::WGPUWagyuShaderLanguage_GLSLRAW,
            "#version 310 es",
        )
    } else {
        (
            super::webgpu_wagyu_decl::WGPUWagyuShaderLanguage_GLSL,
            "#version 460",
        )
    };
    let mut common = String::new();
    if context.m_capabilities.backendType == BackendType::OpenGLES {
        if !targetIsGLFBO0 {
            addGlslDefine(&mut common, GLSL_POST_INVERT_Y);
        }
        common.push_str(&format!(
            "#define {GLSL_BASE_INSTANCE_UNIFORM_NAME} {BASE_INSTANCE_UNIFORM_NAME}\n"
        ));
    } else {
        addGlslDefine(&mut common, GLSL_TARGET_SPIRV);
    }
    match context.m_capabilities.plsType {
        PixelLocalStorageType::GL_EXT_shader_pixel_local_storage => {
            common.push_str("#ifdef GL_EXT_shader_pixel_local_storage\n");
            addGlslDefine(&mut common, GLSL_PLS_IMPL_EXT_NATIVE);
            common.push_str("#else\n#extension GL_EXT_samplerless_texture_functions : enable\n");
            addGlslDefine(&mut common, GLSL_PLS_IMPL_NONE);
            common.push_str("#endif\n");
        }
        PixelLocalStorageType::VK_EXT_rasterization_order_attachment_access => {
            common.push_str("#extension GL_EXT_samplerless_texture_functions : enable\n");
            addGlslDefine(&mut common, GLSL_PLS_IMPL_SUBPASS_LOAD);
        }
        PixelLocalStorageType::none => {
            common.push_str("#extension GL_EXT_samplerless_texture_functions : enable\n");
            addGlslDefine(&mut common, GLSL_PLS_IMPL_NONE);
        }
    }
    if context.m_capabilities.polyfillVertexStorageBuffers {
        addGlslDefine(&mut common, GLSL_DISABLE_SHADER_STORAGE_BUFFERS);
    }
    match drawType {
        DrawType::midpointFanPatches
        | DrawType::midpointFanCenterAAPatches
        | DrawType::outerCurvePatches => {
            addGlslDefine(&mut common, GLSL_DRAW_PATH);
            addGlslDefine(&mut common, GLSL_ENABLE_INSTANCE_INDEX);
        }
        DrawType::interiorTriangulation => addGlslDefine(&mut common, GLSL_DRAW_INTERIOR_TRIANGLES),
        DrawType::featherAtlasBlit => addGlslDefine(&mut common, GLSL_FEATHER_ATLAS_BLIT),
        DrawType::imageMesh => {
            addGlslDefine(&mut common, GLSL_DRAW_IMAGE);
            addGlslDefine(&mut common, GLSL_DRAW_IMAGE_MESH);
        }
        DrawType::imageRect => {
            addGlslDefine(&mut common, GLSL_DRAW_IMAGE);
            addGlslDefine(&mut common, GLSL_DRAW_IMAGE_RECT);
            unreachable!("imageRect is not a PLS draw type in the pinned Wagyu source")
        }
        _ => unreachable!("MSAA/control draw type in a PLS shader family"),
    }
    for (feature, name) in [
        (ShaderFeatures::ENABLE_CLIPPING, GLSL_ENABLE_CLIPPING),
        (ShaderFeatures::ENABLE_CLIP_RECT, GLSL_ENABLE_CLIP_RECT),
        (
            ShaderFeatures::ENABLE_ADVANCED_BLEND,
            GLSL_ENABLE_ADVANCED_BLEND,
        ),
        (ShaderFeatures::ENABLE_FEATHER, GLSL_ENABLE_FEATHER),
        (ShaderFeatures::ENABLE_EVEN_ODD, GLSL_ENABLE_EVEN_ODD),
        (
            ShaderFeatures::ENABLE_NESTED_CLIPPING,
            GLSL_ENABLE_NESTED_CLIPPING,
        ),
        (
            ShaderFeatures::ENABLE_HSL_BLEND_MODES,
            GLSL_ENABLE_HSL_BLEND_MODES,
        ),
        (ShaderFeatures::ENABLE_DITHER, GLSL_ENABLE_DITHER),
    ] {
        if shaderFeatures.0 & feature.0 != 0 {
            addGlslDefine(&mut common, name);
        }
    }
    if shaderMiscFlags.has(ShaderMiscFlags::fixedFunctionColorOutput) {
        addGlslDefine(&mut common, GLSL_FIXED_FUNCTION_COLOR_OUTPUT);
    }
    if shaderMiscFlags.has(ShaderMiscFlags::clockwiseFill) {
        addGlslDefine(&mut common, GLSL_CLOCKWISE_FILL);
    }
    if shaderMiscFlags.has(ShaderMiscFlags::borrowedCoveragePass) {
        addGlslDefine(&mut common, GLSL_BORROWED_COVERAGE_PASS);
    }
    appendGlslParts(
        &mut common,
        &[GLSL_GLSL, GLSL_CONSTANTS, GLSL_FLUSH_UNIFORMS, GLSL_COMMON],
    );
    if shaderFeatures.0 & ShaderFeatures::ENABLE_ADVANCED_BLEND.0 != 0 {
        common.push_str(GLSL_ADVANCED_BLEND);
        common.push('\n');
    }
    common.push_str("#define ");
    common.push_str(GLSL_OPTIONALLY_FLAT);
    if !context.platformFeatures().avoidFlatVaryings {
        common.push_str(" flat");
    }
    common.push('\n');
    match drawType {
        DrawType::midpointFanPatches
        | DrawType::midpointFanCenterAAPatches
        | DrawType::outerCurvePatches
        | DrawType::interiorTriangulation => {
            appendGlslParts(&mut common, &[GLSL_DRAW_PATH_COMMON, GLSL_DRAW_PATH_VERT]);
            common.push_str(if interlockMode == InterlockMode::rasterOrdering {
                GLSL_DRAW_RASTER_ORDER_PATH_FRAG
            } else if shaderMiscFlags.has(ShaderMiscFlags::clipUpdateOnly) {
                GLSL_DRAW_CLOCKWISE_CLIP_FRAG
            } else {
                GLSL_DRAW_CLOCKWISE_PATH_FRAG
            });
        }
        DrawType::featherAtlasBlit => {
            appendGlslParts(
                &mut common,
                &[
                    GLSL_DRAW_PATH_COMMON,
                    GLSL_DRAW_PATH_VERT,
                    GLSL_DRAW_MESH_FRAG,
                ],
            );
        }
        DrawType::imageMesh => {
            appendGlslParts(
                &mut common,
                &[GLSL_DRAW_IMAGE_MESH_VERT, GLSL_DRAW_MESH_FRAG],
            );
        }
        _ => unreachable!("non-PLS draw type reached PLS shader composition"),
    }
    let vertex =
        format!("{version}\n#pragma shader_stage(vertex)\n#define {GLSL_VERTEX} true\n{common}");
    let fragment = format!(
        "{version}\n#pragma shader_stage(fragment)\n#define {GLSL_FRAGMENT} true\n{common}"
    );
    (
        compileShaderModuleWagyu(&context.m_device, &vertex, language),
        compileShaderModuleWagyu(&context.m_device, &fragment, language),
    )
}

fn compileWGSLDrawShader(device: &Device, source: &str, label: &str) -> ShaderModule {
    compileShaderModuleWGSL(device, source, label)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn newDrawPipeline(
    context: &mut RenderContextWebGPUImpl,
    drawType: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawType,
    shaderFeatures: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::ShaderFeatures,
    interlockMode: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode,
    shaderMiscFlags: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::ShaderMiscFlags,
    pipelineState: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PipelineState,
    targetIsGLFBO0: bool,
) -> super::render_context_webgpu_decl::DrawPipeline {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        DrawType, InterlockMode, ShaderFeatures, ShaderMiscFlags,
    };

    let fixedColor = shaderMiscFlags.has(ShaderMiscFlags::fixedFunctionColorOutput);
    let (vertexModule, fragmentModule, vertexSource, fragmentSource) = match interlockMode {
        InterlockMode::rasterOrdering | InterlockMode::clockwise => {
            let (vertex, fragment) = compilePLSDrawShaders(
                context,
                drawType,
                shaderFeatures,
                interlockMode,
                shaderMiscFlags,
                targetIsGLFBO0,
            );
            (vertex, fragment, None, None)
        }
        InterlockMode::atomics => {
            let (vertex, fragment, vertexLabel, fragmentLabel) = match drawType {
                DrawType::midpointFanPatches
                | DrawType::midpointFanCenterAAPatches
                | DrawType::outerCurvePatches => (
                    include_str!("../../generated/atomic_draw_path.webgpu_vert.wgsl"),
                    if fixedColor {
                        include_str!("../../generated/atomic_draw_path.webgpu_fixedcolor_frag.wgsl")
                    } else {
                        include_str!("../../generated/atomic_draw_path.webgpu_frag.wgsl")
                    },
                    "atomic_draw_path.webgpu.vert",
                    "atomic_draw_path.webgpu.frag",
                ),
                DrawType::interiorTriangulation => (
                    include_str!("../../generated/atomic_draw_interior_triangles.webgpu_vert.wgsl"),
                    if fixedColor {
                        include_str!("../../generated/atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl")
                    } else {
                        include_str!(
                            "../../generated/atomic_draw_interior_triangles.webgpu_frag.wgsl"
                        )
                    },
                    "atomic_draw_interior_triangles.webgpu.vert",
                    "atomic_draw_interior_triangles.webgpu.frag",
                ),
                DrawType::featherAtlasBlit => {
                    (
                        include_str!("../../generated/atomic_draw_atlas_blit.webgpu_vert.wgsl"),
                        if fixedColor {
                            include_str!("../../generated/atomic_draw_atlas_blit.webgpu_fixedcolor_frag.wgsl")
                        } else {
                            include_str!("../../generated/atomic_draw_atlas_blit.webgpu_frag.wgsl")
                        },
                        "atomic_draw_atlas_blit.webgpu.vert",
                        "atomic_draw_atlas_blit.webgpu.frag",
                    )
                }
                DrawType::imageRect => {
                    (
                        include_str!("../../generated/atomic_draw_image_rect.webgpu_vert.wgsl"),
                        if fixedColor {
                            include_str!("../../generated/atomic_draw_image_rect.webgpu_fixedcolor_frag.wgsl")
                        } else {
                            include_str!("../../generated/atomic_draw_image_rect.webgpu_frag.wgsl")
                        },
                        "atomic_draw_image_rect.webgpu.vert",
                        "atomic_draw_image_rect.webgpu.frag",
                    )
                }
                DrawType::imageMesh => {
                    (
                        include_str!("../../generated/atomic_draw_image_mesh.webgpu_vert.wgsl"),
                        if fixedColor {
                            include_str!("../../generated/atomic_draw_image_mesh.webgpu_fixedcolor_frag.wgsl")
                        } else {
                            include_str!("../../generated/atomic_draw_image_mesh.webgpu_frag.wgsl")
                        },
                        "atomic_draw_image_mesh.webgpu.vert",
                        "atomic_draw_image_mesh.webgpu.frag",
                    )
                }
                DrawType::renderPassResolve if fixedColor => (
                    include_str!("../../generated/atomic_resolve.webgpu_vert.wgsl"),
                    include_str!("../../generated/atomic_resolve.webgpu_fixedcolor_frag.wgsl"),
                    "atomic_resolve.webgpu.vert",
                    "atomic_resolve.webgpu.frag",
                ),
                DrawType::renderPassResolve => {
                    assert!(shaderMiscFlags.has(ShaderMiscFlags::coalescedResolveAndTransfer));
                    (
                        include_str!("../../generated/atomic_resolve_coalesced.webgpu_vert.wgsl"),
                        include_str!("../../generated/atomic_resolve_coalesced.webgpu_frag.wgsl"),
                        "atomic_resolve_coalesced.webgpu.vert",
                        "atomic_resolve_coalesced.webgpu.frag",
                    )
                }
                DrawType::renderPassInitialize => (
                    include_str!("../../generated/atomic_init.webgpu_vert.wgsl"),
                    if fixedColor {
                        include_str!("../../generated/atomic_init.webgpu_fixedcolor_frag.wgsl")
                    } else {
                        include_str!("../../generated/atomic_init.webgpu_frag.wgsl")
                    },
                    "atomic_init.webgpu.vert",
                    "atomic_init.webgpu.frag",
                ),
                _ => unreachable!("non-atomic draw type in atomic shader selection"),
            };
            (
                compileWGSLDrawShader(&context.m_device, vertex, vertexLabel),
                compileWGSLDrawShader(&context.m_device, fragment, fragmentLabel),
                Some(vertex),
                Some(fragment),
            )
        }
        InterlockMode::msaa => {
            let clipRect = shaderFeatures.0 & ShaderFeatures::ENABLE_CLIP_RECT.0 != 0;
            let (vertex, fragment, vertexLabel, fragmentLabel) = match drawType {
                DrawType::msaaOuterCubics
                | DrawType::msaaStrokes
                | DrawType::msaaMidpointFanBorrowedCoverage
                | DrawType::msaaDynamicMidpointFans
                | DrawType::msaaMidpointFans
                | DrawType::msaaMidpointFanStencilReset
                | DrawType::msaaMidpointFanPathsStencil
                | DrawType::msaaMidpointFanPathsCover => {
                    let vertex = match (
                        context.m_capabilities.polyfillVertexStorageBuffers,
                        clipRect,
                    ) {
                        (true, true) => {
                            include_str!("../../generated/draw_msaa_path.webgpu_nossbo_vert.wgsl")
                        }
                        (true, false) => include_str!(
                            "../../generated/draw_msaa_path.webgpu_nossbo_noclipdistance_vert.wgsl"
                        ),
                        (false, true) => {
                            include_str!("../../generated/draw_msaa_path.webgpu_vert.wgsl")
                        }
                        (false, false) => include_str!(
                            "../../generated/draw_msaa_path.webgpu_noclipdistance_vert.wgsl"
                        ),
                    };
                    (
                        vertex,
                        if fixedColor {
                            include_str!(
                                "../../generated/draw_msaa_path.webgpu_fixedcolor_frag.wgsl"
                            )
                        } else {
                            include_str!("../../generated/draw_msaa_path.webgpu_frag.wgsl")
                        },
                        "draw_msaa_path.webgpu.vert",
                        "draw_msaa_path.webgpu.frag",
                    )
                }
                DrawType::clipReset => (
                    include_str!("../../generated/draw_msaa_stencil.vert.wgsl"),
                    include_str!("../../generated/draw_msaa_stencil.frag.wgsl"),
                    "draw_msaa_stencil.vert",
                    "draw_msaa_stencil.frag",
                ),
                DrawType::featherAtlasBlit => {
                    let vertex = match (context.m_capabilities.polyfillVertexStorageBuffers, clipRect) {
                        (true, true) => include_str!("../../generated/draw_msaa_atlas_blit.webgpu_nossbo_vert.wgsl"),
                        (true, false) => include_str!("../../generated/draw_msaa_atlas_blit.webgpu_nossbo_noclipdistance_vert.wgsl"),
                        (false, true) => include_str!("../../generated/draw_msaa_atlas_blit.webgpu_vert.wgsl"),
                        (false, false) => include_str!("../../generated/draw_msaa_atlas_blit.webgpu_noclipdistance_vert.wgsl"),
                    };
                    (
                        vertex,
                        if fixedColor {
                            include_str!(
                                "../../generated/draw_msaa_atlas_blit.webgpu_fixedcolor_frag.wgsl"
                            )
                        } else {
                            include_str!("../../generated/draw_msaa_atlas_blit.webgpu_frag.wgsl")
                        },
                        "draw_msaa_atlas_blit.webgpu.vert",
                        "draw_msaa_atlas_blit.webgpu.frag",
                    )
                }
                DrawType::imageMesh => {
                    (
                        if clipRect {
                            include_str!("../../generated/draw_msaa_image_mesh.webgpu_vert.wgsl")
                        } else {
                            include_str!("../../generated/draw_msaa_image_mesh.webgpu_noclipdistance_vert.wgsl")
                        },
                        if fixedColor {
                            include_str!(
                                "../../generated/draw_msaa_image_mesh.webgpu_fixedcolor_frag.wgsl"
                            )
                        } else {
                            include_str!("../../generated/draw_msaa_image_mesh.webgpu_frag.wgsl")
                        },
                        "draw_msaa_image_mesh.webgpu.vert",
                        "draw_msaa_image_mesh.webgpu.frag",
                    )
                }
                DrawType::renderPassInitialize => (
                    include_str!("../../generated/blit_texture_as_draw_filtered.webgpu_vert.wgsl"),
                    include_str!("../../generated/blit_texture_as_draw_filtered.webgpu_frag.wgsl"),
                    "blit_texture_as_draw_filtered.webgpu.vert",
                    "blit_texture_as_draw_filtered.webgpu.frag",
                ),
                _ => unreachable!("unsupported draw type in MSAA shader selection"),
            };
            (
                compileWGSLDrawShader(&context.m_device, vertex, vertexLabel),
                compileWGSLDrawShader(&context.m_device, fragment, fragmentLabel),
                Some(vertex),
                Some(fragment),
            )
        }
        InterlockMode::clockwiseAtomic => {
            unreachable!("clockwiseAtomic is not a WebGPU interlock mode")
        }
    };

    let mut renderPipelines =
        std::array::from_fn(|_| super::webgpu_cpp_decl::RenderPipeline::default());
    for framebufferFormat in [TextureFormat::RGBA8Unorm, TextureFormat::BGRA8Unorm] {
        let index = if framebufferFormat == TextureFormat::BGRA8Unorm {
            1
        } else {
            0
        };
        renderPipelines[index] = makeDrawPipeline(
            context,
            drawType,
            shaderFeatures,
            interlockMode,
            shaderMiscFlags,
            framebufferFormat,
            vertexModule.clone(),
            fragmentModule.clone(),
            vertexSource,
            fragmentSource,
            pipelineState,
        );
    }
    super::render_context_webgpu_decl::DrawPipeline {
        m_renderPipelines: std::mem::ManuallyDrop::new(renderPipelines),
    }
}

pub(crate) fn drawPipelineForFormat(
    pipeline: &super::render_context_webgpu_decl::DrawPipeline,
    framebufferFormat: TextureFormat,
) -> super::webgpu_cpp_decl::RenderPipeline {
    assert!(matches!(
        framebufferFormat,
        TextureFormat::BGRA8Unorm | TextureFormat::RGBA8Unorm
    ));
    pipeline.m_renderPipelines[usize::from(framebufferFormat == TextureFormat::BGRA8Unorm)].clone()
}

fn wgpuColorPremul(color: nuxie_render_api::ColorInt) -> super::webgpu_decl::WGPUColor {
    let alpha = ((color >> 24) & 0xff) as f64 / 255.0;
    super::webgpu_decl::WGPUColor {
        r: ((color >> 16) & 0xff) as f64 / 255.0 * alpha,
        g: ((color >> 8) & 0xff) as f64 / 255.0 * alpha,
        b: (color & 0xff) as f64 / 255.0 * alpha,
        a: alpha,
    }
}

trait DrawRenderPassApi {
    fn encoder(&self) -> &super::webgpu_cpp_decl::RenderPassEncoder;
    fn barrier(
        &mut self,
        batch: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawBatch,
    );
    fn end(&mut self);
}

struct DrawRenderPassBase {
    m_impl: *mut RenderContextWebGPUImpl,
    m_desc: *const crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
    m_renderTarget: *mut RenderTargetWebGPU,
    m_commandEncoder: std::mem::ManuallyDrop<CommandEncoder>,
    m_encoder: std::mem::ManuallyDrop<super::webgpu_cpp_decl::RenderPassEncoder>,
}

impl DrawRenderPassBase {
    unsafe fn new(
        implementation: *mut RenderContextWebGPUImpl,
        descriptor: *const crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
        commandEncoder: CommandEncoder,
    ) -> Self {
        let renderTarget = unsafe {
            (*descriptor)
                .renderTarget
                .expect("flush renderTarget")
                .as_ptr()
                .cast::<RenderTargetWebGPU>()
        };
        Self {
            m_impl: implementation,
            m_desc: descriptor,
            m_renderTarget: renderTarget,
            m_commandEncoder: std::mem::ManuallyDrop::new(commandEncoder),
            m_encoder: std::mem::ManuallyDrop::new(
                super::webgpu_cpp_decl::RenderPassEncoder::default(),
            ),
        }
    }

    fn end(&mut self) {
        if !self.m_encoder.Get().is_null() {
            unsafe { self.m_encoder.End() };
            *self.m_encoder = super::webgpu_cpp_decl::RenderPassEncoder::default();
        }
    }

    unsafe fn initDrawRenderPass(&self) {
        let target = unsafe { &*self.m_renderTarget };
        let implementation = unsafe { &*self.m_impl };
        unsafe {
            self.m_encoder.SetViewport(
                0.0,
                0.0,
                target.width() as f32,
                target.height() as f32,
                0.0,
                1.0,
            );
            self.m_encoder.SetBindGroup(
                WEBGPU_SAMPLER_BINDINGS_SET as u32,
                implementation.m_samplerBindings.Get(),
                0,
                std::ptr::null(),
            );
        }
    }
}

impl Drop for DrawRenderPassBase {
    fn drop(&mut self) {
        self.end();
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.m_encoder);
            std::mem::ManuallyDrop::drop(&mut self.m_commandEncoder);
        }
    }
}

struct PLSDrawRenderPass {
    base: DrawRenderPassBase,
}

impl PLSDrawRenderPass {
    unsafe fn new(
        implementation: *mut RenderContextWebGPUImpl,
        descriptor: *const crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
        commandEncoder: CommandEncoder,
    ) -> Self {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::LoadAction;
        let mut pass = Self {
            base: unsafe { DrawRenderPassBase::new(implementation, descriptor, commandEncoder) },
        };
        let desc = unsafe { &*descriptor };
        let target = unsafe { &mut *pass.base.m_renderTarget };
        let colorLoadOp = if desc.colorLoadAction == LoadAction::preserveRenderTarget {
            super::webgpu_cpp_decl::LoadOp::Load
        } else {
            super::webgpu_cpp_decl::LoadOp::Clear
        };
        let mut targetClear = wgpuColorPremul(desc.colorClearValue);
        let mut zeroClear = super::webgpu_decl::WGPUColor::default();
        let mut plsAttachments = Vec::with_capacity(PLS_PLANE_COUNT);
        let mut framebuffer = super::webgpu_decl::WGPURenderPassColorAttachment::default();
        framebuffer.view = target.targetTextureView().Get();
        framebuffer.depthSlice = super::webgpu_decl::WGPU_DEPTH_SLICE_UNDEFINED;
        framebuffer.loadOp = colorLoadOp.into();
        framebuffer.storeOp = super::webgpu_cpp_decl::StoreOp::Store.into();
        framebuffer.clearValue = wgpuColorPremul(desc.colorClearValue);
        plsAttachments.push(framebuffer);

        let implementation = unsafe { &*implementation };
        let mut inputAttachments = Vec::with_capacity(PLS_PLANE_COUNT);
        let mut wagyuDescriptor = super::webgpu_wagyu_decl::WGPUWagyuRenderPassDescriptor {
            chain: super::webgpu_decl::WGPUChainedStruct {
                next: std::ptr::null_mut(),
                sType: super::webgpu_wagyu_decl::WGPUSType_WagyuRenderPassDescriptor,
            },
            inputAttachmentCount: 0,
            inputAttachments: std::ptr::null_mut(),
            pixelLocalStorageEnabled: super::webgpu_decl::WGPUOptionalBool_Undefined,
            pixelLocalStorageSize:
                super::webgpu_wagyu_decl::WGPU_WAGYU_PIXEL_LOCAL_STORAGE_SIZE_UNDEFINED,
        };
        if implementation.m_capabilities.plsType
            == PixelLocalStorageType::VK_EXT_rasterization_order_attachment_access
        {
            inputAttachments.push(
                super::webgpu_wagyu_decl::WGPUWagyuRenderPassInputAttachment {
                    view: target.targetTextureView().Get(),
                    clearValue: &mut targetClear,
                    loadOp: colorLoadOp.into(),
                    storeOp: super::webgpu_cpp_decl::StoreOp::Store.into(),
                },
            );
            for view in [
                clipTextureView(target),
                scratchColorTextureView(target),
                coverageTextureView(target),
            ] {
                let mut attachment = super::webgpu_decl::WGPURenderPassColorAttachment::default();
                attachment.view = view.Get();
                attachment.depthSlice = super::webgpu_decl::WGPU_DEPTH_SLICE_UNDEFINED;
                attachment.loadOp = super::webgpu_cpp_decl::LoadOp::Clear.into();
                attachment.storeOp = super::webgpu_cpp_decl::StoreOp::Discard.into();
                plsAttachments.push(attachment);
                inputAttachments.push(
                    super::webgpu_wagyu_decl::WGPUWagyuRenderPassInputAttachment {
                        view: view.Get(),
                        clearValue: &mut zeroClear,
                        loadOp: super::webgpu_cpp_decl::LoadOp::Clear.into(),
                        storeOp: super::webgpu_cpp_decl::StoreOp::Discard.into(),
                    },
                );
            }
            assert_eq!(plsAttachments.len(), PLS_PLANE_COUNT);
            assert_eq!(inputAttachments.len(), PLS_PLANE_COUNT);
            wagyuDescriptor.inputAttachmentCount = inputAttachments.len();
            wagyuDescriptor.inputAttachments = inputAttachments.as_mut_ptr();
        } else if implementation.m_capabilities.plsType
            == PixelLocalStorageType::GL_EXT_shader_pixel_local_storage
        {
            wagyuDescriptor.pixelLocalStorageEnabled = super::webgpu_decl::WGPUOptionalBool_True;
            if desc.fixedFunctionColorOutput {
                assert!(
                    implementation
                        .m_capabilities
                        .GL_EXT_shader_pixel_local_storage2
                );
                wagyuDescriptor.pixelLocalStorageSize = 2 * std::mem::size_of::<u32>() as u32;
            }
        }
        let mut passDescriptor = super::webgpu_decl::WGPURenderPassDescriptor::default();
        passDescriptor.label = stringView("RIVE_PLS_RenderPass");
        passDescriptor.nextInChain = &mut wagyuDescriptor.chain;
        passDescriptor.colorAttachmentCount = plsAttachments.len();
        passDescriptor.colorAttachments = plsAttachments.as_ptr();
        *pass.base.m_encoder =
            unsafe { pass.base.m_commandEncoder.BeginRenderPass(&passDescriptor) };
        unsafe { pass.base.initDrawRenderPass() };
        pass
    }
}

impl DrawRenderPassApi for PLSDrawRenderPass {
    fn encoder(&self) -> &super::webgpu_cpp_decl::RenderPassEncoder {
        &self.base.m_encoder
    }
    fn barrier(
        &mut self,
        _batch: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawBatch,
    ) {
        unreachable!("PLS dependencies resolve in hardware")
    }
    fn end(&mut self) {
        self.base.end()
    }
}

struct AtomicDrawRenderPass {
    base: std::mem::ManuallyDrop<DrawRenderPassBase>,
    m_plsBindings: std::mem::ManuallyDrop<BindGroup>,
}

impl AtomicDrawRenderPass {
    unsafe fn new(
        implementation: *mut RenderContextWebGPUImpl,
        descriptor: *const crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
        commandEncoder: CommandEncoder,
    ) -> Self {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawType;
        let mut pass = Self {
            base: std::mem::ManuallyDrop::new(unsafe {
                DrawRenderPassBase::new(implementation, descriptor, commandEncoder)
            }),
            m_plsBindings: std::mem::ManuallyDrop::new(BindGroup::default()),
        };
        let desc = unsafe { &*descriptor };
        let implementation = unsafe { &mut *implementation };
        let color = if desc.fixedFunctionColorOutput {
            (&*implementation.m_nullStorageBuffer).clone()
        } else {
            atomicPLSColorBuffer(implementation)
        };
        let buffers = [
            color,
            atomicPLSClipBuffer(implementation),
            atomicPLSCoverageBuffer(implementation),
        ];
        let entries: [WGPUBindGroupEntry; 3] = std::array::from_fn(|index| {
            let mut entry = WGPUBindGroupEntry::default();
            entry.binding = [COLOR_PLANE_IDX, CLIP_PLANE_IDX, COVERAGE_PLANE_IDX][index];
            entry.buffer = buffers[index].Get();
            entry
        });
        let layout = drawPipelineLayout(implementation, desc.interlockMode).m_bindGroupLayouts
            [PLS_TEXTURE_BINDINGS_SET]
            .Get();
        let mut bindGroupDescriptor = WGPUBindGroupDescriptor::default();
        bindGroupDescriptor.layout = layout;
        bindGroupDescriptor.entryCount = entries.len();
        bindGroupDescriptor.entries = entries.as_ptr();
        *pass.m_plsBindings = unsafe {
            implementation
                .m_device
                .CreateBindGroup(&bindGroupDescriptor)
        };
        unsafe { pass.begin(DrawType::renderPassInitialize) };
        pass
    }

    unsafe fn begin(
        &mut self,
        drawType: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawType,
    ) {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
            DrawType, LoadAction,
        };
        let desc = unsafe { &*self.base.m_desc };
        let target = unsafe { &mut *self.base.m_renderTarget };
        let mut attachment = super::webgpu_decl::WGPURenderPassColorAttachment::default();
        attachment.depthSlice = super::webgpu_decl::WGPU_DEPTH_SLICE_UNDEFINED;
        if desc.fixedFunctionColorOutput || drawType == DrawType::renderPassResolve {
            attachment.view = target.targetTextureView().Get();
            attachment.storeOp = super::webgpu_cpp_decl::StoreOp::Store.into();
            if desc.fixedFunctionColorOutput {
                attachment.loadOp = if desc.colorLoadAction == LoadAction::preserveRenderTarget
                    || drawType != DrawType::renderPassInitialize
                {
                    super::webgpu_cpp_decl::LoadOp::Load.into()
                } else {
                    super::webgpu_cpp_decl::LoadOp::Clear.into()
                };
                if desc.colorLoadAction == LoadAction::clear
                    && drawType == DrawType::renderPassInitialize
                {
                    attachment.clearValue = wgpuColorPremul(desc.colorClearValue);
                }
            } else {
                assert_eq!(drawType, DrawType::renderPassResolve);
                let fullBounds = crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB {
                    left: 0,
                    top: 0,
                    right: target.width() as i32,
                    bottom: target.height() as i32,
                };
                attachment.loadOp = if desc.colorLoadAction == LoadAction::preserveRenderTarget
                    && desc.renderTargetUpdateBounds != fullBounds
                {
                    super::webgpu_cpp_decl::LoadOp::Load.into()
                } else {
                    super::webgpu_cpp_decl::LoadOp::Clear.into()
                };
            }
        } else {
            attachment.view = dstColorTextureView(target).Get();
            attachment.loadOp = super::webgpu_cpp_decl::LoadOp::Clear.into();
            attachment.storeOp = super::webgpu_cpp_decl::StoreOp::Discard.into();
        }
        let mut passDescriptor = super::webgpu_decl::WGPURenderPassDescriptor::default();
        passDescriptor.label = stringView("RIVE_Atomic_RenderPass");
        passDescriptor.colorAttachmentCount = 1;
        passDescriptor.colorAttachments = &attachment;
        *self.base.m_encoder =
            unsafe { self.base.m_commandEncoder.BeginRenderPass(&passDescriptor) };
        unsafe { self.base.initDrawRenderPass() };
        unsafe {
            self.base.m_encoder.SetBindGroup(
                PLS_TEXTURE_BINDINGS_SET as u32,
                self.m_plsBindings.Get(),
                0,
                std::ptr::null(),
            )
        };
    }
}

impl Drop for AtomicDrawRenderPass {
    fn drop(&mut self) {
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.m_plsBindings);
            std::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl DrawRenderPassApi for AtomicDrawRenderPass {
    fn encoder(&self) -> &super::webgpu_cpp_decl::RenderPassEncoder {
        &self.base.m_encoder
    }
    fn barrier(
        &mut self,
        batch: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawBatch,
    ) {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
            BarrierFlags, DrawType,
        };
        if batch.barriers.0 & (BarrierFlags::plsAtomic.0 | BarrierFlags::plsAtomicPreResolve.0) != 0
        {
            assert_ne!(batch.drawType, DrawType::renderPassInitialize);
            self.base.end();
            unsafe { self.begin(batch.drawType) };
        }
    }
    fn end(&mut self) {
        self.base.end()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MSAABeginType {
    primary,
    restartAfterDstCopy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MSAAEndType {
    finish,
    breakForDstCopy,
}

struct MSAADrawRenderPass {
    base: std::mem::ManuallyDrop<DrawRenderPassBase>,
    m_msaaColorTextureView: std::mem::ManuallyDrop<TextureView>,
    m_targetTextureView: std::mem::ManuallyDrop<TextureView>,
    m_msaaDepthStencilTextureView: std::mem::ManuallyDrop<TextureView>,
}

impl MSAADrawRenderPass {
    unsafe fn new(
        implementation: *mut RenderContextWebGPUImpl,
        descriptor: *const crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
        commandEncoder: CommandEncoder,
    ) -> Self {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawType;
        let base = unsafe { DrawRenderPassBase::new(implementation, descriptor, commandEncoder) };
        let target = unsafe { &mut *base.m_renderTarget };
        let mut pass = Self {
            base: std::mem::ManuallyDrop::new(base),
            m_msaaColorTextureView: std::mem::ManuallyDrop::new(msaaColorTextureView(target)),
            m_targetTextureView: std::mem::ManuallyDrop::new(target.targetTextureView()),
            m_msaaDepthStencilTextureView: std::mem::ManuallyDrop::new(
                msaaDepthStencilTextureView(target),
            ),
        };
        let desc = unsafe { &*descriptor };
        let beginsWithInitialize = desc.drawList.is_some_and(|list| unsafe {
            list.as_ref()
                .iter()
                .next()
                .is_some_and(|batch| batch.drawType == DrawType::renderPassInitialize)
        });
        if !beginsWithInitialize {
            unsafe {
                pass.begin(
                    MSAABeginType::primary,
                    if desc.firstDstBlendBarrier.is_some() {
                        MSAAEndType::breakForDstCopy
                    } else {
                        MSAAEndType::finish
                    },
                )
            };
        }
        pass
    }

    unsafe fn begin(&mut self, beginType: MSAABeginType, endType: MSAAEndType) {
        let desc = unsafe { &*self.base.m_desc };
        let loadOp = if beginType == MSAABeginType::restartAfterDstCopy {
            super::webgpu_cpp_decl::LoadOp::Load
        } else {
            super::webgpu_cpp_decl::LoadOp::Clear
        };
        let storeOp = if endType == MSAAEndType::breakForDstCopy {
            super::webgpu_cpp_decl::StoreOp::Store
        } else {
            super::webgpu_cpp_decl::StoreOp::Discard
        };
        let mut color = super::webgpu_decl::WGPURenderPassColorAttachment::default();
        color.view = self.m_msaaColorTextureView.Get();
        color.depthSlice = super::webgpu_decl::WGPU_DEPTH_SLICE_UNDEFINED;
        color.resolveTarget = self.m_targetTextureView.Get();
        color.loadOp = loadOp.into();
        color.storeOp = storeOp.into();
        color.clearValue = wgpuColorPremul(desc.colorClearValue);
        let mut depthStencil = super::webgpu_decl::WGPURenderPassDepthStencilAttachment::default();
        depthStencil.view = self.m_msaaDepthStencilTextureView.Get();
        depthStencil.depthLoadOp = loadOp.into();
        depthStencil.depthStoreOp = storeOp.into();
        depthStencil.depthClearValue = desc.depthClearValue;
        depthStencil.depthReadOnly = super::webgpu_decl::WGPU_FALSE;
        depthStencil.stencilLoadOp = loadOp.into();
        depthStencil.stencilStoreOp = storeOp.into();
        depthStencil.stencilClearValue = u32::from(desc.stencilClearValue);
        depthStencil.stencilReadOnly = super::webgpu_decl::WGPU_FALSE;
        let mut passDescriptor = super::webgpu_decl::WGPURenderPassDescriptor::default();
        passDescriptor.label = stringView("RIVE_MSAA_RenderPass");
        passDescriptor.colorAttachmentCount = 1;
        passDescriptor.colorAttachments = &color;
        passDescriptor.depthStencilAttachment = &depthStencil;
        *self.base.m_encoder =
            unsafe { self.base.m_commandEncoder.BeginRenderPass(&passDescriptor) };
        unsafe { self.base.initDrawRenderPass() };
    }
}

impl Drop for MSAADrawRenderPass {
    fn drop(&mut self) {
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.m_msaaDepthStencilTextureView);
            std::mem::ManuallyDrop::drop(&mut self.m_targetTextureView);
            std::mem::ManuallyDrop::drop(&mut self.m_msaaColorTextureView);
            std::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}

fn intersectDstReadBounds(
    update: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB,
    draw: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB,
) -> IAABB {
    IAABB {
        left: update.left.max(draw.left),
        top: update.top.max(draw.top),
        right: update.right.min(draw.right),
        bottom: update.bottom.min(draw.bottom),
    }
}

impl DrawRenderPassApi for MSAADrawRenderPass {
    fn encoder(&self) -> &super::webgpu_cpp_decl::RenderPassEncoder {
        &self.base.m_encoder
    }
    fn barrier(
        &mut self,
        batch: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawBatch,
    ) {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
            BarrierFlags, DrawType, LoadAction,
        };
        if batch.barriers.0 & BarrierFlags::dstBlend.0 == 0 {
            return;
        }
        let desc = unsafe { &*self.base.m_desc };
        assert!(!desc.fixedFunctionColorOutput || batch.drawType == DrawType::renderPassInitialize);
        let beginType;
        if batch.drawType == DrawType::renderPassInitialize {
            assert!(self.base.m_encoder.Get().is_null());
            assert_eq!(desc.colorLoadAction, LoadAction::preserveRenderTarget);
            let target = unsafe { &mut *self.base.m_renderTarget };
            let bounds = target.base.bounds();
            copyTargetToDstColorTexture(target, &self.base.m_commandEncoder, bounds);
            beginType = MSAABeginType::primary;
        } else {
            self.base.end();
            let target = unsafe { &mut *self.base.m_renderTarget };
            let mut draw = batch
                .dstReadList
                .map_or(std::ptr::null(), |value| value.as_ptr());
            while !draw.is_null() {
                let drawRef = unsafe { &*draw };
                assert_ne!(drawRef.blendMode(), nuxie_render_api::BlendMode::SrcOver);
                copyTargetToDstColorTexture(
                    target,
                    &self.base.m_commandEncoder,
                    intersectDstReadBounds(desc.renderTargetUpdateBounds, drawRef.pixelBounds()),
                );
                draw = drawRef.nextDstRead();
            }
            beginType = MSAABeginType::restartAfterDstCopy;
        }
        unsafe {
            self.begin(
                beginType,
                if batch.nextDstBlendBarrier.is_some() {
                    MSAAEndType::breakForDstCopy
                } else {
                    MSAAEndType::finish
                },
            )
        };
    }
    fn end(&mut self) {
        self.base.end()
    }
}

unsafe fn makeDrawRenderPass(
    implementation: *mut RenderContextWebGPUImpl,
    descriptor: *const crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
    commandEncoder: CommandEncoder,
) -> Box<dyn DrawRenderPassApi> {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode;
    match unsafe { (*descriptor).interlockMode } {
        InterlockMode::atomics => Box::new(unsafe {
            AtomicDrawRenderPass::new(implementation, descriptor, commandEncoder)
        }),
        InterlockMode::msaa => {
            Box::new(unsafe { MSAADrawRenderPass::new(implementation, descriptor, commandEncoder) })
        }
        _ => {
            Box::new(unsafe { PLSDrawRenderPass::new(implementation, descriptor, commandEncoder) })
        }
    }
}

unsafe fn webgpuBuffer(bufferRing: *mut BufferRing) -> Buffer {
    assert!(!bufferRing.is_null());
    unsafe { (&*bufferRing.cast::<BufferWebGPU>()).submittedBuffer() }
}

unsafe fn webgpuStorageTextureView(bufferRing: *mut BufferRing) -> TextureView {
    assert!(!bufferRing.is_null());
    unsafe { (&*(&*bufferRing.cast::<StorageTextureBufferWebGPU>()).m_textureView).clone() }
}

unsafe fn updateWebGPUStorageTexture<T>(
    bufferRing: *mut BufferRing,
    count: u32,
    first: usize,
    commandEncoder: &CommandEncoder,
) {
    assert!(!bufferRing.is_null());
    unsafe {
        (&*bufferRing.cast::<StorageTextureBufferWebGPU>()).updateTextureFromBuffer(
            count as usize * std::mem::size_of::<T>(),
            first * std::mem::size_of::<T>(),
            commandEncoder,
        )
    };
}

pub(crate) fn newColorRampPipeline(
    context: &mut RenderContextWebGPUImpl,
) -> super::render_context_webgpu_decl::ColorRampPipeline {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        GradientSpan, InterlockMode,
    };

    let device = context.device();
    let perFlushEntry = {
        let layout = drawPipelineLayout(context, InterlockMode::rasterOrdering);
        &layout.m_perFlushBindingLayoutEntries[..RenderContextWebGPUImpl::COLOR_RAMP_BINDINGS_COUNT]
    };
    let bindGroupLayout = createLayout(&device, perFlushEntry);
    let rawLayout = bindGroupLayout.Get();
    let mut pipelineLayoutDescriptor = WGPUPipelineLayoutDescriptor::default();
    pipelineLayoutDescriptor.bindGroupLayoutCount = 1;
    pipelineLayoutDescriptor.bindGroupLayouts = &rawLayout;
    let pipelineLayout = unsafe { device.CreatePipelineLayout(&pipelineLayoutDescriptor) };

    let (vertexModule, fragmentModule) =
        if context.m_capabilities.backendType == BackendType::OpenGLES {
            let mut common = format!("#define {GLSL_POST_INVERT_Y} true\n");
            appendGlslParts(
                &mut common,
                &[
                    GLSL_GLSL,
                    GLSL_CONSTANTS,
                    GLSL_FLUSH_UNIFORMS,
                    GLSL_COMMON,
                    GLSL_COLOR_RAMP,
                ],
            );
            let vertex = format!("#version 310 es\n#define {GLSL_VERTEX} true\n{common}");
            let fragment = format!("#version 310 es\n#define {GLSL_FRAGMENT} true\n{common}");
            (
                compileShaderModuleWagyuRaw(&device, &vertex),
                compileShaderModuleWagyuRaw(&device, &fragment),
            )
        } else {
            (
                compileShaderModuleWGSL(
                    &device,
                    include_str!("../../generated/color_ramp.vert.wgsl"),
                    "color_ramp.vert",
                ),
                compileShaderModuleWGSL(
                    &device,
                    include_str!("../../generated/color_ramp.frag.wgsl"),
                    "color_ramp.frag",
                ),
            )
        };
    let mut attribute = WGPUVertexAttribute::default();
    attribute.format = super::webgpu_cpp_decl::VertexFormat::Uint32x4.into();
    attribute.shaderLocation = 0;
    let mut vertexBufferLayout = WGPUVertexBufferLayout::default();
    vertexBufferLayout.arrayStride = std::mem::size_of::<GradientSpan>() as u64;
    vertexBufferLayout.stepMode = super::webgpu_cpp_decl::VertexStepMode::Instance.into();
    vertexBufferLayout.attributeCount = 1;
    vertexBufferLayout.attributes = &attribute;
    let mut colorTarget = WGPUColorTargetState::default();
    colorTarget.format = TextureFormat::RGBA8Unorm.into();
    let mut fragmentState = WGPUFragmentState::default();
    fragmentState.module = fragmentModule.Get();
    fragmentState.entryPoint = stringView("main");
    fragmentState.targetCount = 1;
    fragmentState.targets = &colorTarget;
    let mut descriptor = WGPURenderPipelineDescriptor::default();
    descriptor.label = stringView("RIVE_ColorRampPipeline");
    descriptor.layout = pipelineLayout.Get();
    descriptor.vertex.module = vertexModule.Get();
    descriptor.vertex.entryPoint = stringView("main");
    descriptor.vertex.bufferCount = 1;
    descriptor.vertex.buffers = &vertexBufferLayout;
    descriptor.primitive.topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleStrip.into();
    descriptor.primitive.frontFace = RIVE_FRONT_FACE.into();
    descriptor.primitive.cullMode = super::webgpu_cpp_decl::CullMode::None.into();
    descriptor.fragment = &fragmentState;
    let renderPipeline = unsafe { device.CreateRenderPipeline(&descriptor) };
    super::render_context_webgpu_decl::ColorRampPipeline {
        m_bindGroupLayout: std::mem::ManuallyDrop::new(bindGroupLayout),
        m_renderPipeline: std::mem::ManuallyDrop::new(renderPipeline),
    }
}

pub(crate) fn newTessellatePipeline(
    context: &mut RenderContextWebGPUImpl,
) -> super::render_context_webgpu_decl::TessellatePipeline {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        InterlockMode, TessVertexSpan,
    };

    let device = context.device();
    let emptyLayout = context.m_emptyBindingsLayout.Get();
    let (perFlushBindingsLayout, samplerLayout) = {
        let layout = drawPipelineLayout(context, InterlockMode::rasterOrdering);
        (
            createLayout(
                &device,
                &layout.m_perFlushBindingLayoutEntries
                    [..RenderContextWebGPUImpl::TESS_BINDINGS_COUNT],
            ),
            layout.m_bindGroupLayouts[WEBGPU_SAMPLER_BINDINGS_SET].Get(),
        )
    };
    let rawLayouts = [
        perFlushBindingsLayout.Get(),
        emptyLayout,
        emptyLayout,
        samplerLayout,
    ];
    let mut pipelineLayoutDescriptor = WGPUPipelineLayoutDescriptor::default();
    pipelineLayoutDescriptor.bindGroupLayoutCount = rawLayouts.len();
    pipelineLayoutDescriptor.bindGroupLayouts = rawLayouts.as_ptr();
    let pipelineLayout = unsafe { device.CreatePipelineLayout(&pipelineLayoutDescriptor) };

    let (vertexModule, fragmentModule) =
        if context.m_capabilities.backendType == BackendType::OpenGLES {
            let mut common = String::new();
            if context.m_capabilities.polyfillVertexStorageBuffers {
                common.push_str(&format!(
                    "#define {GLSL_DISABLE_SHADER_STORAGE_BUFFERS} true\n"
                ));
            }
            common.push_str(&format!("#define {GLSL_POST_INVERT_Y} true\n"));
            appendGlslParts(
                &mut common,
                &[
                    GLSL_GLSL,
                    GLSL_CONSTANTS,
                    GLSL_FLUSH_UNIFORMS,
                    GLSL_COMMON,
                    GLSL_BEZIER_UTILS,
                    GLSL_TESSELLATE,
                ],
            );
            let vertex = format!("#version 310 es\n#define {GLSL_VERTEX} true\n{common}");
            let fragment = format!("#version 310 es\n#define {GLSL_FRAGMENT} true\n{common}");
            (
                compileShaderModuleWagyuRaw(&device, &vertex),
                compileShaderModuleWagyuRaw(&device, &fragment),
            )
        } else {
            let vertexSource = if context.m_capabilities.polyfillVertexStorageBuffers {
                include_str!("../../generated/tessellate.webgpu_nossbo_vert.wgsl")
            } else {
                include_str!("../../generated/tessellate.webgpu_vert.wgsl")
            };
            (
                compileShaderModuleWGSL(&device, vertexSource, "tessellate.vert"),
                compileShaderModuleWGSL(
                    &device,
                    include_str!("../../generated/tessellate.webgpu_frag.wgsl"),
                    "tessellate.frag",
                ),
            )
        };
    let mut attributes: [WGPUVertexAttribute; 4] =
        std::array::from_fn(|_| WGPUVertexAttribute::default());
    for (index, attribute) in attributes.iter_mut().enumerate() {
        attribute.format = if index == 3 {
            super::webgpu_cpp_decl::VertexFormat::Uint32x4.into()
        } else {
            super::webgpu_cpp_decl::VertexFormat::Float32x4.into()
        };
        attribute.offset = (index * 4 * std::mem::size_of::<f32>()) as u64;
        attribute.shaderLocation = index as u32;
    }
    let mut vertexBufferLayout = WGPUVertexBufferLayout::default();
    vertexBufferLayout.arrayStride = std::mem::size_of::<TessVertexSpan>() as u64;
    vertexBufferLayout.stepMode = super::webgpu_cpp_decl::VertexStepMode::Instance.into();
    vertexBufferLayout.attributeCount = attributes.len();
    vertexBufferLayout.attributes = attributes.as_ptr();
    let mut colorTarget = WGPUColorTargetState::default();
    colorTarget.format = TextureFormat::RGBA32Uint.into();
    let mut fragmentState = WGPUFragmentState::default();
    fragmentState.module = fragmentModule.Get();
    fragmentState.entryPoint = stringView("main");
    fragmentState.targetCount = 1;
    fragmentState.targets = &colorTarget;
    let mut descriptor = WGPURenderPipelineDescriptor::default();
    descriptor.label = stringView("RIVE_TessellatePipeline");
    descriptor.layout = pipelineLayout.Get();
    descriptor.vertex.module = vertexModule.Get();
    descriptor.vertex.entryPoint = stringView("main");
    descriptor.vertex.bufferCount = 1;
    descriptor.vertex.buffers = &vertexBufferLayout;
    descriptor.primitive.topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleList.into();
    descriptor.primitive.frontFace = RIVE_FRONT_FACE.into();
    descriptor.primitive.cullMode = super::webgpu_cpp_decl::CullMode::None.into();
    descriptor.fragment = &fragmentState;
    let renderPipeline = unsafe { device.CreateRenderPipeline(&descriptor) };
    super::render_context_webgpu_decl::TessellatePipeline {
        m_perFlushBindingsLayout: std::mem::ManuallyDrop::new(perFlushBindingsLayout),
        m_renderPipeline: std::mem::ManuallyDrop::new(renderPipeline),
    }
}

pub(crate) fn newFeatherAtlasPipeline(
    context: &mut RenderContextWebGPUImpl,
) -> super::render_context_webgpu_decl::FeatherAtlasPipeline {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        InterlockMode, PatchVertex,
    };

    let device = context.device();
    let emptyLayout = context.m_emptyBindingsLayout.Get();
    let (perFlushBindingsLayout, samplerLayout) = {
        let layout = drawPipelineLayout(context, InterlockMode::rasterOrdering);
        (
            createLayout(
                &device,
                &layout.m_perFlushBindingLayoutEntries
                    [..RenderContextWebGPUImpl::FEATHER_ATLAS_BINDINGS_COUNT],
            ),
            layout.m_bindGroupLayouts[WEBGPU_SAMPLER_BINDINGS_SET].Get(),
        )
    };
    let rawLayouts = [
        perFlushBindingsLayout.Get(),
        emptyLayout,
        emptyLayout,
        samplerLayout,
    ];
    let mut pipelineLayoutDescriptor = WGPUPipelineLayoutDescriptor::default();
    pipelineLayoutDescriptor.bindGroupLayoutCount = rawLayouts.len();
    pipelineLayoutDescriptor.bindGroupLayouts = rawLayouts.as_ptr();
    let pipelineLayout = unsafe { device.CreatePipelineLayout(&pipelineLayoutDescriptor) };

    let (vertexModule, fillFragmentModule, strokeFragmentModule) = if context
        .m_capabilities
        .backendType
        == BackendType::OpenGLES
    {
        let mut common = format!(
            "#define {GLSL_DRAW_PATH} true\n#define {GLSL_ENABLE_FEATHER} true\n#define {GLSL_ENABLE_INSTANCE_INDEX} true\n#define {GLSL_BASE_INSTANCE_UNIFORM_NAME} {BASE_INSTANCE_UNIFORM_NAME}\n#define {GLSL_POST_INVERT_Y} true\n"
        );
        if context.m_capabilities.polyfillVertexStorageBuffers {
            common.push_str(&format!(
                "#define {GLSL_DISABLE_SHADER_STORAGE_BUFFERS} true\n"
            ));
        }
        appendGlslParts(
            &mut common,
            &[
                GLSL_GLSL,
                GLSL_CONSTANTS,
                GLSL_FLUSH_UNIFORMS,
                GLSL_COMMON,
                include_str!("source/generated_glsl/draw_path_common.minified.glsl"),
                GLSL_RENDER_ATLAS,
            ],
        );
        let vertex = format!(
            "#version 310 es\n#pragma shader_stage(vertex)\n#define {GLSL_VERTEX} true\n{common}"
        );
        let fill = format!(
            "#version 310 es\n#pragma shader_stage(fragment)\n#define {GLSL_FRAGMENT} true\n#define {GLSL_ATLAS_FEATHERED_FILL} true\n{common}"
        );
        let stroke = format!(
            "#version 310 es\n#pragma shader_stage(fragment)\n#define {GLSL_FRAGMENT} true\n#define {GLSL_ATLAS_FEATHERED_STROKE} true\n{common}"
        );
        (
            compileShaderModuleWagyuRaw(&device, &vertex),
            compileShaderModuleWagyuRaw(&device, &fill),
            compileShaderModuleWagyuRaw(&device, &stroke),
        )
    } else {
        let vertexSource = if context.m_capabilities.polyfillVertexStorageBuffers {
            include_str!("../../generated/render_atlas.webgpu_nossbo_vert.wgsl")
        } else {
            include_str!("../../generated/render_atlas.webgpu_vert.wgsl")
        };
        (
            compileShaderModuleWGSL(&device, vertexSource, "render_atlas.vert"),
            compileShaderModuleWGSL(
                &device,
                include_str!("../../generated/render_atlas_fill.webgpu_frag.wgsl"),
                "render_atlas_fill.frag",
            ),
            compileShaderModuleWGSL(
                &device,
                include_str!("../../generated/render_atlas_stroke.webgpu_frag.wgsl"),
                "render_atlas_stroke.frag",
            ),
        )
    };
    let mut attributes: [WGPUVertexAttribute; 2] =
        std::array::from_fn(|_| WGPUVertexAttribute::default());
    for (index, attribute) in attributes.iter_mut().enumerate() {
        attribute.format = super::webgpu_cpp_decl::VertexFormat::Float32x4.into();
        attribute.offset = (index * 4 * std::mem::size_of::<f32>()) as u64;
        attribute.shaderLocation = index as u32;
    }
    let mut vertexBufferLayout = WGPUVertexBufferLayout::default();
    vertexBufferLayout.arrayStride = std::mem::size_of::<PatchVertex>() as u64;
    vertexBufferLayout.stepMode = super::webgpu_cpp_decl::VertexStepMode::Vertex.into();
    vertexBufferLayout.attributeCount = attributes.len();
    vertexBufferLayout.attributes = attributes.as_ptr();
    let mut blend = WGPUBlendState::default();
    blend.color.operation = super::webgpu_cpp_decl::BlendOperation::Add.into();
    blend.color.srcFactor = super::webgpu_cpp_decl::BlendFactor::One.into();
    blend.color.dstFactor = super::webgpu_cpp_decl::BlendFactor::One.into();
    let mut colorTarget = WGPUColorTargetState::default();
    colorTarget.format = TextureFormat::R16Float.into();
    colorTarget.blend = &blend;
    let mut fragmentState = WGPUFragmentState::default();
    fragmentState.module = fillFragmentModule.Get();
    fragmentState.entryPoint = stringView("main");
    fragmentState.targetCount = 1;
    fragmentState.targets = &colorTarget;
    let mut descriptor = WGPURenderPipelineDescriptor::default();
    descriptor.label = stringView("RIVE_AtlasPipeline");
    descriptor.layout = pipelineLayout.Get();
    descriptor.vertex.module = vertexModule.Get();
    descriptor.vertex.entryPoint = stringView("main");
    descriptor.vertex.bufferCount = 1;
    descriptor.vertex.buffers = &vertexBufferLayout;
    descriptor.primitive.topology = super::webgpu_cpp_decl::PrimitiveTopology::TriangleList.into();
    descriptor.primitive.frontFace = RIVE_FRONT_FACE.into();
    descriptor.primitive.cullMode = super::webgpu_cpp_decl::CullMode::None.into();
    descriptor.fragment = &fragmentState;
    let fillPipeline = unsafe { device.CreateRenderPipeline(&descriptor) };
    blend.color.operation = super::webgpu_cpp_decl::BlendOperation::Max.into();
    fragmentState.module = strokeFragmentModule.Get();
    let strokePipeline = unsafe { device.CreateRenderPipeline(&descriptor) };
    super::render_context_webgpu_decl::FeatherAtlasPipeline {
        m_perFlushBindingsLayout: std::mem::ManuallyDrop::new(perFlushBindingsLayout),
        m_fillPipeline: std::mem::ManuallyDrop::new(fillPipeline),
        m_strokePipeline: std::mem::ManuallyDrop::new(strokePipeline),
    }
}

fn addressMode(
    wrap: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageWrap,
) -> super::webgpu_cpp_decl::AddressMode {
    match wrap.0 {
        0 => super::webgpu_cpp_decl::AddressMode::ClampToEdge,
        1 => super::webgpu_cpp_decl::AddressMode::Repeat,
        2 => super::webgpu_cpp_decl::AddressMode::MirrorRepeat,
        _ => unreachable!("ImageWrap source discriminant"),
    }
}

fn filterMode(
    filter: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageFilter,
) -> super::webgpu_cpp_decl::FilterMode {
    match filter.0 {
        0 => super::webgpu_cpp_decl::FilterMode::Linear,
        1 => super::webgpu_cpp_decl::FilterMode::Nearest,
        _ => unreachable!("ImageFilter source discriminant"),
    }
}

unsafe fn initializeMappedBuffer<T>(buffer: &Buffer, source: &[T]) {
    let bytes = std::mem::size_of_val(source);
    unsafe {
        std::ptr::copy_nonoverlapping(
            source.as_ptr().cast::<u8>(),
            buffer
                .GetMappedRange(0, super::webgpu_decl::WGPU_WHOLE_MAP_SIZE)
                .cast::<u8>(),
            bytes,
        );
        buffer.Unmap();
    }
}

pub(crate) fn initGPUObjects(context: &mut RenderContextWebGPUImpl) {
    use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        self as gpu, InterlockMode,
    };

    let device = context.device();
    let queue = context.queue();
    *context.m_emptyBindingsLayout = createLayout(&device, &[]);

    let mut nullBufferDesc = WGPUBufferDescriptor::default();
    nullBufferDesc.usage = super::webgpu_cpp_decl::BufferUsage::Storage.into();
    nullBufferDesc.size = std::mem::size_of::<u32>() as u64;
    *context.m_nullStorageBuffer = unsafe { device.CreateBuffer(&nullBufferDesc) };

    let mut linearDesc = WGPUSamplerDescriptor::default();
    linearDesc.addressModeU = super::webgpu_cpp_decl::AddressMode::ClampToEdge.into();
    linearDesc.addressModeV = super::webgpu_cpp_decl::AddressMode::ClampToEdge.into();
    linearDesc.magFilter = super::webgpu_cpp_decl::FilterMode::Linear.into();
    linearDesc.minFilter = super::webgpu_cpp_decl::FilterMode::Linear.into();
    linearDesc.mipmapFilter = super::webgpu_cpp_decl::MipmapFilterMode::Nearest.into();
    *context.m_linearSampler = unsafe { device.CreateSampler(&linearDesc) };
    for index in 0..ImageSampler::MAX_SAMPLER_PERMUTATIONS {
        let filter = ImageSampler::GetFilterOptionFromKey(index as u8);
        let mut descriptor = WGPUSamplerDescriptor::default();
        descriptor.addressModeU =
            addressMode(ImageSampler::GetWrapXOptionFromKey(index as u8)).into();
        descriptor.addressModeV =
            addressMode(ImageSampler::GetWrapYOptionFromKey(index as u8)).into();
        descriptor.magFilter = filterMode(filter).into();
        descriptor.minFilter = filterMode(filter).into();
        descriptor.mipmapFilter = super::webgpu_cpp_decl::MipmapFilterMode::Nearest.into();
        context.m_imageSamplers[index] = unsafe { device.CreateSampler(&descriptor) };
    }

    let samplerLayout = drawPipelineLayout(context, InterlockMode::rasterOrdering)
        .m_bindGroupLayouts[WEBGPU_SAMPLER_BINDINGS_SET]
        .Get();
    let mut samplerEntries: [WGPUBindGroupEntry; 3] =
        std::array::from_fn(|_| WGPUBindGroupEntry::default());
    for (entry, binding) in samplerEntries.iter_mut().zip([
        GRAD_TEXTURE_IDX,
        GAUSSIAN_INTEGRAL_TEXTURE_IDX,
        FEATHER_ATLAS_TEXTURE_IDX,
    ]) {
        entry.binding = binding;
        entry.sampler = context.m_linearSampler.Get();
    }
    let mut samplerGroupDesc = WGPUBindGroupDescriptor::default();
    samplerGroupDesc.layout = samplerLayout;
    samplerGroupDesc.entryCount = samplerEntries.len();
    samplerGroupDesc.entries = samplerEntries.as_ptr();
    *context.m_samplerBindings = unsafe { device.CreateBindGroup(&samplerGroupDesc) };

    if context.m_capabilities.plsType == PixelLocalStorageType::GL_EXT_shader_pixel_local_storage {
        let mut source = format!(
            "#version 310 es\n#define {GLSL_VERTEX} true\n#ifndef GL_EXT_shader_pixel_local_storage\n#define gl_VertexID gl_VertexIndex\n#endif\n#define {GLSL_ENABLE_CLIPPING} true\n"
        );
        BuildLoadStoreEXTGLSL(&mut source, LoadStoreActionsEXT::none);
        *context.m_loadStoreEXTVertexShader = compileShaderModuleWagyuRaw(&device, &source);
        *context.m_loadStoreEXTUniforms = Some(makeUniformBufferRing(
            context,
            std::mem::size_of::<f32>() * 4,
        ));
    }

    let mut bufferDesc = WGPUBufferDescriptor::default();
    bufferDesc.mappedAtCreation = super::webgpu_decl::WGPU_TRUE;
    bufferDesc.usage = super::webgpu_cpp_decl::BufferUsage::Index.into();
    bufferDesc.size = std::mem::size_of_val(&gpu::kTessSpanIndices) as u64;
    *context.m_tessSpanIndexBuffer = unsafe { device.CreateBuffer(&bufferDesc) };
    unsafe { initializeMappedBuffer(&context.m_tessSpanIndexBuffer, &gpu::kTessSpanIndices) };

    bufferDesc.usage = super::webgpu_cpp_decl::BufferUsage::Vertex.into();
    bufferDesc.size =
        u64::from(gpu::kPatchVertexBufferCount) * std::mem::size_of::<gpu::PatchVertex>() as u64;
    *context.m_pathPatchVertexBuffer = unsafe { device.CreateBuffer(&bufferDesc) };
    bufferDesc.usage = super::webgpu_cpp_decl::BufferUsage::Index.into();
    bufferDesc.size = (u64::from(gpu::kPatchIndexBufferCount) * std::mem::size_of::<u16>() as u64)
        .next_multiple_of(4);
    *context.m_pathPatchIndexBuffer = unsafe { device.CreateBuffer(&bufferDesc) };
    unsafe {
        gpu::GeneratePatchBufferData(
            context
                .m_pathPatchVertexBuffer
                .GetMappedRange(0, super::webgpu_decl::WGPU_WHOLE_MAP_SIZE)
                .cast(),
            context
                .m_pathPatchIndexBuffer
                .GetMappedRange(0, super::webgpu_decl::WGPU_WHOLE_MAP_SIZE)
                .cast(),
        );
        context.m_pathPatchVertexBuffer.Unmap();
        context.m_pathPatchIndexBuffer.Unmap();
    }

    bufferDesc.usage = super::webgpu_cpp_decl::BufferUsage::Vertex.into();
    bufferDesc.size = std::mem::size_of_val(&gpu::kImageRectVertices) as u64;
    *context.m_imageRectVertexBuffer = unsafe { device.CreateBuffer(&bufferDesc) };
    unsafe { initializeMappedBuffer(&context.m_imageRectVertexBuffer, &gpu::kImageRectVertices) };
    bufferDesc.usage = super::webgpu_cpp_decl::BufferUsage::Index.into();
    bufferDesc.size = (std::mem::size_of_val(&gpu::kImageRectIndices) as u64).next_multiple_of(4);
    *context.m_imageRectIndexBuffer = unsafe { device.CreateBuffer(&bufferDesc) };
    unsafe { initializeMappedBuffer(&context.m_imageRectIndexBuffer, &gpu::kImageRectIndices) };

    *context.m_gaussianIntegralTexture = makeTexture(
        &device,
        (TextureUsage::TextureBinding | TextureUsage::CopyDst).intoBitmask(),
        gpu::GAUSSIAN_TABLE_SIZE,
        2,
        TextureFormat::R16Float,
        1,
    );
    let mut destination = WGPUTexelCopyTextureInfo::default();
    destination.texture = context.m_gaussianIntegralTexture.Get();
    let mut uploadLayout = WGPUTexelCopyBufferLayout::default();
    uploadLayout.bytesPerRow =
        unsafe { std::mem::size_of_val(&gpu::g_gaussianIntegralTableF16) as u32 };
    let extent = WGPUExtent3D {
        width: gpu::GAUSSIAN_TABLE_SIZE,
        height: 1,
        depthOrArrayLayers: 1,
    };
    unsafe {
        queue.WriteTexture(
            &destination,
            gpu::g_gaussianIntegralTableF16.as_ptr().cast(),
            std::mem::size_of_val(&gpu::g_gaussianIntegralTableF16),
            &uploadLayout,
            &extent,
        );
        destination.origin.y = 1;
        queue.WriteTexture(
            &destination,
            gpu::g_inverseGaussianIntegralTableF16.as_ptr().cast(),
            std::mem::size_of_val(&gpu::g_inverseGaussianIntegralTableF16),
            &uploadLayout,
            &extent,
        );
    }
    *context.m_gaussianIntegralTextureView = makeView(&context.m_gaussianIntegralTexture);
    *context.m_nullTexture = makeTexture(
        &device,
        TextureUsage::TextureBinding,
        1,
        1,
        TextureFormat::RGBA8Unorm,
        1,
    );
    *context.m_nullTextureView = makeView(&context.m_nullTexture);

    let colorRamp = newColorRampPipeline(context);
    let tessellate = newTessellatePipeline(context);
    let featherAtlas = newFeatherAtlasPipeline(context);
    *context.m_colorRampPipeline = Some(Box::new(colorRamp));
    *context.m_tessellatePipeline = Some(Box::new(tessellate));
    *context.m_featherAtlasPipeline = Some(Box::new(featherAtlas));
}

fn makeTexture(
    device: &super::webgpu_cpp_decl::Device,
    usage: TextureUsage,
    width: u32,
    height: u32,
    format: TextureFormat,
    sampleCount: u32,
) -> WagyuTexture {
    let mut desc = WGPUTextureDescriptor::default();
    desc.usage = usage.into();
    desc.size.width = width;
    desc.size.height = height;
    desc.size.depthOrArrayLayers = 1;
    desc.format = format.into();
    desc.mipLevelCount = 1;
    desc.sampleCount = sampleCount;
    unsafe { device.CreateTexture(&desc) }
}

fn makeView(texture: &WagyuTexture) -> TextureView {
    unsafe { texture.CreateView(std::ptr::null()) }
}

pub(crate) fn newRenderTarget(
    device: super::webgpu_cpp_decl::Device,
    platformFeatures: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PlatformFeatures,
    capabilities: &Capabilities,
    framebufferFormat: TextureFormat,
    width: u32,
    height: u32,
) -> RenderTargetWebGPU {
    let mut base = RenderTarget::new(width, height);
    base.destroy_complete =
        |base| unsafe { drop(Box::from_raw(base.cast::<RenderTargetWebGPU>())) };
    let mut transientPLSUsage = TextureUsage::RenderAttachment;
    let mut transientMSAAColorUsage = TextureUsage::RenderAttachment;
    let mut transientMSAADepthStencilUsage = TextureUsage::RenderAttachment;
    if capabilities.plsType == PixelLocalStorageType::VK_EXT_rasterization_order_attachment_access {
        transientPLSUsage |= TextureUsage(
            (WGPUTextureUsage_WagyuInputAttachment | WGPUTextureUsage_WagyuTransientAttachment)
                as u64,
        );
    }
    if platformFeatures.supportsBlendAdvancedKHR {
        transientMSAAColorUsage |= TextureUsage(
            (WGPUTextureUsage_WagyuTransientAttachment | WGPUTextureUsage_WagyuMSAAResolveSource)
                as u64,
        );
        transientMSAADepthStencilUsage |=
            TextureUsage(WGPUTextureUsage_WagyuTransientAttachment as u64);
    }
    RenderTargetWebGPU {
        base: std::mem::ManuallyDrop::new(base),
        m_device: std::mem::ManuallyDrop::new(device),
        m_framebufferFormat: framebufferFormat,
        m_transientPLSUsage: transientPLSUsage,
        m_transientMSAAColorUsage: transientMSAAColorUsage,
        m_transientMSAADepthStencilUsage: transientMSAADepthStencilUsage,
        m_targetTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_coverageTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_clipTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_scratchColorTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_msaaColorTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_msaaDepthStencilTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_dstColorTexture: std::mem::ManuallyDrop::new(WagyuTexture::default()),
        m_targetTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_coverageTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_clipTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_scratchColorTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_msaaColorTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_msaaDepthStencilTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
        m_dstColorTextureView: std::mem::ManuallyDrop::new(TextureView::default()),
    }
}

pub(crate) fn makeRenderTarget(
    context: &RenderContextWebGPUImpl,
    framebufferFormat: TextureFormat,
    width: u32,
    height: u32,
) -> rcp<RenderTargetWebGPU> {
    make_rcp(|| {
        newRenderTarget(
            (&*context.m_device).clone(),
            context.platformFeatures(),
            &context.m_capabilities,
            framebufferFormat,
            width,
            height,
        )
    })
}

pub(crate) fn coverageTextureView(target: &mut RenderTargetWebGPU) -> TextureView {
    if target.m_coverageTextureView.Get().is_null() {
        *target.m_coverageTexture = makeTexture(
            &target.m_device,
            target.m_transientPLSUsage,
            target.width(),
            target.height(),
            TextureFormat::R32Uint,
            1,
        );
        *target.m_coverageTextureView = makeView(&target.m_coverageTexture);
    }
    (&*target.m_coverageTextureView).clone()
}

pub(crate) fn clipTextureView(target: &mut RenderTargetWebGPU) -> TextureView {
    if target.m_clipTexture.Get().is_null() {
        *target.m_clipTexture = makeTexture(
            &target.m_device,
            target.m_transientPLSUsage,
            target.width(),
            target.height(),
            TextureFormat::R32Uint,
            1,
        );
        *target.m_clipTextureView = makeView(&target.m_clipTexture);
    }
    (&*target.m_clipTextureView).clone()
}

pub(crate) fn scratchColorTextureView(target: &mut RenderTargetWebGPU) -> TextureView {
    if target.m_scratchColorTexture.Get().is_null() {
        *target.m_scratchColorTexture = makeTexture(
            &target.m_device,
            target.m_transientPLSUsage,
            target.width(),
            target.height(),
            target.m_framebufferFormat,
            1,
        );
        *target.m_scratchColorTextureView = makeView(&target.m_scratchColorTexture);
    }
    (&*target.m_scratchColorTextureView).clone()
}

pub(crate) fn msaaColorTextureView(target: &mut RenderTargetWebGPU) -> TextureView {
    if target.m_msaaColorTexture.Get().is_null() {
        *target.m_msaaColorTexture = makeTexture(
            &target.m_device,
            target.m_transientMSAAColorUsage,
            target.width(),
            target.height(),
            target.m_framebufferFormat,
            MSAA_SAMPLE_COUNT,
        );
        *target.m_msaaColorTextureView = makeView(&target.m_msaaColorTexture);
    }
    (&*target.m_msaaColorTextureView).clone()
}

pub(crate) fn msaaDepthStencilTextureView(target: &mut RenderTargetWebGPU) -> TextureView {
    if target.m_msaaDepthStencilTexture.Get().is_null() {
        *target.m_msaaDepthStencilTexture = makeTexture(
            &target.m_device,
            target.m_transientMSAADepthStencilUsage,
            target.width(),
            target.height(),
            TextureFormat::Depth24PlusStencil8,
            MSAA_SAMPLE_COUNT,
        );
        *target.m_msaaDepthStencilTextureView = makeView(&target.m_msaaDepthStencilTexture);
    }
    (&*target.m_msaaDepthStencilTextureView).clone()
}

pub(crate) fn dstColorTexture(target: &mut RenderTargetWebGPU) -> WagyuTexture {
    if target.m_dstColorTexture.Get().is_null() {
        *target.m_dstColorTexture = makeTexture(
            &target.m_device,
            (TextureUsage::CopyDst | TextureUsage::TextureBinding | TextureUsage::RenderAttachment)
                .intoBitmask(),
            target.width(),
            target.height(),
            target.m_framebufferFormat,
            1,
        );
    }
    (&*target.m_dstColorTexture).clone()
}

pub(crate) fn dstColorTextureView(target: &mut RenderTargetWebGPU) -> TextureView {
    if target.m_dstColorTextureView.Get().is_null() {
        let texture = dstColorTexture(target);
        *target.m_dstColorTextureView = makeView(&texture);
    }
    (&*target.m_dstColorTextureView).clone()
}

pub(crate) fn copyTargetToDstColorTexture(
    target: &mut RenderTargetWebGPU,
    commandEncoder: &CommandEncoder,
    dstReadBounds: IAABB,
) {
    let sourceOrigin = WGPUOrigin3D {
        x: dstReadBounds.left as u32,
        y: dstReadBounds.top as u32,
        z: 0,
    };
    let mut source = WGPUTexelCopyTextureInfo::default();
    source.texture = target.m_targetTexture.Get();
    source.origin = sourceOrigin;
    let destinationTexture = dstColorTexture(target);
    let mut destination = WGPUTexelCopyTextureInfo::default();
    destination.texture = destinationTexture.Get();
    destination.origin = WGPUOrigin3D {
        x: dstReadBounds.left as u32,
        y: dstReadBounds.top as u32,
        z: 0,
    };
    let copySize = WGPUExtent3D {
        width: dstReadBounds.right.wrapping_sub(dstReadBounds.left) as u32,
        height: dstReadBounds.bottom.wrapping_sub(dstReadBounds.top) as u32,
        depthOrArrayLayers: 1,
    };
    unsafe { commandEncoder.CopyTextureToTexture(&source, &destination, &copySize) };
}

#[derive(Clone, Copy)]
struct FlushBufferRings {
    flush: *mut BufferRing,
    path: *mut BufferRing,
    paint: *mut BufferRing,
    paintAux: *mut BufferRing,
    contour: *mut BufferRing,
    gradSpan: *mut BufferRing,
    tessSpan: *mut BufferRing,
    triangle: *mut BufferRing,
    imageDrawInstance: *mut BufferRing,
}

fn flushBufferRings(context: &mut RenderContextWebGPUImpl) -> FlushBufferRings {
    FlushBufferRings {
        flush: context.base.flushUniformBufferRing(),
        path: context.base.pathBufferRing(),
        paint: context.base.paintBufferRing(),
        paintAux: context.base.paintAuxBufferRing(),
        contour: context.base.contourBufferRing(),
        gradSpan: context.base.gradSpanBufferRing(),
        tessSpan: context.base.tessSpanBufferRing(),
        triangle: context.base.triangleBufferRing(),
        imageDrawInstance: context.base.imageDrawInstanceBufferRing(),
    }
}

unsafe fn encodeOffscreenPasses(
    context: &mut RenderContextWebGPUImpl,
    desc: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
    commandEncoder: &CommandEncoder,
    perFlushEntries: &[WGPUBindGroupEntry; RenderContextWebGPUImpl::DRAW_BINDINGS_COUNT],
    rings: FlushBufferRings,
) {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;

    if desc.gradDataHeight > 0 {
        let pipeline = context
            .m_colorRampPipeline
            .as_ref()
            .expect("color ramp pipeline");
        let mut groupDescriptor = WGPUBindGroupDescriptor::default();
        groupDescriptor.layout = pipeline.m_bindGroupLayout.Get();
        groupDescriptor.entryCount = RenderContextWebGPUImpl::COLOR_RAMP_BINDINGS_COUNT;
        groupDescriptor.entries = perFlushEntries.as_ptr();
        let group = unsafe { context.m_device.CreateBindGroup(&groupDescriptor) };
        let mut attachment = super::webgpu_decl::WGPURenderPassColorAttachment::default();
        attachment.view = context.m_gradientTextureView.Get();
        attachment.depthSlice = super::webgpu_decl::WGPU_DEPTH_SLICE_UNDEFINED;
        attachment.loadOp = super::webgpu_cpp_decl::LoadOp::Clear.into();
        attachment.storeOp = super::webgpu_cpp_decl::StoreOp::Store.into();
        let mut passDescriptor = super::webgpu_decl::WGPURenderPassDescriptor::default();
        passDescriptor.colorAttachmentCount = 1;
        passDescriptor.colorAttachments = &attachment;
        let pass = unsafe { commandEncoder.BeginRenderPass(&passDescriptor) };
        unsafe {
            pass.SetViewport(
                0.0,
                0.0,
                gpu::kGradTextureWidth as f32,
                desc.gradDataHeight as f32,
                0.0,
                1.0,
            );
            pass.SetPipeline(pipeline.m_renderPipeline.Get());
            pass.SetVertexBuffer(
                0,
                webgpuBuffer(rings.gradSpan).Get(),
                (desc.firstGradSpan * std::mem::size_of::<gpu::GradientSpan>()) as u64,
                super::webgpu_decl::WGPU_WHOLE_SIZE,
            );
            pass.SetBindGroup(0, group.Get(), 0, std::ptr::null());
            pass.Draw(
                gpu::GRAD_SPAN_TRI_STRIP_VERTEX_COUNT,
                desc.gradSpanCount,
                0,
                0,
            );
            pass.End();
        }
    }

    if desc.tessVertexSpanCount > 0 {
        let pipeline = context
            .m_tessellatePipeline
            .as_ref()
            .expect("tessellate pipeline");
        let mut groupDescriptor = WGPUBindGroupDescriptor::default();
        groupDescriptor.layout = pipeline.m_perFlushBindingsLayout.Get();
        groupDescriptor.entryCount = RenderContextWebGPUImpl::TESS_BINDINGS_COUNT;
        groupDescriptor.entries = perFlushEntries.as_ptr();
        let group = unsafe { context.m_device.CreateBindGroup(&groupDescriptor) };
        let mut attachment = super::webgpu_decl::WGPURenderPassColorAttachment::default();
        attachment.view = context.m_tessVertexTextureView.Get();
        attachment.depthSlice = super::webgpu_decl::WGPU_DEPTH_SLICE_UNDEFINED;
        attachment.loadOp = super::webgpu_cpp_decl::LoadOp::Clear.into();
        attachment.storeOp = super::webgpu_cpp_decl::StoreOp::Store.into();
        let mut passDescriptor = super::webgpu_decl::WGPURenderPassDescriptor::default();
        passDescriptor.colorAttachmentCount = 1;
        passDescriptor.colorAttachments = &attachment;
        let pass = unsafe { commandEncoder.BeginRenderPass(&passDescriptor) };
        unsafe {
            pass.SetViewport(
                0.0,
                0.0,
                gpu::kTessTextureWidth as f32,
                desc.tessDataHeight as f32,
                0.0,
                1.0,
            );
            pass.SetPipeline(pipeline.m_renderPipeline.Get());
            pass.SetVertexBuffer(
                0,
                webgpuBuffer(rings.tessSpan).Get(),
                (desc.firstTessVertexSpan * std::mem::size_of::<gpu::TessVertexSpan>()) as u64,
                super::webgpu_decl::WGPU_WHOLE_SIZE,
            );
            pass.SetIndexBuffer(
                context.m_tessSpanIndexBuffer.Get(),
                super::webgpu_cpp_decl::IndexFormat::Uint16.into(),
                0,
                super::webgpu_decl::WGPU_WHOLE_SIZE,
            );
            pass.SetBindGroup(
                PER_FLUSH_BINDINGS_SET as u32,
                group.Get(),
                0,
                std::ptr::null(),
            );
            pass.SetBindGroup(
                WEBGPU_SAMPLER_BINDINGS_SET as u32,
                context.m_samplerBindings.Get(),
                0,
                std::ptr::null(),
            );
            pass.DrawIndexed(
                gpu::kTessSpanIndices.len() as u32,
                desc.tessVertexSpanCount,
                0,
                0,
                0,
            );
            pass.End();
        }
    }

    if desc.featherAtlasFillBatchCount | desc.featherAtlasStrokeBatchCount != 0 {
        let pipeline = context
            .m_featherAtlasPipeline
            .as_ref()
            .expect("feather atlas pipeline");
        let mut groupDescriptor = WGPUBindGroupDescriptor::default();
        groupDescriptor.layout = pipeline.m_perFlushBindingsLayout.Get();
        groupDescriptor.entryCount = RenderContextWebGPUImpl::FEATHER_ATLAS_BINDINGS_COUNT;
        groupDescriptor.entries = perFlushEntries.as_ptr();
        let group = unsafe { context.m_device.CreateBindGroup(&groupDescriptor) };
        let mut attachment = super::webgpu_decl::WGPURenderPassColorAttachment::default();
        attachment.view = context.m_featherAtlasTextureView.Get();
        attachment.depthSlice = super::webgpu_decl::WGPU_DEPTH_SLICE_UNDEFINED;
        attachment.loadOp = super::webgpu_cpp_decl::LoadOp::Clear.into();
        attachment.storeOp = super::webgpu_cpp_decl::StoreOp::Store.into();
        let mut passDescriptor = super::webgpu_decl::WGPURenderPassDescriptor::default();
        passDescriptor.colorAttachmentCount = 1;
        passDescriptor.colorAttachments = &attachment;
        let pass = unsafe { commandEncoder.BeginRenderPass(&passDescriptor) };
        unsafe {
            pass.SetViewport(
                0.0,
                0.0,
                desc.featherAtlasContentWidth as f32,
                desc.featherAtlasContentHeight as f32,
                0.0,
                1.0,
            );
            pass.SetVertexBuffer(
                0,
                context.m_pathPatchVertexBuffer.Get(),
                0,
                super::webgpu_decl::WGPU_WHOLE_SIZE,
            );
            pass.SetIndexBuffer(
                context.m_pathPatchIndexBuffer.Get(),
                super::webgpu_cpp_decl::IndexFormat::Uint16.into(),
                0,
                super::webgpu_decl::WGPU_WHOLE_SIZE,
            );
            pass.SetBindGroup(
                PER_FLUSH_BINDINGS_SET as u32,
                group.Get(),
                0,
                std::ptr::null(),
            );
            pass.SetBindGroup(
                WEBGPU_SAMPLER_BINDINGS_SET as u32,
                context.m_samplerBindings.Get(),
                0,
                std::ptr::null(),
            );
        }
        if desc.featherAtlasFillBatchCount != 0 {
            unsafe { pass.SetPipeline(pipeline.m_fillPipeline.Get()) };
            let batches = unsafe {
                std::slice::from_raw_parts(
                    desc.featherAtlasFillBatches.expect("fill batches").as_ptr(),
                    desc.featherAtlasFillBatchCount,
                )
            };
            for batch in batches {
                unsafe {
                    pass.SetScissorRect(
                        u32::from(batch.scissor.left),
                        u32::from(batch.scissor.top),
                        u32::from(batch.scissor.right - batch.scissor.left),
                        u32::from(batch.scissor.bottom - batch.scissor.top),
                    );
                    pass.DrawIndexed(
                        gpu::kMidpointFanCenterAAPatchIndexCount,
                        batch.patchCount,
                        gpu::kMidpointFanCenterAAPatchBaseIndex,
                        0,
                        batch.basePatch,
                    );
                }
            }
        }
        if desc.featherAtlasStrokeBatchCount != 0 {
            unsafe { pass.SetPipeline(pipeline.m_strokePipeline.Get()) };
            let batches = unsafe {
                std::slice::from_raw_parts(
                    desc.featherAtlasStrokeBatches
                        .expect("stroke batches")
                        .as_ptr(),
                    desc.featherAtlasStrokeBatchCount,
                )
            };
            for batch in batches {
                unsafe {
                    pass.SetScissorRect(
                        u32::from(batch.scissor.left),
                        u32::from(batch.scissor.top),
                        u32::from(batch.scissor.right - batch.scissor.left),
                        u32::from(batch.scissor.bottom - batch.scissor.top),
                    );
                    pass.DrawIndexed(
                        gpu::kMidpointFanPatchBorderIndexCount,
                        batch.patchCount,
                        gpu::kMidpointFanPatchBaseIndex,
                        0,
                        batch.basePatch,
                    );
                }
            }
        }
        unsafe { pass.End() };
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn executeDrawList(
    context: &mut RenderContextWebGPUImpl,
    desc: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
    renderTarget: &mut RenderTargetWebGPU,
    rings: FlushBufferRings,
    renderPass: &mut dyn DrawRenderPassApi,
    drawEncoder: &mut super::webgpu_cpp_decl::RenderPassEncoder,
    perFlushBindings: &BindGroup,
) {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        self as gpu, BarrierFlags, DrawType, InterlockMode, LoadAction, ShaderMiscFlags,
    };

    let layoutPointer =
        drawPipelineLayout(context, desc.interlockMode) as *const DrawPipelineLayout;
    let mut needsPerFlushBindings = true;
    let mut boundImageTextureView = TextureView::default();
    let mut boundImageSampler = None;
    let drawList = desc.drawList.expect("WebGPU flush source drawList");
    for batch in unsafe { drawList.as_ref() }.iter() {
        let drawType = batch.drawType;
        if batch.barriers != BarrierFlags::none {
            renderPass.barrier(batch);
            if renderPass.encoder().Get() != drawEncoder.Get() {
                *drawEncoder = renderPass.encoder().clone();
                needsPerFlushBindings = true;
                boundImageTextureView = TextureView::default();
                boundImageSampler = None;
            }
        }
        if needsPerFlushBindings {
            unsafe {
                drawEncoder.SetBindGroup(
                    PER_FLUSH_BINDINGS_SET as u32,
                    perFlushBindings.Get(),
                    0,
                    std::ptr::null(),
                )
            };
            needsPerFlushBindings = false;
        }

        let mut imageTextureView = (&*context.m_nullTextureView).clone();
        if let Some(texture) = batch.imageTexture {
            imageTextureView =
                unsafe { (&*texture.as_ptr().cast::<TextureWebGPUImpl>()).textureView() };
        } else if drawType == DrawType::renderPassInitialize
            && desc.colorLoadAction == LoadAction::preserveRenderTarget
        {
            if desc.interlockMode == InterlockMode::atomics && !desc.fixedFunctionColorOutput {
                imageTextureView = renderTarget.targetTextureView();
            } else if desc.interlockMode == InterlockMode::msaa {
                imageTextureView = dstColorTextureView(renderTarget);
            }
        }
        if boundImageTextureView.Get() != imageTextureView.Get()
            || boundImageSampler != Some(batch.imageSampler)
        {
            let mut textureEntry = WGPUBindGroupEntry::default();
            textureEntry.binding = IMAGE_TEXTURE_IDX;
            textureEntry.textureView = imageTextureView.Get();
            let mut samplerEntry = WGPUBindGroupEntry::default();
            samplerEntry.binding = WEBGPU_IMAGE_SAMPLER_IDX;
            samplerEntry.sampler =
                context.m_imageSamplers[batch.imageSampler.asKey() as usize].Get();
            let entries = [textureEntry, samplerEntry];
            let layout = unsafe { &*layoutPointer }.m_bindGroupLayouts[PER_DRAW_BINDINGS_SET].Get();
            let mut descriptor = WGPUBindGroupDescriptor::default();
            descriptor.layout = layout;
            descriptor.entryCount = entries.len();
            descriptor.entries = entries.as_ptr();
            let bindings = unsafe { context.m_device.CreateBindGroup(&descriptor) };
            unsafe {
                drawEncoder.SetBindGroup(
                    PER_DRAW_BINDINGS_SET as u32,
                    bindings.Get(),
                    0,
                    std::ptr::null(),
                )
            };
            boundImageTextureView = imageTextureView;
            boundImageSampler = Some(batch.imageSampler);
        }

        let targetIsGLFBO0 = context.m_capabilities.backendType == BackendType::OpenGLES
            && unsafe {
                super::webgpu_wagyu_decl::wgpuWagyuTextureIsSwapchain(
                    renderTarget.m_targetTexture.Get(),
                ) == super::webgpu_decl::WGPU_TRUE
            };
        let shaderFeatures = if desc.interlockMode == InterlockMode::atomics {
            desc.combinedShaderFeatures
        } else {
            batch.shaderFeatures
        };
        let mut shaderMiscFlags = batch.shaderMiscFlags;
        if desc.interlockMode == InterlockMode::atomics && !desc.fixedFunctionColorOutput {
            if drawType == DrawType::renderPassInitialize {
                if desc.colorLoadAction == LoadAction::clear {
                    shaderMiscFlags |= ShaderMiscFlags::storeColorClear;
                } else if desc.colorLoadAction == LoadAction::preserveRenderTarget {
                    shaderMiscFlags |= ShaderMiscFlags::loadColorFromDstTexture;
                }
            } else if drawType == DrawType::renderPassResolve {
                shaderMiscFlags |= ShaderMiscFlags::coalescedResolveAndTransfer;
            }
        }
        let mut pipelineState =
            crate::mechanical_port::source::renderer::src::gpu_cpp::get_pipeline_state(
                drawType,
                desc.interlockMode,
                batch.shaderMiscFlags,
                batch.drawContents,
                desc.fixedFunctionColorOutput,
                batch.firstBlendMode,
                context.platformFeatures(),
            );
        if desc.interlockMode == InterlockMode::atomics && !desc.fixedFunctionColorOutput {
            assert!(!pipelineState.colorWriteEnabled);
            if drawType == DrawType::renderPassResolve {
                assert!(shaderMiscFlags.has(ShaderMiscFlags::coalescedResolveAndTransfer));
                pipelineState.colorWriteEnabled = true;
            }
        } else if matches!(
            desc.interlockMode,
            InterlockMode::rasterOrdering | InterlockMode::clockwise
        ) && !desc.fixedFunctionColorOutput
        {
            assert!(!pipelineState.colorWriteEnabled);
            pipelineState.colorWriteEnabled = true;
        }
        let mut pipelineKey =
            crate::mechanical_port::source::renderer::src::gpu_cpp::pipeline_unique_key(
                drawType,
                shaderFeatures,
                desc.interlockMode,
                shaderMiscFlags,
                batch.drawContents,
                desc.fixedFunctionColorOutput,
                batch.firstBlendMode,
                context.platformFeatures(),
            );
        pipelineKey = pipelineKey << 1 | u64::from(targetIsGLFBO0);
        if !context.m_drawPipelines.contains_key(&pipelineKey) {
            let pipeline = newDrawPipeline(
                context,
                drawType,
                shaderFeatures,
                desc.interlockMode,
                shaderMiscFlags,
                &pipelineState,
                targetIsGLFBO0,
            );
            context.m_drawPipelines.insert(pipelineKey, pipeline);
        }
        let pipeline = drawPipelineForFormat(
            context
                .m_drawPipelines
                .get(&pipelineKey)
                .expect("draw pipeline"),
            renderTarget.framebufferFormat(),
        );
        unsafe { drawEncoder.SetPipeline(pipeline.Get()) };
        if pipelineState.stencilTestEnabled {
            unsafe { drawEncoder.SetStencilReference(u32::from(pipelineState.stencilReference)) };
        }

        match drawType {
            DrawType::midpointFanPatches
            | DrawType::midpointFanCenterAAPatches
            | DrawType::outerCurvePatches
            | DrawType::msaaOuterCubics
            | DrawType::msaaStrokes
            | DrawType::msaaMidpointFanBorrowedCoverage
            | DrawType::msaaDynamicMidpointFans
            | DrawType::msaaMidpointFans
            | DrawType::msaaMidpointFanStencilReset
            | DrawType::msaaMidpointFanPathsStencil
            | DrawType::msaaMidpointFanPathsCover => unsafe {
                drawEncoder.SetVertexBuffer(
                    0,
                    context.m_pathPatchVertexBuffer.Get(),
                    0,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.SetIndexBuffer(
                    context.m_pathPatchIndexBuffer.Get(),
                    super::webgpu_cpp_decl::IndexFormat::Uint16.into(),
                    0,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.DrawIndexed(
                    batch.indexCountPerInstance,
                    batch.elementCount,
                    batch.baseIndex,
                    0,
                    batch.baseElement,
                );
            },
            DrawType::clipReset | DrawType::interiorTriangulation | DrawType::featherAtlasBlit => unsafe {
                drawEncoder.SetVertexBuffer(
                    0,
                    webgpuBuffer(rings.triangle).Get(),
                    0,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.Draw(batch.elementCount, 1, batch.baseElement, 0);
            },
            DrawType::imageRect => unsafe {
                assert_eq!(desc.interlockMode, InterlockMode::atomics);
                drawEncoder.SetVertexBuffer(
                    0,
                    context.m_imageRectVertexBuffer.Get(),
                    0,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.SetVertexBuffer(
                    1,
                    webgpuBuffer(rings.imageDrawInstance).Get(),
                    u64::from(batch.baseElement)
                        * std::mem::size_of::<gpu::ImageDrawInstance>() as u64,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.SetIndexBuffer(
                    context.m_imageRectIndexBuffer.Get(),
                    super::webgpu_cpp_decl::IndexFormat::Uint16.into(),
                    0,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.DrawIndexed(
                    batch.indexCountPerInstance,
                    batch.elementCount,
                    batch.baseIndex,
                    0,
                    0,
                );
            },
            DrawType::imageMesh => unsafe {
                let vertex = &*batch
                    .vertexBuffer
                    .expect("image mesh vertex buffer")
                    .as_ptr()
                    .cast::<RenderBufferWebGPUImpl>();
                let uv = &*batch
                    .uvBuffer
                    .expect("image mesh uv buffer")
                    .as_ptr()
                    .cast::<RenderBufferWebGPUImpl>();
                let index = &*batch
                    .indexBuffer
                    .expect("image mesh index buffer")
                    .as_ptr()
                    .cast::<RenderBufferWebGPUImpl>();
                drawEncoder.SetVertexBuffer(
                    0,
                    vertex.submittedBuffer().Get(),
                    0,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.SetVertexBuffer(
                    1,
                    uv.submittedBuffer().Get(),
                    0,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.SetVertexBuffer(
                    2,
                    webgpuBuffer(rings.imageDrawInstance).Get(),
                    u64::from(batch.baseElement)
                        * std::mem::size_of::<gpu::ImageDrawInstance>() as u64,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.SetIndexBuffer(
                    index.submittedBuffer().Get(),
                    super::webgpu_cpp_decl::IndexFormat::Uint16.into(),
                    0,
                    super::webgpu_decl::WGPU_WHOLE_SIZE,
                );
                drawEncoder.DrawIndexed(
                    batch.indexCountPerInstance,
                    batch.elementCount,
                    batch.baseIndex,
                    0,
                    0,
                );
            },
            DrawType::renderPassInitialize | DrawType::renderPassResolve => unsafe {
                drawEncoder.Draw(4, 1, 0, 0)
            },
        }
    }
}

pub(crate) unsafe fn flush(
    context: &mut RenderContextWebGPUImpl,
    desc: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
) {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        self as gpu, InterlockMode,
    };
    let renderTarget = unsafe {
        &mut *desc
            .renderTarget
            .expect("flush renderTarget")
            .as_ptr()
            .cast::<RenderTargetWebGPU>()
    };
    let external = desc
        .externalCommandBuffer
        .expect("WebGPU flush requires an external command buffer");
    let commandEncoder = unsafe {
        CommandEncoder::FromBorrowed(
            external
                .as_ptr()
                .cast::<super::webgpu_decl::WGPUCommandEncoderImpl>(),
        )
    };
    let rings = flushBufferRings(context);

    if context.m_capabilities.polyfillVertexStorageBuffers {
        if desc.pathCount > 0 {
            unsafe {
                updateWebGPUStorageTexture::<gpu::PathData>(
                    rings.path,
                    desc.pathCount,
                    desc.firstPath,
                    &commandEncoder,
                );
                updateWebGPUStorageTexture::<gpu::PaintData>(
                    rings.paint,
                    desc.pathCount,
                    desc.firstPaint,
                    &commandEncoder,
                );
                updateWebGPUStorageTexture::<gpu::PaintAuxData>(
                    rings.paintAux,
                    desc.pathCount,
                    desc.firstPaintAux,
                    &commandEncoder,
                );
            }
        }
        if desc.contourCount > 0 {
            unsafe {
                updateWebGPUStorageTexture::<gpu::ContourData>(
                    rings.contour,
                    desc.contourCount,
                    desc.firstContour,
                    &commandEncoder,
                )
            };
        }
    }

    let mut perFlushEntries: [WGPUBindGroupEntry; RenderContextWebGPUImpl::DRAW_BINDINGS_COUNT] =
        std::array::from_fn(|_| WGPUBindGroupEntry::default());
    perFlushEntries[0].binding = FLUSH_UNIFORM_BUFFER_IDX;
    perFlushEntries[0].buffer = unsafe { webgpuBuffer(rings.flush).Get() };
    perFlushEntries[0].offset = desc.flushUniformDataOffsetInBytes as u64;
    for (index, binding, ring, offset) in [
        (
            1,
            PATH_BUFFER_IDX,
            rings.path,
            desc.firstPath * std::mem::size_of::<gpu::PathData>(),
        ),
        (
            2,
            PAINT_BUFFER_IDX,
            rings.paint,
            desc.firstPaint * std::mem::size_of::<gpu::PaintData>(),
        ),
        (
            3,
            PAINT_AUX_BUFFER_IDX,
            rings.paintAux,
            desc.firstPaintAux * std::mem::size_of::<gpu::PaintAuxData>(),
        ),
        (
            4,
            CONTOUR_BUFFER_IDX,
            rings.contour,
            desc.firstContour * std::mem::size_of::<gpu::ContourData>(),
        ),
    ] {
        perFlushEntries[index].binding = binding;
        if context.m_capabilities.polyfillVertexStorageBuffers {
            perFlushEntries[index].textureView = unsafe { webgpuStorageTextureView(ring).Get() };
        } else {
            perFlushEntries[index].buffer = unsafe { webgpuBuffer(ring).Get() };
            perFlushEntries[index].offset = offset as u64;
        }
    }
    for (index, binding, view) in [
        (
            5,
            GAUSSIAN_INTEGRAL_TEXTURE_IDX,
            context.m_gaussianIntegralTextureView.Get(),
        ),
        (
            6,
            TESS_VERTEX_TEXTURE_IDX,
            context.m_tessVertexTextureView.Get(),
        ),
        (
            7,
            FEATHER_ATLAS_TEXTURE_IDX,
            context.m_featherAtlasTextureView.Get(),
        ),
        (8, GRAD_TEXTURE_IDX, context.m_gradientTextureView.Get()),
        (
            9,
            DST_COLOR_TEXTURE_IDX,
            if desc.interlockMode == InterlockMode::msaa && !desc.fixedFunctionColorOutput {
                dstColorTextureView(renderTarget).Get()
            } else {
                context.m_nullTextureView.Get()
            },
        ),
    ] {
        perFlushEntries[index].binding = binding;
        perFlushEntries[index].textureView = view;
    }
    unsafe { encodeOffscreenPasses(context, desc, &commandEncoder, &perFlushEntries, rings) };

    let contextPointer = context as *mut RenderContextWebGPUImpl;
    let mut renderPass =
        unsafe { makeDrawRenderPass(contextPointer, desc, commandEncoder.clone()) };
    let mut drawEncoder = renderPass.encoder().clone();
    let usingPLS = matches!(
        desc.interlockMode,
        InterlockMode::rasterOrdering | InterlockMode::clockwise
    );
    let usingInputAttachmentBindings = usingPLS
        && context.m_capabilities.plsType
            == PixelLocalStorageType::VK_EXT_rasterization_order_attachment_access;
    if usingInputAttachmentBindings {
        let views = [
            renderTarget.targetTextureView(),
            clipTextureView(renderTarget),
            scratchColorTextureView(renderTarget),
            coverageTextureView(renderTarget),
        ];
        let entries: [WGPUBindGroupEntry; PLS_PLANE_COUNT] = std::array::from_fn(|index| {
            let mut entry = WGPUBindGroupEntry::default();
            entry.binding = index as u32;
            entry.textureView = views[index].Get();
            entry
        });
        let layout = drawPipelineLayout(context, desc.interlockMode).m_bindGroupLayouts
            [PLS_TEXTURE_BINDINGS_SET]
            .Get();
        let mut groupDescriptor = WGPUBindGroupDescriptor::default();
        groupDescriptor.layout = layout;
        groupDescriptor.entryCount = entries.len();
        groupDescriptor.entries = entries.as_ptr();
        let bindings = unsafe { context.m_device.CreateBindGroup(&groupDescriptor) };
        unsafe {
            drawEncoder.SetBindGroup(
                PLS_TEXTURE_BINDINGS_SET as u32,
                bindings.Get(),
                0,
                std::ptr::null(),
            )
        };
    }

    let usingShaderPixelLocalStorageEXT = usingPLS
        && context.m_capabilities.plsType
            == PixelLocalStorageType::GL_EXT_shader_pixel_local_storage;
    if usingShaderPixelLocalStorageEXT {
        if desc.fixedFunctionColorOutput {
            assert!(context.m_capabilities.GL_EXT_shader_pixel_local_storage2);
            let clearValues = [desc.coverageClearValue, 0];
            unsafe {
                super::webgpu_wagyu_decl::wgpuWagyuRenderPassEncoderClearPixelLocalStorage(
                    drawEncoder.Get(),
                    0,
                    clearValues.len() as u32,
                    clearValues.as_ptr(),
                )
            };
        } else {
            let mut clearColor = [0.0; 4];
            let loadActions = BuildLoadActionsEXT(desc, &mut clearColor);
            let key = loadStoreEXTPipelineKey(loadActions, renderTarget.framebufferFormat());
            if !context.m_loadStoreEXTPipelines.contains_key(&key) {
                let pipeline =
                    newLoadStoreEXTPipeline(context, loadActions, renderTarget.framebufferFormat());
                context.m_loadStoreEXTPipelines.insert(key, pipeline);
            }
            let (layout, pipeline) = {
                let selected = context
                    .m_loadStoreEXTPipelines
                    .get(&key)
                    .expect("load pipeline");
                (
                    selected.m_bindGroupLayout.clone(),
                    selected.m_renderPipeline.clone(),
                )
            };
            if loadActions.has(LoadStoreActionsEXT::clearColor) {
                let uniforms = context
                    .m_loadStoreEXTUniforms
                    .as_mut()
                    .expect("load/store EXT uniforms");
                let mapped = uniforms.mapBuffer(std::mem::size_of_val(&clearColor));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        clearColor.as_ptr().cast::<u8>(),
                        mapped.cast::<u8>(),
                        std::mem::size_of_val(&clearColor),
                    )
                };
                uniforms.unmapAndSubmitBuffer();
                let mut entry = WGPUBindGroupEntry::default();
                entry.buffer = uniforms.submittedBuffer().Get();
                let mut groupDescriptor = WGPUBindGroupDescriptor::default();
                groupDescriptor.layout = layout.Get();
                groupDescriptor.entryCount = 1;
                groupDescriptor.entries = &entry;
                let bindings = unsafe { context.m_device.CreateBindGroup(&groupDescriptor) };
                unsafe { drawEncoder.SetBindGroup(0, bindings.Get(), 0, std::ptr::null()) };
            }
            unsafe {
                drawEncoder.SetPipeline(pipeline.Get());
                drawEncoder.Draw(4, 1, 0, 0);
            }
        }
    }

    let perFlushLayout = drawPipelineLayout(context, desc.interlockMode).m_bindGroupLayouts
        [PER_FLUSH_BINDINGS_SET]
        .Get();
    let mut groupDescriptor = WGPUBindGroupDescriptor::default();
    groupDescriptor.layout = perFlushLayout;
    groupDescriptor.entryCount = perFlushEntries.len();
    groupDescriptor.entries = perFlushEntries.as_ptr();
    let perFlushBindings = unsafe { context.m_device.CreateBindGroup(&groupDescriptor) };

    executeDrawList(
        context,
        desc,
        renderTarget,
        rings,
        &mut *renderPass,
        &mut drawEncoder,
        &perFlushBindings,
    );

    if usingShaderPixelLocalStorageEXT && !desc.fixedFunctionColorOutput {
        let actions = LoadStoreActionsEXT::storeColor;
        let key = loadStoreEXTPipelineKey(actions, renderTarget.framebufferFormat());
        if !context.m_loadStoreEXTPipelines.contains_key(&key) {
            let pipeline =
                newLoadStoreEXTPipeline(context, actions, renderTarget.framebufferFormat());
            context.m_loadStoreEXTPipelines.insert(key, pipeline);
        }
        let pipeline = context
            .m_loadStoreEXTPipelines
            .get(&key)
            .expect("store pipeline")
            .m_renderPipeline
            .clone();
        unsafe {
            drawEncoder.SetPipeline(pipeline.Get());
            drawEncoder.Draw(4, 1, 0, 0);
        }
    }
}

pub(crate) fn makeRenderCanvas(
    context: &mut RenderContextWebGPUImpl,
    width: u32,
    height: u32,
) -> rcp<crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas>
{
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
    use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;

    let texture = makeTexture(
        &context.m_device,
        (TextureUsage::TextureBinding | TextureUsage::RenderAttachment | TextureUsage::CopySrc)
            .intoBitmask(),
        width,
        height,
        TextureFormat::RGBA8Unorm,
        1,
    );
    let texture = make_rcp(|| *TextureWebGPUImpl::new(width, height, texture));
    let mut renderTarget = makeRenderTarget(context, TextureFormat::RGBA8Unorm, width, height);
    unsafe {
        (&mut *renderTarget.get())
            .setTargetTextureView((&*texture.get()).textureView(), (&*texture.get()).texture())
    };
    let texture: rcp<RiveTexture> = unsafe { static_rcp_cast(texture) };
    let renderImage = make_rcp(|| unsafe { RiveRenderImage::new(texture) });
    let renderTarget: rcp<RenderTarget> = unsafe { static_rcp_cast(renderTarget) };
    make_rcp(|| unsafe { RenderCanvas::new(renderImage, renderTarget) })
}

pub(crate) fn makeOreContext(
    context: &RenderContextWebGPUImpl,
) -> Option<Box<super::ore_context_wgpu_decl::ContextWGPU>> {
    super::ore_context_wgpu_decl::ContextWGPU::Make(
        (&*context.m_device).clone(),
        (&*context.m_queue).clone(),
        context.m_capabilities.backendType,
    )
}

pub(crate) fn makeCommandBuffer(context: &mut RenderContextWebGPUImpl) -> *mut core::ffi::c_void {
    let encoder = unsafe { context.m_device.CreateCommandEncoder(std::ptr::null()) };
    encoder.MoveToCHandle().cast()
}

pub(crate) unsafe fn commitCommandBuffer(
    context: &mut RenderContextWebGPUImpl,
    commandBuffer: *mut core::ffi::c_void,
) {
    if commandBuffer.is_null() {
        return;
    }
    let encoder = unsafe { CommandEncoder::Acquire(commandBuffer.cast()) };
    let commands = unsafe { encoder.Finish(std::ptr::null()) };
    let command = commands.Get();
    unsafe { context.m_queue.Submit(1, &command) };
}

impl RenderContextHelperImplAccess for RenderContextWebGPUImpl {
    fn renderContextHelperImpl(&self) -> &RenderContextHelperImpl {
        &self.base
    }

    fn renderContextHelperImplMut(&mut self) -> &mut RenderContextHelperImpl {
        &mut self.base
    }
}

impl RenderContextHelperBufferFactoryContract for RenderContextWebGPUImpl {
    fn makeUniformBufferRing(
        &mut self,
        capacityInBytes: usize,
    ) -> Option<Box<dyn BufferRingContract>> {
        let capacity = capacityInBytes.max(256);
        assert_eq!(capacity % 256, 0);
        Some(Box::new(BufferWebGPU::new(
            (&*self.m_device).clone(),
            (&*self.m_queue).clone(),
            capacity,
            super::webgpu_cpp_decl::BufferUsage::Uniform,
        )))
    }

    fn makeStorageBufferRing(
        &mut self,
        capacityInBytes: usize,
        bufferStructure: StorageBufferStructure,
    ) -> Option<Box<dyn BufferRingContract>> {
        if self.m_capabilities.polyfillVertexStorageBuffers {
            Some(Box::new(StorageTextureBufferWebGPU::new(
                (&*self.m_device).clone(),
                (&*self.m_queue).clone(),
                capacityInBytes,
                bufferStructure,
            )))
        } else {
            Some(Box::new(BufferWebGPU::new(
                (&*self.m_device).clone(),
                (&*self.m_queue).clone(),
                capacityInBytes,
                super::webgpu_cpp_decl::BufferUsage::Storage,
            )))
        }
    }

    fn makeVertexBufferRing(
        &mut self,
        capacityInBytes: usize,
    ) -> Option<Box<dyn BufferRingContract>> {
        Some(Box::new(BufferWebGPU::new(
            (&*self.m_device).clone(),
            (&*self.m_queue).clone(),
            capacityInBytes,
            super::webgpu_cpp_decl::BufferUsage::Vertex,
        )))
    }
}

impl RenderContextHelperBackendContract for RenderContextWebGPUImpl {
    fn makeRenderBuffer(
        &mut self,
        ty: RenderBufferType,
        flags: RenderBufferFlags,
        bytes: usize,
    ) -> rcp<RenderBuffer> {
        makeRenderBuffer(self, ty, flags, bytes)
    }

    fn makeImageTexture(
        &mut self,
        width: u32,
        height: u32,
        levels: u32,
        format: GPUTextureFormat,
        data: &[u8],
        blockWidth: u8,
        blockHeight: u8,
        srgb: bool,
        generateRemainingMips: bool,
    ) -> rcp<RiveTexture> {
        makeImageTexture(
            self,
            width,
            height,
            levels,
            format,
            data,
            blockWidth,
            blockHeight,
            srgb,
            generateRemainingMips,
        )
    }

    #[cfg(any(
        feature = "native-ore-metal-experimental",
        feature = "native-ore-vulkan-experimental",
        feature = "ore-gl"
    ))]
    fn makeOreContext(
        &mut self,
    ) -> Option<Box<crate::mechanical_port::source::include::rive::factory_hpp::OreContext>> {
        makeOreContext(self).map(|context| {
            Box::new(
                crate::mechanical_port::source::include::rive::factory_hpp::OreContext::WGPU(
                    context,
                ),
            )
        })
    }

    fn resizeGradientTexture(&mut self, width: u32, height: u32) {
        resizeGradientTexture(self, width, height)
    }

    fn resizeTessellationTexture(&mut self, width: u32, height: u32) {
        resizeTessellationTexture(self, width, height)
    }

    fn resizeFeatherAtlasTexture(&mut self, width: u32, height: u32) {
        resizeFeatherAtlasTexture(self, width, height)
    }

    fn resizeAtomicCoverageBacking(&mut self, width: u32, height: u32) {
        resizeAtomicCoverageBacking(self, width, height)
    }

    unsafe fn flush(
        &mut self,
        descriptor: &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::FlushDescriptor,
    ) {
        unsafe { flush(self, descriptor) }
    }

    fn makeCommandBuffer(&mut self) -> *mut core::ffi::c_void {
        makeCommandBuffer(self)
    }

    unsafe fn commitCommandBuffer(&mut self, commandBuffer: *mut core::ffi::c_void) {
        unsafe { commitCommandBuffer(self, commandBuffer) }
    }
}

pub(crate) fn MakeContext(
    adapter: Adapter,
    device: Device,
    queue: Queue,
    contextOptions: ContextOptions,
) -> std::pin::Pin<Box<RenderContext>> {
    let mut implementation = Box::new(newContext(adapter, device, queue, contextOptions));
    initGPUObjects(&mut implementation);
    <RenderContext as RenderContextContract>::new(implementation)
}

pub(crate) const SOURCE_CPP_LINE_COUNT: usize = 4837;
pub(crate) const SOURCE_TOP_LEVEL_HELPER_COUNT: usize = 14;
const _: [(); 192817] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATED_GLSL_PRODUCTS: [&str; 17] = [
        include_str!("source/generated_glsl/advanced_blend.minified.glsl"),
        include_str!("source/generated_glsl/bezier_utils.minified.glsl"),
        include_str!("source/generated_glsl/color_ramp.minified.glsl"),
        include_str!("source/generated_glsl/common.minified.glsl"),
        include_str!("source/generated_glsl/constants.minified.glsl"),
        include_str!("source/generated_glsl/draw_clockwise_clip.minified.frag"),
        include_str!("source/generated_glsl/draw_clockwise_path.minified.frag"),
        include_str!("source/generated_glsl/draw_image_mesh.minified.vert"),
        include_str!("source/generated_glsl/draw_mesh.minified.frag"),
        include_str!("source/generated_glsl/draw_path.minified.vert"),
        include_str!("source/generated_glsl/draw_path_common.minified.glsl"),
        include_str!("source/generated_glsl/draw_raster_order_path.minified.frag"),
        include_str!("source/generated_glsl/flush_uniforms.minified.glsl"),
        include_str!("source/generated_glsl/glsl.minified.glsl"),
        include_str!("source/generated_glsl/pls_load_store_ext.minified.glsl"),
        include_str!("source/generated_glsl/render_atlas.minified.glsl"),
        include_str!("source/generated_glsl/tessellate.minified.glsl"),
    ];

    #[test]
    fn generated_glsl_output_denominator_is_frozen() {
        assert_eq!(GENERATED_GLSL_PRODUCTS.len(), 17);
        assert_eq!(
            GENERATED_GLSL_PRODUCTS
                .iter()
                .map(|source| source.len())
                .sum::<usize>(),
            50_515
        );
    }

    #[test]
    fn minified_export_names_match_the_frozen_map() {
        let exports = include_str!("source/generated_glsl/glsl.glsl.exports.h");
        for (sourceName, generatedName) in [
            ("VERTEX", GLSL_VERTEX),
            ("FRAGMENT", GLSL_FRAGMENT),
            ("POST_INVERT_Y", GLSL_POST_INVERT_Y),
            (
                "DISABLE_SHADER_STORAGE_BUFFERS",
                GLSL_DISABLE_SHADER_STORAGE_BUFFERS,
            ),
            ("DRAW_PATH", GLSL_DRAW_PATH),
            ("ENABLE_FEATHER", GLSL_ENABLE_FEATHER),
            ("ENABLE_INSTANCE_INDEX", GLSL_ENABLE_INSTANCE_INDEX),
            (
                "BASE_INSTANCE_UNIFORM_NAME",
                GLSL_BASE_INSTANCE_UNIFORM_NAME,
            ),
            ("ATLAS_FEATHERED_FILL", GLSL_ATLAS_FEATHERED_FILL),
            ("ATLAS_FEATHERED_STROKE", GLSL_ATLAS_FEATHERED_STROKE),
            ("CLEAR_COLOR", GLSL_CLEAR_COLOR),
            ("LOAD_COLOR", GLSL_LOAD_COLOR),
            ("STORE_COLOR", GLSL_STORE_COLOR),
            ("CLEAR_COVERAGE", GLSL_CLEAR_COVERAGE),
            ("CLEAR_CLIP", GLSL_CLEAR_CLIP),
            ("ENABLE_CLIPPING", GLSL_ENABLE_CLIPPING),
        ] {
            assert!(exports.contains(&format!("#define GLSL_{sourceName} \"{generatedName}\"")));
        }
    }

    #[test]
    fn load_store_pipeline_key_preserves_all_action_bits() {
        let actions = LoadStoreActionsEXT(
            LoadStoreActionsEXT::clearColor.0
                | LoadStoreActionsEXT::loadColor.0
                | LoadStoreActionsEXT::storeColor.0
                | LoadStoreActionsEXT::clearCoverage.0
                | LoadStoreActionsEXT::clearClip.0,
        );
        let key = loadStoreEXTPipelineKey(actions, TextureFormat::RGBA8Unorm);
        assert_eq!(key & 0x1f, 0x1f);
        assert_eq!(key >> 5, TextureFormat::RGBA8Unorm.0);
    }
}
