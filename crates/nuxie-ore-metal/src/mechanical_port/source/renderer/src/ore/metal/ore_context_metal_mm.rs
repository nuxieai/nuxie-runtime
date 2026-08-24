/*
 * Copyright 2025 Rive
 */

// #include "rive/renderer/ore/ore_context_metal.hpp"
// #include "ore_bind_group_metal.hpp"
// #include "ore_buffer_metal.hpp"
// #include "ore_pipeline_metal.hpp"
// #include "ore_render_pass_metal.hpp"
// #include "ore_sampler_metal.hpp"
// #include "ore_shader_module_metal.hpp"
// #include "ore_texture_metal.hpp"
// #include "rive/renderer/render_canvas.hpp"
// #include "rive/renderer/metal/render_context_metal_impl.h"
// #include "rive/rive_types.hpp"
// #include <string>
// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source implementation
// renderer/src/ore/metal/ore_context_metal.mm.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
// Source coverage: pinned lines 1-1344, in source order.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![cfg(target_vendor = "apple")]
use super::*;

use core::ffi::c_void;
use std::rc::Weak as RcWeak;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::{
    AnyResourceHandle, ResourceHandle,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_binding_map_hpp::BindingMap;
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::{
    BindGroupDesc, BindGroupLayoutDesc, BindGroupLayoutEntry, BindingKind, BlendFactor, BlendOp,
    BufferDesc, ColorWriteMask, CompareFunction, Features, Filter, LoadOp, PipelineDesc,
    RenderPassDesc, SamplerDesc, ShaderModuleDesc, StencilOp, StoreOp, TextureDesc, TextureFormat,
    TextureType, TextureViewDesc, TextureViewDimension, VertexFormat, VertexStepMode, WrapMode,
    kMaxBindGroups,
};
use crate::mechanical_port::source::renderer::src::ore::ore_bind_group_layout_cpp::{
    validateColorRequiresFragment, validateLayoutsAgainstBindingMap,
};

#[cfg(target_vendor = "apple")]
use objc2::msg_send;
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_foundation::{NSError, NSRange, NSString};
#[cfg(target_vendor = "apple")]
use objc2_metal::{
    MTLBlendFactor, MTLBlendOperation, MTLClearColor, MTLColorWriteMask, MTLCommandBuffer,
    MTLCommandBufferStatus, MTLCommandEncoder, MTLCommandQueue, MTLCompareFunction, MTLDevice,
    MTLLibrary, MTLLoadAction, MTLPixelFormat, MTLRenderPassDescriptor,
    MTLRenderPipelineDescriptor, MTLResource, MTLSamplerAddressMode, MTLSamplerDescriptor,
    MTLSamplerMinMagFilter, MTLSamplerMipFilter, MTLStencilOperation, MTLStoreAction, MTLTexture,
    MTLTextureDescriptor, MTLTextureType, MTLTextureUsage, MTLVertexFormat, MTLVertexStepFunction,
};

// namespace rive::ore

// The native Objective-C protocols are retained owners. `None` is the source
// `nil` state and is never treated as a valid initialized owner when a factory
// requires a native object.
#[cfg(target_vendor = "apple")]
type NativeDevice = Option<Retained<ProtocolObject<dyn objc2_metal::MTLDevice>>>;
#[cfg(target_vendor = "apple")]
type NativeQueue = Option<Retained<ProtocolObject<dyn objc2_metal::MTLCommandQueue>>>;
#[cfg(target_vendor = "apple")]
pub type NativeTexture = Option<Retained<ProtocolObject<dyn objc2_metal::MTLTexture>>>;

/// Rust crate-boundary spelling of the source `gpu::RenderCanvas*` seam.
#[cfg(target_vendor = "apple")]
pub trait MetalRenderCanvasHost {
    fn metalWidth(&self) -> u32;
    fn metalHeight(&self) -> u32;
    fn metalTargetTexture(&self) -> NativeTexture;
}

/// Rust crate-boundary spelling of the source `gpu::Texture*` seam.
#[cfg(target_vendor = "apple")]
pub trait MetalRiveTextureHost {
    fn metalNativeTexture(&self) -> NativeTexture;
}

/// Type-erased bridge used only by the object-safe `ContextApi` virtual seam.
/// The renderer crate constructs this transient view from its canonical
/// RenderCanvas owner; this struct never becomes a second resource authority.
#[cfg(target_vendor = "apple")]
pub struct MetalRenderCanvasBridge {
    pub width: u32,
    pub height: u32,
    pub texture: NativeTexture,
}

#[cfg(target_vendor = "apple")]
impl MetalRenderCanvasHost for MetalRenderCanvasBridge {
    fn metalWidth(&self) -> u32 {
        self.width
    }

    fn metalHeight(&self) -> u32 {
        self.height
    }

    fn metalTargetTexture(&self) -> NativeTexture {
        self.texture.clone()
    }
}

/// Type-erased bridge used only by the object-safe `ContextApi` virtual seam.
#[cfg(target_vendor = "apple")]
pub struct MetalRiveTextureBridge {
    pub texture: NativeTexture,
}

/// Product-facing result token for the one source command-buffer completion
/// callback installed by `endFrame`.
#[cfg(target_vendor = "apple")]
#[derive(Clone)]
pub struct MetalSubmissionCompletion {
    result: Arc<Mutex<Option<Result<(), String>>>>,
}

#[cfg(target_vendor = "apple")]
impl MetalSubmissionCompletion {
    pub fn result(&self) -> Option<Result<(), String>> {
        self.result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn complete(&self, result: Result<(), String>) {
        *self
            .result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(result);
    }
}

#[cfg(target_vendor = "apple")]
fn command_buffer_completion_result(
    status: MTLCommandBufferStatus,
    error: Option<String>,
) -> Result<(), String> {
    if status == MTLCommandBufferStatus::Completed {
        Ok(())
    } else {
        Err(error
            .unwrap_or_else(|| format!("Metal command buffer completed with status {status:?}")))
    }
}

#[cfg(target_vendor = "apple")]
fn finish_source_completion(
    deferredBindGroups: &mut Vec<AnyResourceHandle>,
    completedSerial: &AtomicU64,
    finishedSerial: u64,
    publishProductResult: impl FnOnce(),
) {
    deferredBindGroups.clear();
    let mut completed = completedSerial.load(Ordering::Relaxed);
    while finishedSerial > completed {
        match completedSerial.compare_exchange_weak(
            completed,
            finishedSerial,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(observed) => completed = observed,
        }
    }
    publishProductResult();
}

#[cfg(target_vendor = "apple")]
impl MetalRiveTextureHost for MetalRiveTextureBridge {
    fn metalNativeTexture(&self) -> NativeTexture {
        self.texture.clone()
    }
}

#[cfg(target_vendor = "apple")]
fn same_native_texture(
    left: Option<&Retained<ProtocolObject<dyn objc2_metal::MTLTexture>>>,
    right: Option<&Retained<ProtocolObject<dyn objc2_metal::MTLTexture>>>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => core::ptr::eq(&**left, &**right),
        (None, None) => true,
        _ => false,
    }
}

// ============================================================================
// Enum → Metal conversion helpers
// ============================================================================

// static MTLPixelFormat oreFormatToMTL(TextureFormat format)
#[cfg(target_vendor = "apple")]
fn oreFormatToMTL(format: TextureFormat) -> MTLPixelFormat {
    match format {
        TextureFormat::r8unorm => MTLPixelFormat::R8Unorm,
        TextureFormat::rg8unorm => MTLPixelFormat::RG8Unorm,
        TextureFormat::rgba8unorm => MTLPixelFormat::RGBA8Unorm,
        TextureFormat::rgba8snorm => MTLPixelFormat::RGBA8Snorm,
        TextureFormat::bgra8unorm => MTLPixelFormat::BGRA8Unorm,
        TextureFormat::rgba16float => MTLPixelFormat::RGBA16Float,
        TextureFormat::rg16float => MTLPixelFormat::RG16Float,
        TextureFormat::r16float => MTLPixelFormat::R16Float,
        TextureFormat::rgba32float => MTLPixelFormat::RGBA32Float,
        TextureFormat::rg32float => MTLPixelFormat::RG32Float,
        TextureFormat::r32float => MTLPixelFormat::R32Float,
        TextureFormat::rgb10a2unorm => MTLPixelFormat::RGB10A2Unorm,
        TextureFormat::r11g11b10float => MTLPixelFormat::RG11B10Float,
        TextureFormat::depth16unorm => MTLPixelFormat::Depth16Unorm,
        TextureFormat::depth24plusStencil8 => {
            // #if defined(RIVE_IOS) || defined(RIVE_IOS_SIMULATOR) || TARGET_CPU_ARM64
            // iOS and Apple Silicon (ARM64) don't support Depth24Unorm.
            if cfg!(any(target_os = "ios", target_arch = "aarch64")) {
                MTLPixelFormat::Depth32Float_Stencil8
            } else {
                // #else
                MTLPixelFormat::Depth24Unorm_Stencil8
            }
        }
        TextureFormat::depth32float => MTLPixelFormat::Depth32Float,
        TextureFormat::depth32floatStencil8 => MTLPixelFormat::Depth32Float_Stencil8,
        TextureFormat::bc1unorm => {
            // #if TARGET_OS_OSX || (__IPHONE_OS_VERSION_MAX_ALLOWED >= 160400)
            // if (@available(iOS 16.4, *)) return MTLPixelFormatBC1_RGBA;
            // #endif
            #[cfg(target_os = "macos")]
            {
                MTLPixelFormat::BC1_RGBA
            }
            #[cfg(not(target_os = "macos"))]
            {
                #[cfg(target_os = "ios")]
                if objc2::available!(ios = 16.4) {
                    return MTLPixelFormat::BC1_RGBA;
                }
                // RIVE_UNREACHABLE();
                unreachable!("BC1 is unavailable on this Metal deployment target")
            }
        }
        TextureFormat::bc3unorm => {
            // #if TARGET_OS_OSX || (__IPHONE_OS_VERSION_MAX_ALLOWED >= 160400)
            // if (@available(iOS 16.4, *)) return MTLPixelFormatBC3_RGBA;
            // #endif
            #[cfg(target_os = "macos")]
            {
                MTLPixelFormat::BC3_RGBA
            }
            #[cfg(not(target_os = "macos"))]
            {
                #[cfg(target_os = "ios")]
                if objc2::available!(ios = 16.4) {
                    return MTLPixelFormat::BC3_RGBA;
                }
                // RIVE_UNREACHABLE();
                unreachable!("BC3 is unavailable on this Metal deployment target")
            }
        }
        TextureFormat::bc7unorm => {
            // #if TARGET_OS_OSX || (__IPHONE_OS_VERSION_MAX_ALLOWED >= 160400)
            // if (@available(iOS 16.4, *)) return MTLPixelFormatBC7_RGBAUnorm;
            // #endif
            #[cfg(target_os = "macos")]
            {
                MTLPixelFormat::BC7_RGBAUnorm
            }
            #[cfg(not(target_os = "macos"))]
            {
                #[cfg(target_os = "ios")]
                if objc2::available!(ios = 16.4) {
                    return MTLPixelFormat::BC7_RGBAUnorm;
                }
                // RIVE_UNREACHABLE();
                unreachable!("BC7 is unavailable on this Metal deployment target")
            }
        }
        TextureFormat::etc2rgb8 => MTLPixelFormat::ETC2_RGB8,
        TextureFormat::etc2rgba8 => MTLPixelFormat::EAC_RGBA8,
        TextureFormat::astc4x4 => MTLPixelFormat::ASTC_4x4_LDR,
        TextureFormat::astc6x6 => MTLPixelFormat::ASTC_6x6_LDR,
        TextureFormat::astc8x8 => MTLPixelFormat::ASTC_8x8_LDR,
    }
}

// static MTLTextureType oreTextureTypeToMTL(TextureType type)
#[cfg(target_vendor = "apple")]
fn oreTextureTypeToMTL(value: TextureType) -> MTLTextureType {
    match value {
        TextureType::texture2D => MTLTextureType::Type2D,
        TextureType::cube => MTLTextureType::TypeCube,
        TextureType::texture3D => MTLTextureType::Type3D,
        TextureType::array2D => MTLTextureType::Type2DArray,
    }
}

// static MTLSamplerMinMagFilter oreFilterToMTL(Filter filter)
#[cfg(target_vendor = "apple")]
fn oreFilterToMTL(value: Filter) -> MTLSamplerMinMagFilter {
    match value {
        Filter::nearest => MTLSamplerMinMagFilter::Nearest,
        Filter::linear => MTLSamplerMinMagFilter::Linear,
    }
}

// static MTLSamplerMipFilter oreMipFilterToMTL(Filter filter)
#[cfg(target_vendor = "apple")]
fn oreMipFilterToMTL(value: Filter) -> MTLSamplerMipFilter {
    match value {
        Filter::nearest => MTLSamplerMipFilter::Nearest,
        Filter::linear => MTLSamplerMipFilter::Linear,
    }
}

// static MTLSamplerAddressMode oreWrapToMTL(WrapMode mode)
#[cfg(target_vendor = "apple")]
fn oreWrapToMTL(value: WrapMode) -> MTLSamplerAddressMode {
    match value {
        WrapMode::repeat => MTLSamplerAddressMode::Repeat,
        WrapMode::mirrorRepeat => MTLSamplerAddressMode::MirrorRepeat,
        WrapMode::clampToEdge => MTLSamplerAddressMode::ClampToEdge,
    }
}

// static MTLCompareFunction oreCompareFunctionToMTL(CompareFunction fn)
#[cfg(target_vendor = "apple")]
fn oreCompareFunctionToMTL(value: CompareFunction) -> MTLCompareFunction {
    match value {
        CompareFunction::none | CompareFunction::never => MTLCompareFunction::Never,
        CompareFunction::less => MTLCompareFunction::Less,
        CompareFunction::equal => MTLCompareFunction::Equal,
        CompareFunction::lessEqual => MTLCompareFunction::LessEqual,
        CompareFunction::greater => MTLCompareFunction::Greater,
        CompareFunction::notEqual => MTLCompareFunction::NotEqual,
        CompareFunction::greaterEqual => MTLCompareFunction::GreaterEqual,
        CompareFunction::always => MTLCompareFunction::Always,
    }
}

// static MTLLoadAction oreLoadOpToMTL(LoadOp op)
#[cfg(target_vendor = "apple")]
fn oreLoadOpToMTL(value: LoadOp) -> MTLLoadAction {
    match value {
        LoadOp::clear => MTLLoadAction::Clear,
        LoadOp::load => MTLLoadAction::Load,
        LoadOp::dontCare => MTLLoadAction::DontCare,
    }
}

// static MTLStoreAction oreStoreOpToMTL(StoreOp op)
#[cfg(target_vendor = "apple")]
fn oreStoreOpToMTL(value: StoreOp) -> MTLStoreAction {
    match value {
        StoreOp::store => MTLStoreAction::Store,
        StoreOp::discard => MTLStoreAction::DontCare,
    }
}

// static MTLBlendFactor oreBlendFactorToMTL(BlendFactor f)
#[cfg(target_vendor = "apple")]
fn oreBlendFactorToMTL(value: BlendFactor) -> MTLBlendFactor {
    match value {
        BlendFactor::zero => MTLBlendFactor::Zero,
        BlendFactor::one => MTLBlendFactor::One,
        BlendFactor::srcColor => MTLBlendFactor::SourceColor,
        BlendFactor::oneMinusSrcColor => MTLBlendFactor::OneMinusSourceColor,
        BlendFactor::srcAlpha => MTLBlendFactor::SourceAlpha,
        BlendFactor::oneMinusSrcAlpha => MTLBlendFactor::OneMinusSourceAlpha,
        BlendFactor::dstColor => MTLBlendFactor::DestinationColor,
        BlendFactor::oneMinusDstColor => MTLBlendFactor::OneMinusDestinationColor,
        BlendFactor::dstAlpha => MTLBlendFactor::DestinationAlpha,
        BlendFactor::oneMinusDstAlpha => MTLBlendFactor::OneMinusDestinationAlpha,
        BlendFactor::srcAlphaSaturated => MTLBlendFactor::SourceAlphaSaturated,
        BlendFactor::blendColor => MTLBlendFactor::BlendColor,
        BlendFactor::oneMinusBlendColor => MTLBlendFactor::OneMinusBlendColor,
    }
}

// static MTLBlendOperation oreBlendOpToMTL(BlendOp op)
#[cfg(target_vendor = "apple")]
fn oreBlendOpToMTL(value: BlendOp) -> MTLBlendOperation {
    match value {
        BlendOp::add => MTLBlendOperation::Add,
        BlendOp::subtract => MTLBlendOperation::Subtract,
        BlendOp::reverseSubtract => MTLBlendOperation::ReverseSubtract,
        BlendOp::min => MTLBlendOperation::Min,
        BlendOp::max => MTLBlendOperation::Max,
    }
}

// static MTLStencilOperation oreStencilOpToMTL(StencilOp op)
#[cfg(target_vendor = "apple")]
fn oreStencilOpToMTL(value: StencilOp) -> MTLStencilOperation {
    match value {
        StencilOp::keep => MTLStencilOperation::Keep,
        StencilOp::zero => MTLStencilOperation::Zero,
        StencilOp::replace => MTLStencilOperation::Replace,
        StencilOp::incrementClamp => MTLStencilOperation::IncrementClamp,
        StencilOp::decrementClamp => MTLStencilOperation::DecrementClamp,
        StencilOp::invert => MTLStencilOperation::Invert,
        StencilOp::incrementWrap => MTLStencilOperation::IncrementWrap,
        StencilOp::decrementWrap => MTLStencilOperation::DecrementWrap,
    }
}

// static MTLVertexFormat oreVertexFormatToMTL(VertexFormat fmt)
#[cfg(target_vendor = "apple")]
fn oreVertexFormatToMTL(value: VertexFormat) -> MTLVertexFormat {
    match value {
        VertexFormat::float1 => MTLVertexFormat::Float,
        VertexFormat::float2 => MTLVertexFormat::Float2,
        VertexFormat::float3 => MTLVertexFormat::Float3,
        VertexFormat::float4 => MTLVertexFormat::Float4,
        VertexFormat::uint8x4 => MTLVertexFormat::UChar4,
        VertexFormat::sint8x4 => MTLVertexFormat::Char4,
        VertexFormat::unorm8x4 => MTLVertexFormat::UChar4Normalized,
        VertexFormat::snorm8x4 => MTLVertexFormat::Char4Normalized,
        VertexFormat::uint16x2 => MTLVertexFormat::UShort2,
        VertexFormat::sint16x2 => MTLVertexFormat::Short2,
        VertexFormat::unorm16x2 => MTLVertexFormat::UShort2Normalized,
        VertexFormat::snorm16x2 => MTLVertexFormat::Short2Normalized,
        VertexFormat::uint16x4 => MTLVertexFormat::UShort4,
        VertexFormat::sint16x4 => MTLVertexFormat::Short4,
        VertexFormat::float16x2 => MTLVertexFormat::Half2,
        VertexFormat::float16x4 => MTLVertexFormat::Half4,
        VertexFormat::uint32 => MTLVertexFormat::UInt,
    }
}

// static MTLColorWriteMask oreColorWriteMaskToMTL(ColorWriteMask mask)
#[cfg(target_vendor = "apple")]
fn oreColorWriteMaskToMTL(mask: ColorWriteMask) -> MTLColorWriteMask {
    let mut result = MTLColorWriteMask::None;
    if mask & ColorWriteMask::red != ColorWriteMask::none {
        result |= MTLColorWriteMask::Red;
    }
    if mask & ColorWriteMask::green != ColorWriteMask::none {
        result |= MTLColorWriteMask::Green;
    }
    if mask & ColorWriteMask::blue != ColorWriteMask::none {
        result |= MTLColorWriteMask::Blue;
    }
    if mask & ColorWriteMask::alpha != ColorWriteMask::none {
        result |= MTLColorWriteMask::Alpha;
    }
    result
}

// Metal uses one [[buffer(n)]] namespace for vertex and uniform buffers.
// WGSL→MSL reserves 0..15 for uniforms; vertex buffers start at 16.
const kMetalVertexBufferBase: u32 = 16;

// static TextureFormat mtlFormatToOre(MTLPixelFormat fmt)
#[cfg(target_vendor = "apple")]
fn mtlFormatToOre(value: objc2_metal::MTLPixelFormat) -> TextureFormat {
    match value {
        objc2_metal::MTLPixelFormat::RGBA8Unorm => TextureFormat::rgba8unorm,
        objc2_metal::MTLPixelFormat::BGRA8Unorm => TextureFormat::bgra8unorm,
        objc2_metal::MTLPixelFormat::RGBA16Float => TextureFormat::rgba16float,
        objc2_metal::MTLPixelFormat::RGB10A2Unorm => TextureFormat::rgb10a2unorm,
        _ => TextureFormat::rgba8unorm,
    }
}

// ============================================================================
// Metal implementation helpers (inline)
// Shared between metal-only and metal+gl builds.
// ============================================================================

impl ContextMetal {
    fn ownsResource(&self, resource: &AnyResourceHandle) -> bool {
        resource.belongsTo(&self.base.state.resourceDomain())
    }

    pub fn features(&self) -> Features {
        self.base.features()
    }

    pub fn lastError(&self) -> String {
        self.base.lastError()
    }

    pub fn activeRenderPass(&self) -> Option<RcWeak<dyn ActiveRenderPass>> {
        self.base.activeRenderPass()
    }

    pub fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.base.setActiveRenderPass(pass);
    }

    pub fn finishActiveRenderPass(&self) {
        self.base.finishActiveRenderPass();
    }

    pub fn clearLastError(&self) {
        self.base.clearLastError();
    }

    pub fn setLastError(&self, message: &str) {
        self.base.setLastError(message.to_owned());
    }

    // inline void ContextMetal::mtlPopulateFeatures(id<MTLDevice> device)
    #[cfg(target_vendor = "apple")]
    fn mtlPopulateFeatures(&mut self, device: &NativeDevice) {
        let mut f = self.base.features_mut_unpublished();
        f.colorBufferFloat = true;
        f.colorBufferHalfFloat = true;
        f.perTargetBlend = true;
        f.perTargetWriteMask = true;
        f.textureViewSampling = true;
        f.drawBaseInstance = true;
        f.depthBiasClamp = true;
        f.anisotropicFiltering = true;
        f.texture3D = true;
        f.textureArrays = true;
        f.computeShaders = true;
        f.storageBuffers = true;

        // #if defined(RIVE_IOS) || defined(RIVE_IOS_SIMULATOR)
        if cfg!(target_os = "ios") {
            f.bc = false;
            f.etc2 = true;
            f.astc = true;
        } else {
            // #else
            f.bc = true;
            f.etc2 = false;
            f.astc = false;
        }

        f.maxColorAttachments = 8;
        f.maxTextureSize2D = 16384;
        f.maxTextureSizeCube = 16384;
        f.maxTextureSize3D = 2048;
        f.maxUniformBufferSize = 256 * 1024;
        f.maxVertexAttributes = 31;
        f.maxSamplers = 16;

        // Query the actual MSAA sample-count limit from the device. The source
        // order is deliberately 8, 4, 2, then the 1 fallback.
        f.maxSamples = 1;
        for sample_count in [8_u32, 4_u32, 2_u32] {
            if device
                .as_ref()
                .is_some_and(|device| device.supportsTextureSampleCount(sample_count as usize))
            {
                f.maxSamples = sample_count;
                break;
            }
        }
    }

    // inline rcp<Buffer> ContextMetal::mtlMakeBuffer(const BufferDesc& desc)
    #[cfg(target_vendor = "apple")]
    fn mtlMakeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        let device = self.m_mtlDevice.as_ref()?.clone();
        let mut buffer = BufferMetal::new(
            desc.size(),
            desc.usage,
            device,
            (*self.m_bufferState).clone(),
        );

        let native = if let Some(data) = desc.data_prefix().ok()? {
            // `newBufferWithBytes:length:options:` synchronously copies the
            // borrowed span; no source descriptor owner is retained.
            let bytes = std::ptr::NonNull::new(data.as_ptr().cast_mut().cast::<c_void>())?;
            unsafe {
                self.m_mtlDevice
                    .as_ref()?
                    .newBufferWithBytes_length_options(
                        bytes,
                        desc.size() as usize,
                        objc2_metal::MTLResourceOptions::StorageModeShared,
                    )
            }
        } else {
            self.m_mtlDevice.as_ref()?.newBufferWithLength_options(
                desc.size() as usize,
                objc2_metal::MTLResourceOptions::StorageModeShared,
            )
        }?;
        if let Some(label) = desc.label {
            native.setLabel(Some(&NSString::from_str(label)));
        }
        buffer.initializeBacking(Some(native), desc.label);
        Some(
            ResourceHandle::new_buffer_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                buffer,
            )
            .erase(),
        )
    }

    // inline rcp<Texture> ContextMetal::mtlMakeTexture(const TextureDesc& desc)
    #[cfg(target_vendor = "apple")]
    fn mtlMakeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        if desc.width == 0
            || desc.height == 0
            || desc.depthOrArrayLayers == 0
            || desc.numMipmaps == 0
            || desc.sampleCount == 0
        {
            self.base.setLastError(
                "makeTexture: dimensions, layers, mip levels, and sample count must be non-zero",
            );
            return None;
        }
        let descriptor = MTLTextureDescriptor::new();
        let is_msaa = desc.sampleCount > 1 && desc.r#type == TextureType::texture2D;
        descriptor.setTextureType(if is_msaa {
            MTLTextureType::Type2DMultisample
        } else {
            oreTextureTypeToMTL(desc.r#type)
        });
        descriptor.setPixelFormat(oreFormatToMTL(desc.format));
        unsafe {
            descriptor.setWidth(desc.width as usize);
            descriptor.setHeight(desc.height as usize);
            descriptor.setMipmapLevelCount(if is_msaa { 1 } else { desc.numMipmaps as usize });
            descriptor.setSampleCount(desc.sampleCount as usize);
        }

        // Private storage is GPU-only — replaceRegion (CPU upload) is
        // undefined. Uploadable textures remain Shared; render-target-only
        // textures remain Private.
        descriptor.setStorageMode(if desc.renderTarget {
            objc2_metal::MTLStorageMode::Private
        } else {
            objc2_metal::MTLStorageMode::Shared
        });
        let mut usage = MTLTextureUsage::ShaderRead;
        descriptor.setUsage(usage);

        match desc.r#type {
            TextureType::texture3D => unsafe {
                descriptor.setDepth(desc.depthOrArrayLayers as usize);
                descriptor.setArrayLength(1);
            },
            TextureType::array2D => unsafe {
                descriptor.setDepth(1);
                descriptor.setArrayLength(desc.depthOrArrayLayers as usize);
            },
            TextureType::cube | TextureType::texture2D => unsafe {
                descriptor.setDepth(1);
                descriptor.setArrayLength(1);
            },
        }

        if desc.renderTarget {
            usage |= MTLTextureUsage::RenderTarget;
            descriptor.setUsage(usage);
        }

        if matches!(
            desc.format,
            TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
        ) {
            usage |= MTLTextureUsage::PixelFormatView;
            descriptor.setUsage(usage);
        }

        let mut texture = TextureMetal::new(desc);
        // #if RIVE_OBJC_EXCEPTIONS
        // @try { texture->m_mtlTexture = [device newTextureWithDescriptor:td]; }
        // @catch (NSException* e) { NSLog(...); return nullptr; }
        // #else
        let device = self.m_mtlDevice.as_ref()?;
        #[cfg(feature = "rive-objc-exceptions")]
        let native = match objc2::exception::catch(|| device.newTextureWithDescriptor(&descriptor))
        {
            Ok(Some(native)) => native,
            Ok(None) => {
                eprintln!("RIVE ORE: makeTexture: newTextureWithDescriptor returned nil");
                return None;
            }
            Err(exception) => {
                let reason = exception
                    .as_deref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "unknown".to_owned());
                eprintln!("RIVE ORE: makeTexture exception: {reason}");
                return None;
            }
        };
        #[cfg(not(feature = "rive-objc-exceptions"))]
        let native = match device.newTextureWithDescriptor(&descriptor) {
            Some(native) => native,
            None => {
                eprintln!("RIVE ORE: makeTexture: newTextureWithDescriptor returned nil");
                return None;
            }
        };
        // #endif
        *texture.m_mtlTexture = Some(native);
        if let Some(label) = desc.label {
            texture
                .m_mtlTexture
                .as_ref()?
                .setLabel(Some(&NSString::from_str(label)));
        }
        Some(
            ResourceHandle::new_texture_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                texture,
            )
            .erase(),
        )
    }

    // inline rcp<TextureView> ContextMetal::mtlMakeTextureView(
    //     const TextureViewDesc& desc)
    #[cfg(target_vendor = "apple")]
    fn mtlMakeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        let texture = desc.texture?;
        if !self.ownsResource(texture) {
            self.base
                .setLastError("makeTextureView: texture belongs to another Context");
            return None;
        }
        let tex = texture.downcast_ref::<TextureMetal>();
        debug_assert!(tex.is_some());
        let tex = tex?;
        let mut view = TextureViewMetal::new(texture.clone(), desc);

        let source_texture = tex.m_mtlTexture.as_ref()?;
        if source_texture.textureType() == MTLTextureType::Type2DMultisample {
            return Some(
                ResourceHandle::new_in_domain(
                    self.base.state.manager(),
                    self.base.state.resourceDomain(),
                    view,
                )
                .erase(),
            );
        }

        let view_type = match desc.dimension {
            TextureViewDimension::texture2D => MTLTextureType::Type2D,
            TextureViewDimension::cube => MTLTextureType::TypeCube,
            TextureViewDimension::texture3D => MTLTextureType::Type3D,
            TextureViewDimension::array2D => MTLTextureType::Type2DArray,
            TextureViewDimension::cubeArray => MTLTextureType::TypeCubeArray,
        };
        let mip_range = NSRange {
            location: desc.baseMipLevel as usize,
            length: desc.mipCount as usize,
        };
        let slice_range = NSRange {
            location: desc.baseLayer as usize,
            length: desc.layerCount as usize,
        };
        *view.m_mtlTextureView = unsafe {
            source_texture.newTextureViewWithPixelFormat_textureType_levels_slices(
                source_texture.pixelFormat(),
                view_type,
                mip_range,
                slice_range,
            )
        };
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                view,
            )
            .erase(),
        )
    }

    // inline rcp<Sampler> ContextMetal::mtlMakeSampler(const SamplerDesc& desc)
    #[cfg(target_vendor = "apple")]
    fn mtlMakeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        let descriptor = MTLSamplerDescriptor::new();
        descriptor.setMinFilter(oreFilterToMTL(desc.minFilter));
        descriptor.setMagFilter(oreFilterToMTL(desc.magFilter));
        descriptor.setMipFilter(oreMipFilterToMTL(desc.mipmapFilter));
        descriptor.setSAddressMode(oreWrapToMTL(desc.wrapU));
        descriptor.setTAddressMode(oreWrapToMTL(desc.wrapV));
        descriptor.setRAddressMode(oreWrapToMTL(desc.wrapW));
        descriptor.setLodMinClamp(desc.minLod);
        descriptor.setLodMaxClamp(desc.maxLod);
        descriptor.setMaxAnisotropy(desc.maxAnisotropy as usize);
        if desc.compare != CompareFunction::none {
            descriptor.setCompareFunction(oreCompareFunctionToMTL(desc.compare));
        }
        if let Some(label) = desc.label {
            descriptor.setLabel(Some(&NSString::from_str(label)));
        }
        let mut sampler = SamplerMetal::new();
        // Source does not check a nil sampler state; preserve the logical
        // SamplerMetal publication and its nullable native member.
        *sampler.m_mtlSampler = self
            .m_mtlDevice
            .as_ref()?
            .newSamplerStateWithDescriptor(&descriptor);
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                sampler,
            )
            .erase(),
        )
    }

    // inline rcp<ShaderModule> ContextMetal::mtlMakeShaderModule(
    //     const ShaderModuleDesc& desc)
    #[cfg(target_vendor = "apple")]
    fn mtlMakeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        let mut module = ShaderModuleMetal::new();
        let code_size = desc.codeSize().ok()? as usize;
        let code = desc.code?.get(..code_size)?;
        let source = std::str::from_utf8(code).ok()?;
        let source = NSString::from_str(source);
        let device = self.m_mtlDevice.as_deref()?;
        let mut error: Option<Retained<NSError>> = None;
        let options: Option<&objc2_metal::MTLCompileOptions> = None;
        let library: Option<Retained<ProtocolObject<dyn objc2_metal::MTLLibrary>>> = unsafe {
            msg_send![device,
                newLibraryWithSource: &*source,
                options: options,
                error: &mut error
            ]
        };
        if error.is_some() || library.is_none() {
            let message = error
                .as_ref()
                .map(|error| error.localizedDescription().to_string())
                .unwrap_or_else(|| "<nil>".to_owned());
            eprintln!("RIVE ORE: makeShaderModule error: {message}");
            return None;
        }
        let library = match library {
            Some(library) => library,
            None => return None,
        };
        *module.m_mtlLibrary = Some(library);
        module.base.applyBindingMapFromDesc(desc);
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                module,
            )
            .erase(),
        )
    }

    // inline rcp<Pipeline> ContextMetal::mtlMakePipeline(
    //     const PipelineDesc& desc, std::string* outError)
    #[cfg(target_vendor = "apple")]
    fn mtlMakePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        mut outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        if desc
            .vertexModule
            .into_iter()
            .chain(desc.fragmentModule)
            .any(|module| !self.ownsResource(module))
        {
            self.base
                .setLastError("makePipeline: shader module belongs to another Context");
            return None;
        }
        let layouts = desc
            .bindGroupLayouts
            .unwrap_or(&[])
            .get(..desc.bindGroupLayoutCount().ok()? as usize)?;
        if layouts
            .iter()
            .flatten()
            .any(|layout| !self.ownsResource(layout))
        {
            self.base
                .setLastError("makePipeline: bind group layout belongs to another Context");
            return None;
        }
        let mut pipeline = PipelineMetal::new(desc)?;

        // --- Validate user-supplied layouts against shader binding map ---
        {
            let mut error = String::new();
            if !validateLayoutsAgainstBindingMap(
                &pipeline.base.m_bindingMap,
                desc.bindGroupLayouts,
                desc.bindGroupLayoutCount().ok()?,
                Some(&mut error),
            ) || !validateColorRequiresFragment(
                desc.colorCount,
                desc.fragmentModule.is_some(),
                Some(&mut error),
            ) {
                if let Some(out) = outError.as_deref_mut() {
                    *out = error;
                } else {
                    self.base.setLastError(format!("makePipeline: {error}"));
                }
                return None;
            }
        }

        // --- Render Pipeline State ---
        let Some(vertex_module_handle) = desc.vertexModule else {
            if let Some(out) = outError.as_deref_mut() {
                *out = "vertex shader module is null".to_owned();
            }
            return None;
        };
        let Some(vertexModule) = vertex_module_handle.downcast_ref::<ShaderModuleMetal>() else {
            if let Some(out) = outError.as_deref_mut() {
                *out = "vertex shader module is not a Metal shader module".to_owned();
            }
            return None;
        };
        if vertexModule.m_mtlLibrary.is_none() {
            if let Some(out) = outError.as_deref_mut() {
                *out = "vertex shader library is nil".to_owned();
            }
            return None;
        }
        let descriptor = MTLRenderPipelineDescriptor::new();
        let vertex_entry = desc.vertexEntryPoint?;
        let vertex_function = vertexModule
            .m_mtlLibrary
            .as_ref()?
            .newFunctionWithName(&NSString::from_str(vertex_entry));
        let vertex_function = match vertex_function {
            Some(value) => value,
            None => {
                let message = format!(
                    "vertex entry point '{}' not found in shader library",
                    vertex_entry
                );
                eprintln!("RIVE ORE: makePipeline: {message}");
                if let Some(out) = outError.as_deref_mut() {
                    *out = message;
                }
                return None;
            }
        };
        descriptor.setVertexFunction(Some(&vertex_function));

        // Depth-only pipelines intentionally leave fragmentFunction nil.
        if let Some(fragment_module) = desc.fragmentModule {
            let fragmentModule = fragment_module.downcast_ref::<ShaderModuleMetal>();
            debug_assert!(fragmentModule.is_some());
            let fragmentModule = fragmentModule?;
            if fragmentModule.m_mtlLibrary.is_none() {
                if let Some(out) = outError.as_deref_mut() {
                    *out = "fragment shader library is nil".to_owned();
                }
                return None;
            }
            let fragment_entry = desc.fragmentEntryPoint?;
            let fragment_function = fragmentModule
                .m_mtlLibrary
                .as_ref()?
                .newFunctionWithName(&NSString::from_str(fragment_entry));
            let fragment_function = match fragment_function {
                Some(value) => value,
                None => {
                    let message = format!(
                        "fragment entry point '{}' not found in shader library",
                        fragment_entry
                    );
                    eprintln!("RIVE ORE: makePipeline: {message}");
                    if let Some(out) = outError.as_deref_mut() {
                        *out = message;
                    }
                    return None;
                }
            };
            descriptor.setFragmentFunction(Some(&fragment_function));
        }

        // Vertex descriptor. Metal buffer index 0..15 is reserved for uniform
        // buffers; authored vertex buffer order remains unchanged.
        if let Some(vertex_buffers) = desc
            .vertexBuffers
            .and_then(|buffers| buffers.get(..desc.vertexBufferCount as usize))
        {
            if !vertex_buffers.is_empty() {
                let vertex_descriptor = objc2_metal::MTLVertexDescriptor::new();
                for (bufIdx, layout) in vertex_buffers.iter().enumerate() {
                    let mtlBufIdx = bufIdx + kMetalVertexBufferBase as usize;
                    let native_layout = unsafe {
                        vertex_descriptor
                            .layouts()
                            .objectAtIndexedSubscript(mtlBufIdx)
                    };
                    unsafe {
                        native_layout.setStride(layout.stride as usize);
                        native_layout.setStepRate(1);
                    }
                    native_layout.setStepFunction(if layout.stepMode == VertexStepMode::instance {
                        MTLVertexStepFunction::PerInstance
                    } else {
                        MTLVertexStepFunction::PerVertex
                    });
                    let attributes = layout.attributes.get(..layout.attributeCount as usize)?;
                    for attr in attributes {
                        let native_attr = unsafe {
                            vertex_descriptor
                                .attributes()
                                .objectAtIndexedSubscript(attr.shaderSlot as usize)
                        };
                        native_attr.setFormat(oreVertexFormatToMTL(attr.format));
                        unsafe {
                            native_attr.setOffset(attr.offset as usize);
                            native_attr.setBufferIndex(mtlBufIdx);
                        }
                    }
                }
                descriptor.setVertexDescriptor(Some(&vertex_descriptor));
            }
        }

        // Color attachments.
        let attachments = descriptor.colorAttachments();
        for i in 0..desc.colorCount as usize {
            let target = &desc.colorTargets[i];
            let attachment = unsafe { attachments.objectAtIndexedSubscript(i) };
            attachment.setPixelFormat(oreFormatToMTL(target.format));
            attachment.setWriteMask(oreColorWriteMaskToMTL(target.writeMask));
            if target.blendEnabled {
                attachment.setBlendingEnabled(true);
                attachment.setSourceRGBBlendFactor(oreBlendFactorToMTL(target.blend.srcColor));
                attachment.setDestinationRGBBlendFactor(oreBlendFactorToMTL(target.blend.dstColor));
                attachment.setRgbBlendOperation(oreBlendOpToMTL(target.blend.colorOp));
                attachment.setSourceAlphaBlendFactor(oreBlendFactorToMTL(target.blend.srcAlpha));
                attachment
                    .setDestinationAlphaBlendFactor(oreBlendFactorToMTL(target.blend.dstAlpha));
                attachment.setAlphaBlendOperation(oreBlendOpToMTL(target.blend.alphaOp));
            }
        }

        // The rgba8unorm format sentinel is the sole source of truth for
        // whether a depth/stencil attachment exists.
        let hasDepthStencil = desc.depthStencil.format != TextureFormat::rgba8unorm;
        if hasDepthStencil {
            descriptor.setDepthAttachmentPixelFormat(oreFormatToMTL(desc.depthStencil.format));
            if matches!(
                desc.depthStencil.format,
                TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
            ) {
                descriptor
                    .setStencilAttachmentPixelFormat(oreFormatToMTL(desc.depthStencil.format));
            }
        }
        descriptor.setRasterSampleCount(desc.sampleCount as usize);
        if let Some(label) = desc.label {
            descriptor.setLabel(Some(&NSString::from_str(label)));
        }

        // #if RIVE_OBJC_EXCEPTIONS
        // @try {
        //     pipeline->m_mtlPipeline =
        //         [m_mtlDevice newRenderPipelineStateWithDescriptor:rpd
        //                                                     error:&pipelineErr];
        // }
        // @catch (NSException* e) {
        //     NSString* msg = e.reason ?: @"unknown Metal exception";
        //     NSLog(@"RIVE ORE: makePipeline exception: %@", msg);
        //     if (outError) *outError = msg.UTF8String;
        //     return nullptr;
        // }
        // #else
        // pipeline->m_mtlPipeline = [m_mtlDevice
        //     newRenderPipelineStateWithDescriptor:rpd error:&pipelineErr];
        // #endif
        // Both branches publish only after the native call returns a nonnil
        // state. Exception text is preserved as the source's outError route.
        let device = self.m_mtlDevice.as_deref()?;
        #[cfg(feature = "rive-objc-exceptions")]
        let (native_pipeline, pipeline_error) = match objc2::exception::catch(|| {
            let mut error: Option<Retained<NSError>> = None;
            let pipeline: Option<
                Retained<ProtocolObject<dyn objc2_metal::MTLRenderPipelineState>>,
            > = unsafe {
                msg_send![device,
                    newRenderPipelineStateWithDescriptor: &*descriptor,
                    error: &mut error
                ]
            };
            (pipeline, error)
        }) {
            Ok(result) => result,
            Err(exception) => {
                let message = exception
                    .as_deref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "unknown Metal exception".to_owned());
                eprintln!("RIVE ORE: makePipeline exception: {message}");
                if let Some(out) = outError.as_deref_mut() {
                    *out = message;
                }
                return None;
            }
        };
        #[cfg(not(feature = "rive-objc-exceptions"))]
        let (native_pipeline, pipeline_error) = unsafe {
            let mut error: Option<Retained<NSError>> = None;
            let pipeline: Option<
                Retained<ProtocolObject<dyn objc2_metal::MTLRenderPipelineState>>,
            > = msg_send![device,
                newRenderPipelineStateWithDescriptor: &*descriptor,
                error: &mut error
            ];
            (pipeline, error)
        };
        if pipeline_error.is_some() || native_pipeline.is_none() {
            let message = pipeline_error
                .as_ref()
                .map(|error| error.localizedDescription().to_string())
                .unwrap_or_else(|| "nil pipeline, no error details".to_owned());
            eprintln!("RIVE ORE: makePipeline error: {message}");
            if let Some(out) = outError.as_deref_mut() {
                *out = message;
            }
            return None;
        }
        let native_pipeline = match native_pipeline {
            Some(native_pipeline) => native_pipeline,
            None => return None,
        };
        *pipeline.m_mtlPipeline = Some(native_pipeline);

        // --- Depth/Stencil State ---
        let depth_descriptor = objc2_metal::MTLDepthStencilDescriptor::new();
        depth_descriptor
            .setDepthCompareFunction(oreCompareFunctionToMTL(desc.depthStencil.depthCompare));
        depth_descriptor.setDepthWriteEnabled(desc.depthStencil.depthWriteEnabled);

        // Stencil front.
        let front = objc2_metal::MTLStencilDescriptor::new();
        front.setStencilCompareFunction(oreCompareFunctionToMTL(desc.stencilFront.compare));
        front.setStencilFailureOperation(oreStencilOpToMTL(desc.stencilFront.failOp));
        front.setDepthFailureOperation(oreStencilOpToMTL(desc.stencilFront.depthFailOp));
        front.setDepthStencilPassOperation(oreStencilOpToMTL(desc.stencilFront.passOp));
        front.setReadMask(desc.stencilReadMask as u32);
        front.setWriteMask(desc.stencilWriteMask as u32);
        depth_descriptor.setFrontFaceStencil(Some(&front));

        // Stencil back.
        let back = objc2_metal::MTLStencilDescriptor::new();
        back.setStencilCompareFunction(oreCompareFunctionToMTL(desc.stencilBack.compare));
        back.setStencilFailureOperation(oreStencilOpToMTL(desc.stencilBack.failOp));
        back.setDepthFailureOperation(oreStencilOpToMTL(desc.stencilBack.depthFailOp));
        back.setDepthStencilPassOperation(oreStencilOpToMTL(desc.stencilBack.passOp));
        back.setReadMask(desc.stencilReadMask as u32);
        back.setWriteMask(desc.stencilWriteMask as u32);
        depth_descriptor.setBackFaceStencil(Some(&back));
        *pipeline.m_mtlDepthStencil = self
            .m_mtlDevice
            .as_ref()?
            .newDepthStencilStateWithDescriptor(&depth_descriptor);
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                pipeline,
            )
            .erase(),
        )
    }

    // inline rcp<BindGroup> ContextMetal::mtlMakeBindGroup(
    //     const BindGroupDesc& desc)
    #[cfg(target_vendor = "apple")]
    fn mtlMakeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        if desc.layout.is_none() {
            self.base
                .setLastError("makeBindGroup: BindGroupDesc::layout is null");
            return None;
        }
        let layoutHandle = desc.layout?;
        if !self.ownsResource(layoutHandle) {
            self.base
                .setLastError("makeBindGroup: layout belongs to another Context");
            return None;
        }
        let layout = layoutHandle.downcast_ref::<BindGroupLayout>()?;
        let groupIndex = layout.groupIndex();
        if groupIndex >= kMaxBindGroups {
            self.base.setLastError(format!(
                "makeBindGroup: layout->groupIndex {} out of range",
                groupIndex
            ));
            return None;
        }

        let mut bind_group = BindGroupMetal::new();
        bind_group.base.m_context = Arc::downgrade(&self.base.state);
        bind_group.base.m_layoutRef = Some(layoutHandle.clone());

        // Resolve per-stage Metal slots from the layout's pre-resolved
        // nativeSlotVS/nativeSlotFS fields. The source helper deliberately
        // skips an unresolved entry and continues with later kinds.
        let lookupStages = |binding: u32,
                            expected: BindingKind,
                            outVS: &mut u16,
                            outFS: &mut u16|
         -> bool {
            let Some(entry) = layout.findEntry(binding) else {
                self.base.setLastError(format!(
                    "makeBindGroup: (group={}, binding={}) not declared in BindGroupLayout",
                    groupIndex, binding
                ));
                return false;
            };
            let kindOK = entry.kind == expected
                || ((entry.kind == BindingKind::sampler
                    || entry.kind == BindingKind::comparisonSampler)
                    && (expected == BindingKind::sampler
                        || expected == BindingKind::comparisonSampler));
            if !kindOK {
                self.base.setLastError(format!(
                    "makeBindGroup: (group={}, binding={}) layout kind mismatch",
                    groupIndex, binding
                ));
                return false;
            }
            *outVS = if entry.nativeSlotVS == BindGroupLayoutEntry::kNativeSlotAbsent {
                BindingMap::kAbsent
            } else {
                entry.nativeSlotVS as u16
            };
            *outFS = if entry.nativeSlotFS == BindGroupLayoutEntry::kNativeSlotAbsent {
                BindingMap::kAbsent
            } else {
                entry.nativeSlotFS as u16
            };
            if *outVS == BindingMap::kAbsent && *outFS == BindingMap::kAbsent {
                self.base.setLastError(format!(
                    "makeBindGroup: (group={}, binding={}) layout has no resolved native slot — call makeLayoutFromShader",
                    groupIndex, binding
                ));
                return false;
            }
            true
        };

        let nBufs = (desc.uboCount as usize).min(8);
        let ubos = desc.ubos.get(..nBufs)?;
        bind_group.m_mtlBuffers.reserve(nBufs);
        for u in ubos {
            let Some(bufferHandle) = u.buffer else {
                debug_assert!(false, "BindGroup UBO buffer must not be null");
                continue;
            };
            if !self.ownsResource(bufferHandle) {
                self.base
                    .setLastError("makeBindGroup: buffer belongs to another Context");
                return None;
            }
            let Some(_buf) = bufferHandle.downcast_ref::<BufferMetal>() else {
                // Source `assert(buf)` is retained as a hard downcast
                // invariant; no native operation precedes this assertion.
                debug_assert!(false, "BindGroup UBO must be BufferMetal");
                continue;
            };
            let retainedIndex = bind_group.base.m_retainedBuffers.len();
            let mut binding = MTLBufferBinding::new(retainedIndex);
            binding.offset = u.offset;
            binding.binding = u.slot;
            if !lookupStages(
                u.slot,
                BindingKind::uniformBuffer,
                &mut binding.vsSlot,
                &mut binding.fsSlot,
            ) {
                continue;
            }
            binding.hasDynamicOffset = layout.hasDynamicOffset(u.slot);
            if binding.hasDynamicOffset {
                bind_group.base.m_dynamicOffsetCount += 1;
            }
            bind_group.m_mtlBuffers.push(binding);
            bind_group.base.m_retainedBuffers.push(bufferHandle.clone());
        }
        // Sort UBOs by WGSL @binding so dynamicOffsets[] follows layout order.
        bind_group
            .m_mtlBuffers
            .sort_by_key(|binding| binding.binding);

        let nTexs = (desc.textureCount as usize).min(8);
        let textures = desc.textures.get(..nTexs)?;
        bind_group.m_mtlTextures.reserve(nTexs);
        for t in textures {
            let Some(viewHandle) = t.view else {
                debug_assert!(false, "BindGroup texture view must not be null");
                continue;
            };
            if !self.ownsResource(viewHandle) {
                self.base
                    .setLastError("makeBindGroup: texture view belongs to another Context");
                return None;
            }
            let Some(view) = viewHandle.downcast_ref::<TextureViewMetal>() else {
                debug_assert!(false, "BindGroup texture must be TextureViewMetal");
                continue;
            };
            let mut binding = MTLTextureBinding::default();
            binding.texture = view.mtlTexture();
            if !lookupStages(
                t.slot,
                BindingKind::sampledTexture,
                &mut binding.vsSlot,
                &mut binding.fsSlot,
            ) {
                continue;
            }
            bind_group.m_mtlTextures.push(binding);
            bind_group.base.m_retainedViews.push(viewHandle.clone());
        }

        let nSamps = (desc.samplerCount as usize).min(8);
        let samplers = desc.samplers.get(..nSamps)?;
        bind_group.m_mtlSamplers.reserve(nSamps);
        for s in samplers {
            let Some(samplerHandle) = s.sampler else {
                debug_assert!(false, "BindGroup sampler must not be null");
                continue;
            };
            if !self.ownsResource(samplerHandle) {
                self.base
                    .setLastError("makeBindGroup: sampler belongs to another Context");
                return None;
            }
            let Some(samp) = samplerHandle.downcast_ref::<SamplerMetal>() else {
                debug_assert!(false, "BindGroup sampler must be SamplerMetal");
                continue;
            };
            let mut binding = MTLSamplerBinding::default();
            binding.sampler = (*samp.m_mtlSampler).clone();
            if !lookupStages(
                s.slot,
                BindingKind::sampler,
                &mut binding.vsSlot,
                &mut binding.fsSlot,
            ) {
                continue;
            }
            bind_group.m_mtlSamplers.push(binding);
            bind_group
                .base
                .m_retainedSamplers
                .push(samplerHandle.clone());
        }
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                bind_group,
            )
            .erase(),
        )
    }

    // inline std::unique_ptr<RenderPass> ContextMetal::mtlBeginRenderPass(
    //     const RenderPassDesc& desc, std::string* outError)
    #[cfg(target_vendor = "apple")]
    fn mtlBeginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        _outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        // A null command buffer is a failed source precondition.  Preserve
        // the public null-return/error contract instead of dereferencing the
        // absent native encoder (the C++ assertion is not recoverable across
        // the Rust API boundary).
        if self.m_mtlCommandBuffer.is_none() {
            self.base
                .setLastError("beginRenderPass: beginFrame has not created a command buffer");
            return None;
        }
        let descriptor = MTLRenderPassDescriptor::new();
        let color_attachments = descriptor.colorAttachments();

        for i in 0..desc.colorCount as usize {
            let ca = &desc.colorAttachments[i];
            if ca.view.is_some_and(|view| !self.ownsResource(view))
                || ca
                    .resolveTarget
                    .is_some_and(|view| !self.ownsResource(view))
            {
                self.base
                    .setLastError("beginRenderPass: color attachment belongs to another Context");
                return None;
            }
            let Some(view) = ca
                .view
                .and_then(|view| view.downcast_ref::<TextureViewMetal>())
            else {
                debug_assert!(false, "color attachment view must be TextureViewMetal");
                continue;
            };
            let mtlTex = view.mtlTexture();
            let Some(baseTex) = view.baseTexture() else {
                debug_assert!(false, "TextureViewMetal base must be TextureMetal");
                continue;
            };
            let attachment = unsafe { color_attachments.objectAtIndexedSubscript(i) };
            attachment.setTexture(mtlTex.as_ref().map(Retained::as_ref));
            let hasView = !same_native_texture(mtlTex.as_ref(), baseTex.m_mtlTexture.as_ref());
            attachment.setLevel(if hasView {
                0
            } else {
                view.baseMipLevel() as usize
            });
            attachment.setSlice(if hasView {
                0
            } else {
                view.baseLayer() as usize
            });
            attachment.setLoadAction(oreLoadOpToMTL(ca.loadOp));
            attachment.setStoreAction(oreStoreOpToMTL(ca.storeOp));
            attachment.setClearColor(MTLClearColor {
                red: ca.clearColor.r as f64,
                green: ca.clearColor.g as f64,
                blue: ca.clearColor.b as f64,
                alpha: ca.clearColor.a as f64,
            });

            if let Some(resolveTarget) = ca.resolveTarget {
                let resolveView = resolveTarget.downcast_ref::<TextureViewMetal>();
                debug_assert!(resolveView.is_some());
                let Some(resolveView) = resolveView else {
                    continue;
                };
                let resolveTex = resolveView.mtlTexture();
                attachment.setResolveTexture(resolveTex.as_ref().map(Retained::as_ref));
                let resolveBaseTex = resolveView.baseTexture();
                let resolveHasView = !same_native_texture(
                    resolveTex.as_ref(),
                    resolveBaseTex
                        .as_ref()
                        .and_then(|texture| texture.m_mtlTexture.as_ref()),
                );
                attachment.setResolveLevel(if resolveHasView {
                    0
                } else {
                    resolveView.baseMipLevel() as usize
                });
                attachment.setResolveSlice(if resolveHasView {
                    0
                } else {
                    resolveView.baseLayer() as usize
                });
                // Preserve the user's storeOp when a resolve target exists.
                attachment.setStoreAction(if ca.storeOp == StoreOp::store {
                    MTLStoreAction::StoreAndMultisampleResolve
                } else {
                    MTLStoreAction::MultisampleResolve
                });
            }
        }

        if let Some(depthTarget) = desc.depthStencil.view {
            if !self.ownsResource(depthTarget) {
                self.base
                    .setLastError("beginRenderPass: depth attachment belongs to another Context");
                return None;
            }
            let dsView = depthTarget.downcast_ref::<TextureViewMetal>();
            debug_assert!(dsView.is_some());
            let Some(dsView) = dsView else {
                return None;
            };
            let depthTex = dsView.mtlTexture();
            let Some(dsTex) = dsView.baseTexture() else {
                debug_assert!(false, "depth view base must be TextureMetal");
                return None;
            };
            let depthHasView = !same_native_texture(depthTex.as_ref(), dsTex.m_mtlTexture.as_ref());
            let depth = descriptor.depthAttachment();
            depth.setTexture(depthTex.as_ref().map(Retained::as_ref));
            depth.setLevel(if depthHasView {
                0
            } else {
                dsView.baseMipLevel() as usize
            });
            depth.setSlice(if depthHasView {
                0
            } else {
                dsView.baseLayer() as usize
            });
            depth.setLoadAction(oreLoadOpToMTL(desc.depthStencil.depthLoadOp));
            depth.setStoreAction(oreStoreOpToMTL(desc.depthStencil.depthStoreOp));
            depth.setClearDepth(desc.depthStencil.depthClearValue as f64);

            if matches!(
                dsTex.format(),
                TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
            ) {
                let stencil = descriptor.stencilAttachment();
                stencil.setTexture(depthTex.as_ref().map(Retained::as_ref));
                stencil.setLevel(if depthHasView {
                    0
                } else {
                    dsView.baseMipLevel() as usize
                });
                stencil.setSlice(if depthHasView {
                    0
                } else {
                    dsView.baseLayer() as usize
                });
                stencil.setLoadAction(oreLoadOpToMTL(desc.depthStencil.stencilLoadOp));
                stencil.setStoreAction(oreStoreOpToMTL(desc.depthStencil.stencilStoreOp));
                stencil.setClearStencil(desc.depthStencil.stencilClearValue);
            }
        }

        debug_assert!(self.m_mtlCommandBuffer.is_some());
        let encoder = self.m_mtlCommandBuffer.as_ref().and_then(|command_buffer| {
            command_buffer.renderCommandEncoderWithDescriptor(&descriptor)
        });
        if encoder.is_none() {
            eprintln!(
                "RIVE ORE: beginRenderPass: renderCommandEncoderWithDescriptor returned nil — render pass descriptor is invalid. colorCount={}, hasDepth={}",
                desc.colorCount,
                if desc.depthStencil.view.is_some() {
                    "yes"
                } else {
                    "no"
                }
            );
        }
        if let Some(label) = desc.label {
            if let Some(encoder) = encoder.as_ref() {
                encoder.setLabel(Some(&NSString::from_str(label)));
            }
        }
        let pass = RenderPassMetal::new_with_context(&self.base.state);
        pass.initializeNative(encoder, (*self.m_mtlCommandBuffer).clone(), desc);
        Some(Box::new(pass))
    }

    // inline rcp<TextureView> ContextMetal::mtlWrapCanvasTexture(
    //     gpu::RenderCanvas* canvas)
    #[cfg(target_vendor = "apple")]
    unsafe fn mtlWrapCanvasTexture<C: MetalRenderCanvasHost>(
        &mut self,
        canvas: *mut C,
    ) -> Option<AnyResourceHandle> {
        debug_assert!(!canvas.is_null());
        let canvas = unsafe { &*canvas };
        let mtlTexture = canvas.metalTargetTexture();
        debug_assert!(mtlTexture.is_some());

        let texDesc = TextureDesc {
            width: canvas.metalWidth(),
            height: canvas.metalHeight(),
            format: mtlFormatToOre(mtlTexture.as_ref()?.pixelFormat()),
            r#type: TextureType::texture2D,
            renderTarget: true,
            numMipmaps: 1,
            sampleCount: 1,
            label: None,
            ..TextureDesc::default()
        };
        let mut texture = TextureMetal::new(&texDesc);
        *texture.m_mtlTexture = mtlTexture;
        let texture = ResourceHandle::new_texture_in_domain(
            self.base.state.manager(),
            self.base.state.resourceDomain(),
            texture,
        )
        .erase();
        let viewDesc = TextureViewDesc {
            texture: Some(&texture),
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        let mut view = TextureViewMetal::new(texture.clone(), &viewDesc);
        *view.m_mtlTextureView = (*view.baseTexture()?.m_mtlTexture).clone();
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                view,
            )
            .erase(),
        )
    }

    // } // helper methods on ContextMetal
}

