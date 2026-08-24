/*
 * Copyright 2025 Rive
 */

// #pragma once

// #include <cstdarg>
// #include <cstdio>
// #include <memory>
// #include <string>
// #include <vector>
// #include "rive/refcnt.hpp"
// #include "rive/renderer/ore/ore_types.hpp"
// #include "rive/renderer/ore/ore_buffer.hpp"
// #include "rive/renderer/ore/ore_texture.hpp"
// #include "rive/renderer/ore/ore_sampler.hpp"
// #include "rive/renderer/ore/ore_shader_module.hpp"
// #include "rive/renderer/ore/ore_pipeline.hpp"
// #include "rive/renderer/ore/ore_bind_group.hpp"
// #include "rive/renderer/ore/ore_render_pass.hpp"

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_context.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::rc::Weak as RcWeak;
use std::sync::{Arc, Mutex, MutexGuard};

use super::super::gpu_resource_hpp::{
    AnyResourceHandle, GPUResourceManager, ResourceDomain, ResourceFinalReleaseDrain,
};
use super::ore_types_hpp::{
    BindGroupDesc, BindGroupLayoutDesc, BufferDesc, Features, PipelineDesc, RenderPassDesc,
    SamplerDesc, ShaderModuleDesc, TextureDesc, TextureFormat, TextureViewDesc,
};
#[cfg(target_vendor = "apple")]
use crate::mechanical_port::source::renderer::src::ore::metal::ore_buffer_metal_hpp::BufferErrorSink;
#[cfg(target_vendor = "apple")]
use crate::mechanical_port::source::renderer::src::ore::metal::ore_texture_metal_hpp::TextureViewMetal;

// namespace rive::gpu
// class RenderCanvas;
// class Texture;
// The source GPU host objects remain opaque to this source-shaped header.

// namespace rive::ore

// RSTB asset target ID. Wire format, must match editor export. 4 is unused.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderTarget {
    // enum class ShaderTarget : uint8_t
    wgsl = 0,
    glsl = 1,
    msl = 2,
    hlsl = 3,
    spirv = 5,
}

pub trait ActiveRenderPass {
    fn isFinished(&self) -> bool;
    fn finish(&self);
}

/// Cross-cutting Context state. The source active-pass pointer becomes a weak
/// token, and resources receive only a weak error sink plus cloned manager.
pub struct ContextState {
    // Rust safety sidecars. Identity remains tied to this source Context;
    // a concrete backend may clone only the drain into its execution root so
    // destruction can finish after the ORE Context has gone away.
    domainIdentity: Arc<()>,
    domainFinalReleases: ResourceFinalReleaseDrain,
    // Rust declaration order matches the required source destruction order.
    manager: Option<GPUResourceManager>,
    lastError: Mutex<String>,
    features: Mutex<Features>,
}

impl ContextState {
    pub fn new(features: Features, manager: Option<GPUResourceManager>) -> Arc<Self> {
        Self::newWithFinalReleaseDrain(features, manager, ResourceFinalReleaseDrain::new())
    }

    #[doc(hidden)]
    pub fn newWithFinalReleaseDrain(
        features: Features,
        manager: Option<GPUResourceManager>,
        domainFinalReleases: ResourceFinalReleaseDrain,
    ) -> Arc<Self> {
        Arc::new(Self {
            domainIdentity: Arc::new(()),
            domainFinalReleases,
            manager,
            lastError: Mutex::new(String::new()),
            features: Mutex::new(features),
        })
    }

