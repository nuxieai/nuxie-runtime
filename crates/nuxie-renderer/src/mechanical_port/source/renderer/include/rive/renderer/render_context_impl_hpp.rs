/*
 * Copyright 2025 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/render_context_impl.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// /*
//  * Copyright 2023 Rive
//  */
//
// #pragma once
//
// #include "rive/renderer/render_context.hpp"
// #include "rive/gpu_texture_format.hpp"
//
// #ifdef RIVE_CANVAS
// #include <memory>
// #endif
//
// #ifdef RIVE_CANVAS
// namespace rive::ore
// {
// class Context;
// };
// #endif
//
// namespace rive::gpu
// {
// #ifdef RIVE_CANVAS
// class RenderCanvas;
// #endif
// class Texture;
//
// // This class manages GPU buffers and isues the actual rendering commands from
// // RenderContext.
// class RenderContextImpl
// {
// public:
//     virtual ~RenderContextImpl() {}
//
//     const PlatformFeatures& platformFeatures() const
//     {
//         return m_platformFeatures;
//     }
//
//     virtual rcp<RenderBuffer> makeRenderBuffer(RenderBufferType,
//                                                RenderBufferFlags,
//                                                size_t) = 0;
//
//     // Use platform apis to decode the image bytes and creates a texture if
//     // available. If not available leaving its default implementation will cause
//     // rive decoders to be used instead
//     virtual rcp<Texture> platformDecodeImageTexture(
//         Span<const uint8_t> encodedBytes)
//     {
//         return nullptr;
//     };
//
//     // this is called in the case of the default Bitmap class being used to
//     // decode images so that it can be converted into a backend specific image.
//     // For compressed `format`s, `blockWidth`/`blockHeight` give the format's
//     // block footprint (e.g. 4x4 for BC7 and ASTC 4x4) and `srgb` selects the
//     // sRGB variant of the format. For rgba32 these are ignored.
//     //
//     // `mipLevelCount` is the number of stored mip levels in `imageData`,
//     // packed largest-first with no inter-level padding. When
//     // `generateRemainingMips` is true (PNG/JPEG path), only mip 0 bytes are
//     // expected in `imageData` and the backend fills the remaining levels
//     // via GPU blits. When false (KTX2 path), the caller has supplied the
//     // full chain and the backend uploads it verbatim.
//     virtual rcp<Texture> makeImageTexture(
//         uint32_t width,
//         uint32_t height,
//         uint32_t mipLevelCount,
//         GPUTextureFormat format,
//         const uint8_t imageData[],
//         uint8_t blockWidth = 1,
//         uint8_t blockHeight = 1,
//         bool srgb = false,
//         bool generateRemainingMips = false) = 0;
//
// #ifdef RIVE_CANVAS
//     // Creates a RenderCanvas: a GPU texture usable as both a render target
//     // and a render image. Returns nullptr if not supported by this backend.
//     virtual rcp<RenderCanvas> makeRenderCanvas(uint32_t width, uint32_t height)
//     {
//         return nullptr;
//     }
//
//     // If canvas is enabled then the backend Impl MUST implement this.
//     virtual std::unique_ptr<rive::ore::Context> makeOreContext() = 0;
// #endif
//
//     // Resize GPU buffers. These methods cannot fail, and must allocate the
//     // exact size requested.
//     //
//     // RenderContext takes care to minimize how often these methods are called,
//     // while also growing and shrinking the memory footprint to fit current
//     // usage.
//     //
//     // 'elementSizeInBytes' represents the size of one array element when the
//     // shader accesses this buffer as a storage buffer.
//     virtual void resizeFlushUniformBuffer(size_t sizeInBytes) = 0;
//     virtual void resizePathBuffer(size_t sizeInBytes,
//                                   gpu::StorageBufferStructure) = 0;
//     virtual void resizePaintBuffer(size_t sizeInBytes,
//                                    gpu::StorageBufferStructure) = 0;
//     virtual void resizePaintAuxBuffer(size_t sizeInBytes,
//                                       gpu::StorageBufferStructure) = 0;
//     virtual void resizeContourBuffer(size_t sizeInBytes,
//                                      gpu::StorageBufferStructure) = 0;
//     virtual void resizeGradSpanBuffer(size_t sizeInBytes) = 0;
//     virtual void resizeTessVertexSpanBuffer(size_t sizeInBytes) = 0;
//     virtual void resizeTriangleVertexBuffer(size_t sizeInBytes) = 0;
//     virtual void resizeImageDrawInstanceBuffer(size_t sizeInBytes) = 0;
//
//     virtual void preBeginFrame(RenderContext*) {}
//
//     // Returns true if the render context should end the drawList with a batch
//     // of type DrawType::renderPassResolve (and set "manuallyResolved" in the
//     // flush descriptor).
//     // This may be used, e.g., to manually resolve MSAA or to transfer pixels
//     // from an offscreen texture back to the main render target.
//     virtual bool wantsManualRenderPassResolve(
//         gpu::InterlockMode,
//         const RenderTarget*,
//         const IAABB& renderTargetUpdateBounds,
//         uint32_t virtualTileWidth,
//         uint32_t virtualTileHeight,
//         gpu::DrawContents combinedDrawContents) const
//     {
//         return false;
//     }
//
//     // Perform any bookkeeping or other tasks that need to run before
//     // RenderContext begins accessing GPU resources for the flush. (Update
//     // counters, advance buffer pools, etc.)
//     //
//     // The provided resource lifetime counters communicate how the client is
//     // performing CPU-GPU synchronization. Resources used during the upcoming
//     // flush will belong to 'nextFrameNumber'. Resources last used on or before
//     // 'safeFrameNumber' are safe to be released or recycled.
//     virtual void prepareToFlush(uint64_t nextFrameNumber,
//                                 uint64_t safeFrameNumber)
//     {}
//
//     // Map GPU buffers. (The implementation may wish to allocate the mappable
//     // buffers in rings, in order to avoid expensive synchronization with the
//     // GPU pipeline. See RenderContextBufferRingImpl.)
//     virtual void* mapFlushUniformBuffer(size_t mapSizeInBytes) = 0;
//     virtual void* mapPathBuffer(size_t mapSizeInBytes) = 0;
//     virtual void* mapPaintBuffer(size_t mapSizeInBytes) = 0;
//     virtual void* mapPaintAuxBuffer(size_t mapSizeInBytes) = 0;
//     virtual void* mapContourBuffer(size_t mapSizeInBytes) = 0;
//     virtual void* mapGradSpanBuffer(size_t mapSizeInBytes) = 0;
//     virtual void* mapTessVertexSpanBuffer(size_t mapSizeInBytes) = 0;
//     virtual void* mapTriangleVertexBuffer(size_t mapSizeInBytes) = 0;
//     virtual void* mapImageDrawInstanceBuffer(size_t mapSizeInBytes) = 0;
//
//     // Unmap GPU buffers. All buffers will be unmapped before flush().
//     virtual void unmapFlushUniformBuffer(size_t mapSizeInBytes) = 0;
//     virtual void unmapPathBuffer(size_t mapSizeInBytes) = 0;
//     virtual void unmapPaintBuffer(size_t mapSizeInBytes) = 0;
//     virtual void unmapPaintAuxBuffer(size_t mapSizeInBytes) = 0;
//     virtual void unmapContourBuffer(size_t mapSizeInBytes) = 0;
//     virtual void unmapGradSpanBuffer(size_t mapSizeInBytes) = 0;
//     virtual void unmapTessVertexSpanBuffer(size_t mapSizeInBytes) = 0;
//     virtual void unmapTriangleVertexBuffer(size_t mapSizeInBytes) = 0;
//     virtual void unmapImageDrawInstanceBuffer(size_t mapSizeInBytes) = 0;
//
//     // Allocate resources that are updated and used during flush().
//     virtual void resizeGradientTexture(uint32_t width, uint32_t height) = 0;
//     virtual void resizeTessellationTexture(uint32_t width, uint32_t height) = 0;
//     virtual void resizeFeatherAtlasTexture(uint32_t width, uint32_t height)
//     {
//         // Override this method to support atlas feathering.
//         assert(width == 0 && height == 0);
//     }
//     // Not all APIs support pure memoryless pixel local storage. This optional
//     // resource is a space to store PLS data that does not persist outside a
//     // render pass. (Namely, coverage, clip, and scratch.)
//     // NOTE: It is specified as a TEXTURE_2D_ARRAY because that gets better
//     // cache performance on Intel Arc than separate textures.
//     constexpr static uint32_t PLS_TRANSIENT_BACKING_MAX_PLANE_COUNT = 3;
//     virtual void resizeTransientPLSBacking(uint32_t width,
//                                            uint32_t height,
//                                            uint32_t planeCount)
//     {}
//     // Used in atomic mode. Similar to transient PLS backing, except it's a
//     // single 2D resource that also supports atomic operations.
//     virtual void resizeAtomicCoverageBacking(uint32_t width, uint32_t height) {}
//     virtual void resizeCoverageBuffer(size_t sizeInBytes)
//     {
//         // Override this method to support the experimental clockwiseAtomic
//         // mode.
//         assert(sizeInBytes == 0);
//     }
//
//     // Perform rendering in three steps:
//     //
//     //  1. Prepare the gradient texture:
//     //      * Render the GradientSpan instances into the gradient texture.
//     //      * Copy the TwoTexelRamp data directly into the gradient texture.
//     //
//     //  2. Render the TessVertexSpan instances into the tessellation texture.
//     //
//     //  3. Execute the draw list. (The Rive renderer shaders read the gradient
//     //     and tessellation textures in order to do path rendering.)
//     //
//     // A single frame may have multiple logical flushes (and call flush()
//     // multiple times).
//     virtual void flush(const gpu::FlushDescriptor&) = 0;
//
//     // Called after all logical flushes in a frame have completed.
//     virtual void postFlush(const RenderContext::FlushResources&) {}
//
//     // Creates a platform-specific command buffer for use with flush().
//     // Returns an opaque pointer that should be passed as
//     // FlushResources::externalCommandBuffer.
//     // The default implementation returns nullptr (not supported).
//     virtual void* makeCommandBuffer() { return nullptr; }
//
//     // Commits a command buffer previously created by makeCommandBuffer().
//     // Called after flush() to submit the GPU work.
//     virtual void commitCommandBuffer(void* commandBuffer) {}
//
//     // Steady clock, used to determine when we should trim our resource
//     // allocations.
//     virtual double secondsNow() const = 0;
//
// protected:
//     PlatformFeatures m_platformFeatures;
// };
// } // namespace rive::gpu
//
// #if defined(ORE_BACKEND_GL) && defined(RIVE_CANVAS)
// namespace rive
// {
// class RiveRenderImage;
// // Returns a Y-flipped companion of a GL canvas texture, or nullptr on
// // non-GL backends. Hides the RenderContextGLImpl downcast so callers
// // don't need GL headers.
// rcp<RiveRenderImage> getCanvasImportMirrorGL(gpu::RenderContext*,
//                                              gpu::Texture* sourceTex,
//                                              uint32_t width,
//                                              uint32_t height);
// } // namespace rive
// #endif

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_void;