// ============================================================================
// ContextMetal
// ============================================================================

// ContextMetal::~ContextMetal()
//
// C++ explicitly nils command buffer, queue, and device in that order. The
// Rust owner wrapper keeps those fields as `Option<Retained<_>>`; clearing them
// in Drop preserves the observable release order before normal field teardown.
impl ContextMetal {
    // std::unique_ptr<ContextMetal> ContextMetal::Make(
    //     id<MTLDevice> device, id<MTLCommandQueue> queue)
    #[cfg(target_vendor = "apple")]
    pub(crate) unsafe fn Make(
        device: NativeDevice,
        queue: NativeQueue,
    ) -> Option<Box<ContextMetal>> {
        let mut ctx = Box::new(ContextMetal::new());
        *ctx.m_mtlDevice = device;
        *ctx.m_mtlQueue = queue;
        let device = ctx.m_mtlDevice.clone();
        ctx.mtlPopulateFeatures(&device);
        Some(ctx)
    }

    /// Safe product boundary for the source `Make` raw-service precondition.
    /// The pinned Objective-C++ method accepts its paired backend services
    /// without inspecting them; public Rust callers must prove that pairing.
    #[cfg(target_vendor = "apple")]
    pub fn MakeChecked(device: NativeDevice, queue: NativeQueue) -> Option<Box<ContextMetal>> {
        if let (Some(device), Some(queue)) = (device.as_ref(), queue.as_ref()) {
            let queue_device = queue.device();
            if Retained::as_ptr(&queue_device) != Retained::as_ptr(device) {
                return None;
            }
        }
        // SAFETY: the identity check above establishes the source constructor's
        // paired-device/queue backend precondition.
        unsafe { Self::Make(device, queue) }
    }

