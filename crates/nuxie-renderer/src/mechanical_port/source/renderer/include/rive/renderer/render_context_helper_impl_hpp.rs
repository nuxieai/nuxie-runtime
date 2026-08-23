/*
 * Copyright 2023 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/render_context_helper_impl.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// The complete source is retained below in declaration order. The Rust
// declarations following it keep the inherited owner, every virtual
// declaration, inline clock calculation, protected ring accessors, factory
// contracts, field order, and member default visible to later translations.

// /*
//  * Copyright 2023 Rive
//  */
//
// #pragma once
//
// #include "rive/renderer/render_context_impl.hpp"
// #include "rive/renderer/buffer_ring.hpp"
// #include <chrono>
//
// namespace rive::gpu
// {
// // RenderContextImpl that uses BufferRing to manage GPU resources.
// class RenderContextHelperImpl : public RenderContextImpl
// {
// public:
//     void resizeFlushUniformBuffer(size_t sizeInBytes) override;
//     void resizePathBuffer(size_t sizeInBytes,
//                           gpu::StorageBufferStructure) override;
//     void resizePaintBuffer(size_t sizeInBytes,
//                            gpu::StorageBufferStructure) override;
//     void resizePaintAuxBuffer(size_t sizeInBytes,
//                               gpu::StorageBufferStructure) override;
//     void resizeContourBuffer(size_t sizeInBytes,
//                              gpu::StorageBufferStructure) override;
//     void resizeGradSpanBuffer(size_t sizeInBytes) override;
//     void resizeTessVertexSpanBuffer(size_t sizeInBytes) override;
//     void resizeTriangleVertexBuffer(size_t sizeInBytes) override;
//     void resizeImageDrawInstanceBuffer(size_t sizeInBytes) override;
//
//     void* mapFlushUniformBuffer(size_t mapSizeInBytes) override;
//     void* mapPathBuffer(size_t mapSizeInBytes) override;
//     void* mapPaintBuffer(size_t mapSizeInBytes) override;
//     void* mapPaintAuxBuffer(size_t mapSizeInBytes) override;
//     void* mapContourBuffer(size_t mapSizeInBytes) override;
//     void* mapGradSpanBuffer(size_t mapSizeInBytes) override;
//     void* mapTessVertexSpanBuffer(size_t mapSizeInBytes) override;
//     void* mapTriangleVertexBuffer(size_t mapSizeInBytes) override;
//     void* mapImageDrawInstanceBuffer(size_t mapSizeInBytes) override;
//
//     void unmapFlushUniformBuffer(size_t mapSizeInBytes) override;
//     void unmapPathBuffer(size_t mapSizeInBytes) override;
//     void unmapPaintBuffer(size_t mapSizeInBytes) override;
//     void unmapPaintAuxBuffer(size_t mapSizeInBytes) override;
//     void unmapContourBuffer(size_t mapSizeInBytes) override;
//     void unmapGradSpanBuffer(size_t mapSizeInBytes) override;
//     void unmapTessVertexSpanBuffer(size_t mapSizeInBytes) override;
//     void unmapTriangleVertexBuffer(size_t mapSizeInBytes) override;
//     void unmapImageDrawInstanceBuffer(size_t mapSizeInBytes) override;
//
//     double secondsNow() const override
//     {
//         auto elapsed = std::chrono::steady_clock::now() - m_localEpoch;
//         return std::chrono::duration<double>(elapsed).count();
//     }
//
// protected:
//     BufferRing* flushUniformBufferRing() { return m_flushUniformBuffer.get(); }
//     BufferRing* pathBufferRing() { return m_pathBuffer.get(); }
//     BufferRing* paintBufferRing() { return m_paintBuffer.get(); }
//     BufferRing* paintAuxBufferRing() { return m_paintAuxBuffer.get(); }
//     BufferRing* contourBufferRing() { return m_contourBuffer.get(); }
//     BufferRing* gradSpanBufferRing() { return m_gradSpanBuffer.get(); }
//     BufferRing* tessSpanBufferRing() { return m_tessSpanBuffer.get(); }
//     BufferRing* triangleBufferRing() { return m_triangleBuffer.get(); }
//     BufferRing* imageDrawInstanceBufferRing()
//     {
//         return m_imageDrawInstanceBuffer.get();
//     }
//
//     virtual std::unique_ptr<BufferRing> makeUniformBufferRing(
//         size_t capacityInBytes) = 0;
//     virtual std::unique_ptr<BufferRing> makeStorageBufferRing(
//         size_t capacityInBytes,
//         gpu::StorageBufferStructure) = 0;
//     virtual std::unique_ptr<BufferRing> makeVertexBufferRing(
//         size_t capacityInBytes) = 0;
//
// private:
//     std::unique_ptr<BufferRing> m_flushUniformBuffer;
//     std::unique_ptr<BufferRing> m_pathBuffer;
//     std::unique_ptr<BufferRing> m_paintBuffer;
//     std::unique_ptr<BufferRing> m_paintAuxBuffer;
//     std::unique_ptr<BufferRing> m_contourBuffer;
//     std::unique_ptr<BufferRing> m_gradSpanBuffer;
//     std::unique_ptr<BufferRing> m_tessSpanBuffer;
//     std::unique_ptr<BufferRing> m_triangleBuffer;
//     std::unique_ptr<BufferRing> m_imageDrawInstanceBuffer;
//     std::chrono::steady_clock::time_point m_localEpoch =
//         std::chrono::steady_clock::now();
// };
// } // namespace rive::gpu

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_void;
use core::mem::ManuallyDrop;
use std::time::Instant;