    fn lockLastError(&self) -> MutexGuard<'_, String> {
        self.lastError
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn features(&self) -> Features {
        *self
            .features
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn manager(&self) -> Option<GPUResourceManager> {
        self.manager.clone()
    }

    pub fn resourceDomain(&self) -> ResourceDomain {
        self.domainFinalReleases
            .resource_domain(&self.domainIdentity)
    }

    pub fn resourceFinalReleaseDrain(&self) -> ResourceFinalReleaseDrain {
        self.domainFinalReleases.clone()
    }

    pub fn lastError(&self) -> String {
        self.lockLastError().clone()
    }

    pub fn clearLastError(&self) {
        self.lockLastError().clear();
    }

    pub fn setLastError(&self, message: impl Into<String>) {
        let mut message = message.into();
        if message.len() > 1023 {
            let mut end = 1023;
            while !message.is_char_boundary(end) {
                end -= 1;
            }
            message.truncate(end);
        }
        *self.lockLastError() = message;
    }
}

#[cfg(target_vendor = "apple")]
impl BufferErrorSink for ContextState {
    fn setBufferError(&self, message: &str) {
        self.setLastError(message);
    }
}

pub trait ContextApi {
    fn features(&self) -> Features;
    fn lastError(&self) -> String;
    fn activeRenderPass(&self) -> Option<RcWeak<dyn ActiveRenderPass>>;
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>);
    fn finishActiveRenderPass(&self);
    fn clearLastError(&self);
    fn setLastError(&self, message: &str);
    fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle>;
    fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle>;
    fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle>;
    fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle>;
    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle>;
    fn makeBindGroupLayout(&mut self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle>;
    fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle>;
    fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle>;
    fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>>;
    fn beginFrame(&mut self, descriptor: &FrameDescriptor);
    fn endFrame(&mut self);
    fn waitForGPU(&mut self);
    unsafe fn wrapCanvasTexture(&mut self, canvas: *mut c_void) -> Option<AnyResourceHandle>;
    unsafe fn wrapRiveTexture(
        &mut self,
        texture: *mut c_void,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle>;
    fn shaderTarget(&self) -> ShaderTarget;
}

// Backend-agnostic Ore graphics context.
//
// Held only via std::unique_ptr<Context>. Concrete subclasses live in
// per-backend headers and own the GPU-API state and dispatch:
//
//   ore_context_metal.hpp    → ContextMetal::Make(id<MTLDevice>,
//                                                 id<MTLCommandQueue>)
//   ore_context_gl.hpp       → ContextGL::Make()
//   ore_context_d3d11.hpp    → ContextD3D11::Make(ID3D11Device*,
//                                                 ID3D11DeviceContext*)
//   ore_context_d3d12.hpp    → ContextD3D12::Make(ID3D12Device*,
//                                                 ID3D12CommandQueue*)
//   ore_context_wgpu.hpp     → ContextWGPU::Make(wgpu::Device, wgpu::Queue,
//                                                wgpu::BackendType)
//   ore_context_vulkan.hpp   → ContextVulkan::Make(VkInstance, ...,
//   VmaAllocator,
//                                                  PFN_vkGetInstanceProcAddr)
//
// Backend-specific public methods (the typed external-CL beginFrame
// overloads, Vulkan render-pass cache lookup, WGPU GLES detection, etc.)
// live on the per-backend subclasses, not on this base — so this header
// pulls in no GPU-API headers and stays free of #ifdef branches.
//
// To call a backend-specific method, hold the subclass type at the call
// site (the subclass `Make` factory returns std::unique_ptr<Subclass>,
// which converts implicitly to std::unique_ptr<Context> for cross-backend
// use). Code that only needs the cross-backend API takes Context*.
pub struct Context {
    pub(crate) state: Arc<ContextState>,
    activeRenderPass: RefCell<Option<RcWeak<dyn ActiveRenderPass>>>,
}

impl Context {
    // public:

    // virtual ~Context() = default;
    // Rust's default drop glue supplies the virtual-destructor boundary for
    // each concrete context owner.

    // Resource factories. Rust has no pure-virtual member declaration; the
    // complete source signatures remain visible here and are implemented by
    // each backend-specific context translation.
    // virtual rcp<Buffer> makeBuffer(const BufferDesc& desc) = 0;
    // virtual rcp<Texture> makeTexture(const TextureDesc& desc) = 0;
    // virtual rcp<TextureView> makeTextureView(const TextureViewDesc& desc) = 0;
    // virtual rcp<Sampler> makeSampler(const SamplerDesc& desc) = 0;
    // virtual rcp<ShaderModule> makeShaderModule(
    //     const ShaderModuleDesc& desc) = 0;
    // virtual rcp<BindGroupLayout> makeBindGroupLayout(
    //     const BindGroupLayoutDesc& desc) = 0;
    // virtual rcp<Pipeline> makePipeline(const PipelineDesc& desc,
    //                                    std::string* outError = nullptr) = 0;
    // virtual rcp<BindGroup> makeBindGroup(const BindGroupDesc& desc) = 0;
    //
    // Rust spelling of the owning unique_ptr return is
    // `Option<Box<RenderPass>>`; concrete backends retain the source nullable
    // factory result and publish no partial pass on failure.
    // virtual std::unique_ptr<RenderPass> beginRenderPass(
    //     const RenderPassDesc& desc,
    //     std::string* outError = nullptr) = 0;

    // struct FrameDescriptor
    // {
    //     // Because ore is currently imidiate mode, the command buffer must be
    //     // passed in on begin frame instead of end frame.
    //     void* externalCommandBuffer = nullptr;
    //     uint64_t safeFrameNumber;
    //     uint64_t currentFrameNumber;
    // };

    // Rust does not permit nested struct declarations in a struct body. The
    // source nested record is translated as the source-shaped sibling below.

    // virtual void beginFrame(const FrameDescriptor&) = 0;
    // virtual void endFrame() = 0;
    //
    // Block until the most recent endFrame() submission completes on the
    // GPU. Call this before destroying Ore resources (textures, views,
    // pipelines) that were used in the current frame. Not needed if
    // resources stay alive until the next beginFrame(), which waits
    // automatically.
    // virtual void waitForGPU() = 0;
    //
    // The source GPU host pointers are represented as nullable opaque
    // borrows in each concrete backend's wrapper.
    // virtual rcp<TextureView> wrapCanvasTexture(gpu::RenderCanvas* canvas) = 0;
    // virtual rcp<TextureView> wrapRiveTexture(gpu::Texture* gpuTex,
    //                                          uint32_t width,
    //                                          uint32_t height) = 0;
    //
    // Which RSTB shader variant this backend consumes.
    // virtual ShaderTarget shaderTarget() const = 0;

    // ------------------------------------------------------------------------
    // Cross-cutting state and accessors. Non-virtual; live on this base
    // because they are uniform across backends.
    // ------------------------------------------------------------------------

    // const Features& features() const { return m_features; }
    pub fn features(&self) -> Features {
        self.state.features()
    }

    pub(crate) fn features_mut_unpublished(&self) -> MutexGuard<'_, Features> {
        self.state
            .features
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    // Active render pass tracking — used by Lua bindings to auto-finish
    // stale passes and by backends that enforce one-encoder-at-a-time.
    // RenderPass* activeRenderPass() const { return m_activeRenderPass; }
    pub fn activeRenderPass(&self) -> Option<RcWeak<dyn ActiveRenderPass>> {
        self.activeRenderPass.borrow().clone()
    }

    // void setActiveRenderPass(RenderPass* pass) { m_activeRenderPass = pass; }
    pub fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        *self.activeRenderPass.borrow_mut() = pass.map(RenderPassApi::activeToken);
    }

    // Called at the top of every backend's beginRenderPass(). If a prior pass
    // is still open, finish it — matches the Lua binding's auto-finish
    // contract and means backends that enforce one-encoder-at-a-time (Metal,
    // D3D12) won't assert when a second beginRenderPass happens within the
    // same command buffer. Does not clear m_activeRenderPass, because the
    // pointer identity is owned by the Lua wrapper that called setActive…().
    // inline void finishActiveRenderPass()
    pub fn finishActiveRenderPass(&self) {
        let active = self.activeRenderPass().and_then(|pass| pass.upgrade());
        if let Some(pass) = active
            && !pass.isFinished()
        {
            pass.finish();
        }
    }

    // Last validation error — set by setPipeline() / setBindGroup() when
    // format or layout checks fail. Cleared at beginFrame(). The Lua layer
    // reads this after native calls and fires luaL_error() if non-empty.
    // const std::string& lastError() const { return m_lastError; }
    pub fn lastError(&self) -> String {
        self.state.lastError()
    }

    // void clearLastError() { m_lastError.clear(); }
    pub fn clearLastError(&self) {
        self.state.clearLastError();
    }

    // Populate m_lastError with a printf-style message. Used by factory
    // methods to report construction failures to the Lua layer in lieu of
    // fprintf(stderr) / assert — which either spam the console or abort
    // the process.
    // void setLastError(const char* fmt, ...)
    // #if defined(__GNUC__) || defined(__clang__)
    //     __attribute__((format(printf, 2, 3)))
    // #endif
    // {
    //     va_list args;
    //     va_start(args, fmt);
    //     char buf[1024];
    //     vsnprintf(buf, sizeof(buf), fmt, args);
    //     va_end(args);
    //     m_lastError = buf;
    // }
    //
    // C variadic formatting has no direct Rust ABI. The source-shaped
    // translation receives the already formatted message; callers preserve
    // the source format string, truncation bound, and publication order at
    // their concrete backend boundary.
    pub fn setLastError(&self, message: impl Into<String>) {
        self.state.setLastError(message);
    }

    // Context(const Context&) = delete;
    // Context& operator=(const Context&) = delete;
    // Context(Context&&) = delete;
    // Context& operator=(Context&&) = delete;
    // Rust's Context has no Clone implementation; ordinary moves remain the
    // only transfer operation for this source-shaped owner.

    // protected:
    // Context(rcp<rive::gpu::GPUResourceManager> manager) :
    //     m_manager(std::move(manager))
    // {}
    pub(crate) fn new(features: Features, manager: Option<GPUResourceManager>) -> Self {
        Self::newWithFinalReleaseDrain(features, manager, ResourceFinalReleaseDrain::new())
    }

    #[doc(hidden)]
    pub(crate) fn newWithFinalReleaseDrain(
        features: Features,
        manager: Option<GPUResourceManager>,
        domainFinalReleases: ResourceFinalReleaseDrain,
    ) -> Self {
        Self {
            state: ContextState::newWithFinalReleaseDrain(features, manager, domainFinalReleases),
            activeRenderPass: RefCell::new(None),
        }
    }
}

// The source nested Context::FrameDescriptor record is emitted as a sibling
// because Rust does not permit nested struct declarations in a struct body.
pub struct FrameDescriptor {
    // Because ore is currently imidiate mode, the command buffer must be
    // passed in on begin frame instead of end frame.
    // void* externalCommandBuffer = nullptr;
    externalCommandBuffer: Option<NonNull<c_void>>,
    // uint64_t safeFrameNumber;
    pub safeFrameNumber: u64,
    // uint64_t currentFrameNumber;
    pub currentFrameNumber: u64,
}

impl FrameDescriptor {
    pub const fn new(safeFrameNumber: u64, currentFrameNumber: u64) -> Self {
        Self {
            externalCommandBuffer: None,
            safeFrameNumber,
            currentFrameNumber,
        }
    }