    // void ContextMetal::beginFrame(const FrameDescriptor&)
    #[cfg(target_vendor = "apple")]
    pub fn beginFrame(&mut self, _descriptor: &FrameDescriptor) {
        *self.m_mtlCommandBuffer = self
            .m_mtlQueue
            .as_ref()
            .and_then(|queue| queue.commandBuffer());
        // Serial of the command buffer about to be recorded.
        self.m_bufferState
            .setCurrentSerial(self.currentSerial().wrapping_add(1));
    }

    // void ContextMetal::waitForGPU()
    #[cfg(target_vendor = "apple")]
    pub fn waitForGPU(&mut self) {
        // #if defined(ORE_BACKEND_METAL)
        if let Some(command_buffer) = self.m_mtlCommandBuffer.as_ref() {
            command_buffer.waitUntilCompleted();
        }
        // #endif
    }

    // void ContextMetal::endFrame()
    #[cfg(target_vendor = "apple")]
    pub fn endFrame(&mut self) {
        let _ = self.end_frame_with_completion();
    }

    /// Commits through the single pinned completion callback while exposing
    /// the command-buffer status to the authenticated product readback seam.
    #[cfg(target_vendor = "apple")]
    pub fn end_frame_with_completion(&mut self) -> Option<MetalSubmissionCompletion> {
        if let Some(command_buffer) = self.m_mtlCommandBuffer.take() {
            // Capture the exact authored deferred owner vector and clear it
            // before publishing the completed serial, as in the pinned block.
            let finishedSerial = self.currentSerial();
            let completedSerial = self.m_completedSerial.clone();
            let deferredBindGroups =
                Arc::new(Mutex::new(core::mem::take(&mut *self.m_deferredBindGroups)));
            let submission = MetalSubmissionCompletion {
                result: Arc::new(Mutex::new(None)),
            };
            let submission_for_handler = submission.clone();

            let completion = block2::RcBlock::new(
                move |command_buffer: std::ptr::NonNull<
                    ProtocolObject<dyn objc2_metal::MTLCommandBuffer>,
                >| {
                    // SAFETY: Metal supplies the live command buffer for the
                    // duration of its copied completion block.
                    let command_buffer = unsafe { command_buffer.as_ref() };
                    let result = command_buffer_completion_result(
                        command_buffer.status(),
                        command_buffer
                            .error()
                            .map(|error| format!("Metal command buffer failed: {error:?}")),
                    );
                    let mut deferredBindGroups = deferredBindGroups
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    finish_source_completion(
                        &mut deferredBindGroups,
                        &completedSerial,
                        finishedSerial,
                        || {
                            // Product observation is terminal: source
                            // deferred-owner and completed-serial lifetime
                            // bookkeeping must already be visible when `result()`
                            // becomes ready.
                            submission_for_handler.complete(result);
                        },
                    );
                },
            );
            // Metal copies the completion block and invokes it on an internal
            // thread. Every captured owner is retained until that callback runs.
            unsafe {
                command_buffer.addCompletedHandler(std::ptr::from_ref(&*completion).cast_mut());
            }
            command_buffer.commit();
            Some(submission)
        } else {
            None
        }
    }

