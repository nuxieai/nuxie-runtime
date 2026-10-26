/*
 * Copyright 2025 Rive
 */

// #include "ore_render_pass_metal.hpp"
// #include "ore_buffer_metal.hpp"
// #include "ore_pipeline_metal.hpp"
// #include "ore_bind_group_metal.hpp"
// #include "rive/renderer/ore/ore_context_metal.hpp"
// #include "rive/rive_types.hpp"

// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source implementation
// renderer/src/ore/metal/ore_render_pass_metal.mm.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
// Source coverage: implementation through upstream e3b9461b (298 lines).

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![cfg(all(target_vendor = "apple", feature = "metal-backend"))]
use super::ore_render_pass_metal_hpp::RenderPassMetalState;
use super::*;

use crate::bind_group_layout::SlotSpace;
use std::any::Any;
use std::mem::size_of;
use std::rc::Weak as RcWeak;

use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::AnyResourceHandle;
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_binding_map_hpp::BindingMap;
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::{
    CullMode, FaceWinding, IndexFormat, PrimitiveTopology,
};

#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
use objc2::rc::Retained;
#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
use objc2_metal::{
    MTLCommandEncoder, MTLCullMode, MTLIndexType, MTLPrimitiveType, MTLRenderCommandEncoder,
    MTLScissorRect, MTLViewport, MTLWinding,
};

// namespace rive::ore

// The source Objective-C `id` values are nullable strong owners.  The
// companion mechanical header uses these same `Option<Retained<...>>`
// shapes; retaining the value here keeps the encoder, command buffer, and
// index backing alive for exactly the pass lifetime.
#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
// Vertex buffers are bound at slots [kMetalVertexBufferBase, ...) to avoid
// colliding with uniform buffers mapped to the low buffer indices
// ([[buffer(0)]] etc). Must stay in sync with ore_context_metal.mm.
const kMetalVertexBufferBase: u32 = 16;

// static MTLPrimitiveType orePrimitiveTopologyToMTL(PrimitiveTopology topo)
#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
fn orePrimitiveTopologyToMTL(topo: PrimitiveTopology) -> MTLPrimitiveType {
    match topo {
        PrimitiveTopology::pointList => MTLPrimitiveType::Point,
        PrimitiveTopology::lineList => MTLPrimitiveType::Line,
        PrimitiveTopology::lineStrip => MTLPrimitiveType::LineStrip,
        PrimitiveTopology::triangleList => MTLPrimitiveType::Triangle,
        PrimitiveTopology::triangleStrip => MTLPrimitiveType::TriangleStrip,
    }
}

// static MTLIndexType oreIndexFormatToMTL(IndexFormat format)
#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
fn oreIndexFormatToMTL(format: IndexFormat) -> MTLIndexType {
    match format {
        IndexFormat::uint16 => MTLIndexType::UInt16,
        IndexFormat::uint32 => MTLIndexType::UInt32,
        IndexFormat::none => unreachable!("RIVE_UNREACHABLE: index format none"),
    }
}

// static MTLCullMode oreCullModeToMTL(CullMode mode)
#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
fn oreCullModeToMTL(mode: CullMode) -> MTLCullMode {
    match mode {
        CullMode::none => MTLCullMode::None,
        CullMode::front => MTLCullMode::Front,
        CullMode::back => MTLCullMode::Back,
    }
}

// static MTLWinding oreWindingToMTL(FaceWinding winding)
#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
fn oreWindingToMTL(winding: FaceWinding) -> MTLWinding {
    match winding {
        FaceWinding::clockwise => MTLWinding::Clockwise,
        FaceWinding::counterClockwise => MTLWinding::CounterClockwise,
    }
}

// ============================================================================
// RenderPassMetal
// ============================================================================

impl RenderPassMetalState {
    // void RenderPassMetal::validate() const
    pub fn validate(&self) {
        // assert(!m_finished && "RenderPassMetal already finished");
        debug_assert!(!self.base.m_finished, "RenderPassMetal already finished");
        // assert(m_mtlEncoder != nil);
        debug_assert!(self.m_mtlEncoder.is_some());
    }

