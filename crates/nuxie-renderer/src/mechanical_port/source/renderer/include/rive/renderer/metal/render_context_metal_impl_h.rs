/*
 * Copyright 2023 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/metal/render_context_metal_impl.h.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// The complete pinned source is retained below in declaration order. The Rust
// declarations following it preserve the two class owners, nested types,
// source defaults, reverse destruction order, native ownership, configuration branches, and
// virtual/private seams. Inline accessors and native ownership transfers below
// are executable and connect to the paired implementation owners.

// /*
//  * Copyright 2023 Rive
//  */
//
// #pragma once
//
// #include "rive/renderer/render_context_helper_impl.hpp"
// #include "rive/rive_types.hpp"
// #include "rive/shapes/paint/image_sampler.hpp"
// #include <unordered_map>
// #include <mutex>
//
// #ifndef RIVE_OBJC_NOP
// #import <Metal/Metal.h>
// #endif
//
// namespace rive::gpu
// {
// class BackgroundShaderCompiler;
//
// // Metal backend implementation of RenderTarget.
// class RenderTargetMetal : public RenderTarget
// {
// public:
//     ~RenderTargetMetal() override {}
//
//     MTLPixelFormat pixelFormat() const { return m_pixelFormat; }
//
//     bool compatibleWith(id<MTLTexture> texture) const
//     {
//         assert(texture.usage & MTLTextureUsageRenderTarget);
//         return width() == texture.width && height() == texture.height &&
//                m_pixelFormat == texture.pixelFormat;
//     }
//
//     void setTargetTexture(id<MTLTexture> texture);
//     id<MTLTexture> targetTexture() const { return m_targetTexture; }
//
// private:
//     friend class RenderContextMetalImpl;
//
//     RenderTargetMetal(id<MTLDevice>,
//                       MTLPixelFormat,
//                       uint32_t width,
//                       uint32_t height,
//                       const PlatformFeatures&);
//
//     // Lazily-allocated buffers for atomic mode. Unlike the memoryless textures,
//     // these buffers have actual physical storage that gets allocated the first
//     // time they're accessed.
//     id<MTLBuffer> colorAtomicBuffer()
//     {
//         return m_colorAtomicBuffer != nil
//                    ? m_colorAtomicBuffer
//                    : m_colorAtomicBuffer = makeAtomicBuffer();
//     }
//     id<MTLBuffer> coverageAtomicBuffer()
//     {
//         return m_coverageAtomicBuffer != nil
//                    ? m_coverageAtomicBuffer
//                    : m_coverageAtomicBuffer = makeAtomicBuffer();
//     }
//     id<MTLBuffer> clipAtomicBuffer()
//     {
//         return m_clipAtomicBuffer != nil
//                    ? m_clipAtomicBuffer
//                    : m_clipAtomicBuffer = makeAtomicBuffer();
//     }
//     id<MTLBuffer> makeAtomicBuffer()
//     {
//         return [m_gpu newBufferWithLength:height() * width() * sizeof(uint32_t)
//                                   options:MTLResourceStorageModePrivate];
//     }
//
//     const id<MTLDevice> m_gpu;
//     const MTLPixelFormat m_pixelFormat;
//
//     id<MTLTexture> m_targetTexture = nil;
//
//     id<MTLTexture> m_coverageMemorylessTexture = nil;
//     id<MTLTexture> m_clipMemorylessTexture = nil;
//     id<MTLTexture> m_scratchColorMemorylessTexture = nil;
//
//     id<MTLBuffer> m_colorAtomicBuffer = nil;
//     id<MTLBuffer> m_coverageAtomicBuffer = nil;
//     id<MTLBuffer> m_clipAtomicBuffer = nil;
// };
//
// // Metal backend implementation of RenderContextImpl.
// class RenderContextMetalImpl : public RenderContextHelperImpl
// {
// public:
//     struct ContextOptions
//     {
//         // Wait for shaders to compile inline with rendering (causing jank),
//         // instead of compiling asynchronously in a background thread.
//         // (Primarily for testing.)
//         ShaderCompilationMode shaderCompilationMode =
//             ShaderCompilationMode::standard;
//
//         // (macOS only -- ignored on iOS). Override
//         // m_platformFeatures.supportsRasterOrdering to false, forcing us to
//         // always render in atomic mode.
//         bool disableFramebufferReads = false;
//
// #ifdef WITH_RIVE_TOOLS
//         SynthesizedFailureType synthesizedFailureType =
//             SynthesizedFailureType::none;
// #endif
//     };
//
//     static std::unique_ptr<RenderContext> MakeContext(id<MTLDevice>,
//                                                       const ContextOptions&);
//
//     static std::unique_ptr<RenderContext> MakeContext(id<MTLDevice> gpu)
//     {
//         return MakeContext(gpu, ContextOptions());
//     }
//
//     ~RenderContextMetalImpl() override;
//
//     id<MTLDevice> gpu() const { return m_gpu; }
//
//     // Set the command queue used by makeCommandBuffer(). Must be called
//     // before any ScriptedCanvas flush if canvas support is needed.
//     void setCommandQueue(id<MTLCommandQueue> queue) { m_commandQueue = queue; }
//
//     void* makeCommandBuffer() override;
//     void commitCommandBuffer(void* commandBuffer) override;
//
//     rcp<RenderTargetMetal> makeRenderTarget(MTLPixelFormat,
//                                             uint32_t width,
//                                             uint32_t height);
//
//     rcp<RenderBuffer> makeRenderBuffer(RenderBufferType,
//                                        RenderBufferFlags,
//                                        size_t) override;
//
//     rcp<Texture> makeImageTexture(uint32_t width,
//                                   uint32_t height,
//                                   uint32_t mipLevelCount,
//                                   GPUTextureFormat format,
//                                   const uint8_t imageData[],
//                                   uint8_t blockWidth = 1,
//                                   uint8_t blockHeight = 1,
//                                   bool srgb = false,
//                                   bool generateRemainingMips = false) override;
//
//     // Wrap an externally-owned MTLTexture as a Rive Texture for sampling.
//     // No upload, no allocation; the wrapper retains the MTLTexture via ARC.
//     rcp<Texture> adoptImageTexture(id<MTLTexture> texture,
//                                    uint32_t width,
//                                    uint32_t height);
//
// #ifdef RIVE_CANVAS
//     rcp<RenderCanvas> makeRenderCanvas(uint32_t width,
//                                        uint32_t height) override;
//     std::unique_ptr<rive::ore::Context> makeOreContext() override;
// #endif
//
//     // Atomic mode requires a barrier between overlapping draws. We have to
//     // implement this barrier in various different ways, depending on which
//     // hardware we're on.
//     enum class AtomicBarrierType
//     {
//         // The hardware supports a normal fragment-fragment memory barrier. (Not
//         // supported on Apple-Silicon).
//         memoryBarrier,
//
//         // Apple Silicon is very fast at raster ordering, and doesn't support
//         // fragment-fragment memory barriers anyway, so on this hardware we just
//         // use raster order groups in atomic mode.
//         rasterOrderGroup,
//
//         // On very old hardware that can't support barriers, we just take a
//         // sledge hammer and break the entire render pass between overlapping
//         // draws.
//         // TODO: Is there a lighter way to accomplish this?
//         renderPassBreak,
//     };
//
//     struct MetalFeatures
//     {
//         AtomicBarrierType atomicBarrierType =
//             AtomicBarrierType::renderPassBreak;
//     };
//
//     const MetalFeatures& metalFeatures() const { return m_metalFeatures; }
//
// protected:
//     RenderContextMetalImpl(id<MTLDevice>, const ContextOptions&);
//
//     std::unique_ptr<BufferRing> makeUniformBufferRing(
//         size_t capacityInBytes) override;
//     std::unique_ptr<BufferRing> makeStorageBufferRing(
//         size_t capacityInBytes, StorageBufferStructure) override;
//     std::unique_ptr<BufferRing> makeVertexBufferRing(
//         size_t capacityInBytes) override;
//
// private:
//     // Renders paths to the main render target.
//     class DrawPipeline;
//
//     void resizeGradientTexture(uint32_t width, uint32_t height) override;
//     void resizeTessellationTexture(uint32_t width, uint32_t height) override;
//     void resizeFeatherAtlasTexture(uint32_t width, uint32_t height) override;
//
//     // Obtains an exclusive lock on the next buffer ring index, potentially
//     // blocking until the GPU has finished rendering with it. This ensures it is
//     // safe for the CPU to begin modifying the next buffers in our rings.
//     void prepareToFlush(uint64_t nextFrameNumber,
//                         uint64_t safeFrameNumber) override;
//
//     // Creates a MTLRenderCommandEncoder and sets the common state for PLS
//     // draws.
//     id<MTLRenderCommandEncoder> makeRenderPassForDraws(
//         const gpu::FlushDescriptor&,
//         MTLRenderPassDescriptor*,
//         id<MTLCommandBuffer>,
//         gpu::ShaderMiscFlags baselineMiscFlags);
//
//     // Returns the specific DrawPipeline for the given feature set, if it has
//     // been compiled. If it has not finished compiling yet, this method may
//     // return a (potentially slower) DrawPipeline that can draw a superset of
//     // the given features.
//     const DrawPipeline* findCompatibleDrawPipeline(gpu::DrawType,
//                                                    gpu::ShaderFeatures,
//                                                    const gpu::FlushDescriptor&,
//                                                    gpu::ShaderMiscFlags);
//
//     void flush(const FlushDescriptor&) override;
//
//     void postFlush(const RenderContext::FlushResources&) override;
//
//     const ContextOptions m_contextOptions;
//     const id<MTLDevice> m_gpu;
//     id<MTLCommandQueue> m_commandQueue = nil;
//
//     MetalFeatures m_metalFeatures;
//     std::unique_ptr<BackgroundShaderCompiler> m_backgroundShaderCompiler;
//     id<MTLLibrary> m_plsPrecompiledLibrary; // Many shaders come precompiled in
//                                             // a static library.
//
//     // Renders color ramps to the gradient texture.
//     class ColorRampPipeline;
//     std::unique_ptr<ColorRampPipeline> m_colorRampPipeline;
//     id<MTLTexture> m_gradientTexture = nullptr;
//
//     // Gaussian integral table for feathering.
//     id<MTLTexture> m_gaussianIntegralTexture = nullptr;
//
//     // Renders tessellated vertices to the tessellation texture.
//     class TessellatePipeline;
//     std::unique_ptr<TessellatePipeline> m_tessPipeline;
//     id<MTLBuffer> m_tessSpanIndexBuffer = nullptr;
//     id<MTLTexture> m_tessVertexTexture = nullptr;
//
//     // Atlas rendering.
//     class FeatherAtlasPipeline;
//     std::unique_ptr<FeatherAtlasPipeline> m_featherAtlasFillPipeline;
//     std::unique_ptr<FeatherAtlasPipeline> m_featherAtlasStrokePipeline;
//     id<MTLTexture> m_featherAtlasTexture = nullptr;
//
//     id<MTLSamplerState> m_imageSamplers[ImageSampler::MAX_SAMPLER_PERMUTATIONS];
//
//     std::unordered_map<uint32_t, std::unique_ptr<DrawPipeline>> m_drawPipelines;
//
//     // Vertex/index buffers for drawing path patches.
//     id<MTLBuffer> m_pathPatchVertexBuffer;
//     id<MTLBuffer> m_pathPatchIndexBuffer;
//
//     // Vertex/index buffers for drawing image rects.
//     // (gpu::InterlockMode::atomics only.)
//     id<MTLBuffer> m_imageRectVertexBuffer;
//     id<MTLBuffer> m_imageRectIndexBuffer;
//
//     // Locks buffer contents until the GPU has finished rendering with them.
//     // Prevents the CPU from overriding data before the GPU is done with it.
//     std::mutex m_bufferRingLocks[kBufferRingSize];
//     int m_bufferRingIdx = 0;
// };
// } // namespace rive::gpu
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(test)]
use core::sync::atomic::Ordering;