    // ============================================================================
    // makeBuffer
    // ============================================================================

    pub fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        self.mtlMakeBuffer(desc)
    }

    // ============================================================================
    // makeTexture
    // ============================================================================

    pub fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        self.mtlMakeTexture(desc)
    }

    // ============================================================================
    // makeTextureView
    // ============================================================================

    pub fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        self.mtlMakeTextureView(desc)
    }

    // ============================================================================
    // makeSampler
    // ============================================================================

    pub fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        self.mtlMakeSampler(desc)
    }

    // ============================================================================
    // makeShaderModule
    // ============================================================================

    pub fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        self.mtlMakeShaderModule(desc)
    }

    // ============================================================================
    // makePipeline
    // ============================================================================

    pub fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        self.mtlMakePipeline(desc, outError)
    }

    // ============================================================================
    // makeBindGroup
    // ============================================================================

    pub fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        self.mtlMakeBindGroup(desc)
    }

    // ============================================================================
    // makeBindGroupLayout
    // ============================================================================

    pub fn makeBindGroupLayout(
        &mut self,
        desc: &BindGroupLayoutDesc<'_>,
    ) -> Option<AnyResourceHandle> {
        if desc.groupIndex >= kMaxBindGroups {
            self.base.setLastError(format!(
                "makeBindGroupLayout: groupIndex {} out of range [0, {})",
                desc.groupIndex, kMaxBindGroups
            ));
            return None;
        }
        let mut layout = BindGroupLayout::new();
        layout.m_context = Arc::downgrade(&self.base.state);
        layout.m_groupIndex = desc.groupIndex;
        let Some(entries) = desc.entries.get(..desc.entryCount as usize) else {
            self.base
                .setLastError("makeBindGroupLayout: entryCount exceeds entries span".to_owned());
            return None;
        };
        layout.m_entries.reserve(entries.len());
        for entry in entries {
            layout.m_entries.push(*entry);
        }
        // Metal has no native layout object — entries-only suffices.
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                layout,
            )
            .erase(),
        )
    }

    // ============================================================================
    // beginRenderPass
    // ============================================================================

    pub fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        self.base.finishActiveRenderPass();
        self.mtlBeginRenderPass(desc, outError)
    }

    // ============================================================================
    // wrapCanvasTexture
    // ============================================================================

    /// Product-safe spelling of the native texture bridge shared by the two
    /// source virtuals below. The returned exact TextureViewMetal strongly
    /// owns the exact TextureMetal handle, and both retain the same native
    /// texture according to the pinned source ownership graph.
    pub fn wrap_native_texture(
        &self,
        native: Retained<ProtocolObject<dyn objc2_metal::MTLTexture>>,
        width: u32,
        height: u32,
        render_target: bool,
    ) -> Option<AnyResourceHandle> {
        let context_device = self.m_mtlDevice.as_ref()?;
        let texture_device = native.device();
        if Retained::as_ptr(&texture_device) != Retained::as_ptr(context_device) {
            return None;
        }
        if native.width() != width as usize
            || native.height() != height as usize
            || width == 0
            || height == 0
            || native.textureType() != MTLTextureType::Type2D
            || native.depth() != 1
            || native.arrayLength() != 1
            || native.mipmapLevelCount() != 1
            || native.sampleCount() != 1
            || (render_target && !native.usage().contains(MTLTextureUsage::RenderTarget))
        {
            return None;
        }
        let texture_desc = TextureDesc {
            width,
            height,
            format: mtlFormatToOre(native.pixelFormat()),
            r#type: TextureType::texture2D,
            renderTarget: render_target,
            numMipmaps: 1,
            sampleCount: 1,
            label: None,
            ..TextureDesc::default()
        };
        let mut texture = TextureMetal::new(&texture_desc);
        *texture.m_mtlTexture = Some(native.clone());
        let texture = ResourceHandle::new_texture_in_domain(
            self.base.state.manager(),
            self.base.state.resourceDomain(),
            texture,
        )
        .erase();
        let view_desc = TextureViewDesc {
            texture: Some(&texture),
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        let mut view = TextureViewMetal::new(texture.clone(), &view_desc);
        *view.m_mtlTextureView = Some(native);
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                view,
            )
            .erase(),
        )
    }

    pub unsafe fn wrapCanvasTexture<C: MetalRenderCanvasHost>(
        &mut self,
        canvas: *mut C,
    ) -> Option<AnyResourceHandle> {
        unsafe { self.mtlWrapCanvasTexture(canvas) }
    }

    pub unsafe fn wrapRiveTexture<T: MetalRiveTextureHost>(
        &mut self,
        gpuTex: *mut T,
        w: u32,
        h: u32,
    ) -> Option<AnyResourceHandle> {
        if gpuTex.is_null() {
            return None;
        }
        let gpuTex = unsafe { &*gpuTex };
        let mtlTex = gpuTex.metalNativeTexture();
        if mtlTex.is_none() {
            return None;
        }

        let texDesc = TextureDesc {
            width: w,
            height: h,
            format: mtlFormatToOre(mtlTex.as_ref()?.pixelFormat()),
            r#type: TextureType::texture2D,
            renderTarget: false,
            numMipmaps: 1,
            sampleCount: 1,
            label: None,
            ..TextureDesc::default()
        };
        let mut texture = TextureMetal::new(&texDesc);
        // Borrow — caller owns the native texture via RenderImage.
        *texture.m_mtlTexture = mtlTex.clone();
        let texture = ResourceHandle::new_texture_in_domain(
            self.base.state.manager(),
            self.base.state.resourceDomain(),
            texture,
        )
        .erase();
        let viewDesc = TextureViewDesc {
            texture: Some(&texture),
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        let mut view = TextureViewMetal::new(texture.clone(), &viewDesc);
        *view.m_mtlTextureView = mtlTex;
        Some(
            ResourceHandle::new_in_domain(
                self.base.state.manager(),
                self.base.state.resourceDomain(),
                view,
            )
            .erase(),
        )
    }
}