// Mapped source dependency: renderer/include/rive/renderer/render_context_impl.hpp.
// The complete RenderContextImpl base owner is supplied by that source-shaped
// header; this translation preserves it as the first field rather than
// substituting a local partial base.
use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat;
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawContents, FlushDescriptor, InterlockMode, IAABB,
};
#[cfg(feature = "native-ore-metal-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
#[cfg(feature = "native-ore-metal-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::OreContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, RenderContext,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::{
    RenderContextImpl, RenderContextImplContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;

// Mapped source dependency: renderer/include/rive/renderer/buffer_ring.hpp.
use crate::mechanical_port::source::renderer::include::rive::renderer::buffer_ring_hpp::{
    BufferRing, BufferRingContract,
};

// Mapped source dependency: renderer/include/rive/renderer/gpu.hpp.
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::StorageBufferStructure;

// class RenderContextHelperImpl : public RenderContextImpl
//
// Rust drops fields in declaration order. The fields below therefore follow
// C++ destruction order: epoch, rings from last to first, then the base.
#[repr(C)]
pub struct RenderContextHelperImpl {
    // The C++ base subobject is address zero; static_impl_cast relies on this.
    // ManuallyDrop preserves C++ reverse-member destruction while retaining
    // the authored base-first memory topology.
    pub(crate) base: ManuallyDrop<RenderContextImpl>,
    pub(crate) m_flushUniformBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    pub(crate) m_pathBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    pub(crate) m_paintBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    pub(crate) m_paintAuxBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    pub(crate) m_contourBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    pub(crate) m_gradSpanBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    pub(crate) m_tessSpanBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    pub(crate) m_triangleBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    pub(crate) m_imageDrawInstanceBuffer: ManuallyDrop<Option<Box<dyn BufferRingContract>>>,
    m_localEpoch: ManuallyDrop<Instant>,
}

impl RenderContextHelperImpl {
    pub fn new(base: RenderContextImpl) -> Self {
        Self {
            base: ManuallyDrop::new(base),
            m_flushUniformBuffer: ManuallyDrop::new(None),
            m_pathBuffer: ManuallyDrop::new(None),
            m_paintBuffer: ManuallyDrop::new(None),
            m_paintAuxBuffer: ManuallyDrop::new(None),
            m_contourBuffer: ManuallyDrop::new(None),
            m_gradSpanBuffer: ManuallyDrop::new(None),
            m_tessSpanBuffer: ManuallyDrop::new(None),
            m_triangleBuffer: ManuallyDrop::new(None),
            m_imageDrawInstanceBuffer: ManuallyDrop::new(None),
            m_localEpoch: ManuallyDrop::new(Instant::now()),
        }
    }
}

impl Drop for RenderContextHelperImpl {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_localEpoch);
            ManuallyDrop::drop(&mut self.m_imageDrawInstanceBuffer);
            ManuallyDrop::drop(&mut self.m_triangleBuffer);
            ManuallyDrop::drop(&mut self.m_tessSpanBuffer);
            ManuallyDrop::drop(&mut self.m_gradSpanBuffer);
            ManuallyDrop::drop(&mut self.m_contourBuffer);
            ManuallyDrop::drop(&mut self.m_paintAuxBuffer);
            ManuallyDrop::drop(&mut self.m_paintBuffer);
            ManuallyDrop::drop(&mut self.m_pathBuffer);
            ManuallyDrop::drop(&mut self.m_flushUniformBuffer);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