    // void RenderPassMetal::setPipeline(Pipeline* pipeline)
    pub fn setPipeline(&mut self, pipeline: &AnyResourceHandle) {
        self.validate();
        if !self.base.ownsResource(pipeline) {
            if let Some(context) = self.base.m_context.upgrade() {
                context.setLastError("setPipeline: pipeline belongs to another Context");
            }
            return;
        }
        let Some(p) = pipeline.downcast_ref::<PipelineMetal>() else {
            return;
        };
        // if (!checkPipelineCompat(pipeline))
        if !self.base.checkPipelineCompat(Some(&p.base)) {
            return;
        }

        self.base.m_unappliedGroups |= self.base.movedGroups(
            self.m_currentPipeline
                .as_ref()
                .and_then(|old| old.pipelineBase()),
            &p.base,
        );

        // auto* p = static_cast<PipelineMetal*>(pipeline);
        // [m_mtlEncoder setRenderPipelineState:p->m_mtlPipeline];
        // [m_mtlEncoder setDepthStencilState:p->m_mtlDepthStencil];
        let encoder = self.m_mtlEncoder.as_ref();
        if let (Some(encoder), Some(native_pipeline)) = (encoder, p.m_mtlPipeline.as_ref()) {
            encoder.setRenderPipelineState(native_pipeline);
            encoder.setDepthStencilState(p.m_mtlDepthStencil.as_ref().map(Retained::as_ref));
        }

        // const auto& desc = pipeline->desc();
        let desc = p.base.desc();
        // [m_mtlEncoder setCullMode:oreCullModeToMTL(desc.cullMode)];
        // [m_mtlEncoder setFrontFacingWinding:oreWindingToMTL(desc.winding)];
        if let Some(encoder) = encoder {
            encoder.setCullMode(oreCullModeToMTL(desc.cullMode));
            encoder.setFrontFacingWinding(oreWindingToMTL(desc.winding));
        }
        // m_mtlPrimitiveType = orePrimitiveTopologyToMTL(desc.topology);
        self.m_mtlPrimitiveType = orePrimitiveTopologyToMTL(desc.topology);
        // m_currentPipeline = ref_rcp(pipeline);
        *self.m_currentPipeline = Some(pipeline.clone());

        // if (desc.depthStencil.depthBias != 0 ||
        //     desc.depthStencil.depthBiasSlopeScale != 0.0f)
        if desc.depthStencil.depthBias != 0 || desc.depthStencil.depthBiasSlopeScale != 0.0 {
            // [m_mtlEncoder setDepthBias:(float)desc.depthStencil.depthBias
            //                 slopeScale:desc.depthStencil.depthBiasSlopeScale
            //                      clamp:desc.depthStencil.depthBiasClamp];
            if let Some(encoder) = encoder {
                encoder.setDepthBias_slopeScale_clamp(
                    desc.depthStencil.depthBias as f32,
                    desc.depthStencil.depthBiasSlopeScale,
                    desc.depthStencil.depthBiasClamp,
                );
            }
        }
    }

    // void RenderPassMetal::setVertexBuffer(uint32_t slot,
    //                                       Buffer* buffer,
    //                                       uint32_t offset)
    pub fn setVertexBuffer(&mut self, slot: u32, buffer: &AnyResourceHandle, offset: u32) {
        self.validate();
        if !self.base.ownsResource(buffer) {
            if let Some(context) = self.base.m_context.upgrade() {
                context.setLastError("setVertexBuffer: buffer belongs to another Context");
            }
            return;
        }
        let Some(buffer) = buffer.downcast_ref::<BufferMetal>() else {
            return;
        };
        // auto* b = static_cast<BufferMetal*>(buffer);
        // b->markBound();
        let current = buffer.currentAndMarkBound();
        // [m_mtlEncoder setVertexBuffer:b->current()
        //                            offset:offset
        //                           atIndex:slot + kMetalVertexBufferBase];
        if let Some(encoder) = self.m_mtlEncoder.as_ref() {
            unsafe {
                encoder.setVertexBuffer_offset_atIndex(
                    current.as_ref().map(|value| value.as_ref()),
                    offset as usize,
                    slot.wrapping_add(kMetalVertexBufferBase) as usize,
                );
            }
        }
    }