use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat;
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawContents, FlushDescriptor, InterlockMode, PlatformFeatures, StorageBufferStructure, IAABB,
};
#[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::OreContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, RenderContext,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;

#[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
#[cfg(feature = "ore-gl")]
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;

// namespace rive::gpu
// {
// #ifdef RIVE_CANVAS
// class RenderCanvas;
// #endif
// class Texture;
//
// The C++ class below is abstract because its pure virtual members define the
// backend contract. Rust keeps the data-bearing owner and the overridable
// member surface as separate source-shaped declarations. The field remains
// in source declaration order and retains the PlatformFeatures defaults.
#[repr(C)]
pub struct RenderContextImpl {
    // protected:
    // PlatformFeatures m_platformFeatures;
    pub(crate) m_platformFeatures: PlatformFeatures,
}

impl Default for RenderContextImpl {
    // The C++ implicit constructor value-initializes the PlatformFeatures member,
    // applying every pinned field default from gpu.hpp.
    fn default() -> Self {
        Self {
            m_platformFeatures: PlatformFeatures::default(),
        }
    }
}

impl Drop for RenderContextImpl {
    // virtual ~RenderContextImpl() {}
    // Rust's default drop glue is the empty virtual-destructor body; the
    // source-owned platform feature snapshot has no independent resources.
    fn drop(&mut self) {}
}