use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{RefCntTarget, rcp};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
use crate::mechanical_port::source::renderer::include::rive::renderer::buffer_ring_hpp::BufferRing;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawType, FlushDescriptor, PlatformFeatures, ShaderFeatures, ShaderMiscFlags,
    StorageBufferStructure, kBufferRingSize,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, RenderContext,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::RenderContextHelperImpl;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;

// The executable owner is shared with the paired `.mm` translation unit.  The
// source header owns the complete target identity; the implementation module
// supplies the recording/Metal execution trait used by its out-of-line bodies.
use crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::{
    Handle, MetalExecution, MetalObjectKind, OwnedMetalHandle, PixelFormat as SourcePixelFormat,
    OwnerEventPhase, PlatformFeatures as SourcePlatformFeatures, Value,
};

#[cfg(feature = "native-ore-metal-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
#[cfg(feature = "native-ore-metal-experimental")]
use nuxie_ore_metal::metal::context::ContextMetal as OreContext;

// A real +1 Objective-C owner. The pointee remains opaque, but the wrapper is
// pointer-sized and performs the same retain/release transfers as `id<T>`
// strong fields under ARC.
#[repr(transparent)]
pub struct Retained<T> {
    ptr: NonNull<T>,
    marker: PhantomData<T>,
}