// The source class overrides the RenderContextImpl virtual surface in the
// paired out-of-line implementation. Rust expresses those declarations as a
// source-shaped contract; the paired implementation owner supplies each body
// without changing declaration order or callback argument shape.
pub trait RenderContextHelperImplAccess {
    fn renderContextHelperImpl(&self) -> &RenderContextHelperImpl;
    fn renderContextHelperImplMut(&mut self) -> &mut RenderContextHelperImpl;
}

// These three pure virtuals remain backend-owned. The helper never invents a
// buffer-ring implementation; a concrete backend supplies this contract.
pub trait RenderContextHelperBufferFactoryContract {
    fn makeUniformBufferRing(&mut self, capacityInBytes: usize) -> Box<dyn BufferRingContract>;
    fn makeStorageBufferRing(
        &mut self,
        capacityInBytes: usize,
        bufferStructure: StorageBufferStructure,
    ) -> Box<dyn BufferRingContract>;
    fn makeVertexBufferRing(&mut self, capacityInBytes: usize) -> Box<dyn BufferRingContract>;
}

pub trait RenderContextHelperImplContract:
    RenderContextHelperImplAccess + RenderContextHelperBufferFactoryContract
{
    // void resizeFlushUniformBuffer(size_t sizeInBytes) override;
    fn resizeFlushUniformBuffer(&mut self, sizeInBytes: usize);
    // void resizePathBuffer(size_t sizeInBytes,
    //                       gpu::StorageBufferStructure) override;
    fn resizePathBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure);
    // void resizePaintBuffer(size_t sizeInBytes,
    //                        gpu::StorageBufferStructure) override;
    fn resizePaintBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure);
    // void resizePaintAuxBuffer(size_t sizeInBytes,
    //                           gpu::StorageBufferStructure) override;
    fn resizePaintAuxBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure);
    // void resizeContourBuffer(size_t sizeInBytes,
    //                          gpu::StorageBufferStructure) override;
    fn resizeContourBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure);
    // void resizeGradSpanBuffer(size_t sizeInBytes) override;
    fn resizeGradSpanBuffer(&mut self, sizeInBytes: usize);
    // void resizeTessVertexSpanBuffer(size_t sizeInBytes) override;
    fn resizeTessVertexSpanBuffer(&mut self, sizeInBytes: usize);
    // void resizeTriangleVertexBuffer(size_t sizeInBytes) override;
    fn resizeTriangleVertexBuffer(&mut self, sizeInBytes: usize);
    // void resizeImageDrawInstanceBuffer(size_t sizeInBytes) override;
    fn resizeImageDrawInstanceBuffer(&mut self, sizeInBytes: usize);

    // void* mapFlushUniformBuffer(size_t mapSizeInBytes) override;
    fn mapFlushUniformBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;
    // void* mapPathBuffer(size_t mapSizeInBytes) override;
    fn mapPathBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;
    // void* mapPaintBuffer(size_t mapSizeInBytes) override;
    fn mapPaintBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;
    // void* mapPaintAuxBuffer(size_t mapSizeInBytes) override;
    fn mapPaintAuxBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;
    // void* mapContourBuffer(size_t mapSizeInBytes) override;
    fn mapContourBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;
    // void* mapGradSpanBuffer(size_t mapSizeInBytes) override;
    fn mapGradSpanBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;
    // void* mapTessVertexSpanBuffer(size_t mapSizeInBytes) override;
    fn mapTessVertexSpanBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;
    // void* mapTriangleVertexBuffer(size_t mapSizeInBytes) override;
    fn mapTriangleVertexBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;
    // void* mapImageDrawInstanceBuffer(size_t mapSizeInBytes) override;
    fn mapImageDrawInstanceBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void;

    // void unmapFlushUniformBuffer(size_t mapSizeInBytes) override;
    fn unmapFlushUniformBuffer(&mut self, mapSizeInBytes: usize);
    // void unmapPathBuffer(size_t mapSizeInBytes) override;
    fn unmapPathBuffer(&mut self, mapSizeInBytes: usize);
    // void unmapPaintBuffer(size_t mapSizeInBytes) override;
    fn unmapPaintBuffer(&mut self, mapSizeInBytes: usize);
    // void unmapPaintAuxBuffer(size_t mapSizeInBytes) override;
    fn unmapPaintAuxBuffer(&mut self, mapSizeInBytes: usize);
    // void unmapContourBuffer(size_t mapSizeInBytes) override;
    fn unmapContourBuffer(&mut self, mapSizeInBytes: usize);
    // void unmapGradSpanBuffer(size_t mapSizeInBytes) override;
    fn unmapGradSpanBuffer(&mut self, mapSizeInBytes: usize);
    // void unmapTessVertexSpanBuffer(size_t mapSizeInBytes) override;
    fn unmapTessVertexSpanBuffer(&mut self, mapSizeInBytes: usize);
    // void unmapTriangleVertexBuffer(size_t mapSizeInBytes) override;
    fn unmapTriangleVertexBuffer(&mut self, mapSizeInBytes: usize);
    // void unmapImageDrawInstanceBuffer(size_t mapSizeInBytes) override;
    fn unmapImageDrawInstanceBuffer(&mut self, mapSizeInBytes: usize);

    // double secondsNow() const override;
    fn secondsNow(&self) -> f64 {
        self.renderContextHelperImpl().secondsNow()
    }
}