impl RenderContextImpl {
    // const PlatformFeatures& platformFeatures() const
    // {
    //     return m_platformFeatures;
    // }
    pub fn platformFeatures(&self) -> &PlatformFeatures {
        &self.m_platformFeatures
    }

    // constexpr static uint32_t PLS_TRANSIENT_BACKING_MAX_PLANE_COUNT = 3;
    pub const PLS_TRANSIENT_BACKING_MAX_PLANE_COUNT: u32 = 3;
}

// Rust has no C++ virtual table slots. This trait preserves each pure-virtual
// declaration and every source-provided default implementation in declaration
// order. Concrete backend owners implement the trait without changing the
// source dispatch seam.
pub trait RenderContextImplContract {
    fn renderContextImpl(&self) -> &RenderContextImpl;
    fn renderContextImplMut(&mut self) -> &mut RenderContextImpl;

    // virtual rcp<RenderBuffer> makeRenderBuffer(RenderBufferType,
    //                                            RenderBufferFlags,
    //                                            size_t) = 0;
    fn makeRenderBuffer(
        &mut self,
        bufferType: RenderBufferType,
        bufferFlags: RenderBufferFlags,
        sizeInBytes: usize,
    ) -> rcp<RenderBuffer>;

    // virtual rcp<Texture> platformDecodeImageTexture(
    //     Span<const uint8_t> encodedBytes)
    // {
    //     return nullptr;
    // };
    //
    // Span<const uint8_t> is a borrowed byte slice; the default empty rcp is
    // the source nullptr result and does not publish a texture owner.
    fn platformDecodeImageTexture(&mut self, encodedBytes: &[u8]) -> rcp<Texture> {
        let _ = encodedBytes;
        rcp::new()
    }