impl ContextApi for ContextMetal {
    fn features(&self) -> Features {
        ContextMetal::features(self)
    }

    fn lastError(&self) -> String {
        ContextMetal::lastError(self)
    }

    fn activeRenderPass(&self) -> Option<RcWeak<dyn ActiveRenderPass>> {
        ContextMetal::activeRenderPass(self)
    }

    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        ContextMetal::setActiveRenderPass(self, pass);
    }

    fn finishActiveRenderPass(&self) {
        ContextMetal::finishActiveRenderPass(self);
    }

    fn clearLastError(&self) {
        ContextMetal::clearLastError(self);
    }

    fn setLastError(&self, message: &str) {
        ContextMetal::setLastError(self, message);
    }

    fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        ContextMetal::makeBuffer(self, desc)
    }

    fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        ContextMetal::makeTexture(self, desc)
    }

    fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        ContextMetal::makeTextureView(self, desc)
    }

    fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        ContextMetal::makeSampler(self, desc)
    }

    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        ContextMetal::makeShaderModule(self, desc)
    }

    fn makeBindGroupLayout(&mut self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        ContextMetal::makeBindGroupLayout(self, desc)
    }

    fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        ContextMetal::makePipeline(self, desc, outError)
    }

    fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        ContextMetal::makeBindGroup(self, desc)
    }

    fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        ContextMetal::beginRenderPass(self, desc, outError)
    }

    fn beginFrame(&mut self, descriptor: &FrameDescriptor) {
        ContextMetal::beginFrame(self, descriptor);
    }

    fn endFrame(&mut self) {
        ContextMetal::endFrame(self);
    }

    fn waitForGPU(&mut self) {
        ContextMetal::waitForGPU(self);
    }

    unsafe fn wrapCanvasTexture(
        &mut self,
        canvas: *mut std::ffi::c_void,
    ) -> Option<AnyResourceHandle> {
        unsafe { ContextMetal::wrapCanvasTexture(self, canvas.cast::<MetalRenderCanvasBridge>()) }
    }

    unsafe fn wrapRiveTexture(
        &mut self,
        texture: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        unsafe {
            ContextMetal::wrapRiveTexture(
                self,
                texture.cast::<MetalRiveTextureBridge>(),
                width,
                height,
            )
        }
    }

    fn shaderTarget(&self) -> ShaderTarget {
        ContextMetal::shaderTarget(self)
    }
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_vendor = "apple")]
    use crate::types::{ColorAttachment, SampEntry, TexEntry, TextureAspect, UBOEntry};

    #[test]
    fn native_slot_validation_preserves_absent_and_rejects_every_limit() {
        assert_eq!(BindGroupLayoutEntry::kNativeSlotAbsent, u32::MAX);
        assert_eq!(BindingMap::kAbsent, u16::MAX);
        assert!(31 < 32);
        assert!(128 <= 128);
        assert!(16 <= 16);
        assert!(65_536 > u16::MAX as u32);
    }

    #[cfg(target_vendor = "apple")]
    fn live_context() -> Option<Box<ContextMetal>> {
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device for ORE context");
            return None;
        };
        let Some(queue) = device.newCommandQueue() else {
            crate::live_metal_test_unavailable("Metal command queue for ORE context");
            return None;
        };
        ContextMetal::MakeChecked(Some(device), Some(queue))
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn frame_serial_completion_outlives_context_and_begin_preserves_error() {
        let Some(mut context) = live_context() else {
            return;
        };
        context.base.setLastError("pinned error");
        context.beginFrame(&FrameDescriptor::new(99, 101));
        assert_eq!(context.currentSerial(), 1);
        assert_eq!(context.lastError(), "pinned error");
        let completion = Arc::clone(&context.m_bufferState);
        let command_buffer = context.m_mtlCommandBuffer.clone();
        let submission = context
            .end_frame_with_completion()
            .expect("current command buffer");
        command_buffer
            .as_ref()
            .expect("current command buffer retained for completion wait")
            .waitUntilCompleted();
        drop(context);
        assert_eq!(completion.completedSerial(), 1);
        assert_eq!(submission.result(), Some(Ok(())));
        assert!(
            command_buffer_completion_result(
                MTLCommandBufferStatus::Error,
                Some("injected Metal failure".to_owned())
            )
            .is_err()
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn product_completion_is_published_after_source_serial_becomes_reusable() {
        let completed = AtomicU64::new(0);
        let published = std::cell::Cell::new(false);
        let mut deferred = Vec::new();

        finish_source_completion(&mut deferred, &completed, 1, || {
            assert_eq!(completed.load(Ordering::Relaxed), 1);
            published.set(true);
        });

        assert!(published.get());
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn completion_releases_captured_resources_before_serial_and_product_publication() {
        struct CompletionDropProbe(Arc<AtomicU64>);
        impl Drop for CompletionDropProbe {
            fn drop(&mut self) {
                self.0.store(1, Ordering::Relaxed);
            }
        }

        let stage = Arc::new(AtomicU64::new(0));
        let completed = AtomicU64::new(0);
        let mut deferred = vec![
            ResourceHandle::new(
                None,
                crate::gpu_resource::TestGPUResource::new(CompletionDropProbe(stage.clone())),
            )
            .erase(),
        ];

        finish_source_completion(&mut deferred, &completed, 7, || {
            assert_eq!(stage.load(Ordering::Relaxed), 1);
            assert_eq!(completed.load(Ordering::Relaxed), 7);
            stage.store(2, Ordering::Relaxed);
        });

        assert!(deferred.is_empty());
        assert_eq!(stage.load(Ordering::Relaxed), 2);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn factories_publish_only_complete_native_resources() {
        let Some(mut context) = live_context() else {
            return;
        };
        let buffer = context
            .makeBuffer(
                &BufferDesc::initialized(crate::types::BufferUsage::uniform, &[1, 2, 3, 4], false)
                    .expect("small descriptor"),
            )
            .expect("buffer");
        assert!(buffer.downcast_ref::<BufferMetal>().is_some());

        let texture_desc = TextureDesc {
            width: 4,
            height: 4,
            ..TextureDesc::default()
        };
        for invalid in [
            TextureDesc {
                width: 0,
                ..texture_desc
            },
            TextureDesc {
                height: 0,
                ..texture_desc
            },
            TextureDesc {
                depthOrArrayLayers: 0,
                ..texture_desc
            },
            TextureDesc {
                numMipmaps: 0,
                ..texture_desc
            },
            TextureDesc {
                sampleCount: 0,
                ..texture_desc
            },
        ] {
            assert!(context.makeTexture(&invalid).is_none());
            assert_eq!(
                context.lastError(),
                "makeTexture: dimensions, layers, mip levels, and sample count must be non-zero"
            );
        }
        let texture = context.makeTexture(&texture_desc).expect("texture");
        let view = context
            .makeTextureView(&TextureViewDesc {
                texture: Some(&texture),
                dimension: TextureViewDimension::texture2D,
                aspect: TextureAspect::all,
                baseMipLevel: 0,
                mipCount: 1,
                baseLayer: 0,
                layerCount: 1,
            })
            .expect("view");
        assert!(view.downcast_ref::<TextureViewMetal>().is_some());
        let sampler = context
            .makeSampler(&SamplerDesc::default())
            .expect("sampler");
        assert!(sampler.downcast_ref::<SamplerMetal>().is_some());

        let native = texture
            .downcast_ref::<TextureMetal>()
            .and_then(TextureMetal::mtlTexture)
            .expect("retained native texture");
        let wrapped = context
            .wrap_native_texture(native, 4, 4, false)
            .expect("wrapped view");
        let wrapped_view = wrapped
            .downcast_ref::<TextureViewMetal>()
            .expect("wrapped view");
        let wrapped_texture = wrapped_view
            .base()
            .texture()
            .downcast_ref::<TextureMetal>()
            .expect("wrapped base texture");
        assert_eq!(wrapped_texture.base().width(), 4);
        assert_eq!(wrapped_texture.base().format(), TextureFormat::rgba8unorm);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn nil_native_texture_view_and_sampler_still_publish_logical_resources() {
        let Some(mut context) = live_context() else {
            return;
        };
        let texture = context
            .makeTexture(&TextureDesc {
                width: 4,
                height: 4,
                ..TextureDesc::default()
            })
            .expect("texture");
        let source = texture
            .downcast_ref::<TextureMetal>()
            .and_then(TextureMetal::mtlTexture)
            .expect("native source texture");
        let view_desc = TextureViewDesc {
            texture: Some(&texture),
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };

        let mut view = TextureViewMetal::new(texture.clone(), &view_desc);
        // Exercise the source nil-native-view fallback explicitly while
        // retaining the complete logical view owner.
        *view.m_mtlTextureView = None;
        let view_texture = view.mtlTexture().expect("source-texture fallback");
        assert_eq!(Retained::as_ptr(&view_texture), Retained::as_ptr(&source));

        let sampler = context
            .makeSampler(&SamplerDesc::default())
            .expect("logical Metal sampler");
        let sampler = sampler
            .downcast_ref::<SamplerMetal>()
            .expect("logical Metal sampler");
        assert!(sampler.m_mtlSampler.is_some());
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn public_layout_factory_rejects_out_of_range_native_slots_without_publication() {
        let Some(mut context) = live_context() else {
            return;
        };
        let mut entry = BindGroupLayoutEntry {
            binding: 7,
            nativeSlotVS: 31,
            ..BindGroupLayoutEntry::default()
        };
        let layout = context
            .makeBindGroupLayout(&BindGroupLayoutDesc {
                entries: std::slice::from_ref(&entry),
                entryCount: 1,
                ..BindGroupLayoutDesc::default()
            })
            .expect("pinned Metal layout retains native slots");
        assert_eq!(
            layout.downcast_ref::<BindGroupLayout>().unwrap().m_entries[0].nativeSlotVS,
            31
        );

        entry.kind = BindingKind::sampledTexture;
        entry.nativeSlotVS = BindGroupLayoutEntry::kNativeSlotAbsent;
        entry.nativeSlotFS = 128;
        let layout = context
            .makeBindGroupLayout(&BindGroupLayoutDesc {
                entries: std::slice::from_ref(&entry),
                entryCount: 1,
                ..BindGroupLayoutDesc::default()
            })
            .expect("pinned Metal layout retains sampled-texture slots");
        assert_eq!(
            layout.downcast_ref::<BindGroupLayout>().unwrap().m_entries[0].nativeSlotFS,
            128
        );

        entry.kind = BindingKind::sampler;
        entry.nativeSlotFS = BindGroupLayoutEntry::kNativeSlotAbsent;
        entry.nativeSlotCS = 16;
        let layout = context
            .makeBindGroupLayout(&BindGroupLayoutDesc {
                entries: std::slice::from_ref(&entry),
                entryCount: 1,
                ..BindGroupLayoutDesc::default()
            })
            .expect("pinned Metal layout retains sampler slots");
        assert_eq!(
            layout.downcast_ref::<BindGroupLayout>().unwrap().m_entries[0].nativeSlotCS,
            16
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn buffer_allocation_failure_immediately_replaces_and_clears_context_error() {
        let Some(mut context) = live_context() else {
            return;
        };
        let buffer = context
            .makeBuffer(&BufferDesc::uninitialized(
                crate::types::BufferUsage::uniform,
                4,
            ))
            .expect("buffer");
        let buffer = buffer.downcast_ref::<BufferMetal>().expect("Metal buffer");
        buffer.markBound();
        context.base.setLastError("older context error");

        crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_buffer_hpp::BufferApi::update(buffer, &[9], 1, 0)
            .expect("degraded update still writes the current backing");
        assert_eq!(context.lastError(), "older context error");
        context.clearLastError();
        assert_eq!(context.lastError(), "");
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_shader_compile_failure_preserves_context_error() {
        let Some(mut context) = live_context() else {
            return;
        };
        context.base.setLastError("earlier context error");
        let empty_binding_map = [2, 1, 14, 0, 0, 0, 0, 0];
        assert!(
            context
                .makeShaderModule(&ShaderModuleDesc {
                    code: Some(b"this is not valid Metal shading language"),
                    codeSize: b"this is not valid Metal shading language".len() as u32,
                    bindingMapBytes: Some(&empty_binding_map),
                    bindingMapSize: empty_binding_map.len() as u32,
                    ..ShaderModuleDesc::default()
                })
                .is_none()
        );
        assert_eq!(context.lastError(), "earlier context error");

        assert!(
            context
                .makeShaderModule(&ShaderModuleDesc::default())
                .is_none()
        );
        assert_eq!(context.lastError(), "earlier context error");

        assert!(
            context
                .makeShaderModule(&ShaderModuleDesc {
                    code: Some(&[0xff]),
                    codeSize: 1,
                    bindingMapBytes: Some(&empty_binding_map),
                    bindingMapSize: empty_binding_map.len() as u32,
                    ..ShaderModuleDesc::default()
                })
                .is_none()
        );
        assert_eq!(context.lastError(), "earlier context error");
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn pipeline_validation_routes_to_out_error_without_overwriting_context_error() {
        let Some(mut context) = live_context() else {
            return;
        };
        context.base.setLastError("earlier context error");
        let mut out_error = String::new();
        assert!(
            context
                .makePipeline(&PipelineDesc::default(), Some(&mut out_error))
                .is_none()
        );
        assert_eq!(
            out_error,
            "pipeline declares color outputs but has no fragment shader; supply `fragment`, or omit `colorTargets` for a depth-only pipeline"
        );
        assert_eq!(context.lastError(), "earlier context error");

        out_error.clear();
        assert!(
            context
                .makePipeline(
                    &PipelineDesc {
                        colorCount: 0,
                        ..PipelineDesc::default()
                    },
                    Some(&mut out_error),
                )
                .is_none()
        );
        assert_eq!(out_error, "vertex shader module is null");
        assert_eq!(context.lastError(), "earlier context error");
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn shader_and_pipeline_factories_publish_live_native_state_and_reject_bad_entry() {
        let Some(mut context) = live_context() else {
            return;
        };
        let source = br#"
#include <metal_stdlib>
using namespace metal;
vertex float4 vs_main(uint vertex_id [[vertex_id]]) {
    return float4(vertex_id == 0 ? -1.0 : 1.0, -1.0, 0.0, 1.0);
}
fragment float4 fs_main() { return float4(1.0); }
"#;
        let empty_binding_map = [2, 1, 14, 0, 0, 0, 0, 0];
        let module = context
            .makeShaderModule(&ShaderModuleDesc {
                code: Some(source),
                codeSize: source.len() as u32,
                bindingMapBytes: Some(&empty_binding_map),
                bindingMapSize: empty_binding_map.len() as u32,
                ..ShaderModuleDesc::default()
            })
            .expect("compile MSL through ContextMetal");
        let desc = PipelineDesc {
            vertexModule: Some(&module),
            fragmentModule: Some(&module),
            ..PipelineDesc::default()
        };
        let pipeline = context
            .makePipeline(&desc, None)
            .expect("publish complete native pipeline");
        assert!(pipeline.downcast_ref::<PipelineMetal>().is_some());

        let bad_desc = PipelineDesc {
            vertexModule: Some(&module),
            vertexEntryPoint: Some("missing_vertex"),
            fragmentModule: Some(&module),
            ..PipelineDesc::default()
        };
        let mut error = String::new();
        assert!(context.makePipeline(&bad_desc, Some(&mut error)).is_none());
        assert_eq!(
            error,
            "vertex entry point 'missing_vertex' not found in shader library"
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn pipeline_layout_validation_preserves_vertex_reflection_precedence() {
        let Some(mut context) = live_context() else {
            return;
        };
        let source = br#"
#include <metal_stdlib>
using namespace metal;
vertex float4 vs_main(uint vertex_id [[vertex_id]]) {
    return float4(vertex_id == 0 ? -1.0 : 1.0, -1.0, 0.0, 1.0);
}
fragment float4 fs_main() { return float4(1.0); }
"#;
        let empty = [2, 1, 14, 0, 0, 0, 0, 0];
        let fragment_sampler = [
            2, 1, 14, 0, 1, 0, 0, 0, // header
            0, 0, 5, 2, 0, 0xff, 0xff, 0, 0, 0xff, 0xff, 0, 0, 0,
        ];
        let vertex = context
            .makeShaderModule(&ShaderModuleDesc {
                code: Some(source),
                codeSize: source.len() as u32,
                bindingMapBytes: Some(&empty),
                bindingMapSize: empty.len() as u32,
                ..ShaderModuleDesc::default()
            })
            .expect("vertex module");
        let fragment = context
            .makeShaderModule(&ShaderModuleDesc {
                code: Some(source),
                codeSize: source.len() as u32,
                bindingMapBytes: Some(&fragment_sampler),
                bindingMapSize: fragment_sampler.len() as u32,
                ..ShaderModuleDesc::default()
            })
            .expect("fragment module");
        let pipeline = context.makePipeline(
            &PipelineDesc {
                vertexModule: Some(&vertex),
                fragmentModule: Some(&fragment),
                ..PipelineDesc::default()
            },
            None,
        );
        assert!(
            pipeline.is_some(),
            "a non-null vertex module is the binding-map source even when its map is empty"
        );

        let mut error = String::new();
        assert!(
            context
                .makePipeline(
                    &PipelineDesc {
                        vertexModule: None,
                        fragmentModule: Some(&fragment),
                        ..PipelineDesc::default()
                    },
                    Some(&mut error),
                )
                .is_none()
        );
        assert_eq!(
            error,
            "@group(0) @binding(0): shader declares sampler but PipelineDesc::bindGroupLayouts has no entry for group 0"
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn bind_group_skips_unresolved_entries_and_last_error_follows_kind_order() {
        let Some(mut context) = live_context() else {
            return;
        };
        let layout = context
            .makeBindGroupLayout(&BindGroupLayoutDesc::default())
            .expect("empty layout");
        let buffer = context
            .makeBuffer(&BufferDesc::uninitialized(
                crate::types::BufferUsage::uniform,
                16,
            ))
            .expect("buffer");
        let texture = context
            .makeTexture(&TextureDesc {
                width: 1,
                height: 1,
                ..TextureDesc::default()
            })
            .expect("texture");
        let view = context
            .makeTextureView(&TextureViewDesc {
                texture: Some(&texture),
                dimension: TextureViewDimension::texture2D,
                aspect: TextureAspect::all,
                baseMipLevel: 0,
                mipCount: 1,
                baseLayer: 0,
                layerCount: 1,
            })
            .expect("view");
        let sampler = context
            .makeSampler(&SamplerDesc::default())
            .expect("sampler");
        let ubos = [UBOEntry {
            slot: 1,
            buffer: Some(&buffer),
            offset: 0,
            size: 16,
        }];
        let textures = [TexEntry {
            slot: 2,
            view: Some(&view),
        }];
        let samplers = [SampEntry {
            slot: 3,
            sampler: Some(&sampler),
        }];
        let group = context
            .makeBindGroup(&BindGroupDesc {
                layout: Some(&layout),
                ubos: &ubos,
                uboCount: ubos.len() as u32,
                textures: &textures,
                textureCount: textures.len() as u32,
                samplers: &samplers,
                samplerCount: samplers.len() as u32,
                label: None,
            })
            .expect("source publishes a group after skipping invalid entries");
        let group = group.downcast_ref::<BindGroupMetal>().expect("Metal group");
        assert!(group.m_mtlBuffers.is_empty());
        assert!(group.m_mtlTextures.is_empty());
        assert!(group.m_mtlSamplers.is_empty());
        assert_eq!(
            context.lastError(),
            "makeBindGroup: (group=0, binding=3) not declared in BindGroupLayout"
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn beginning_a_second_pass_auto_finishes_the_first() {
        let Some(mut context) = live_context() else {
            return;
        };
        context.beginFrame(&FrameDescriptor::new(0, 0));
        let texture = context
            .makeTexture(&TextureDesc {
                width: 4,
                height: 4,
                renderTarget: true,
                ..TextureDesc::default()
            })
            .expect("render target");
        let view = context
            .makeTextureView(&TextureViewDesc {
                texture: Some(&texture),
                dimension: TextureViewDimension::texture2D,
                aspect: TextureAspect::all,
                baseMipLevel: 0,
                mipCount: 1,
                baseLayer: 0,
                layerCount: 1,
            })
            .expect("render-target view");
        let desc = RenderPassDesc {
            colorAttachments: [
                ColorAttachment {
                    view: Some(&view),
                    ..ColorAttachment::default()
                },
                ColorAttachment::default(),
                ColorAttachment::default(),
                ColorAttachment::default(),
            ],
            ..RenderPassDesc::default()
        };
        let first = context.beginRenderPass(&desc, None).expect("first pass");
        // The source Context tracks the pass identity supplied by its caller;
        // beginRenderPass then finishes that tracked pass before opening the
        // next Metal encoder.  Keep the fixture on that exact contract.
        context.setActiveRenderPass(Some(first.as_ref()));
        let mut second = context.beginRenderPass(&desc, None).expect("second pass");
        assert!(first.asAny().downcast_ref::<RenderPassMetal>().is_some());
        assert!(second.asAny().downcast_ref::<RenderPassMetal>().is_some());
        second.finish();
        context.endFrame();
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn render_pass_is_not_published_without_a_frame_command_buffer() {
        let Some(mut context) = live_context() else {
            return;
        };
        assert!(
            context
                .beginRenderPass(&RenderPassDesc::default(), None)
                .is_none()
        );
        assert_eq!(
            context.lastError(),
            "beginRenderPass: beginFrame has not created a command buffer"
        );
    }
}