// Backend-only remainder of RenderContextImpl. Combined with the blanket
// implementation below, a concrete helper backend implements only the pure
// virtuals it actually owns; all buffer resize/map/unmap overrides are supplied
// once by RenderContextHelperImplContract.
pub trait RenderContextHelperBackendContract:
    RenderContextHelperImplAccess + RenderContextHelperBufferFactoryContract
{
    fn makeRenderBuffer(
        &mut self,
        ty: RenderBufferType,
        flags: RenderBufferFlags,
        bytes: usize,
    ) -> rcp<RenderBuffer>;
    fn platformDecodeImageTexture(&mut self, encoded: &[u8]) -> rcp<Texture> {
        let _ = encoded;
        rcp::new()
    }
    fn makeImageTexture(
        &mut self,
        width: u32,
        height: u32,
        levels: u32,
        format: GPUTextureFormat,
        data: &[u8],
        block_width: u8,
        block_height: u8,
        srgb: bool,
        generate_mips: bool,
    ) -> rcp<Texture>;
    #[cfg(feature = "native-ore-metal-experimental")]
    fn makeRenderCanvas(&mut self, width: u32, height: u32) -> rcp<RenderCanvas> {
        let _ = (width, height);
        rcp::new()
    }
    #[cfg(feature = "native-ore-metal-experimental")]
    fn makeOreContext(&mut self) -> Option<Box<OreContext>>;
    unsafe fn preBeginFrame(&mut self, context: *mut RenderContext) {
        let _ = context;
    }
    unsafe fn wantsManualRenderPassResolve(
        &self,
        mode: InterlockMode,
        target: *const RenderTarget,
        bounds: &IAABB,
        tile_width: u32,
        tile_height: u32,
        contents: DrawContents,
    ) -> bool {
        let _ = (mode, target, bounds, tile_width, tile_height, contents);
        false
    }
    fn prepareToFlush(&mut self, next: u64, safe: u64) {
        let _ = (next, safe);
    }
    fn resizeGradientTexture(&mut self, width: u32, height: u32);
    fn resizeTessellationTexture(&mut self, width: u32, height: u32);
    fn resizeFeatherAtlasTexture(&mut self, width: u32, height: u32) {
        debug_assert!(width == 0 && height == 0);
    }
    fn resizeTransientPLSBacking(&mut self, width: u32, height: u32, planes: u32) {
        let _ = (width, height, planes);
    }
    fn resizeAtomicCoverageBacking(&mut self, width: u32, height: u32) {
        let _ = (width, height);
    }
    fn resizeCoverageBuffer(&mut self, bytes: usize) {
        debug_assert_eq!(bytes, 0);
    }
    unsafe fn flush(&mut self, descriptor: &FlushDescriptor);
    unsafe fn postFlush(&mut self, resources: &FlushResources) {
        let _ = resources;
    }
    fn makeCommandBuffer(&mut self) -> *mut c_void {
        core::ptr::null_mut()
    }
    unsafe fn commitCommandBuffer(&mut self, command: *mut c_void) {
        let _ = command;
    }
}