    // virtual rcp<Texture> makeImageTexture(
    //     uint32_t width,
    //     uint32_t height,
    //     uint32_t mipLevelCount,
    //     GPUTextureFormat format,
    //     const uint8_t imageData[],
    //     uint8_t blockWidth = 1,
    //     uint8_t blockHeight = 1,
    //     bool srgb = false,
    //     bool generateRemainingMips = false) = 0;
    //
    // The source array parameter is a borrowed pointer with no length field;
    // the source-shaped Rust mapping keeps a borrowed byte slice at the seam.
    // Callers pass the explicit source defaults (1, 1, false, false).
    fn makeImageTexture(
        &mut self,
        width: u32,
        height: u32,
        mipLevelCount: u32,
        format: GPUTextureFormat,
        imageData: &[u8],
        blockWidth: u8,
        blockHeight: u8,
        srgb: bool,
        generateRemainingMips: bool,
    ) -> rcp<Texture>;

    // Rust has no default arguments. This explicit call preserves the source
    // makeImageTexture(..., 1, 1, false, false) defaults.
    fn makeImageTextureDefault(
        &mut self,
        width: u32,
        height: u32,
        mipLevelCount: u32,
        format: GPUTextureFormat,
        imageData: &[u8],
    ) -> rcp<Texture> {
        self.makeImageTexture(
            width,
            height,
            mipLevelCount,
            format,
            imageData,
            1,
            1,
            false,
            false,
        )
    }

    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    // #ifdef RIVE_CANVAS
    // virtual rcp<RenderCanvas> makeRenderCanvas(uint32_t width,
    //                                             uint32_t height)
    // {
    //     return nullptr;
    // }
    //
    // rcp<RenderCanvas> retains the source intrusive nullable owner.
    fn makeRenderCanvas(&mut self, width: u32, height: u32) -> rcp<RenderCanvas> {
        let _ = (width, height);
        rcp::new()
    }

    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    // virtual std::unique_ptr<rive::ore::Context> makeOreContext() = 0;
    fn makeOreContext(&mut self) -> Option<Box<OreContext>>;

    // virtual void resizeFlushUniformBuffer(size_t sizeInBytes) = 0;
    fn resizeFlushUniformBuffer(&mut self, sizeInBytes: usize);

