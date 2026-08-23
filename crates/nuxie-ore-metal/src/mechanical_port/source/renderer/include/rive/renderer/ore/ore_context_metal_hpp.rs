/*
 * Copyright 2025 Rive
 */

// #pragma once

// #include "rive/renderer/ore/ore_context.hpp"

// #import <Metal/Metal.h>

// #include <atomic>
// #include <memory>

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_context_metal.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

use crate::mechanical_port::source::renderer::src::ore::metal::ore_buffer_metal_hpp::{
    BufferErrorSink, BufferMetalContextState,
};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::AnyResourceHandle;

// `id<MTLDevice>`, `id<MTLCommandQueue>`, and `id<MTLCommandBuffer>` are
// nullable, strong Objective-C owners under ARC. `Retained<T>` is the
// corresponding Rust owner; `Option` preserves each source `nil` state. The
// non-Apple stand-ins keep this source-shaped translation available to tools
// that inspect it off Apple.
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::{MTLCommandBuffer, MTLCommandQueue, MTLDevice};

#[cfg(target_vendor = "apple")]
type NativeMetalDevice = Option<Retained<ProtocolObject<dyn MTLDevice>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalDevice = Option<()>;

#[cfg(target_vendor = "apple")]
type NativeMetalQueue = Option<Retained<ProtocolObject<dyn MTLCommandQueue>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalQueue = Option<()>;

#[cfg(target_vendor = "apple")]
type NativeMetalCommandBuffer = Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalCommandBuffer = Option<()>;

// namespace rive::ore

// class RenderPassMetal;
// class BindGroupMetal;
// class TextureMetal;
// The source forward declarations are retained for the friend relationships
// below. Each concrete type is owned by its own translation unit.

// class ContextMetal : public Context
// {
// Rust has no class inheritance. `base` is the Context base subobject and is
// kept first to preserve the source inheritance order and owner relationship.
// The concrete Metal type remains the source lite-RTTI/backend identity; it
// is not duplicated as a payload field here.
#[repr(C)]
pub struct ContextMetalMembers {
    // private:
    // friend class RenderPassMetal;
    // friend class BindGroupMetal;
    // friend class TextureMetal;
    // Rust has no friend declarations; these source access boundaries remain
    // visible here, and the owning translation units use their crate-local
    // access to the source-shaped owner.

    // id<MTLDevice> m_mtlDevice = nil;
    // Strong device owner. `None` preserves the source nil state accepted by
    // Make and retained until the context owner is dropped.
    pub(crate) m_mtlDevice: ManuallyDrop<NativeMetalDevice>,
    // id<MTLCommandQueue> m_mtlQueue = nil;
    // Strong queue owner. `None` preserves the source nil state accepted by
    // Make and retained until the context owner is dropped.
    pub(crate) m_mtlQueue: ManuallyDrop<NativeMetalQueue>,
    // id<MTLCommandBuffer> m_mtlCommandBuffer = nil;
    // Optional current recording command-buffer owner. beginFrame replaces
    // this slot; endFrame commits and clears it, and waitForGPU observes only
    // the current slot as in the pinned implementation.
    pub(crate) m_mtlCommandBuffer: ManuallyDrop<NativeMetalCommandBuffer>,
    // std::vector<rcp<BindGroup>> m_deferredBindGroups;
    pub(crate) m_deferredBindGroups: ManuallyDrop<Vec<AnyResourceHandle>>,
    // Authored `uint64_t`, represented atomically only because the Rust safety
    // adapter replaces BufferMetal's raw ContextMetal pointer with shared
    // access to this exact storage.
    pub(crate) m_currentSerial: ManuallyDrop<Arc<AtomicU64>>,
    // Exact shared_ptr<atomic<uint64_t>> completion owner.
    pub(crate) m_completedSerial: ManuallyDrop<Arc<AtomicU64>>,
}

#[repr(C)]
pub struct ContextMetal {
    pub(crate) base: ManuallyDrop<Context>,
    pub(crate) members: ManuallyDrop<ContextMetalMembers>,
    // Nonowning-behavior safety adapter: its serial Arcs alias the two exact
    // authored member allocations above; it owns no parallel serial state.
    pub(crate) m_bufferState: ManuallyDrop<Arc<BufferMetalContextState>>,
}