    /// # Safety
    /// `externalCommandBuffer` must identify the concrete command-buffer type
    /// required by the selected backend, belong to that backend's live device,
    /// and remain valid for the complete begin-frame operation.
    pub const unsafe fn withExternalCommandBuffer(
        externalCommandBuffer: NonNull<c_void>,
        safeFrameNumber: u64,
        currentFrameNumber: u64,
    ) -> Self {
        Self {
            externalCommandBuffer: Some(externalCommandBuffer),
            safeFrameNumber,
            currentFrameNumber,
        }
    }

    /// # Safety
    /// The caller must uphold the backend-specific validity contract recorded
    /// by `withExternalCommandBuffer` before interpreting or using the pointer.
    pub unsafe fn externalCommandBuffer(&self) -> Option<NonNull<c_void>> {
        self.externalCommandBuffer
    }
}

/// Exact raw ABI adapter for the source-nested
/// `Context::FrameDescriptor`. The safe record above deliberately uses
/// `Option<NonNull<_>>`; only this boundary is `repr(C)` and preserves the
/// authored nullable thin pointer followed by the two `uint64_t` fields.
pub mod raw_abi {
    use super::FrameDescriptor as SafeFrameDescriptor;
    use core::ffi::c_void;
    use core::ptr::NonNull;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct FrameDescriptor {
        pub externalCommandBuffer: *mut c_void,
        pub safeFrameNumber: u64,
        pub currentFrameNumber: u64,
    }

