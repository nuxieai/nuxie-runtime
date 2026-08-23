/*
 * Copyright 2022 Rive
 */

// #include "rive/renderer/render_context_helper_impl.hpp"

// #include "rive/renderer/rive_render_image.hpp"
// #include "shaders/constants.glsl"

// #ifdef RIVE_DECODERS
// #include "rive/decoders/bitmap_decoder.hpp"
// #endif

// Mechanical translation of the complete pinned source implementation
// renderer/src/render_context_helper_impl.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// The RIVE_DECODERS include branch above is retained in source order. This
// implementation does not name a decoder symbol; the mapped image-decoder
// owner supplies that dependency when the feature configuration is wired.

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_void;

// Mapped source dependency: renderer/include/rive/renderer/render_context_helper_impl.hpp.
use crate::mechanical_port::source::renderer::include::rive::renderer::
    render_context_helper_impl_hpp::{
        RenderContextHelperBufferFactoryContract, RenderContextHelperImplAccess,
        RenderContextHelperImplContract,
    };
use crate::mechanical_port::source::renderer::include::rive::renderer::buffer_ring_hpp::BufferRingContract;

// Mapped source dependency: renderer/include/rive/renderer/gpu.hpp.
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::StorageBufferStructure;

// Mapped source dependency: renderer/include/rive/renderer/rive_render_image.hpp.
// The pinned implementation includes this header for the complete renderer
// translation unit, but does not name a declaration from it.

// Mapped source dependency: renderer/src/shaders/constants.glsl.
// The shader constants are likewise included by the source translation unit
// without a direct definition use in this implementation.

// The pinned C++ BufferRing calls below are one-argument virtual dispatches.
// The mapped Rust BufferRing declaration currently exposes its pure-virtual
// callback seam through hook arguments; this source-shaped owner retains the
// C++ call shape until the compiler queue resolves that dependency seam.

// namespace rive::gpu
// {