    // void RenderPassMetal::setIndexBuffer(Buffer* buffer,
    //                                      IndexFormat format,
    //                                      uint32_t offset)
    pub fn setIndexBuffer(&mut self, buffer: &AnyResourceHandle, format: IndexFormat, offset: u32) {
        self.validate();
        if !self.base.ownsResource(buffer) {
            if let Some(context) = self.base.m_context.upgrade() {
                context.setLastError("setIndexBuffer: buffer belongs to another Context");
            }
            return;
        }
        let Some(buffer) = buffer.downcast_ref::<BufferMetal>() else {
            return;
        };
        // auto* b = static_cast<BufferMetal*>(buffer);
        // b->markBound();
        let current = buffer.currentAndMarkBound();
        // m_mtlIndexBuffer = b->current();
        *self.m_mtlIndexBuffer = current;
        // m_mtlIndexType = oreIndexFormatToMTL(format);
        self.m_mtlIndexType = oreIndexFormatToMTL(format);
        // m_mtlIndexBufferOffset = offset;
        self.m_mtlIndexBufferOffset = offset as usize;
    }

    // void RenderPassMetal::setBindGroup(uint32_t groupIndex,
    //                                    BindGroup* bg,
    //                                    const uint32_t* dynamicOffsets,
    //                                    uint32_t dynamicOffsetCount)
    pub fn setBindGroup(
        &mut self,
        groupIndex: u32,
        bg: &AnyResourceHandle,
        dynamicOffsets: Option<&[u32]>,
        dynamicOffsetCount: u32,
    ) {
        self.validate();
        if !self.base.ownsResource(bg) {
            if let Some(context) = self.base.m_context.upgrade() {
                context.setLastError("setBindGroup: bind group belongs to another Context");
            }
            return;
        }
        self.base
            .storeBindGroup(groupIndex, Some(bg), dynamicOffsets, dynamicOffsetCount);
    }