impl<T> RenderContextImplContract for T
where
    T: RenderContextHelperBackendContract,
{
    fn renderContextImpl(&self) -> &RenderContextImpl {
        &*self.renderContextHelperImpl().base
    }
    fn renderContextImplMut(&mut self) -> &mut RenderContextImpl {
        &mut *self.renderContextHelperImplMut().base
    }
    fn makeRenderBuffer(
        &mut self,
        t: RenderBufferType,
        f: RenderBufferFlags,
        b: usize,
    ) -> rcp<RenderBuffer> {
        RenderContextHelperBackendContract::makeRenderBuffer(self, t, f, b)
    }
    fn platformDecodeImageTexture(&mut self, e: &[u8]) -> rcp<Texture> {
        RenderContextHelperBackendContract::platformDecodeImageTexture(self, e)
    }
    fn makeImageTexture(
        &mut self,
        w: u32,
        h: u32,
        l: u32,
        f: GPUTextureFormat,
        d: &[u8],
        bw: u8,
        bh: u8,
        s: bool,
        g: bool,
    ) -> rcp<Texture> {
        RenderContextHelperBackendContract::makeImageTexture(self, w, h, l, f, d, bw, bh, s, g)
    }
    #[cfg(feature = "native-ore-metal-experimental")]
    fn makeRenderCanvas(&mut self, w: u32, h: u32) -> rcp<RenderCanvas> {
        RenderContextHelperBackendContract::makeRenderCanvas(self, w, h)
    }
    #[cfg(feature = "native-ore-metal-experimental")]
    fn makeOreContext(&mut self) -> Option<Box<OreContext>> {
        RenderContextHelperBackendContract::makeOreContext(self)
    }
    fn resizeFlushUniformBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::resizeFlushUniformBuffer(self, b)
    }
    fn resizePathBuffer(&mut self, b: usize, s: StorageBufferStructure) {
        RenderContextHelperImplContract::resizePathBuffer(self, b, s)
    }
    fn resizePaintBuffer(&mut self, b: usize, s: StorageBufferStructure) {
        RenderContextHelperImplContract::resizePaintBuffer(self, b, s)
    }
    fn resizePaintAuxBuffer(&mut self, b: usize, s: StorageBufferStructure) {
        RenderContextHelperImplContract::resizePaintAuxBuffer(self, b, s)
    }
    fn resizeContourBuffer(&mut self, b: usize, s: StorageBufferStructure) {
        RenderContextHelperImplContract::resizeContourBuffer(self, b, s)
    }
    fn resizeGradSpanBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::resizeGradSpanBuffer(self, b)
    }
    fn resizeTessVertexSpanBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::resizeTessVertexSpanBuffer(self, b)
    }
    fn resizeTriangleVertexBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::resizeTriangleVertexBuffer(self, b)
    }
    fn resizeImageDrawInstanceBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::resizeImageDrawInstanceBuffer(self, b)
    }
    unsafe fn preBeginFrame(&mut self, c: *mut RenderContext) {
        unsafe { RenderContextHelperBackendContract::preBeginFrame(self, c) }
    }
    unsafe fn wantsManualRenderPassResolve(
        &self,
        m: InterlockMode,
        t: *const RenderTarget,
        b: &IAABB,
        w: u32,
        h: u32,
        c: DrawContents,
    ) -> bool {
        unsafe {
            RenderContextHelperBackendContract::wantsManualRenderPassResolve(self, m, t, b, w, h, c)
        }
    }
    fn prepareToFlush(&mut self, n: u64, s: u64) {
        RenderContextHelperBackendContract::prepareToFlush(self, n, s)
    }
    fn mapFlushUniformBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapFlushUniformBuffer(self, b)
    }
    fn mapPathBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapPathBuffer(self, b)
    }
    fn mapPaintBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapPaintBuffer(self, b)
    }
    fn mapPaintAuxBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapPaintAuxBuffer(self, b)
    }
    fn mapContourBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapContourBuffer(self, b)
    }
    fn mapGradSpanBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapGradSpanBuffer(self, b)
    }
    fn mapTessVertexSpanBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapTessVertexSpanBuffer(self, b)
    }
    fn mapTriangleVertexBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapTriangleVertexBuffer(self, b)
    }
    fn mapImageDrawInstanceBuffer(&mut self, b: usize) -> *mut c_void {
        RenderContextHelperImplContract::mapImageDrawInstanceBuffer(self, b)
    }
    fn unmapFlushUniformBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapFlushUniformBuffer(self, b)
    }
    fn unmapPathBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapPathBuffer(self, b)
    }
    fn unmapPaintBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapPaintBuffer(self, b)
    }
    fn unmapPaintAuxBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapPaintAuxBuffer(self, b)
    }
    fn unmapContourBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapContourBuffer(self, b)
    }
    fn unmapGradSpanBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapGradSpanBuffer(self, b)
    }
    fn unmapTessVertexSpanBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapTessVertexSpanBuffer(self, b)
    }
    fn unmapTriangleVertexBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapTriangleVertexBuffer(self, b)
    }
    fn unmapImageDrawInstanceBuffer(&mut self, b: usize) {
        RenderContextHelperImplContract::unmapImageDrawInstanceBuffer(self, b)
    }
    fn resizeGradientTexture(&mut self, w: u32, h: u32) {
        RenderContextHelperBackendContract::resizeGradientTexture(self, w, h)
    }
    fn resizeTessellationTexture(&mut self, w: u32, h: u32) {
        RenderContextHelperBackendContract::resizeTessellationTexture(self, w, h)
    }
    fn resizeFeatherAtlasTexture(&mut self, w: u32, h: u32) {
        RenderContextHelperBackendContract::resizeFeatherAtlasTexture(self, w, h)
    }
    fn resizeTransientPLSBacking(&mut self, w: u32, h: u32, p: u32) {
        RenderContextHelperBackendContract::resizeTransientPLSBacking(self, w, h, p)
    }
    fn resizeAtomicCoverageBacking(&mut self, w: u32, h: u32) {
        RenderContextHelperBackendContract::resizeAtomicCoverageBacking(self, w, h)
    }
    fn resizeCoverageBuffer(&mut self, b: usize) {
        RenderContextHelperBackendContract::resizeCoverageBuffer(self, b)
    }
    unsafe fn flush(&mut self, d: &FlushDescriptor) {
        unsafe { RenderContextHelperBackendContract::flush(self, d) }
    }
    unsafe fn postFlush(&mut self, r: &FlushResources) {
        unsafe { RenderContextHelperBackendContract::postFlush(self, r) }
    }
    fn makeCommandBuffer(&mut self) -> *mut c_void {
        RenderContextHelperBackendContract::makeCommandBuffer(self)
    }
    unsafe fn commitCommandBuffer(&mut self, c: *mut c_void) {
        unsafe { RenderContextHelperBackendContract::commitCommandBuffer(self, c) }
    }
    fn secondsNow(&self) -> f64 {
        RenderContextHelperImplContract::secondsNow(self)
    }
}