    // virtual void resizePathBuffer(size_t sizeInBytes,
    //                               gpu::StorageBufferStructure) = 0;
    fn resizePathBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure);

    // virtual void resizePaintBuffer(size_t sizeInBytes,
    //                                gpu::StorageBufferStructure) = 0;
    fn resizePaintBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure);

    // virtual void resizePaintAuxBuffer(size_t sizeInBytes,
    //                                    gpu::StorageBufferStructure) = 0;
    fn resizePaintAuxBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure);

    // virtual void resizeContourBuffer(size_t sizeInBytes,
    //                                  gpu::StorageBufferStructure) = 0;
    fn resizeContourBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure);

    // virtual void resizeGradSpanBuffer(size_t sizeInBytes) = 0;
    fn resizeGradSpanBuffer(&mut self, sizeInBytes: usize);

    // virtual void resizeTessVertexSpanBuffer(size_t sizeInBytes) = 0;
    fn resizeTessVertexSpanBuffer(&mut self, sizeInBytes: usize);

    // virtual void resizeTriangleVertexBuffer(size_t sizeInBytes) = 0;
    fn resizeTriangleVertexBuffer(&mut self, sizeInBytes: usize);

    // virtual void resizeImageDrawInstanceBuffer(size_t sizeInBytes) = 0;
    fn resizeImageDrawInstanceBuffer(&mut self, sizeInBytes: usize);

    // virtual void preBeginFrame(RenderContext*) {}
    unsafe fn preBeginFrame(&mut self, renderContext: *mut RenderContext) {
        let _ = renderContext;
    }

    // virtual bool wantsManualRenderPassResolve(
    //     gpu::InterlockMode,
    //     const RenderTarget*,
    //     const IAABB& renderTargetUpdateBounds,
    //     uint32_t virtualTileWidth,
    //     uint32_t virtualTileHeight,
    //     gpu::DrawContents combinedDrawContents) const
    // {
    //     return false;
    // }
    unsafe fn wantsManualRenderPassResolve(
        &self,
        interlockMode: InterlockMode,
        renderTarget: *const RenderTarget,
        renderTargetUpdateBounds: &IAABB,
        virtualTileWidth: u32,
        virtualTileHeight: u32,
        combinedDrawContents: DrawContents,
    ) -> bool {
        let _ = (
            interlockMode,
            renderTarget,
            renderTargetUpdateBounds,
            virtualTileWidth,
            virtualTileHeight,
            combinedDrawContents,
        );
        false
    }

    // virtual void prepareToFlush(uint64_t nextFrameNumber,
    //                             uint64_t safeFrameNumber)
    // {}
    fn prepareToFlush(&mut self, nextFrameNumber: u64, safeFrameNumber: u64) {
        let _ = (nextFrameNumber, safeFrameNumber);
    }

    // virtual void* mapFlushUniformBuffer(size_t mapSizeInBytes) = 0;
    fn mapFlushUniformBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void* mapPathBuffer(size_t mapSizeInBytes) = 0;
    fn mapPathBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void* mapPaintBuffer(size_t mapSizeInBytes) = 0;
    fn mapPaintBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void* mapPaintAuxBuffer(size_t mapSizeInBytes) = 0;
    fn mapPaintAuxBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void* mapContourBuffer(size_t mapSizeInBytes) = 0;
    fn mapContourBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void* mapGradSpanBuffer(size_t mapSizeInBytes) = 0;
    fn mapGradSpanBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void* mapTessVertexSpanBuffer(size_t mapSizeInBytes) = 0;
    fn mapTessVertexSpanBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void* mapTriangleVertexBuffer(size_t mapSizeInBytes) = 0;
    fn mapTriangleVertexBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void* mapImageDrawInstanceBuffer(size_t mapSizeInBytes) = 0;
    fn mapImageDrawInstanceBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // virtual void unmapFlushUniformBuffer(size_t mapSizeInBytes) = 0;
    fn unmapFlushUniformBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void unmapPathBuffer(size_t mapSizeInBytes) = 0;
    fn unmapPathBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void unmapPaintBuffer(size_t mapSizeInBytes) = 0;
    fn unmapPaintBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void unmapPaintAuxBuffer(size_t mapSizeInBytes) = 0;
    fn unmapPaintAuxBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void unmapContourBuffer(size_t mapSizeInBytes) = 0;
    fn unmapContourBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void unmapGradSpanBuffer(size_t mapSizeInBytes) = 0;
    fn unmapGradSpanBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void unmapTessVertexSpanBuffer(size_t mapSizeInBytes) = 0;
    fn unmapTessVertexSpanBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void unmapTriangleVertexBuffer(size_t mapSizeInBytes) = 0;
    fn unmapTriangleVertexBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void unmapImageDrawInstanceBuffer(size_t mapSizeInBytes) = 0;
    fn unmapImageDrawInstanceBuffer(&mut self, mapSizeInBytes: usize);

    // virtual void resizeGradientTexture(uint32_t width, uint32_t height) = 0;
    fn resizeGradientTexture(&mut self, width: u32, height: u32);

    // virtual void resizeTessellationTexture(uint32_t width, uint32_t height) = 0;
    fn resizeTessellationTexture(&mut self, width: u32, height: u32);

    // virtual void resizeFeatherAtlasTexture(uint32_t width, uint32_t height)
    // {
    //     // Override this method to support atlas feathering.
    //     assert(width == 0 && height == 0);
    // }
    fn resizeFeatherAtlasTexture(&mut self, width: u32, height: u32) {
        // Override this method to support atlas feathering.
        debug_assert!(width == 0 && height == 0);
    }

    // virtual void resizeTransientPLSBacking(uint32_t width,
    //                                        uint32_t height,
    //                                        uint32_t planeCount)
    // {}
    fn resizeTransientPLSBacking(&mut self, width: u32, height: u32, planeCount: u32) {
        let _ = (width, height, planeCount);
    }

    // virtual void resizeAtomicCoverageBacking(uint32_t width, uint32_t height) {}
    fn resizeAtomicCoverageBacking(&mut self, width: u32, height: u32) {
        let _ = (width, height);
    }

    // virtual void resizeCoverageBuffer(size_t sizeInBytes)
    // {
    //     // Override this method to support the experimental clockwiseAtomic
    //     // mode.
    //     assert(sizeInBytes == 0);
    // }
    fn resizeCoverageBuffer(&mut self, sizeInBytes: usize) {
        // Override this method to support the experimental clockwiseAtomic mode.
        debug_assert!(sizeInBytes == 0);
    }

    // virtual void flush(const gpu::FlushDescriptor&) = 0;
    unsafe fn flush(&mut self, flushDescriptor: &FlushDescriptor);

    // virtual void postFlush(const RenderContext::FlushResources&) {}
    //
    // Rust hoists the source nested FlushResources record to the mapped
    // sibling declaration in render_context_hpp.rs.
    unsafe fn postFlush(&mut self, flushResources: &FlushResources) {
        let _ = flushResources;
    }

    // virtual void* makeCommandBuffer() { return nullptr; }
    fn makeCommandBuffer(&mut self) -> *mut c_void {
        core::ptr::null_mut()
    }

    // virtual void commitCommandBuffer(void* commandBuffer) {}
    unsafe fn commitCommandBuffer(&mut self, commandBuffer: *mut c_void) {
        let _ = commandBuffer;
    }

    // virtual double secondsNow() const = 0;
    fn secondsNow(&self) -> f64;
}