#[cfg(target_vendor = "apple")]
unsafe extern "C" {
    fn objc_retain(value: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn objc_release(value: *mut core::ffi::c_void);
}

impl<T> Retained<T> {
    /// Adopt an Objective-C +1 result without adding another retain.
    pub unsafe fn from_raw_retained(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self {
            ptr,
            marker: PhantomData,
        })
    }

    /// Retain a borrowed/autoreleased Objective-C result and adopt that +1.
    pub unsafe fn retain(ptr: *mut T) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        #[cfg(target_vendor = "apple")]
        objc_retain(ptr.as_ptr().cast());
        Some(Self {
            ptr,
            marker: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn into_raw(self) -> *mut T {
        let ptr = self.ptr.as_ptr();
        core::mem::forget(self);
        ptr
    }
}

impl<T> Clone for Retained<T> {
    fn clone(&self) -> Self {
        #[cfg(target_vendor = "apple")]
        unsafe {
            objc_retain(self.ptr.as_ptr().cast());
        }
        Self {
            ptr: self.ptr,
            marker: PhantomData,
        }
    }
}

impl<T> Drop for Retained<T> {
    fn drop(&mut self) {
        #[cfg(target_vendor = "apple")]
        unsafe {
            objc_release(self.ptr.as_ptr().cast());
        }
    }
}

// Objective-C protocol declarations retained as source-shaped type names.
pub struct MTLDevice {
    pub new_private_buffer: unsafe fn(device: *mut MTLDevice, length: usize) -> *mut MTLBuffer,
}
pub struct MTLBuffer;
pub struct MTLCommandBuffer {
    pub commit: unsafe fn(command_buffer: *mut MTLCommandBuffer),
}
pub struct MTLCommandQueue {
    pub command_buffer: unsafe fn(command_queue: *mut MTLCommandQueue) -> *mut MTLCommandBuffer,
}
pub struct MTLLibrary;
pub struct MTLRenderCommandEncoder;
pub struct MTLRenderPassDescriptor;
pub struct MTLTexture {
    pub usage: u64,
    pub width: usize,
    pub height: usize,
    pub pixelFormat: MTLPixelFormat,
}
pub struct MTLSamplerState;

// Metal device and library protocol objects are explicitly documented as
// thread-safe and are transferred through the pinned background worker.
unsafe impl Send for Retained<MTLDevice> {}
unsafe impl Sync for Retained<MTLDevice> {}
unsafe impl Send for Retained<MTLLibrary> {}
unsafe impl Sync for Retained<MTLLibrary> {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MTLPixelFormat(pub u64);

pub const MTLTextureUsageRenderTarget: u64 = 0x0004;

pub use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_h::BackgroundShaderCompiler;
pub use crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RenderContextMetal as RenderContextMetalImpl;

// The transitive shader-compilation header supplies this enum in C++. It is
// retained here because the pinned context option stores it by value.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderCompilationMode {
    allowAsynchronous = 0,
    alwaysSynchronous = 1,
    onlyUbershaders = 2,
}

impl ShaderCompilationMode {
    // The source `standard = allowAsynchronous` enum alias remains an
    // associated value because Rust enum variants cannot share a discriminant.
    pub const standard: Self = Self::allowAsynchronous;
}

#[cfg(feature = "with-rive-tools")]
pub use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType;

// void setTargetTexture(id<MTLTexture> texture);
// id<MTLTexture> targetTexture() const { return m_targetTexture; }
// The borrowed selector argument and nullable retained return preserve the
// Objective-C ownership distinction. The out-of-line setter remains a
// contract for the paired implementation owner.
// The executable RenderTargetMetal owner is defined below; the header
// declaration and the paired .mm implementation share that single identity.
/// The one executable `RenderTargetMetal` owner for the header/.mm pair.
///
/// This is deliberately declared in the header translation, at offset zero
/// with the inherited `RenderTarget` base first.  The `.mm` translation unit
/// re-exports this identity and provides the flush/context call sites; it does
/// not define a second complete target class.  Field names and order mirror
/// the pinned Objective-C++ declaration exactly: device, format, target,
/// coverage, clip, scratch, color atomic, coverage atomic, clip atomic.
#[repr(C)]
pub struct RenderTargetMetal {
    pub(crate) base: ManuallyDrop<RenderTarget>,
    pub(crate) m_gpu: OwnedMetalHandle,
    pub(crate) m_pixelFormat: SourcePixelFormat,
    pub(crate) m_targetTexture: Option<OwnedMetalHandle>,
    pub(crate) m_coverageMemorylessTexture: Option<OwnedMetalHandle>,
    pub(crate) m_clipMemorylessTexture: Option<OwnedMetalHandle>,
    pub(crate) m_scratchColorMemorylessTexture: Option<OwnedMetalHandle>,
    pub(crate) m_colorAtomicBuffer: Option<OwnedMetalHandle>,
    pub(crate) m_coverageAtomicBuffer: Option<OwnedMetalHandle>,
    pub(crate) m_clipAtomicBuffer: Option<OwnedMetalHandle>,
}

impl Drop for RenderTargetMetal {
    fn drop(&mut self) {
        // Source reverse declaration order.  Handles are registry-owned
        // tokens, so taking each option is the explicit release boundary; the
        // outer mechanical adapter queues the corresponding native retirements
        // after this complete owner is gone.
        let _ = self.m_clipAtomicBuffer.take();
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("clipAtomic");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        let _ = self.m_coverageAtomicBuffer.take();
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("coverageAtomic");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        let _ = self.m_colorAtomicBuffer.take();
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("colorAtomic");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        let _ = self.m_scratchColorMemorylessTexture.take();
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("scratch");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        let _ = self.m_clipMemorylessTexture.take();
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("clip");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        let _ = self.m_coverageMemorylessTexture.take();
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("coverage");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        let _ = self.m_targetTexture.take();
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("target");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        let gpu = core::mem::replace(&mut self.m_gpu, OwnedMetalHandle::token(Handle::NIL));
        let _gpu = gpu.handle();
        drop(gpu);
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("gpu");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        // The inherited RefCntTarget base is the final source member.  It is
        // manually released only after every derived native member and the
        // retained device have been released, matching the pinned destructor
        // order instead of relying on Rust's declaration-order glue.
        unsafe { ManuallyDrop::drop(&mut self.base) };
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_TRACE
            .lock().unwrap().push("base");
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_STAGE.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::RENDER_TARGET_METAL_DROP_COUNT
            .fetch_add(1, Ordering::SeqCst);
        // m_pixelFormat is Copy; all source-owned resource boundaries above
        // have now been released explicitly.
    }
}

// SAFETY: the intrusive count lives in the offset-zero `RenderTarget` base,
// exactly as in the C++ inherited RefCntTarget implementation.  Complete
// destruction is routed back through the base's derived-owner callback.
unsafe impl RefCntTarget for RenderTargetMetal {
    fn r#ref(&self) {
        unsafe { (&*self.base).r#ref() };
    }

    unsafe fn unref(&self) {
        unsafe { (&*self.base).unref() };
    }

    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        let base = ptr.cast::<RenderTarget>().cast_mut();
        unsafe { ((*base).destroy_complete)(base) };
    }
}

fn source_memoryless_texture<E: MetalExecution>(
    metal: &mut E,
    device: Handle,
    format: SourcePixelFormat,
    width: u32,
    height: u32,
) -> Option<OwnedMetalHandle> {
    let Some(descriptor) = metal.call("MTLTextureDescriptor", "alloc/init", vec![]) else {
        return None;
    };
    metal.owner_event("RC-TD-MEMORYLESS-X3", OwnerEventPhase::Create, descriptor);
    let set = |metal: &mut E, selector: &'static str, value: Value| {
        let _ = metal.call(
            "textureDescriptor",
            selector,
            vec![Value::Handle(descriptor), value],
        );
    };
    set(metal, "setPixelFormat:", Value::U64(format as u64));
    set(metal, "setTextureType:", Value::U64(2));
    set(metal, "setWidth:", Value::U64(width as u64));
    set(metal, "setHeight:", Value::U64(height as u64));
    set(metal, "setMipmapLevelCount:", Value::U64(1));
    set(metal, "setUsage:", Value::U64(4));
    set(metal, "setStorageMode:", Value::U64(3));
    let handle = metal.call(
        "gpu",
        "newTextureWithDescriptor:",
        vec![Value::Handle(device), Value::Handle(descriptor)],
    );
    metal.owner_event(
        "RC-TD-MEMORYLESS-X3",
        OwnerEventPhase::LastUse,
        descriptor,
    );
    // The source descriptor is a lexical local.  Release it after the
    // texture selector returns, including the nil allocation path.
    metal.retire_handle(descriptor);
    metal.owner_event("RC-TD-MEMORYLESS-X3", OwnerEventPhase::Release, descriptor);
    handle.and_then(|handle| metal.take_owned(handle, MetalObjectKind::Texture))
}