    impl FrameDescriptor {
        /// Convert the source ABI record to the safe Rust view. This is
        /// unsafe because a non-null command-buffer pointer must remain valid
        /// under the backend-specific begin-frame contract.
        pub unsafe fn borrow(&self) -> SafeFrameDescriptor {
            match NonNull::new(self.externalCommandBuffer) {
                Some(command_buffer) => unsafe {
                    SafeFrameDescriptor::withExternalCommandBuffer(
                        command_buffer,
                        self.safeFrameNumber,
                        self.currentFrameNumber,
                    )
                },
                None => SafeFrameDescriptor::new(self.safeFrameNumber, self.currentFrameNumber),
            }
        }

        pub fn from_safe(descriptor: &SafeFrameDescriptor) -> Self {
            Self {
                externalCommandBuffer: unsafe { descriptor.externalCommandBuffer() }
                    .map_or(core::ptr::null_mut(), NonNull::as_ptr),
                safeFrameNumber: descriptor.safeFrameNumber,
                currentFrameNumber: descriptor.currentFrameNumber,
            }
        }
    }
}

// ============================================================================
// RenderPass inline helpers — defined here rather than in ore_render_pass.hpp
// because they depend on Pipeline::desc() / TextureView::texture()->format(),
// which are pulled in by this header's full set of Ore includes.
// ============================================================================

// inline void RenderPass::populateAttachmentMetadata(const RenderPassDesc& desc)
impl RenderPass {
    #[cfg(target_vendor = "apple")]
    pub(crate) fn populateAttachmentMetadata(&mut self, desc: &RenderPassDesc<'_>) {
        // m_colorCount = desc.colorCount;
        self.m_colorCount = desc.colorCount;
        // for (uint32_t i = 0; i < desc.colorCount; ++i)
        for i in 0..desc.colorCount {
            // TextureView* view = desc.colorAttachments[i].view;
            let Some(view) = desc.colorAttachments[i as usize].view else {
                continue;
            };
            let Some(view) = view.downcast_ref::<TextureViewMetal>() else {
                continue;
            };
            // if (!view || !view->texture())
            let Some(texture) = view.baseTexture() else {
                continue;
            };
            // m_colorFormats[i] = view->texture()->format();
            self.m_colorFormats[i as usize] = texture.format();
            // m_sampleCount = view->texture()->sampleCount();
            self.m_sampleCount = texture.sampleCount();
        }
        // if (desc.depthStencil.view && desc.depthStencil.view->texture())
        if let Some(view) = desc.depthStencil.view
            && let Some(view) = view.downcast_ref::<TextureViewMetal>()
            && let Some(texture) = view.baseTexture()
        {
            // m_depthFormat = desc.depthStencil.view->texture()->format();
            self.m_depthFormat = texture.format();
            // m_hasDepth = true;
            self.m_hasDepth = true;
            // If no colour attachments drove sampleCount, take it from depth.
            // if (desc.colorCount == 0)
            if desc.colorCount == 0 {
                // m_sampleCount = desc.depthStencil.view->texture()->sampleCount();
                self.m_sampleCount = texture.sampleCount();
            }
        }
    }