impl RenderContextHelperImpl {
    // double secondsNow() const override
    // {
    //     auto elapsed = std::chrono::steady_clock::now() - m_localEpoch;
    //     return std::chrono::duration<double>(elapsed).count();
    // }
    pub fn secondsNow(&self) -> f64 {
        let elapsed = Instant::now().duration_since(*self.m_localEpoch);
        elapsed.as_secs_f64()
    }

    // BufferRing* flushUniformBufferRing() { return m_flushUniformBuffer.get(); }
    pub(crate) fn flushUniformBufferRing(&mut self) -> *mut BufferRing {
        self.m_flushUniformBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }

    // BufferRing* pathBufferRing() { return m_pathBuffer.get(); }
    pub(crate) fn pathBufferRing(&mut self) -> *mut BufferRing {
        self.m_pathBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }

    // BufferRing* paintBufferRing() { return m_paintBuffer.get(); }
    pub(crate) fn paintBufferRing(&mut self) -> *mut BufferRing {
        self.m_paintBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }

    // BufferRing* paintAuxBufferRing() { return m_paintAuxBuffer.get(); }
    pub(crate) fn paintAuxBufferRing(&mut self) -> *mut BufferRing {
        self.m_paintAuxBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }

    // BufferRing* contourBufferRing() { return m_contourBuffer.get(); }
    pub(crate) fn contourBufferRing(&mut self) -> *mut BufferRing {
        self.m_contourBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }

    // BufferRing* gradSpanBufferRing() { return m_gradSpanBuffer.get(); }
    pub(crate) fn gradSpanBufferRing(&mut self) -> *mut BufferRing {
        self.m_gradSpanBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }

    // BufferRing* tessSpanBufferRing() { return m_tessSpanBuffer.get(); }
    pub(crate) fn tessSpanBufferRing(&mut self) -> *mut BufferRing {
        self.m_tessSpanBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }

    // BufferRing* triangleBufferRing() { return m_triangleBuffer.get(); }
    pub(crate) fn triangleBufferRing(&mut self) -> *mut BufferRing {
        self.m_triangleBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }

    // BufferRing* imageDrawInstanceBufferRing()
    // {
    //     return m_imageDrawInstanceBuffer.get();
    // }
    pub(crate) fn imageDrawInstanceBufferRing(&mut self) -> *mut BufferRing {
        self.m_imageDrawInstanceBuffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |ring| {
                ring.bufferRingMut() as *mut BufferRing
            })
    }
}