impl RenderTargetMetal {
    #[cfg(test)]
    pub(crate) fn new<E: MetalExecution>(
        metal: &mut E,
        format: SourcePixelFormat,
        width: u32,
        height: u32,
        features: SourcePlatformFeatures,
    ) -> Self {
        unsafe fn destroy_complete(ptr: *mut RenderTarget) {
            unsafe { drop(Box::from_raw(ptr.cast::<RenderTargetMetal>())) };
        }
        let device = metal
            .clone_owned(metal.device_handle(), MetalObjectKind::Device)
            .unwrap_or_else(|| OwnedMetalHandle::token(metal.device_handle()));
        Self::new_with_device(metal, device, format, width, height, features)
    }

    pub(crate) fn new_with_device<E: MetalExecution>(
        metal: &mut E,
        mut device: OwnedMetalHandle,
        format: SourcePixelFormat,
        width: u32,
        height: u32,
        features: SourcePlatformFeatures,
    ) -> Self {
        let _ = metal.publish_owned(&mut device);
        unsafe fn destroy_complete(ptr: *mut RenderTarget) {
            unsafe { drop(Box::from_raw(ptr.cast::<RenderTargetMetal>())) };
        }
        let mut base = RenderTarget::new(width, height);
        base.destroy_complete = destroy_complete;
        let mut target = Self {
            base: ManuallyDrop::new(base),
            m_gpu: device,
            m_pixelFormat: format,
            m_targetTexture: None,
            m_coverageMemorylessTexture: None,
            m_clipMemorylessTexture: None,
            m_scratchColorMemorylessTexture: None,
            m_colorAtomicBuffer: None,
            m_coverageAtomicBuffer: None,
            m_clipAtomicBuffer: None,
        };
        if features.supportsRasterOrderingMode {
            target.m_coverageMemorylessTexture = source_memoryless_texture(
                metal,
                target.m_gpu.handle(),
                SourcePixelFormat::R32Uint,
                width,
                height,
            );
            target.m_clipMemorylessTexture = source_memoryless_texture(
                metal,
                target.m_gpu.handle(),
                SourcePixelFormat::R32Uint,
                width,
                height,
            );
            target.m_scratchColorMemorylessTexture =
                source_memoryless_texture(metal, target.m_gpu.handle(), format, width, height);
        }
        target
    }