    // inline bool RenderPass::checkPipelineCompat(const Pipeline* pipeline) const
    pub(crate) fn checkPipelineCompat(&self, pipeline: Option<&Pipeline>) -> bool {
        // if (!pipeline)
        let Some(pipeline) = pipeline else {
            return true;
        };
        // const PipelineDesc& pd = pipeline->desc();
        let pd = pipeline.desc();

        // if (pd.colorCount != m_colorCount)
        if pd.colorCount != self.m_colorCount {
            if let Some(context) = self.m_context.upgrade() {
                context.setLastError(format!(
                    "setPipeline: pipeline has {} color targets but render pass was begun with {}",
                    pd.colorCount, self.m_colorCount
                ));
            }
            return false;
        }
        // for (uint32_t i = 0; i < m_colorCount; ++i)
        for i in 0..self.m_colorCount {
            // if (pd.colorTargets[i].format != m_colorFormats[i])
            if pd.colorTargets[i as usize].format != self.m_colorFormats[i as usize] {
                if let Some(context) = self.m_context.upgrade() {
                    context.setLastError(format!(
                        "setPipeline: color target {} format mismatch (pipeline={}, pass={})",
                        i,
                        pd.colorTargets[i as usize].format as u8,
                        self.m_colorFormats[i as usize] as u8
                    ));
                }
                return false;
            }
        }
        // if (pd.sampleCount != m_sampleCount)
        if pd.sampleCount != self.m_sampleCount {
            if let Some(context) = self.m_context.upgrade() {
                context.setLastError(format!(
                    "setPipeline: sample count mismatch (pipeline={}, pass={})",
                    pd.sampleCount, self.m_sampleCount
                ));
            }
            return false;
        }
        // DepthStencilState::format == rgba8unorm is the "no depth" sentinel
        // (see ore_types.hpp:443). Treat that as "pipeline has no depth."
        // const bool pipelineHasDepth =
        //     pd.depthStencil.format != TextureFormat::rgba8unorm;
        let pipelineHasDepth = pd.depthStencil.format != TextureFormat::rgba8unorm;
        // if (pipelineHasDepth != m_hasDepth)
        if pipelineHasDepth != self.m_hasDepth {
            if let Some(context) = self.m_context.upgrade() {
                context.setLastError(format!(
                    "setPipeline: depth attachment {} (pipeline={}, pass={})",
                    if pipelineHasDepth {
                        "pipeline expects depth but pass has none"
                    } else {
                        "pipeline has no depth but pass provides it"
                    },
                    pipelineHasDepth as i32,
                    self.m_hasDepth as i32
                ));
            }
            return false;
        }
        // if (pipelineHasDepth && pd.depthStencil.format != m_depthFormat)
        if pipelineHasDepth && pd.depthStencil.format != self.m_depthFormat {
            if let Some(context) = self.m_context.upgrade() {
                context.setLastError(format!(
                    "setPipeline: depth format mismatch (pipeline={}, pass={})",
                    pd.depthStencil.format as u8, self.m_depthFormat as u8
                ));
            }
            return false;
        }
        // return true;
        true
    }
}

// } // namespace rive::ore
#[cfg(all(test, target_vendor = "apple"))]
mod tests {
    use super::*;
    use crate::gpu_resource::GPUResourceManagerOwner;
    use crate::metal::render_pass::RenderPassMetal;