// #if defined(ORE_BACKEND_GL) && defined(RIVE_CANVAS)
// namespace rive
// {
// class RiveRenderImage;
// // Returns a Y-flipped companion of a GL canvas texture, or nullptr on
// // non-GL backends. Hides the RenderContextGLImpl downcast so callers
// // don't need GL headers.
// rcp<RiveRenderImage> getCanvasImportMirrorGL(gpu::RenderContext*,
//                                              gpu::Texture* sourceTex,
//                                              uint32_t width,
//                                              uint32_t height);
// } // namespace rive
// #endif
//
// The declaration-only free function intentionally remains source-commented;
// its implementation belongs to the GL backend owner. ORE_BACKEND_GL maps
// to the source-equivalent Cargo feature ore-gl.
#[cfg(feature = "ore-gl")]
pub trait CanvasImportMirrorGL {
    // rcp<RiveRenderImage> getCanvasImportMirrorGL(gpu::RenderContext*,
    //                                              gpu::Texture* sourceTex,
    //                                              uint32_t width,
    //                                              uint32_t height);
    unsafe fn getCanvasImportMirrorGL(
        renderContext: *mut RenderContext,
        sourceTex: *mut Texture,
        width: u32,
        height: u32,
    ) -> rcp<RiveRenderImage>;
}