    pub(crate) fn set_target_texture<E: MetalExecution>(
        &mut self,
        metal: &mut E,
        texture: Option<Handle>,
    ) {
        #[cfg(debug_assertions)]
        {
            let compatible = texture.is_none_or(|handle| {
                metal.texture_compatible(
                    handle,
                    unsafe { (&*self.base).width() },
                    unsafe { (&*self.base).height() },
                    self.m_pixelFormat,
                )
            });
            debug_assert!(compatible);
        }
        // C++ only asserts compatibility; in NDEBUG it still performs the
        // strong assignment. Product adapters perform their checked admission
        // before entering this source seam.
        self.m_targetTexture =
            texture.and_then(|handle| metal.clone_owned(handle, MetalObjectKind::Texture));
    }

    pub(crate) fn target_handle(&self) -> Option<Handle> {
        self.m_targetTexture.as_ref().map(OwnedMetalHandle::handle)
    }

    pub(crate) fn format(&self) -> SourcePixelFormat {
        self.m_pixelFormat
    }

    pub(crate) fn coverage_handle(&self) -> Option<Handle> {
        self.m_coverageMemorylessTexture
            .as_ref()
            .map(OwnedMetalHandle::handle)
    }

    pub(crate) fn clip_handle(&self) -> Option<Handle> {
        self.m_clipMemorylessTexture
            .as_ref()
            .map(OwnedMetalHandle::handle)
    }