    #[test]
    fn error_state_is_replaced_and_cleared_explicitly() {
        let owner = GPUResourceManagerOwner::new();
        let state = ContextState::new(Features::default(), Some(owner.manager()));
        state.setLastError("previous failure");
        assert_eq!(state.lastError(), "previous failure");
        state.setLastError("next failure");
        assert_eq!(state.lastError(), "next failure");
        state.clearLastError();
        assert_eq!(state.lastError(), "");
        let manager = state.manager().expect("manager");
        assert_eq!(manager.safeFrameNumber(), 0);
        assert_eq!(manager.currentFrameNumber(), 0);
        owner.shutdown();
    }

    #[test]
    fn active_pass_is_weakly_held_and_finished_at_most_once() {
        let context = Context::new(Features::default(), None);
        let pass = RenderPassMetal::new();
        context.setActiveRenderPass(Some(&pass));

        context.finishActiveRenderPass();
        let weak = context.activeRenderPass().expect("active pass token");
        assert!(weak.upgrade().expect("live pass").isFinished());
        context.finishActiveRenderPass();
        assert!(weak.upgrade().expect("live pass").isFinished());

        drop(pass);
        assert!(weak.upgrade().is_none(), "context must not own the pass");
        context.finishActiveRenderPass();
    }

    #[test]
    fn shader_target_values_preserve_the_rstb_wire_format_gap() {
        assert_eq!(ShaderTarget::wgsl as u8, 0);
        assert_eq!(ShaderTarget::msl as u8, 2);
        assert_eq!(ShaderTarget::spirv as u8, 5);
    }
}