    fn applyBindGroup(&mut self, groupIndex: u32) {
        let bg = self.base.m_boundGroups[groupIndex as usize]
            .as_ref()
            .expect("bound group");
        let bgMetal = bg
            .downcast_ref::<BindGroupMetal>()
            .expect("Metal bind group");
        let pipeline = self
            .m_currentPipeline
            .as_ref()
            .and_then(|p| p.pipelineBase())
            .expect("current pipeline");
        let dynamicOffsets = &self.base.m_groupOffsets[groupIndex as usize];
        let dynamicOffsetCount = self.base.m_groupOffsetCounts[groupIndex as usize];
        // uint32_t dynIdx = 0;
        let mut dynIdx = 0_u32;
        // for (auto& b : bgMetal->m_mtlBuffers)
        for b in bgMetal.m_mtlBuffers.iter() {
            // uint32_t offset = b.offset;
            let mut offset = b.offset;
            // if (b.hasDynamicOffset && dynIdx < dynamicOffsetCount)
            if b.hasDynamicOffset && dynIdx < dynamicOffsetCount {
                // offset += dynamicOffsets[dynIdx++];
                let offsets = dynamicOffsets;
                offset = offset.wrapping_add(offsets[dynIdx as usize]);
                dynIdx += 1;
            }
            let slots = pipeline.slots(groupIndex, b.binding, SlotSpace::buffer);
            if slots.vs == BindingMap::kAbsent && slots.fs == BindingMap::kAbsent {
                continue;
            }
            // Resolve the live backing and mark it bound.
            // b.src->markBound();
            let src = b
                .source(bgMetal)
                .expect("BindGroupMetal retained buffer index must be valid");
            // `markBound(); current()` is atomic at the Rust safety boundary.
            let mtlBuffer = src.currentAndMarkBound();
            let encoder = self.m_mtlEncoder.as_ref();
            // if (slots.vs != BindingMap::kAbsent)
            if slots.vs != BindingMap::kAbsent
                && let Some(encoder) = encoder
            {
                // [m_mtlEncoder setVertexBuffer:mtlBuffer
                //                            offset:offset
                //                           atIndex:slots.vs];
                unsafe {
                    encoder.setVertexBuffer_offset_atIndex(
                        mtlBuffer.as_ref().map(|value| value.as_ref()),
                        offset as usize,
                        slots.vs as usize,
                    );
                }
            }
            // if (slots.fs != BindingMap::kAbsent)
            if slots.fs != BindingMap::kAbsent
                && let Some(encoder) = encoder
            {
                // [m_mtlEncoder setFragmentBuffer:mtlBuffer
                //                              offset:offset
                //                             atIndex:slots.fs];
                unsafe {
                    encoder.setFragmentBuffer_offset_atIndex(
                        mtlBuffer.as_ref().map(|value| value.as_ref()),
                        offset as usize,
                        slots.fs as usize,
                    );
                }
            }
        }
        // for (auto& t : bgMetal->m_mtlTextures)
        for t in bgMetal.m_mtlTextures.iter() {
            let slots = pipeline.slots(groupIndex, t.binding, SlotSpace::texture);
            let encoder = self.m_mtlEncoder.as_ref();
            // if (slots.vs != BindingMap::kAbsent)
            if slots.vs != BindingMap::kAbsent
                && let Some(encoder) = encoder
            {
                // [m_mtlEncoder setVertexTexture:t.texture atIndex:slots.vs];
                unsafe {
                    encoder.setVertexTexture_atIndex(
                        t.texture.as_ref().map(|value| value.as_ref()),
                        slots.vs as usize,
                    );
                }
            }
            // if (slots.fs != BindingMap::kAbsent)
            if slots.fs != BindingMap::kAbsent
                && let Some(encoder) = encoder
            {
                // [m_mtlEncoder setFragmentTexture:t.texture atIndex:slots.fs];
                unsafe {
                    encoder.setFragmentTexture_atIndex(
                        t.texture.as_ref().map(|value| value.as_ref()),
                        slots.fs as usize,
                    );
                }
            }
        }
        // for (auto& s : bgMetal->m_mtlSamplers)
        for s in bgMetal.m_mtlSamplers.iter() {
            let slots = pipeline.slots(groupIndex, s.binding, SlotSpace::sampler);
            let encoder = self.m_mtlEncoder.as_ref();
            // if (slots.vs != BindingMap::kAbsent)
            if slots.vs != BindingMap::kAbsent
                && let Some(encoder) = encoder
            {
                // [m_mtlEncoder setVertexSamplerState:s.sampler atIndex:slots.vs];
                unsafe {
                    encoder.setVertexSamplerState_atIndex(
                        s.sampler.as_ref().map(|value| value.as_ref()),
                        slots.vs as usize,
                    );
                }
            }
            // if (slots.fs != BindingMap::kAbsent)
            if slots.fs != BindingMap::kAbsent
                && let Some(encoder) = encoder
            {
                // [m_mtlEncoder setFragmentSamplerState:s.sampler atIndex:slots.fs];
                unsafe {
                    encoder.setFragmentSamplerState_atIndex(
                        s.sampler.as_ref().map(|value| value.as_ref()),
                        slots.fs as usize,
                    );
                }
            }
        }
    }