    pub(crate) fn scratch_handle(&self) -> Option<Handle> {
        self.m_scratchColorMemorylessTexture
            .as_ref()
            .map(OwnedMetalHandle::handle)
    }

    fn make_atomic(&self) -> Option<OwnedMetalHandle> {
        let length =
            unsafe { (&*self.base).width() as usize * (&*self.base).height() as usize * 4 };
        #[cfg(target_vendor = "apple")]
        {
            return self.m_gpu.new_buffer_with_length(
                length,
                objc2_metal::MTLResourceOptions::StorageModePrivate,
            );
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            let _ = length;
            None
        }
    }

    /// Source-shaped lazy getter: allocation is performed directly by the
    /// constructor-retained device and does not depend on a live executor.
    pub(crate) fn color_atomic_buffer(&mut self) -> Option<&OwnedMetalHandle> {
        if self.m_colorAtomicBuffer.is_none() {
            self.m_colorAtomicBuffer = self.make_atomic();
        }
        self.m_colorAtomicBuffer.as_ref()
    }

    /// Mechanical adapter publication for selector-based flush recording.
    pub(crate) fn color_atomic_buffer_handle<E: MetalExecution>(
        &mut self,
        metal: &mut E,
    ) -> Option<Handle> {
        self.color_atomic_buffer()?;
        let owner = self.m_colorAtomicBuffer.as_mut()?;
        metal.publish_owned(owner)?;
        self.m_colorAtomicBuffer
            .as_ref()
            .map(OwnedMetalHandle::handle)
    }

    pub(crate) fn coverage_atomic_buffer(&mut self) -> Option<&OwnedMetalHandle> {
        if self.m_coverageAtomicBuffer.is_none() {
            self.m_coverageAtomicBuffer = self.make_atomic();
        }
        self.m_coverageAtomicBuffer.as_ref()
    }

    pub(crate) fn coverage_atomic_buffer_handle<E: MetalExecution>(
        &mut self,
        metal: &mut E,
    ) -> Option<Handle> {
        self.coverage_atomic_buffer()?;
        let owner = self.m_coverageAtomicBuffer.as_mut()?;
        metal.publish_owned(owner)?;
        self.m_coverageAtomicBuffer
            .as_ref()
            .map(OwnedMetalHandle::handle)
    }

    pub(crate) fn clip_atomic_buffer(&mut self) -> Option<&OwnedMetalHandle> {
        if self.m_clipAtomicBuffer.is_none() {
            self.m_clipAtomicBuffer = self.make_atomic();
        }
        self.m_clipAtomicBuffer.as_ref()
    }

    pub(crate) fn clip_atomic_buffer_handle<E: MetalExecution>(
        &mut self,
        metal: &mut E,
    ) -> Option<Handle> {
        self.clip_atomic_buffer()?;
        let owner = self.m_clipAtomicBuffer.as_mut()?;
        metal.publish_owned(owner)?;
        self.m_clipAtomicBuffer
            .as_ref()
            .map(OwnedMetalHandle::handle)
    }
}

// struct ContextOptions
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextOptions {
    // ShaderCompilationMode shaderCompilationMode =
    //     ShaderCompilationMode::standard;
    pub shaderCompilationMode: ShaderCompilationMode,

    // bool disableFramebufferReads = false;
    pub disableFramebufferReads: bool,

    #[cfg(feature = "with-rive-tools")]
    // SynthesizedFailureType synthesizedFailureType =
    //     SynthesizedFailureType::none;
    pub synthesizedFailureType: SynthesizedFailureType,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            shaderCompilationMode: ShaderCompilationMode::standard,
            disableFramebufferReads: false,
            #[cfg(feature = "with-rive-tools")]
            synthesizedFailureType: SynthesizedFailureType::none,
        }
    }
}

// static std::unique_ptr<RenderContext> MakeContext(id<MTLDevice>,
//                                                   const ContextOptions&);
// static overload MakeContext(id<MTLDevice>) delegates to ContextOptions().
// The factory body belongs to the pinned .mm owner; the source-visible Rust
// contract retains both overloads and exact option-copy timing.
// MakeContext is implemented by the canonical RenderContextMetal owner in
// the paired .mm translation. No parallel factory trait is materialized here.
// enum class AtomicBarrierType
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomicBarrierType {
    // memoryBarrier,
    memoryBarrier = 0,
    // rasterOrderGroup,
    rasterOrderGroup = 1,
    // renderPassBreak,
    renderPassBreak = 2,
}

// struct MetalFeatures
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetalFeatures {
    // AtomicBarrierType atomicBarrierType =
    //     AtomicBarrierType::renderPassBreak;
    pub atomicBarrierType: AtomicBarrierType,
}

impl Default for MetalFeatures {
    fn default() -> Self {
        Self {
            atomicBarrierType: AtomicBarrierType::renderPassBreak,
        }
    }
}