impl Deref for ContextMetal {
    type Target = ContextMetalMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for ContextMetal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for ContextMetal {
    fn drop(&mut self) {
        unsafe {
            // The authored `~ContextMetal()` body nils these three ARC owners
            // before C++ begins automatic reverse member destruction.
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("ContextMetal.commandBuffer");
            *self.m_mtlCommandBuffer = None;
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("ContextMetal.queue");
            *self.m_mtlQueue = None;
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("ContextMetal.device");
            *self.m_mtlDevice = None;

            // C++ now performs automatic reverse member destruction. Release
            // the Rust safety adapter first because it is declared after the
            // exact source aggregate, then tear that aggregate down in reverse.
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("ContextMetal.bufferStateAdapter");
            ManuallyDrop::drop(&mut self.m_bufferState);
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("ContextMetal.completedSerial");
            ManuallyDrop::drop(&mut self.m_completedSerial);
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("ContextMetal.currentSerial");
            ManuallyDrop::drop(&mut self.m_currentSerial);
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("ContextMetal.deferredBindGroups");
            ManuallyDrop::drop(&mut self.m_deferredBindGroups);
            ManuallyDrop::drop(&mut self.m_mtlCommandBuffer);
            ManuallyDrop::drop(&mut self.m_mtlQueue);
            ManuallyDrop::drop(&mut self.m_mtlDevice);
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("ContextMetal.base");
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl ContextMetal {
    // public:

    // static std::unique_ptr<ContextMetal> Make(id<MTLDevice> device,
    //                                           id<MTLCommandQueue> queue);
    // Rust spelling of the owning unique_ptr return is
    // `Option<Box<ContextMetal>>`; nullable device and queue arguments remain
    // `NativeMetalDevice` / `NativeMetalQueue` in the source-shaped factory.

    // ~ContextMetal() override;
    // The paired ore_context_metal.mm translation owns the explicit release
    // of the current command buffer, queue, and device. Rust's default drop
    // glue releases all retained handles and source-owned state.

    // rcp<Buffer> makeBuffer(const BufferDesc& desc) override;
    // rcp<Texture> makeTexture(const TextureDesc& desc) override;
    // rcp<TextureView> makeTextureView(const TextureViewDesc& desc) override;
    // rcp<Sampler> makeSampler(const SamplerDesc& desc) override;
    // rcp<ShaderModule> makeShaderModule(const ShaderModuleDesc& desc) override;
    // rcp<BindGroupLayout> makeBindGroupLayout(
    //     const BindGroupLayoutDesc& desc) override;
    // rcp<Pipeline> makePipeline(const PipelineDesc& desc,
    //                            std::string* outError = nullptr) override;
    // rcp<BindGroup> makeBindGroup(const BindGroupDesc& desc) override;

    // std::unique_ptr<RenderPass> beginRenderPass(
    //     const RenderPassDesc& desc,
    //     std::string* outError = nullptr) override;
    // The source nullable result remains an optional owning pass. The paired
    // Metal implementation preserves its non-null command-buffer assertion
    // seam when constructing a render pass.

    // void beginFrame(const FrameDescriptor&) override;
    // void endFrame() override;
    // void waitForGPU() override;

    // rcp<TextureView> wrapCanvasTexture(gpu::RenderCanvas* canvas) override;
    // rcp<TextureView> wrapRiveTexture(gpu::Texture* gpuTex,
    //                                  uint32_t width,
    //                                  uint32_t height) override;

    // ShaderTarget shaderTarget() const override { return ShaderTarget::msl; }
    pub fn shaderTarget(&self) -> ShaderTarget {
        ShaderTarget::msl
    }

    // Buffer versioning serials. currentSerial is the command buffer being
    // recorded. completedSerial is the highest the GPU has finished, so a
    // backing at or below it can be recycled. See ore_buffer_metal.mm.
    // uint64_t currentSerial() const { return m_currentSerial; }
    pub fn currentSerial(&self) -> u64 {
        self.m_currentSerial.load(Ordering::Relaxed)
    }

    // uint64_t completedSerial() const
    // {
    //     return m_completedSerial->load(std::memory_order_relaxed);
    // }
    pub fn completedSerial(&self) -> u64 {
        self.m_completedSerial.load(Ordering::Relaxed)
    }

    // id<MTLDevice> device() const { return m_mtlDevice; }
    // Returning the alias clones the strong ARC-equivalent owner, preserving
    // the source nullable handle result.
    pub fn device(&self) -> NativeMetalDevice {
        (*self.m_mtlDevice).clone()
    }

    // ContextMetal(const ContextMetal&) = delete;
    // ContextMetal& operator=(const ContextMetal&) = delete;
    // Rust's ContextMetal has no Clone implementation; ordinary moves remain
    // the only transfer operation for this source-shaped owner.

    // private:

    // ContextMetal() : Context(nullptr) {}
    pub(crate) fn new() -> Self {
        let base = Context::new(Features::default(), None);
        let errorSink: Arc<dyn BufferErrorSink> = base.state.clone();
        let currentSerial = Arc::new(AtomicU64::new(0));
        let completedSerial = Arc::new(AtomicU64::new(0));
        let bufferState = BufferMetalContextState::fromSerials(
            currentSerial.clone(),
            completedSerial.clone(),
            Some(Arc::downgrade(&errorSink)),
        );
        Self {
            base: ManuallyDrop::new(base),
            members: ManuallyDrop::new(ContextMetalMembers {
                m_mtlDevice: ManuallyDrop::new(None),
                m_mtlQueue: ManuallyDrop::new(None),
                m_mtlCommandBuffer: ManuallyDrop::new(None),
                m_deferredBindGroups: ManuallyDrop::new(Vec::new()),
                m_currentSerial: ManuallyDrop::new(currentSerial),
                m_completedSerial: ManuallyDrop::new(completedSerial),
            }),
            m_bufferState: ManuallyDrop::new(bufferState),
        }
    }

    // Metal implementation helpers — defined in ore_context_metal.mm.
    // The public make*/begin*/wrap* overrides delegate to these.
    // void mtlPopulateFeatures(id<MTLDevice> device);
    // rcp<Buffer> mtlMakeBuffer(const BufferDesc& desc);
    // rcp<Texture> mtlMakeTexture(const TextureDesc& desc);
    // rcp<TextureView> mtlMakeTextureView(const TextureViewDesc& desc);
    // rcp<Sampler> mtlMakeSampler(const SamplerDesc& desc);
    // rcp<ShaderModule> mtlMakeShaderModule(const ShaderModuleDesc& desc);
    // rcp<Pipeline> mtlMakePipeline(const PipelineDesc& desc,
    //                               std::string* outError);
    // rcp<BindGroup> mtlMakeBindGroup(const BindGroupDesc& desc);
    // std::unique_ptr<RenderPass> mtlBeginRenderPass(
    //     const RenderPassDesc& desc,
    //     std::string* outError);
    // rcp<TextureView> mtlWrapCanvasTexture(gpu::RenderCanvas* canvas);
}

// } // namespace rive::ore