    // void RenderPassMetal::setViewport(float x, float y, float width,
    //                                    float height, float minDepth,
    //                                    float maxDepth)
    pub fn setViewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        minDepth: f32,
        maxDepth: f32,
    ) {
        self.validate();
        // MTLViewport vp = {
        //     .originX = (double)x,
        //     .originY = (double)y,
        //     .width = (double)width,
        //     .height = (double)height,
        //     .znear = (double)minDepth,
        //     .zfar = (double)maxDepth,
        // };
        let vp = MTLViewport {
            originX: x as f64,
            originY: y as f64,
            width: width as f64,
            height: height as f64,
            znear: minDepth as f64,
            zfar: maxDepth as f64,
        };
        // [m_mtlEncoder setViewport:vp];
        if let Some(encoder) = self.m_mtlEncoder.as_ref() {
            encoder.setViewport(vp);
        }
    }

    // void RenderPassMetal::setScissorRect(uint32_t x, uint32_t y,
    //                                      uint32_t width, uint32_t height)
    pub fn setScissorRect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.validate();
        // MTLScissorRect rect = {.x = x, .y = y, .width = width, .height = height};
        let rect = MTLScissorRect {
            x: x as usize,
            y: y as usize,
            width: width as usize,
            height: height as usize,
        };
        // [m_mtlEncoder setScissorRect:rect];
        if let Some(encoder) = self.m_mtlEncoder.as_ref() {
            encoder.setScissorRect(rect);
        }
    }

    // void RenderPassMetal::setStencilReference(uint32_t ref)
    pub fn setStencilReference(&mut self, reference: u32) {
        self.validate();
        // [m_mtlEncoder setStencilReferenceValue:ref];
        if let Some(encoder) = self.m_mtlEncoder.as_ref() {
            encoder.setStencilReferenceValue(reference);
        }
    }

    // void RenderPassMetal::setBlendColor(float r, float g, float b, float a)
    pub fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.validate();
        // [m_mtlEncoder setBlendColorRed:r green:g blue:b alpha:a];
        if let Some(encoder) = self.m_mtlEncoder.as_ref() {
            encoder.setBlendColorRed_green_blue_alpha(r, g, b, a);
        }
    }

    // void RenderPassMetal::draw(uint32_t vertexCount, uint32_t instanceCount,
    //                            uint32_t firstVertex, uint32_t firstInstance)
    pub fn draw(
        &mut self,
        vertexCount: u32,
        instanceCount: u32,
        firstVertex: u32,
        firstInstance: u32,
    ) {
        self.validate();
        if self.m_currentPipeline.is_none() {
            return;
        }
        while let Some(group) = self.base.takeNextUnappliedGroup() {
            self.applyBindGroup(group);
        }
        // [m_mtlEncoder drawPrimitives:m_mtlPrimitiveType
        //                  vertexStart:firstVertex
        //                  vertexCount:vertexCount
        //                instanceCount:instanceCount
        //                 baseInstance:firstInstance];
        if let Some(encoder) = self.m_mtlEncoder.as_ref() {
            unsafe {
                encoder.drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance(
                    self.m_mtlPrimitiveType,
                    firstVertex as usize,
                    vertexCount as usize,
                    instanceCount as usize,
                    firstInstance as usize,
                );
            }
        }
    }

    // void RenderPassMetal::drawIndexed(uint32_t indexCount,
    //                                   uint32_t instanceCount,
    //                                   uint32_t firstIndex,
    //                                   int32_t baseVertex,
    //                                   uint32_t firstInstance)
    pub fn drawIndexed(
        &mut self,
        indexCount: u32,
        instanceCount: u32,
        firstIndex: u32,
        baseVertex: i32,
        firstInstance: u32,
    ) {
        self.validate();
        if self.m_currentPipeline.is_none() || self.m_mtlIndexBuffer.is_none() {
            return;
        }
        while let Some(group) = self.base.takeNextUnappliedGroup() {
            self.applyBindGroup(group);
        }
        // assert(m_mtlIndexBuffer != nil &&
        //        "Must call setIndexBuffer before drawIndexed");
        debug_assert!(
            self.m_mtlIndexBuffer.is_some(),
            "Must call setIndexBuffer before drawIndexed"
        );

        // uint32_t indexSize = (m_mtlIndexType == MTLIndexTypeUInt32)
        //                          ? sizeof(uint32_t)
        //                          : sizeof(uint16_t);
        let indexSize = if self.m_mtlIndexType == MTLIndexType::UInt32 {
            size_of::<u32>()
        } else {
            size_of::<u16>()
        };
        // NSUInteger indexBufferOffset =
        //     m_mtlIndexBufferOffset + firstIndex * indexSize;
        let authored_offset = firstIndex.wrapping_mul(indexSize as u32);
        let indexBufferOffset = self.m_mtlIndexBufferOffset + authored_offset as usize;

        // [m_mtlEncoder drawIndexedPrimitives:m_mtlPrimitiveType
        //                          indexCount:indexCount
        //                           indexType:m_mtlIndexType
        //                         indexBuffer:m_mtlIndexBuffer
        //                   indexBufferOffset:indexBufferOffset
        //                       instanceCount:instanceCount
        //                          baseVertex:baseVertex
        //                        baseInstance:firstInstance];
        if let (Some(encoder), Some(index_buffer)) =
            (self.m_mtlEncoder.as_ref(), self.m_mtlIndexBuffer.as_ref())
        {
            unsafe {
                encoder.drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount_baseVertex_baseInstance(
                    self.m_mtlPrimitiveType,
                    indexCount as usize,
                    self.m_mtlIndexType,
                    index_buffer.as_ref(),
                    indexBufferOffset,
                    instanceCount as usize,
                    baseVertex as isize,
                    firstInstance as usize,
                );
            }
        }
    }

    // void RenderPassMetal::finish()
    pub fn finish(&mut self) {
        // if (m_finished)
        if self.base.m_finished {
            return;
        }
        // m_finished = true;
        self.base.m_finished = true;
        // if (m_mtlEncoder != nil)
        if let Some(encoder) = self.m_mtlEncoder.take() {
            // [m_mtlEncoder endEncoding];
            encoder.endEncoding();
            // m_mtlEncoder = nil;
        }
        self.base.releaseBoundResources();
        // m_currentPipeline.reset();
        *self.m_currentPipeline = None;
    }
}