impl<T> RenderContextHelperImplContract for T
where
    T: RenderContextHelperImplAccess + RenderContextHelperBufferFactoryContract,
{
    // void RenderContextHelperImpl::resizeFlushUniformBuffer(size_t sizeInBytes)
    fn resizeFlushUniformBuffer(&mut self, sizeInBytes: usize) {
        // m_flushUniformBuffer = makeUniformBufferRing(sizeInBytes);
        let ring =
            RenderContextHelperBufferFactoryContract::makeUniformBufferRing(self, sizeInBytes);
        *self.renderContextHelperImplMut().m_flushUniformBuffer = ring;
    }

    // void RenderContextHelperImpl::resizePathBuffer(
    //     size_t sizeInBytes,
    //     gpu::StorageBufferStructure bufferStructure)
    fn resizePathBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure) {
        // m_pathBuffer = makeStorageBufferRing(sizeInBytes, bufferStructure);
        let ring = RenderContextHelperBufferFactoryContract::makeStorageBufferRing(
            self,
            sizeInBytes,
            bufferStructure,
        );
        *self.renderContextHelperImplMut().m_pathBuffer = ring;
    }

    // void RenderContextHelperImpl::resizePaintBuffer(
    //     size_t sizeInBytes,
    //     gpu::StorageBufferStructure bufferStructure)
    fn resizePaintBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure) {
        // m_paintBuffer = makeStorageBufferRing(sizeInBytes, bufferStructure);
        let ring = RenderContextHelperBufferFactoryContract::makeStorageBufferRing(
            self,
            sizeInBytes,
            bufferStructure,
        );
        *self.renderContextHelperImplMut().m_paintBuffer = ring;
    }

    // void RenderContextHelperImpl::resizePaintAuxBuffer(
    //     size_t sizeInBytes,
    //     gpu::StorageBufferStructure bufferStructure)
    fn resizePaintAuxBuffer(
        &mut self,
        sizeInBytes: usize,
        bufferStructure: StorageBufferStructure,
    ) {
        // m_paintAuxBuffer = makeStorageBufferRing(sizeInBytes, bufferStructure);
        let ring = RenderContextHelperBufferFactoryContract::makeStorageBufferRing(
            self,
            sizeInBytes,
            bufferStructure,
        );
        *self.renderContextHelperImplMut().m_paintAuxBuffer = ring;
    }

    // void RenderContextHelperImpl::resizeContourBuffer(
    //     size_t sizeInBytes,
    //     gpu::StorageBufferStructure bufferStructure)
    fn resizeContourBuffer(&mut self, sizeInBytes: usize, bufferStructure: StorageBufferStructure) {
        // m_contourBuffer = makeStorageBufferRing(sizeInBytes, bufferStructure);
        let ring = RenderContextHelperBufferFactoryContract::makeStorageBufferRing(
            self,
            sizeInBytes,
            bufferStructure,
        );
        *self.renderContextHelperImplMut().m_contourBuffer = ring;
    }

    // void RenderContextHelperImpl::resizeGradSpanBuffer(size_t sizeInBytes)
    fn resizeGradSpanBuffer(&mut self, sizeInBytes: usize) {
        // m_gradSpanBuffer = makeVertexBufferRing(sizeInBytes);
        let ring =
            RenderContextHelperBufferFactoryContract::makeVertexBufferRing(self, sizeInBytes);
        *self.renderContextHelperImplMut().m_gradSpanBuffer = ring;
    }

    // void RenderContextHelperImpl::resizeTessVertexSpanBuffer(size_t sizeInBytes)
    fn resizeTessVertexSpanBuffer(&mut self, sizeInBytes: usize) {
        // m_tessSpanBuffer = makeVertexBufferRing(sizeInBytes);
        let ring =
            RenderContextHelperBufferFactoryContract::makeVertexBufferRing(self, sizeInBytes);
        *self.renderContextHelperImplMut().m_tessSpanBuffer = ring;
    }

    // void RenderContextHelperImpl::resizeTriangleVertexBuffer(size_t sizeInBytes)
    fn resizeTriangleVertexBuffer(&mut self, sizeInBytes: usize) {
        // m_triangleBuffer = makeVertexBufferRing(sizeInBytes);
        let ring =
            RenderContextHelperBufferFactoryContract::makeVertexBufferRing(self, sizeInBytes);
        *self.renderContextHelperImplMut().m_triangleBuffer = ring;
    }

    // void RenderContextHelperImpl::resizeImageDrawInstanceBuffer(size_t sizeInBytes)
    fn resizeImageDrawInstanceBuffer(&mut self, sizeInBytes: usize) {
        // m_imageDrawInstanceBuffer = makeVertexBufferRing(sizeInBytes);
        let ring =
            RenderContextHelperBufferFactoryContract::makeVertexBufferRing(self, sizeInBytes);
        *self.renderContextHelperImplMut().m_imageDrawInstanceBuffer = ring;
    }

    // void* RenderContextHelperImpl::mapFlushUniformBuffer(size_t mapSizeInBytes)
    fn mapFlushUniformBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_flushUniformBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_flushUniformBuffer
            .as_mut()
            .expect("flush uniform buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void* RenderContextHelperImpl::mapPathBuffer(size_t mapSizeInBytes)
    fn mapPathBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_pathBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_pathBuffer
            .as_mut()
            .expect("path buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void* RenderContextHelperImpl::mapPaintBuffer(size_t mapSizeInBytes)
    fn mapPaintBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_paintBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_paintBuffer
            .as_mut()
            .expect("paint buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void* RenderContextHelperImpl::mapPaintAuxBuffer(size_t mapSizeInBytes)
    fn mapPaintAuxBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_paintAuxBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_paintAuxBuffer
            .as_mut()
            .expect("paint auxiliary buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void* RenderContextHelperImpl::mapContourBuffer(size_t mapSizeInBytes)
    fn mapContourBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_contourBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_contourBuffer
            .as_mut()
            .expect("contour buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void* RenderContextHelperImpl::mapGradSpanBuffer(size_t mapSizeInBytes)
    fn mapGradSpanBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_gradSpanBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_gradSpanBuffer
            .as_mut()
            .expect("gradient span buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void* RenderContextHelperImpl::mapTessVertexSpanBuffer(size_t mapSizeInBytes)
    fn mapTessVertexSpanBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_tessSpanBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_tessSpanBuffer
            .as_mut()
            .expect("tessellation span buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void* RenderContextHelperImpl::mapTriangleVertexBuffer(size_t mapSizeInBytes)
    fn mapTriangleVertexBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_triangleBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_triangleBuffer
            .as_mut()
            .expect("triangle buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void* RenderContextHelperImpl::mapImageDrawInstanceBuffer(size_t mapSizeInBytes)
    fn mapImageDrawInstanceBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        // return m_imageDrawInstanceBuffer->mapBuffer(mapSizeInBytes);
        self.renderContextHelperImplMut()
            .m_imageDrawInstanceBuffer
            .as_mut()
            .expect("image draw instance buffer ring is required before mapping")
            .mapBuffer(mapSizeInBytes)
    }

    // void RenderContextHelperImpl::unmapFlushUniformBuffer(size_t mapSizeInBytes)
    fn unmapFlushUniformBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_flushUniformBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_flushUniformBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_flushUniformBuffer
            .as_mut()
            .expect("flush uniform buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }

    // void RenderContextHelperImpl::unmapPathBuffer(size_t mapSizeInBytes)
    fn unmapPathBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_pathBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_pathBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_pathBuffer
            .as_mut()
            .expect("path buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }

    // void RenderContextHelperImpl::unmapPaintBuffer(size_t mapSizeInBytes)
    fn unmapPaintBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_paintBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_paintBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_paintBuffer
            .as_mut()
            .expect("paint buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }

    // void RenderContextHelperImpl::unmapPaintAuxBuffer(size_t mapSizeInBytes)
    fn unmapPaintAuxBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_paintAuxBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_paintAuxBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_paintAuxBuffer
            .as_mut()
            .expect("paint auxiliary buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }

    // void RenderContextHelperImpl::unmapContourBuffer(size_t mapSizeInBytes)
    fn unmapContourBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_contourBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_contourBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_contourBuffer
            .as_mut()
            .expect("contour buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }

    // void RenderContextHelperImpl::unmapGradSpanBuffer(size_t mapSizeInBytes)
    fn unmapGradSpanBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_gradSpanBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_gradSpanBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_gradSpanBuffer
            .as_mut()
            .expect("gradient span buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }

    // void RenderContextHelperImpl::unmapTessVertexSpanBuffer(size_t mapSizeInBytes)
    fn unmapTessVertexSpanBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_tessSpanBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_tessSpanBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_tessSpanBuffer
            .as_mut()
            .expect("tessellation span buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }

    // void RenderContextHelperImpl::unmapTriangleVertexBuffer(size_t mapSizeInBytes)
    fn unmapTriangleVertexBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_triangleBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_triangleBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_triangleBuffer
            .as_mut()
            .expect("triangle buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }

    // void RenderContextHelperImpl::unmapImageDrawInstanceBuffer(
    //     size_t mapSizeInBytes)
    fn unmapImageDrawInstanceBuffer(&mut self, mapSizeInBytes: usize) {
        // assert(m_imageDrawInstanceBuffer->mapSizeInBytes() == mapSizeInBytes);
        // m_imageDrawInstanceBuffer->unmapAndSubmitBuffer();
        let ring = self
            .renderContextHelperImplMut()
            .m_imageDrawInstanceBuffer
            .as_mut()
            .expect("image draw instance buffer ring is required before unmapping");
        debug_assert!(ring.mapSizeInBytes() == mapSizeInBytes);
        ring.unmapAndSubmitBuffer();
    }
}

// } // namespace rive::gpu