// Forward declarations of the nested pipeline owners. Their complete source
// definitions are owned by the pinned implementation translation unit.
pub use crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::{
    ColorRampPipeline, DrawPipeline, FeatherAtlasPipeline, TessellatePipeline,
};

/// A +1 command-buffer transfer created exactly once and consumed exactly
/// once by `commitCommandBuffer`. It is intentionally neither Copy nor Clone.
// Command-buffer ownership uses the canonical raw *mut c_void source contract
// and the live Handle bridge in the implementation translation. This header
// intentionally does not define a second token or contract universe.
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::{
        Handle, MetalObjectKind, PixelFormat, PlatformFeatures as SourcePlatformFeatures,
        RecordingMetal, RenderContextMetal as SourceRenderContextMetal,
        RenderTargetMetal as SourceRenderTargetMetal,
        BufferRingMetal,
        TextureMetal, RENDER_TARGET_METAL_DROP_COUNT, RENDER_TARGET_METAL_DROP_STAGE,
        RENDER_TARGET_METAL_DROP_TRACE, RENDER_CONTEXT_METAL_DROP_TRACE,
        TEXTURE_METAL_DROP_COUNT,
    };

    static RENDER_TARGET_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn render_target_metal_base_is_offset_zero() {
        assert_eq!(core::mem::offset_of!(RenderTargetMetal, base), 0);
    }

    #[test]
    fn texture_metal_base_is_offset_zero_and_intrusive_release_is_once() {
        assert_eq!(TextureMetal::base_offset_for_test(), 0);
        TEXTURE_METAL_DROP_COUNT.store(0, Ordering::SeqCst);
        let texture = TextureMetal::from_native(
            crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::OwnedMetalHandle::token(Handle::NIL),
            4,
            4,
        );
        let owner = Box::into_raw(Box::new(texture));
        unsafe { TextureMetal::release_for_test(owner) };
        assert_eq!(TEXTURE_METAL_DROP_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn buffer_ring_metal_base_is_offset_zero() {
        assert_eq!(BufferRingMetal::base_offset_for_test(), 0);
    }

    #[test]
    fn render_target_metal_constructor_is_the_translated_source_seam() {
        let _lock = RENDER_TARGET_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut metal = RecordingMetal::default();
        let target = RenderTargetMetal::new(
            &mut metal,
            PixelFormat::RGBA8Unorm,
            11,
            13,
            SourcePlatformFeatures {
                supportsRasterOrderingMode: false,
                supportsAtomicMode: true,
                ..Default::default()
            },
        );
        assert_eq!(target.base.width(), 11);
        assert_eq!(target.base.height(), 13);
        assert_eq!(target.format(), PixelFormat::RGBA8Unorm);
    }

    #[test]
    fn render_target_metal_rcp_clone_drops_complete_owner_once() {
        let _lock = RENDER_TARGET_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        RENDER_TARGET_METAL_DROP_COUNT.store(0, Ordering::SeqCst);
        RENDER_TARGET_METAL_DROP_STAGE.store(0, Ordering::SeqCst);
        RENDER_TARGET_METAL_DROP_TRACE.lock().unwrap().clear();
        let mut metal = RecordingMetal::default();
        let target = RenderTargetMetal::new(
            &mut metal,
            PixelFormat::RGBA8Unorm,
            11,
            13,
            SourcePlatformFeatures {
                supportsRasterOrderingMode: false,
                supportsAtomicMode: true,
                ..Default::default()
            },
        );
        let owner = unsafe { rcp::from_ptr(Box::into_raw(Box::new(target))) };
        let clone = owner.clone();
        drop(clone);
        assert_eq!(RENDER_TARGET_METAL_DROP_COUNT.load(Ordering::SeqCst), 0);
        drop(owner);
        assert_eq!(RENDER_TARGET_METAL_DROP_COUNT.load(Ordering::SeqCst), 1);
        assert_eq!(RENDER_TARGET_METAL_DROP_STAGE.load(Ordering::SeqCst), 9);
        assert_eq!(
            *RENDER_TARGET_METAL_DROP_TRACE.lock().unwrap(),
            [
                "clipAtomic",
                "coverageAtomic",
                "colorAtomic",
                "scratch",
                "clip",
                "coverage",
                "target",
                "gpu",
                "base",
            ]
        );
    }

    #[test]
    fn render_target_metal_identity_is_shared_by_header_and_mm_owner() {
        let _: Option<RenderTargetMetal> = None::<SourceRenderTargetMetal>;
        let _ = core::mem::size_of::<SourceRenderContextMetal>();
    }

    #[test]
    fn render_context_metal_releases_source_members_in_reverse_order() {
        let _lock = RENDER_TARGET_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        RENDER_CONTEXT_METAL_DROP_TRACE.lock().unwrap().clear();
        let mut metal = RecordingMetal::default();
        let context = SourceRenderContextMetal::new(
            &mut metal,
            Handle::new(1, MetalObjectKind::Device),
            ContextOptions {
                shaderCompilationMode: ShaderCompilationMode::alwaysSynchronous,
                ..ContextOptions::default()
            },
        );
        drop(context);
        let trace = RENDER_CONTEXT_METAL_DROP_TRACE.lock().unwrap().clone();
        let position = |name| trace.iter().position(|stage| *stage == name).unwrap();
        assert!(position("backgroundShaderCompiler") < position("commandQueue"));
        assert!(position("commandQueue") < position("gpu"));
        assert!(position("gpu") < position("contextOptions"));
    }

    #[test]
    fn texture_allocation_nil_still_constructs_source_owner() {
        let mut metal = RecordingMetal::default();
        metal.fail.push_back("newTextureWithDescriptor:");
        let device = metal.device_handle();
        let texture = TextureMetal::new(
            &mut metal,
            device,
            1,
            1,
            1,
            Arc::from([0_u8; 4]),
            PixelFormat::RGBA8Unorm,
            1,
            1,
            4,
            false,
        )
        .expect("pinned constructor returns an owner after nil allocation");
        assert_eq!(texture.native_handle(), Handle::NIL);
        assert!(metal.calls.iter().any(|call| {
            call.selector == "replaceRegion:mipmapLevel:withBytes:bytesPerRow:"
                && call.args.first() == Some(&crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::Value::Handle(Handle::NIL))
        }));
    }

    #[test]
    fn nil_blit_encoder_still_clears_mipmap_dirty_state() {
        let mut metal = RecordingMetal::default();
        metal.fail.push_back("blitCommandEncoder");
        let mut texture = TextureMetal::adopt(
            Some(OwnedMetalHandle::token(Handle::new(
                7,
                MetalObjectKind::Texture,
            ))),
            4,
            4,
        )
        .expect("valid source texture");
        texture.mark_mipmaps_dirty_for_test();
        texture.ensure_mipmaps(&mut metal, Handle::new(9, MetalObjectKind::CommandBuffer));
        assert!(!texture.mipmaps_dirty_for_test());
        assert!(!metal
            .calls
            .iter()
            .any(|call| { call.selector == "generateMipmapsForTexture:" }));
    }

    #[test]
    fn adopted_nil_texture_is_rejected() {
        assert!(TextureMetal::adopt(None, 4, 4).is_none());
        assert!(TextureMetal::adopt(None, 4, 4).is_none());
        assert!(TextureMetal::adopt(Some(OwnedMetalHandle::token(Handle::NIL)), 4, 4).is_none());
        assert!(TextureMetal::adopt(
            Some(OwnedMetalHandle::token(Handle::new(
                7,
                MetalObjectKind::Texture
            ))),
            4,
            4,
        )
        .is_some());
    }

    #[cfg(feature = "native-ore-metal-experimental")]
    #[test]
    fn render_target_outlives_context_and_queue_field_replacement() {
        use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::RenderContextHelperImpl;
        use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImpl;
        use crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::{
            source_execution::RecordingCloneOwnerEvent, AtomicBarrierType,
        };

        let _lock = RENDER_TARGET_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut metal = RecordingMetal::default();
        let mut context = SourceRenderContextMetal::new(
            &mut metal,
            Handle::new(1, MetalObjectKind::Device),
            ContextOptions {
                shaderCompilationMode: ShaderCompilationMode::alwaysSynchronous,
                ..ContextOptions::default()
            },
        );
        metal.fail.push_back("newTextureWithDescriptor:");
        let (texture, target, descriptor) = context
            .make_render_canvas(&mut metal, 3, 5)
            .expect("pinned makeRenderCanvas returns its source owners after nil allocation");
        let first_queue = Handle::new(100, MetalObjectKind::CommandQueue);
        context.set_command_queue(&mut metal, Some(first_queue));
        let first_alias = context.command_queue().expect("first queue strong member");
        assert_ne!(first_alias, first_queue);
        assert_eq!(first_alias.kind, MetalObjectKind::CommandQueue);

        let replacement_queue = Handle::new(200, MetalObjectKind::CommandQueue);
        context.set_command_queue(&mut metal, Some(replacement_queue));
        let replacement_alias = context
            .command_queue()
            .expect("replacement queue strong member");
        assert_ne!(replacement_alias, replacement_queue);
        assert_ne!(replacement_alias, first_alias);
        assert_eq!(texture.native_handle(), Handle::NIL);
        assert_eq!(target.target_handle(), None);
        assert_eq!((target.base.width(), target.base.height()), (3, 5));
        metal.retire_handle(descriptor);
        drop(context);
        assert_eq!((target.base.width(), target.base.height()), (3, 5));
        let queue_events = metal
            .recording_clone_events()
            .into_iter()
            .filter(|event| match event {
                RecordingCloneOwnerEvent::Clone { alias, .. }
                | RecordingCloneOwnerEvent::Drop { alias } => {
                    alias.kind == MetalObjectKind::CommandQueue
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            queue_events,
            vec![
                RecordingCloneOwnerEvent::Clone {
                    alias: first_alias,
                    source: first_queue,
                },
                RecordingCloneOwnerEvent::Clone {
                    alias: replacement_alias,
                    source: replacement_queue,
                },
                RecordingCloneOwnerEvent::Drop { alias: first_alias },
                RecordingCloneOwnerEvent::Drop {
                    alias: replacement_alias,
                },
            ],
            "source strong assignment retains the replacement before releasing the prior queue"
        );
    }
}