impl RenderPassApi for RenderPassMetal {
    fn asAny(&self) -> &dyn Any {
        self
    }

    fn asAnyMut(&mut self) -> &mut dyn Any {
        self
    }

    fn intoAny(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn activeToken(&self) -> RcWeak<dyn ActiveRenderPass> {
        RenderPassMetal::activeToken(self)
    }

    fn setPipeline(&mut self, pipeline: Option<&AnyResourceHandle>) {
        if let Some(pipeline) = pipeline {
            self.inner.borrowState().setPipeline(pipeline);
        }
    }

    fn setVertexBuffer(&mut self, slot: u32, buffer: Option<&AnyResourceHandle>, offset: u32) {
        if let Some(buffer) = buffer {
            self.inner
                .borrowState()
                .setVertexBuffer(slot, buffer, offset);
        }
    }

    fn setIndexBuffer(
        &mut self,
        buffer: Option<&AnyResourceHandle>,
        format: IndexFormat,
        offset: u32,
    ) {
        if let Some(buffer) = buffer {
            self.inner
                .borrowState()
                .setIndexBuffer(buffer, format, offset);
        }
    }

    fn setBindGroup(
        &mut self,
        groupIndex: u32,
        bindGroup: Option<&AnyResourceHandle>,
        dynamicOffsets: Option<&[u32]>,
        dynamicOffsetCount: u32,
    ) {
        if let Some(bindGroup) = bindGroup {
            self.inner.borrowState().setBindGroup(
                groupIndex,
                bindGroup,
                dynamicOffsets,
                dynamicOffsetCount,
            );
        }
    }

    fn setViewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        minDepth: f32,
        maxDepth: f32,
    ) {
        self.inner
            .borrowState()
            .setViewport(x, y, width, height, minDepth, maxDepth);
    }

    fn setScissorRect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.inner.borrowState().setScissorRect(x, y, width, height);
    }

    fn setStencilReference(&mut self, reference: u32) {
        self.inner.borrowState().setStencilReference(reference);
    }

    fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.inner.borrowState().setBlendColor(r, g, b, a);
    }

    fn draw(&mut self, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
        self.inner
            .borrowState()
            .draw(vertexCount, instanceCount, firstVertex, firstInstance);
    }

    fn drawIndexed(
        &mut self,
        indexCount: u32,
        instanceCount: u32,
        firstIndex: u32,
        baseVertex: i32,
        firstInstance: u32,
    ) {
        self.inner.borrowState().drawIndexed(
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        );
    }

    fn finish(&mut self) {
        self.inner.borrowState().finish();
    }

    fn validate(&self) {
        self.inner.borrowState().validate();
    }
}

// } // namespace rive::ore
